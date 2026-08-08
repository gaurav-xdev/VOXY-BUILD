pub struct RuntimeConfig {
    pub worker_count: usize,
    pub task_queue_size: usize,
    pub default_timeout_seconds: u64,
    pub enable_task_tracing: bool,
    pub max_concurrent_tasks: usize,
    pub shutdown_timeout_seconds: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            task_queue_size: 1024,
            default_timeout_seconds: 30,
            enable_task_tracing: false,
            max_concurrent_tasks: 16,
            shutdown_timeout_seconds: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.worker_count, 4);
        assert_eq!(config.task_queue_size, 1024);
        assert_eq!(config.default_timeout_seconds, 30);
        assert!(!config.enable_task_tracing);
        assert_eq!(config.max_concurrent_tasks, 16);
        assert_eq!(config.shutdown_timeout_seconds, 10);
    }
}
