use std::fmt;

#[derive(Debug)]
pub enum IpcError {
    ConnectionFailed(String),
    ConnectionClosed(String),
    Timeout { operation: String, duration_ms: u64 },
    ProtocolViolation(String),
    InvalidFrame(String),
    Serialization(String),
    Deserialization(String),
    AuthFailed(String),
    CapabilityDenied { capability: String, reason: String },
    VersionMismatch { local: String, remote: String },
    StreamClosed(String),
    StreamNotFound(String),
    Cancelled { reason: String },
    TransportError(String),
    CodecError(String),
    ListenerError(String),
    NotConnected,
    AlreadyConnected,
    ResourceExhausted(String),
    Internal(String),
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            Self::ConnectionClosed(msg) => write!(f, "Connection closed: {msg}"),
            Self::Timeout {
                operation,
                duration_ms,
            } => {
                write!(f, "Timeout on {operation} after {duration_ms}ms")
            }
            Self::ProtocolViolation(msg) => write!(f, "Protocol violation: {msg}"),
            Self::InvalidFrame(msg) => write!(f, "Invalid frame: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Deserialization(msg) => write!(f, "Deserialization error: {msg}"),
            Self::AuthFailed(msg) => write!(f, "Authentication failed: {msg}"),
            Self::CapabilityDenied { capability, reason } => {
                write!(f, "Capability '{capability}' denied: {reason}")
            }
            Self::VersionMismatch { local, remote } => {
                write!(f, "Version mismatch: local={local}, remote={remote}")
            }
            Self::StreamClosed(msg) => write!(f, "Stream closed: {msg}"),
            Self::StreamNotFound(msg) => write!(f, "Stream not found: {msg}"),
            Self::Cancelled { reason } => write!(f, "Cancelled: {reason}"),
            Self::TransportError(msg) => write!(f, "Transport error: {msg}"),
            Self::CodecError(msg) => write!(f, "Codec error: {msg}"),
            Self::ListenerError(msg) => write!(f, "Listener error: {msg}"),
            Self::NotConnected => write!(f, "Not connected"),
            Self::AlreadyConnected => write!(f, "Already connected"),
            Self::ResourceExhausted(msg) => write!(f, "Resource exhausted: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for IpcError {}

impl From<std::io::Error> for IpcError {
    fn from(e: std::io::Error) -> Self {
        Self::TransportError(e.to_string())
    }
}

impl From<serde_json::Error> for IpcError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, IpcError>;
