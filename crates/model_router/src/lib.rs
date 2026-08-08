pub mod error;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use voxy_provider_core::{
    CapabilityDiscovery, DefaultCapabilityDiscovery, EmbeddingProvider, LlmProvider,
    ProviderCapability, ProviderInfo, ProviderRegistry,
};

pub use error::{Result, RouterError};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
struct BreakerEntry {
    state: BreakerState,
    failures: u32,
    last_failure: Instant,
    opened_at: Instant,
}

impl BreakerEntry {
    fn new() -> Self {
        Self {
            state: BreakerState::Closed,
            failures: 0,
            last_failure: Instant::now(),
            opened_at: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum RoutingMode {
    #[default]
    Auto,
    LocalOnly,
    CloudOnly,
    ForceModel(String),
    ForceProvider(String),
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub mode: RoutingMode,
    pub local_first: bool,
    pub priority_order: Vec<String>,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_reset_secs: u64,
    pub provider_timeout_secs: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            mode: RoutingMode::Auto,
            local_first: true,
            priority_order: Vec::new(),
            max_retries: 3,
            retry_delay_ms: 200,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_secs: 30,
            provider_timeout_secs: 15,
        }
    }
}

pub struct ModelRouter {
    registry: Arc<dyn ProviderRegistry>,
    config: parking_lot::RwLock<RouterConfig>,
    breakers: parking_lot::RwLock<HashMap<String, BreakerEntry>>,
    last_health_check: AtomicU64,
}

impl ModelRouter {
    pub fn new(registry: Arc<dyn ProviderRegistry>) -> Self {
        Self {
            registry,
            config: parking_lot::RwLock::new(RouterConfig::default()),
            breakers: parking_lot::RwLock::new(HashMap::new()),
            last_health_check: AtomicU64::new(0),
        }
    }

    pub fn with_config(registry: Arc<dyn ProviderRegistry>, config: RouterConfig) -> Self {
        Self {
            registry,
            config: parking_lot::RwLock::new(config),
            breakers: parking_lot::RwLock::new(HashMap::new()),
            last_health_check: AtomicU64::new(0),
        }
    }

    pub fn config(&self) -> RouterConfig {
        self.config.read().clone()
    }

    pub fn set_config(&self, config: RouterConfig) {
        *self.config.write() = config;
    }

    pub fn set_mode(&self, mode: RoutingMode) {
        self.config.write().mode = mode;
    }

    fn is_breaker_open(&self, provider_id: &str) -> bool {
        let reset_secs = {
            let config = self.config.read();
            config.circuit_breaker_reset_secs
        };
        let reset = Duration::from_secs(reset_secs);
        let breakers = self.breakers.read();
        if let Some(entry) = breakers.get(provider_id) {
            match entry.state {
                BreakerState::Open => {
                    if entry.opened_at.elapsed() > reset {
                        drop(breakers);
                        let mut breakers = self.breakers.write();
                        if let Some(e) = breakers.get_mut(provider_id) {
                            if e.state == BreakerState::Open {
                                e.state = BreakerState::HalfOpen;
                            }
                        }
                        return false;
                    }
                    true
                }
                BreakerState::HalfOpen => false,
                BreakerState::Closed => false,
            }
        } else {
            false
        }
    }

    fn record_failure(&self, provider_id: &str) {
        let mut breakers = self.breakers.write();
        let config = self.config.read();
        let entry = breakers
            .entry(provider_id.to_string())
            .or_insert_with(BreakerEntry::new);
        entry.failures += 1;
        entry.last_failure = Instant::now();
        if entry.failures >= config.circuit_breaker_threshold {
            entry.state = BreakerState::Open;
            entry.opened_at = Instant::now();
        }
    }

    fn record_success(&self, provider_id: &str) {
        let mut breakers = self.breakers.write();
        if let Some(entry) = breakers.get_mut(provider_id) {
            entry.state = BreakerState::Closed;
            entry.failures = 0;
        }
    }

    async fn refresh_health(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let last = self.last_health_check.load(Ordering::Relaxed);
        if now - last < 60 {
            return;
        }
        self.last_health_check.store(now, Ordering::Relaxed);
        if let Ok(providers) = self.registry.list_all().await {
            for p in &providers {
                if p.health.is_healthy {
                    self.record_success(&p.id);
                }
            }
        }
    }

    fn filter_healthy(&self, providers: Vec<ProviderInfo>) -> Vec<ProviderInfo> {
        providers
            .into_iter()
            .filter(|p| {
                let cb_open = self.is_breaker_open(&p.id);
                let unhealthy = !p.health.is_healthy
                    || !matches!(p.status, voxy_provider_core::ProviderStatus::Available);
                !cb_open && !unhealthy
            })
            .collect()
    }

    async fn select_fallback(&self, providers: Vec<ProviderInfo>) -> Result<ProviderInfo> {
        for p in &providers {
            if !self.is_breaker_open(&p.id) {
                return Ok(p.clone());
            }
        }
        Err(RouterError::AllProvidersExhausted(
            "all providers are circuit-broken or unhealthy".into(),
        ))
    }

    pub async fn select_provider(&self, capability: &ProviderCapability) -> Result<ProviderInfo> {
        self.refresh_health().await;
        let config_mode = { self.config.read().mode.clone() };
        let providers = self
            .registry
            .find_by_capability(capability.clone())
            .await
            .map_err(|e| RouterError::RoutingFailed(e.to_string()))?;

        if providers.is_empty() {
            return Err(RouterError::NoProviderAvailable(format!(
                "No provider found for {:?}",
                capability
            )));
        }

        let best = match &config_mode {
            RoutingMode::ForceModel(model_id) => {
                let mut best = None;
                for p in &providers {
                    if p.models.iter().any(|m| m.id == *model_id) {
                        best = Some(p.clone());
                        break;
                    }
                }
                best.ok_or_else(|| {
                    RouterError::NoProviderAvailable(format!(
                        "Model '{}' not found in any provider",
                        model_id
                    ))
                })
            }
            RoutingMode::ForceProvider(provider_id) => providers
                .into_iter()
                .find(|p| p.id == *provider_id)
                .ok_or_else(|| {
                    RouterError::NoProviderAvailable(format!(
                        "Provider '{}' not found",
                        provider_id
                    ))
                }),
            RoutingMode::LocalOnly => {
                let local: Vec<ProviderInfo> = providers
                    .into_iter()
                    .filter(|p| matches!(p.kind, voxy_provider_core::ProviderKind::Local))
                    .collect();
                if local.is_empty() {
                    Err(RouterError::NoProviderAvailable(
                        "No local provider available".into(),
                    ))
                } else {
                    Ok(local.into_iter().next().unwrap())
                }
            }
            RoutingMode::CloudOnly => {
                let cloud: Vec<ProviderInfo> = providers
                    .into_iter()
                    .filter(|p| matches!(p.kind, voxy_provider_core::ProviderKind::Cloud))
                    .collect();
                if cloud.is_empty() {
                    Err(RouterError::NoProviderAvailable(
                        "No cloud provider available".into(),
                    ))
                } else {
                    Ok(cloud.into_iter().next().unwrap())
                }
            }
            RoutingMode::Auto => {
                let discovery = DefaultCapabilityDiscovery::new(providers);
                if let Some(best_match) = discovery.find_best_match(capability) {
                    let id = best_match.provider_id.clone();
                    self.registry
                        .get(&id)
                        .await
                        .map_err(|e| RouterError::RoutingFailed(e.to_string()))
                } else {
                    Err(RouterError::NoProviderAvailable(format!(
                        "No suitable provider for {:?}",
                        capability
                    )))
                }
            }
        };

        let mut provider = match best {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let all_providers = self
            .registry
            .find_by_capability(capability.clone())
            .await
            .unwrap_or_default();
        let all_healthy = self.filter_healthy(all_providers);

        let cfg = self.config.read().clone();
        let max_retries = cfg.max_retries;
        let retry_delay_ms = cfg.retry_delay_ms;
        drop(cfg);

        for attempt in 0..max_retries.saturating_add(1) {
            if self.is_breaker_open(&provider.id) {
                provider = match self.select_fallback(all_healthy.clone()).await {
                    Ok(p) => p,
                    Err(e) => {
                        if attempt < max_retries {
                            tokio::time::sleep(Duration::from_millis(
                                retry_delay_ms * (1u64 << attempt),
                            ))
                            .await;
                            continue;
                        }
                        return Err(e);
                    }
                };
            }

            let result = self
                .registry
                .get(&provider.id)
                .await
                .map_err(|e| RouterError::RoutingFailed(e.to_string()));

            match result {
                Ok(p) => {
                    self.record_success(&p.id);
                    return Ok(p);
                }
                Err(e) => {
                    self.record_failure(&provider.id);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(
                            retry_delay_ms * (1u64 << attempt),
                        ))
                        .await;
                        provider = match self.select_fallback(all_healthy.clone()).await {
                            Ok(p) => p,
                            Err(fallback_err) => {
                                if attempt >= max_retries.saturating_sub(1) {
                                    return Err(fallback_err);
                                }
                                continue;
                            }
                        };
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(RouterError::AllProvidersExhausted(
            "retries exhausted".into(),
        ))
    }

    pub async fn complete(&self, provider: &dyn LlmProvider, prompt: &str) -> Result<String> {
        let timeout = {
            let config = self.config.read();
            Duration::from_secs(config.provider_timeout_secs)
        };
        let result = tokio::time::timeout(timeout, provider.complete(prompt)).await;
        match result {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => Err(RouterError::ProviderError(format!(
                "LLM completion failed: {}",
                e
            ))),
            Err(_) => Err(RouterError::ProviderError(
                "LLM completion timed out".into(),
            )),
        }
    }

    pub async fn embed(&self, provider: &dyn EmbeddingProvider, text: &str) -> Result<Vec<f32>> {
        let timeout = {
            let config = self.config.read();
            Duration::from_secs(config.provider_timeout_secs)
        };
        let result = tokio::time::timeout(timeout, provider.embed(text)).await;
        match result {
            Ok(Ok(vec)) => Ok(vec),
            Ok(Err(e)) => Err(RouterError::ProviderError(format!(
                "Embedding failed: {}",
                e
            ))),
            Err(_) => Err(RouterError::ProviderError("Embedding timed out".into())),
        }
    }

    pub fn registry(&self) -> &Arc<dyn ProviderRegistry> {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxy_provider_core::{
        DefaultProviderRegistry, ModelInfo, ProviderHealth, ProviderKind, ProviderStatus,
    };

    fn make_provider(
        id: &str,
        kind: ProviderKind,
        cap: ProviderCapability,
        status: ProviderStatus,
        priority: u32,
    ) -> ProviderInfo {
        ProviderInfo {
            id: id.to_string(),
            name: id.to_string(),
            kind,
            capability: cap,
            status,
            models: vec![ModelInfo::new(id, id)],
            health: ProviderHealth {
                is_healthy: true,
                last_check: chrono::Utc::now(),
                latency_ms: Some(10.0),
                details: None,
            },
            base_url: None,
            priority,
        }
    }

    #[tokio::test]
    async fn test_router_selects_local_first() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        registry
            .register(make_provider(
                "cloud-llm",
                ProviderKind::Cloud,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                0,
            ))
            .await
            .unwrap();
        registry
            .register(make_provider(
                "local-llm",
                ProviderKind::Local,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                10,
            ))
            .await
            .unwrap();

        let router = ModelRouter::new(registry);
        router.set_mode(RoutingMode::Auto);
        let selected = router
            .select_provider(&ProviderCapability::Llm)
            .await
            .unwrap();
        assert_eq!(selected.id, "local-llm");
    }

    #[tokio::test]
    async fn test_router_force_model() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        registry
            .register(make_provider(
                "ollama",
                ProviderKind::Local,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                0,
            ))
            .await
            .unwrap();

        let router = ModelRouter::new(registry);
        router.set_mode(RoutingMode::ForceModel("ollama".into()));
        let selected = router.select_provider(&ProviderCapability::Llm).await;
        assert!(selected.is_ok());
    }

    #[tokio::test]
    async fn test_router_force_model_not_found() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        registry
            .register(make_provider(
                "ollama",
                ProviderKind::Local,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                0,
            ))
            .await
            .unwrap();

        let router = ModelRouter::new(registry);
        router.set_mode(RoutingMode::ForceModel("nonexistent".into()));
        let result = router.select_provider(&ProviderCapability::Llm).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_router_local_only() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        registry
            .register(make_provider(
                "local-llm",
                ProviderKind::Local,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                0,
            ))
            .await
            .unwrap();

        let router = ModelRouter::new(registry);
        router.set_mode(RoutingMode::LocalOnly);
        let selected = router
            .select_provider(&ProviderCapability::Llm)
            .await
            .unwrap();
        assert_eq!(selected.id, "local-llm");
    }

    #[tokio::test]
    async fn test_router_local_only_no_local() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        registry
            .register(make_provider(
                "cloud-llm",
                ProviderKind::Cloud,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                0,
            ))
            .await
            .unwrap();

        let router = ModelRouter::new(registry);
        router.set_mode(RoutingMode::LocalOnly);
        let result = router.select_provider(&ProviderCapability::Llm).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_router_cloud_only() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        registry
            .register(make_provider(
                "cloud-llm",
                ProviderKind::Cloud,
                ProviderCapability::Llm,
                ProviderStatus::Available,
                0,
            ))
            .await
            .unwrap();

        let router = ModelRouter::new(registry);
        router.set_mode(RoutingMode::CloudOnly);
        let selected = router
            .select_provider(&ProviderCapability::Llm)
            .await
            .unwrap();
        assert_eq!(selected.id, "cloud-llm");
    }

    #[tokio::test]
    async fn test_router_no_providers() {
        let registry = Arc::new(DefaultProviderRegistry::new());
        let router = ModelRouter::new(registry);
        let result = router.select_provider(&ProviderCapability::Llm).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_routing_mode_default() {
        assert!(matches!(RoutingMode::default(), RoutingMode::Auto));
    }

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert!(config.local_first);
        assert!(matches!(config.mode, RoutingMode::Auto));
    }
}
