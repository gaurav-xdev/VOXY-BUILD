#[derive(Debug, Clone)]
pub struct CognitionConfig {
    pub max_goals_per_intent: usize,
    pub max_plan_steps: usize,
    pub max_reasoning_depth: usize,
    pub confidence_threshold: f64,
    pub require_validation: bool,
    pub enable_reflection: bool,
    pub enable_recovery: bool,
    pub max_recovery_attempts: u32,
    pub planning_timeout_seconds: u64,
    pub reasoning_timeout_seconds: u64,
    pub execution_timeout_seconds: u64,
}

impl Default for CognitionConfig {
    fn default() -> Self {
        Self {
            max_goals_per_intent: 10,
            max_plan_steps: 50,
            max_reasoning_depth: 5,
            confidence_threshold: 0.6,
            require_validation: true,
            enable_reflection: true,
            enable_recovery: true,
            max_recovery_attempts: 3,
            planning_timeout_seconds: 30,
            reasoning_timeout_seconds: 30,
            execution_timeout_seconds: 120,
        }
    }
}
