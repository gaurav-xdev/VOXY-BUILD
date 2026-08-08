#[derive(Debug, Clone)]
pub struct WorldModelConfig {
    pub enable_desktop_monitoring: bool,
    pub desktop_poll_interval_ms: u64,
    pub enable_device_monitoring: bool,
    pub device_poll_interval_ms: u64,
    pub max_active_tasks: usize,
    pub enable_environment_tracking: bool,
}

impl Default for WorldModelConfig {
    fn default() -> Self {
        Self {
            enable_desktop_monitoring: true,
            desktop_poll_interval_ms: 1000,
            enable_device_monitoring: true,
            device_poll_interval_ms: 5000,
            max_active_tasks: 20,
            enable_environment_tracking: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_model_config_default() {
        let cfg = WorldModelConfig::default();
        assert!(cfg.enable_desktop_monitoring);
        assert_eq!(cfg.desktop_poll_interval_ms, 1000);
        assert!(cfg.enable_device_monitoring);
        assert_eq!(cfg.device_poll_interval_ms, 5000);
        assert_eq!(cfg.max_active_tasks, 20);
        assert!(cfg.enable_environment_tracking);
    }
}
