//! IPC server implementation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Handler, IPCConfig, IPCError, Request, Response};

/// IPC server using named pipes.
pub struct IPCServer {
    config: IPCConfig,
    handlers: Arc<RwLock<HashMap<String, Box<dyn Handler>>>>,
    running: Arc<RwLock<bool>>,
}

impl IPCServer {
    /// Create a new IPC server.
    pub fn new(config: IPCConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a handler for a method.
    pub async fn register_handler(&self, method: &str, handler: Box<dyn Handler>) {
        self.handlers
            .write()
            .await
            .insert(method.to_string(), handler);
    }

    /// Start the server (stub — real implementation uses named pipes).
    pub async fn start(&self) -> crate::Result<()> {
        *self.running.write().await = true;
        tracing::info!("IPC server started on {}", self.config.pipe_name);
        Ok(())
    }

    /// Stop the server.
    pub async fn stop(&self) -> crate::Result<()> {
        *self.running.write().await = false;
        tracing::info!("IPC server stopped");
        Ok(())
    }

    /// Handle an incoming request.
    pub async fn handle_request(&self, request: Request) -> Result<Response, IPCError> {
        let handlers = self.handlers.read().await;
        let handler = handlers.get(&request.method).ok_or_else(|| {
            IPCError::InvalidRequest(format!("Unknown method: {}", request.method))
        })?;
        handler.handle(request).await
    }

    /// Check if the server is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

/// Echo handler for testing.
pub struct EchoHandler;

#[async_trait::async_trait]
impl Handler for EchoHandler {
    async fn handle(&self, request: Request) -> Result<Response, IPCError> {
        Ok(Response::success(request.id, request.params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn server_creation() {
        let server = IPCServer::new(IPCConfig::default());
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn server_start_stop() {
        let server = IPCServer::new(IPCConfig::default());
        server.start().await.unwrap();
        assert!(server.is_running().await);

        server.stop().await.unwrap();
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn handle_echo_request() {
        let server = IPCServer::new(IPCConfig::default());
        server.register_handler("echo", Box::new(EchoHandler)).await;
        server.start().await.unwrap();

        let request = Request::new("echo", vec![1, 2, 3]);
        let response = server.handle_request(request).await.unwrap();
        assert!(response.is_success());
        assert_eq!(response.result, Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn handle_unknown_method() {
        let server = IPCServer::new(IPCConfig::default());
        server.start().await.unwrap();

        let request = Request::new("unknown", vec![]);
        let result = server.handle_request(request).await;
        assert!(result.is_err());
    }
}
