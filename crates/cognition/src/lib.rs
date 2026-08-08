pub mod attention;
pub mod confidence;
pub mod config;
pub mod context;
pub mod diagnostics;
pub mod error;
pub mod event;
pub mod goals;
pub mod in_memory;
pub mod intent;
pub mod orchestration;
pub mod planner;
pub mod reasoning;
pub mod recovery;
pub mod reflection;
pub mod self_monitoring;
pub mod tools;
pub mod types;
pub mod validation;

pub use attention::{AttentionFactors, AttentionRecommendation, AttentionScore, AttentionSystem};
pub use confidence::ConfidenceEstimator;
pub use config::CognitionConfig;
pub use context::{AssembledContext, ContextAssembler, ContextAssemblyInput, ContextSource};
pub use diagnostics::{ContextSummary, Diagnostics, DiagnosticsSnapshot, LatencyDiagnostics};
pub use error::{CognitionError, Result};
pub use event::CognitionEvent;
pub use goals::{
    Goal, GoalDecomposer, GoalDecomposition, GoalPriority, GoalState, GoalTrigger,
    PrioritizationResult,
};
pub use intent::{IntentAnalysis, IntentAnalyzer, IntentInput, IntentType};
pub use orchestration::{CognitiveEngine, CognitiveResult, DecisionOutput};
pub use planner::{
    Plan, PlanAction, PlanState, PlanStep, Planner, PlanningContext, StepState, StepType,
};
pub use reasoning::{Reasoner, ReasoningInput, ReasoningOutput, ReasoningStep, ReasoningStrategy};
pub use recovery::{RecoveryManager, RecoveryPlan, RecoveryStrategy};
pub use reflection::{
    ContextChange, InsightSource, OutcomeComparison, ReflectionEngine, ReflectionInput,
    ReflectionInsight, ReflectionReport,
};
pub use self_monitoring::{HealthStatus, LatencyRecord, SelfMonitor, SystemSnapshot};
pub use tools::{ToolDescription, ToolSelection, ToolSelectionInput, ToolSelector};
pub use types::{
    ActionId, CognitiveState, ConfidenceLevel, ConfidenceScore, ContextId, GoalId, IntentId,
    PlanId, ReasoningId, ReflectionId, StepId, Urgency,
};
pub use validation::{ActionValidator, ValidationResult, ValidationStatus};
pub use voxy_skills::capabilities::CapabilityId;
pub use voxy_skills::types::SkillId;

// In-memory concrete implementations
pub use in_memory::{
    InMemoryActionValidator, InMemoryAttentionSystem, InMemoryCognitiveEngine,
    InMemoryConfidenceEstimator, InMemoryContextAssembler, InMemoryGoalDecomposer,
    InMemoryIntentAnalyzer, InMemoryPlanner, InMemoryReasoner, InMemoryRecoveryManager,
    InMemoryReflectionEngine, InMemorySelfMonitor, InMemoryToolSelector,
};

pub mod prelude {
    pub use crate::attention::{
        AttentionFactors, AttentionRecommendation, AttentionScore, AttentionSystem,
    };
    pub use crate::confidence::ConfidenceEstimator;
    pub use crate::config::CognitionConfig;
    pub use crate::context::{AssembledContext, ContextAssembler, ContextSource};
    pub use crate::diagnostics::{Diagnostics, DiagnosticsSnapshot};
    pub use crate::error::{CognitionError, Result};
    pub use crate::event::CognitionEvent;
    pub use crate::goals::{Goal, GoalDecomposer, GoalDecomposition, GoalPriority, GoalState};
    pub use crate::intent::{IntentAnalysis, IntentAnalyzer, IntentInput, IntentType};
    pub use crate::orchestration::{CognitiveEngine, CognitiveResult, DecisionOutput};
    pub use crate::planner::{
        Plan, PlanAction, PlanState, PlanStep, Planner, PlanningContext, StepState, StepType,
    };
    pub use crate::reasoning::{
        Reasoner, ReasoningInput, ReasoningOutput, ReasoningStep, ReasoningStrategy,
    };
    pub use crate::recovery::{RecoveryManager, RecoveryPlan, RecoveryStrategy};
    pub use crate::reflection::{
        ContextChange, InsightSource, ReflectionEngine, ReflectionInput, ReflectionInsight,
        ReflectionReport,
    };
    pub use crate::self_monitoring::{HealthStatus, SelfMonitor, SystemSnapshot};
    pub use crate::tools::{ToolDescription, ToolSelection, ToolSelector};
    pub use crate::types::{
        ActionId, CognitiveState, ConfidenceLevel, ConfidenceScore, ContextId, GoalId, IntentId,
        PlanId, ReasoningId, ReflectionId, StepId, Urgency,
    };
    pub use crate::validation::{ActionValidator, ValidationResult, ValidationStatus};
    pub use voxy_skills::capabilities::CapabilityId;
    pub use voxy_skills::types::SkillId;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use super::*;

    #[test]
    fn test_cognitive_state_display() {
        assert_eq!(format!("{}", CognitiveState::Idle), "Idle");
        assert_eq!(
            format!("{}", CognitiveState::AnalyzingIntent),
            "AnalyzingIntent"
        );
        assert_eq!(
            format!("{}", CognitiveState::DecomposingGoals),
            "DecomposingGoals"
        );
        assert_eq!(format!("{}", CognitiveState::Planning), "Planning");
        assert_eq!(format!("{}", CognitiveState::Reasoning), "Reasoning");
        assert_eq!(
            format!("{}", CognitiveState::AssemblingContext),
            "AssemblingContext"
        );
        assert_eq!(
            format!("{}", CognitiveState::SelectingTools),
            "SelectingTools"
        );
        assert_eq!(format!("{}", CognitiveState::Executing), "Executing");
        assert_eq!(format!("{}", CognitiveState::Validating), "Validating");
        assert_eq!(format!("{}", CognitiveState::Recovering), "Recovering");
        assert_eq!(format!("{}", CognitiveState::Reflecting), "Reflecting");
        assert_eq!(format!("{}", CognitiveState::Completed), "Completed");
        assert_eq!(
            format!("{}", CognitiveState::Failed("err".to_string())),
            "Failed(err)"
        );
    }

    #[test]
    fn test_confidence_level_value() {
        assert!((ConfidenceLevel::VeryLow.value() - 0.1).abs() < f64::EPSILON);
        assert!((ConfidenceLevel::Low.value() - 0.3).abs() < f64::EPSILON);
        assert!((ConfidenceLevel::Medium.value() - 0.5).abs() < f64::EPSILON);
        assert!((ConfidenceLevel::High.value() - 0.7).abs() < f64::EPSILON);
        assert!((ConfidenceLevel::VeryHigh.value() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_confidence_score_valid() {
        let score = ConfidenceScore::new(0.75).unwrap();
        assert!((score.value - 0.75).abs() < f64::EPSILON);
        assert_eq!(score.level, ConfidenceLevel::High);
        assert!(score.explanations.is_empty());
    }

    #[test]
    fn test_confidence_score_clamp() {
        let score = ConfidenceScore::new(1.5).unwrap();
        assert!((score.value - 1.0).abs() < f64::EPSILON);

        let score = ConfidenceScore::new(-0.5).unwrap();
        assert!((score.value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_confidence_score_is_sufficient() {
        let score = ConfidenceScore::new(0.7).unwrap();
        assert!(score.is_sufficient(0.5));
        assert!(!score.is_sufficient(0.8));
    }

    #[test]
    fn test_intent_id_newtype() {
        let id = IntentId("test-intent".to_string());
        assert_eq!(id.to_string(), "test-intent");
        assert_eq!(id.0, "test-intent");
    }

    #[test]
    fn test_plan_id_newtype() {
        let id = PlanId("test-plan".to_string());
        assert_eq!(id.to_string(), "test-plan");
        assert_eq!(id.0, "test-plan");
    }

    #[test]
    fn test_step_id_newtype() {
        let id = StepId("step-1".to_string());
        assert_eq!(id.to_string(), "step-1");
        assert_eq!(id.0, "step-1");
    }

    #[test]
    fn test_intent_type_display() {
        assert_eq!(format!("{}", IntentType::Query), "Query");
        assert_eq!(format!("{}", IntentType::Command), "Command");
        assert_eq!(
            format!("{}", IntentType::Custom("test".to_string())),
            "Custom(test)"
        );
    }

    #[test]
    fn test_urgency_ordering() {
        assert!(Urgency::Low < Urgency::Medium);
        assert!(Urgency::Medium < Urgency::High);
        assert!(Urgency::High < Urgency::Critical);
        assert!(Urgency::Low < Urgency::Critical);
    }

    #[test]
    fn test_goal_id_newtype() {
        let id = GoalId("goal-1".to_string());
        assert_eq!(id.to_string(), "goal-1");
        assert_eq!(id.0, "goal-1");
    }

    #[test]
    fn test_goal_priority_ordering() {
        assert!(GoalPriority::Low < GoalPriority::Medium);
        assert!(GoalPriority::Medium < GoalPriority::High);
        assert!(GoalPriority::High < GoalPriority::Critical);
    }

    #[test]
    fn test_goal_state_display() {
        assert_eq!(format!("{}", GoalState::Pending), "Pending");
        assert_eq!(format!("{}", GoalState::Active), "Active");
        assert_eq!(format!("{}", GoalState::InProgress), "InProgress");
        assert_eq!(
            format!("{}", GoalState::Blocked("reason".to_string())),
            "Blocked(reason)"
        );
        assert_eq!(format!("{}", GoalState::Completed), "Completed");
        assert_eq!(
            format!("{}", GoalState::Failed("err".to_string())),
            "Failed(err)"
        );
        assert_eq!(format!("{}", GoalState::Cancelled), "Cancelled");
    }

    #[test]
    fn test_goal_creation() {
        let now = Utc::now();
        let goal = Goal {
            id: GoalId("g1".to_string()),
            description: "Test goal".to_string(),
            priority: GoalPriority::High,
            dependencies: vec![],
            state: GoalState::Pending,
            acceptance_criteria: vec!["criterion 1".to_string()],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            context_snapshot: None,
            paused_by_context: false,
        };
        assert_eq!(goal.id.0, "g1");
        assert_eq!(goal.description, "Test goal");
        assert_eq!(goal.priority, GoalPriority::High);
        assert_eq!(goal.state, GoalState::Pending);
    }

    #[test]
    fn test_plan_state_variants() {
        let states = [
            PlanState::Draft,
            PlanState::Validated,
            PlanState::InProgress,
            PlanState::Completed { success: true },
            PlanState::Completed { success: false },
            PlanState::Failed("error".to_string()),
            PlanState::Cancelled,
            PlanState::RolledBack,
        ];
        assert_eq!(states.len(), 8);
    }

    #[test]
    fn test_step_type_variants() {
        let atomic = StepType::Atomic;
        let composite = StepType::Composite(vec![StepId("s1".to_string())]);
        let conditional = StepType::Conditional {
            condition: "x > 5".to_string(),
            then_branch: vec![],
            else_branch: vec![],
        };
        let parallel = StepType::Parallel(vec![StepId("s2".to_string())]);
        let loop_ = StepType::Loop { max_iterations: 3 };
        match atomic {
            StepType::Atomic => {}
            _ => panic!("expected Atomic"),
        }
        match composite {
            StepType::Composite(_) => {}
            _ => panic!("expected Composite"),
        }
        match conditional {
            StepType::Conditional { .. } => {}
            _ => panic!("expected Conditional"),
        }
        match parallel {
            StepType::Parallel(_) => {}
            _ => panic!("expected Parallel"),
        }
        match loop_ {
            StepType::Loop { .. } => {}
            _ => panic!("expected Loop"),
        }
    }

    #[test]
    fn test_assembled_context_creation() {
        let ctx = AssembledContext {
            id: ContextId("ctx-1".to_string()),
            sources: vec![ContextSource::WorldModel, ContextSource::Personality],
            world_snapshot: None,
            personality_context: None,
            relevant_history: vec!["history item".to_string()],
            constraints: vec![],
            priority_hints: vec!["urgent".to_string()],
            assembly_time_ms: 42,
            timestamp: Utc::now(),
            fusion_data: None,
            fusion_confidence: 0.8,
            source_count: 2,
        };
        assert_eq!(ctx.id.0, "ctx-1");
        assert_eq!(ctx.sources.len(), 2);
        assert_eq!(ctx.assembly_time_ms, 42);
    }

    #[test]
    fn test_context_source_variants() {
        let sources = [
            ContextSource::WorldModel,
            ContextSource::Personality,
            ContextSource::UserHistory,
            ContextSource::SystemState,
            ContextSource::ExternalService("api".to_string()),
        ];
        assert_eq!(sources.len(), 5);
    }

    #[test]
    fn test_tool_description_creation() {
        let tool = ToolDescription {
            tool_id: "tool-1".to_string(),
            name: "Calculator".to_string(),
            description: "Performs arithmetic".to_string(),
            capabilities: vec![CapabilityId("math".to_string())],
            confidence_score: Some(0.95),
        };
        assert_eq!(tool.tool_id, "tool-1");
        assert_eq!(tool.name, "Calculator");
        assert_eq!(tool.capabilities.len(), 1);
    }

    #[test]
    fn test_validation_status_variants() {
        let approved = ValidationStatus::Approved;
        let rejected = ValidationStatus::Rejected("bad".to_string());
        let review = ValidationStatus::RequiresReview("check".to_string());
        let consent = ValidationStatus::RequiresConsent;
        assert_eq!(approved, ValidationStatus::Approved);
        assert_ne!(approved, rejected);
        match consent {
            ValidationStatus::RequiresConsent => {}
            _ => panic!("expected RequiresConsent"),
        }
        if let ValidationStatus::Rejected(msg) = &rejected {
            assert_eq!(msg, "bad");
        } else {
            panic!("expected Rejected")
        }
        if let ValidationStatus::RequiresReview(msg) = &review {
            assert_eq!(msg, "check");
        } else {
            panic!("expected RequiresReview")
        }
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult {
            action_id: ActionId("act-1".to_string()),
            status: ValidationStatus::Approved,
            validated_by: vec!["guardian".to_string()],
            risk_level: "low".to_string(),
            conditions: vec![],
            timestamp: Utc::now(),
        };
        assert_eq!(result.action_id.0, "act-1");
        assert_eq!(result.status, ValidationStatus::Approved);
        assert_eq!(result.risk_level, "low");
    }

    #[test]
    fn test_recovery_strategy_variants() {
        let retry = RecoveryStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 1000,
        };
        let fallback = RecoveryStrategy::FallbackPlan(PlanId("fp-1".to_string()));
        let simplify = RecoveryStrategy::SimplifyGoal;
        let skip = RecoveryStrategy::SkipStep;
        let abort = RecoveryStrategy::AbortWithError("fatal".to_string());
        let assist = RecoveryStrategy::RequestHumanAssistance("help".to_string());
        match retry {
            RecoveryStrategy::Retry {
                max_attempts: 3, ..
            } => {}
            _ => panic!("expected Retry"),
        }
        match fallback {
            RecoveryStrategy::FallbackPlan(_) => {}
            _ => panic!("expected FallbackPlan"),
        }
        match simplify {
            RecoveryStrategy::SimplifyGoal => {}
            _ => panic!("expected SimplifyGoal"),
        }
        match skip {
            RecoveryStrategy::SkipStep => {}
            _ => panic!("expected SkipStep"),
        }
        match abort {
            RecoveryStrategy::AbortWithError(_) => {}
            _ => panic!("expected AbortWithError"),
        }
        match assist {
            RecoveryStrategy::RequestHumanAssistance(_) => {}
            _ => panic!("expected RequestHumanAssistance"),
        }
    }

    #[test]
    fn test_recovery_plan_creation() {
        let plan = RecoveryPlan {
            strategy: RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 500,
            },
            affected_steps: vec![StepId("s1".to_string())],
            rationale: "transient error".to_string(),
            estimated_recovery_time_ms: 1500,
            requires_user_consent: false,
        };
        assert_eq!(plan.affected_steps.len(), 1);
        assert!(!plan.requires_user_consent);
        assert_eq!(plan.rationale, "transient error");
    }

    #[test]
    fn test_reflection_insight_creation() {
        let insight = ReflectionInsight {
            category: "performance".to_string(),
            description: "slow execution".to_string(),
            impact: "high".to_string(),
            suggestion: Some("optimize query".to_string()),
            confidence: 0.85,
            source_type: InsightSource::Execution,
        };
        assert_eq!(insight.category, "performance");
        assert!(insight.suggestion.is_some());
    }

    #[test]
    fn test_reflection_report_creation() {
        let report = ReflectionReport {
            reflection_id: ReflectionId("ref-1".to_string()),
            insights: vec![],
            overall_assessment: "good".to_string(),
            improvement_suggestions: vec!["use caching".to_string()],
            lessons_learned: vec!["test more".to_string()],
            duration_ms: 100,
            timestamp: Utc::now(),
            outcome_comparison: None,
            context_impact: vec![],
        };
        assert_eq!(report.reflection_id.0, "ref-1");
        assert_eq!(report.overall_assessment, "good");
        assert_eq!(report.improvement_suggestions.len(), 1);
    }

    #[test]
    fn test_cognitive_result_creation() {
        let now = Utc::now();
        let intent = IntentAnalysis {
            intent_id: IntentId("int-1".to_string()),
            intent_type: IntentType::Query,
            confidence: ConfidenceScore::new(0.9).unwrap(),
            primary_action: "search".to_string(),
            parameters: HashMap::new(),
            requires_planning: false,
            requires_reasoning: false,
            urgency: Urgency::Medium,
            alternate_interpretations: vec![],
            timestamp: now,
        };
        let result = CognitiveResult {
            intent,
            plan: None,
            context: None,
            result: serde_json::json!({"status": "ok"}),
            confidence: ConfidenceScore::new(0.9).unwrap(),
            reflection: None,
            duration_ms: 42,
            success: true,
            errors: vec![],
            decision: None,
            attention_score: None,
        };
        assert!(result.success);
        assert_eq!(result.duration_ms, 42);
        assert_eq!(result.result["status"], "ok");
    }

    #[test]
    fn test_cognition_config_default() {
        let config = CognitionConfig::default();
        assert_eq!(config.max_goals_per_intent, 10);
        assert_eq!(config.max_plan_steps, 50);
        assert_eq!(config.max_reasoning_depth, 5);
        assert!((config.confidence_threshold - 0.6).abs() < f64::EPSILON);
        assert!(config.require_validation);
        assert!(config.enable_reflection);
        assert!(config.enable_recovery);
        assert_eq!(config.max_recovery_attempts, 3);
        assert_eq!(config.planning_timeout_seconds, 30);
        assert_eq!(config.reasoning_timeout_seconds, 30);
        assert_eq!(config.execution_timeout_seconds, 120);
    }

    #[test]
    fn test_cognition_event_display_variants() {
        let e1 = CognitionEvent::IntentDetected {
            intent_id: "i1".to_string(),
            intent_type: "Query".to_string(),
            confidence: 0.8,
        };
        assert!(format!("{}", e1).contains("Intent detected"));

        let e2 = CognitionEvent::GoalsDecomposed {
            intent_id: "i1".to_string(),
            goal_count: 5,
        };
        assert!(format!("{}", e2).contains("Goals decomposed"));

        let e3 = CognitionEvent::CognitiveStateChanged {
            previous: "Idle".to_string(),
            current: "AnalyzingIntent".to_string(),
        };
        assert!(format!("{}", e3).contains("Cognitive state changed"));
    }

    #[test]
    fn test_plan_step_creation() {
        let step = PlanStep {
            id: StepId("s1".to_string()),
            description: "Do something".to_string(),
            step_type: StepType::Atomic,
            dependencies: vec![],
            required_capabilities: vec![CapabilityId("compute".to_string())],
            state: StepState::Pending,
            estimated_duration_ms: 1000,
            max_retries: 2,
            metadata: HashMap::new(),
        };
        assert_eq!(step.id.0, "s1");
        assert_eq!(step.description, "Do something");
        assert_eq!(step.estimated_duration_ms, 1000);
        assert_eq!(step.max_retries, 2);
        match step.state {
            StepState::Pending => {}
            _ => panic!("expected Pending"),
        }
    }

    #[test]
    fn test_goal_decomposition_creation() {
        let now = Utc::now();
        let decomp = GoalDecomposition {
            intent_id: IntentId("int-1".to_string()),
            goals: vec![],
            dependency_graph: vec![],
            estimated_complexity: 0.5,
            timestamp: now,
        };
        assert_eq!(decomp.intent_id.0, "int-1");
        assert!((decomp.estimated_complexity - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_action_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ActionId("a1".to_string()));
        set.insert(ActionId("a1".to_string()));
        assert_eq!(set.len(), 1);
        set.insert(ActionId("a2".to_string()));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_intent_input_creation() {
        let input = IntentInput {
            raw_text: "hello".to_string(),
            context: None,
            source: "voice".to_string(),
            metadata: HashMap::new(),
        };
        assert_eq!(input.raw_text, "hello");
        assert_eq!(input.source, "voice");
    }

    #[test]
    fn test_plan_creation() {
        let now = Utc::now();
        let plan = Plan {
            id: PlanId("p1".to_string()),
            goals: vec![GoalId("g1".to_string())],
            steps: vec![],
            state: PlanState::Draft,
            estimated_total_duration_ms: 5000,
            parallelism_possible: true,
            fallback_plan_id: None,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(plan.id.0, "p1");
        assert_eq!(plan.goals.len(), 1);
        assert!(plan.parallelism_possible);
    }

    #[test]
    fn test_confidence_score_levels() {
        let vl = ConfidenceScore::new(0.05).unwrap();
        assert_eq!(vl.level, ConfidenceLevel::VeryLow);

        let l = ConfidenceScore::new(0.25).unwrap();
        assert_eq!(l.level, ConfidenceLevel::Low);

        let m = ConfidenceScore::new(0.5).unwrap();
        assert_eq!(m.level, ConfidenceLevel::Medium);

        let h = ConfidenceScore::new(0.75).unwrap();
        assert_eq!(h.level, ConfidenceLevel::High);

        let vh = ConfidenceScore::new(0.95).unwrap();
        assert_eq!(vh.level, ConfidenceLevel::VeryHigh);
    }

    #[test]
    fn test_capability_id_newtype() {
        let cap = CapabilityId("vision".to_string());
        assert_eq!(cap.to_string(), "vision");
        assert_eq!(cap.0, "vision");
    }

    #[test]
    fn test_skill_id_newtype() {
        let skill = SkillId("ocr".to_string());
        assert_eq!(skill.to_string(), "ocr");
        assert_eq!(skill.0, "ocr");
    }

    #[test]
    fn test_reasoning_output_creation() {
        let output = ReasoningOutput {
            conclusion: "Therefore X".to_string(),
            confidence: ConfidenceScore::new(0.8).unwrap(),
            steps: vec![ReasoningStep {
                index: 0,
                premise: "A is B".to_string(),
                inference: "B is C".to_string(),
                confidence: 0.9,
                source: "rule1".to_string(),
            }],
            duration_ms: 50,
            context_freshness: 0.9,
            contributing_sources: vec!["WorldModel".to_string()],
        };
        assert_eq!(output.conclusion, "Therefore X");
        assert_eq!(output.steps.len(), 1);
        assert_eq!(output.duration_ms, 50);
    }

    #[test]
    fn test_assembled_context_fusion_fields() {
        let ctx = AssembledContext {
            id: ContextId("ctx-1".to_string()),
            sources: vec![],
            world_snapshot: None,
            personality_context: None,
            relevant_history: vec![],
            constraints: vec![],
            priority_hints: vec![],
            assembly_time_ms: 0,
            timestamp: Utc::now(),
            fusion_data: Some(serde_json::json!({"test": "value"})),
            fusion_confidence: 0.95,
            source_count: 3,
        };
        assert!((ctx.fusion_confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(ctx.source_count, 3);
        assert!(ctx.get_field("test").is_some());
        assert!(ctx.get_field("nonexistent").is_none());
    }
}
