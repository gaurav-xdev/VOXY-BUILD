#[derive(Debug, thiserror::Error)]
pub enum GuardianError {
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    #[error("Authentication required")]
    AuthenticationRequired,
    #[error("Consent required: {0}")]
    ConsentRequired(String),
    #[error("MFA required")]
    MfaRequired,
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Guardian not initialized")]
    NotInitialized,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Audit error: {0}")]
    AuditError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, GuardianError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let err = GuardianError::AccessDenied("no permission".into());
        assert_eq!(err.to_string(), "Access denied: no permission");

        let err = GuardianError::AuthenticationRequired;
        assert_eq!(err.to_string(), "Authentication required");
    }

    #[test]
    fn test_error_trait() {
        let err = GuardianError::Internal("oops".into());
        let err_ref: &dyn std::error::Error = &err;
        assert_eq!(err_ref.to_string(), "Internal error: oops");
    }

    #[test]
    fn test_all_variants() {
        let variants: Vec<GuardianError> = vec![
            GuardianError::AccessDenied("a".into()),
            GuardianError::PolicyViolation("b".into()),
            GuardianError::AuthenticationRequired,
            GuardianError::ConsentRequired("c".into()),
            GuardianError::MfaRequired,
            GuardianError::ResourceNotFound("d".into()),
            GuardianError::NotInitialized,
            GuardianError::InvalidRequest("e".into()),
            GuardianError::AuditError("f".into()),
            GuardianError::Internal("g".into()),
        ];
        for v in variants {
            let _ = v.to_string();
        }
    }
}
