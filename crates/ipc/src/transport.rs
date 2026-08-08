use crate::error::Result;
use crate::protocol::Frame;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransportAddr {
    NamedPipe(String),
    UnixSocket(String),
    Tcp { host: String, port: u16 },
    WebSocket { url: String },
    Quic { host: String, port: u16 },
    InMemory(String),
    Custom(String),
}

impl std::fmt::Display for TransportAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NamedPipe(name) => write!(f, "pipe://{name}"),
            Self::UnixSocket(path) => write!(f, "unix://{path}"),
            Self::Tcp { host, port } => write!(f, "tcp://{host}:{port}"),
            Self::WebSocket { url } => write!(f, "ws://{url}"),
            Self::Quic { host, port } => write!(f, "quic://{host}:{port}"),
            Self::InMemory(id) => write!(f, "memory://{id}"),
            Self::Custom(s) => write!(f, "custom://{s}"),
        }
    }
}

#[async_trait]
pub trait Transport: Send + Sync {
    fn addr(&self) -> &TransportAddr;
    async fn connect(&self) -> Result<Box<dyn TransportConnection>>;
    async fn bind(&self) -> Result<Box<dyn TransportListener>>;
    fn max_frame_size(&self) -> usize;
    fn is_reliable(&self) -> bool;
    fn latency_hint_ms(&self) -> u32;
    fn supports_keepalive(&self) -> bool;
}

#[async_trait]
pub trait TransportConnection: Send + Sync {
    async fn send_frame(&self, frame: Frame) -> Result<()>;
    async fn receive_frame(&self) -> Result<Frame>;
    async fn close(&self) -> Result<()>;
    fn is_open(&self) -> bool;
}

#[async_trait]
pub trait TransportListener: Send + Sync {
    async fn accept(&self) -> Result<Box<dyn TransportConnection>>;
    async fn close(&self) -> Result<()>;
    fn local_addr(&self) -> &TransportAddr;
}

#[async_trait]
pub trait WireCodec: Send + Sync {
    fn encode(&self, frame: &Frame) -> Result<Vec<u8>>;
    fn decode(&self, bytes: &[u8]) -> Result<Frame>;
}

pub struct JsonCodec;

impl JsonCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WireCodec for JsonCodec {
    fn encode(&self, frame: &Frame) -> Result<Vec<u8>> {
        serde_json::to_vec(frame).map_err(|e| crate::error::IpcError::Serialization(e.to_string()))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Frame> {
        const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;
        if bytes.len() > MAX_FRAME_SIZE {
            return Err(crate::error::IpcError::Deserialization(format!(
                "Frame too large: {} bytes (max {})",
                bytes.len(),
                MAX_FRAME_SIZE
            )));
        }
        serde_json::from_slice(bytes)
            .map_err(|e| crate::error::IpcError::Deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::MessageType;

    #[test]
    fn transport_addr_display() {
        assert_eq!(
            TransportAddr::Tcp {
                host: "localhost".to_string(),
                port: 9000
            }
            .to_string(),
            "tcp://localhost:9000"
        );
        assert_eq!(
            TransportAddr::NamedPipe("voxy".to_string()).to_string(),
            "pipe://voxy"
        );
        assert_eq!(
            TransportAddr::InMemory("test".to_string()).to_string(),
            "memory://test"
        );
    }

    #[test]
    fn json_codec_roundtrip() {
        let codec = JsonCodec::new();
        let frame = Frame::new(MessageType::Heartbeat);
        let bytes = codec.encode(&frame).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        assert_eq!(frame.message_type, decoded.message_type);
    }

    #[test]
    fn json_codec_with_payload() {
        let codec = JsonCodec::new();
        let frame = Frame::new(MessageType::MethodCall).with_payload(vec![1, 2, 3, 4]);
        let bytes = codec.encode(&frame).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn transport_addr_equality() {
        let a = TransportAddr::Tcp {
            host: "127.0.0.1".to_string(),
            port: 8080,
        };
        let b = TransportAddr::Tcp {
            host: "127.0.0.1".to_string(),
            port: 8080,
        };
        let c = TransportAddr::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9090,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn transport_addr_custom() {
        let addr = TransportAddr::Custom("my-transport".to_string());
        assert_eq!(addr.to_string(), "custom://my-transport");
    }
}
