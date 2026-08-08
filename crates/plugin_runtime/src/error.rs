//! Plugin error types.

use std::fmt;

/// Plugin error type.
#[derive(Debug)]
pub enum PluginError {
    NotFound(String),
    LoadFailed(String),
    ExecutionFailed(String),
    PermissionDenied(String),
    Timeout,
    Crashed(String),
    IPCFailed(String),
    InvalidManifest(String),
    DependencyMissing(String),
    VersionMismatch(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Plugin not found: {}", msg),
            Self::LoadFailed(msg) => write!(f, "Load failed: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::Timeout => write!(f, "Plugin timeout"),
            Self::Crashed(msg) => write!(f, "Plugin crashed: {}", msg),
            Self::IPCFailed(msg) => write!(f, "IPC failed: {}", msg),
            Self::InvalidManifest(msg) => write!(f, "Invalid manifest: {}", msg),
            Self::DependencyMissing(msg) => write!(f, "Dependency missing: {}", msg),
            Self::VersionMismatch(msg) => write!(f, "Version mismatch: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

pub type Result<T> = std::result::Result<T, PluginError>;
