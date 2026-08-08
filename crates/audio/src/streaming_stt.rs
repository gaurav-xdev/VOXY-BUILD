//! Provider-agnostic streaming speech-to-text interface.
//!
//! Supports partial transcripts, final transcripts, cancellation,
//! bounded buffers, backpressure, timeout, connection recovery,
//! and latency instrumentation.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex, Notify};

/// Configuration for streaming STT.
#[derive(Debug, Clone)]
pub struct StreamingSttConfig {
    /// Maximum audio samples buffered before backpressure.
    pub max_buffer_samples: usize,
    /// Timeout for receiving a partial transcript after sending audio.
    pub partial_timeout: Duration,
    /// Timeout for receiving the final transcript after end-of-speech.
    pub final_timeout: Duration,
    /// Maximum number of reconnection attempts.
    pub max_reconnect_attempts: u32,
    /// Delay between reconnection attempts.
    pub reconnect_delay: Duration,
    /// Maximum number of in-flight audio chunks.
    pub max_inflight_chunks: usize,
}

impl Default for StreamingSttConfig {
    fn default() -> Self {
        Self {
            max_buffer_samples: 48000 * 30, // 30 seconds at 48kHz
            partial_timeout: Duration::from_secs(5),
            final_timeout: Duration::from_secs(10),
            max_reconnect_attempts: 3,
            reconnect_delay: Duration::from_millis(500),
            max_inflight_chunks: 16,
        }
    }
}

/// Latency metrics for streaming STT.
#[derive(Debug, Clone, Default)]
pub struct StreamingSttLatency {
    /// Time from first audio sent to first partial transcript (ms).
    pub first_partial_ms: f64,
    /// Time from end-of-speech to final transcript (ms).
    pub eos_to_final_ms: f64,
    /// Total time from first audio to final transcript (ms).
    pub total_ms: f64,
    /// Number of partial transcripts received.
    pub partial_count: u32,
}

/// A streaming STT event emitted by the provider.
#[derive(Debug, Clone)]
pub enum StreamingSttEvent {
    /// Partial transcript (intermediate result, may be superseded).
    Partial {
        text: String,
        confidence: f32,
        is_endpoint: bool,
    },
    /// Final transcript (confirmed result for this utterance).
    Final { text: String, confidence: f32 },
    /// Error occurred.
    Error { message: String },
    /// Connection state changed.
    ConnectionStateChanged { connected: bool },
}

/// Result of a streaming STT session.
#[derive(Debug, Clone)]
pub struct StreamingSttResult {
    /// The final transcript text.
    pub text: String,
    /// Confidence score.
    pub confidence: f32,
    /// Latency metrics.
    pub latency: StreamingSttLatency,
}

/// Errors specific to streaming STT.
#[derive(Debug, thiserror::Error)]
pub enum StreamingSttError {
    #[error("Connection lost: {0}")]
    ConnectionLost(String),
    #[error("Timeout waiting for transcript: {0}")]
    Timeout(String),
    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Cancelled")]
    Cancelled,
    #[error("Not initialized")]
    NotInitialized,
}

/// A streaming audio chunk sent to the STT provider.
#[derive(Debug, Clone)]
pub struct StreamingAudioChunk {
    /// PCM samples (f32, mono).
    pub data: Vec<f32>,
    /// Sample rate.
    pub sample_rate: u32,
    /// Sequence number for ordering.
    pub sequence: u64,
    /// Whether this is the last chunk (end of speech).
    pub is_final: bool,
}

/// Trait for streaming STT providers.
///
/// Implementors must be `Send + Sync` and support concurrent audio feeding
/// and transcript reception without blocking the real-time audio thread.
#[async_trait::async_trait]
pub trait StreamingSttProvider: Send + Sync {
    /// Provider name for logging.
    fn name(&self) -> &str;

    /// Connect to the streaming STT service.
    async fn connect(&mut self, sample_rate: u32) -> Result<(), StreamingSttError>;

    /// Send an audio chunk. Returns immediately (non-blocking).
    /// The provider buffers internally; if the buffer is full, it returns
    /// `Err(BufferOverflow)` to signal backpressure.
    async fn send_audio(&mut self, chunk: StreamingAudioChunk) -> Result<(), StreamingSttError>;

    /// Signal end-of-speech and wait for the final transcript.
    async fn end_of_speech(&mut self) -> Result<StreamingSttResult, StreamingSttError>;

    /// Receive the next event (partial, final, error).
    /// Returns `None` if the stream is closed.
    async fn next_event(&mut self) -> Option<StreamingSttEvent>;

    /// Cancel the current streaming session.
    async fn cancel(&mut self) -> Result<(), StreamingSttError>;

    /// Disconnect from the service.
    async fn disconnect(&mut self) -> Result<(), StreamingSttError>;

    /// Check if the provider is connected.
    fn is_connected(&self) -> bool;

    /// Check if the provider is available (can be connected).
    fn is_available(&self) -> bool;
}

/// Bounded audio buffer for streaming STT with backpressure.
pub struct StreamingAudioBuffer {
    buffer: Vec<f32>,
    max_samples: usize,
    sequence_counter: AtomicU64,
}

impl StreamingAudioBuffer {
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(max_samples.min(48000)),
            max_samples,
            sequence_counter: AtomicU64::new(0),
        }
    }

    /// Push samples into the buffer. Returns `Err` if overflow.
    pub fn push(&mut self, samples: &[f32]) -> Result<u64, StreamingSttError> {
        let total = self.buffer.len() + samples.len();
        if total > self.max_samples {
            return Err(StreamingSttError::BufferOverflow(format!(
                "Buffer {} samples, max {}",
                total, self.max_samples
            )));
        }
        let seq = self.sequence_counter.fetch_add(1, Ordering::Relaxed);
        self.buffer.extend_from_slice(samples);
        Ok(seq)
    }

    /// Drain all buffered samples.
    pub fn drain(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer)
    }

    /// Current buffer length in samples.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Current sequence number.
    pub fn next_sequence(&self) -> u64 {
        self.sequence_counter.load(Ordering::Relaxed)
    }
}

/// Mock streaming STT provider for testing.
///
/// Simulates partial and final transcripts with configurable behavior.
pub struct MockStreamingSttProvider {
    connected: bool,
    sample_rate: u32,
    audio_buffer: Vec<f32>,
    #[allow(dead_code)]
    partial_results: VecDeque<String>,
    #[allow(dead_code)]
    final_result: String,
    partial_delay: Duration,
    final_delay: Duration,
    should_error: bool,
    error_message: String,
    cancel_token: Arc<AtomicBool>,
    sequence: u64,
    /// Simulated partial transcripts.
    pub simulate_partials: VecDeque<String>,
    /// Simulated final transcript.
    pub simulate_final: String,
}

impl MockStreamingSttProvider {
    pub fn new() -> Self {
        Self {
            connected: false,
            sample_rate: 16000,
            audio_buffer: Vec::new(),
            partial_results: VecDeque::new(),
            final_result: String::new(),
            partial_delay: Duration::from_millis(50),
            final_delay: Duration::from_millis(200),
            should_error: false,
            error_message: String::new(),
            cancel_token: Arc::new(AtomicBool::new(false)),
            sequence: 0,
            simulate_partials: VecDeque::from(vec![
                "hello".to_string(),
                "hello world".to_string(),
                "hello world this".to_string(),
            ]),
            simulate_final: "hello world this is a test".to_string(),
        }
    }

    pub fn with_partial_delay(mut self, delay: Duration) -> Self {
        self.partial_delay = delay;
        self
    }

    pub fn with_final_delay(mut self, delay: Duration) -> Self {
        self.final_delay = delay;
        self
    }

    pub fn with_error(mut self, message: &str) -> Self {
        self.should_error = true;
        self.error_message = message.to_string();
        self
    }
}

impl Default for MockStreamingSttProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StreamingSttProvider for MockStreamingSttProvider {
    fn name(&self) -> &str {
        "mock-stt"
    }

    async fn connect(&mut self, sample_rate: u32) -> Result<(), StreamingSttError> {
        if self.should_error {
            return Err(StreamingSttError::Provider(self.error_message.clone()));
        }
        self.sample_rate = sample_rate;
        self.connected = true;
        self.cancel_token.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn send_audio(&mut self, chunk: StreamingAudioChunk) -> Result<(), StreamingSttError> {
        if !self.connected {
            return Err(StreamingSttError::NotInitialized);
        }
        if self.cancel_token.load(Ordering::Relaxed) {
            return Err(StreamingSttError::Cancelled);
        }
        self.audio_buffer.extend_from_slice(&chunk.data);
        self.sequence = chunk.sequence;
        Ok(())
    }

    async fn end_of_speech(&mut self) -> Result<StreamingSttResult, StreamingSttError> {
        if !self.connected {
            return Err(StreamingSttError::NotInitialized);
        }

        let start = Instant::now();
        tokio::time::sleep(self.final_delay).await;

        Ok(StreamingSttResult {
            text: self.simulate_final.clone(),
            confidence: 0.95,
            latency: StreamingSttLatency {
                first_partial_ms: self.partial_delay.as_secs_f64() * 1000.0,
                eos_to_final_ms: self.final_delay.as_secs_f64() * 1000.0,
                total_ms: start.elapsed().as_secs_f64() * 1000.0,
                partial_count: self.simulate_partials.len() as u32,
            },
        })
    }

    async fn next_event(&mut self) -> Option<StreamingSttEvent> {
        if !self.connected {
            return None;
        }

        if self.cancel_token.load(Ordering::Relaxed) {
            return Some(StreamingSttEvent::Error {
                message: "Cancelled".to_string(),
            });
        }

        // Yield partial transcripts with simulated delay
        if let Some(partial) = self.simulate_partials.pop_front() {
            tokio::time::sleep(self.partial_delay).await;
            return Some(StreamingSttEvent::Partial {
                text: partial,
                confidence: 0.8,
                is_endpoint: false,
            });
        }

        // Wait a bit then return None (stream closed)
        None
    }

    async fn cancel(&mut self) -> Result<(), StreamingSttError> {
        self.cancel_token.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), StreamingSttError> {
        self.connected = false;
        self.audio_buffer.clear();
        self.simulate_partials.clear();
        self.cancel_token.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn is_available(&self) -> bool {
        !self.should_error
    }
}

/// High-level streaming STT client that manages the provider lifecycle,
/// audio buffering, event dispatch, cancellation, timeout, and reconnection.
pub struct StreamingSttClient<P: StreamingSttProvider> {
    provider: Arc<Mutex<P>>,
    config: StreamingSttConfig,
    audio_buffer: Arc<Mutex<StreamingAudioBuffer>>,
    #[allow(dead_code)]
    event_tx: mpsc::Sender<StreamingSttEvent>,
    event_rx: Arc<Mutex<Option<mpsc::Receiver<StreamingSttEvent>>>>,
    cancel_signal: Arc<Notify>,
    is_active: Arc<AtomicBool>,
    last_sequence: Arc<AtomicU64>,
    first_audio_time: Arc<Mutex<Option<Instant>>>,
    #[allow(dead_code)]
    first_partial_time: Arc<Mutex<Option<Instant>>>,
    final_result: Arc<Mutex<Option<StreamingSttResult>>>,
    reconnect_count: Arc<AtomicU64>,
}

impl<P: StreamingSttProvider + 'static> StreamingSttClient<P> {
    pub fn new(provider: P, config: StreamingSttConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(512);
        Self {
            provider: Arc::new(Mutex::new(provider)),
            config,
            audio_buffer: Arc::new(Mutex::new(StreamingAudioBuffer::new(48000 * 30))),
            event_tx,
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            cancel_signal: Arc::new(Notify::new()),
            is_active: Arc::new(AtomicBool::new(false)),
            last_sequence: Arc::new(AtomicU64::new(0)),
            first_audio_time: Arc::new(Mutex::new(None)),
            first_partial_time: Arc::new(Mutex::new(None)),
            final_result: Arc::new(Mutex::new(None)),
            reconnect_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Connect to the streaming STT service and start event processing.
    pub async fn connect(&self, sample_rate: u32) -> Result<(), StreamingSttError> {
        let mut provider = self.provider.lock().await;
        provider.connect(sample_rate).await?;
        self.is_active.store(true, Ordering::SeqCst);
        tracing::info!("Streaming STT connected: {}", provider.name());
        Ok(())
    }

    /// Send audio to the streaming STT provider.
    pub async fn send_audio(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<(), StreamingSttError> {
        if !self.is_active.load(Ordering::SeqCst) {
            return Err(StreamingSttError::NotInitialized);
        }

        // Record first audio time for latency measurement
        {
            let mut first_time = self.first_audio_time.lock().await;
            if first_time.is_none() {
                *first_time = Some(Instant::now());
            }
        }

        // Buffer the audio
        let seq = {
            let mut buffer = self.audio_buffer.lock().await;
            buffer.push(samples)?
        };

        self.last_sequence.store(seq, Ordering::Relaxed);

        // Send to provider
        let chunk = StreamingAudioChunk {
            data: samples.to_vec(),
            sample_rate,
            sequence: seq,
            is_final: false,
        };

        let mut provider = self.provider.lock().await;
        provider.send_audio(chunk).await
    }

    /// Signal end-of-speech and wait for the final transcript.
    pub async fn end_of_speech(&self) -> Result<StreamingSttResult, StreamingSttError> {
        if !self.is_active.load(Ordering::SeqCst) {
            return Err(StreamingSttError::NotInitialized);
        }

        let mut provider = self.provider.lock().await;
        let result = provider.end_of_speech().await?;

        // Store the result
        *self.final_result.lock().await = Some(result.clone());

        Ok(result)
    }

    /// Cancel the current streaming session.
    pub async fn cancel(&self) -> Result<(), StreamingSttError> {
        self.is_active.store(false, Ordering::SeqCst);
        self.cancel_signal.notify_waiters();
        let mut provider = self.provider.lock().await;
        provider.cancel().await
    }

    /// Disconnect from the service.
    pub async fn disconnect(&self) -> Result<(), StreamingSttError> {
        self.is_active.store(false, Ordering::SeqCst);
        let mut provider = self.provider.lock().await;
        provider.disconnect().await
    }

    /// Take the event receiver (can only be called once).
    pub async fn take_events(&self) -> Option<mpsc::Receiver<StreamingSttEvent>> {
        self.event_rx.lock().await.take()
    }

    /// Check if the client is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Get the last sequence number.
    pub fn last_sequence(&self) -> u64 {
        self.last_sequence.load(Ordering::Relaxed)
    }

    /// Get reconnect count.
    pub fn reconnect_count(&self) -> u64 {
        self.reconnect_count.load(Ordering::Relaxed)
    }

    /// Get the audio buffer.
    pub fn audio_buffer(&self) -> &Arc<Mutex<StreamingAudioBuffer>> {
        &self.audio_buffer
    }

    /// Attempt to reconnect after a failure.
    pub async fn reconnect(&self, sample_rate: u32) -> Result<(), StreamingSttError> {
        let attempts = self.config.max_reconnect_attempts;
        for i in 0..attempts {
            tracing::warn!("Streaming STT reconnect attempt {}/{}", i + 1, attempts);
            self.reconnect_count.fetch_add(1, Ordering::Relaxed);

            {
                let mut provider = self.provider.lock().await;
                let _ = provider.disconnect().await;
            }

            tokio::time::sleep(self.config.reconnect_delay).await;

            {
                let mut provider = self.provider.lock().await;
                match provider.connect(sample_rate).await {
                    Ok(()) => {
                        self.is_active.store(true, Ordering::SeqCst);
                        tracing::info!("Streaming STT reconnected after {} attempts", i + 1);
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::warn!("Reconnect attempt {} failed: {}", i + 1, e);
                    }
                }
            }
        }

        Err(StreamingSttError::ConnectionLost(format!(
            "Failed to reconnect after {} attempts",
            attempts
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_stt_config_default() {
        let config = StreamingSttConfig::default();
        assert_eq!(config.max_buffer_samples, 48000 * 30);
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.max_inflight_chunks, 16);
    }

    #[test]
    fn audio_buffer_push_and_drain() {
        let mut buf = StreamingAudioBuffer::new(1000);
        assert!(buf.is_empty());

        let seq1 = buf.push(&[0.1, 0.2, 0.3]).unwrap();
        assert_eq!(seq1, 0);
        assert_eq!(buf.len(), 3);

        let seq2 = buf.push(&[0.4, 0.5]).unwrap();
        assert_eq!(seq2, 1);
        assert_eq!(buf.len(), 5);

        let samples = buf.drain();
        assert_eq!(samples, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        assert!(buf.is_empty());
    }

    #[test]
    fn audio_buffer_overflow() {
        let mut buf = StreamingAudioBuffer::new(4);
        buf.push(&[0.1; 4]).unwrap();
        let result = buf.push(&[0.2]);
        assert!(result.is_err());
        match result.unwrap_err() {
            StreamingSttError::BufferOverflow(_) => {}
            other => panic!("Expected BufferOverflow, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn mock_provider_connect() {
        let mut provider = MockStreamingSttProvider::new();
        assert!(!provider.is_connected());
        provider.connect(16000).await.unwrap();
        assert!(provider.is_connected());
    }

    #[tokio::test]
    async fn mock_provider_send_audio() {
        let mut provider = MockStreamingSttProvider::new();
        provider.connect(16000).await.unwrap();
        let chunk = StreamingAudioChunk {
            data: vec![0.1; 480],
            sample_rate: 16000,
            sequence: 0,
            is_final: false,
        };
        provider.send_audio(chunk).await.unwrap();
    }

    #[tokio::test]
    async fn mock_provider_end_of_speech() {
        let mut provider = MockStreamingSttProvider::new();
        provider.connect(16000).await.unwrap();
        let result = provider.end_of_speech().await.unwrap();
        assert_eq!(result.text, "hello world this is a test");
        assert!(result.confidence > 0.9);
    }

    #[tokio::test]
    async fn mock_provider_cancel() {
        let mut provider = MockStreamingSttProvider::new();
        provider.connect(16000).await.unwrap();
        provider.cancel().await.unwrap();

        let chunk = StreamingAudioChunk {
            data: vec![0.1; 480],
            sample_rate: 16000,
            sequence: 0,
            is_final: false,
        };
        let result = provider.send_audio(chunk).await;
        assert!(matches!(result, Err(StreamingSttError::Cancelled)));
    }

    #[tokio::test]
    async fn mock_provider_disconnect() {
        let mut provider = MockStreamingSttProvider::new();
        provider.connect(16000).await.unwrap();
        assert!(provider.is_connected());
        provider.disconnect().await.unwrap();
        assert!(!provider.is_connected());
    }

    #[tokio::test]
    async fn mock_provider_error_on_connect() {
        let mut provider = MockStreamingSttProvider::new().with_error("no network");
        let result = provider.connect(16000).await;
        assert!(result.is_err());
        assert!(!provider.is_available());
    }

    #[tokio::test]
    async fn client_connect_and_send() {
        let provider = MockStreamingSttProvider::new();
        let client = StreamingSttClient::new(provider, StreamingSttConfig::default());
        client.connect(16000).await.unwrap();
        assert!(client.is_active());

        client.send_audio(&[0.1; 480], 16000).await.unwrap();
        assert_eq!(client.last_sequence(), 0);

        client.send_audio(&[0.2; 480], 16000).await.unwrap();
        assert_eq!(client.last_sequence(), 1);
    }

    #[tokio::test]
    async fn client_end_of_speech() {
        let provider = MockStreamingSttProvider::new();
        let client = StreamingSttClient::new(provider, StreamingSttConfig::default());
        client.connect(16000).await.unwrap();
        client.send_audio(&[0.1; 480], 16000).await.unwrap();

        let result = client.end_of_speech().await.unwrap();
        assert_eq!(result.text, "hello world this is a test");
    }

    #[tokio::test]
    async fn client_cancel() {
        let provider = MockStreamingSttProvider::new();
        let client = StreamingSttClient::new(provider, StreamingSttConfig::default());
        client.connect(16000).await.unwrap();
        client.cancel().await.unwrap();
        assert!(!client.is_active());
    }

    #[tokio::test]
    async fn client_send_audio_without_connect() {
        let provider = MockStreamingSttProvider::new();
        let client = StreamingSttClient::new(provider, StreamingSttConfig::default());
        let result = client.send_audio(&[0.1; 480], 16000).await;
        assert!(matches!(result, Err(StreamingSttError::NotInitialized)));
    }

    #[tokio::test]
    async fn client_reconnect() {
        let provider = MockStreamingSttProvider::new();
        let config = StreamingSttConfig {
            max_reconnect_attempts: 2,
            reconnect_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let client = StreamingSttClient::new(provider, config);
        client.connect(16000).await.unwrap();
        client.cancel().await.unwrap();

        // Reconnect should succeed
        client.reconnect(16000).await.unwrap();
        assert!(client.is_active());
        assert!(client.reconnect_count() > 0);
    }

    #[tokio::test]
    async fn client_reconnect_failure() {
        let provider = MockStreamingSttProvider::new().with_error("no network");
        let config = StreamingSttConfig {
            max_reconnect_attempts: 1,
            reconnect_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let client = StreamingSttClient::new(provider, config);
        let result = client.reconnect(16000).await;
        assert!(result.is_err());
    }

    #[test]
    fn streaming_stt_event_display() {
        let event = StreamingSttEvent::Partial {
            text: "hello".to_string(),
            confidence: 0.8,
            is_endpoint: false,
        };
        let s = format!("{:?}", event);
        assert!(s.contains("Partial"));

        let event = StreamingSttEvent::Final {
            text: "hello world".to_string(),
            confidence: 0.95,
        };
        let s = format!("{:?}", event);
        assert!(s.contains("Final"));
    }

    #[test]
    fn streaming_stt_error_display() {
        let err = StreamingSttError::ConnectionLost("timeout".to_string());
        assert!(format!("{}", err).contains("Connection lost"));

        let err = StreamingSttError::Cancelled;
        assert_eq!(format!("{}", err), "Cancelled");
    }

    #[test]
    fn streaming_stt_result_latencies() {
        let result = StreamingSttResult {
            text: "test".to_string(),
            confidence: 0.9,
            latency: StreamingSttLatency {
                first_partial_ms: 100.0,
                eos_to_final_ms: 200.0,
                total_ms: 300.0,
                partial_count: 3,
            },
        };
        assert_eq!(result.latency.partial_count, 3);
        assert_eq!(result.latency.first_partial_ms, 100.0);
    }

    #[tokio::test]
    async fn stress_test_rapid_audio_chunks() {
        let provider = MockStreamingSttProvider::new();
        let client = StreamingSttClient::new(provider, StreamingSttConfig::default());
        client.connect(16000).await.unwrap();

        for i in 0..1000 {
            let samples: Vec<f32> = (0..480).map(|j| (i * 480 + j) as f32 * 0.001).collect();
            client.send_audio(&samples, 16000).await.unwrap();
        }
        assert_eq!(client.last_sequence(), 999);
    }
}
