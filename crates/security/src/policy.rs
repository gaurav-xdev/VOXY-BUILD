use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInput {
    pub subject: String,
    pub capability: String,
    pub resource: Option<String>,
    pub action: String,
    pub risk_level: String,
    pub trust_level: String,
    pub context: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub conditions: Vec<String>,
    pub requires_consent: bool,
    pub requires_mfa: bool,
    pub audit_level: AuditLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditLevel {
    None,
    Basic,
    Detailed,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub description: String,
    pub effect: PolicyEffect,
    pub capabilities: Vec<String>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    Allow,
    Deny,
    RequireConsent,
    RequireMfa,
    Audit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    TrustLevelAtLeast(String),
    RiskLevelAtMost(String),
    ResourceMatches(String),
    TimeBetween { start: String, end: String },
    Custom { key: String, value: String },
}

pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        self.rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
    }

    pub fn evaluate(&self, input: &PolicyInput) -> PolicyResult {
        let mut result = PolicyResult {
            allowed: false,
            reason: None,
            conditions: Vec::new(),
            requires_consent: false,
            requires_mfa: false,
            audit_level: AuditLevel::Basic,
        };

        for rule in &self.rules {
            if !self.capability_matches(&rule.capabilities, &input.capability) {
                continue;
            }

            let conditions_met = rule
                .conditions
                .iter()
                .all(|c| self.evaluate_condition(c, input));

            if !conditions_met {
                continue;
            }

            result.conditions.push(rule.id.clone());

            match rule.effect {
                PolicyEffect::Allow => {
                    result.allowed = true;
                    result.reason = Some(format!("Rule '{}' allowed", rule.id));
                    return result;
                }
                PolicyEffect::Deny => {
                    result.allowed = false;
                    result.reason = Some(format!("Rule '{}' denied", rule.id));
                    return result;
                }
                PolicyEffect::RequireConsent => {
                    result.requires_consent = true;
                    result.reason = Some(format!("Rule '{}' requires consent", rule.id));
                    return result;
                }
                PolicyEffect::RequireMfa => {
                    result.requires_mfa = true;
                    result.reason = Some(format!("Rule '{}' requires MFA", rule.id));
                    return result;
                }
                PolicyEffect::Audit => {
                    result.audit_level = AuditLevel::Full;
                }
            }
        }

        result.reason = Some("No matching policy rule".to_string());
        result
    }

    fn capability_matches(&self, patterns: &[String], capability: &str) -> bool {
        patterns.iter().any(|p| {
            p == "*"
                || p == capability
                || (p.ends_with(":*") && capability.starts_with(&p[..p.len() - 2]))
                || (p.ends_with(':') && capability.starts_with(&p[..p.len()]))
        })
    }

    fn evaluate_condition(&self, condition: &PolicyCondition, input: &PolicyInput) -> bool {
        match condition {
            PolicyCondition::TrustLevelAtLeast(min) => {
                let trust_order = ["unknown", "known", "trusted", "verified"];
                let idx = trust_order
                    .iter()
                    .position(|&t| t == input.trust_level)
                    .unwrap_or(0);
                let min_idx = trust_order.iter().position(|&t| t == min).unwrap_or(0);
                idx >= min_idx
            }
            PolicyCondition::RiskLevelAtMost(max) => {
                let risk_order = ["none", "low", "medium", "high", "critical"];
                let idx = risk_order
                    .iter()
                    .position(|&r| r == input.risk_level)
                    .unwrap_or(0);
                let max_idx = risk_order.iter().position(|&r| r == max).unwrap_or(0);
                idx <= max_idx
            }
            PolicyCondition::ResourceMatches(pattern) => input
                .resource
                .as_ref()
                .map(|r| r == pattern || pattern == "*")
                .unwrap_or(false),
            PolicyCondition::TimeBetween { .. } => true,
            PolicyCondition::Custom { key, value } => {
                input.context.get(key).map(|v| v == value).unwrap_or(false)
            }
        }
    }

    pub fn rules(&self) -> &[PolicyRule] {
        &self.rules
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(trust: &str, risk: &str, capability: &str) -> PolicyInput {
        PolicyInput {
            subject: "test".to_string(),
            capability: capability.to_string(),
            resource: None,
            action: "execute".to_string(),
            risk_level: risk.to_string(),
            trust_level: trust.to_string(),
            context: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn policy_allow_rule() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: "allow-voice".to_string(),
            description: "Allow voice capture".to_string(),
            effect: PolicyEffect::Allow,
            capabilities: vec!["audio:capture".to_string()],
            conditions: vec![PolicyCondition::TrustLevelAtLeast("trusted".to_string())],
            priority: 100,
        });
        let result = engine.evaluate(&make_input("trusted", "low", "audio:capture"));
        assert!(result.allowed);
    }

    #[test]
    fn policy_deny_rule() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: "deny-all".to_string(),
            description: "Deny everything".to_string(),
            effect: PolicyEffect::Deny,
            capabilities: vec!["*".to_string()],
            conditions: vec![],
            priority: 0,
        });
        let result = engine.evaluate(&make_input("unknown", "low", "anything"));
        assert!(!result.allowed);
    }

    #[test]
    fn policy_priority_ordering() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: "deny-dangerous".to_string(),
            description: "Deny dangerous".to_string(),
            effect: PolicyEffect::Deny,
            capabilities: vec!["admin:*".to_string()],
            conditions: vec![],
            priority: 200,
        });
        engine.add_rule(PolicyRule {
            id: "allow-all".to_string(),
            description: "Allow all".to_string(),
            effect: PolicyEffect::Allow,
            capabilities: vec!["*".to_string()],
            conditions: vec![],
            priority: 100,
        });
        assert!(
            engine
                .evaluate(&make_input("trusted", "low", "audio:capture"))
                .allowed
        );
        assert!(
            !engine
                .evaluate(&make_input("trusted", "critical", "admin:delete"))
                .allowed
        );
    }

    #[test]
    fn policy_requires_consent() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: "consent-needed".to_string(),
            description: "Consent required".to_string(),
            effect: PolicyEffect::RequireConsent,
            capabilities: vec!["screen:capture".to_string()],
            conditions: vec![],
            priority: 100,
        });
        let result = engine.evaluate(&make_input("trusted", "high", "screen:capture"));
        assert!(!result.allowed);
        assert!(result.requires_consent);
    }

    #[test]
    fn condition_trust_level() {
        let cond = PolicyCondition::TrustLevelAtLeast("trusted".to_string());
        let mut input = make_input("verified", "low", "test");
        assert!(PolicyEngine::new().evaluate_condition(&cond, &input));
        input.trust_level = "unknown".to_string();
        assert!(!PolicyEngine::new().evaluate_condition(&cond, &input));
    }

    #[test]
    fn condition_resource_match() {
        let cond = PolicyCondition::ResourceMatches("namespace:data".to_string());
        let mut input = make_input("trusted", "low", "test");
        input.resource = Some("namespace:data".to_string());
        assert!(PolicyEngine::new().evaluate_condition(&cond, &input));
        input.resource = Some("other".to_string());
        assert!(!PolicyEngine::new().evaluate_condition(&cond, &input));
    }

    #[test]
    fn condition_custom() {
        let cond = PolicyCondition::Custom {
            key: "region".to_string(),
            value: "us-east".to_string(),
        };
        let mut input = make_input("trusted", "low", "test");
        input
            .context
            .insert("region".to_string(), "us-east".to_string());
        assert!(PolicyEngine::new().evaluate_condition(&cond, &input));
        input
            .context
            .insert("region".to_string(), "eu-west".to_string());
        assert!(!PolicyEngine::new().evaluate_condition(&cond, &input));
    }

    #[test]
    fn audit_level_ordering() {
        assert!(AuditLevel::None < AuditLevel::Basic);
        assert!(AuditLevel::Basic < AuditLevel::Detailed);
        assert!(AuditLevel::Detailed < AuditLevel::Full);
    }
}
