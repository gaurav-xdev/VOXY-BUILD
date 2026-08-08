use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub check_interval_ms: u64,
    pub enable_watchdog: bool,
    pub enable_memory_monitoring: bool,
    pub enable_cpu_monitoring: bool,
    pub enable_event_bus_monitoring: bool,
    pub enable_ipc_monitoring: bool,
    pub enable_db_monitoring: bool,
    pub watchdog_interval_ms: u64,
    pub auto_recovery: bool,
    pub degradation_threshold: u32,
    pub failure_threshold: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 30000,
            enable_watchdog: true,
            enable_memory_monitoring: true,
            enable_cpu_monitoring: true,
            enable_event_bus_monitoring: true,
            enable_ipc_monitoring: false,
            enable_db_monitoring: false,
            watchdog_interval_ms: 60000,
            auto_recovery: true,
            degradation_threshold: 3,
            failure_threshold: 5,
        }
    }
}

impl HealthConfig {
    pub fn validate(&self) -> Result<(), crate::HealthError> {
        if self.check_interval_ms == 0 {
            return Err(crate::HealthError::InvalidConfig(
                "check_interval_ms must be > 0".into(),
            ));
        }
        if self.watchdog_interval_ms == 0 {
            return Err(crate::HealthError::InvalidConfig(
                "watchdog_interval_ms must be > 0".into(),
            ));
        }
        if self.degradation_threshold == 0 {
            return Err(crate::HealthError::InvalidConfig(
                "degradation_threshold must be > 0".into(),
            ));
        }
        if self.failure_threshold == 0 {
            return Err(crate::HealthError::InvalidConfig(
                "failure_threshold must be > 0".into(),
            ));
        }
        if self.failure_threshold < self.degradation_threshold {
            return Err(crate::HealthError::InvalidConfig(
                "failure_threshold must be >= degradation_threshold".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval_ms, 30000);
        assert_eq!(config.watchdog_interval_ms, 60000);
        assert!(config.enable_watchdog);
        assert!(config.enable_memory_monitoring);
        assert!(config.auto_recovery);
        assert_eq!(config.degradation_threshold, 3);
        assert_eq!(config.failure_threshold, 5);
    }

    #[test]
    fn config_validation_ok() {
        let config = HealthConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_validation_fails_zero_interval() {
        let config = HealthConfig {
            check_interval_ms: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validation_fails_zero_watchdog() {
        let config = HealthConfig {
            watchdog_interval_ms: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validation_fails_threshold_mismatch() {
        let config = HealthConfig {
            degradation_threshold: 5,
            failure_threshold: 3,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = HealthConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HealthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.check_interval_ms, config.check_interval_ms);
    }
}
