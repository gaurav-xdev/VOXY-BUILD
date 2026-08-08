#[derive(Debug, Clone, PartialEq)]
pub enum PolicyEnforcementMode {
    Enforce,
    WarnOnly,
    AuditOnly,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct GuardianConfig {
    pub enabled: bool,
    pub require_authentication: bool,
    pub require_consent_for_high_risk: bool,
    pub require_mfa_for_critical: bool,
    pub audit_enabled: bool,
    pub max_audit_entries: usize,
    pub policy_enforcement: PolicyEnforcementMode,
    pub default_decision_timeout_ms: u64,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_authentication: true,
            require_consent_for_high_risk: true,
            require_mfa_for_critical: true,
            audit_enabled: true,
            max_audit_entries: 10000,
            policy_enforcement: PolicyEnforcementMode::Enforce,
            default_decision_timeout_ms: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardian_config_defaults() {
        let config = GuardianConfig::default();
        assert!(config.enabled);
        assert!(config.require_authentication);
        assert!(config.require_consent_for_high_risk);
        assert!(config.require_mfa_for_critical);
        assert!(config.audit_enabled);
        assert_eq!(config.max_audit_entries, 10000);
        assert_eq!(config.policy_enforcement, PolicyEnforcementMode::Enforce);
        assert_eq!(config.default_decision_timeout_ms, 5000);
    }
}
