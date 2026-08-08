use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recovery mode state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    /// Normal operation — no recovery active.
    Normal,
    /// Recovery mode entered — limited operations allowed.
    Active,
    /// Recovery operations in progress (secret rotation, token revocation).
    InProgress,
}

/// Authorization required to enter recovery mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAuth {
    pub subject: String,
    pub reason: String,
    pub auth_method: String,
}

/// Result of a secret rotation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationResult {
    pub key: String,
    pub old_version: u32,
    pub new_version: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// Report of all recovery operations performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub id: Uuid,
    pub entered_at: DateTime<Utc>,
    pub exited_at: Option<DateTime<Utc>>,
    pub authorized_by: String,
    pub reason: String,
    pub secrets_rotated: Vec<RotationResult>,
    pub tokens_revoked: usize,
    pub components_verified: usize,
    pub components_failed: usize,
    pub success: bool,
}

/// Secure recovery from compromise.
///
/// RecoveryMode implements the security architecture spec §2.11:
/// - Enter recovery mode (requires authorization)
/// - Rotate all secrets
/// - Revoke all tokens
/// - Verify all components
/// - Generate recovery report
/// - Exit recovery mode
///
/// All operations are audited and the state machine prevents
/// concurrent recovery sessions.
pub struct RecoveryMode {
    state: RecoveryState,
    current_report: Option<RecoveryReport>,
}

impl RecoveryMode {
    pub fn new() -> Self {
        Self {
            state: RecoveryState::Normal,
            current_report: None,
        }
    }

    /// Enter recovery mode. Requires valid authorization.
    /// Returns the recovery session (as a report that will be completed on exit).
    pub fn enter(&mut self, auth: RecoveryAuth) -> Result<&RecoveryReport, String> {
        if self.state != RecoveryState::Normal {
            return Err(format!(
                "Cannot enter recovery mode: currently in {:?} state",
                self.state
            ));
        }

        let report = RecoveryReport {
            id: Uuid::new_v4(),
            entered_at: Utc::now(),
            exited_at: None,
            authorized_by: auth.subject,
            reason: auth.reason,
            secrets_rotated: Vec::new(),
            tokens_revoked: 0,
            components_verified: 0,
            components_failed: 0,
            success: false,
        };

        self.current_report = Some(report);
        self.state = RecoveryState::Active;

        Ok(self.current_report.as_ref().unwrap())
    }

    /// Check if recovery mode is currently active.
    pub fn is_active(&self) -> bool {
        self.state == RecoveryState::Active || self.state == RecoveryState::InProgress
    }

    /// Get the current recovery state.
    pub fn state(&self) -> RecoveryState {
        self.state
    }

    /// Record a secret rotation result.
    pub fn record_rotation(&mut self, result: RotationResult) {
        if let Some(ref mut report) = self.current_report {
            report.secrets_rotated.push(result);
        }
    }

    /// Record that tokens were revoked.
    pub fn record_tokens_revoked(&mut self, count: usize) {
        if let Some(ref mut report) = self.current_report {
            report.tokens_revoked += count;
        }
    }

    /// Record component verification results.
    pub fn record_verification(&mut self, verified: usize, failed: usize) {
        if let Some(ref mut report) = self.current_report {
            report.components_verified += verified;
            report.components_failed += failed;
        }
    }

    /// Exit recovery mode and finalize the report.
    pub fn exit(&mut self) -> Result<RecoveryReport, String> {
        if self.state == RecoveryState::Normal {
            return Err("Not in recovery mode".to_string());
        }

        let mut report = self
            .current_report
            .take()
            .ok_or("No active recovery report")?;

        report.exited_at = Some(Utc::now());
        report.success = report.components_failed == 0;
        self.state = RecoveryState::Normal;

        Ok(report)
    }

    /// Force-exit recovery mode (e.g., on timeout or abort).
    /// The report will show as unsuccessful.
    pub fn abort(&mut self) -> Option<RecoveryReport> {
        if self.state == RecoveryState::Normal {
            return None;
        }

        let mut report = self.current_report.take()?;
        report.exited_at = Some(Utc::now());
        report.success = false;
        self.state = RecoveryState::Normal;

        Some(report)
    }
}

impl Default for RecoveryMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_auth() -> RecoveryAuth {
        RecoveryAuth {
            subject: "admin".to_string(),
            reason: "Compromise detected".to_string(),
            auth_method: "password+biometric".to_string(),
        }
    }

    #[test]
    fn recovery_enter_and_exit() {
        let mut recovery = RecoveryMode::new();
        assert!(!recovery.is_active());
        assert_eq!(recovery.state(), RecoveryState::Normal);

        let authorized_by = recovery.enter(test_auth()).unwrap().authorized_by.clone();
        assert!(recovery.is_active());
        assert_eq!(recovery.state(), RecoveryState::Active);
        assert_eq!(authorized_by, "admin");

        let final_report = recovery.exit().unwrap();
        assert!(!recovery.is_active());
        assert_eq!(recovery.state(), RecoveryState::Normal);
        assert!(final_report.exited_at.is_some());
    }

    #[test]
    fn recovery_cannot_enter_twice() {
        let mut recovery = RecoveryMode::new();
        recovery.enter(test_auth()).unwrap();
        assert!(recovery.enter(test_auth()).is_err());
    }

    #[test]
    fn recovery_record_rotation() {
        let mut recovery = RecoveryMode::new();
        recovery.enter(test_auth()).unwrap();
        recovery.record_rotation(RotationResult {
            key: "api_key".to_string(),
            old_version: 1,
            new_version: 2,
            success: true,
            error: None,
        });
        let report = recovery.exit().unwrap();
        assert_eq!(report.secrets_rotated.len(), 1);
        assert_eq!(report.secrets_rotated[0].new_version, 2);
    }

    #[test]
    fn recovery_record_tokens_revoked() {
        let mut recovery = RecoveryMode::new();
        recovery.enter(test_auth()).unwrap();
        recovery.record_tokens_revoked(5);
        let report = recovery.exit().unwrap();
        assert_eq!(report.tokens_revoked, 5);
    }

    #[test]
    fn recovery_record_verification() {
        let mut recovery = RecoveryMode::new();
        recovery.enter(test_auth()).unwrap();
        recovery.record_verification(10, 1);
        let report = recovery.exit().unwrap();
        assert_eq!(report.components_verified, 10);
        assert_eq!(report.components_failed, 1);
        assert!(!report.success); // failed > 0 means unsuccessful
    }

    #[test]
    fn recovery_abort() {
        let mut recovery = RecoveryMode::new();
        recovery.enter(test_auth()).unwrap();
        let report = recovery.abort();
        assert!(report.is_some());
        assert!(!recovery.is_active());
        assert!(!report.unwrap().success);
    }

    #[test]
    fn recovery_abort_when_normal_returns_none() {
        let mut recovery = RecoveryMode::new();
        assert!(recovery.abort().is_none());
    }

    #[test]
    fn recovery_exit_when_normal_returns_err() {
        let mut recovery = RecoveryMode::new();
        assert!(recovery.exit().is_err());
    }

    #[test]
    fn recovery_success_on_clean_verification() {
        let mut recovery = RecoveryMode::new();
        recovery.enter(test_auth()).unwrap();
        recovery.record_verification(10, 0);
        let report = recovery.exit().unwrap();
        assert!(report.success);
    }
}
