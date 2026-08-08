use crate::fusion::freshness::{FreshnessEngine, FreshnessStatus};
use crate::types::{ContextSnapshot, ContextSource};
use std::collections::HashMap;

/// Result of an invalidation check.
#[derive(Debug, Clone)]
pub struct InvalidationResult {
    /// Sources that were invalidated.
    pub invalidated: Vec<ContextSource>,

    /// Sources that need refreshing.
    pub needs_refresh: Vec<ContextSource>,

    /// Sources that are still valid.
    pub valid: Vec<ContextSource>,

    /// Snapshot of freshness statuses.
    pub statuses: HashMap<ContextSource, FreshnessStatus>,
}

/// Handles stale data invalidation across context sources.
pub struct ContextInvalidation {
    freshness_engine: FreshnessEngine,
    /// Track how many times a source has been flagged for invalidation.
    invalidation_counts: HashMap<ContextSource, u32>,
    /// Maximum consecutive invalidations before source is marked degraded.
    max_consecutive_invalidations: u32,
}

impl ContextInvalidation {
    pub fn new(freshness_engine: FreshnessEngine) -> Self {
        Self {
            freshness_engine,
            invalidation_counts: HashMap::new(),
            max_consecutive_invalidations: 3,
        }
    }

    pub fn with_max_consecutive(mut self, max: u32) -> Self {
        self.max_consecutive_invalidations = max;
        self
    }

    /// Check all snapshots and return invalidation results.
    pub fn check(&self, snapshots: &[ContextSnapshot]) -> InvalidationResult {
        let mut invalidated = Vec::new();
        let mut needs_refresh = Vec::new();
        let mut valid = Vec::new();
        let mut statuses = HashMap::new();

        for snapshot in snapshots {
            let status = self.freshness_engine.status(snapshot);
            statuses.insert(snapshot.source.clone(), status.clone());

            match status {
                FreshnessStatus::Expired => {
                    invalidated.push(snapshot.source.clone());
                }
                FreshnessStatus::Stale | FreshnessStatus::Aging => {
                    needs_refresh.push(snapshot.source.clone());
                }
                FreshnessStatus::Fresh | FreshnessStatus::Unknown => {
                    valid.push(snapshot.source.clone());
                }
            }
        }

        InvalidationResult {
            invalidated,
            needs_refresh,
            valid,
            statuses,
        }
    }

    /// Record that a source was invalidated (for consecutive tracking).
    pub fn record_invalidation(&mut self, source: &ContextSource) {
        let count = self.invalidation_counts.entry(source.clone()).or_insert(0);
        *count += 1;
    }

    /// Clear invalidation count for a source (e.g., after successful refresh).
    pub fn clear_invalidation(&mut self, source: &ContextSource) {
        self.invalidation_counts.remove(source);
    }

    /// Check if a source is degraded (too many consecutive invalidations).
    pub fn is_degraded(&self, source: &ContextSource) -> bool {
        self.invalidation_counts
            .get(source)
            .map(|&count| count >= self.max_consecutive_invalidations)
            .unwrap_or(false)
    }

    /// Get the invalidation count for a source.
    pub fn invalidation_count(&self, source: &ContextSource) -> u32 {
        self.invalidation_counts.get(source).copied().unwrap_or(0)
    }

    /// Get all degraded sources.
    pub fn degraded_sources(&self) -> Vec<&ContextSource> {
        self.invalidation_counts
            .iter()
            .filter(|(_, &count)| count >= self.max_consecutive_invalidations)
            .map(|(source, _)| source)
            .collect()
    }

    /// Filter snapshots, removing expired and degraded sources.
    pub fn filter_valid_snapshots<'a>(
        &self,
        snapshots: &'a [ContextSnapshot],
    ) -> Vec<&'a ContextSnapshot> {
        snapshots
            .iter()
            .filter(|s| {
                let status = self.freshness_engine.status(s);
                status.is_usable() && !self.is_degraded(&s.source)
            })
            .collect()
    }
}

impl Default for ContextInvalidation {
    fn default() -> Self {
        Self::new(FreshnessEngine::default())
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
    fn test_check_fresh() {
        let invalidator = ContextInvalidation::default();
        let snaps = vec![make_snapshot(ContextSource::Environment, 10)];
        let result = invalidator.check(&snaps);
        assert!(result.invalidated.is_empty());
        assert!(result.needs_refresh.is_empty());
        assert_eq!(result.valid.len(), 1);
    }

    #[test]
    fn test_check_expired() {
        let invalidator = ContextInvalidation::default();
        let snaps = vec![make_snapshot(ContextSource::Environment, 200)];
        let result = invalidator.check(&snaps);
        assert_eq!(result.invalidated.len(), 1);
    }

    #[test]
    fn test_check_needs_refresh() {
        let invalidator = ContextInvalidation::default();
        let snaps = vec![make_snapshot(ContextSource::Environment, 55)];
        let result = invalidator.check(&snaps);
        assert_eq!(result.needs_refresh.len(), 1);
    }

    #[test]
    fn test_consecutive_invalidation_tracking() {
        let mut invalidator = ContextInvalidation::default();
        let source = ContextSource::Environment;

        assert!(!invalidator.is_degraded(&source));

        invalidator.record_invalidation(&source);
        invalidator.record_invalidation(&source);
        assert!(!invalidator.is_degraded(&source));

        invalidator.record_invalidation(&source);
        assert!(invalidator.is_degraded(&source));
    }

    #[test]
    fn test_clear_invalidation() {
        let mut invalidator = ContextInvalidation::default();
        let source = ContextSource::Environment;

        invalidator.record_invalidation(&source);
        invalidator.record_invalidation(&source);
        invalidator.record_invalidation(&source);
        assert!(invalidator.is_degraded(&source));

        invalidator.clear_invalidation(&source);
        assert!(!invalidator.is_degraded(&source));
    }

    #[test]
    fn test_filter_valid_snapshots() {
        let invalidator = ContextInvalidation::default();
        let snaps = vec![
            make_snapshot(ContextSource::Environment, 10),
            make_snapshot(ContextSource::Activity, 200),
        ];
        let valid = invalidator.filter_valid_snapshots(&snaps);
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].source, ContextSource::Environment);
    }
}
