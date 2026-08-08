use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Risk score thresholds for different defense actions.
const LOW_RISK_THRESHOLD: f64 = 0.3;
const MEDIUM_RISK_THRESHOLD: f64 = 0.6;
const HIGH_RISK_THRESHOLD: f64 = 0.85;

/// Action taken against a subject based on risk score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefenseAction {
    None,
    RateLimited,
    Warned,
    TempBan,
    PermBan,
}

/// Risk scoring entry for a subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub subject: String,
    pub score: f64,
    pub total_violations: u64,
    pub recent_violations: Vec<ViolationRecord>,
    pub current_action: DefenseAction,
    pub banned_until: Option<DateTime<Utc>>,
    pub permanent_ban: bool,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// A single violation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub violation_type: String,
    pub severity_weight: f64,
    pub description: String,
}

/// Auto-defense engine with progressive risk scoring and bans.
pub struct AutoDefense {
    profiles: HashMap<String, RiskProfile>,
    /// Maximum recent violations kept per subject.
    max_recent_violations: usize,
    /// Window in seconds for recent violation decay.
    decay_window_secs: i64,
    /// Score multiplier per additional violation (escalation).
    escalation_factor: f64,
}

impl AutoDefense {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            max_recent_violations: 50,
            decay_window_secs: 3600, // 1 hour
            escalation_factor: 1.5,
        }
    }

    pub fn with_decay_window(mut self, secs: i64) -> Self {
        self.decay_window_secs = secs;
        self
    }

    /// Record a violation and compute the new risk score.
    pub fn record_violation(
        &mut self,
        subject: &str,
        violation_type: &str,
        severity_weight: f64,
        description: &str,
    ) -> DefenseAction {
        let now = Utc::now();

        let profile = self
            .profiles
            .entry(subject.to_string())
            .or_insert_with(|| RiskProfile {
                subject: subject.to_string(),
                score: 0.0,
                total_violations: 0,
                recent_violations: Vec::new(),
                current_action: DefenseAction::None,
                banned_until: None,
                permanent_ban: false,
                created_at: now,
                last_updated: now,
            });

        // Record violation
        let violation = ViolationRecord {
            id: Uuid::new_v4(),
            timestamp: now,
            violation_type: violation_type.to_string(),
            severity_weight,
            description: description.to_string(),
        };
        profile.recent_violations.push(violation);
        profile.total_violations += 1;

        // Evict old violations
        let cutoff = now - chrono::Duration::seconds(self.decay_window_secs);
        profile.recent_violations.retain(|v| v.timestamp >= cutoff);
        if profile.recent_violations.len() > self.max_recent_violations {
            let excess = profile.recent_violations.len() - self.max_recent_violations;
            profile.recent_violations.drain(..excess);
        }

        // Compute risk score: base severity * escalation based on violation count
        let recent_count = profile.recent_violations.len() as f64;
        let escalation = self.escalation_factor.powf(recent_count - 1.0);
        let raw_score = severity_weight * escalation;
        // Normalize to [0, 1] range using sigmoid-like clamp
        profile.score = (raw_score / (1.0 + raw_score)).min(1.0);
        profile.last_updated = now;

        // Determine defense action
        let action = if profile.permanent_ban {
            DefenseAction::PermBan
        } else if let Some(banned_until) = profile.banned_until {
            if now < banned_until {
                DefenseAction::TempBan
            } else {
                profile.banned_until = None;
                Self::score_to_action(profile.score)
            }
        } else {
            Self::score_to_action(profile.score)
        };

        profile.current_action = action.clone();
        action
    }

    fn score_to_action(score: f64) -> DefenseAction {
        if score >= HIGH_RISK_THRESHOLD {
            DefenseAction::PermBan
        } else if score >= MEDIUM_RISK_THRESHOLD {
            DefenseAction::TempBan
        } else if score >= LOW_RISK_THRESHOLD {
            DefenseAction::RateLimited
        } else {
            DefenseAction::None
        }
    }

    /// Manually apply a temporary ban.
    pub fn apply_temp_ban(&mut self, subject: &str, duration_secs: i64) {
        let profile = self
            .profiles
            .entry(subject.to_string())
            .or_insert_with(|| RiskProfile {
                subject: subject.to_string(),
                score: 0.0,
                total_violations: 0,
                recent_violations: Vec::new(),
                current_action: DefenseAction::None,
                banned_until: None,
                permanent_ban: false,
                created_at: Utc::now(),
                last_updated: Utc::now(),
            });
        profile.banned_until = Some(Utc::now() + chrono::Duration::seconds(duration_secs));
        profile.current_action = DefenseAction::TempBan;
    }

    /// Manually apply a permanent ban.
    pub fn apply_perm_ban(&mut self, subject: &str) {
        let profile = self
            .profiles
            .entry(subject.to_string())
            .or_insert_with(|| RiskProfile {
                subject: subject.to_string(),
                score: 0.0,
                total_violations: 0,
                recent_violations: Vec::new(),
                current_action: DefenseAction::None,
                banned_until: None,
                permanent_ban: false,
                created_at: Utc::now(),
                last_updated: Utc::now(),
            });
        profile.permanent_ban = true;
        profile.current_action = DefenseAction::PermBan;
    }

    /// Unban a subject (clears both temp and permanent ban, resets score).
    pub fn unban(&mut self, subject: &str) -> bool {
        if let Some(profile) = self.profiles.get_mut(subject) {
            profile.permanent_ban = false;
            profile.banned_until = None;
            profile.score = 0.0;
            profile.recent_violations.clear();
            profile.current_action = DefenseAction::None;
            true
        } else {
            false
        }
    }

    /// Check if a subject is currently banned.
    pub fn is_banned(&self, subject: &str) -> bool {
        self.profiles
            .get(subject)
            .map(|p| {
                if p.permanent_ban {
                    return true;
                }
                if let Some(banned_until) = p.banned_until {
                    if Utc::now() < banned_until {
                        return true;
                    }
                }
                // Also check current_action for score-based bans
                matches!(
                    p.current_action,
                    DefenseAction::PermBan | DefenseAction::TempBan
                )
            })
            .unwrap_or(false)
    }

    /// Get the current defense action for a subject.
    pub fn get_action(&self, subject: &str) -> DefenseAction {
        self.profiles
            .get(subject)
            .map(|p| p.current_action.clone())
            .unwrap_or(DefenseAction::None)
    }

    /// Get the risk profile for a subject.
    pub fn get_profile(&self, subject: &str) -> Option<&RiskProfile> {
        self.profiles.get(subject)
    }

    /// Get all profiles above a given risk threshold.
    pub fn profiles_above_threshold(&self, threshold: f64) -> Vec<&RiskProfile> {
        self.profiles
            .values()
            .filter(|p| p.score >= threshold)
            .collect()
    }

    /// Decay all scores over time (call periodically).
    pub fn decay_all_scores(&mut self, decay_factor: f64) {
        for profile in self.profiles.values_mut() {
            profile.score *= decay_factor;
            profile.last_updated = Utc::now();
        }
    }

    pub fn subject_count(&self) -> usize {
        self.profiles.len()
    }
}

impl Default for AutoDefense {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_violation_no_action() {
        let mut defense = AutoDefense::new();
        let action = defense.record_violation("user-1", "minor", 0.1, "Minor policy deviation");
        assert_eq!(action, DefenseAction::None);
    }

    #[test]
    fn escalating_violations_trigger_ban() {
        let mut defense = AutoDefense::new().with_decay_window(3600);
        // Record multiple high-severity violations rapidly
        for i in 0..10 {
            defense.record_violation("attacker", "brute_force", 0.8, &format!("Violation {i}"));
        }
        // Score should be high due to escalation
        let profile = defense.get_profile("attacker").unwrap();
        assert!(profile.score > MEDIUM_RISK_THRESHOLD);
        assert!(defense.is_banned("attacker"));
    }

    #[test]
    fn temp_ban_with_expiry() {
        let mut defense = AutoDefense::new();
        defense.apply_temp_ban("user-2", 1); // 1 second
        assert!(defense.is_banned("user-2"));
        defense.unban("user-2");
        assert!(!defense.is_banned("user-2"));
    }

    #[test]
    fn perm_ban() {
        let mut defense = AutoDefense::new();
        defense.apply_perm_ban("evil-user");
        assert!(defense.is_banned("evil-user"));
        assert!(defense.unban("evil-user"));
        assert!(!defense.is_banned("evil-user"));
        // Re-ban and verify
        defense.apply_perm_ban("evil-user");
        assert!(defense.is_banned("evil-user"));
        assert!(!defense.unban("unknown-user"));
    }

    #[test]
    fn unbanned_subject_gets_action_reset() {
        let mut defense = AutoDefense::new();
        for _ in 0..5 {
            defense.record_violation("user", "injection", 0.7, "attempt");
        }
        assert!(defense.is_banned("user"));
        defense.unban("user");
        assert_eq!(defense.get_action("user"), DefenseAction::None);
    }

    #[test]
    fn profiles_above_threshold() {
        let mut defense = AutoDefense::new();
        defense.record_violation("low-user", "minor", 0.1, "minor");
        defense.record_violation("high-user", "attack", 0.9, "major");
        for _ in 0..5 {
            defense.record_violation("high-user", "attack", 0.9, "major");
        }
        let high_risk = defense.profiles_above_threshold(MEDIUM_RISK_THRESHOLD);
        assert!(high_risk.iter().any(|p| p.subject == "high-user"));
        assert!(!high_risk.iter().any(|p| p.subject == "low-user"));
    }

    #[test]
    fn decay_reduces_scores() {
        let mut defense = AutoDefense::new();
        defense.record_violation("user", "test", 0.5, "test");
        let original_score = defense.get_profile("user").unwrap().score;
        defense.decay_all_scores(0.5);
        let decayed_score = defense.get_profile("user").unwrap().score;
        assert!(decayed_score < original_score);
    }

    #[test]
    fn nonexistent_subject_defaults() {
        let defense = AutoDefense::new();
        assert!(!defense.is_banned("unknown"));
        assert_eq!(defense.get_action("unknown"), DefenseAction::None);
    }
}
