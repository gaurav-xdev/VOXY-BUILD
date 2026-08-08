use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use tokio::sync::{mpsc, Notify, RwLock};

use crate::config::VoiceConfig;
use crate::error::{Result, VoiceError};
use crate::VoiceEvent;

type EventHandler = Arc<RwLock<Option<Box<dyn Fn(VoiceEvent) + Send + Sync>>>>;
type ResponseFn = Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync>;
type ResponseHandler = Arc<RwLock<Option<ResponseFn>>>;
type WakeAliases = Arc<RwLock<Vec<String>>>;

const PREFERRED_FRAME_SIZE: usize = 480;
const SPEECH_BUF_INITIAL: usize = 48000;
const MAX_SPEECH_SECONDS: usize = 30;
/// Maximum samples stored in the TTS reference buffer for AEC (1 second at 48kHz).
const AEC_REFERENCE_BUF_SAMPLES: usize = 48000;
/// TTS fade-in duration in milliseconds.
const TTS_FADE_IN_MS: u32 = 50;
/// Barge-in crossfade duration in milliseconds.
const BARGE_IN_CROSSFADE_MS: u32 = 80;

/// Barge-in configuration.
#[derive(Debug, Clone)]
pub struct InterruptionPolicy {
    /// Whether barge-in is enabled.
    pub enabled: bool,
    /// Minimum voice frames before interrupting TTS.
    pub min_voice_frames: usize,
    /// Fade-out duration in milliseconds.
    pub fade_out_ms: u32,
    /// Whether to preserve conversation context on interruption.
    pub preserve_context: bool,
}

impl Default for InterruptionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            min_voice_frames: 2,
            fade_out_ms: 80,
            preserve_context: true,
        }
    }
}

/// Streaming metrics for pipeline stages.
#[derive(Debug, Clone, Default)]
pub struct StreamingMetrics {
    pub stt_latency_ms: f64,
    pub llm_first_token_ms: f64,
    pub tts_first_chunk_ms: f64,
    pub end_to_end_ms: f64,
    pub interruption_count: u64,
    pub barge_in_latency_ms: f64,
}

pub struct VoicePipeline {
    config: VoiceConfig,
    audio_mgr: Arc<Box<dyn voxy_audio::AudioDeviceManager>>,
    audio_input: Arc<RwLock<Option<Box<dyn voxy_audio::AudioInputStream>>>>,
    audio_output: Arc<RwLock<Option<Box<dyn voxy_audio::AudioOutputStream>>>>,
    session_mgr: Arc<Box<dyn voxy_conversation::SessionManager>>,
    session: Arc<RwLock<Option<Box<dyn voxy_conversation::ConversationSession>>>>,
    diagnostics: Arc<Box<dyn voxy_audio::AudioDiagnostics>>,
    wake_word_detector: Arc<RwLock<Option<Box<dyn voxy_voice_orchestrator::WakeWordDetector>>>>,
    vad_detector: Arc<RwLock<Option<Box<dyn voxy_voice_orchestrator::VadDetector>>>>,
    stt_engine: Arc<RwLock<Option<Box<dyn voxy_voice_orchestrator::SttEngine>>>>,
    tts_engine: Arc<RwLock<Option<Box<dyn voxy_voice_orchestrator::TtsEngine>>>>,
    event_handler: EventHandler,
    response_handler: ResponseHandler,
    listening_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    tts_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    tts_stop: Arc<Notify>,
    stop_signal: Arc<Notify>,
    is_running: Arc<AtomicBool>,
    is_initialized: Arc<AtomicBool>,
    is_sleeping: Arc<AtomicBool>,
    is_speaking: Arc<AtomicBool>,
    wake_aliases: WakeAliases,
    last_activity: Arc<RwLock<Instant>>,
    consecutive_silence_s: Arc<AtomicU64>,
    interruption_policy: Arc<RwLock<InterruptionPolicy>>,
    metrics: Arc<RwLock<StreamingMetrics>>,
    /// Channel for streaming partial transcripts to the LLM.
    #[allow(dead_code)]
    partial_tx: Arc<RwLock<Option<mpsc::Sender<String>>>>,
    // ── Voice Engine V2 modules ──────────────────────────────────────
    v2_mixer: Option<Arc<voxy_audio::AudioMixer>>,
    v2_calibrator: Option<Arc<voxy_audio::SelfCalibrator>>,
    v2_watchdog: Option<Arc<voxy_audio::HealthWatchdog>>,
    v2_metrics_collector: Option<Arc<voxy_audio::MetricsCollector>>,
    v2_voice_memory: Option<Arc<voxy_audio::VoiceMemory>>,
    v2_noise_suppressor: Option<Arc<parking_lot::Mutex<voxy_audio::AdaptiveNoiseSuppressor>>>,
    v2_echo_canceller: Option<Arc<parking_lot::Mutex<voxy_audio::SpectralEchoCanceller>>>,
    v2_watchdog_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Circular buffer holding recent TTS output samples, used as AEC reference.
    v2_tts_reference: Arc<parking_lot::Mutex<Vec<f32>>>,
    /// Write position in the TTS reference circular buffer.
    v2_tts_ref_pos: Arc<std::sync::atomic::AtomicUsize>,
    /// Hot-swap manager for audio device recovery.
    hot_swap: Arc<voxy_audio::HotSwapManager>,
    /// Voice speed multiplier (0.5–2.0) applied to TTS output.
    voice_speed: Arc<RwLock<f64>>,
}

impl VoicePipeline {
    pub fn new(config: VoiceConfig) -> Self {
        let session_mgr =
            voxy_conversation::InMemorySessionManager::new(config.conversation.clone());
        let sr = config.audio.input.sample_rate;
        Self {
            config,
            audio_mgr: Arc::new(Box::new(voxy_audio::InMemoryDeviceManager::default())),
            audio_input: Arc::new(RwLock::new(None)),
            audio_output: Arc::new(RwLock::new(None)),
            session_mgr: Arc::new(Box::new(session_mgr)),
            session: Arc::new(RwLock::new(None)),
            diagnostics: Arc::new(Box::new(voxy_audio::InMemoryDiagnostics::new())),
            wake_word_detector: Arc::new(RwLock::new(None)),
            vad_detector: Arc::new(RwLock::new(None)),
            stt_engine: Arc::new(RwLock::new(None)),
            tts_engine: Arc::new(RwLock::new(None)),
            event_handler: Arc::new(RwLock::new(None)),
            response_handler: Arc::new(RwLock::new(None)),
            listening_task: Arc::new(RwLock::new(None)),
            tts_task: Arc::new(RwLock::new(None)),
            tts_stop: Arc::new(Notify::new()),
            stop_signal: Arc::new(Notify::new()),
            is_running: Arc::new(AtomicBool::new(false)),
            is_initialized: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(false)),
            is_speaking: Arc::new(AtomicBool::new(false)),
            wake_aliases: Arc::new(RwLock::new(Vec::new())),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            consecutive_silence_s: Arc::new(AtomicU64::new(0)),
            interruption_policy: Arc::new(RwLock::new(InterruptionPolicy::default())),
            metrics: Arc::new(RwLock::new(StreamingMetrics::default())),
            partial_tx: Arc::new(RwLock::new(None)),
            // ── V2 modules ──
            v2_mixer: None,
            v2_calibrator: Some(Arc::new(voxy_audio::SelfCalibrator::new())),
            v2_watchdog: Some(Arc::new(voxy_audio::HealthWatchdog::new())),
            v2_metrics_collector: Some(Arc::new(voxy_audio::MetricsCollector::new())),
            v2_voice_memory: Some(Arc::new(voxy_audio::VoiceMemory::new())),
            v2_noise_suppressor: Some(Arc::new(parking_lot::Mutex::new(
                voxy_audio::AdaptiveNoiseSuppressor::new(sr),
            ))),
            v2_echo_canceller: Some(Arc::new(parking_lot::Mutex::new(
                voxy_audio::SpectralEchoCanceller::new(sr, 200),
            ))),
            v2_watchdog_task: Arc::new(RwLock::new(None)),
            v2_tts_reference: Arc::new(parking_lot::Mutex::new(vec![
                0.0;
                AEC_REFERENCE_BUF_SAMPLES
            ])),
            v2_tts_ref_pos: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            hot_swap: Arc::new(voxy_audio::HotSwapManager::new()),
            voice_speed: Arc::new(RwLock::new(1.0)),
        }
    }

    pub fn with_audio_mgr(
        config: VoiceConfig,
        audio_mgr: Box<dyn voxy_audio::AudioDeviceManager>,
    ) -> Self {
        let session_mgr =
            voxy_conversation::InMemorySessionManager::new(config.conversation.clone());
        let sr = config.audio.input.sample_rate;
        Self {
            config,
            audio_mgr: Arc::new(audio_mgr),
            audio_input: Arc::new(RwLock::new(None)),
            audio_output: Arc::new(RwLock::new(None)),
            session_mgr: Arc::new(Box::new(session_mgr)),
            session: Arc::new(RwLock::new(None)),
            diagnostics: Arc::new(Box::new(voxy_audio::InMemoryDiagnostics::new())),
            wake_word_detector: Arc::new(RwLock::new(None)),
            vad_detector: Arc::new(RwLock::new(None)),
            stt_engine: Arc::new(RwLock::new(None)),
            tts_engine: Arc::new(RwLock::new(None)),
            event_handler: Arc::new(RwLock::new(None)),
            response_handler: Arc::new(RwLock::new(None)),
            listening_task: Arc::new(RwLock::new(None)),
            tts_task: Arc::new(RwLock::new(None)),
            tts_stop: Arc::new(Notify::new()),
            stop_signal: Arc::new(Notify::new()),
            is_running: Arc::new(AtomicBool::new(false)),
            is_initialized: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(false)),
            is_speaking: Arc::new(AtomicBool::new(false)),
            wake_aliases: Arc::new(RwLock::new(Vec::new())),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            consecutive_silence_s: Arc::new(AtomicU64::new(0)),
            interruption_policy: Arc::new(RwLock::new(InterruptionPolicy::default())),
            metrics: Arc::new(RwLock::new(StreamingMetrics::default())),
            partial_tx: Arc::new(RwLock::new(None)),
            // ── V2 modules ──
            v2_mixer: None,
            v2_calibrator: Some(Arc::new(voxy_audio::SelfCalibrator::new())),
            v2_watchdog: Some(Arc::new(voxy_audio::HealthWatchdog::new())),
            v2_metrics_collector: Some(Arc::new(voxy_audio::MetricsCollector::new())),
            v2_voice_memory: Some(Arc::new(voxy_audio::VoiceMemory::new())),
            v2_noise_suppressor: Some(Arc::new(parking_lot::Mutex::new(
                voxy_audio::AdaptiveNoiseSuppressor::new(sr),
            ))),
            v2_echo_canceller: Some(Arc::new(parking_lot::Mutex::new(
                voxy_audio::SpectralEchoCanceller::new(sr, 200),
            ))),
            v2_watchdog_task: Arc::new(RwLock::new(None)),
            v2_tts_reference: Arc::new(parking_lot::Mutex::new(vec![
                0.0;
                AEC_REFERENCE_BUF_SAMPLES
            ])),
            v2_tts_ref_pos: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            hot_swap: Arc::new(voxy_audio::HotSwapManager::new()),
            voice_speed: Arc::new(RwLock::new(1.0)),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        if self.is_initialized.load(Ordering::SeqCst) {
            return Err(VoiceError::AlreadyInitialized);
        }
        let _span = tracing::debug_span!("pipeline_initialize");
        self.audio_mgr
            .initialize(&self.config.audio)
            .await
            .map_err(|e| VoiceError::AudioDeviceError(e.to_string()))?;
        self.is_initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(VoiceError::NotInitialized);
        }
        let _span = tracing::debug_span!("pipeline_shutdown");
        self.interrupt_tts().await;
        self.stop_listening().await;
        self.stop_v2_watchdog().await;
        if self.is_running.load(Ordering::SeqCst) {
            if let Err(e) = self.stop_capture().await {
                tracing::warn!("stop_capture during shutdown: {e}");
            }
        }
        if let Err(e) = self.audio_mgr.shutdown().await {
            tracing::warn!("audio_mgr shutdown: {e}");
        }
        *self.session.write().await = None;
        self.is_initialized.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub async fn start_capture(&self) -> Result<()> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(VoiceError::NotInitialized);
        }
        if self.is_running.load(Ordering::SeqCst) {
            return Err(VoiceError::AlreadyRunning);
        }

        let input = self
            .audio_mgr
            .open_input(&self.config.audio.input)
            .await
            .map_err(|e| VoiceError::CaptureError(e.to_string()))?;
        *self.audio_input.write().await = Some(input);

        let output = self
            .audio_mgr
            .open_output(&self.config.audio.output)
            .await
            .map_err(|e| VoiceError::PlaybackError(e.to_string()))?;
        *self.audio_output.write().await = Some(output);

        if self.config.auto_start_capture {
            match self.session_mgr.create_session().await {
                Ok(mut session) => {
                    if session.start(None, None).await.is_ok() {
                        *self.session.write().await = Some(session);
                    }
                }
                Err(e) => tracing::warn!("session creation: {e}"),
            }
        }

        *self.last_activity.write().await = Instant::now();
        self.consecutive_silence_s.store(0, Ordering::Relaxed);
        self.is_sleeping.store(false, Ordering::Relaxed);
        self.is_running.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub async fn stop_capture(&self) -> Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(VoiceError::NotRunning);
        }

        if let Some(ref mut input) = *self.audio_input.write().await {
            if let Err(e) = input.close().await {
                tracing::warn!("Failed to close audio input: {e}");
            }
        }
        *self.audio_input.write().await = None;

        if let Some(ref mut output) = *self.audio_output.write().await {
            if let Err(e) = output.close().await {
                tracing::warn!("Failed to close audio output: {e}");
            }
        }
        *self.audio_output.write().await = None;

        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub async fn add_wake_alias(&self, alias: &str) {
        self.wake_aliases.write().await.push(alias.to_string());
    }

    pub async fn remove_wake_alias(&self, alias: &str) {
        self.wake_aliases.write().await.retain(|a| a != alias);
    }

    /// Interrupt TTS with a fast fade-out.
    pub async fn interrupt_tts(&self) {
        if !self.is_speaking.load(Ordering::Relaxed) {
            return;
        }

        let policy = self.interruption_policy.read().await;
        let fade_ms = policy.fade_out_ms;
        drop(policy);

        // Signal TTS to stop
        self.is_speaking.store(false, Ordering::Relaxed);
        self.tts_stop.notify_one();

        // Update interruption metrics
        {
            let mut m = self.metrics.write().await;
            m.interruption_count += 1;
        }

        if let Some(task) = self.tts_task.write().await.take() {
            task.abort();
            let _ = tokio::time::timeout(
                tokio::time::Duration::from_millis(fade_ms as u64 + 100),
                task,
            )
            .await;
        }
    }

    /// Fast barge-in: fade TTS in <100ms and resume STT instantly.
    pub async fn barge_in(&self) {
        if !self.is_speaking.load(Ordering::Relaxed) {
            return;
        }

        let barge_start = Instant::now();

        // Signal immediate stop
        self.is_speaking.store(false, Ordering::Relaxed);
        self.tts_stop.notify_one();

        // Abort TTS task
        if let Some(task) = self.tts_task.write().await.take() {
            task.abort();
            let _ = tokio::time::timeout(tokio::time::Duration::from_millis(50), task).await;
        }

        // Write silence to output to achieve instant fade
        if let Some(ref mut output) = *self.audio_output.write().await {
            let silence =
                voxy_audio::AudioPacket::silence(480, self.config.audio.output.sample_rate, 1);
            let _ = output.write(&silence).await;
        }

        let elapsed = barge_start.elapsed().as_millis() as f64;
        {
            let mut m = self.metrics.write().await;
            m.barge_in_latency_ms = elapsed;
            m.interruption_count += 1;
        }

        tracing::info!("Barge-in completed in {:.1}ms", elapsed);
    }

    pub async fn wake(&self) {
        self.is_sleeping.store(false, Ordering::Relaxed);
        *self.last_activity.write().await = Instant::now();
    }

    pub async fn sleep(&self) {
        self.is_sleeping.store(true, Ordering::Relaxed);
        if let Some(ref handler) = *self.event_handler.read().await {
            handler(VoiceEvent::VoiceActivityEnded { duration_ms: 0 });
        }
    }

    pub async fn set_response_handler(&self, handler: ResponseFn) {
        *self.response_handler.write().await = Some(handler);
    }

    /// Set the interruption policy.
    pub async fn set_interruption_policy(&self, policy: InterruptionPolicy) {
        *self.interruption_policy.write().await = policy;
    }

    /// Set the voice speed multiplier (0.5–2.0). Applied to TTS output.
    pub async fn set_voice_speed(&self, speed: f64) {
        let clamped = speed.clamp(0.5, 2.0);
        *self.voice_speed.write().await = clamped;
    }

    /// Get the current voice speed multiplier.
    pub async fn voice_speed(&self) -> f64 {
        *self.voice_speed.read().await
    }

    /// Get current streaming metrics.
    pub async fn streaming_metrics(&self) -> StreamingMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn start_listening(&self) -> Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(VoiceError::NotRunning);
        }
        if self.listening_task.read().await.is_some() {
            return Ok(());
        }

        let audio_input = self.audio_input.clone();
        let audio_output = self.audio_output.clone();
        let config = self.config.clone();
        let wake_word_detector = self.wake_word_detector.clone();
        let vad_detector = self.vad_detector.clone();
        let stt_engine = self.stt_engine.clone();
        let tts_engine = self.tts_engine.clone();
        let event_handler = self.event_handler.clone();
        let response_handler = self.response_handler.clone();
        let diagnostics = self.diagnostics.clone();
        let is_running = self.is_running.clone();
        let is_sleeping = self.is_sleeping.clone();
        let is_speaking = self.is_speaking.clone();
        let session = self.session.clone();
        let stop_signal = self.stop_signal.clone();
        let tts_stop = self.tts_stop.clone();
        let tts_task_handle = self.tts_task.clone();
        let last_activity = self.last_activity.clone();
        let consecutive_silence_s = self.consecutive_silence_s.clone();
        let interruption_policy = self.interruption_policy.clone();
        let streaming_metrics = self.metrics.clone();
        let noise_suppressor = self.v2_noise_suppressor.clone();
        let echo_canceller = self.v2_echo_canceller.clone();
        let watchdog = self.v2_watchdog.clone();
        let _voice_memory = self.v2_voice_memory.clone();
        let metrics_collector = self.v2_metrics_collector.clone();
        let tts_reference = self.v2_tts_reference.clone();
        let tts_ref_pos = self.v2_tts_ref_pos.clone();
        let hot_swap = self.hot_swap.clone();
        let audio_mgr_for_recovery = self.audio_mgr.clone();
        let config_for_recovery = self.config.clone();
        let voice_speed_for_loop = self.voice_speed.clone();

        let frame_size = PREFERRED_FRAME_SIZE;
        let sleep_timeout_s = (config.conversation.auto_sleep_after_ms / 1000).max(30);
        let idle_timeout_s = (config.conversation.idle_timeout_ms / 1000).max(10);
        let max_silence_frames = (config.orchestrator.silence_timeout_ms / 30).max(1) as usize;
        let max_buf = (config.orchestrator.max_duration_seconds as usize)
            .max(MAX_SPEECH_SECONDS)
            .saturating_mul(48000);

        let task = tokio::spawn(async move {
            let _listening_span = tracing::debug_span!("listening_loop");
            let mut speech_buffer: Vec<f32> = Vec::with_capacity(SPEECH_BUF_INITIAL);
            let mut reuse_chunk = voxy_voice_orchestrator::AudioChunk {
                data: Vec::with_capacity(frame_size),
                sample_rate: 0,
                channels: 0,
                timestamp: Utc::now(),
                sequence: 0,
                is_final: false,
            };
            let mut is_awake = false;
            let mut silence_frames_after_speech = 0usize;
            let mut voice_frames_during_tts = 0usize;

            loop {
                tokio::select! {
                    _ = stop_signal.notified() => {
                        tracing::info!("Continuous listening stopped.");
                        break;
                    }
                    _ = async {} => {}
                }

                if !is_running.load(Ordering::SeqCst) {
                    break;
                }

                let packet = {
                    let mut input_guard = audio_input.write().await;
                    match input_guard.as_mut() {
                        Some(input) => match input.read(frame_size).await {
                            Ok(p) => p,
                            Err(e) => {
                                let err_msg = e.to_string();
                                tracing::warn!(
                                    "Audio read error: {err_msg} — initiating device recovery"
                                );
                                drop(input_guard);

                                // Attempt hot-swap recovery with exponential backoff
                                let recovered = {
                                    let max_attempts = 3u32;
                                    let mut success = false;
                                    for attempt in 0..max_attempts {
                                        tokio::time::sleep(Duration::from_millis(
                                            500 * 2u64.pow(attempt),
                                        ))
                                        .await;
                                        tracing::info!(
                                            "Device recovery attempt {}/{}",
                                            attempt + 1,
                                            max_attempts
                                        );

                                        // Close existing streams
                                        {
                                            let mut in_guard = audio_input.write().await;
                                            if let Some(ref mut inp) = *in_guard {
                                                let _ = inp.close().await;
                                            }
                                            *in_guard = None;
                                        }
                                        {
                                            let mut out_guard = audio_output.write().await;
                                            if let Some(ref mut out) = *out_guard {
                                                let _ = out.close().await;
                                            }
                                            *out_guard = None;
                                        }

                                        // Try to reopen
                                        match audio_mgr_for_recovery
                                            .open_input(&config_for_recovery.audio.input)
                                            .await
                                        {
                                            Ok(new_input) => {
                                                match audio_mgr_for_recovery
                                                    .open_output(&config_for_recovery.audio.output)
                                                    .await
                                                {
                                                    Ok(new_output) => {
                                                        *audio_input.write().await =
                                                            Some(new_input);
                                                        *audio_output.write().await =
                                                            Some(new_output);
                                                        hot_swap.record_device_change();
                                                        tracing::info!("Device recovery successful on attempt {}", attempt + 1);
                                                        success = true;
                                                        break;
                                                    }
                                                    Err(out_err) => {
                                                        tracing::warn!(
                                                            "Output reopen failed: {out_err}"
                                                        );
                                                    }
                                                }
                                            }
                                            Err(in_err) => {
                                                tracing::warn!("Input reopen failed: {in_err}");
                                            }
                                        }
                                    }
                                    success
                                };

                                if !recovered {
                                    tracing::error!("Device recovery failed after 3 attempts — pausing pipeline");
                                    is_running.store(false, Ordering::SeqCst);
                                    hot_swap.set_state(voxy_audio::PipelineState::Sleeping);
                                }
                                continue;
                            }
                        },
                        None => {
                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            continue;
                        }
                    }
                };

                diagnostics.record_packet_captured(&packet).await;
                if let Some(ref w) = watchdog {
                    w.heartbeat("audio_input");
                }

                // ── V2 DSP: noise suppression + echo cancellation ──────
                let processed_data = {
                    let mut data = packet.data.clone();

                    // Noise suppression
                    if let Some(ref ns) = noise_suppressor {
                        let mut ns = ns.lock();
                        let mut suppressed = Vec::with_capacity(data.len());
                        if ns.process(&data, &mut suppressed).is_ok() {
                            data = suppressed;
                        }
                    }

                    // Echo cancellation with real TTS reference signal
                    if let Some(ref ec) = echo_canceller {
                        let ref_pos = tts_ref_pos.load(Ordering::Relaxed);
                        let reference: Vec<f32> = {
                            let ref_buf = tts_reference.lock();
                            let buf_len = ref_buf.len();
                            if buf_len == 0 {
                                vec![0.0; data.len()]
                            } else {
                                let mut ref_samples = Vec::with_capacity(data.len());
                                for i in 0..data.len() {
                                    let idx = (ref_pos + buf_len - data.len() + i) % buf_len;
                                    ref_samples.push(ref_buf[idx]);
                                }
                                ref_samples
                            }
                        };
                        let mut ec = ec.lock();
                        let mut cancelled = Vec::with_capacity(data.len());
                        if ec.process(&data, &reference, &mut cancelled).is_ok() {
                            data = cancelled;
                        }
                    }

                    data
                };

                reuse_chunk.data.clear();
                reuse_chunk.data.extend_from_slice(&processed_data);
                reuse_chunk.sample_rate = packet.sample_rate;
                reuse_chunk.channels = packet.channels;
                reuse_chunk.timestamp = packet.timestamp;
                reuse_chunk.sequence = packet.sequence;
                reuse_chunk.is_final = false;

                let is_voice = if config.vad_enabled {
                    let vad_guard = vad_detector.read().await;
                    match vad_guard.as_ref() {
                        Some(vad) => vad.is_voice(&reuse_chunk).await.unwrap_or(true),
                        None => true,
                    }
                } else {
                    true
                };

                // ── Barge-in detection ────────────────────────────────
                if is_voice && is_speaking.load(Ordering::Relaxed) {
                    let policy = interruption_policy.read().await;
                    if policy.enabled {
                        voice_frames_during_tts += 1;
                        if voice_frames_during_tts >= policy.min_voice_frames {
                            tracing::info!(
                                "Barge-in: {} voice frames during TTS",
                                voice_frames_during_tts
                            );

                            // Crossfade: write fade-out samples to smooth the cut
                            {
                                let fade_samples = (config.audio.output.sample_rate as u64
                                    * BARGE_IN_CROSSFADE_MS as u64
                                    / 1000)
                                    as usize;
                                let mut ref_buf = tts_reference.lock();
                                let ref_len = ref_buf.len();
                                let pos = tts_ref_pos.load(Ordering::Relaxed);
                                for i in 0..fade_samples.min(ref_len) {
                                    let idx = (pos + i) % ref_len;
                                    let progress = i as f32 / fade_samples.max(1) as f32;
                                    ref_buf[idx] *= 1.0 - progress;
                                }
                            }

                            // Signal TTS to stop
                            is_speaking.store(false, Ordering::Relaxed);
                            tts_stop.notify_one();

                            if let Some(task) = tts_task_handle.write().await.take() {
                                task.abort();
                                let _ = tokio::time::timeout(
                                    tokio::time::Duration::from_millis(50),
                                    task,
                                )
                                .await;
                            }

                            // Write fade-out silence to output for smooth transition
                            if let Some(ref mut output) = *audio_output.write().await {
                                let fade_frames = (config.audio.output.sample_rate as u64
                                    * BARGE_IN_CROSSFADE_MS as u64
                                    / 1000)
                                    as usize;
                                let fade_samples =
                                    fade_frames * config.audio.output.channels as usize;
                                let mut fade_buf: Vec<f32> = Vec::with_capacity(fade_samples);
                                for i in 0..fade_frames {
                                    let progress = i as f32 / fade_frames.max(1) as f32;
                                    let sample = (1.0 - progress) * 0.0;
                                    for _ in 0..config.audio.output.channels {
                                        fade_buf.push(sample);
                                    }
                                }
                                let pkt = voxy_audio::AudioPacket::new(
                                    fade_buf,
                                    config.audio.output.sample_rate,
                                    config.audio.output.channels,
                                );
                                let _ = output.write(&pkt).await;
                            }

                            let mut m = streaming_metrics.write().await;
                            m.interruption_count += 1;
                            m.barge_in_latency_ms = 0.0;

                            voice_frames_during_tts = 0;
                        }
                    }
                } else if !is_voice {
                    voice_frames_during_tts = 0;
                }

                // ── End-of-utterance → STT → LLM → TTS ──────────────
                if !is_voice && is_awake && silence_frames_after_speech >= max_silence_frames {
                    if !speech_buffer.is_empty() {
                        let stt_start = Instant::now();

                        let final_chunk = voxy_voice_orchestrator::AudioChunk {
                            data: std::mem::take(&mut speech_buffer),
                            sample_rate: packet.sample_rate,
                            channels: packet.channels,
                            timestamp: Utc::now(),
                            sequence: 0,
                            is_final: true,
                        };

                        let text = {
                            let stt_guard = stt_engine.read().await;
                            match stt_guard.as_ref() {
                                Some(stt) => match stt.transcribe(&final_chunk).await {
                                    Ok(t) => t,
                                    Err(e) => {
                                        tracing::warn!("STT error: {e}");
                                        if let Some(ref handler) = *event_handler.read().await {
                                            handler(VoiceEvent::TranscriptionError {
                                                error: e.to_string(),
                                            });
                                        }
                                        String::new()
                                    }
                                },
                                None => String::new(),
                            }
                        };

                        let stt_latency = stt_start.elapsed().as_millis() as f64;
                        {
                            let mut m = streaming_metrics.write().await;
                            m.stt_latency_ms = stt_latency;
                        }
                        if let Some(ref w) = watchdog {
                            w.heartbeat("stt");
                        }
                        if let Some(ref mc) = metrics_collector {
                            mc.record_stt_latency(stt_latency);
                        }

                        if !text.is_empty() {
                            if let Some(ref handler) = *event_handler.read().await {
                                handler(VoiceEvent::TranscriptionResult {
                                    text: text.clone(),
                                    is_final: true,
                                    confidence: 0.9,
                                });
                            }

                            if let Some(ref mut sess) = *session.write().await {
                                let _ = sess.process_input(&text, true).await;
                            }

                            let llm_start = Instant::now();
                            let handler = response_handler.read().await;
                            let response = match handler.as_ref() {
                                Some(h) => h(text).await,
                                None => String::new(),
                            };
                            let llm_latency = llm_start.elapsed().as_millis() as f64;

                            if !response.is_empty() {
                                *last_activity.write().await = Instant::now();

                                {
                                    let mut m = streaming_metrics.write().await;
                                    m.llm_first_token_ms = llm_latency;
                                    m.end_to_end_ms = stt_latency + llm_latency;
                                }
                                if let Some(ref w) = watchdog {
                                    w.heartbeat("llm");
                                }
                                if let Some(ref mc) = metrics_collector {
                                    mc.record_llm_latency(llm_latency);
                                }

                                let tts_engine_clone = tts_engine.clone();
                                let audio_output_clone = audio_output.clone();
                                let diagnostics_clone = diagnostics.clone();
                                let event_handler_clone = event_handler.clone();
                                let session_clone = session.clone();
                                let is_speaking_clone = is_speaking.clone();
                                let tts_stop_clone = tts_stop.clone();
                                let tts_handle = tts_task_handle.clone();
                                let metrics_clone = streaming_metrics.clone();
                                let watchdog_tts = watchdog.clone();
                                let mc_tts = metrics_collector.clone();

                                is_speaking_clone.store(true, Ordering::Relaxed);

                                if let Some(old_task) = tts_handle.write().await.take() {
                                    old_task.abort();
                                }

                                let tts_cancelled = tts_stop_clone.clone();
                                let tts_ref_for_write = tts_reference.clone();
                                let tts_ref_pos_write = tts_ref_pos.clone();
                                let voice_speed_clone = voice_speed_for_loop.clone();
                                let tts_task = tokio::spawn(async move {
                                    let tts_start = Instant::now();
                                    let mut total_written = 0usize;
                                    let speed = *voice_speed_clone.read().await;
                                    tokio::select! {
                                        _ = tts_cancelled.notified() => {
                                            tracing::info!("TTS interrupted");
                                        }
                                        _ = async {
                                            if let Some(ref handler) = *event_handler_clone.read().await {
                                                handler(VoiceEvent::SynthesisStarted { text: response.clone() });
                                            }
                                            let tts_guard = tts_engine_clone.read().await;
                                            if let Some(ref tts) = *tts_guard {
                                                if let Ok(mut stream) = tts.synthesize_stream(&response).await {
                                                    let mut first_chunk = true;
                                                    let mut fade_samples_remaining =
                                                        (48000u32 * TTS_FADE_IN_MS / 1000) as usize;
                                                    while let Some(chunk) = stream.next_chunk().await {
                                                        if !is_speaking_clone.load(Ordering::Relaxed) { break; }

                                                        if first_chunk {
                                                            let tts_first = tts_start.elapsed().as_millis() as f64;
                                                            let mut m = metrics_clone.write().await;
                                                            m.tts_first_chunk_ms = tts_first;
                                                            first_chunk = false;
                                                            if let Some(ref w) = watchdog_tts { w.heartbeat("tts"); }
                                                            if let Some(ref mc) = mc_tts { mc.record_tts_latency(tts_first); }
                                                        }

                                                        // Apply TTS fade-in on first chunk
                                                        let mut faded_data = chunk.data;
                                                        if fade_samples_remaining > 0 {
                                                            for sample in faded_data.iter_mut() {
                                                                if fade_samples_remaining == 0 { break; }
                                                                let progress = 1.0 - (fade_samples_remaining as f32 / (48000u32 * TTS_FADE_IN_MS / 1000) as f32);
                                                                let gain = progress.max(0.0);
                                                                *sample *= gain;
                                                                fade_samples_remaining = fade_samples_remaining.saturating_sub(1);
                                                            }
                                                        }

                                                        // Apply voice speed
                                                        if (speed - 1.0).abs() > 0.01 {
                                                            faded_data = VoicePipeline::resample_speed(&faded_data, speed);
                                                        }

                                                        // Write TTS output into AEC reference buffer
                                                        {
                                                            let mut ref_buf = tts_ref_for_write.lock();
                                                            let buf_len = ref_buf.len();
                                                            for &s in &faded_data {
                                                                let pos = tts_ref_pos_write.load(Ordering::Relaxed);
                                                                ref_buf[pos % buf_len] = s;
                                                                tts_ref_pos_write.store((pos + 1) % buf_len, Ordering::Relaxed);
                                                            }
                                                            total_written += faded_data.len();
                                                        }

                                                        let pkt = voxy_audio::AudioPacket::new(
                                                            faded_data, chunk.sample_rate, chunk.channels,
                                                        );
                                                        if let Some(ref mut output) = *audio_output_clone.write().await {
                                                            let _ = output.write(&pkt).await;
                                                        }
                                                        diagnostics_clone.record_packet_played(&pkt).await;
                                                        if let Some(ref w) = watchdog_tts { w.heartbeat("audio_output"); }
                                                    }
                                                    if let Some(ref mut sess) = *session_clone.write().await {
                                                        let _ = sess.generate_output(&response).await;
                                                    }
                                                } else {
                                                    tracing::warn!("TTS synthesis_stream failed");
                                                    if let Some(ref handler) = *event_handler_clone.read().await {
                                                        handler(VoiceEvent::SynthesisError { error: "synthesis_stream failed".to_string() });
                                                    }
                                                }
                                            }
                                            if let Some(ref handler) = *event_handler_clone.read().await {
                                                handler(VoiceEvent::SynthesisCompleted { duration_ms: 0 });
                                            }
                                        } => {}
                                    }
                                    is_speaking_clone.store(false, Ordering::Relaxed);
                                });

                                *tts_handle.write().await = Some(tts_task);
                            }
                        }
                    }
                    is_awake = false;
                    silence_frames_after_speech = 0;
                    speech_buffer.clear();
                }

                if !is_voice && !is_awake {
                    let sil_s = consecutive_silence_s.fetch_add(1, Ordering::Relaxed);
                    if !is_sleeping.load(Ordering::Relaxed) && sil_s >= sleep_timeout_s * 1000 / 30
                    {
                        is_sleeping.store(true, Ordering::Relaxed);
                        if let Some(ref handler) = *event_handler.read().await {
                            handler(VoiceEvent::VoiceActivityEnded { duration_ms: 0 });
                        }
                    }
                }

                if is_voice {
                    silence_frames_after_speech = 0;
                    consecutive_silence_s.store(0, Ordering::Relaxed);
                    *last_activity.write().await = Instant::now();

                    if is_sleeping.load(Ordering::Relaxed) {
                        is_sleeping.store(false, Ordering::Relaxed);
                        if let Some(ref handler) = *event_handler.read().await {
                            handler(VoiceEvent::WakeWordDetected { confidence: 0.5 });
                        }
                    }

                    if !is_awake {
                        if config.wake_word_enabled {
                            let ww_guard = wake_word_detector.read().await;
                            if let Some(ref detector) = *ww_guard {
                                if let Ok(Some(confidence)) = detector.detect(&reuse_chunk).await {
                                    is_awake = true;
                                    speech_buffer.clear();
                                    if let Some(ref handler) = *event_handler.read().await {
                                        handler(VoiceEvent::WakeWordDetected { confidence });
                                    }
                                }
                            } else {
                                is_awake = true;
                            }
                        } else {
                            is_awake = true;
                        }
                    }

                    if is_awake {
                        if speech_buffer.len() + packet.data.len() > max_buf {
                            let drop = speech_buffer
                                .len()
                                .saturating_add(packet.data.len())
                                .saturating_sub(max_buf);
                            if drop < speech_buffer.len() {
                                speech_buffer.drain(..drop);
                            } else {
                                speech_buffer.clear();
                            }
                        }
                        speech_buffer.extend_from_slice(&packet.data);
                    }

                    let elapsed = last_activity.read().await.elapsed().as_secs();
                    if elapsed > idle_timeout_s && !is_awake {
                        is_sleeping.store(true, Ordering::Relaxed);
                    }
                }
            }

            tracing::info!("Continuous listening task ended.");
        });

        *self.listening_task.write().await = Some(task);
        Ok(())
    }

    pub async fn stop_listening(&self) {
        self.stop_signal.notify_one();
        if let Some(task) = self.listening_task.write().await.take() {
            task.abort();
            let _ = tokio::time::timeout(tokio::time::Duration::from_secs(5), task).await;
        }
    }

    pub async fn with_default_engines(&self) -> Result<()> {
        let sr = self.config.audio.input.sample_rate;

        let vad = crate::detection::EnergyVadDetector::new(self.config.vad_threshold, sr)
            .with_min_speech_frames(3)
            .with_silence_frames_for_end(
                (self.config.orchestrator.silence_timeout_ms / 30).max(1) as usize
            );
        self.set_vad_detector(Box::new(vad)).await?;

        let ww = crate::detection::EnergyWakeWordDetector::new(&self.config.wake_word, 0.3, sr)
            .with_min_duration_frames(5)
            .with_cooldown_frames((2000.0_f64 / 30.0_f64).ceil() as usize);
        self.set_wake_word_detector(Box::new(ww)).await?;

        // Always set stub engines so the pipeline compiles and runs without feature flags.
        // These return meaningful errors directing users to enable real providers.
        self.set_stt_engine(Box::new(crate::stubs::StableStubSttEngine::new()))
            .await?;
        self.set_tts_engine(Box::new(crate::stubs::StableStubTtsEngine::new()))
            .await?;

        Ok(())
    }

    pub fn config(&self) -> &VoiceConfig {
        &self.config
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized.load(Ordering::SeqCst)
    }

    pub fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Relaxed)
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }

    pub async fn set_wake_word_detector(
        &self,
        detector: Box<dyn voxy_voice_orchestrator::WakeWordDetector>,
    ) -> Result<()> {
        *self.wake_word_detector.write().await = Some(detector);
        Ok(())
    }

    pub async fn set_vad_detector(
        &self,
        vad: Box<dyn voxy_voice_orchestrator::VadDetector>,
    ) -> Result<()> {
        *self.vad_detector.write().await = Some(vad);
        Ok(())
    }

    pub async fn set_stt_engine(
        &self,
        engine: Box<dyn voxy_voice_orchestrator::SttEngine>,
    ) -> Result<()> {
        *self.stt_engine.write().await = Some(engine);
        Ok(())
    }

    pub async fn set_tts_engine(
        &self,
        engine: Box<dyn voxy_voice_orchestrator::TtsEngine>,
    ) -> Result<()> {
        *self.tts_engine.write().await = Some(engine);
        Ok(())
    }

    pub async fn start_session(
        &self,
        user_id: Option<&str>,
    ) -> Result<Box<dyn voxy_conversation::ConversationSession>> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(VoiceError::NotInitialized);
        }
        let mut session = self
            .session_mgr
            .create_session()
            .await
            .map_err(|e| VoiceError::SpeechSessionError(e.to_string()))?;
        session
            .start(user_id, None)
            .await
            .map_err(|e| VoiceError::SpeechSessionError(e.to_string()))?;
        let session_id = session.id().clone();
        *self.session.write().await = Some(session);
        self.session_mgr
            .get_session(&session_id)
            .await
            .map_err(|e| VoiceError::SpeechSessionError(e.to_string()))
    }

    pub async fn end_session(&self) -> Result<()> {
        if let Some(session) = self.session.write().await.take() {
            let id = session.id().clone();
            self.session_mgr
                .end_session(&id)
                .await
                .map_err(|e| VoiceError::SpeechSessionError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn current_session(&self) -> Option<Box<dyn voxy_conversation::ConversationSession>> {
        let id = {
            let guard = self.session.read().await;
            guard.as_ref().map(|s| s.id().clone())
        };
        if let Some(id) = id {
            self.session_mgr.get_session(&id).await.ok()
        } else {
            None
        }
    }

    pub async fn process_audio(&self, packet: voxy_audio::AudioPacket) -> Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(VoiceError::NotRunning);
        }

        self.diagnostics.record_packet_captured(&packet).await;

        let chunk = voxy_voice_orchestrator::AudioChunk {
            data: packet.data,
            sample_rate: packet.sample_rate,
            channels: packet.channels,
            timestamp: packet.timestamp,
            sequence: packet.sequence,
            is_final: false,
        };

        if self.config.vad_enabled {
            let vad = self.vad_detector.read().await;
            if let Some(ref vad) = *vad {
                let is_voice = vad
                    .is_voice(&chunk)
                    .await
                    .map_err(|e| VoiceError::CaptureError(e.to_string()))?;
                if !is_voice {
                    return Ok(());
                }
            } else {
                return Err(VoiceError::NoVadDetector);
            }
        }

        if self.config.wake_word_enabled {
            let detector = self.wake_word_detector.read().await;
            if let Some(ref detector) = *detector {
                let confidence = detector
                    .detect(&chunk)
                    .await
                    .map_err(|e| VoiceError::CaptureError(e.to_string()))?;
                if let Some(conf) = confidence {
                    if let Some(ref handler) = *self.event_handler.read().await {
                        handler(VoiceEvent::WakeWordDetected { confidence: conf });
                    }
                }
            } else {
                return Err(VoiceError::NoWakeWordDetector);
            }
        }

        {
            let stt = self.stt_engine.read().await;
            if let Some(ref stt) = *stt {
                let text = stt
                    .transcribe(&chunk)
                    .await
                    .map_err(|e| VoiceError::ProviderError(e.to_string()))?;

                if let Some(ref mut session) = *self.session.write().await {
                    let _ = session.process_input(&text, chunk.is_final).await;
                }

                if let Some(ref handler) = *self.event_handler.read().await {
                    handler(VoiceEvent::TranscriptionResult {
                        text,
                        is_final: chunk.is_final,
                        confidence: 0.9,
                    });
                }
            } else {
                return Err(VoiceError::NoSttEngine);
            }
        }

        Ok(())
    }

    pub async fn speak(&self, text: &str) -> Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(VoiceError::NotRunning);
        }

        self.interrupt_tts().await;

        let tts = self.tts_engine.read().await;
        if let Some(ref tts) = *tts {
            if let Some(ref handler) = *self.event_handler.read().await {
                handler(VoiceEvent::SynthesisStarted {
                    text: text.to_string(),
                });
            }

            let mut stream = tts
                .synthesize_stream(text)
                .await
                .map_err(|e| VoiceError::ProviderError(e.to_string()))?;

            let mut total_samples = 0usize;
            let mut last_sr = stream.sample_rate();
            let mut fade_samples_remaining = (48000u32 * TTS_FADE_IN_MS / 1000) as usize;
            let tts_ref = self.v2_tts_reference.clone();
            let tts_ref_pos = self.v2_tts_ref_pos.clone();
            let speed = *self.voice_speed.read().await;

            while let Some(chunk) = stream.next_chunk().await {
                total_samples += chunk.data.len();
                last_sr = chunk.sample_rate;

                // Apply fade-in
                let mut faded_data = chunk.data;
                if fade_samples_remaining > 0 {
                    for sample in faded_data.iter_mut() {
                        if fade_samples_remaining == 0 {
                            break;
                        }
                        let progress = 1.0
                            - (fade_samples_remaining as f32
                                / (48000u32 * TTS_FADE_IN_MS / 1000) as f32);
                        *sample *= progress.max(0.0);
                        fade_samples_remaining = fade_samples_remaining.saturating_sub(1);
                    }
                }

                // Apply voice speed (linear interpolation resampling)
                if (speed - 1.0).abs() > 0.01 {
                    faded_data = Self::resample_speed(&faded_data, speed);
                }

                // Write TTS output into AEC reference buffer
                {
                    let mut ref_buf = tts_ref.lock();
                    let buf_len = ref_buf.len();
                    for &s in &faded_data {
                        let pos = tts_ref_pos.load(Ordering::Relaxed);
                        ref_buf[pos % buf_len] = s;
                        tts_ref_pos.store((pos + 1) % buf_len, Ordering::Relaxed);
                    }
                }

                let packet =
                    voxy_audio::AudioPacket::new(faded_data, chunk.sample_rate, chunk.channels);

                if let Some(ref mut output) = *self.audio_output.write().await {
                    let _ = output.write(&packet).await;
                }

                self.diagnostics.record_packet_played(&packet).await;
            }

            if let Some(ref mut session) = *self.session.write().await {
                let _ = session.generate_output(text).await;
            }

            if let Some(ref handler) = *self.event_handler.read().await {
                handler(VoiceEvent::SynthesisCompleted {
                    duration_ms: (total_samples as f64 / last_sr as f64 * 1000.0) as u64,
                });
            }

            Ok(())
        } else {
            Err(VoiceError::NoTtsEngine)
        }
    }

    /// Resample audio by speed factor using linear interpolation.
    /// speed > 1.0 = faster, speed < 1.0 = slower.
    pub(crate) fn resample_speed(data: &[f32], speed: f64) -> Vec<f32> {
        let speed = speed as f32;
        if data.is_empty() || speed <= 0.0 {
            return data.to_vec();
        }
        let out_len = (data.len() as f32 / speed) as usize;
        let mut out = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let pos = i as f32 * speed;
            let idx = pos as usize;
            let frac = pos - idx as f32;
            let sample = if idx + 1 < data.len() {
                data[idx] * (1.0 - frac) + data[idx + 1] * frac
            } else if idx < data.len() {
                data[idx]
            } else {
                0.0
            };
            out.push(sample);
        }
        out
    }

    pub async fn diagnostics(&self) -> voxy_audio::AudioMetrics {
        self.diagnostics.metrics().await
    }

    pub async fn on_event(&self, handler: Box<dyn Fn(VoiceEvent) + Send + Sync>) -> Result<()> {
        *self.event_handler.write().await = Some(handler);
        Ok(())
    }

    pub fn stop_signal(&self) -> Arc<Notify> {
        self.stop_signal.clone()
    }

    // ── Hot-swap: Audio Device Recovery ─────────────────────────────────

    /// Initialize the hot-swap manager with default configuration.
    pub fn initialize_hot_swap(&self) {
        self.hot_swap
            .initialize(voxy_audio::HotSwapConfig::default());
        tracing::info!("Hot-swap manager initialized for audio device recovery");
    }

    /// Get the current hot-swap pipeline state.
    pub fn hot_swap_state(&self) -> voxy_audio::PipelineState {
        self.hot_swap.state()
    }

    /// Get the number of hot-swap recovery attempts.
    pub fn hot_swap_recovery_attempts(&self) -> u32 {
        self.hot_swap.recovery_attempts()
    }

    /// Handle an audio device error with automatic recovery.
    /// Returns Ok(()) if recovery succeeded, Err if pipeline should stop.
    pub async fn handle_device_error(&self, error: &str) -> Result<()> {
        tracing::warn!("Audio device error: {error} — attempting recovery");

        if !self.hot_swap.is_initialized() {
            self.initialize_hot_swap();
        }

        // Stop current capture
        if self.is_running.load(Ordering::SeqCst) {
            if let Err(e) = self.stop_capture().await {
                tracing::warn!("stop_capture during recovery: {e}");
            }
        }

        // Wait a moment for device to settle
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Attempt to restart capture with exponential backoff
        let max_attempts = 3u32;
        for attempt in 0..max_attempts {
            match self.start_capture().await {
                Ok(()) => {
                    tracing::info!("Audio device recovery successful (attempt {})", attempt + 1);
                    self.hot_swap.record_device_change();
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Recovery attempt {} failed: {e}", attempt + 1);
                    if attempt + 1 >= max_attempts {
                        tracing::error!("Max recovery attempts reached — entering sleep mode");
                        self.hot_swap.set_state(voxy_audio::PipelineState::Sleeping);
                        return Err(VoiceError::AudioDeviceError(format!(
                            "Device recovery failed after {max_attempts} attempts: {e}"
                        )));
                    }
                    let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                    tracing::info!("Retrying recovery in {delay:?} (attempt {})", attempt + 2);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(VoiceError::AudioDeviceError(
            "Recovery exhausted".to_string(),
        ))
    }

    // ── Voice Engine V2 ──────────────────────────────────────────────

    /// Initialize V2 modules: start watchdog background task, run calibration.
    pub async fn initialize_v2(&self) -> Result<()> {
        // Start watchdog background monitoring
        if let Some(ref watchdog) = self.v2_watchdog {
            watchdog.register_stage("audio_input");
            watchdog.register_stage("stt");
            watchdog.register_stage("llm");
            watchdog.register_stage("tts");
            watchdog.register_stage("audio_output");

            let watchdog = watchdog.clone();
            let is_running = self.is_running.clone();
            let check_interval = Duration::from_secs(5);
            let task = tokio::spawn(async move {
                let mut interval = tokio::time::interval(check_interval);
                interval.tick().await; // skip first immediate tick
                loop {
                    interval.tick().await;
                    if !is_running.load(Ordering::Relaxed) {
                        break;
                    }
                    let needs_recovery = watchdog.check_stages();
                    if !needs_recovery.is_empty() {
                        tracing::warn!(stages = ?needs_recovery, "Watchdog: stages need recovery");
                    }
                }
            });
            *self.v2_watchdog_task.write().await = Some(task);
            tracing::info!("V2 watchdog started");
        }

        // Run self-calibration (non-blocking, collects data over time)
        if let Some(ref calibrator) = self.v2_calibrator {
            if !calibrator.is_calibrated() {
                tracing::info!("V2 self-calibration: will calibrate during first use");
            }
        }

        Ok(())
    }

    /// Get the V2 mixer (if initialized).
    pub fn mixer(&self) -> Option<&Arc<voxy_audio::AudioMixer>> {
        self.v2_mixer.as_ref()
    }

    /// Get the V2 calibrator (if initialized).
    pub fn calibrator(&self) -> Option<&Arc<voxy_audio::SelfCalibrator>> {
        self.v2_calibrator.as_ref()
    }

    /// Get the V2 watchdog (if initialized).
    pub fn watchdog(&self) -> Option<&Arc<voxy_audio::HealthWatchdog>> {
        self.v2_watchdog.as_ref()
    }

    /// Get the V2 metrics collector (if initialized).
    pub fn metrics_collector(&self) -> Option<&Arc<voxy_audio::MetricsCollector>> {
        self.v2_metrics_collector.as_ref()
    }

    /// Get the V2 voice memory (if initialized).
    pub fn voice_memory(&self) -> Option<&Arc<voxy_audio::VoiceMemory>> {
        self.v2_voice_memory.as_ref()
    }

    /// Get current V2 engine metrics snapshot.
    pub async fn v2_snapshot(&self) -> Option<voxy_audio::LatencyMetrics> {
        self.v2_metrics_collector
            .as_ref()
            .map(|mc| mc.latency_snapshot())
    }

    /// Set the V2 audio mixer.
    pub fn set_mixer(&mut self, mixer: voxy_audio::AudioMixer) {
        self.v2_mixer = Some(Arc::new(mixer));
    }

    /// Heartbeat to watchdog for a specific stage.
    pub fn watchdog_heartbeat(&self, stage: &str) {
        if let Some(ref watchdog) = self.v2_watchdog {
            watchdog.heartbeat(stage);
        }
    }

    /// Record a failure to watchdog for a specific stage.
    pub fn watchdog_failure(&self, stage: &str) {
        if let Some(ref watchdog) = self.v2_watchdog {
            watchdog.record_failure(stage);
        }
    }

    /// Record a latency sample in the metrics collector.
    pub fn record_stt_latency(&self, ms: f64) {
        if let Some(ref mc) = self.v2_metrics_collector {
            mc.record_stt_latency(ms);
        }
    }

    pub fn record_llm_latency(&self, ms: f64) {
        if let Some(ref mc) = self.v2_metrics_collector {
            mc.record_llm_latency(ms);
        }
    }

    pub fn record_tts_latency(&self, ms: f64) {
        if let Some(ref mc) = self.v2_metrics_collector {
            mc.record_tts_latency(ms);
        }
    }

    /// Stop the V2 watchdog background task.
    async fn stop_v2_watchdog(&self) {
        if let Some(task) = self.v2_watchdog_task.write().await.take() {
            task.abort();
            let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
        }
        if let Some(ref watchdog) = self.v2_watchdog {
            watchdog.stop_monitoring();
        }
    }
}
