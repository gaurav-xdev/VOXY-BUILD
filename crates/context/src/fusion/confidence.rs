use crate::fusion::policy::FusionPolicy;
use crate::types::{ContextPriority, ContextSnapshot};

/// Computes and adjusts confidence scores for context snapshots.
pub struct ConfidenceEngine {
    policy: FusionPolicy,
}

impl ConfidenceEngine {
    pub fn new(policy: FusionPolicy) -> Self {
        Self { policy }
    }

    /// Compute effective confidence for a snapshot, considering source weight and freshness.
    pub fn effective_confidence(&self, snapshot: &ContextSnapshot) -> f64 {
        let base = snapshot.confidence;
        let weight = self.policy.weight_for(&snapshot.source);
        let freshness_factor = self.freshness_factor(snapshot);
        let priority_bonus = self.priority_bonus(&snapshot.priority);

        let raw = (base * weight * freshness_factor) + priority_bonus;
        raw.clamp(0.0, 1.0)
    }

    /// Check if a snapshot meets the confidence floor for its source.
    pub fn meets_confidence_floor(&self, snapshot: &ContextSnapshot) -> bool {
        let floor = self.policy.confidence_floor_for(&snapshot.source);
        snapshot.confidence >= floor
    }

    /// Compute a composite score for conflict resolution.
    pub fn composite_score(&self, snapshot: &ContextSnapshot) -> f64 {
        let effective = self.effective_confidence(snapshot);
        let priority_score = self.priority_score(&snapshot.priority);
        let relevance = snapshot.relevance;

        // Weighted combination for conflict resolution
        (effective * 0.5) + (priority_score * 0.3) + (relevance * 0.2)
    }

    /// Freshness factor: decays confidence as the snapshot ages.
    fn freshness_factor(&self, snapshot: &ContextSnapshot) -> f64 {
        let max_age = self
            .policy
            .policy_for(&snapshot.source)
            .max_staleness
            .unwrap_or(300);

        if snapshot.freshness == 0 {
            return 1.0;
        }

        let ratio = snapshot.freshness as f64 / max_age as f64;
        // Exponential decay: older = lower factor
        (-ratio * 2.0).exp().max(0.1)
    }

    /// Priority bonus: higher priority sources get a small confidence boost.
    fn priority_bonus(&self, priority: &ContextPriority) -> f64 {
        match priority {
            ContextPriority::Critical => 0.15,
            ContextPriority::High => 0.10,
            ContextPriority::Medium => 0.05,
            ContextPriority::Low => 0.0,
            ContextPriority::Background => -0.05,
        }
    }

    /// Numeric priority score for compositing.
    fn priority_score(&self, priority: &ContextPriority) -> f64 {
        match priority {
            ContextPriority::Critical => 1.0,
            ContextPriority::High => 0.75,
            ContextPriority::Medium => 0.5,
            ContextPriority::Low => 0.25,
            ContextPriority::Background => 0.1,
        }
    }

    /// Filter snapshots that don't meet the confidence floor.
    pub fn filter_by_confidence<'a>(
        &self,
        snapshots: &'a [ContextSnapshot],
    ) -> Vec<&'a ContextSnapshot> {
        snapshots
            .iter()
            .filter(|s| self.meets_confidence_floor(s))
            .collect()
    }

    /// Rank snapshots by composite score (highest first).
    pub fn rank_by_score(snapshots: &[ContextSnapshot]) -> Vec<(usize, f64)> {
        let mut indexed: Vec<(usize, f64)> = snapshots
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.score()))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed
    }
}

impl Default for ConfidenceEngine {
    fn default() -> Self {
        Self::new(FusionPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContextSource;

    fn make_snapshot(source: ContextSource, confidence: f64, freshness: u64) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source, serde_json::json!({"test": true}));
        s.confidence = confidence;
        s.freshness = freshness;
        s
    }

    #[test]
    fn test_effective_confidence_fresh() {
        let engine = ConfidenceEngine::default();
        let snap = make_snapshot(ContextSource::Environment, 0.8, 0);
        let eff = engine.effective_confidence(&snap);
        assert!((0.7..=1.0).contains(&eff));
    }

    #[test]
    fn test_effective_confidence_stale() {
        let engine = ConfidenceEngine::default();
        let snap_fresh = make_snapshot(ContextSource::Environment, 0.8, 0);
        let snap_stale = make_snapshot(ContextSource::Environment, 0.8, 300);
        let eff_fresh = engine.effective_confidence(&snap_fresh);
        let eff_stale = engine.effective_confidence(&snap_stale);
        assert!(eff_fresh > eff_stale);
    }

    #[test]
    fn test_meets_confidence_floor() {
        let engine = ConfidenceEngine::default();
        let high = make_snapshot(ContextSource::Environment, 0.9, 0);
        let low = make_snapshot(ContextSource::Environment, 0.05, 0);
        assert!(engine.meets_confidence_floor(&high));
        assert!(!engine.meets_confidence_floor(&low));
    }

    #[test]
    fn test_composite_score_ordering() {
        let engine = ConfidenceEngine::default();
        let a = make_snapshot(ContextSource::Environment, 0.9, 0);
        let b = make_snapshot(ContextSource::Environment, 0.3, 0);
        assert!(engine.composite_score(&a) > engine.composite_score(&b));
    }

    #[test]
    fn test_filter_by_confidence() {
        let engine = ConfidenceEngine::default();
        let snapshots = vec![
            make_snapshot(ContextSource::Environment, 0.9, 0),
            make_snapshot(ContextSource::Environment, 0.05, 0),
        ];
        let filtered = engine.filter_by_confidence(&snapshots);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_rank_by_score() {
        let snapshots = vec![
            make_snapshot(ContextSource::Environment, 0.3, 100),
            make_snapshot(ContextSource::Environment, 0.9, 0),
            make_snapshot(ContextSource::Environment, 0.6, 50),
        ];
        let ranked = ConfidenceEngine::rank_by_score(&snapshots);
        assert_eq!(ranked[0].0, 1); // 0.9 confidence wins
    }

    #[test]
    fn test_priority_bonus() {
        let engine = ConfidenceEngine::default();
        let mut crit = make_snapshot(ContextSource::SystemState, 0.8, 0);
        crit.priority = ContextPriority::Critical;
        let mut low = make_snapshot(ContextSource::Environment, 0.8, 0);
        low.priority = ContextPriority::Low;
        assert!(engine.effective_confidence(&crit) > engine.effective_confidence(&low));
    }
}
