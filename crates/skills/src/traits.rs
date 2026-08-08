use std::collections::HashMap;

use async_trait::async_trait;

use crate::capabilities::CapabilityId;
use crate::config::SkillsConfig;
use crate::error::Result;
use crate::types::SkillId;

#[async_trait]
pub trait SkillContext: Send + Sync {
    fn world_model(&self) -> Option<&dyn voxy_world_model::traits::WorldModelProvider>;
    fn personality(&self) -> Option<&dyn voxy_personality::traits::PersonalityProfile>;
    fn config(&self) -> &SkillsConfig;
}

pub struct SkillInput {
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: Box<dyn SkillContext>,
}

pub struct SkillOutput {
    pub result: serde_json::Value,
    pub confidence: Option<f64>,
    pub duration_ms: u64,
}

#[async_trait]
pub trait Skill: Send + Sync {
    fn skill_id(&self) -> &SkillId;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn tags(&self) -> Vec<String>;
    fn required_capabilities(&self) -> Vec<CapabilityId>;
    fn input_schema(&self) -> Option<serde_json::Value>;
    fn output_schema(&self) -> Option<serde_json::Value>;
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput>;
    async fn dry_run(&self, input: &SkillInput) -> Result<SkillOutput>;
    fn max_duration_seconds(&self) -> Option<u64>;
    fn is_long_running(&self) -> bool;
}

#[async_trait]
pub trait SkillRegistry: Send + Sync {
    async fn register(&self, skill: Box<dyn Skill>) -> Result<SkillId>;
    async fn unregister(&self, skill_id: &SkillId) -> Result<()>;
    async fn get(&self, skill_id: &SkillId) -> Result<Box<dyn Skill>>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Box<dyn Skill>>>;
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Box<dyn Skill>>>;
    async fn find_by_capability(&self, capability_id: &CapabilityId)
        -> Result<Vec<Box<dyn Skill>>>;
    async fn list_skills(&self) -> Result<Vec<Box<dyn Skill>>>;
    async fn execute(&self, skill_id: &SkillId, input: SkillInput) -> Result<SkillOutput>;
    async fn skill_count(&self) -> usize;
}
