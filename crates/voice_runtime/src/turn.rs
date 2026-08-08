use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use crate::config::TurnDetectionConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnBoundary {
    None,
    EndOfUtterance,
    LongPause,
    MaxDurationReached,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct TurnState {
    pub is_in_turn: bool,
    pub speech_started: Option<Instant>,
    pub last_speech_time: Option<Instant>,
    pub last_silence_time: Option<Instant>,
    pub speech_duration_ms: u64,
    pub silence_duration_ms: u64,
    pub frame_count: u64,
}

pub struct TurnDetector {
    config: TurnDetectionConfig,
    is_in_turn: Arc<AtomicBool>,
    speech_start: Arc<RwLock<Option<Instant>>>,
    last_speech_time: Arc<RwLock<Option<Instant>>>,
    last_silence_time: Arc<RwLock<Option<Instant>>>,
    speech_duration_ms: Arc<AtomicU64>,
    silence_duration_ms: Arc<AtomicU64>,
    frame_count: Arc<AtomicU64>,
    turn_start_time: Arc<RwLock<Option<Instant>>>,
}

impl TurnDetector {
    pub fn new(config: TurnDetectionConfig) -> Self {
        Self {
            config,
            is_in_turn: Arc::new(AtomicBool::new(false)),
            speech_start: Arc::new(RwLock::new(None)),
            last_speech_time: Arc::new(RwLock::new(None)),
            last_silence_time: Arc::new(RwLock::new(None)),
            speech_duration_ms: Arc::new(AtomicU64::new(0)),
            silence_duration_ms: Arc::new(AtomicU64::new(0)),
            frame_count: Arc::new(AtomicU64::new(0)),
            turn_start_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn process_frame(&self, is_voice: bool) -> TurnBoundary {
        let now = Instant::now();
        self.frame_count.fetch_add(1, Ordering::Relaxed);

        if is_voice {
            self.silence_duration_ms.store(0, Ordering::Relaxed);

            if !self.is_in_turn.load(Ordering::SeqCst) {
                self.is_in_turn.store(true, Ordering::SeqCst);
                *self.speech_start.write().await = Some(now);
                *self.turn_start_time.write().await = Some(now);
                *self.last_speech_time.write().await = Some(now);
                self.speech_duration_ms.store(0, Ordering::Relaxed);
            } else {
                *self.last_speech_time.write().await = Some(now);
                if let Some(start) = *self.speech_start.read().await {
                    let elapsed = start.elapsed().as_millis() as u64;
                    self.speech_duration_ms.store(elapsed, Ordering::Relaxed);

                    if elapsed >= self.config.max_speech_duration_ms {
                        self.reset().await;
                        return TurnBoundary::MaxDurationReached;
                    }
                }
            }

            TurnBoundary::None
        } else {
            if self.is_in_turn.load(Ordering::SeqCst) {
                let silence = {
                    if let Some(last_speech) = *self.last_speech_time.read().await {
                        last_speech.elapsed().as_millis() as u64
                    } else {
                        0
                    }
                };
                self.silence_duration_ms.store(silence, Ordering::Relaxed);

                if silence >= self.config.end_of_utterance_silence_ms {
                    self.reset().await;
                    return TurnBoundary::EndOfUtterance;
                }

                if silence >= self.config.long_pause_threshold_ms {
                    self.reset().await;
                    return TurnBoundary::LongPause;
                }
            }

            if let Some(turn_start) = *self.turn_start_time.read().await {
                if turn_start.elapsed().as_millis() as u64 >= self.config.turn_timeout_ms {
                    self.reset().await;
                    return TurnBoundary::Timeout;
                }
            }

            TurnBoundary::None
        }
    }

    pub async fn reset(&self) {
        self.is_in_turn.store(false, Ordering::SeqCst);
        *self.speech_start.write().await = None;
        *self.last_speech_time.write().await = None;
        *self.last_silence_time.write().await = Some(Instant::now());
        self.speech_duration_ms.store(0, Ordering::Relaxed);
        self.silence_duration_ms.store(0, Ordering::Relaxed);
    }

    pub fn is_in_turn(&self) -> bool {
        self.is_in_turn.load(Ordering::SeqCst)
    }

    pub async fn state(&self) -> TurnState {
        TurnState {
            is_in_turn: self.is_in_turn.load(Ordering::SeqCst),
            speech_started: *self.speech_start.read().await,
            last_speech_time: *self.last_speech_time.read().await,
            last_silence_time: *self.last_silence_time.read().await,
            speech_duration_ms: self.speech_duration_ms.load(Ordering::Relaxed),
            silence_duration_ms: self.silence_duration_ms.load(Ordering::Relaxed),
            frame_count: self.frame_count.load(Ordering::Relaxed),
        }
    }
}
