use std::fmt;

#[derive(Debug, Clone)]
pub enum SkillsEvent {
    SkillRegistered {
        skill_id: String,
        skill_name: String,
    },
    SkillUnregistered {
        skill_id: String,
    },
    SkillExecutionStarted {
        skill_id: String,
        invocation_id: String,
    },
    SkillExecutionCompleted {
        skill_id: String,
        invocation_id: String,
        duration_ms: u64,
    },
    SkillExecutionFailed {
        skill_id: String,
        invocation_id: String,
        error: String,
    },
    CapabilityDiscovered {
        capability_id: String,
        description: String,
    },
}

impl fmt::Display for SkillsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkillRegistered {
                skill_id,
                skill_name,
            } => {
                write!(f, "Skill registered: {} ({})", skill_id, skill_name)
            }
            Self::SkillUnregistered { skill_id } => {
                write!(f, "Skill unregistered: {}", skill_id)
            }
            Self::SkillExecutionStarted {
                skill_id,
                invocation_id,
            } => {
                write!(
                    f,
                    "Skill execution started: {} ({})",
                    skill_id, invocation_id
                )
            }
            Self::SkillExecutionCompleted {
                skill_id,
                invocation_id,
                duration_ms,
            } => {
                write!(
                    f,
                    "Skill execution completed: {} ({}) in {}ms",
                    skill_id, invocation_id, duration_ms
                )
            }
            Self::SkillExecutionFailed {
                skill_id,
                invocation_id,
                error,
            } => {
                write!(
                    f,
                    "Skill execution failed: {} ({}) - {}",
                    skill_id, invocation_id, error
                )
            }
            Self::CapabilityDiscovered {
                capability_id,
                description,
            } => {
                write!(
                    f,
                    "Capability discovered: {} - {}",
                    capability_id, description
                )
            }
        }
    }
}
