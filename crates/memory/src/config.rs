#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub working_memory_capacity: usize,
    pub short_term_capacity: usize,
    pub short_term_ttl_seconds: u64,
    pub episodic_ttl_seconds: u64,
    pub consolidation_interval_seconds: u64,
    pub compression_threshold_days: u64,
    pub forgetting_check_interval_seconds: u64,
    pub max_vector_items: usize,
    pub importance_threshold_working: f64,
    pub importance_threshold_long_term: f64,
    pub enable_hermes: bool,
    pub enable_forgetting: bool,
    pub enable_consolidation: bool,
    pub enable_versioning: bool,
    pub graph_max_nodes: usize,
    pub graph_max_edges: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            working_memory_capacity: 10,
            short_term_capacity: 100,
            short_term_ttl_seconds: 3600,
            episodic_ttl_seconds: 86400,
            consolidation_interval_seconds: 300,
            compression_threshold_days: 7,
            forgetting_check_interval_seconds: 600,
            max_vector_items: 10000,
            importance_threshold_working: 0.3,
            importance_threshold_long_term: 0.7,
            enable_hermes: true,
            enable_forgetting: true,
            enable_consolidation: true,
            enable_versioning: true,
            graph_max_nodes: 50000,
            graph_max_edges: 200000,
        }
    }
}
