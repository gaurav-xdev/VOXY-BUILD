use crate::audit::AuditLog;
use crate::capability::{CapabilityRegistry, RiskLevel};
use crate::consent::ConsentManager;
use crate::integrity::IntegrityVerifier;
use crate::policy::{AuditLevel, PolicyEngine, PolicyInput, PolicyResult};
use crate::rollback::RollbackManager;
use crate::threat::{ThreatDetector, ThreatSeverity};
use crate::trust::TrustManager;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GuardianDecision {
    pub allowed: bool,
    pub requires_consent: bool,
    pub requires_mfa: bool,
    pub audit_level: AuditLevel,
    pub reason: String,
    pub policy_result: Option<PolicyResult>,
    pub execution_id: Uuid,
}

pub struct GuardianConfig {
    pub require_threat_analysis: bool,
    pub require_integrity_check: bool,
    pub auto_recovery: bool,
    pub max_risk_without_consent: RiskLevel,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            require_threat_analysis: true,
            require_integrity_check: true,
            auto_recovery: false,
            max_risk_without_consent: RiskLevel::Low,
        }
    }
}

pub struct GuardianEngine {
    pub capability_registry: CapabilityRegistry,
    pub policy_engine: PolicyEngine,
    pub consent_manager: Mutex<ConsentManager>,
    pub trust_manager: Mutex<TrustManager>,
    pub threat_detector: Mutex<ThreatDetector>,
    pub integrity_verifier: IntegrityVerifier,
    pub audit_log: Mutex<AuditLog>,
    pub rollback_manager: Mutex<RollbackManager>,
    config: GuardianConfig,
    /// SECURITY: Rate limiter — per-subject request timestamps (millis since epoch).
    rate_limiter: Mutex<HashMap<String, Vec<u64>>>,
}

impl GuardianEngine {
    pub fn new(
        capability_registry: CapabilityRegistry,
        policy_engine: PolicyEngine,
        config: GuardianConfig,
    ) -> Self {
        Self {
            capability_registry,
            policy_engine,
            consent_manager: Mutex::new(ConsentManager::new()),
            trust_manager: Mutex::new(TrustManager::new()),
            threat_detector: Mutex::new(ThreatDetector::new()),
            integrity_verifier: IntegrityVerifier::new(),
            audit_log: Mutex::new(AuditLog::new()),
            rollback_manager: Mutex::new(RollbackManager::new()),
            config,
            rate_limiter: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_consent_manager(mut self, cm: ConsentManager) -> Self {
        self.consent_manager = Mutex::new(cm);
        self
    }

    pub fn with_trust_manager(mut self, tm: TrustManager) -> Self {
        self.trust_manager = Mutex::new(tm);
        self
    }

    pub fn evaluate(
        &self,
        subject: &str,
        capability: &str,
        resource: Option<&str>,
        action: &str,
        context: std::collections::HashMap<String, String>,
    ) -> GuardianDecision {
        let execution_id = Uuid::new_v4();
        let mut audit_log = self.audit_log.lock().unwrap_or_else(|e| e.into_inner());
        let trust_level = {
            let trust_manager = self.trust_manager.lock().unwrap_or_else(|e| e.into_inner());
            trust_manager.get(subject)
        };

        let capability_match = self.capability_registry.find_matching(capability);
        let capability_entry = match capability_match.first() {
            Some(c) => *c,
            None => {
                audit_log.record(
                    subject,
                    action,
                    resource,
                    "denied",
                    Some("Unknown capability"),
                    "unknown",
                    "unknown",
                    AuditLevel::Basic,
                );
                return GuardianDecision {
                    allowed: false,
                    requires_consent: false,
                    requires_mfa: false,
                    audit_level: AuditLevel::Basic,
                    reason: format!("Unknown capability: {capability}"),
                    policy_result: None,
                    execution_id,
                };
            }
        };
        let risk_level = capability_entry.risk_level;

        if trust_level.is_blocked() {
            audit_log.record(
                subject,
                action,
                resource,
                "denied",
                Some("Entity is blocked"),
                &format!("{risk_level:?}"),
                &format!("{trust_level:?}"),
                AuditLevel::Detailed,
            );
            return GuardianDecision {
                allowed: false,
                requires_consent: false,
                requires_mfa: false,
                audit_level: AuditLevel::Detailed,
                reason: format!("Entity '{subject}' is blocked (trust: {trust_level:?})"),
                policy_result: None,
                execution_id,
            };
        }

        // SECURITY: Rate limiting — max 30 requests per subject per 60-second window.
        // This prevents abuse from compromised or overly aggressive entities.
        {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let mut limiter = self.rate_limiter.lock().unwrap_or_else(|e| e.into_inner());
            let timestamps = limiter.entry(subject.to_string()).or_default();
            // Evict entries older than 60 seconds
            timestamps.retain(|&t| now_ms.saturating_sub(t) <= 60_000);
            if timestamps.len() >= 30 {
                audit_log.record(
                    subject,
                    action,
                    resource,
                    "blocked",
                    Some("Rate limit exceeded"),
                    &format!("{risk_level:?}"),
                    &format!("{trust_level:?}"),
                    AuditLevel::Full,
                );
                return GuardianDecision {
                    allowed: false,
                    requires_consent: false,
                    requires_mfa: false,
                    audit_level: AuditLevel::Full,
                    reason: format!(
                        "Rate limit exceeded for '{subject}': {} requests in 60s window",
                        timestamps.len()
                    ),
                    policy_result: None,
                    execution_id,
                };
            }
            timestamps.push(now_ms);
        }

        let policy_input = PolicyInput {
            subject: subject.to_string(),
            capability: capability.to_string(),
            resource: resource.map(|r| r.to_string()),
            action: action.to_string(),
            risk_level: format!("{:?}", risk_level).to_lowercase(),
            trust_level: format!("{:?}", trust_level).to_lowercase(),
            context,
        };

        // SECURITY: No auto-grant path. All actions go through policy evaluation.
        // Trust level is passed to the policy engine so rules can factor it in,
        // but no action bypasses evaluation entirely.
        let policy_result = self.policy_engine.evaluate(&policy_input);
        let policy_denies = !policy_result.allowed
            && !policy_result.requires_consent
            && !policy_result.requires_mfa;

        if policy_denies {
            let is_medium_plus = risk_level >= RiskLevel::Medium;
            let is_critical = risk_level >= RiskLevel::Critical;

            if is_critical {
                let consent_manager = self
                    .consent_manager
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let has_consent = consent_manager.is_granted(subject, capability);
                drop(consent_manager);
                if !has_consent {
                    audit_log.record(
                        subject,
                        action,
                        resource,
                        "pending",
                        Some("Requires MFA + consent for critical risk"),
                        &format!("{risk_level:?}"),
                        &format!("{trust_level:?}"),
                        AuditLevel::Full,
                    );
                    return GuardianDecision {
                        allowed: false,
                        requires_consent: true,
                        requires_mfa: true,
                        audit_level: AuditLevel::Full,
                        reason: "Critical risk requires MFA and consent".to_string(),
                        policy_result: Some(policy_result),
                        execution_id,
                    };
                }
            } else if is_medium_plus {
                let consent_manager = self
                    .consent_manager
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let has_consent = consent_manager.is_granted(subject, capability);
                drop(consent_manager);
                if !has_consent {
                    audit_log.record(
                        subject,
                        action,
                        resource,
                        "pending",
                        Some("Requires consent for medium+ risk"),
                        &format!("{risk_level:?}"),
                        &format!("{trust_level:?}"),
                        AuditLevel::Detailed,
                    );
                    return GuardianDecision {
                        allowed: false,
                        requires_consent: true,
                        requires_mfa: false,
                        audit_level: AuditLevel::Detailed,
                        reason: "Medium risk requires consent".to_string(),
                        policy_result: Some(policy_result),
                        execution_id,
                    };
                }
            }

            let reason = policy_result
                .reason
                .clone()
                .unwrap_or_else(|| "Policy denied".to_string());
            audit_log.record(
                subject,
                action,
                resource,
                "denied",
                Some(&reason),
                &format!("{risk_level:?}"),
                &format!("{trust_level:?}"),
                policy_result.audit_level,
            );
            return GuardianDecision {
                allowed: false,
                requires_consent: false,
                requires_mfa: false,
                audit_level: policy_result.audit_level,
                reason,
                policy_result: Some(policy_result),
                execution_id,
            };
        }

        let consent_manager = self
            .consent_manager
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let requires_consent = if risk_level >= RiskLevel::Medium
            && risk_level > self.config.max_risk_without_consent
        {
            if !consent_manager.is_granted(subject, capability) {
                true
            } else {
                policy_result.requires_consent
            }
        } else {
            false
        };
        drop(consent_manager);

        let requires_mfa = risk_level >= RiskLevel::Critical || policy_result.requires_mfa;

        if self.config.require_threat_analysis && risk_level >= RiskLevel::High {
            let mut threat_detector = self
                .threat_detector
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if threat_detector.rapid_failure_detected(subject, 60, 3) {
                threat_detector.record_event(
                    subject,
                    "rapid_requests",
                    ThreatSeverity::Medium,
                    &format!("Rapid {action} requests from {subject}"),
                );
                audit_log.record(
                    subject,
                    action,
                    resource,
                    "blocked",
                    Some("Rapid request rate detected"),
                    &format!("{risk_level:?}"),
                    &format!("{trust_level:?}"),
                    AuditLevel::Full,
                );
                return GuardianDecision {
                    allowed: false,
                    requires_consent: false,
                    requires_mfa: false,
                    audit_level: AuditLevel::Full,
                    reason: "Rapid request rate detected by threat analysis".to_string(),
                    policy_result: Some(policy_result),
                    execution_id,
                };
            }
        }

        let result = if requires_consent || requires_mfa {
            "pending"
        } else {
            "allowed"
        };

        audit_log.record(
            subject,
            action,
            resource,
            result,
            None,
            &format!("{risk_level:?}"),
            &format!("{trust_level:?}"),
            policy_result.audit_level,
        );

        GuardianDecision {
            allowed: !requires_consent && !requires_mfa,
            requires_consent,
            requires_mfa,
            audit_level: policy_result.audit_level,
            reason: policy_result
                .reason
                .clone()
                .unwrap_or_else(|| result.to_string()),
            policy_result: Some(policy_result),
            execution_id,
        }
    }

    pub fn handle_consent_granted(
        &self,
        _execution_id: Uuid,
        subject: &str,
        capability: &str,
    ) -> bool {
        let mut audit_log = self.audit_log.lock().unwrap_or_else(|e| e.into_inner());
        audit_log.record(
            subject,
            capability,
            None,
            "consent_granted",
            None,
            "medium",
            "trusted",
            AuditLevel::Detailed,
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{Capability, CapabilityCategory};

    fn make_engine() -> GuardianEngine {
        let mut registry = CapabilityRegistry::new();
        let _ = registry.register(Capability::new(
            "test:low",
            "Low Risk",
            RiskLevel::Low,
            CapabilityCategory::System,
        ));
        let _ = registry.register(Capability::new(
            "test:high",
            "High Risk",
            RiskLevel::High,
            CapabilityCategory::System,
        ));
        let _ = registry.register(Capability::new(
            "test:critical",
            "Critical Risk",
            RiskLevel::Critical,
            CapabilityCategory::System,
        ));
        let mut trust = TrustManager::new();
        trust.set("user-1", crate::trust::TrustLevel::Verified);

        // Add explicit allow rules — auto-grants are disabled so policy must evaluate
        let mut policy = PolicyEngine::new();
        policy.add_rule(crate::policy::PolicyRule {
            id: "allow-low-risk".to_string(),
            description: "Allow low risk actions".to_string(),
            effect: crate::policy::PolicyEffect::Allow,
            capabilities: vec!["test:low".to_string()],
            conditions: vec![crate::policy::PolicyCondition::TrustLevelAtLeast(
                "unknown".to_string(),
            )],
            priority: 100,
        });
        policy.add_rule(crate::policy::PolicyRule {
            id: "allow-high-for-verified".to_string(),
            description: "Allow high risk for verified".to_string(),
            effect: crate::policy::PolicyEffect::RequireConsent,
            capabilities: vec!["test:high".to_string()],
            conditions: vec![crate::policy::PolicyCondition::TrustLevelAtLeast(
                "verified".to_string(),
            )],
            priority: 50,
        });

        let engine = GuardianEngine::new(registry, policy, GuardianConfig::default())
            .with_trust_manager(trust);
        engine
    }

    #[test]
    fn guardian_allows_low_risk() {
        let engine = make_engine();
        let decision = engine.evaluate(
            "user-1",
            "test:low",
            None,
            "read",
            std::collections::HashMap::new(),
        );
        assert!(decision.allowed);
    }

    #[test]
    fn guardian_requires_consent_for_high() {
        let engine = make_engine();
        let decision = engine.evaluate(
            "user-1",
            "test:high",
            None,
            "write",
            std::collections::HashMap::new(),
        );
        assert!(decision.requires_consent || decision.allowed);
    }

    #[test]
    fn guardian_blocks_unknown_capability() {
        let engine = make_engine();
        let decision = engine.evaluate(
            "user-1",
            "unknown:cap",
            None,
            "execute",
            std::collections::HashMap::new(),
        );
        assert!(!decision.allowed);
    }

    #[test]
    fn guardian_blocks_blocked_entity() {
        let mut trust = TrustManager::new();
        trust.set("bad-actor", crate::trust::TrustLevel::Blocked);
        let engine = GuardianEngine::new(
            CapabilityRegistry::new(),
            PolicyEngine::new(),
            GuardianConfig::default(),
        )
        .with_trust_manager(trust);
        let decision = engine.evaluate(
            "bad-actor",
            "test:low",
            None,
            "read",
            std::collections::HashMap::new(),
        );
        assert!(!decision.allowed);
    }

    #[test]
    fn guardian_requires_mfa_for_critical() {
        let engine = make_engine();
        let decision = engine.evaluate(
            "user-1",
            "test:critical",
            None,
            "delete",
            std::collections::HashMap::new(),
        );
        assert!(decision.requires_mfa || decision.allowed);
    }

    #[test]
    fn guardian_tracks_execution_id() {
        let engine = make_engine();
        let d1 = engine.evaluate(
            "user-1",
            "test:low",
            None,
            "read",
            std::collections::HashMap::new(),
        );
        let d2 = engine.evaluate(
            "user-1",
            "test:low",
            None,
            "read",
            std::collections::HashMap::new(),
        );
        assert_ne!(d1.execution_id, d2.execution_id);
    }
}
