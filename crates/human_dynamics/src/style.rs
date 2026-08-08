use crate::config::StyleConfig;
use crate::types::{InteractionStyle, RelationshipLevel, SentenceLength};

/// Interaction style engine — adapts communication style without changing personality.
pub struct StyleEngine {
    config: StyleConfig,
    current: InteractionStyle,
}

impl StyleEngine {
    pub fn new(config: StyleConfig) -> Self {
        Self {
            config,
            current: InteractionStyle::professional(),
        }
    }

    /// Adapt style based on relationship and context.
    pub fn adapt(
        &mut self,
        relationship: RelationshipLevel,
        focus_level: f64,
        time_pressure: f64,
        topic_complexity: f64,
    ) -> InteractionStyle {
        if !self.config.adapt_to_relationship {
            return self.current.clone();
        }

        let base = match relationship {
            RelationshipLevel::Professional => InteractionStyle::professional(),
            RelationshipLevel::Familiar => InteractionStyle::familiar(),
            RelationshipLevel::Trusted | RelationshipLevel::LongTermCompanion => {
                InteractionStyle::companion()
            }
        };

        let mut style = base;

        if focus_level > 0.8 {
            style.sentence_length = SentenceLength::Terse;
            style.verbosity = 0.2;
        } else if focus_level > 0.6 {
            style.sentence_length = SentenceLength::Short;
            style.verbosity = 0.3;
        }

        if time_pressure > 0.7 {
            style.pace = 0.9;
            style.sentence_length = SentenceLength::Terse;
        }

        if topic_complexity > 0.7 {
            style.verbosity = (style.verbosity + 0.2).min(self.config.max_verbosity);
            style.sentence_length = SentenceLength::Long;
        }

        style.formality = style
            .formality
            .clamp(self.config.min_formality, self.config.max_formality);
        style.verbosity = style
            .verbosity
            .clamp(self.config.min_verbosity, self.config.max_verbosity);

        self.current = style.clone();
        style
    }

    pub fn current(&self) -> &InteractionStyle {
        &self.current
    }
}

impl Default for StyleEngine {
    fn default() -> Self {
        Self::new(StyleConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_professional_style() {
        let mut engine = StyleEngine::new(StyleConfig::default());
        let style = engine.adapt(RelationshipLevel::Professional, 0.5, 0.3, 0.3);
        assert!(style.formality > 0.6);
    }

    #[test]
    fn test_companion_style() {
        let mut engine = StyleEngine::new(StyleConfig::default());
        let style = engine.adapt(RelationshipLevel::LongTermCompanion, 0.5, 0.3, 0.3);
        assert!(style.formality < 0.5);
    }

    #[test]
    fn test_focus_adapts() {
        let mut engine = StyleEngine::new(StyleConfig::default());
        let style = engine.adapt(RelationshipLevel::Trusted, 0.9, 0.3, 0.3);
        assert_eq!(style.sentence_length, SentenceLength::Terse);
    }

    #[test]
    fn test_time_pressure_adapts() {
        let mut engine = StyleEngine::new(StyleConfig::default());
        let style = engine.adapt(RelationshipLevel::Trusted, 0.5, 0.9, 0.3);
        assert!(style.pace > 0.8);
    }

    #[test]
    fn test_formality_clamped() {
        let mut engine = StyleEngine::new(StyleConfig::default());
        let style = engine.adapt(RelationshipLevel::Professional, 0.5, 0.3, 0.9);
        assert!(style.formality >= 0.2);
        assert!(style.formality <= 0.9);
    }
}
