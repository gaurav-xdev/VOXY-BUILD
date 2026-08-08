#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub enable_preference_evolution: bool,
    pub enable_behavior_adaptation: bool,
    pub enable_reinforcement: bool,
    pub enable_confidence_calibration: bool,
    pub enable_strategy_evolution: bool,
    pub feedback_window_size: usize,
    pub min_feedback_for_adjustment: usize,
    pub learning_rate: f64,
    pub adaptation_cooldown_seconds: u64,
    pub max_preference_history: usize,
    pub calibration_samples_needed: usize,
    pub strategy_review_interval_seconds: u64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enable_preference_evolution: true,
            enable_behavior_adaptation: true,
            enable_reinforcement: true,
            enable_confidence_calibration: true,
            enable_strategy_evolution: true,
            feedback_window_size: 50,
            min_feedback_for_adjustment: 5,
            learning_rate: 0.1,
            adaptation_cooldown_seconds: 60,
            max_preference_history: 1000,
            calibration_samples_needed: 20,
            strategy_review_interval_seconds: 3600,
        }
    }
}
