use crate::fusion::confidence::ConfidenceEngine;
use crate::fusion::policy::{ConflictResolution, FusionPolicy};
use crate::fusion::priority::ContextPriorityResolver;
use crate::types::{ContextSnapshot, ContextSource};
use serde::{Deserialize, Serialize};

/// Represents a conflict between two context snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConflict {
    /// Key or field path where the conflict exists.
    pub field: String,

    /// Source claiming value A.
    pub source_a: ContextSource,

    /// Source claiming value B.
    pub source_b: ContextSource,

    /// Value from source A.
    pub value_a: serde_json::Value,

    /// Value from value B.
    pub value_b: serde_json::Value,

    /// Confidence of source A.
    pub confidence_a: f64,

    /// Confidence of source B.
    pub confidence_b: f64,
}

/// Resolution result for a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionResult {
    /// The winning value.
    pub winner: serde_json::Value,

    /// Which source won.
    pub winning_source: ContextSource,

    /// Reason for the resolution.
    pub reason: String,
}

/// Detects and resolves conflicts between context snapshots from different sources.
pub struct ContextConflictResolver {
    policy: FusionPolicy,
    confidence_engine: ConfidenceEngine,
    priority_resolver: ContextPriorityResolver,
}

impl ContextConflictResolver {
    pub fn new(policy: FusionPolicy) -> Self {
        let confidence_engine = ConfidenceEngine::new(policy.clone());
        let priority_resolver = ContextPriorityResolver::new();
        Self {
            policy,
            confidence_engine,
            priority_resolver,
        }
    }

    /// Detect conflicts between two snapshots.
    pub fn detect_conflicts(
        &self,
        a: &ContextSnapshot,
        b: &ContextSnapshot,
    ) -> Vec<ContextConflict> {
        let mut conflicts = Vec::new();

        if a.source == b.source {
            return conflicts;
        }

        let data_a = match a.data.as_object() {
            Some(m) => m,
            None => return conflicts,
        };
        let data_b = match b.data.as_object() {
            Some(m) => m,
            None => return conflicts,
        };

        for (key, val_a) in data_a {
            if let Some(val_b) = data_b.get(key) {
                if val_a != val_b {
                    conflicts.push(ContextConflict {
                        field: key.clone(),
                        source_a: a.source.clone(),
                        source_b: b.source.clone(),
                        value_a: val_a.clone(),
                        value_b: val_b.clone(),
                        confidence_a: a.confidence,
                        confidence_b: b.confidence,
                    });
                }
            }
        }

        conflicts
    }

    /// Resolve a single conflict using the configured strategy.
    pub fn resolve_conflict(&self, conflict: &ContextConflict) -> ConflictResolutionResult {
        match self.policy.conflict_resolution {
            ConflictResolution::ConfidenceBased => {
                if conflict.confidence_a >= conflict.confidence_b {
                    ConflictResolutionResult {
                        winner: conflict.value_a.clone(),
                        winning_source: conflict.source_a.clone(),
                        reason: "Higher confidence".to_string(),
                    }
                } else {
                    ConflictResolutionResult {
                        winner: conflict.value_b.clone(),
                        winning_source: conflict.source_b.clone(),
                        reason: "Higher confidence".to_string(),
                    }
                }
            }
            ConflictResolution::PriorityBased => {
                if self
                    .priority_resolver
                    .is_higher_priority(&conflict.source_a, &conflict.source_b)
                {
                    ConflictResolutionResult {
                        winner: conflict.value_a.clone(),
                        winning_source: conflict.source_a.clone(),
                        reason: "Higher priority source".to_string(),
                    }
                } else {
                    ConflictResolutionResult {
                        winner: conflict.value_b.clone(),
                        winning_source: conflict.source_b.clone(),
                        reason: "Higher priority source".to_string(),
                    }
                }
            }
            ConflictResolution::CompositeScore => {
                // Build temporary snapshots for scoring
                let snap_a = self.make_temp_snapshot(
                    &conflict.source_a,
                    conflict.confidence_a,
                    &conflict.value_a,
                );
                let snap_b = self.make_temp_snapshot(
                    &conflict.source_b,
                    conflict.confidence_b,
                    &conflict.value_b,
                );

                let score_a = self.confidence_engine.composite_score(&snap_a);
                let score_b = self.confidence_engine.composite_score(&snap_b);

                if score_a >= score_b {
                    ConflictResolutionResult {
                        winner: conflict.value_a.clone(),
                        winning_source: conflict.source_a.clone(),
                        reason: format!("Composite score: {score_a:.3} vs {score_b:.3}"),
                    }
                } else {
                    ConflictResolutionResult {
                        winner: conflict.value_b.clone(),
                        winning_source: conflict.source_b.clone(),
                        reason: format!("Composite score: {score_b:.3} vs {score_a:.3}"),
                    }
                }
            }
            ConflictResolution::TemporalFirst => {
                // First source in the conflict wins (temporal ordering)
                ConflictResolutionResult {
                    winner: conflict.value_a.clone(),
                    winning_source: conflict.source_a.clone(),
                    reason: "Temporal first".to_string(),
                }
            }
            ConflictResolution::FlagAndKeep => ConflictResolutionResult {
                winner: serde_json::Value::Null,
                winning_source: conflict.source_a.clone(),
                reason: "Conflict flagged for downstream resolution".to_string(),
            },
        }
    }

    /// Resolve all conflicts between two snapshots.
    pub fn resolve_all(
        &self,
        a: &ContextSnapshot,
        b: &ContextSnapshot,
    ) -> Vec<ConflictResolutionResult> {
        let conflicts = self.detect_conflicts(a, b);
        conflicts.iter().map(|c| self.resolve_conflict(c)).collect()
    }

    /// Find the winning snapshot between two, considering conflicts.
    pub fn pick_winner<'a>(
        &self,
        a: &'a ContextSnapshot,
        b: &'a ContextSnapshot,
    ) -> &'a ContextSnapshot {
        let score_a = self.confidence_engine.composite_score(a);
        let score_b = self.confidence_engine.composite_score(b);

        if score_a >= score_b {
            a
        } else {
            b
        }
    }

    fn make_temp_snapshot(
        &self,
        source: &ContextSource,
        confidence: f64,
        data: &serde_json::Value,
    ) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source.clone(), data.clone());
        s.confidence = confidence;
        s
    }
}

impl Default for ContextConflictResolver {
    fn default() -> Self {
        Self::new(FusionPolicy::default())
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
    fn test_detect_conflicts() {
        let resolver = ContextConflictResolver::default();
        let a = make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"network": "online"}),
        );
        let b = make_snapshot(
            ContextSource::Activity,
            0.8,
            serde_json::json!({"network": "offline"}),
        );
        let conflicts = resolver.detect_conflicts(&a, &b);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].field, "network");
    }

    #[test]
    fn test_no_conflicts_same_source() {
        let resolver = ContextConflictResolver::default();
        let a = make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"status": "ok"}),
        );
        let b = make_snapshot(
            ContextSource::Environment,
            0.8,
            serde_json::json!({"status": "ok"}),
        );
        let conflicts = resolver.detect_conflicts(&a, &b);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_resolve_confidence_based() {
        let policy = FusionPolicy {
            conflict_resolution: ConflictResolution::ConfidenceBased,
            ..FusionPolicy::default()
        };
        let resolver = ContextConflictResolver::new(policy);

        let conflict = ContextConflict {
            field: "status".to_string(),
            source_a: ContextSource::Environment,
            source_b: ContextSource::Activity,
            value_a: serde_json::json!("online"),
            value_b: serde_json::json!("offline"),
            confidence_a: 0.9,
            confidence_b: 0.3,
        };

        let result = resolver.resolve_conflict(&conflict);
        assert_eq!(result.winner, serde_json::json!("online"));
        assert_eq!(result.winning_source, ContextSource::Environment);
    }

    #[test]
    fn test_resolve_priority_based() {
        let policy = FusionPolicy {
            conflict_resolution: ConflictResolution::PriorityBased,
            ..FusionPolicy::default()
        };
        let resolver = ContextConflictResolver::new(policy);

        let conflict = ContextConflict {
            field: "status".to_string(),
            source_a: ContextSource::Device,
            source_b: ContextSource::User,
            value_a: serde_json::json!("low"),
            value_b: serde_json::json!("high"),
            confidence_a: 0.9,
            confidence_b: 0.5,
        };

        let result = resolver.resolve_conflict(&conflict);
        // User has higher rank than Device
        assert_eq!(result.winning_source, ContextSource::User);
    }

    #[test]
    fn test_pick_winner() {
        let resolver = ContextConflictResolver::default();
        let a = make_snapshot(
            ContextSource::Environment,
            0.9,
            serde_json::json!({"k": "v1"}),
        );
        let b = make_snapshot(ContextSource::Activity, 0.3, serde_json::json!({"k": "v2"}));
        let winner = resolver.pick_winner(&a, &b);
        assert_eq!(winner.source, ContextSource::Environment);
    }
}
