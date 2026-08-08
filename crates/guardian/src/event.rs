use std::fmt;

#[derive(Debug, Clone)]
pub enum GuardianEvent {
    AccessRequested {
        request_id: String,
        subject: String,
        action: String,
        resource: String,
    },
    AccessGranted {
        request_id: String,
        subject: String,
        action: String,
        resource: String,
    },
    AccessDenied {
        request_id: String,
        subject: String,
        action: String,
        resource: String,
        reason: String,
    },
    PolicyEvaluated {
        policy_id: String,
        result: String,
    },
    ConsentRequested {
        request_id: String,
        subject: String,
        reason: String,
    },
    ConsentGranted {
        request_id: String,
        subject: String,
    },
    ConsentDenied {
        request_id: String,
        subject: String,
        reason: String,
    },
    AuditLogged {
        entry_id: String,
        action: String,
        subject: String,
    },
    MfaRequired {
        request_id: String,
        subject: String,
    },
    MfaCompleted {
        request_id: String,
        subject: String,
        success: bool,
    },
}

impl fmt::Display for GuardianEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessRequested {
                request_id,
                subject,
                action,
                resource,
            } => {
                write!(
                    f,
                    "Access requested [{}]: {} wants to {} on {}",
                    request_id, subject, action, resource
                )
            }
            Self::AccessGranted {
                request_id,
                subject,
                action,
                resource,
            } => {
                write!(
                    f,
                    "Access granted [{}]: {} allowed to {} on {}",
                    request_id, subject, action, resource
                )
            }
            Self::AccessDenied {
                request_id,
                subject,
                action,
                resource,
                reason,
            } => {
                write!(
                    f,
                    "Access denied [{}]: {} cannot {} on {}: {}",
                    request_id, subject, action, resource, reason
                )
            }
            Self::PolicyEvaluated { policy_id, result } => {
                write!(f, "Policy evaluated [{}]: {}", policy_id, result)
            }
            Self::ConsentRequested {
                request_id,
                subject,
                reason,
            } => {
                write!(
                    f,
                    "Consent requested [{}] from {}: {}",
                    request_id, subject, reason
                )
            }
            Self::ConsentGranted {
                request_id,
                subject,
            } => {
                write!(f, "Consent granted [{}] by {}", request_id, subject)
            }
            Self::ConsentDenied {
                request_id,
                subject,
                reason,
            } => {
                write!(
                    f,
                    "Consent denied [{}] by {}: {}",
                    request_id, subject, reason
                )
            }
            Self::AuditLogged {
                entry_id,
                action,
                subject,
            } => {
                write!(f, "Audit logged [{}]: {} by {}", entry_id, action, subject)
            }
            Self::MfaRequired {
                request_id,
                subject,
            } => {
                write!(f, "MFA required [{}] for {}", request_id, subject)
            }
            Self::MfaCompleted {
                request_id,
                subject,
                success,
            } => {
                write!(
                    f,
                    "MFA completed [{}] for {}: {}",
                    request_id,
                    subject,
                    if *success { "success" } else { "failure" }
                )
            }
        }
    }
}
