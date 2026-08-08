use crate::types::ContextSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy for merging context data from multiple sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MergeStrategy {
    /// Higher priority source wins entirely.
    PriorityOverride,

    /// Merge all sources, deeper nesting overwrites shallower.
    DeepMerge,

    /// Merge all sources, higher priority wins on conflicts.
    #[default]
    WeightedMerge,

    /// Keep the most recent value for each key.
    LatestWins,

    /// Concatenate values into arrays.
    Concatenate,
}

/// How to resolve conflicts between sources claiming contradictory state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConflictResolution {
    /// Use confidence scores to pick the winner.
    ConfidenceBased,

    /// Use priority hierarchy to pick the winner.
    PriorityBased,

    /// Use a combination of confidence and priority.
    #[default]
    CompositeScore,

    /// Flag the conflict and keep both values for downstream resolution.
    FlagAndKeep,

    /// Use the most recent update.
    TemporalFirst,
}

/// Defines a per-source override for fusion behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePolicy {
    /// Maximum staleness in seconds before this source is excluded.
    pub max_staleness: Option<u64>,

    /// Confidence floor — below this, the source is ignored.
    pub confidence_floor: f64,

    /// Weight multiplier for this source (default 1.0).
    pub weight: f64,

    /// Whether this source can override higher-priority sources.
    pub can_override: bool,
}

impl Default for SourcePolicy {
    fn default() -> Self {
        Self {
            max_staleness: None,
            confidence_floor: 0.1,
            weight: 1.0,
            can_override: false,
        }
    }
}

/// Configuration for the context fusion engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionPolicy {
    /// Global merge strategy.
    pub merge_strategy: MergeStrategy,

    /// Global conflict resolution strategy.
    pub conflict_resolution: ConflictResolution,

    /// Default confidence floor across all sources.
    pub default_confidence_floor: f64,

    /// Default weight for sources without explicit overrides.
    pub default_weight: f64,

    /// Maximum total context size in bytes.
    pub max_total_size_bytes: usize,

    /// Maximum number of context sources in the assembled output.
    pub max_sources: usize,

    /// Per-source policy overrides.
    pub source_policies: HashMap<ContextSource, SourcePolicy>,
}

impl Default for FusionPolicy {
    fn default() -> Self {
        let mut source_policies = HashMap::new();

        // System/Safety sources get special treatment
        source_policies.insert(
            ContextSource::SystemState,
            SourcePolicy {
                confidence_floor: 0.5,
                weight: 2.0,
                can_override: true,
                ..Default::default()
            },
        );

        source_policies.insert(
            ContextSource::User,
            SourcePolicy {
                confidence_floor: 0.3,
                weight: 1.5,
                can_override: true,
                ..Default::default()
            },
        );

        // Conversation and Emotional are high-weight for intent detection
        source_policies.insert(
            ContextSource::Conversation,
            SourcePolicy {
                weight: 1.3,
                ..Default::default()
            },
        );

        source_policies.insert(
            ContextSource::Emotional,
            SourcePolicy {
                weight: 1.2,
                ..Default::default()
            },
        );

        Self {
            merge_strategy: MergeStrategy::default(),
            conflict_resolution: ConflictResolution::default(),
            default_confidence_floor: 0.1,
            default_weight: 1.0,
            max_total_size_bytes: 1024 * 1024, // 1MB
            max_sources: 16,
            source_policies,
        }
    }
}

impl FusionPolicy {
    /// Get the policy for a specific source, falling back to defaults.
    pub fn policy_for(&self, source: &ContextSource) -> SourcePolicy {
        self.source_policies
            .get(source)
            .cloned()
            .unwrap_or_default()
    }

    /// Get the effective weight for a source.
    pub fn weight_for(&self, source: &ContextSource) -> f64 {
        self.policy_for(source).weight
    }

    /// Get the effective confidence floor for a source.
    pub fn confidence_floor_for(&self, source: &ContextSource) -> f64 {
        self.policy_for(source)
            .confidence_floor
            .max(self.default_confidence_floor)
    }

    /// Check if a source can override higher-priority sources.
    pub fn can_override(&self, source: &ContextSource) -> bool {
        self.policy_for(source).can_override
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = FusionPolicy::default();
        assert_eq!(policy.merge_strategy, MergeStrategy::WeightedMerge);
        assert_eq!(
            policy.conflict_resolution,
            ConflictResolution::CompositeScore
        );
    }

    #[test]
    fn test_source_policy_override() {
        let policy = FusionPolicy::default();
        let sys_policy = policy.policy_for(&ContextSource::SystemState);
        assert!(sys_policy.can_override);
        assert_eq!(sys_policy.weight, 2.0);
    }

    #[test]
    fn test_default_fallback() {
        let policy = FusionPolicy::default();
        let unknown_policy = policy.policy_for(&ContextSource::ExternalService("test".to_string()));
        assert_eq!(unknown_policy.weight, 1.0);
        assert!(!unknown_policy.can_override);
    }

    #[test]
    fn test_confidence_floor() {
        let policy = FusionPolicy::default();
        assert!(policy.confidence_floor_for(&ContextSource::SystemState) >= 0.5);
    }
}
