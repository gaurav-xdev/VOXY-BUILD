use crate::error::Result;
use crate::types::{ReinforcementSignal, StrategyEffectiveness};
use async_trait::async_trait;

#[async_trait]
pub trait StrategyEvolution: Send + Sync {
    async fn evaluate_strategy(&self, strategy_id: &str) -> Result<StrategyEffectiveness>;
    async fn evolve_strategy(
        &self,
        current: &StrategyEffectiveness,
        signal: &ReinforcementSignal,
    ) -> Result<StrategyEffectiveness>;
    async fn compare_strategies(&self, strategies: &[String]) -> Result<Vec<(String, f64)>>;
    async fn get_best_strategy(&self, context: &str) -> Result<Option<String>>;
    async fn record_trial(&self, strategy_id: &str, success: bool, duration_ms: f64) -> Result<()>;
}
