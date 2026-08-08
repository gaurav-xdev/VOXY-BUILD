use async_trait::async_trait;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::config::{DatabaseConfig, DatabaseKind};
use crate::error::{DatabaseError, Result};
use crate::storage::{
    AppliedMigration, HealthStatus, Migration, SearchResult, StorageProvider, Value,
};

#[derive(Debug, Clone)]
pub struct RemoteConfig {
    pub base_url: String,
    /// SECURITY: Auth token stored as Zeroizing to prevent memory exposure on drop.
    pub auth_token: Option<zeroize::Zeroizing<String>>,
    pub timeout: std::time::Duration,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".into(),
            auth_token: None,
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

pub struct RemoteDatabase {
    client: reqwest::Client,
    config: Mutex<Option<RemoteConfig>>,
    connected: AtomicBool,
}

impl fmt::Debug for RemoteDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteDatabase")
            .field("connected", &self.connected.load(Ordering::Relaxed))
            .finish()
    }
}

impl RemoteDatabase {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            config: Mutex::new(None),
            connected: AtomicBool::new(false),
        }
    }

    fn get_config(&self) -> Result<std::sync::MutexGuard<'_, Option<RemoteConfig>>> {
        let guard = self.config.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_some() {
            Ok(guard)
        } else {
            Err(DatabaseError::ConnectionFailed("Not configured".into()))
        }
    }

    fn cfg_base_url(&self) -> Result<String> {
        let guard = self.get_config()?;
        Ok(guard.as_ref().unwrap().base_url.clone())
    }

    fn cfg_auth_token(&self) -> Result<Option<String>> {
        let guard = self.get_config()?;
        Ok(guard
            .as_ref()
            .unwrap()
            .auth_token
            .as_ref()
            .map(|t| (**t).clone()))
    }

    fn cfg_timeout(&self) -> Result<std::time::Duration> {
        let guard = self.get_config()?;
        Ok(guard.as_ref().unwrap().timeout)
    }

    async fn build_get(&self, path: &str) -> Result<reqwest::Response> {
        let base_url = self.cfg_base_url()?;
        let timeout = self.cfg_timeout()?;
        let token = self.cfg_auth_token()?;
        let url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut req = self.client.get(&url).timeout(timeout);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        req.send()
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))
    }

    async fn build_put(&self, path: &str, body: Vec<u8>) -> Result<reqwest::Response> {
        let base_url = self.cfg_base_url()?;
        let timeout = self.cfg_timeout()?;
        let token = self.cfg_auth_token()?;
        let url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut req = self.client.put(&url).body(body).timeout(timeout);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        req.send()
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))
    }

    async fn build_delete(&self, path: &str) -> Result<reqwest::Response> {
        let base_url = self.cfg_base_url()?;
        let timeout = self.cfg_timeout()?;
        let token = self.cfg_auth_token()?;
        let url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut req = self.client.delete(&url).timeout(timeout);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        req.send()
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))
    }

    async fn build_post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response> {
        let base_url = self.cfg_base_url()?;
        let timeout = self.cfg_timeout()?;
        let token = self.cfg_auth_token()?;
        let url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut req = self.client.post(&url).json(body).timeout(timeout);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        req.send()
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))
    }

    fn check_response(resp: reqwest::Response) -> Result<reqwest::Response> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp)
        } else {
            let body = resp.text().unwrap_or_default();
            Err(DatabaseError::QueryFailed(format!(
                "HTTP {}: {}",
                status, body
            )))
        }
    }
}

impl Default for RemoteDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageProvider for RemoteDatabase {
    async fn connect(&self, config: &DatabaseConfig) -> Result<()> {
        let remote_config = RemoteConfig {
            base_url: config
                .remote_url
                .clone()
                .unwrap_or_else(|| "http://localhost:8080".into()),
            auth_token: None,
            timeout: std::time::Duration::from_secs(config.pool_size as u64 * 10),
        };

        let url = format!("{}/health", remote_config.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .get(&url)
            .timeout(remote_config.timeout)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                *self.config.lock().unwrap_or_else(|e| e.into_inner()) = Some(remote_config);
                self.connected.store(true, Ordering::SeqCst);
                Ok(())
            }
            Ok(r) => Err(DatabaseError::ConnectionFailed(format!(
                "HTTP {}",
                r.status()
            ))),
            Err(e) => Err(DatabaseError::ConnectionFailed(e.to_string())),
        }
    }

    async fn disconnect(&self) -> Result<()> {
        *self.config.lock().unwrap_or_else(|e| e.into_inner()) = None;
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn execute(&self, _query: &str, _params: &[Value]) -> Result<u64> {
        Err(DatabaseError::UnsupportedOperation(
            "Direct SQL execution not supported on remote backend".into(),
        ))
    }

    async fn query(&self, _query: &str, _params: &[Value]) -> Result<Vec<serde_json::Value>> {
        Err(DatabaseError::UnsupportedOperation(
            "Direct SQL queries not supported on remote backend".into(),
        ))
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let resp = self
            .build_put(&format!("kv/{}", key), value.to_vec())
            .await?;
        Self::check_response(resp).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let resp = self.build_get(&format!("kv/{}", key)).await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let resp = Self::check_response(resp).await?;
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(Some(bytes.to_vec()))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let resp = self.build_delete(&format!("kv/{}", key)).await?;
        Self::check_response(resp).await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let resp = self.build_get(&format!("kv/{}", key)).await?;
        Ok(resp.status().is_success())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let resp = self.build_get(&format!("kv?prefix={}", prefix)).await?;
        let resp = Self::check_response(resp).await?;
        let keys: Vec<String> = resp
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(keys)
    }

    async fn store_embedding(&self, id: &str, _embedding: &[f32], metadata: &str) -> Result<()> {
        let body = serde_json::json!({ "id": id, "metadata": metadata });
        let resp = self.build_post_json("embeddings", &body).await?;
        Self::check_response(resp).await?;
        Ok(())
    }

    async fn search_similar(&self, _embedding: &[f32], _limit: usize) -> Result<Vec<SearchResult>> {
        Err(DatabaseError::UnsupportedOperation(
            "Vector search not supported on remote backend".into(),
        ))
    }

    async fn delete_embedding(&self, id: &str) -> Result<()> {
        let resp = self.build_delete(&format!("embeddings/{}", id)).await?;
        Self::check_response(resp).await?;
        Ok(())
    }

    async fn run_migrations(&self, _migrations: &[Migration]) -> Result<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Migration management not supported on remote backend".into(),
        ))
    }

    async fn applied_migrations(&self) -> Result<Vec<AppliedMigration>> {
        Err(DatabaseError::UnsupportedOperation(
            "Migration management not supported on remote backend".into(),
        ))
    }

    async fn backup(&self, _path: &str) -> Result<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Backup not supported on remote backend".into(),
        ))
    }

    async fn restore(&self, _path: &str) -> Result<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Restore not supported on remote backend".into(),
        ))
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        let connected = self.is_connected();
        if !connected {
            return Ok(HealthStatus {
                connected: false,
                latency_ms: 0,
                version: None,
                storage_usage_bytes: None,
            });
        }
        match self.build_get("health").await {
            Ok(r) if r.status().is_success() => {
                let latency = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    connected: true,
                    latency_ms: latency,
                    version: None,
                    storage_usage_bytes: None,
                })
            }
            _ => Ok(HealthStatus {
                connected: false,
                latency_ms: start.elapsed().as_millis() as u64,
                version: None,
                storage_usage_bytes: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remote_new() {
        let db = RemoteDatabase::new();
        assert!(!db.is_connected());
    }

    #[tokio::test]
    async fn test_remote_disconnect() {
        let db = RemoteDatabase::new();
        db.disconnect().await.unwrap();
        assert!(!db.is_connected());
    }

    #[tokio::test]
    async fn test_remote_unsupported_execute() {
        let db = RemoteDatabase::new();
        let result = db.execute("SELECT 1", &[]).await;
        assert!(matches!(
            result,
            Err(DatabaseError::UnsupportedOperation(_))
        ));
    }

    #[tokio::test]
    async fn test_remote_unsupported_query() {
        let db = RemoteDatabase::new();
        let result = db.query("SELECT 1", &[]).await;
        assert!(matches!(
            result,
            Err(DatabaseError::UnsupportedOperation(_))
        ));
    }

    #[tokio::test]
    async fn test_remote_unsupported_search() {
        let db = RemoteDatabase::new();
        let result = db.search_similar(&[1.0, 0.0], 5).await;
        assert!(matches!(
            result,
            Err(DatabaseError::UnsupportedOperation(_))
        ));
    }

    #[tokio::test]
    async fn test_remote_connect_no_server() {
        let db = RemoteDatabase::new();
        let config = DatabaseConfig {
            kind: DatabaseKind::Remote,
            remote_url: Some("http://127.0.0.1:1".into()),
            ..Default::default()
        };
        let result = db.connect(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remote_health_disconnected() {
        let db = RemoteDatabase::new();
        let health = db.health_check().await.unwrap();
        assert!(!health.connected);
    }
}
