use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;

use crate::config::AudioRuntimeConfig;
use crate::config::AudioStreamConfig;
use crate::error::Result;
use crate::stream::{AudioInputStream, AudioOutputStream};

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: voxy_hardware::DeviceType,
    pub status: voxy_hardware::DeviceStatus,
    pub supported_sample_rates: Vec<u32>,
    pub supported_channels: Vec<u8>,
    pub is_default: bool,
}

impl AudioDeviceInfo {
    pub fn from_hardware(info: voxy_hardware::DeviceInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            device_type: info.device_type,
            status: info.status,
            supported_sample_rates: vec![8000, 16000, 44100, 48000],
            supported_channels: vec![1, 2],
            is_default: false,
        }
    }
}

#[async_trait]
pub trait AudioDeviceManager: Send + Sync {
    async fn initialize(&self, config: &AudioRuntimeConfig) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn list_inputs(&self) -> Result<Vec<AudioDeviceInfo>>;
    async fn list_outputs(&self) -> Result<Vec<AudioDeviceInfo>>;
    async fn default_input(&self) -> Result<AudioDeviceInfo>;
    async fn default_output(&self) -> Result<AudioDeviceInfo>;
    async fn open_input(&self, config: &AudioStreamConfig) -> Result<Box<dyn AudioInputStream>>;
    async fn open_output(&self, config: &AudioStreamConfig) -> Result<Box<dyn AudioOutputStream>>;
    async fn get_device(&self, id: &str) -> Result<AudioDeviceInfo>;
    fn is_initialized(&self) -> bool;
}

pub struct InMemoryDeviceManager {
    initialized: AtomicBool,
    config: Arc<Mutex<Option<AudioRuntimeConfig>>>,
}

impl Default for InMemoryDeviceManager {
    fn default() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

impl InMemoryDeviceManager {
    fn make_default_device(
        id: &str,
        name: &str,
        device_type: voxy_hardware::DeviceType,
    ) -> AudioDeviceInfo {
        AudioDeviceInfo {
            id: id.to_string(),
            name: name.to_string(),
            device_type,
            status: voxy_hardware::DeviceStatus::Available,
            supported_sample_rates: vec![8000, 16000, 44100, 48000],
            supported_channels: vec![1, 2],
            is_default: true,
        }
    }
}

#[async_trait]
impl AudioDeviceManager for InMemoryDeviceManager {
    async fn initialize(&self, config: &AudioRuntimeConfig) -> Result<()> {
        if self.initialized.load(Ordering::SeqCst) {
            return Err(crate::error::AudioError::AlreadyInitialized);
        }
        self.initialized.store(true, Ordering::SeqCst);
        *self.config.lock() = Some(config.clone());
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(crate::error::AudioError::NotInitialized);
        }
        self.initialized.store(false, Ordering::SeqCst);
        *self.config.lock() = None;
        Ok(())
    }

    async fn list_inputs(&self) -> Result<Vec<AudioDeviceInfo>> {
        Ok(vec![Self::make_default_device(
            "mem-input-001",
            "In-Memory Microphone",
            voxy_hardware::DeviceType::Microphone,
        )])
    }

    async fn list_outputs(&self) -> Result<Vec<AudioDeviceInfo>> {
        Ok(vec![Self::make_default_device(
            "mem-output-001",
            "In-Memory Speaker",
            voxy_hardware::DeviceType::Speaker,
        )])
    }

    async fn default_input(&self) -> Result<AudioDeviceInfo> {
        Ok(Self::make_default_device(
            "mem-input-001",
            "In-Memory Microphone",
            voxy_hardware::DeviceType::Microphone,
        ))
    }

    async fn default_output(&self) -> Result<AudioDeviceInfo> {
        Ok(Self::make_default_device(
            "mem-output-001",
            "In-Memory Speaker",
            voxy_hardware::DeviceType::Speaker,
        ))
    }

    async fn open_input(&self, _config: &AudioStreamConfig) -> Result<Box<dyn AudioInputStream>> {
        Err(crate::error::AudioError::NotInitialized)
    }

    async fn open_output(&self, _config: &AudioStreamConfig) -> Result<Box<dyn AudioOutputStream>> {
        Err(crate::error::AudioError::NotInitialized)
    }

    async fn get_device(&self, id: &str) -> Result<AudioDeviceInfo> {
        match id {
            "mem-input-001" => Ok(Self::make_default_device(
                "mem-input-001",
                "In-Memory Microphone",
                voxy_hardware::DeviceType::Microphone,
            )),
            "mem-output-001" => Ok(Self::make_default_device(
                "mem-output-001",
                "In-Memory Speaker",
                voxy_hardware::DeviceType::Speaker,
            )),
            _ => Err(crate::error::AudioError::DeviceNotFound(id.to_string())),
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_device_manager_initialize() {
        let manager = InMemoryDeviceManager::default();
        let config = AudioRuntimeConfig::default();
        assert!(!manager.is_initialized());
        manager.initialize(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_device_manager_list_inputs() {
        let manager = InMemoryDeviceManager::default();
        let config = AudioRuntimeConfig::default();
        manager.initialize(&config).await.unwrap();
        let inputs = manager.list_inputs().await.unwrap();
        assert!(!inputs.is_empty());
        assert_eq!(inputs[0].device_type, voxy_hardware::DeviceType::Microphone);
    }

    #[tokio::test]
    async fn test_in_memory_device_manager_list_outputs() {
        let manager = InMemoryDeviceManager::default();
        let config = AudioRuntimeConfig::default();
        manager.initialize(&config).await.unwrap();
        let outputs = manager.list_outputs().await.unwrap();
        assert!(!outputs.is_empty());
        assert_eq!(outputs[0].device_type, voxy_hardware::DeviceType::Speaker);
    }

    #[tokio::test]
    async fn test_in_memory_device_manager_defaults() {
        let manager = InMemoryDeviceManager::default();
        let config = AudioRuntimeConfig::default();
        manager.initialize(&config).await.unwrap();
        let input = manager.default_input().await.unwrap();
        assert_eq!(input.id, "mem-input-001");
        let output = manager.default_output().await.unwrap();
        assert_eq!(output.id, "mem-output-001");
    }

    #[tokio::test]
    async fn test_in_memory_device_manager_get_device() {
        let manager = InMemoryDeviceManager::default();
        let config = AudioRuntimeConfig::default();
        manager.initialize(&config).await.unwrap();
        let dev = manager.get_device("mem-input-001").await.unwrap();
        assert_eq!(dev.id, "mem-input-001");
    }

    #[tokio::test]
    async fn test_in_memory_device_manager_get_device_not_found() {
        let manager = InMemoryDeviceManager::default();
        let config = AudioRuntimeConfig::default();
        manager.initialize(&config).await.unwrap();
        let result = manager.get_device("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_audio_device_info_from_hardware() {
        let hw_info = voxy_hardware::DeviceInfo {
            id: "hw-001".to_string(),
            name: "Test Device".to_string(),
            device_type: voxy_hardware::DeviceType::Microphone,
            status: voxy_hardware::DeviceStatus::Available,
        };
        let info = AudioDeviceInfo::from_hardware(hw_info);
        assert_eq!(info.id, "hw-001");
        assert_eq!(info.name, "Test Device");
        assert_eq!(info.device_type, voxy_hardware::DeviceType::Microphone);
        assert_eq!(info.status, voxy_hardware::DeviceStatus::Available);
        assert!(!info.is_default);
        assert!(info.supported_sample_rates.contains(&44100));
    }
}
