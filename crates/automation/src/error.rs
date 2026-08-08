use std::fmt;

use voxy_orchestrator::OrchestratorError;

#[derive(Debug)]
pub enum AutomationError {
    ActionFailed(String),
    ElementNotFound(String),
    BackendUnavailable(String),
    VerificationFailed(String),
    Timeout(String),
    InitializationFailed(String),
    UnsupportedOperation(String),
    InvalidConfiguration(String),
    PlatformError(String),
    DpiMismatch { expected: u32, actual: u32 },
    MultiMonitorError(String),
    RollbackFailed(String),
    Cancelled(String),
    OcrFailed(String),
}

impl fmt::Display for AutomationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActionFailed(msg) => write!(f, "Action failed: {}", msg),
            Self::ElementNotFound(msg) => write!(f, "Element not found: {}", msg),
            Self::BackendUnavailable(msg) => write!(f, "Backend unavailable: {}", msg),
            Self::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            Self::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::PlatformError(msg) => write!(f, "Platform error: {}", msg),
            Self::DpiMismatch { expected, actual } => {
                write!(f, "DPI mismatch: expected {}, actual {}", expected, actual)
            }
            Self::MultiMonitorError(msg) => write!(f, "Multi-monitor error: {}", msg),
            Self::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            Self::Cancelled(msg) => write!(f, "Cancelled: {}", msg),
            Self::OcrFailed(msg) => write!(f, "OCR failed: {}", msg),
        }
    }
}

impl std::error::Error for AutomationError {}

impl From<AutomationError> for OrchestratorError {
    fn from(e: AutomationError) -> Self {
        OrchestratorError::AutomationError(e.to_string())
    }
}

pub fn action_err(msg: impl Into<String>) -> OrchestratorError {
    OrchestratorError::AutomationError(format!("Action failed: {}", msg.into()))
}

pub fn not_found_err(msg: impl Into<String>) -> OrchestratorError {
    OrchestratorError::AutomationError(format!("Not found: {}", msg.into()))
}

pub fn unavail_err(msg: impl Into<String>) -> OrchestratorError {
    OrchestratorError::AutomationError(format!("Unavailable: {}", msg.into()))
}

pub fn timeout_err(msg: impl Into<String>) -> OrchestratorError {
    OrchestratorError::AutomationError(format!("Timeout: {}", msg.into()))
}

pub fn unsupported_err(msg: impl Into<String>) -> OrchestratorError {
    OrchestratorError::AutomationError(format!("Unsupported: {}", msg.into()))
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;
