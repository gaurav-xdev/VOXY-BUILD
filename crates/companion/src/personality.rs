use serde::{Deserialize, Serialize};

use crate::types::ExpressionMetadata;

/// Companion personality traits (NOT emotions — behavioral tendencies).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionPersonality {
    /// Calmness level (0.0 - 1.0). Higher = calmer.
    pub calmness: f64,
    /// Helpfulness tendency (0.0 - 1.0).
    pub helpfulness: f64,
    /// Protectiveness (respects focus, doesn't interrupt) (0.0 - 1.0).
    pub protectiveness: f64,
    /// Curiosity (willingness to ask, reference) (0.0 - 1.0).
    pub curiosity: f64,
    /// Professionalism (formality, precision) (0.0 - 1.0).
    pub professionalism: f64,
    /// Reliability (consistency, predictability) (0.0 - 1.0).
    pub reliability: f64,
}

impl CompanionPersonality {
    pub fn default_desktop() -> Self {
        Self {
            calmness: 0.8,
            helpfulness: 0.7,
            protectiveness: 0.9,
            curiosity: 0.5,
            professionalism: 0.7,
            reliability: 0.9,
        }
    }

    pub fn default_robot() -> Self {
        Self {
            calmness: 0.9,
            helpfulness: 0.8,
            protectiveness: 0.7,
            curiosity: 0.4,
            professionalism: 0.8,
            reliability: 0.95,
        }
    }

    pub fn default_mobile() -> Self {
        Self {
            calmness: 0.7,
            helpfulness: 0.8,
            protectiveness: 0.85,
            curiosity: 0.6,
            professionalism: 0.6,
            reliability: 0.85,
        }
    }

    /// Compute expression metadata from personality and context.
    pub fn express(
        &self,
        focus_level: f64,
        activity_name: &str,
        energy: f64,
    ) -> ExpressionMetadata {
        use crate::types::OrbState;

        let orb_state = if focus_level >= 0.8 {
            OrbState::Focused
        } else if energy < 0.3 {
            OrbState::Resting
        } else if energy > 0.7 {
            OrbState::Alert
        } else {
            OrbState::Calm
        };

        let urgency = if focus_level > 0.9 { 0.2 } else { 0.5 };

        let confidence = self.reliability * 0.5 + self.calmness * 0.5;

        ExpressionMetadata {
            orb_state,
            energy,
            urgency,
            confidence,
            context_tags: vec![activity_name.to_string()],
        }
    }

    /// Should this personality interrupt the user given current conditions?
    pub fn should_interrupt(&self, focus_level: f64, importance: f64) -> bool {
        let threshold = self.protectiveness * 0.8 + (1.0 - self.helpfulness) * 0.2;
        importance > threshold && focus_level < self.protectiveness
    }

    /// How formal should the next response be?
    pub fn formality_level(&self) -> f64 {
        self.professionalism * 0.7 + self.calmness * 0.3
    }
}

impl Default for CompanionPersonality {
    fn default() -> Self {
        Self::default_desktop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_express() {
        let p = CompanionPersonality::default_desktop();
        let meta = p.express(0.8, "Coding", 0.6);
        assert_eq!(meta.orb_state, crate::types::OrbState::Focused);
    }

    #[test]
    fn test_interrupt_respects_protectiveness() {
        let p = CompanionPersonality::default_desktop();
        assert!(!p.should_interrupt(0.9, 0.3));
        assert!(p.should_interrupt(0.2, 0.9));
    }

    #[test]
    fn test_formality() {
        let p = CompanionPersonality::default_desktop();
        let f = p.formality_level();
        assert!(f > 0.5);
    }

    #[test]
    fn test_robot_personality() {
        let p = CompanionPersonality::default_robot();
        assert!(p.reliability > 0.9);
    }
}
