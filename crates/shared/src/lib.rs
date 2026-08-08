//! Common types, traits, error model, and events for VOXY.
//!
//! This crate provides the foundational types that all other VOXY crates depend on.
//! It should have zero dependencies on other VOXY crates.

pub mod error;
pub mod event;
pub mod production;
pub mod providers;
pub mod traits;
pub mod types;
pub mod version;

pub use error::{ErrorKind, Result, Severity, VoxyError};
pub use event::{Event, Priority, TypedEvent};
pub use production::{ErrorContext, RetryPolicy};
pub use providers::{
    AuthContext, AuthProvider, AuthResult, BillingProvider, Credentials, LocalAuthProvider,
    OfflineBillingProvider, OfflineSyncProvider, Subscription, SubscriptionStatus, SyncConflict,
    SyncProvider, TokenPair, TrustLevel, UsageStats,
};
pub use traits::{Configurable, HealthStatus, Lifecycle};
pub use types::Rect;
pub use version::{BuildInfo, VersionInfo};

/// Get the current VOXY version.
pub fn version() -> VersionInfo {
    version::version()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_correct() {
        let v = version();
        assert_eq!(v.major(), 0);
        assert_eq!(v.minor(), 1);
        assert_eq!(v.patch(), 0);
    }
}
