use crate::fusion::policy::{FusionPolicy, MergeStrategy};
use crate::types::ContextSnapshot;
use serde_json::Value;

/// Merges context data from multiple sources into a single coherent output.
pub struct ContextMerger {
    policy: FusionPolicy,
}

impl ContextMerger {
    pub fn new(policy: FusionPolicy) -> Self {
        Self { policy }
    }

    /// Merge multiple snapshots into a single data value.
    pub fn merge(&self, snapshots: &[ContextSnapshot]) -> Value {
        match self.policy.merge_strategy {
            MergeStrategy::PriorityOverride => self.priority_override_merge(snapshots),
            MergeStrategy::DeepMerge => self.deep_merge(snapshots),
            MergeStrategy::WeightedMerge => self.weighted_merge(snapshots),
            MergeStrategy::LatestWins => self.latest_wins_merge(snapshots),
            MergeStrategy::Concatenate => self.concatenate_merge(snapshots),
        }
    }

    /// Merge two specific snapshots.
    pub fn merge_pair(&self, base: &ContextSnapshot, overlay: &ContextSnapshot) -> Value {
        self.merge_two(&base.data, &overlay.data)
    }

    /// Deep merge two JSON values. overlay wins on conflicts.
    pub fn merge_two(&self, base: &Value, overlay: &Value) -> Value {
        match (base, overlay) {
            (Value::Object(base_map), Value::Object(overlay_map)) => {
                let mut result = serde_json::Map::new();

                // Add all base keys
                for (k, v) in base_map {
                    result.insert(k.clone(), v.clone());
                }

                // Overlay wins on conflicts, add new keys
                for (k, v) in overlay_map {
                    result.insert(k.clone(), v.clone());
                }

                Value::Object(result)
            }
            (Value::Array(base_arr), Value::Array(overlay_arr)) => {
                let mut result = base_arr.clone();
                result.extend(overlay_arr.iter().cloned());
                Value::Array(result)
            }
            // Non-matching types or primitives: overlay wins
            (_, overlay) => overlay.clone(),
        }
    }

    /// Priority override: highest-priority source's data wins entirely.
    fn priority_override_merge(&self, snapshots: &[ContextSnapshot]) -> Value {
        // Snapshots should already be sorted by priority
        snapshots
            .first()
            .map(|s| s.data.clone())
            .unwrap_or(Value::Null)
    }

    /// Deep merge all sources, higher priority wins on key conflicts.
    fn deep_merge(&self, snapshots: &[ContextSnapshot]) -> Value {
        if snapshots.is_empty() {
            return Value::Null;
        }

        let mut result = Value::Object(serde_json::Map::new());

        // Process in reverse order so lower-priority is overwritten
        for snapshot in snapshots.iter().rev() {
            result = self.merge_two(&result, &snapshot.data);
        }

        result
    }

    /// Weighted merge: each source contributes with its weight.
    fn weighted_merge(&self, snapshots: &[ContextSnapshot]) -> Value {
        let mut result = serde_json::Map::new();
        let mut contribution_count: serde_json::Map<String, Value> = serde_json::Map::new();

        for snapshot in snapshots {
            if let Some(obj) = snapshot.data.as_object() {
                let weight = self.policy.weight_for(&snapshot.source);
                for (key, val) in obj {
                    // Higher weight or existing value comparison
                    if let Some(existing) = result.get(key) {
                        // Simple heuristic: if both are numbers, average them
                        if let (Some(e_num), Some(v_num)) = (existing.as_f64(), val.as_f64()) {
                            let count = contribution_count
                                .get(key)
                                .and_then(|v| v.as_f64())
                                .unwrap_or(1.0);
                            let new_val = (e_num * count + v_num * weight) / (count + weight);
                            result.insert(key.clone(), Value::from(new_val));
                            contribution_count.insert(key.clone(), Value::from(count + weight));
                            continue;
                        }
                    }
                    result.insert(key.clone(), val.clone());
                    contribution_count.insert(key.clone(), Value::from(1.0));
                }
            }
        }

        Value::Object(result)
    }

    /// Latest wins: most recently captured snapshot's data wins per key.
    fn latest_wins_merge(&self, snapshots: &[ContextSnapshot]) -> Value {
        let mut result = serde_json::Map::new();

        // Snapshots sorted oldest to newest; later entries overwrite
        for snapshot in snapshots {
            if let Some(obj) = snapshot.data.as_object() {
                for (key, val) in obj {
                    result.insert(key.clone(), val.clone());
                }
            }
        }

        Value::Object(result)
    }

    /// Concatenate: combine array values, merge objects.
    fn concatenate_merge(&self, snapshots: &[ContextSnapshot]) -> Value {
        let mut arrays: Vec<Vec<Value>> = Vec::new();
        let mut objects = serde_json::Map::new();

        for snapshot in snapshots {
            match &snapshot.data {
                Value::Array(arr) => arrays.push(arr.clone()),
                Value::Object(obj) => {
                    for (k, v) in obj {
                        objects.insert(k.clone(), v.clone());
                    }
                }
                _ => {}
            }
        }

        if !arrays.is_empty() {
            let mut combined: Vec<Value> = arrays.into_iter().flatten().collect();
            // Append object data as a single element if present
            if !objects.is_empty() {
                combined.push(Value::Object(objects));
            }
            Value::Array(combined)
        } else {
            Value::Object(objects)
        }
    }
}

impl Default for ContextMerger {
    fn default() -> Self {
        Self::new(FusionPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContextPriority, ContextSource};

    fn make_snapshot(
        source: ContextSource,
        data: serde_json::Value,
        priority: ContextPriority,
    ) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source, data);
        s.priority = priority;
        s
    }

    #[test]
    fn test_priority_override() {
        let policy = FusionPolicy {
            merge_strategy: MergeStrategy::PriorityOverride,
            ..FusionPolicy::default()
        };
        let merger = ContextMerger::new(policy);

        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"status": "ok"}),
                ContextPriority::Low,
            ),
            make_snapshot(
                ContextSource::SystemState,
                serde_json::json!({"status": "critical"}),
                ContextPriority::Critical,
            ),
        ];

        let result = merger.merge(&snapshots);
        // PriorityOverride: first snapshot wins (should be pre-sorted)
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn test_deep_merge() {
        let policy = FusionPolicy {
            merge_strategy: MergeStrategy::DeepMerge,
            ..FusionPolicy::default()
        };
        let merger = ContextMerger::new(policy);

        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"a": 1, "b": 2}),
                ContextPriority::Low,
            ),
            make_snapshot(
                ContextSource::Activity,
                serde_json::json!({"b": 3, "c": 4}),
                ContextPriority::Medium,
            ),
        ];

        let result = merger.merge(&snapshots);
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2); // First (highest priority) source wins
        assert_eq!(result["c"], 4);
    }

    #[test]
    fn test_merge_two() {
        let merger = ContextMerger::default();
        let base = serde_json::json!({"a": 1, "b": 2});
        let overlay = serde_json::json!({"b": 3, "c": 4});
        let result = merger.merge_two(&base, &overlay);
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 3);
        assert_eq!(result["c"], 4);
    }

    #[test]
    fn test_latest_wins() {
        let policy = FusionPolicy {
            merge_strategy: MergeStrategy::LatestWins,
            ..FusionPolicy::default()
        };
        let merger = ContextMerger::new(policy);

        let snapshots = vec![
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"key": "old"}),
                ContextPriority::Low,
            ),
            make_snapshot(
                ContextSource::Activity,
                serde_json::json!({"key": "new"}),
                ContextPriority::Medium,
            ),
        ];

        let result = merger.merge(&snapshots);
        assert_eq!(result["key"], "new");
    }

    #[test]
    fn test_merge_empty() {
        let merger = ContextMerger::default();
        let result = merger.merge(&[]);
        assert_eq!(result, serde_json::json!({}));
    }
}
