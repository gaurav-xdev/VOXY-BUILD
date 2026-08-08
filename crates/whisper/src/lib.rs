use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
#[cfg(feature = "whisper-engine")]
use tracing::info;
use tracing::warn;
use voxy_voice_orchestrator::{AudioChunk, AudioStream, SttEngine};

pub use voxy_provider_core as core_traits;

#[derive(Debug, thiserror::Error)]
pub enum WhisperError {
    #[error("Model not loaded")]
    ModelNotLoaded,
    #[error("Model file not found: {0}")]
    ModelNotFound(String),
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    #[error("Audio error: {0}")]
    AudioError(String),
}

pub struct WhisperSttEngine {
    name: String,
    model_path: Option<PathBuf>,
    #[cfg(feature = "whisper-engine")]
    context: Arc<Mutex<Option<whisper_rs::WhisperContext>>>,
    #[cfg(feature = "whisper-engine")]
    cached_state: Arc<Mutex<Option<whisper_rs::WhisperState>>>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u8,
    max_buffer_seconds: f64,
    is_loaded: AtomicBool,
    language: String,
    translate: bool,
    #[cfg(feature = "whisper-engine")]
    no_timestamps: bool,
}

impl WhisperSttEngine {
    pub fn new() -> Self {
        Self {
            name: "whisper-stt".into(),
            model_path: None,
            #[cfg(feature = "whisper-engine")]
            context: Arc::new(Mutex::new(None)),
            #[cfg(feature = "whisper-engine")]
            cached_state: Arc::new(Mutex::new(None)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000,
            channels: 1,
            max_buffer_seconds: 30.0,
            is_loaded: AtomicBool::new(false),
            language: "en".to_string(),
            translate: false,
            #[cfg(feature = "whisper-engine")]
            no_timestamps: true,
        }
    }

    pub fn with_model_path(mut self, path: PathBuf) -> Self {
        self.model_path = Some(path);
        self
    }

    pub fn with_language(mut self, lang: &str) -> Self {
        self.language = lang.to_string();
        self
    }

    pub fn with_translate(mut self, translate: bool) -> Self {
        self.translate = translate;
        self
    }

    #[cfg(feature = "whisper-engine")]
    pub fn load_model(&self) -> Result<(), WhisperError> {
        let path = self
            .model_path
            .as_ref()
            .ok_or(WhisperError::ModelNotLoaded)?;

        if !path.exists() {
            return Err(WhisperError::ModelNotFound(path.display().to_string()));
        }

        let ctx = whisper_rs::WhisperContext::new_with_params(
            path.to_str()
                .ok_or_else(|| WhisperError::ModelNotFound("Invalid model path".to_string()))?,
            whisper_rs::WhisperContextParameters::default(),
        )
        .map_err(|e| WhisperError::TranscriptionFailed(e.to_string()))?;

        let state = ctx
            .create_state()
            .map_err(|e| WhisperError::TranscriptionFailed(e.to_string()))?;

        *self.context.lock() = Some(ctx);
        *self.cached_state.lock() = Some(state);
        self.is_loaded.store(true, Ordering::SeqCst);

        info!("Whisper model loaded from {}", path.display());
        Ok(())
    }

    #[cfg(not(feature = "whisper-engine"))]
    pub fn load_model(&self) -> Result<(), WhisperError> {
        let path = self
            .model_path
            .as_ref()
            .ok_or(WhisperError::ModelNotLoaded)?;

        if !path.exists() {
            return Err(WhisperError::ModelNotFound(path.display().to_string()));
        }

        warn!(
            "whisper-engine feature disabled; model at {} not loaded",
            path.display()
        );
        Ok(())
    }

    pub fn is_model_loaded(&self) -> bool {
        self.is_loaded.load(Ordering::SeqCst)
    }

    pub fn clear_buffer(&self) {
        self.buffer.lock().clear();
    }

    pub fn buffered_duration_ms(&self) -> f64 {
        let buf = self.buffer.lock();
        if self.sample_rate == 0 || self.channels == 0 {
            return 0.0;
        }
        (buf.len() as f64 / (self.sample_rate as f64 * self.channels as f64)) * 1000.0
    }

    fn max_buffer_samples(&self) -> usize {
        (self.sample_rate as f64 * self.max_buffer_seconds) as usize * self.channels as usize
    }

    #[cfg(feature = "whisper-engine")]
    #[allow(dead_code)]
    fn transcribe_audio(&self, audio: &[f32], _sample_rate: u32) -> Result<String, WhisperError> {
        let mut state_guard = self.cached_state.lock();
        let state = state_guard.as_mut().ok_or(WhisperError::ModelNotLoaded)?;

        let mut params =
            whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });

        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(4);
        params.set_n_threads(n_threads);
        params.set_translate(self.translate);
        params.set_no_timestamps(self.no_timestamps);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);

        state
            .full(params, audio)
            .map_err(|e| WhisperError::TranscriptionFailed(e.to_string()))?;

        let num_segments = state.full_n_segments();

        let mut text = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(segment_text) = segment.to_str_lossy() {
                    text.push_str(&segment_text);
                }
            }
        }

        Ok(text.trim().to_string())
    }

    #[cfg(not(feature = "whisper-engine"))]
    #[allow(dead_code)]
    fn transcribe_audio(&self, _audio: &[f32], _sample_rate: u32) -> Result<String, WhisperError> {
        Ok(String::new())
    }

    #[cfg(feature = "whisper-engine")]
    async fn transcribe_audio_async(
        &self,
        audio: Vec<f32>,
        _sample_rate: u32,
    ) -> Result<String, WhisperError> {
        let cached_state = self.cached_state.clone();
        let translate = self.translate;
        let no_timestamps = self.no_timestamps;

        tokio::task::spawn_blocking(move || {
            let mut guard = cached_state.lock();
            let state = guard.as_mut().ok_or(WhisperError::ModelNotLoaded)?;

            let mut params =
                whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });

            let n_threads = std::thread::available_parallelism()
                .map(|n| n.get() as i32)
                .unwrap_or(4);
            params.set_n_threads(n_threads);
            params.set_translate(translate);
            params.set_no_timestamps(no_timestamps);
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            params.set_suppress_blank(true);
            params.set_suppress_nst(true);

            state
                .full(params, &audio)
                .map_err(|e| WhisperError::TranscriptionFailed(e.to_string()))?;

            let num_segments = state.full_n_segments();

            let mut text = String::new();
            for i in 0..num_segments {
                if let Some(segment) = state.get_segment(i) {
                    if let Ok(segment_text) = segment.to_str_lossy() {
                        text.push_str(&segment_text);
                    }
                }
            }

            Ok(text.trim().to_string())
        })
        .await
        .map_err(|e| WhisperError::TranscriptionFailed(format!("Task join error: {e}")))?
    }

    #[cfg(not(feature = "whisper-engine"))]
    async fn transcribe_audio_async(
        &self,
        _audio: Vec<f32>,
        _sample_rate: u32,
    ) -> Result<String, WhisperError> {
        Ok(String::new())
    }
}

impl Default for WhisperSttEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SttEngine for WhisperSttEngine {
    fn name(&self) -> &str {
        &self.name
    }

    async fn transcribe(
        &self,
        audio: &AudioChunk,
    ) -> Result<String, voxy_voice_orchestrator::VoiceOrchestratorError> {
        if audio.data.is_empty() {
            return Ok(String::new());
        }

        {
            let mut buf = self.buffer.lock();
            let max_samples = self.max_buffer_samples();
            let overflow = buf
                .len()
                .saturating_add(audio.data.len())
                .saturating_sub(max_samples);
            if overflow > 0 {
                let end = overflow.min(buf.len());
                buf.drain(..end);
            }
            buf.extend_from_slice(&audio.data);
        }

        if !self.is_model_loaded() {
            return Ok(String::new());
        }

        if audio.is_final {
            let accumulated = {
                let mut buf = self.buffer.lock();
                std::mem::take(&mut *buf)
            };

            let rms = compute_rms(&accumulated);
            if rms < 0.01 {
                return Ok(String::new());
            }

            let result = self
                .transcribe_audio_async(accumulated, audio.sample_rate)
                .await;
            match result {
                Ok(text) => Ok(text),
                Err(e) => {
                    warn!("Transcription error: {}", e);
                    Ok(String::new())
                }
            }
        } else {
            Ok(String::new())
        }
    }

    async fn transcribe_stream(
        &self,
        mut stream: Box<dyn AudioStream>,
    ) -> Result<String, voxy_voice_orchestrator::VoiceOrchestratorError> {
        let mut accumulated = Vec::new();
        let sr = stream.sample_rate();

        while let Some(chunk) = stream.next_chunk().await {
            accumulated.extend_from_slice(&chunk.data);
        }

        self.buffer.lock().clear();

        let rms = compute_rms(&accumulated);
        if rms < 0.01 {
            return Ok(String::new());
        }

        match self.transcribe_audio_async(accumulated, sr).await {
            Ok(text) => Ok(text),
            Err(e) => {
                warn!("Stream transcription error: {}", e);
                Ok(String::new())
            }
        }
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "en".to_string(),
            "es".to_string(),
            "fr".to_string(),
            "de".to_string(),
            "it".to_string(),
            "pt".to_string(),
            "ru".to_string(),
            "ja".to_string(),
            "ko".to_string(),
            "zh".to_string(),
        ]
    }

    fn is_available(&self) -> bool {
        self.is_model_loaded()
    }
}

fn compute_rms(data: &[f32]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = data.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / data.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_chunk(data: Vec<f32>, sample_rate: u32, is_final: bool) -> AudioChunk {
        AudioChunk {
            data,
            sample_rate,
            channels: 1,
            timestamp: Utc::now(),
            sequence: 0,
            is_final,
        }
    }

    #[tokio::test]
    async fn test_whisper_empty_audio() {
        let engine = WhisperSttEngine::new();
        let chunk = make_chunk(vec![], 16000, false);
        let result = engine.transcribe(&chunk).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_whisper_silence() {
        let engine = WhisperSttEngine::new();
        let chunk = make_chunk(vec![0.0; 480], 16000, true);
        let result = engine.transcribe(&chunk).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_whisper_non_final_returns_empty() {
        let engine = WhisperSttEngine::new();
        let data: Vec<f32> = (0..480).map(|i| (i as f32 / 480.0 * 0.5).sin()).collect();
        let chunk = make_chunk(data, 16000, false);
        let result = engine.transcribe(&chunk).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_whisper_no_model_returns_empty() {
        let engine = WhisperSttEngine::new();
        let data: Vec<f32> = (0..16000)
            .map(|i| (i as f32 / 16000.0 * 0.5).sin())
            .collect();
        let chunk = make_chunk(data, 16000, true);
        let result = engine.transcribe(&chunk).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_whisper_name_and_languages() {
        let engine = WhisperSttEngine::new();
        assert_eq!(engine.name(), "whisper-stt");
        assert!(engine.supported_languages().contains(&"en".to_string()));
    }

    #[tokio::test]
    async fn test_whisper_buffer_management() {
        let engine = WhisperSttEngine::new();
        let data: Vec<f32> = (0..480).map(|i| (i as f32 / 480.0 * 0.5).sin()).collect();
        let chunk = make_chunk(data, 16000, false);
        let _ = engine.transcribe(&chunk).await.unwrap();
        assert!(engine.buffered_duration_ms() > 0.0);
        engine.clear_buffer();
        assert!(engine.buffered_duration_ms() < 0.1);
    }

    #[tokio::test]
    async fn test_whisper_config() {
        let engine = WhisperSttEngine::new()
            .with_language("es")
            .with_translate(true);
        assert_eq!(engine.language, "es");
        assert!(engine.translate);
    }
}
