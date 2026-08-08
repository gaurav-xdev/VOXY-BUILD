use std::cmp::Ordering;
use std::collections::HashMap;

use async_trait::async_trait;

use crate::config::GuardianConfig;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct AccessRequest {
    pub id: String,
    pub subject: String,
    pub action: String,
    pub resource: String,
    pub context: HashMap<String, String>,
    pub risk_level: RiskLevel,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum AccessDecision {
    Granted {
        request_id: String,
        reason: String,
        conditions: Vec<String>,
    },
    Denied {
        request_id: String,
        reason: String,
    },
    PendingConsent {
        request_id: String,
        consent_id: String,
    },
    PendingMfa {
        request_id: String,
        challenge: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl PartialOrd for RiskLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RiskLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        let rank = |r: &RiskLevel| -> u8 {
            match r {
                RiskLevel::None => 0,
                RiskLevel::Low => 1,
                RiskLevel::Medium => 2,
                RiskLevel::High => 3,
                RiskLevel::Critical => 4,
            }
        };
        rank(self).cmp(&rank(other))
    }
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub effect: PolicyEffect,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyEffect {
    Allow,
    Deny,
    RequireConsent,
    RequireMfa,
    Audit,
}

#[derive(Debug, Clone)]
pub enum PolicyCondition {
    SubjectMatch(Vec<String>),
    ActionMatch(Vec<String>),
    ResourceMatch(Vec<String>),
    RiskLevelMax(RiskLevel),
    TimeRestriction { start: String, end: String },
    Custom(String, String),
}

#[async_trait]
pub trait GuardianContract: Send + Sync {
    async fn init(&self, config: &GuardianConfig) -> Result<()>;
    async fn check_access(&self, request: &AccessRequest) -> Result<AccessDecision>;
    async fn request_access(&self, request: &AccessRequest) -> Result<String>;
    async fn grant_consent(&self, request_id: &str, grant: bool) -> Result<AccessDecision>;
    async fn verify_mfa(&self, request_id: &str, token: &str) -> Result<AccessDecision>;
    async fn register_policy(&self, rule: PolicyRule) -> Result<()>;
    async fn remove_policy(&self, policy_id: &str) -> Result<()>;
    async fn list_policies(&self) -> Result<Vec<PolicyRule>>;
    async fn evaluate_policy(&self, request: &AccessRequest) -> Result<Vec<(PolicyRule, bool)>>;
}

#[async_trait]
pub trait PolicyContract: Send + Sync {
    async fn register_rule(&self, rule: PolicyRule) -> Result<()>;
    async fn unregister_rule(&self, rule_id: &str) -> Result<()>;
    async fn evaluate(&self, request: &AccessRequest) -> Result<Vec<PolicyEvaluation>>;
    async fn list_rules(&self) -> Result<Vec<PolicyRule>>;
    async fn clear_rules(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PolicyEvaluation {
    pub rule_id: String,
    pub rule_name: String,
    pub matched: bool,
    pub effect: PolicyEffect,
    pub reason: Option<String>,
}

#[async_trait]
pub trait AuditContract: Send + Sync {
    async fn record(&self, request: &AccessRequest, decision: &AccessDecision) -> Result<String>;
    async fn query(&self, subject: &str) -> Result<Vec<AuditRecord>>;
    async fn query_by_action(&self, action: &str) -> Result<Vec<AuditRecord>>;
    async fn query_by_time_range(
        &self,
        start: &chrono::DateTime<chrono::Utc>,
        end: &chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AuditRecord>>;
    async fn recent(&self, limit: usize) -> Result<Vec<AuditRecord>>;
    async fn export_json(&self) -> Result<String>;
    async fn verify_integrity(&self) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub subject: String,
    pub action: String,
    pub resource: String,
    pub decision: String,
    pub reason: Option<String>,
    pub risk_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_request_creation() {
        let request = AccessRequest {
            id: "req-1".into(),
            subject: "user1".into(),
            action: "read".into(),
            resource: "file.txt".into(),
            context: HashMap::new(),
            risk_level: RiskLevel::Medium,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(request.id, "req-1");
        assert_eq!(request.subject, "user1");
        assert_eq!(request.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_access_decision_variants() {
        let granted = AccessDecision::Granted {
            request_id: "r1".into(),
            reason: "allowed".into(),
            conditions: vec!["cond1".into()],
        };
        let denied = AccessDecision::Denied {
            request_id: "r2".into(),
            reason: "blocked".into(),
        };
        let consent = AccessDecision::PendingConsent {
            request_id: "r3".into(),
            consent_id: "c1".into(),
        };
        let mfa = AccessDecision::PendingMfa {
            request_id: "r4".into(),
            challenge: "ch1".into(),
        };
        match granted {
            AccessDecision::Granted { ref request_id, .. } => assert_eq!(request_id, "r1"),
            _ => panic!("wrong variant"),
        }
        match denied {
            AccessDecision::Denied { ref request_id, .. } => assert_eq!(request_id, "r2"),
            _ => panic!("wrong variant"),
        }
        match consent {
            AccessDecision::PendingConsent { ref request_id, .. } => assert_eq!(request_id, "r3"),
            _ => panic!("wrong variant"),
        }
        match mfa {
            AccessDecision::PendingMfa { ref request_id, .. } => assert_eq!(request_id, "r4"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_policy_rule_creation() {
        let rule = PolicyRule {
            id: "policy-1".into(),
            name: "Admin Access".into(),
            effect: PolicyEffect::Allow,
            conditions: vec![
                PolicyCondition::SubjectMatch(vec!["admin".into()]),
                PolicyCondition::RiskLevelMax(RiskLevel::High),
            ],
            priority: 10,
        };
        assert_eq!(rule.id, "policy-1");
        assert_eq!(rule.priority, 10);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::None < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
        assert!(RiskLevel::Critical > RiskLevel::None);
        assert_eq!(RiskLevel::Medium, RiskLevel::Medium);
    }

    #[test]
    fn test_policy_evaluation() {
        let eval = PolicyEvaluation {
            rule_id: "r1".into(),
            rule_name: "Test Rule".into(),
            matched: true,
            effect: PolicyEffect::Allow,
            reason: Some("matched".into()),
        };
        assert!(eval.matched);
        assert_eq!(eval.rule_id, "r1");
    }

    #[test]
    fn test_audit_record() {
        let record = AuditRecord {
            id: "audit-1".into(),
            timestamp: chrono::Utc::now(),
            subject: "user1".into(),
            action: "delete".into(),
            resource: "file.txt".into(),
            decision: "Denied".into(),
            reason: Some("no permission".into()),
            risk_level: "High".into(),
        };
        assert_eq!(record.subject, "user1");
        assert_eq!(record.decision, "Denied");
    }

    #[test]
    fn test_risk_level_ord_derived() {
        let mut levels = vec![
            RiskLevel::Critical,
            RiskLevel::None,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Low,
        ];
        levels.sort();
        assert_eq!(
            levels,
            vec![
                RiskLevel::None,
                RiskLevel::Low,
                RiskLevel::Medium,
                RiskLevel::High,
                RiskLevel::Critical,
            ]
        );
    }
}
