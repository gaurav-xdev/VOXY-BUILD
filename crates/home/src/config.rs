#[derive(Debug, Clone)]
pub struct HomeConfig {
    pub home_id: String,
    pub home_name: String,
    pub location: String,
    pub timezone: String,
    pub default_environment_id: String,
    pub auto_discover_devices: bool,
    pub scan_interval_seconds: u64,
}

impl Default for HomeConfig {
    fn default() -> Self {
        Self {
            home_id: "home-001".into(),
            home_name: "My Home".into(),
            location: "Unknown".into(),
            timezone: "UTC".into(),
            default_environment_id: "default".into(),
            auto_discover_devices: true,
            scan_interval_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_config_defaults() {
        let config = HomeConfig::default();
        assert_eq!(config.home_id, "home-001");
        assert_eq!(config.home_name, "My Home");
        assert_eq!(config.location, "Unknown");
        assert_eq!(config.timezone, "UTC");
        assert_eq!(config.default_environment_id, "default");
        assert!(config.auto_discover_devices);
        assert_eq!(config.scan_interval_seconds, 60);
    }
}
