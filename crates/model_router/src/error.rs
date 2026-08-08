//! Model router error types.

use std::fmt;

/// Router error type.
#[derive(Debug)]
pub enum RouterError {
    NoProviderAvailable(String),
    RoutingFailed(String),
    ProviderError(String),
    AllProvidersExhausted(String),
    CircuitBreakerOpen(String),
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProviderAvailable(msg) => write!(f, "No provider available: {}", msg),
            Self::RoutingFailed(msg) => write!(f, "Routing failed: {}", msg),
            Self::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            Self::AllProvidersExhausted(msg) => write!(f, "All providers exhausted: {}", msg),
            Self::CircuitBreakerOpen(msg) => write!(f, "Circuit breaker open for: {}", msg),
        }
    }
}

impl std::error::Error for RouterError {}

pub type Result<T> = std::result::Result<T, RouterError>;
