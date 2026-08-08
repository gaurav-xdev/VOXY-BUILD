//! macOS platform stub.

use async_trait::async_trait;
use voxy_platform_core::error::{PlatformError, Result};
use voxy_platform_core::traits::*;
use voxy_platform_core::types::*;

/// macOS platform stub.
pub struct MacOSPlatform;

impl Default for MacOSPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOSPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Platform for MacOSPlatform {
    fn info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "macos".to_string(),
            arch: std::env::consts::ARCH.to_string(),
            version: "unknown".to_string(),
            hostname: None,
        }
    }

    fn name(&self) -> &str {
        "macos"
    }

    async fn initialize(&mut self) -> Result<()> {
        Err(PlatformError::UnsupportedPlatform("macos".to_string()))
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_platform_info() {
        let platform = MacOSPlatform::new();
        assert_eq!(platform.name(), "macos");
    }
}
