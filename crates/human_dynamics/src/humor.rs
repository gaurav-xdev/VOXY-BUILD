use std::time::{Duration, Instant};

use crate::config::HumorConfig;
use crate::types::{HumorContext, HumorDecision};

/// Adaptive humor engine — decides when subtle humor is appropriate.
pub struct HumorEngine {
    config: HumorConfig,
    last_humor: Option<Instant>,
    humor_count_hour: usize,
    hour_start: Instant,
    total_humor_count: usize,
}

impl HumorEngine {
    pub fn new(config: HumorConfig) -> Self {
        Self {
            config,
            last_humor: None,
            humor_count_hour: 0,
            hour_start: Instant::now(),
            total_humor_count: 0,
        }
    }

    /// Decide whether to use humor in this context.
    pub fn decide(&mut self, context: &HumorContext, now: Instant) -> HumorDecision {
        if !self.config.enabled {
            return HumorDecision {
                use_humor: false,
                confidence: 0.0,
                reason: "Humor disabled".to_string(),
            };
        }

        if now.duration_since(self.hour_start) > Duration::from_secs(3600) {
            self.humor_count_hour = 0;
            self.hour_start = now;
        }

        if self.humor_count_hour >= self.config.max_per_hour {
            return HumorDecision {
                use_humor: false,
                confidence: 0.0,
                reason: "Hourly humor limit reached".to_string(),
            };
        }

        if let Some(last) = self.last_humor {
            if now.duration_since(last) < self.config.cooldown {
                return HumorDecision {
                    use_humor: false,
                    confidence: 0.0,
                    reason: "Humor cooldown active".to_string(),
                };
            }
        }

        if context.relationship_score < 0.4 {
            return HumorDecision {
                use_humor: false,
                confidence: 0.0,
                reason: "Relationship too new for humor".to_string(),
            };
        }

        if context.confidence < self.config.min_confidence {
            return HumorDecision {
                use_humor: false,
                confidence: context.confidence,
                reason: "Confidence too low for humor".to_string(),
            };
        }

        if context.context_appropriateness < 0.5 {
            return HumorDecision {
                use_humor: false,
                confidence: context.context_appropriateness,
                reason: "Context not appropriate for humor".to_string(),
            };
        }

        let score = context.relationship_score * 0.3
            + context.context_appropriateness * 0.3
            + context.timing_score * 0.2
            + context.confidence * 0.2;

        if score < 0.4 {
            return HumorDecision {
                use_humor: false,
                confidence: score,
                reason: "Combined score too low".to_string(),
            };
        }

        self.last_humor = Some(now);
        self.humor_count_hour += 1;
        self.total_humor_count += 1;

        HumorDecision {
            use_humor: true,
            confidence: score,
            reason: "Humor appropriate for context".to_string(),
        }
    }

    pub fn total_count(&self) -> usize {
        self.total_humor_count
    }

    pub fn hourly_count(&self) -> usize {
        self.humor_count_hour
    }
}

impl Default for HumorEngine {
    fn default() -> Self {
        Self::new(HumorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(relationship: f64, confidence: f64, appropriateness: f64) -> HumorContext {
        HumorContext {
            relationship_score: relationship,
            context_appropriateness: appropriateness,
            timing_score: 0.7,
            confidence,
            recent_humor_count: 0,
        }
    }

    #[test]
    fn test_humor_disabled() {
        let config = HumorConfig {
            enabled: false,
            ..HumorConfig::default()
        };
        let mut engine = HumorEngine::new(config);
        let ctx = make_context(0.8, 0.9, 0.8);
        let decision = engine.decide(&ctx, Instant::now());
        assert!(!decision.use_humor);
    }

    #[test]
    fn test_humor_low_relationship() {
        let mut engine = HumorEngine::new(HumorConfig::default());
        let ctx = make_context(0.2, 0.9, 0.8);
        let decision = engine.decide(&ctx, Instant::now());
        assert!(!decision.use_humor);
    }

    #[test]
    fn test_humor_appropriate() {
        let mut engine = HumorEngine::new(HumorConfig::default());
        let ctx = make_context(0.8, 0.9, 0.8);
        let decision = engine.decide(&ctx, Instant::now());
        assert!(decision.use_humor);
    }

    #[test]
    fn test_humor_cooldown() {
        let mut engine = HumorEngine::new(HumorConfig::default());
        let ctx = make_context(0.8, 0.9, 0.8);
        let now = Instant::now();
        engine.decide(&ctx, now);
        let decision = engine.decide(&ctx, now);
        assert!(!decision.use_humor);
    }
}
