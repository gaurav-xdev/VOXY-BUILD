use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillId(pub String);

impl SkillId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvocationId(pub String);

impl InvocationId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InvocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
