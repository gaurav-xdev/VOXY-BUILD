use crate::config::ProtectionConfig;
use crate::types::{Action, ActionKind, ProtectionDecision, ProtectionLevel};

/// Protection engine — determines when VOXY should refuse or delay actions.
pub struct ProtectionEngine {
    config: ProtectionConfig,
    refusal_count: usize,
    override_count: usize,
}

impl ProtectionEngine {
    pub fn new(config: ProtectionConfig) -> Self {
        Self {
            config,
            refusal_count: 0,
            override_count: 0,
        }
    }

    /// Evaluate whether an action is permitted.
    pub fn evaluate(
        &mut self,
        action: &Action,
        trust_score: f64,
        autonomy_level: f64,
        is_meeting: bool,
        focus_level: f64,
    ) -> ProtectionDecision {
        if is_meeting && action.protection_level as u8 >= ProtectionLevel::Medium as u8 {
            self.refusal_count += 1;
            return ProtectionDecision {
                allowed: false,
                reason: "Meeting in progress — action deferred".to_string(),
                requires_confirmation: false,
                alternative: Some("Wait until meeting ends".to_string()),
            };
        }

        if focus_level > 0.85 && action.protection_level as u8 >= ProtectionLevel::High as u8 {
            self.refusal_count += 1;
            return ProtectionDecision {
                allowed: false,
                reason: "User in deep focus — action deferred".to_string(),
                requires_confirmation: false,
                alternative: Some("Wait for natural pause".to_string()),
            };
        }

        if self.config.auto_protect_delete
            && action.kind == ActionKind::Delete
            && (!action.reversible || action.impact > self.config.max_reversible_impact)
        {
            self.refusal_count += 1;
            return ProtectionDecision {
                allowed: false,
                reason: "Destructive action requires explicit confirmation".to_string(),
                requires_confirmation: true,
                alternative: Some("Move to trash instead of permanent delete".to_string()),
            };
        }

        if action.protection_level == ProtectionLevel::Critical {
            self.refusal_count += 1;
            return ProtectionDecision {
                allowed: false,
                reason: "Critical action requires manual execution".to_string(),
                requires_confirmation: true,
                alternative: Some("Execute manually for safety".to_string()),
            };
        }

        if action.protection_level == ProtectionLevel::High && autonomy_level < 0.8 {
            self.refusal_count += 1;
            return ProtectionDecision {
                allowed: false,
                reason: "High-impact action requires higher trust level".to_string(),
                requires_confirmation: true,
                alternative: None,
            };
        }

        let requires_confirmation =
            action.protection_level.confirmation_required() && trust_score < 0.9;

        ProtectionDecision {
            allowed: true,
            reason: "Action approved".to_string(),
            requires_confirmation,
            alternative: None,
        }
    }

    pub fn refusal_count(&self) -> usize {
        self.refusal_count
    }

    pub fn override_count(&self) -> usize {
        self.override_count
    }

    pub fn record_override(&mut self) {
        self.override_count += 1;
    }
}

impl Default for ProtectionEngine {
    fn default() -> Self {
        Self::new(ProtectionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_action(level: ProtectionLevel, kind: ActionKind) -> Action {
        Action {
            id: "test".to_string(),
            kind,
            description: "Test action".to_string(),
            protection_level: level,
            reversible: true,
            impact: 0.3,
        }
    }

    #[test]
    fn test_low_protection_allowed() {
        let mut engine = ProtectionEngine::new(ProtectionConfig::default());
        let action = test_action(ProtectionLevel::Low, ActionKind::Speak);
        let decision = engine.evaluate(&action, 0.5, 0.5, false, 0.3);
        assert!(decision.allowed);
    }

    #[test]
    fn test_critical_protection_denied() {
        let mut engine = ProtectionEngine::new(ProtectionConfig::default());
        let action = test_action(ProtectionLevel::Critical, ActionKind::Delete);
        let decision = engine.evaluate(&action, 0.9, 0.9, false, 0.3);
        assert!(!decision.allowed);
    }

    #[test]
    fn test_meeting_blocks() {
        let mut engine = ProtectionEngine::new(ProtectionConfig::default());
        let action = test_action(ProtectionLevel::Medium, ActionKind::Speak);
        let decision = engine.evaluate(&action, 0.9, 0.9, true, 0.3);
        assert!(!decision.allowed);
    }

    #[test]
    fn test_deep_focus_blocks() {
        let mut engine = ProtectionEngine::new(ProtectionConfig::default());
        let action = test_action(ProtectionLevel::High, ActionKind::Speak);
        let decision = engine.evaluate(&action, 0.9, 0.9, false, 0.9);
        assert!(!decision.allowed);
    }

    #[test]
    fn test_delete_auto_protected() {
        let mut engine = ProtectionEngine::new(ProtectionConfig::default());
        let action = Action {
            id: "test".to_string(),
            kind: ActionKind::Delete,
            description: "Test action".to_string(),
            protection_level: ProtectionLevel::Low,
            reversible: false,
            impact: 0.3,
        };
        let decision = engine.evaluate(&action, 0.5, 0.5, false, 0.3);
        assert!(!decision.allowed);
    }
}
