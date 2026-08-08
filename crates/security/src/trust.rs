use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TrustLevel {
    Compromised,
    Blocked,
    Unknown,
    Known,
    Trusted,
    Verified,
}

impl TrustLevel {
    pub fn max_allowed_risk(&self) -> crate::capability::RiskLevel {
        use crate::capability::RiskLevel;
        match self {
            Self::Verified => RiskLevel::Critical,
            Self::Trusted => RiskLevel::High,
            Self::Known => RiskLevel::Low,
            Self::Unknown => RiskLevel::None,
            Self::Blocked | Self::Compromised => RiskLevel::None,
        }
    }

    /// SECURITY: Auto-grants removed. All actions must go through full
    /// policy evaluation regardless of trust level. This prevents the
    /// auto-grant bypass where Trusted entities could skip policy checks
    /// on low-risk actions. Trust level now only affects *which* policies
    /// apply, not whether policies are evaluated at all.
    pub fn auto_grants(&self, _risk: crate::capability::RiskLevel) -> bool {
        false
    }

    pub fn demote_on_failure(&self) -> Self {
        match self {
            Self::Verified => Self::Trusted,
            Self::Trusted => Self::Known,
            Self::Known => Self::Unknown,
            Self::Unknown => Self::Blocked,
            Self::Blocked => Self::Compromised,
            Self::Compromised => Self::Compromised,
        }
    }

    pub fn promote_on_success(&self) -> Self {
        match self {
            Self::Verified => Self::Verified,
            Self::Trusted => Self::Verified,
            Self::Known => Self::Trusted,
            Self::Unknown => Self::Known,
            Self::Blocked => Self::Unknown,
            Self::Compromised => Self::Blocked,
        }
    }

    pub fn can_escalate_to(&self, target: TrustLevel) -> bool {
        self.promote_on_success() == target || *self == target
    }

    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Verified | Self::Trusted | Self::Known)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked | Self::Compromised)
    }
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified => write!(f, "verified"),
            Self::Trusted => write!(f, "trusted"),
            Self::Known => write!(f, "known"),
            Self::Unknown => write!(f, "unknown"),
            Self::Blocked => write!(f, "blocked"),
            Self::Compromised => write!(f, "compromised"),
        }
    }
}

pub struct TrustManager {
    trust_levels: std::collections::HashMap<String, TrustLevel>,
}

impl TrustManager {
    pub fn new() -> Self {
        Self {
            trust_levels: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, subject: &str) -> TrustLevel {
        self.trust_levels
            .get(subject)
            .copied()
            .unwrap_or(TrustLevel::Unknown)
    }

    pub fn set(&mut self, subject: &str, level: TrustLevel) {
        self.trust_levels.insert(subject.to_string(), level);
    }

    pub fn demote(&mut self, subject: &str) -> TrustLevel {
        let current = self.get(subject);
        let new_level = current.demote_on_failure();
        self.set(subject, new_level);
        new_level
    }

    pub fn promote(&mut self, subject: &str) -> TrustLevel {
        let current = self.get(subject);
        let new_level = current.promote_on_success();
        self.set(subject, new_level);
        new_level
    }

    pub fn check_authorized(
        &self,
        subject: &str,
        required_risk: crate::capability::RiskLevel,
    ) -> bool {
        let level = self.get(subject);
        level.max_allowed_risk() >= required_risk
    }
}

impl Default for TrustManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::RiskLevel;

    #[test]
    fn trust_level_ordering() {
        assert!(TrustLevel::Verified > TrustLevel::Trusted);
        assert!(TrustLevel::Trusted > TrustLevel::Known);
        assert!(TrustLevel::Known > TrustLevel::Unknown);
    }

    #[test]
    fn max_allowed_risk() {
        assert_eq!(TrustLevel::Verified.max_allowed_risk(), RiskLevel::Critical);
        assert_eq!(TrustLevel::Trusted.max_allowed_risk(), RiskLevel::High);
        assert_eq!(TrustLevel::Known.max_allowed_risk(), RiskLevel::Low);
        assert_eq!(TrustLevel::Unknown.max_allowed_risk(), RiskLevel::None);
    }

    #[test]
    fn auto_grants_disabled() {
        // SECURITY: auto_grants is disabled — all actions go through policy evaluation
        assert!(!TrustLevel::Verified.auto_grants(RiskLevel::None));
        assert!(!TrustLevel::Verified.auto_grants(RiskLevel::Low));
        assert!(!TrustLevel::Trusted.auto_grants(RiskLevel::Low));
        assert!(!TrustLevel::Known.auto_grants(RiskLevel::None));
    }

    #[test]
    fn demotion_chain() {
        assert_eq!(
            TrustLevel::Verified.demote_on_failure(),
            TrustLevel::Trusted
        );
        assert_eq!(TrustLevel::Trusted.demote_on_failure(), TrustLevel::Known);
        assert_eq!(
            TrustLevel::Blocked.demote_on_failure(),
            TrustLevel::Compromised
        );
        assert_eq!(
            TrustLevel::Compromised.demote_on_failure(),
            TrustLevel::Compromised
        );
    }

    #[test]
    fn promotion_chain() {
        assert_eq!(TrustLevel::Unknown.promote_on_success(), TrustLevel::Known);
        assert_eq!(TrustLevel::Known.promote_on_success(), TrustLevel::Trusted);
        assert_eq!(
            TrustLevel::Trusted.promote_on_success(),
            TrustLevel::Verified
        );
        assert_eq!(
            TrustLevel::Verified.promote_on_success(),
            TrustLevel::Verified
        );
    }

    #[test]
    fn trust_manager_operations() {
        let mut manager = TrustManager::new();
        assert_eq!(manager.get("unknown-entity"), TrustLevel::Unknown);
        manager.set("device-1", TrustLevel::Verified);
        assert_eq!(manager.get("device-1"), TrustLevel::Verified);
        manager.demote("device-1");
        assert_eq!(manager.get("device-1"), TrustLevel::Trusted);
    }

    #[test]
    fn trust_level_display() {
        assert_eq!(TrustLevel::Verified.to_string(), "verified");
        assert_eq!(TrustLevel::Compromised.to_string(), "compromised");
    }

    #[test]
    fn is_operational_checks() {
        assert!(TrustLevel::Verified.is_operational());
        assert!(TrustLevel::Known.is_operational());
        assert!(!TrustLevel::Blocked.is_operational());
        assert!(!TrustLevel::Compromised.is_operational());
    }

    #[test]
    fn check_authorized_gating() {
        let mut manager = TrustManager::new();
        manager.set("device-1", TrustLevel::Verified);
        assert!(manager.check_authorized("device-1", RiskLevel::Critical));
        assert!(manager.check_authorized("device-1", RiskLevel::High));
        manager.set("device-2", TrustLevel::Known);
        assert!(manager.check_authorized("device-2", RiskLevel::Low));
        assert!(!manager.check_authorized("device-2", RiskLevel::Medium));
    }
}
