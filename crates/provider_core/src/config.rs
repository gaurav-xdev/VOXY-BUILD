use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub timeout: Duration,
    pub retry: RetryConfig,
    pub max_concurrent_requests: u32,
    pub enable_health_checks: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry: RetryConfig::default(),
            max_concurrent_requests: 10,
            enable_health_checks: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_retries, 3);
        assert_eq!(cfg.base_delay_ms, 100);
    }

    #[test]
    fn test_provider_config_default() {
        let cfg = ProviderConfig::default();
        assert_eq!(cfg.timeout, Duration::from_secs(30));
        assert_eq!(cfg.max_concurrent_requests, 10);
        assert!(cfg.enable_health_checks);
    }

    #[test]
    fn test_provider_config_custom() {
        let cfg = ProviderConfig {
            timeout: Duration::from_secs(60),
            retry: RetryConfig {
                max_retries: 5,
                ..Default::default()
            },
            max_concurrent_requests: 20,
            enable_health_checks: false,
        };
        assert_eq!(cfg.timeout, Duration::from_secs(60));
        assert_eq!(cfg.retry.max_retries, 5);
        assert!(!cfg.enable_health_checks);
    }
}
