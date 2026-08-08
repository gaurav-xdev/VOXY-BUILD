use std::fmt;

#[derive(Debug, Clone)]
pub enum LearningEvent {
    PreferenceUpdated {
        preference_id: String,
        category: String,
        old_value: String,
        new_value: String,
        confidence: f64,
    },
    BehaviorAdapted {
        behavior_id: String,
        adaptation: String,
        trigger: String,
    },
    ReinforcementApplied {
        hook_id: String,
        action: String,
        reward: f64,
    },
    ConfidenceCalibrated {
        estimator_id: String,
        old_threshold: f64,
        new_threshold: f64,
        samples: usize,
    },
    FeedbackProcessed {
        feedback_id: String,
        sentiment: String,
        impact: f64,
    },
    StrategyEvolved {
        strategy_id: String,
        old_effectiveness: f64,
        new_effectiveness: f64,
    },
    LearningPolicyUpdated {
        policy_id: String,
        change: String,
    },
    ThresholdAdjusted {
        threshold_id: String,
        old_value: f64,
        new_value: f64,
        reason: String,
    },
}

impl fmt::Display for LearningEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreferenceUpdated {
                preference_id,
                category,
                old_value,
                new_value,
                confidence,
            } => {
                write!(
                    f,
                    "PreferenceUpdated: id={}, category={}, old={}, new={}, confidence={}",
                    preference_id, category, old_value, new_value, confidence
                )
            }
            Self::BehaviorAdapted {
                behavior_id,
                adaptation,
                trigger,
            } => {
                write!(
                    f,
                    "BehaviorAdapted: id={}, adaptation={}, trigger={}",
                    behavior_id, adaptation, trigger
                )
            }
            Self::ReinforcementApplied {
                hook_id,
                action,
                reward,
            } => {
                write!(
                    f,
                    "ReinforcementApplied: hook={}, action={}, reward={}",
                    hook_id, action, reward
                )
            }
            Self::ConfidenceCalibrated {
                estimator_id,
                old_threshold,
                new_threshold,
                samples,
            } => {
                write!(f, "ConfidenceCalibrated: estimator={}, old_threshold={}, new_threshold={}, samples={}", estimator_id, old_threshold, new_threshold, samples)
            }
            Self::FeedbackProcessed {
                feedback_id,
                sentiment,
                impact,
            } => {
                write!(
                    f,
                    "FeedbackProcessed: id={}, sentiment={}, impact={}",
                    feedback_id, sentiment, impact
                )
            }
            Self::StrategyEvolved {
                strategy_id,
                old_effectiveness,
                new_effectiveness,
            } => {
                write!(
                    f,
                    "StrategyEvolved: id={}, old_eff={}, new_eff={}",
                    strategy_id, old_effectiveness, new_effectiveness
                )
            }
            Self::LearningPolicyUpdated { policy_id, change } => {
                write!(
                    f,
                    "LearningPolicyUpdated: id={}, change={}",
                    policy_id, change
                )
            }
            Self::ThresholdAdjusted {
                threshold_id,
                old_value,
                new_value,
                reason,
            } => {
                write!(
                    f,
                    "ThresholdAdjusted: id={}, old={}, new={}, reason={}",
                    threshold_id, old_value, new_value, reason
                )
            }
        }
    }
}
