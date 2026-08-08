//! IPC message types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// IPC request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: Uuid,
    pub method: String,
    pub params: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

impl Request {
    /// Create a new request.
    pub fn new(method: impl Into<String>, params: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            method: method.into(),
            params,
            timestamp: Utc::now(),
        }
    }

    /// Create a request from JSON params.
    pub fn from_json<T: Serialize>(method: &str, data: &T) -> crate::Result<Self> {
        let params = serde_json::to_vec(data)?;
        Ok(Self::new(method, params))
    }

    /// Deserialize params from JSON.
    pub fn params_json<T: for<'de> Deserialize<'de>>(&self) -> crate::Result<T> {
        Ok(serde_json::from_slice(&self.params)?)
    }
}

/// IPC response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: Uuid,
    pub result: Option<Vec<u8>>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl Response {
    /// Create a success response.
    pub fn success(id: Uuid, result: Vec<u8>) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response.
    pub fn error(id: Uuid, error: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(error.into()),
            timestamp: Utc::now(),
        }
    }

    /// Create a response from JSON.
    pub fn from_json<T: Serialize>(id: Uuid, data: &T) -> crate::Result<Self> {
        let result = serde_json::to_vec(data)?;
        Ok(Self::success(id, result))
    }

    /// Deserialize result from JSON.
    pub fn result_json<T: for<'de> Deserialize<'de>>(&self) -> crate::Result<T> {
        let bytes = self
            .result
            .as_ref()
            .ok_or_else(|| crate::error::IPCError::InvalidRequest("No result".to_string()))?;
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Check if this is a success response.
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_creation() {
        let req = Request::new("echo", vec![1, 2, 3]);
        assert_eq!(req.method, "echo");
        assert_eq!(req.params, vec![1, 2, 3]);
    }

    #[test]
    fn response_success() {
        let id = Uuid::new_v4();
        let resp = Response::success(id, vec![4, 5, 6]);
        assert!(resp.is_success());
        assert_eq!(resp.result, Some(vec![4, 5, 6]));
    }

    #[test]
    fn response_error() {
        let id = Uuid::new_v4();
        let resp = Response::error(id, "something went wrong");
        assert!(!resp.is_success());
        assert_eq!(resp.error, Some("something went wrong".to_string()));
    }
}
