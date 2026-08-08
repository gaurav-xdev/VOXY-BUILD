use crate::fusion::policy::FusionPolicy;
use crate::types::{ContextSnapshot, ContextSource, FreshnessConfig};
use std::collections::HashMap;

/// Status of a context snapshot's freshness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FreshnessStatus {
    /// Fresh and valid.
    Fresh,
    /// Approaching staleness (within 20% of TTL).
    Aging,
    /// Past TTL but within grace period.
    Stale,
    /// Past grace period, should be invalidated.
    Expired,
    /// Never refreshed since initialization.
    Unknown,
}

impl FreshnessStatus {
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Fresh | Self::Aging | Self::Stale)
    }
}

/// Manages TTL, decay, and staleness detection for context snapshots.
pub struct FreshnessEngine {
    freshness_config: FreshnessConfig,
    #[allow(dead_code)]
    policy: FusionPolicy,
    /// Grace period in seconds past TTL before a snapshot is considered expired.
    grace_period: u64,
}

impl FreshnessEngine {
    pub fn new(freshness_config: FreshnessConfig, policy: FusionPolicy) -> Self {
        Self {
            freshness_config,
            policy,
            grace_period: 30,
        }
    }

    pub fn with_grace_period(mut self, grace_secs: u64) -> Self {
        self.grace_period = grace_secs;
        self
    }

    /// Determine the freshness status of a snapshot.
    pub fn status(&self, snapshot: &ContextSnapshot) -> FreshnessStatus {
        let max_age = self.max_age_for(&snapshot.source);
        let age = snapshot.freshness;

        if age == 0 && snapshot.confidence == 1.0 {
            return FreshnessStatus::Unknown;
        }

        if age <= max_age {
            let remaining = max_age - age;
            let threshold = (max_age as f64 * 0.2) as u64;
            if remaining <= threshold {
                FreshnessStatus::Aging
            } else {
                FreshnessStatus::Fresh
            }
        } else if age <= max_age + self.grace_period {
            FreshnessStatus::Stale
        } else {
            FreshnessStatus::Expired
        }
    }

    /// Compute a decay multiplier (1.0 = fresh, approaches 0.0 as age increases).
    pub fn decay_factor(&self, snapshot: &ContextSnapshot) -> f64 {
        let max_age = self.max_age_for(&snapshot.source);
        if max_age == 0 || snapshot.freshness == 0 {
            return 1.0;
        }

        let ratio = snapshot.freshness as f64 / max_age as f64;
        if ratio <= 1.0 {
            1.0
        } else {
            // Exponential decay past TTL
            (-3.0 * (ratio - 1.0)).exp().max(0.0)
        }
    }

    /// Check if a snapshot needs refreshing.
    pub fn needs_refresh(&self, snapshot: &ContextSnapshot) -> bool {
        matches!(
            self.status(snapshot),
            FreshnessStatus::Aging | FreshnessStatus::Stale
        )
    }

    /// Check if a snapshot should be invalidated.
    pub fn should_invalidate(&self, snapshot: &ContextSnapshot) -> bool {
        self.status(snapshot) == FreshnessStatus::Expired
    }

    /// Filter out expired snapshots from a collection.
    pub fn filter_valid<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Vec<&'a ContextSnapshot> {
        snapshots
            .iter()
            .filter(|s| self.status(s).is_usable())
            .collect()
    }

    /// Identify which sources need refreshing.
    pub fn sources_needing_refresh<'a>(
        &self,
        snapshots: &'a [ContextSnapshot],
    ) -> Vec<&'a ContextSnapshot> {
        snapshots.iter().filter(|s| self.needs_refresh(s)).collect()
    }

    /// Get a summary of freshness across all provided snapshots.
    pub fn freshness_summary(
        &self,
        snapshots: &[ContextSnapshot],
    ) -> HashMap<ContextSource, FreshnessStatus> {
        snapshots
            .iter()
            .map(|s| (s.source.clone(), self.status(s)))
            .collect()
    }

    fn max_age_for(&self, source: &ContextSource) -> u64 {
        self.freshness_config.max_age_for(source)
    }
}

impl Default for FreshnessEngine {
    fn default() -> Self {
        Self::new(FreshnessConfig::default(), FusionPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContextSource;

    fn make_snapshot(source: ContextSource, freshness: u64) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source, serde_json::json!({"test": true}));
        s.freshness = freshness;
        s.confidence = 0.8;
        s
    }

    #[test]
    fn test_fresh_snapshot() {
        let engine = FreshnessEngine::default();
        let snap = make_snapshot(ContextSource::Environment, 10);
        assert_eq!(engine.status(&snap), FreshnessStatus::Fresh);
        assert!(engine.status(&snap).is_usable());
    }

    #[test]
    fn test_aging_snapshot() {
        let engine = FreshnessEngine::default();
        // Environment max_age = 60, so 50 is within 20% threshold
        let snap = make_snapshot(ContextSource::Environment, 50);
        assert_eq!(engine.status(&snap), FreshnessStatus::Aging);
    }

    #[test]
    fn test_stale_snapshot() {
        let engine = FreshnessEngine::default();
        // Environment max_age = 60, stale = 61-90
        let snap = make_snapshot(ContextSource::Environment, 70);
        assert_eq!(engine.status(&snap), FreshnessStatus::Stale);
        assert!(engine.status(&snap).is_usable());
    }

    #[test]
    fn test_expired_snapshot() {
        let engine = FreshnessEngine::default();
        // Environment max_age = 60, expired = >90
        let snap = make_snapshot(ContextSource::Environment, 100);
        assert_eq!(engine.status(&snap), FreshnessStatus::Expired);
        assert!(!engine.status(&snap).is_usable());
    }

    #[test]
    fn test_decay_factor() {
        let engine = FreshnessEngine::default();
        let fresh = make_snapshot(ContextSource::Environment, 0);
        let aging = make_snapshot(ContextSource::Environment, 50);
        let stale = make_snapshot(ContextSource::Environment, 80);
        assert_eq!(engine.decay_factor(&fresh), 1.0);
        assert!(engine.decay_factor(&aging) >= 0.8);
        assert!(engine.decay_factor(&stale) < 0.6);
    }

    #[test]
    fn test_needs_refresh() {
        let engine = FreshnessEngine::default();
        let fresh = make_snapshot(ContextSource::Environment, 10);
        let aging = make_snapshot(ContextSource::Environment, 50);
        assert!(!engine.needs_refresh(&fresh));
        assert!(engine.needs_refresh(&aging));
    }

    #[test]
    fn test_should_invalidate() {
        let engine = FreshnessEngine::default();
        let ok = make_snapshot(ContextSource::Environment, 10);
        let expired = make_snapshot(ContextSource::Environment, 100);
        assert!(!engine.should_invalidate(&ok));
        assert!(engine.should_invalidate(&expired));
    }

    #[test]
    fn test_filter_valid() {
        let engine = FreshnessEngine::default();
        let snaps = vec![
            make_snapshot(ContextSource::Environment, 10),
            make_snapshot(ContextSource::Environment, 100),
        ];
        let valid = engine.filter_valid(&snaps);
        assert_eq!(valid.len(), 1);
    }
}
