use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::context::AssembledContext;
use crate::error::Result;
use crate::planner::{Plan, PlanStep};
use crate::types::ActionId;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Approved,
    Rejected(String),
    RequiresReview(String),
    RequiresConsent,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub action_id: ActionId,
    pub status: ValidationStatus,
    pub validated_by: Vec<String>,
    pub risk_level: String,
    pub conditions: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
pub trait ActionValidator: Send + Sync {
    async fn validate_action(
        &self,
        action: &PlanStep,
        context: &AssembledContext,
    ) -> Result<ValidationResult>;
    async fn validate_plan(
        &self,
        plan: &Plan,
        context: &AssembledContext,
    ) -> Result<Vec<ValidationResult>>;
    async fn check_safety(
        &self,
        action: &PlanStep,
        context: &AssembledContext,
    ) -> Result<ValidationStatus>;
    async fn check_permissions(
        &self,
        action: &PlanStep,
        context: &AssembledContext,
    ) -> Result<ValidationStatus>;
}
