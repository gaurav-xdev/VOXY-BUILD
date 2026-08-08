//! Stable public APIs for plugins, providers, and external integrations.

pub mod error;

pub use error::{Result, SdkError};

/// SDK version.
pub const SDK_VERSION: &str = "0.1.0";

/// SDK entry point for plugin authors.
pub struct Sdk;

impl Sdk {
    pub fn version() -> &'static str {
        SDK_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_version() {
        assert_eq!(Sdk::version(), "0.1.0");
    }
}
