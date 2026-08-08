use crate::types::{ContextSnapshot, ContextSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single field-level change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDelta {
    /// The field path that changed.
    pub field: String,

    /// Previous value (None if newly added).
    pub old_value: Option<serde_json::Value>,

    /// New value (None if removed).
    pub new_value: Option<serde_json::Value>,
}

/// Represents a set of changes between two context states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDelta {
    /// Source that produced the delta.
    pub source: ContextSource,

    /// Field-level changes.
    pub changes: Vec<FieldDelta>,

    /// Sources that were added in the new state.
    pub sources_added: Vec<ContextSource>,

    /// Sources that were removed in the new state.
    pub sources_removed: Vec<ContextSource>,

    /// Whether the overall context changed significantly.
    pub significant: bool,
}

impl ContextDelta {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.sources_added.is_empty() && self.sources_removed.is_empty()
    }

    pub fn change_count(&self) -> usize {
        self.changes.len() + self.sources_added.len() + self.sources_removed.len()
    }
}

/// Generates change deltas between two sets of context snapshots.
pub struct ContextDeltaGenerator {
    /// Threshold for considering a change "significant" (fraction of fields changed).
    significance_threshold: f64,
}

impl ContextDeltaGenerator {
    pub fn new() -> Self {
        Self {
            significance_threshold: 0.3,
        }
    }

    pub fn with_significance_threshold(mut self, threshold: f64) -> Self {
        self.significance_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Compute deltas between previous and current snapshot sets, keyed by source.
    pub fn compute_deltas(
        &self,
        previous: &HashMap<ContextSource, ContextSnapshot>,
        current: &HashMap<ContextSource, ContextSnapshot>,
    ) -> Vec<ContextDelta> {
        let mut deltas = Vec::new();

        // Find sources added and removed
        let prev_sources: HashSet<&ContextSource> = previous.keys().collect();
        let curr_sources: HashSet<&ContextSource> = current.keys().collect();

        let sources_added: Vec<ContextSource> = curr_sources
            .difference(&prev_sources)
            .map(|s| (*s).clone())
            .collect();

        let sources_removed: Vec<ContextSource> = prev_sources
            .difference(&curr_sources)
            .map(|s| (*s).clone())
            .collect();

        // Compute field-level deltas for sources present in both
        for source in prev_sources.intersection(&curr_sources) {
            let prev = &previous[*source];
            let curr = &current[*source];

            let changes = self.compute_field_deltas(&prev.data, &curr.data);

            if !changes.is_empty() {
                let significant = changes.len() as f64
                    / self.estimate_field_count(&prev.data).max(1) as f64
                    >= self.significance_threshold;

                deltas.push(ContextDelta {
                    source: (*source).clone(),
                    changes,
                    sources_added: Vec::new(),
                    sources_removed: Vec::new(),
                    significant,
                });
            }
        }

        // Add delta entries for added/removed sources
        if !sources_added.is_empty() || !sources_removed.is_empty() {
            deltas.push(ContextDelta {
                source: ContextSource::SystemState,
                changes: Vec::new(),
                sources_added,
                sources_removed,
                significant: true,
            });
        }

        deltas
    }

    /// Compute field-level deltas between two JSON values.
    fn compute_field_deltas(
        &self,
        old: &serde_json::Value,
        new: &serde_json::Value,
    ) -> Vec<FieldDelta> {
        let mut deltas = Vec::new();

        match (old, new) {
            (serde_json::Value::Object(old_map), serde_json::Value::Object(new_map)) => {
                // Find changed and added keys
                for (key, new_val) in new_map {
                    match old_map.get(key) {
                        Some(old_val) => {
                            if old_val != new_val {
                                deltas.push(FieldDelta {
                                    field: key.clone(),
                                    old_value: Some(old_val.clone()),
                                    new_value: Some(new_val.clone()),
                                });
                            }
                        }
                        None => {
                            deltas.push(FieldDelta {
                                field: key.clone(),
                                old_value: None,
                                new_value: Some(new_val.clone()),
                            });
                        }
                    }
                }

                // Find removed keys
                for key in old_map.keys() {
                    if !new_map.contains_key(key) {
                        deltas.push(FieldDelta {
                            field: key.clone(),
                            old_value: old_map.get(key).cloned(),
                            new_value: None,
                        });
                    }
                }
            }
            _ => {
                if old != new {
                    deltas.push(FieldDelta {
                        field: ".".to_string(),
                        old_value: Some(old.clone()),
                        new_value: Some(new.clone()),
                    });
                }
            }
        }

        deltas
    }

    /// Estimate the total number of fields in a JSON value.
    fn estimate_field_count(&self, value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Object(map) => map
                .values()
                .map(|v| self.estimate_field_count(v).max(1))
                .sum::<usize>()
                .max(1),
            serde_json::Value::Array(arr) => arr.len().max(1),
            _ => 1,
        }
    }
}

impl Default for ContextDeltaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContextSource;

    fn make_snapshot(source: ContextSource, data: serde_json::Value) -> ContextSnapshot {
        ContextSnapshot::new(source, data)
    }

    #[test]
    fn test_no_changes() {
        let generator = ContextDeltaGenerator::default();
        let mut map = std::collections::HashMap::new();
        map.insert(
            ContextSource::Environment,
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"status": "ok"}),
            ),
        );

        let deltas = generator.compute_deltas(&map, &map);
        assert!(deltas.is_empty());
    }

    #[test]
    fn test_field_changed() {
        let generator = ContextDeltaGenerator::default();
        let mut prev = std::collections::HashMap::new();
        let mut curr = std::collections::HashMap::new();

        prev.insert(
            ContextSource::Environment,
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"status": "online"}),
            ),
        );
        curr.insert(
            ContextSource::Environment,
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"status": "offline"}),
            ),
        );

        let deltas = generator.compute_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].changes.len(), 1);
        assert_eq!(deltas[0].changes[0].field, "status");
    }

    #[test]
    fn test_source_added() {
        let generator = ContextDeltaGenerator::default();
        let prev = std::collections::HashMap::new();
        let mut curr = std::collections::HashMap::new();

        curr.insert(
            ContextSource::Activity,
            make_snapshot(
                ContextSource::Activity,
                serde_json::json!({"activity": "idle"}),
            ),
        );

        let deltas = generator.compute_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].sources_added.len(), 1);
    }

    #[test]
    fn test_source_removed() {
        let generator = ContextDeltaGenerator::default();
        let mut prev = std::collections::HashMap::new();
        let curr = std::collections::HashMap::new();

        prev.insert(
            ContextSource::Activity,
            make_snapshot(
                ContextSource::Activity,
                serde_json::json!({"activity": "idle"}),
            ),
        );

        let deltas = generator.compute_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].sources_removed.len(), 1);
    }

    #[test]
    fn test_delta_empty() {
        let delta = ContextDelta {
            source: ContextSource::Environment,
            changes: Vec::new(),
            sources_added: Vec::new(),
            sources_removed: Vec::new(),
            significant: false,
        };
        assert!(delta.is_empty());
    }

    #[test]
    fn test_field_added() {
        let generator = ContextDeltaGenerator::default();
        let mut prev = std::collections::HashMap::new();
        let mut curr = std::collections::HashMap::new();

        prev.insert(
            ContextSource::Environment,
            make_snapshot(ContextSource::Environment, serde_json::json!({"a": 1})),
        );
        curr.insert(
            ContextSource::Environment,
            make_snapshot(
                ContextSource::Environment,
                serde_json::json!({"a": 1, "b": 2}),
            ),
        );

        let deltas = generator.compute_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        assert!(deltas[0].changes[0].old_value.is_none());
        assert_eq!(deltas[0].changes[0].new_value, Some(serde_json::json!(2)));
    }
}
