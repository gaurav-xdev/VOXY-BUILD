use async_trait::async_trait;
use std::time::Duration;

use voxy_provider_core::{EmbeddingProvider, LlmProvider, ProviderError, Result};

pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new("http://127.0.0.1:11434", "llama3.2:3b").unwrap_or_else(|e| {
            tracing::warn!("Failed to create default Ollama client: {e}, using fallback");
            // Create a minimal provider without panicking
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to create HTTP client: {e}, using default");
                    reqwest::Client::new()
                });
            Self {
                base_url: "http://127.0.0.1:11434".into(),
                model: "llama3.2:3b".into(),
                client,
            }
        })
    }
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| {
                ProviderError::ConnectionFailed(format!("Failed to create HTTP client: {e}"))
            })?;
        Ok(Self {
            base_url: base_url.into(),
            model: model.into(),
            client,
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self> {
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| {
                ProviderError::ConnectionFailed(format!("Failed to create HTTP client: {e}"))
            })?;
        Ok(self)
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn chat_completion(&self, prompt: &str) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": false,
        });

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderError::RequestFailed("Ollama request timed out".into())
                } else if e.is_connect() {
                    ProviderError::ConnectionFailed(format!(
                        "Cannot connect to Ollama at {}. Is Ollama running?",
                        self.base_url
                    ))
                } else {
                    ProviderError::RequestFailed(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            return Err(ProviderError::RequestFailed(format!(
                "Ollama returned HTTP {}",
                resp.status()
            )));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        data["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                ProviderError::InvalidResponse("Missing content in Ollama response".into())
            })
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": text,
        });

        let url = format!("{}/api/embeddings", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        data["embedding"]
            .as_array()
            .ok_or_else(|| ProviderError::InvalidResponse("Missing embedding in response".into()))?
            .iter()
            .map(|v| {
                v.as_f64().map(|f| f as f32).ok_or_else(|| {
                    ProviderError::InvalidResponse("Non-numeric embedding value".into())
                })
            })
            .collect()
    }

    pub async fn list_local_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        Ok(data["models"]
            .as_array()
            .map(|models| {
                models
                    .iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn health(&self) -> std::result::Result<bool, ProviderError> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, prompt: &str) -> Result<String> {
        self.chat_completion(prompt).await
    }

    fn available_models(&self) -> Vec<String> {
        vec![self.model.clone()]
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.generate_embeddings(text).await
    }

    fn dimensions(&self) -> usize {
        4096
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_creation() {
        use voxy_provider_core::LlmProvider;
        let provider = OllamaProvider::default();
        assert_eq!(LlmProvider::name(&provider), "ollama");
        assert_eq!(provider.base_url(), "http://127.0.0.1:11434");
        assert_eq!(provider.model(), "llama3.2:3b");
    }

    #[test]
    fn test_ollama_with_custom_config() {
        let provider = OllamaProvider::new("http://localhost:11434", "mistral")
            .unwrap()
            .with_timeout(Duration::from_secs(30))
            .unwrap();
        assert_eq!(provider.base_url(), "http://localhost:11434");
        assert_eq!(provider.model(), "mistral");
    }

    #[tokio::test]
    async fn test_ollama_complete_connection_error() {
        let provider = OllamaProvider::new("http://127.0.0.1:1", "test-model").unwrap();
        let result = provider.complete("Hello").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ollama_health_no_server() {
        let provider = OllamaProvider::new("http://127.0.0.1:1", "test").unwrap();
        let healthy = provider.health().await.unwrap_or(false);
        assert!(!healthy);
    }

    #[tokio::test]
    async fn test_ollama_embedding_error() {
        let provider = OllamaProvider::new("http://127.0.0.1:1", "test").unwrap();
        let result = provider.embed("hello").await;
        assert!(result.is_err());
    }
}
