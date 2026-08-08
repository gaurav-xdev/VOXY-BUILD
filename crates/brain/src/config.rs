use std::time::Duration;

use voxy_companion::config::CompanionConfig;
use voxy_companion::CompanionPersonality;
use voxy_context::ManagerConfig;
use voxy_human_dynamics::config::HdrConfig;

#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub context: ManagerConfig,
    pub companion: CompanionConfig,
    pub companion_personality: CompanionPersonality,
    pub human_dynamics: HdrConfig,
    pub pipeline_timeout: Duration,
    pub max_concurrent_sessions: usize,
    pub enable_streaming: bool,
    pub enable_telemetry: bool,
    pub health_check_interval: Duration,
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            context: ManagerConfig::default(),
            companion: CompanionConfig::default(),
            companion_personality: CompanionPersonality::default(),
            human_dynamics: HdrConfig::default(),
            pipeline_timeout: Duration::from_secs(30),
            max_concurrent_sessions: 10,
            enable_streaming: true,
            enable_telemetry: true,
            health_check_interval: Duration::from_secs(30),
        }
    }
}
