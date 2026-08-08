use std::fmt;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum CognitiveState {
    Idle,
    AnalyzingIntent,
    DecomposingGoals,
    Planning,
    Reasoning,
    AssemblingContext,
    SelectingTools,
    Executing,
    Validating,
    Recovering,
    Reflecting,
    Completed,
    Failed(String),
}

impl fmt::Display for CognitiveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::AnalyzingIntent => write!(f, "AnalyzingIntent"),
            Self::DecomposingGoals => write!(f, "DecomposingGoals"),
            Self::Planning => write!(f, "Planning"),
            Self::Reasoning => write!(f, "Reasoning"),
            Self::AssemblingContext => write!(f, "AssemblingContext"),
            Self::SelectingTools => write!(f, "SelectingTools"),
            Self::Executing => write!(f, "Executing"),
            Self::Validating => write!(f, "Validating"),
            Self::Recovering => write!(f, "Recovering"),
            Self::Reflecting => write!(f, "Reflecting"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(reason) => write!(f, "Failed({})", reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentId(pub String);

impl fmt::Display for IntentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanId(pub String);

impl fmt::Display for PlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StepId(pub String);

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReasoningId(pub String);

impl fmt::Display for ReasoningId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextId(pub String);

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId(pub String);

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectionId(pub String);

impl fmt::Display for ReflectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GoalId(pub String);

impl fmt::Display for GoalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Low,
    Medium,
    High,
    Critical,
}

impl PartialOrd for Urgency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Urgency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn rank(u: &Urgency) -> u8 {
            match u {
                Urgency::Low => 0,
                Urgency::Medium => 1,
                Urgency::High => 2,
                Urgency::Critical => 3,
            }
        }
        rank(self).cmp(&rank(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl ConfidenceLevel {
    pub fn value(&self) -> f64 {
        match self {
            Self::VeryLow => 0.1,
            Self::Low => 0.3,
            Self::Medium => 0.5,
            Self::High => 0.7,
            Self::VeryHigh => 0.9,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfidenceScore {
    pub value: f64,
    pub level: ConfidenceLevel,
    pub explanations: Vec<String>,
}

impl ConfidenceScore {
    pub fn new(value: f64) -> Result<Self> {
        let clamped = value.clamp(0.0, 1.0);
        let level = if clamped < 0.2 {
            ConfidenceLevel::VeryLow
        } else if clamped < 0.4 {
            ConfidenceLevel::Low
        } else if clamped < 0.6 {
            ConfidenceLevel::Medium
        } else if clamped < 0.8 {
            ConfidenceLevel::High
        } else {
            ConfidenceLevel::VeryHigh
        };
        Ok(Self {
            value: clamped,
            level,
            explanations: Vec::new(),
        })
    }

    pub fn is_sufficient(&self, threshold: f64) -> bool {
        self.value >= threshold
    }
}
