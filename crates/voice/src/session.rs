use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::config::VoiceConfig;
use crate::error::{Result, VoiceError};
use crate::VoiceEvent;

type EventHandler = Arc<RwLock<Option<Box<dyn Fn(VoiceEvent) + Send + Sync>>>>;

#[allow(dead_code)]
pub struct SpeechSession {
    conversation: Mutex<Box<dyn voxy_conversation::ConversationSession>>,
    is_active: AtomicBool,
    config: VoiceConfig,
    audio_input: Arc<RwLock<Option<Box<dyn voxy_audio::AudioInputStream>>>>,
    audio_output: Arc<RwLock<Option<Box<dyn voxy_audio::AudioOutputStream>>>>,
    event_handler: EventHandler,
}

impl SpeechSession {
    pub fn new(
        session: Box<dyn voxy_conversation::ConversationSession>,
        config: VoiceConfig,
    ) -> Self {
        let active = session.state() == voxy_conversation::SessionState::Active;
        Self {
            conversation: Mutex::new(session),
            is_active: AtomicBool::new(active),
            config,
            audio_input: Arc::new(RwLock::new(None)),
            audio_output: Arc::new(RwLock::new(None)),
            event_handler: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn process_speech(&self, packet: voxy_audio::AudioPacket) -> Result<()> {
        if self.config.vad_enabled {
            let is_silent = packet.is_silent(self.config.vad_threshold);
            if is_silent {
                return Ok(());
            }
        }

        {
            let mut conv = self.conversation.lock().await;
            conv.process_input("speech captured", true)
                .await
                .map_err(|e| VoiceError::SpeechSessionError(e.to_string()))?;
        }

        if let Some(ref handler) = *self.event_handler.read().await {
            handler(VoiceEvent::VoiceActivityStarted);
        }

        Ok(())
    }

    pub async fn respond(&self, text: &str) -> Result<()> {
        {
            let mut conv = self.conversation.lock().await;
            conv.generate_output(text)
                .await
                .map_err(|e| VoiceError::SpeechSessionError(e.to_string()))?;
        }

        if let Some(ref handler) = *self.event_handler.read().await {
            handler(VoiceEvent::SynthesisStarted {
                text: text.to_string(),
            });
        }

        Ok(())
    }

    pub async fn interrupt(&self) -> Result<()> {
        if let Some(ref handler) = *self.event_handler.read().await {
            handler(VoiceEvent::VoiceActivityEnded { duration_ms: 0 });
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
}
