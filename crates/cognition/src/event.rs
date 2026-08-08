use std::fmt;

#[derive(Debug, Clone)]
pub enum CognitionEvent {
    IntentDetected {
        intent_id: String,
        intent_type: String,
        confidence: f64,
    },
    GoalsDecomposed {
        intent_id: String,
        goal_count: usize,
    },
    PlanCreated {
        plan_id: String,
        step_count: usize,
    },
    PlanStepStarted {
        plan_id: String,
        step_id: String,
        description: String,
    },
    PlanStepCompleted {
        plan_id: String,
        step_id: String,
    },
    PlanStepFailed {
        plan_id: String,
        step_id: String,
        error: String,
    },
    PlanCompleted {
        plan_id: String,
        success: bool,
    },
    ReasoningPerformed {
        reasoning_id: String,
        conclusion: String,
    },
    ContextAssembled {
        context_id: String,
        source_count: usize,
    },
    ToolSelected {
        tool_id: String,
        confidence: f64,
    },
    ConfidenceEstimated {
        value: f64,
        threshold: f64,
        sufficient: bool,
    },
    ActionValidated {
        action_id: String,
        status: String,
    },
    RecoveryAttempted {
        plan_id: String,
        attempt: u32,
        strategy: String,
    },
    ReflectionCompleted {
        reflection_id: String,
        insight_count: usize,
    },
    CognitiveStateChanged {
        previous: String,
        current: String,
    },
}

impl fmt::Display for CognitionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntentDetected {
                intent_id,
                intent_type,
                confidence,
            } => {
                write!(
                    f,
                    "Intent detected: {} (type: {}, confidence: {})",
                    intent_id, intent_type, confidence
                )
            }
            Self::GoalsDecomposed {
                intent_id,
                goal_count,
            } => {
                write!(
                    f,
                    "Goals decomposed for intent {}: {} goals",
                    intent_id, goal_count
                )
            }
            Self::PlanCreated {
                plan_id,
                step_count,
            } => {
                write!(f, "Plan created: {} with {} steps", plan_id, step_count)
            }
            Self::PlanStepStarted {
                plan_id,
                step_id,
                description,
            } => {
                write!(
                    f,
                    "Plan step started: {}/{} - {}",
                    plan_id, step_id, description
                )
            }
            Self::PlanStepCompleted { plan_id, step_id } => {
                write!(f, "Plan step completed: {}/{}", plan_id, step_id)
            }
            Self::PlanStepFailed {
                plan_id,
                step_id,
                error,
            } => {
                write!(f, "Plan step failed: {}/{} - {}", plan_id, step_id, error)
            }
            Self::PlanCompleted { plan_id, success } => {
                write!(f, "Plan completed: {} success={}", plan_id, success)
            }
            Self::ReasoningPerformed {
                reasoning_id,
                conclusion,
            } => {
                write!(f, "Reasoning performed: {} -> {}", reasoning_id, conclusion)
            }
            Self::ContextAssembled {
                context_id,
                source_count,
            } => {
                write!(
                    f,
                    "Context assembled: {} from {} sources",
                    context_id, source_count
                )
            }
            Self::ToolSelected {
                tool_id,
                confidence,
            } => {
                write!(f, "Tool selected: {} confidence={}", tool_id, confidence)
            }
            Self::ConfidenceEstimated {
                value,
                threshold,
                sufficient,
            } => {
                write!(
                    f,
                    "Confidence estimated: value={} threshold={} sufficient={}",
                    value, threshold, sufficient
                )
            }
            Self::ActionValidated { action_id, status } => {
                write!(f, "Action validated: {} status={}", action_id, status)
            }
            Self::RecoveryAttempted {
                plan_id,
                attempt,
                strategy,
            } => {
                write!(
                    f,
                    "Recovery attempted: {} attempt={} strategy={}",
                    plan_id, attempt, strategy
                )
            }
            Self::ReflectionCompleted {
                reflection_id,
                insight_count,
            } => {
                write!(
                    f,
                    "Reflection completed: {} insights={}",
                    reflection_id, insight_count
                )
            }
            Self::CognitiveStateChanged { previous, current } => {
                write!(f, "Cognitive state changed: {} -> {}", previous, current)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognition_event_display() {
        let event = CognitionEvent::IntentDetected {
            intent_id: "int-1".to_string(),
            intent_type: "Query".to_string(),
            confidence: 0.9,
        };
        let s = format!("{}", event);
        assert!(s.contains("Intent detected"));
        assert!(s.contains("int-1"));

        let event = CognitionEvent::PlanCreated {
            plan_id: "plan-1".to_string(),
            step_count: 5,
        };
        let s = format!("{}", event);
        assert!(s.contains("Plan created"));

        let event = CognitionEvent::PlanCompleted {
            plan_id: "plan-1".to_string(),
            success: true,
        };
        let s = format!("{}", event);
        assert!(s.contains("success=true"));
    }
}
