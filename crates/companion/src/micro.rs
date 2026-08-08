use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::config::MicroConfig;
use crate::types::{InteractionTiming, MicroInteraction, MicroKind};

/// Template for a micro interaction.
#[derive(Debug, Clone)]
pub struct MicroTemplate {
    pub text: String,
    pub kind: MicroKind,
    pub min_focus_level: f64,
    pub max_focus_level: f64,
    pub applicable_kinds: Vec<MicroKind>,
}

/// Micro interaction engine.
pub struct MicroEngine {
    config: MicroConfig,
    templates: Vec<MicroTemplate>,
    recent_texts: VecDeque<String>,
    interaction_count_hour: usize,
    hour_start: Instant,
    last_interaction: Option<Instant>,
}

impl MicroEngine {
    pub fn new(config: MicroConfig) -> Self {
        let templates = Self::build_templates();
        Self {
            config,
            templates,
            recent_texts: VecDeque::new(),
            interaction_count_hour: 0,
            hour_start: Instant::now(),
            last_interaction: None,
        }
    }

    /// Try to generate a micro interaction.
    pub fn generate(
        &mut self,
        focus_level: f64,
        completed_tasks: usize,
        pending_tasks: usize,
        milestone: bool,
        now: Instant,
    ) -> Option<MicroInteraction> {
        if self.interaction_count_hour >= self.config.max_per_hour {
            return None;
        }

        if let Some(last) = self.last_interaction {
            if now.duration_since(last) < self.config.min_interval {
                return None;
            }
        }

        if now.duration_since(self.hour_start) > Duration::from_secs(3600) {
            self.interaction_count_hour = 0;
            self.hour_start = now;
        }

        let context = MicroContext {
            completed_tasks,
            pending_tasks,
            milestone,
        };

        let mut candidates: Vec<(&MicroTemplate, f64)> = self
            .templates
            .iter()
            .filter(|t| focus_level >= t.min_focus_level && focus_level <= t.max_focus_level)
            .map(|t| (t, self.score_template(t, &context)))
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (template, score) in &candidates {
            if *score < 0.3 {
                continue;
            }
            if self.recent_texts.contains(&template.text) {
                continue;
            }

            let timing = if focus_level > 0.7 {
                InteractionTiming::NextNaturalPause
            } else {
                InteractionTiming::Immediate
            };

            let interaction = MicroInteraction {
                text: template.text.clone(),
                kind: template.kind,
                confidence: *score,
                timing,
            };

            self.recent_texts.push_back(template.text.clone());
            if self.recent_texts.len() > 20 {
                self.recent_texts.pop_front();
            }
            self.interaction_count_hour += 1;
            self.last_interaction = Some(now);

            return Some(interaction);
        }

        None
    }

    fn score_template(&self, template: &MicroTemplate, context: &MicroContext) -> f64 {
        let mut score: f64 = 0.5;

        match template.kind {
            MicroKind::Completion if context.completed_tasks > 0 => score += 0.3,
            MicroKind::Progress if context.pending_tasks > 0 => score += 0.2,
            MicroKind::Encouragement if context.completed_tasks >= 3 => score += 0.2,
            MicroKind::Observation => score += 0.1,
            MicroKind::Reminder if context.pending_tasks > 2 => score += 0.15,
            _ => {}
        }

        if context.milestone && template.kind == MicroKind::Completion {
            score += 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    pub fn reset_hour(&mut self) {
        self.interaction_count_hour = 0;
        self.hour_start = Instant::now();
    }

    fn build_templates() -> Vec<MicroTemplate> {
        vec![
            MicroTemplate {
                text: "Nice.".to_string(),
                kind: MicroKind::Acknowledgment,
                min_focus_level: 0.0,
                max_focus_level: 1.0,
                applicable_kinds: vec![MicroKind::Acknowledgment],
            },
            MicroTemplate {
                text: "Good progress.".to_string(),
                kind: MicroKind::Progress,
                min_focus_level: 0.0,
                max_focus_level: 0.8,
                applicable_kinds: vec![MicroKind::Progress],
            },
            MicroTemplate {
                text: "We're almost done.".to_string(),
                kind: MicroKind::Progress,
                min_focus_level: 0.0,
                max_focus_level: 0.7,
                applicable_kinds: vec![MicroKind::Progress],
            },
            MicroTemplate {
                text: "Mission complete.".to_string(),
                kind: MicroKind::Completion,
                min_focus_level: 0.0,
                max_focus_level: 1.0,
                applicable_kinds: vec![MicroKind::Completion],
            },
            MicroTemplate {
                text: "That's a milestone.".to_string(),
                kind: MicroKind::Completion,
                min_focus_level: 0.0,
                max_focus_level: 1.0,
                applicable_kinds: vec![MicroKind::Completion],
            },
            MicroTemplate {
                text: "Keep going.".to_string(),
                kind: MicroKind::Encouragement,
                min_focus_level: 0.3,
                max_focus_level: 0.8,
                applicable_kinds: vec![MicroKind::Encouragement],
            },
            MicroTemplate {
                text: "Good work.".to_string(),
                kind: MicroKind::Observation,
                min_focus_level: 0.0,
                max_focus_level: 1.0,
                applicable_kinds: vec![MicroKind::Observation],
            },
            MicroTemplate {
                text: "Task pending.".to_string(),
                kind: MicroKind::Reminder,
                min_focus_level: 0.0,
                max_focus_level: 0.5,
                applicable_kinds: vec![MicroKind::Reminder],
            },
        ]
    }
}

struct MicroContext {
    completed_tasks: usize,
    pending_tasks: usize,
    milestone: bool,
}

impl Default for MicroEngine {
    fn default() -> Self {
        Self::new(MicroConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_generation() {
        let mut engine = MicroEngine::new(MicroConfig::default());
        let result = engine.generate(0.5, 1, 1, false, Instant::now());
        assert!(result.is_some());
    }

    #[test]
    fn test_micro_respects_limit() {
        let mut engine = MicroEngine::new(MicroConfig::default());
        for _ in 0..10 {
            engine.generate(0.5, 1, 1, false, Instant::now());
        }
        assert!(engine.interaction_count_hour <= MicroConfig::default().max_per_hour);
    }

    #[test]
    fn test_micro_dedup() {
        let mut engine = MicroEngine::new(MicroConfig::default());
        let i1 = engine.generate(0.5, 1, 1, false, Instant::now());
        let i2 = engine.generate(0.5, 1, 1, false, Instant::now());
        if let (Some(i1), Some(i2)) = (i1, i2) {
            assert_ne!(i1.text, i2.text);
        }
    }

    #[test]
    fn test_micro_cooldown() {
        let mut engine = MicroEngine::new(MicroConfig::default());
        let now = Instant::now();
        engine.generate(0.5, 1, 1, false, now);
        let result = engine.generate(0.5, 1, 1, false, now);
        assert!(result.is_none());
    }
}
