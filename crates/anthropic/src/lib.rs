use async_trait::async_trait;
use std::time::Duration;

use voxy_provider_core::{LlmProvider, ProviderError, Result};

const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

pub struct AnthropicProvider {
    api_key: zeroize::Zeroizing<String>,
    base_url: String,
    model: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let timeout = Duration::from_secs(120);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to create HTTP client with custom config, using default");
                reqwest::Client::new()
            });
        Self {
            api_key: zeroize::Zeroizing::new(api_key.into()),
            base_url: "https://api.anthropic.com".into(),
            model: DEFAULT_MODEL.into(),
            timeout,
            client,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to create HTTP client with custom timeout, using default");
                reqwest::Client::new()
            });
        self
    }

    async fn send_message(&self, prompt: &str) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let url = format!("{}/v1/messages", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderError::RequestFailed("Anthropic request timed out".into())
                } else if e.is_connect() {
                    ProviderError::ConnectionFailed(format!(
                        "Cannot connect to Anthropic API at {}",
                        self.base_url
                    ))
                } else {
                    ProviderError::RequestFailed(e.to_string())
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 | 403 => {
                    ProviderError::AuthenticationFailed(format!("HTTP {}: {}", status, text))
                }
                429 => ProviderError::RateLimited,
                _ => ProviderError::RequestFailed(format!("HTTP {}: {}", status, text)),
            });
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let content = data["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                ProviderError::InvalidResponse("Missing text content in response".into())
            })?;

        Ok(content)
    }

    pub async fn health(&self) -> std::result::Result<bool, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| ProviderError::ConnectionFailed("Failed to build client".into()))?;

        let url = format!("{}/v1/messages", self.base_url);
        match client
            .post(&url)
            .header("x-api-key", self.api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": self.model,
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "ping"}]
            }))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => Ok(true),
            Ok(r) if r.status().as_u16() == 401 => Ok(false),
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, prompt: &str) -> Result<String> {
        self.send_message(prompt).await
    }

    fn available_models(&self) -> Vec<String> {
        vec![
            "claude-sonnet-4-20250514".into(),
            "claude-3-5-sonnet-20241022".into(),
            "claude-3-5-haiku-20241022".into(),
            "claude-3-opus-20240229".into(),
        ]
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("sk-ant-test");
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), DEFAULT_MODEL);
    }

    #[test]
    fn test_anthropic_with_custom_model() {
        let provider = AnthropicProvider::new("sk-ant-test").with_model("claude-3-opus-20240229");
        assert_eq!(provider.model(), "claude-3-opus-20240229");
    }

    #[test]
    fn test_anthropic_available_models() {
        let provider = AnthropicProvider::new("sk-ant-test");
        let models = provider.available_models();
        assert!(models.contains(&"claude-sonnet-4-20250514".to_string()));
        assert!(models.contains(&"claude-3-5-sonnet-20241022".to_string()));
    }

    #[tokio::test]
    async fn test_anthropic_complete_no_api_key() {
        let provider = AnthropicProvider::new("");
        let result = provider.complete("Hello").await;
        assert!(result.is_err());
    }
}
