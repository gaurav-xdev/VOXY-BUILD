use async_trait::async_trait;

use crate::config::AudioStreamConfig;
use crate::device::AudioDeviceInfo;
use crate::error::{AudioError, Result};
use crate::stream::{AudioInputStream, AudioOutputStream};

#[async_trait]
pub trait AudioBackend: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn enumerate_inputs(&self) -> Result<Vec<AudioDeviceInfo>>;
    async fn enumerate_outputs(&self) -> Result<Vec<AudioDeviceInfo>>;
    async fn default_input(&self) -> Result<AudioDeviceInfo>;
    async fn default_output(&self) -> Result<AudioDeviceInfo>;
    async fn open_input(&self, config: &AudioStreamConfig) -> Result<Box<dyn AudioInputStream>>;
    async fn open_output(&self, config: &AudioStreamConfig) -> Result<Box<dyn AudioOutputStream>>;
}

pub struct FallbackBackend;

impl FallbackBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FallbackBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn unsupported() -> AudioError {
    AudioError::PlatformError("Not supported by fallback backend".to_string())
}

#[async_trait]
impl AudioBackend for FallbackBackend {
    fn name(&self) -> &str {
        "fallback"
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn enumerate_inputs(&self) -> Result<Vec<AudioDeviceInfo>> {
        Ok(Vec::new())
    }

    async fn enumerate_outputs(&self) -> Result<Vec<AudioDeviceInfo>> {
        Ok(Vec::new())
    }

    async fn default_input(&self) -> Result<AudioDeviceInfo> {
        Err(unsupported())
    }

    async fn default_output(&self) -> Result<AudioDeviceInfo> {
        Err(unsupported())
    }

    async fn open_input(&self, _config: &AudioStreamConfig) -> Result<Box<dyn AudioInputStream>> {
        Err(unsupported())
    }

    async fn open_output(&self, _config: &AudioStreamConfig) -> Result<Box<dyn AudioOutputStream>> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_backend_name() {
        let backend = FallbackBackend::new();
        assert_eq!(backend.name(), "fallback");
    }

    #[tokio::test]
    async fn test_fallback_backend_is_available() {
        let backend = FallbackBackend::new();
        assert!(backend.is_available());
    }

    #[tokio::test]
    async fn test_fallback_backend_enumerate_inputs() {
        let backend = FallbackBackend::new();
        let inputs = backend.enumerate_inputs().await.unwrap();
        assert!(inputs.is_empty());
    }

    #[tokio::test]
    async fn test_fallback_backend_enumerate_outputs() {
        let backend = FallbackBackend::new();
        let outputs = backend.enumerate_outputs().await.unwrap();
        assert!(outputs.is_empty());
    }

    #[tokio::test]
    async fn test_fallback_backend_default_input_error() {
        let backend = FallbackBackend::new();
        let result = backend.default_input().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fallback_backend_default_output_error() {
        let backend = FallbackBackend::new();
        let result = backend.default_output().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fallback_backend_open_input_error() {
        let backend = FallbackBackend::new();
        let config = AudioStreamConfig::default();
        let result = backend.open_input(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fallback_backend_open_output_error() {
        let backend = FallbackBackend::new();
        let config = AudioStreamConfig::default();
        let result = backend.open_output(&config).await;
        assert!(result.is_err());
    }
}
