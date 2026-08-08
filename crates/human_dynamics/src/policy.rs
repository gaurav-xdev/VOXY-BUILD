use serde::{Deserialize, Serialize};

use crate::config::PolicyConfig;
use crate::types::{Action, ActionKind, BehaviorState, ProtectionLevel};

/// Policy violation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub action_description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Behavior policy engine — global behavior rules.
pub struct PolicyEngine {
    config: PolicyConfig,
    violations: Vec<PolicyViolation>,
    violation_count: usize,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self {
            config,
            violations: Vec::new(),
            violation_count: 0,
        }
    }

    /// Check if an action violates any policies.
    pub fn check(
        &mut self,
        action: &Action,
        state: BehaviorState,
        trust_score: f64,
    ) -> Vec<String> {
        let mut violations = Vec::new();

        if self.config.never_interrupt_meetings
            && state == BehaviorState::Observing
            && action.kind == ActionKind::Speak
            && action.protection_level as u8 >= ProtectionLevel::Medium as u8
        {
            violations.push("Never interrupt meetings".to_string());
        }

        if self.config.never_joke_on_failure
            && (action.description.to_lowercase().contains("humor")
                || action.description.to_lowercase().contains("joke"))
            && trust_score < 0.5
        {
            violations.push("Never joke during failures or low trust".to_string());
        }

        if self.config.never_celebrate_partial
            && action.kind == ActionKind::Speak
            && action.description.to_lowercase().contains("celebrate")
        {
            violations.push("Never celebrate partial success".to_string());
        }

        if self.config.protect_before_obey
            && action.protection_level == ProtectionLevel::Critical
            && trust_score < 0.9
        {
            violations.push("Protect before obey — critical action at low trust".to_string());
        }

        if self.config.always_explain_refusal
            && action.kind == ActionKind::Speak
            && action.description.to_lowercase().contains("refuse")
            && !action.description.to_lowercase().contains("explain")
        {
            violations.push("Always explain refusals".to_string());
        }

        for v in &violations {
            self.violations.push(PolicyViolation {
                rule: v.clone(),
                action_description: action.description.clone(),
                timestamp: chrono::Utc::now(),
            });
            self.violation_count += 1;
        }

        violations
    }

    pub fn violation_count(&self) -> usize {
        self.violation_count
    }

    pub fn recent_violations(&self) -> &[PolicyViolation] {
        &self.violations
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new(PolicyConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_action(kind: ActionKind, level: ProtectionLevel, desc: &str) -> Action {
        Action {
            id: "test".to_string(),
            kind,
            description: desc.to_string(),
            protection_level: level,
            reversible: true,
            impact: 0.3,
        }
    }

    #[test]
    fn test_meeting_interruption_blocked() {
        let mut engine = PolicyEngine::new(PolicyConfig::default());
        let action = test_action(
            ActionKind::Speak,
            ProtectionLevel::Medium,
            "Important update",
        );
        let violations = engine.check(&action, BehaviorState::Observing, 0.8);
        assert!(violations.iter().any(|v| v.contains("meeting")));
    }

    #[test]
    fn test_no_violation_normal_speech() {
        let mut engine = PolicyEngine::new(PolicyConfig::default());
        let action = test_action(ActionKind::Speak, ProtectionLevel::Low, "Hello");
        let violations = engine.check(&action, BehaviorState::Listening, 0.8);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_protect_before_obey() {
        let mut engine = PolicyEngine::new(PolicyConfig::default());
        let action = test_action(ActionKind::Delete, ProtectionLevel::Critical, "Delete all");
        let violations = engine.check(&action, BehaviorState::Working, 0.5);
        assert!(violations.iter().any(|v| v.contains("Protect")));
    }
}
