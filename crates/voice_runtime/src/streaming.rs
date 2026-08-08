use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::debug;

use crate::types::VoiceStreamEvent;

pub struct StreamingManager {
    event_tx: broadcast::Sender<VoiceStreamEvent>,
    event_buffer: Arc<parking_lot::Mutex<VecDeque<VoiceStreamEvent>>>,
    buffer_size: usize,
    event_count: Arc<std::sync::atomic::AtomicU64>,
    partial_transcription_interval_ms: u64,
    last_partial_transcription: Arc<std::sync::atomic::AtomicU64>,
}

impl StreamingManager {
    pub fn new(buffer_size: usize, partial_transcription_interval_ms: u64) -> Self {
        let (event_tx, _) = broadcast::channel(buffer_size.max(1));
        Self {
            event_tx,
            event_buffer: Arc::new(parking_lot::Mutex::new(VecDeque::with_capacity(
                buffer_size,
            ))),
            buffer_size,
            event_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            partial_transcription_interval_ms,
            last_partial_transcription: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<VoiceStreamEvent> {
        self.event_tx.subscribe()
    }

    pub fn emit(&self, event: VoiceStreamEvent) {
        if Self::should_buffer_event(&event) {
            let mut buf = self.event_buffer.lock();
            if buf.len() >= self.buffer_size {
                buf.pop_front();
            }
            buf.push_back(event.clone());
        }

        self.event_count.fetch_add(1, Ordering::Relaxed);

        if let Err(e) = self.event_tx.send(event) {
            debug!("No active stream subscribers: {}", e);
        }
    }

    fn should_buffer_event(event: &VoiceStreamEvent) -> bool {
        matches!(
            event,
            VoiceStreamEvent::TurnStarted { .. }
                | VoiceStreamEvent::TurnCompleted { .. }
                | VoiceStreamEvent::TurnFailed { .. }
                | VoiceStreamEvent::PartialTranscription { .. }
        )
    }

    pub fn should_send_partial_transcription(&self) -> bool {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let last = self.last_partial_transcription.load(Ordering::Relaxed);
        if now_ms.saturating_sub(last) >= self.partial_transcription_interval_ms {
            self.last_partial_transcription
                .store(now_ms, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn buffered_events(&self) -> Vec<VoiceStreamEvent> {
        self.event_buffer.lock().iter().cloned().collect()
    }

    pub fn event_count(&self) -> u64 {
        self.event_count.load(Ordering::Relaxed)
    }

    pub fn clear_buffer(&self) {
        self.event_buffer.lock().clear();
    }
}
