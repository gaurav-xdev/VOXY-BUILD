use std::time::Instant;

use crate::config::MemoryConfig;
use crate::types::{MemoryKind, MemoryMoment};

/// Memory moments engine — references past work at appropriate times.
pub struct MemoryMoments {
    config: MemoryConfig,
    moments: Vec<StoredMoment>,
    references_this_session: usize,
}

struct StoredMoment {
    text: String,
    kind: MemoryKind,
    relevance: f64,
    used_count: usize,
    last_used: Option<Instant>,
}

impl MemoryMoments {
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            config,
            moments: Vec::new(),
            references_this_session: 0,
        }
    }

    /// Record a memory moment.
    pub fn record(&mut self, text: &str, kind: MemoryKind, relevance: f64) {
        self.moments.push(StoredMoment {
            text: text.to_string(),
            kind,
            relevance,
            used_count: 0,
            last_used: None,
        });
    }

    /// Try to generate a memory moment reference.
    pub fn generate(
        &mut self,
        context: &str,
        _completed_tasks: usize,
        milestones: &[String],
        now: Instant,
    ) -> Option<MemoryMoment> {
        if self.references_this_session >= self.config.max_references_per_session {
            return None;
        }

        let lower_context = context.to_lowercase();

        let mut candidates: Vec<(usize, f64)> = self
            .moments
            .iter()
            .enumerate()
            .filter(|(_, m)| m.used_count < 3)
            .filter(|(_, m)| {
                m.last_used
                    .map(|t| now.duration_since(t) >= self.config.min_referral_interval)
                    .unwrap_or(true)
            })
            .map(|(i, m)| {
                let context_bonus = if lower_context.contains(&m.text.to_lowercase()) {
                    0.3
                } else {
                    0.0
                };
                let recency_bonus = m
                    .last_used
                    .map(|t| (1.0 - (now.duration_since(t).as_secs_f64() / 7200.0)).max(0.0))
                    .unwrap_or(0.5);
                let milestone_bonus = if m.kind == MemoryKind::Milestone && !milestones.is_empty() {
                    0.2
                } else {
                    0.0
                };
                let score =
                    m.relevance * 0.4 + recency_bonus * 0.3 + context_bonus + milestone_bonus;
                (i, score)
            })
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((idx, _score)) = candidates.first() {
            let moment = &mut self.moments[*idx];
            let result = MemoryMoment {
                text: moment.text.clone(),
                kind: moment.kind,
                relevance: moment.relevance,
                timestamp: chrono::Utc::now(),
            };
            moment.used_count += 1;
            moment.last_used = Some(now);
            self.references_this_session += 1;
            Some(result)
        } else {
            None
        }
    }

    pub fn reset_session(&mut self) {
        self.references_this_session = 0;
    }

    pub fn moment_count(&self) -> usize {
        self.moments.len()
    }

    pub fn references_this_session(&self) -> usize {
        self.references_this_session
    }
}

impl Default for MemoryMoments {
    fn default() -> Self {
        Self::new(MemoryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_reference() {
        let mut mm = MemoryMoments::new(MemoryConfig::default());
        mm.record("Context Fusion Engine", MemoryKind::Milestone, 0.9);
        let moment = mm.generate("Context", 0, &[], Instant::now());
        assert!(moment.is_some());
        assert_eq!(mm.references_this_session(), 1);
    }

    #[test]
    fn test_max_references() {
        let mut mm = MemoryMoments::new(MemoryConfig::default());
        for i in 0..10 {
            mm.record(&format!("Event {}", i), MemoryKind::Achievement, 0.8);
        }
        for _ in 0..10 {
            mm.generate("Event", 0, &[], Instant::now());
        }
        let result = mm.generate("Event", 0, &[], Instant::now());
        assert!(result.is_none());
    }

    #[test]
    fn test_used_count_limit() {
        let mut mm = MemoryMoments::new(MemoryConfig::default());
        mm.record("Test", MemoryKind::Milestone, 0.9);
        for _ in 0..5 {
            mm.generate("Test", 0, &[], Instant::now());
        }
        let result = mm.generate("Test", 0, &[], Instant::now());
        assert!(result.is_none());
    }
}
