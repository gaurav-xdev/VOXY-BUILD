//! Platform abstraction traits and types for cross-platform support.
//!
//! This crate defines the interfaces that platform-specific implementations must provide.
//! No Windows/Linux/macOS API calls should appear outside platform crates.
//!
//! Platform detection and instantiation should be done in the consuming crate
//! (e.g., daemon) using the platform-specific crates directly.

pub mod error;
pub mod traits;
pub mod types;

pub use error::{PlatformError, Result};
pub use traits::{
    AudioPlatform, DisplayPlatform, FileSystemPlatform, InputPlatform, NetworkPlatform, Platform,
    ProcessPlatform, WindowPlatform,
};
pub use types::{
    AudioDevice, DisplayInfo, FileInfo, NetworkInfo, PlatformInfo, ProcessInfo, WindowInfo,
};

/// Platform detection and initialization.
pub struct PlatformRegistry {
    platform: Box<dyn Platform>,
}

impl PlatformRegistry {
    /// Create a platform registry with the given platform implementation.
    pub fn new(platform: Box<dyn Platform>) -> Self {
        Self { platform }
    }

    /// Get the current platform.
    pub fn platform(&self) -> &dyn Platform {
        self.platform.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_registry_creation() {
        // Platform detection depends on the OS running the test
        // This test validates the interface exists
        let _ = std::any::type_name::<PlatformRegistry>();
    }
}
