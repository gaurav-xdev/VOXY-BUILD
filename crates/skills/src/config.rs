#[derive(Debug, Clone)]
pub struct SkillsConfig {
    pub max_concurrent_skills: usize,
    pub skill_timeout_seconds: u64,
    pub enable_capability_discovery: bool,
    pub enable_skill_caching: bool,
}

impl Default for SkillsConfig {
    fn default() -> Self {
        Self {
            max_concurrent_skills: 10,
            skill_timeout_seconds: 30,
            enable_capability_discovery: true,
            enable_skill_caching: false,
        }
    }
}
