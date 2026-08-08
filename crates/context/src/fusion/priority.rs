use crate::types::{ContextSnapshot, ContextSource};
use std::collections::HashMap;

/// Documented priority hierarchy for context sources.
/// Higher number = higher priority.
const SOURCE_PRIORITY_HIERARCHY: &[(ContextSource, u8)] = &[
    (ContextSource::SystemState, 10),
    (ContextSource::User, 9),
    (ContextSource::Conversation, 8),
    (ContextSource::Emotional, 7),
    (ContextSource::Visual, 6),
    (ContextSource::Audio, 5),
    (ContextSource::Memory, 4),
    (ContextSource::Activity, 3),
    (ContextSource::Environment, 2),
    (ContextSource::Device, 1),
    (ContextSource::Personality, 3),
    (ContextSource::WorldModel, 2),
];

/// Resolves priority conflicts between context sources.
pub struct ContextPriorityResolver {
    hierarchy: HashMap<ContextSource, u8>,
}

impl ContextPriorityResolver {
    pub fn new() -> Self {
        let hierarchy = SOURCE_PRIORITY_HIERARCHY.iter().cloned().collect();
        Self { hierarchy }
    }

    /// Get the hierarchy rank for a source (higher = more important).
    pub fn rank(&self, source: &ContextSource) -> u8 {
        self.hierarchy.get(source).copied().unwrap_or(0)
    }

    /// Resolve which of two snapshots should win based on source hierarchy.
    pub fn resolve<'a>(
        &self,
        a: &'a ContextSnapshot,
        b: &'a ContextSnapshot,
    ) -> &'a ContextSnapshot {
        let rank_a = self.rank(&a.source);
        let rank_b = self.rank(&b.source);

        match rank_a.cmp(&rank_b) {
            std::cmp::Ordering::Greater => a,
            std::cmp::Ordering::Less => b,
            std::cmp::Ordering::Equal => {
                // Tie-break by priority level
                match a.priority.cmp(&b.priority) {
                    std::cmp::Ordering::Greater => a,
                    std::cmp::Ordering::Less => b,
                    std::cmp::Ordering::Equal => {
                        // Final tie-break: higher confidence wins
                        if a.confidence >= b.confidence {
                            a
                        } else {
                            b
                        }
                    }
                }
            }
        }
    }

    /// Sort snapshots by priority hierarchy (highest first).
    pub fn sort_by_priority(&self, snapshots: &mut [ContextSnapshot]) {
        snapshots.sort_by(|a, b| {
            let rank_a = self.rank(&a.source);
            let rank_b = self.rank(&b.source);
            rank_b
                .cmp(&rank_a)
                .then_with(|| b.priority.cmp(&a.priority))
                .then_with(|| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
    }

    /// Get the winning source from a group of snapshots.
    pub fn winner<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots.iter().max_by_key(|s| {
            let rank = self.rank(&s.source) as u32;
            let priority = s.priority as u32;
            let conf = (s.confidence * 1000.0) as u32;
            (rank << 24) | (priority << 16) | conf
        })
    }

    /// Check if a source has higher priority than another.
    pub fn is_higher_priority(&self, source: &ContextSource, other: &ContextSource) -> bool {
        self.rank(source) > self.rank(other)
    }
}

impl Default for ContextPriorityResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContextPriority;

    fn make_snapshot(
        source: ContextSource,
        priority: ContextPriority,
        confidence: f64,
    ) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source, serde_json::json!({"test": true}));
        s.priority = priority;
        s.confidence = confidence;
        s
    }

    #[test]
    fn test_system_state_highest_rank() {
        let resolver = ContextPriorityResolver::default();
        assert!(resolver.rank(&ContextSource::SystemState) > resolver.rank(&ContextSource::User));
        assert!(resolver.rank(&ContextSource::User) > resolver.rank(&ContextSource::Conversation));
        assert!(
            resolver.rank(&ContextSource::Conversation)
                > resolver.rank(&ContextSource::Environment)
        );
    }

    #[test]
    fn test_resolve_prefers_higher_rank() {
        let resolver = ContextPriorityResolver::default();
        let sys = make_snapshot(ContextSource::SystemState, ContextPriority::Medium, 0.8);
        let env = make_snapshot(ContextSource::Environment, ContextPriority::Critical, 0.9);
        // SystemState rank (10) > Environment rank (2), so SystemState wins
        let winner = resolver.resolve(&sys, &env);
        assert_eq!(winner.source, ContextSource::SystemState);
    }

    #[test]
    fn test_resolve_same_rank_prefers_priority() {
        let resolver = ContextPriorityResolver::default();
        let a = make_snapshot(ContextSource::Environment, ContextPriority::Low, 0.9);
        let b = make_snapshot(ContextSource::Environment, ContextPriority::High, 0.7);
        let winner = resolver.resolve(&a, &b);
        assert_eq!(winner.priority, ContextPriority::High);
    }

    #[test]
    fn test_sort_by_priority() {
        let resolver = ContextPriorityResolver::default();
        let mut snaps = vec![
            make_snapshot(ContextSource::Environment, ContextPriority::Medium, 0.8),
            make_snapshot(ContextSource::SystemState, ContextPriority::Medium, 0.8),
            make_snapshot(ContextSource::Activity, ContextPriority::Medium, 0.8),
        ];
        resolver.sort_by_priority(&mut snaps);
        assert_eq!(snaps[0].source, ContextSource::SystemState);
        assert_eq!(snaps[1].source, ContextSource::Activity);
        assert_eq!(snaps[2].source, ContextSource::Environment);
    }

    #[test]
    fn test_winner() {
        let resolver = ContextPriorityResolver::default();
        let snaps = vec![
            make_snapshot(ContextSource::Activity, ContextPriority::Medium, 0.8),
            make_snapshot(ContextSource::SystemState, ContextPriority::Low, 0.5),
        ];
        let winner = resolver.winner(&snaps).unwrap();
        assert_eq!(winner.source, ContextSource::SystemState);
    }

    #[test]
    fn test_is_higher_priority() {
        let resolver = ContextPriorityResolver::default();
        assert!(resolver.is_higher_priority(&ContextSource::User, &ContextSource::Memory));
        assert!(!resolver.is_higher_priority(&ContextSource::Device, &ContextSource::Visual));
    }
}
