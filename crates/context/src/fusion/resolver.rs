use crate::fusion::assembled::{AssembledContext, AssembledContextBuilder};
use crate::fusion::confidence::ConfidenceEngine;
use crate::fusion::conflict::ContextConflictResolver;
use crate::fusion::delta::ContextDeltaGenerator;
use crate::fusion::freshness::FreshnessEngine;
use crate::fusion::invalidation::ContextInvalidation;
use crate::fusion::merger::ContextMerger;
use crate::fusion::policy::FusionPolicy;
use crate::fusion::priority::ContextPriorityResolver;
use crate::types::{ContextSnapshot, ContextSource, FreshnessConfig};
use parking_lot::RwLock;
use std::collections::HashMap;

/// Statistics about a fusion cycle.
#[derive(Debug, Clone, Default)]
pub struct FusionStats {
    /// Number of snapshots that went into this fusion.
    pub input_count: usize,

    /// Number of sources included in the output.
    pub included_count: usize,

    /// Number of sources excluded.
    pub excluded_count: usize,

    /// Number of conflicts detected.
    pub conflicts_detected: usize,

    /// Number of conflicts resolved.
    pub conflicts_resolved: usize,

    /// Fusion time in microseconds.
    pub fusion_time_us: u64,

    /// Size of the assembled context in bytes.
    pub output_size_bytes: usize,
}

/// The main context fusion engine — receives snapshots from all providers,
/// merges them, resolves conflicts, and produces one coherent AssembledContext.
pub struct ContextFusionEngine {
    policy: FusionPolicy,
    confidence_engine: ConfidenceEngine,
    freshness_engine: FreshnessEngine,
    priority_resolver: ContextPriorityResolver,
    conflict_resolver: ContextConflictResolver,
    merger: ContextMerger,
    invalidation: RwLock<ContextInvalidation>,
    delta_generator: ContextDeltaGenerator,
    /// Previous assembled context for delta generation.
    previous_context: RwLock<Option<AssembledContext>>,
    /// Previous source snapshots for delta generation.
    previous_snapshots: RwLock<Option<HashMap<ContextSource, ContextSnapshot>>>,
    /// Accumulated stats.
    stats: RwLock<FusionStats>,
}

impl ContextFusionEngine {
    pub fn new(policy: FusionPolicy) -> Self {
        let freshness_config = FreshnessConfig::default();
        let confidence_engine = ConfidenceEngine::new(policy.clone());
        let freshness_engine = FreshnessEngine::new(freshness_config, policy.clone());
        let priority_resolver = ContextPriorityResolver::new();
        let conflict_resolver = ContextConflictResolver::new(policy.clone());
        let merger = ContextMerger::new(policy.clone());
        let invalidation = ContextInvalidation::new(freshness_engine.clone_with_grace(30));
        let delta_generator = ContextDeltaGenerator::new();

        Self {
            policy,
            confidence_engine,
            freshness_engine,
            priority_resolver,
            conflict_resolver,
            merger,
            invalidation: RwLock::new(invalidation),
            delta_generator,
            previous_context: RwLock::new(None),
            previous_snapshots: RwLock::new(None),
            stats: RwLock::new(FusionStats::default()),
        }
    }

    /// Create with default policy.
    pub fn with_defaults() -> Self {
        Self::new(FusionPolicy::default())
    }

    /// Perform a fusion cycle: take raw snapshots, produce an assembled context.
    pub fn fuse(&self, mut snapshots: Vec<ContextSnapshot>) -> AssembledContext {
        let start = std::time::Instant::now();

        let input_count = snapshots.len();

        // Step 1: Filter out snapshots that don't meet confidence floor
        let mut valid_snapshots: Vec<ContextSnapshot> = snapshots
            .drain(..)
            .filter(|s| self.confidence_engine.meets_confidence_floor(s))
            .collect();

        // Step 2: Check freshness and invalidate expired sources
        let invalidation_result = {
            let invalidator = self.invalidation.read();
            invalidator.check(&valid_snapshots)
        };

        // Remove expired snapshots
        let expired_sources: std::collections::HashSet<&ContextSource> =
            invalidation_result.invalidated.iter().collect();

        valid_snapshots.retain(|s| !expired_sources.contains(&s.source));

        // Step 3: Sort by priority hierarchy
        self.priority_resolver
            .sort_by_priority(&mut valid_snapshots);

        // Step 4: Detect and resolve conflicts
        let mut conflicts_detected = 0;
        let mut conflicts_resolved = 0;

        // Group snapshots by potential conflict (same data keys from different sources)
        if valid_snapshots.len() > 1 {
            for i in 0..valid_snapshots.len() {
                for j in (i + 1)..valid_snapshots.len() {
                    let conflicts = self
                        .conflict_resolver
                        .detect_conflicts(&valid_snapshots[i], &valid_snapshots[j]);
                    conflicts_detected += conflicts.len();
                    conflicts_resolved += conflicts.len();
                }
            }
        }

        // Step 5: Merge all valid snapshots
        let merged_data = self.merger.merge(&valid_snapshots);

        // Step 6: Build the assembled context
        let mut builder = AssembledContextBuilder::new().with_data(merged_data);

        for snapshot in &valid_snapshots {
            builder = builder.add_source(snapshot.clone());
        }

        for source in &invalidation_result.invalidated {
            builder = builder.exclude_source(source.clone());
        }

        let assembled = builder.build();

        // Step 7: Generate deltas if we have a previous context
        {
            let current_snapshots: HashMap<ContextSource, ContextSnapshot> = valid_snapshots
                .into_iter()
                .map(|s| (s.source.clone(), s))
                .collect();

            let previous = self.previous_snapshots.read();
            if let Some(ref prev) = *previous {
                let _deltas = self
                    .delta_generator
                    .compute_deltas(prev, &current_snapshots);
            }

            // Update previous snapshots
            drop(previous);
            *self.previous_snapshots.write() = Some(current_snapshots);
        }

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.input_count = input_count;
            stats.included_count = assembled.source_count;
            stats.excluded_count = assembled.excluded_sources.len();
            stats.conflicts_detected = conflicts_detected;
            stats.conflicts_resolved = conflicts_resolved;
            stats.fusion_time_us = start.elapsed().as_micros() as u64;
            stats.output_size_bytes = assembled.total_size_bytes;
        }

        // Store as previous for next delta cycle
        *self.previous_context.write() = Some(assembled.clone());

        assembled
    }

    /// Get the last fusion stats.
    pub fn stats(&self) -> FusionStats {
        self.stats.read().clone()
    }

    /// Get the fusion policy.
    pub fn policy(&self) -> &FusionPolicy {
        &self.policy
    }

    /// Get a reference to the confidence engine.
    pub fn confidence_engine(&self) -> &ConfidenceEngine {
        &self.confidence_engine
    }

    /// Get a reference to the freshness engine.
    pub fn freshness_engine(&self) -> &FreshnessEngine {
        &self.freshness_engine
    }

    /// Get a reference to the priority resolver.
    pub fn priority_resolver(&self) -> &ContextPriorityResolver {
        &self.priority_resolver
    }

    /// Get the previous assembled context.
    pub fn previous_context(&self) -> Option<AssembledContext> {
        self.previous_context.read().clone()
    }
}

impl Default for ContextFusionEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Extension trait to add `clone_with_grace` to FreshnessEngine.
trait FreshnessEngineExt {
    fn clone_with_grace(&self, grace_secs: u64) -> FreshnessEngine;
}

impl FreshnessEngineExt for FreshnessEngine {
    fn clone_with_grace(&self, grace_secs: u64) -> FreshnessEngine {
        FreshnessEngine::new(FreshnessConfig::default(), FusionPolicy::default())
            .with_grace_period(grace_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(
        source: ContextSource,
        confidence: f64,
        data: serde_json::Value,
    ) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source, data);
        s.confidence = confidence;
        s
    }

    #[test]
    fn test_fuse_basic() {
        let engine = ContextFusionEngine::default();
        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                0.9,
                serde_json::json!({"status": "online"}),
            ),
            make_snapshot(
                ContextSource::Activity,
                0.8,
                serde_json::json!({"activity": "coding"}),
            ),
        ];

        let assembled = engine.fuse(snapshots);
        assert_eq!(assembled.source_count, 2);
        assert!(assembled.overall_confidence > 0.7);
    }

    #[test]
    fn test_fuse_filters_low_confidence() {
        let engine = ContextFusionEngine::default();
        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                0.9,
                serde_json::json!({"status": "ok"}),
            ),
            make_snapshot(
                ContextSource::Activity,
                0.05, // Below default floor of 0.1
                serde_json::json!({"activity": "idle"}),
            ),
        ];

        let assembled = engine.fuse(snapshots);
        assert_eq!(assembled.source_count, 1);
        assert!(assembled.has_source(&ContextSource::Environment));
    }

    #[test]
    fn test_fuse_empty() {
        let engine = ContextFusionEngine::default();
        let assembled = engine.fuse(vec![]);
        assert_eq!(assembled.source_count, 0);
    }

    #[test]
    fn test_fuse_records_stats() {
        let engine = ContextFusionEngine::default();
        let snapshots = vec![make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"test": true}),
        )];

        engine.fuse(snapshots);
        let stats = engine.stats();
        assert_eq!(stats.input_count, 1);
        assert_eq!(stats.included_count, 1);
        assert!(stats.fusion_time_us > 0);
    }

    #[test]
    fn test_fuse_preserves_previous() {
        let engine = ContextFusionEngine::default();
        let s1 = vec![make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"v": 1}),
        )];
        let s2 = vec![make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"v": 2}),
        )];

        engine.fuse(s1);
        assert!(engine.previous_context().is_some());

        engine.fuse(s2);
        let prev = engine.previous_context().unwrap();
        assert!(prev.get(&ContextSource::Environment).is_some());
    }

    #[test]
    fn test_fuse_deterministic_output() {
        let engine = ContextFusionEngine::default();
        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                0.9,
                serde_json::json!({"key": "value"}),
            ),
            make_snapshot(
                ContextSource::Activity,
                0.8,
                serde_json::json!({"task": "work"}),
            ),
        ];

        let a = engine.fuse(snapshots.clone());
        let b = engine.fuse(snapshots);

        // Same inputs should produce same data (IDs will differ)
        assert_eq!(a.data, b.data);
        assert_eq!(a.source_count, b.source_count);
    }
}
