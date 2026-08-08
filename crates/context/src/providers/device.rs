use crate::error::Result;
use crate::provider::ContextProvider;
use crate::types::{ContextPriority, ContextSnapshot, ContextSource};
use async_trait::async_trait;

/// Provides device context (hardware info, OS, capabilities).
pub struct DeviceContextProvider {
    device_id: String,
    device_name: String,
    os: String,
    os_version: String,
    arch: String,
    has_microphone: bool,
    has_speaker: bool,
    has_camera: bool,
    has_display: bool,
}

impl DeviceContextProvider {
    /// Create a new device context provider with system-detected values.
    pub fn new() -> Self {
        let info = os_info::get();
        Self {
            device_id: generate_device_id(),
            device_name: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            os: info.os_type().to_string(),
            os_version: info.version().to_string(),
            arch: std::env::consts::ARCH.to_string(),
            has_microphone: true,
            has_speaker: true,
            has_camera: false,
            has_display: true,
        }
    }

    /// Create with custom values (for testing).
    pub fn with_values(device_id: String, device_name: String, os: String) -> Self {
        Self {
            device_id,
            device_name,
            os,
            os_version: "unknown".to_string(),
            arch: std::env::consts::ARCH.to_string(),
            has_microphone: true,
            has_speaker: true,
            has_camera: false,
            has_display: true,
        }
    }

    /// Set device capabilities.
    pub fn set_capabilities(
        &mut self,
        microphone: bool,
        speaker: bool,
        camera: bool,
        display: bool,
    ) {
        self.has_microphone = microphone;
        self.has_speaker = speaker;
        self.has_camera = camera;
        self.has_display = display;
    }
}

impl Default for DeviceContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a stable device ID from hostname + OS.
fn generate_device_id() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    format!("{}-{}", hostname, std::env::consts::OS)
}

#[async_trait]
impl ContextProvider for DeviceContextProvider {
    fn name(&self) -> &str {
        "device"
    }

    fn source(&self) -> ContextSource {
        ContextSource::Device
    }

    fn default_priority(&self) -> ContextPriority {
        ContextPriority::Low
    }

    async fn collect(&self) -> Result<ContextSnapshot> {
        let data = serde_json::json!({
            "device_id": self.device_id,
            "device_name": self.device_name,
            "os": self.os,
            "os_version": self.os_version,
            "arch": self.arch,
            "capabilities": {
                "microphone": self.has_microphone,
                "speaker": self.has_speaker,
                "camera": self.has_camera,
                "display": self.has_display,
            },
        });

        Ok(ContextSnapshot::new(ContextSource::Device, data))
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_device_id() {
        let id = generate_device_id();
        assert!(!id.is_empty());
        assert!(id.contains(std::env::consts::OS));
    }

    #[tokio::test]
    async fn test_collect_device() {
        let provider = DeviceContextProvider::with_values(
            "test-device".to_string(),
            "Test Machine".to_string(),
            "linux".to_string(),
        );

        let snapshot = provider.collect().await.unwrap();
        assert_eq!(snapshot.source, ContextSource::Device);
        assert_eq!(snapshot.data["device_id"], "test-device");
        assert_eq!(snapshot.data["os"], "linux");
    }

    #[test]
    fn test_capabilities() {
        let mut provider = DeviceContextProvider::new();
        provider.set_capabilities(true, true, true, true);
        assert!(provider.has_camera);
    }
}
