//! Linux platform stub.

use async_trait::async_trait;
use voxy_platform_core::error::{PlatformError, Result};
use voxy_platform_core::traits::*;
use voxy_platform_core::types::*;

/// Linux platform stub.
pub struct LinuxPlatform;

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Platform for LinuxPlatform {
    fn info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "linux".to_string(),
            arch: std::env::consts::ARCH.to_string(),
            version: "unknown".to_string(),
            hostname: None,
        }
    }

    fn name(&self) -> &str {
        "linux"
    }

    async fn initialize(&mut self) -> Result<()> {
        Err(PlatformError::UnsupportedPlatform("linux".to_string()))
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_platform_info() {
        let platform = LinuxPlatform::new();
        assert_eq!(platform.name(), "linux");
    }
}
