use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a context snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(pub String);

impl ContextId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifies the source of a context snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextSource {
    Environment,
    Conversation,
    Memory,
    Activity,
    Device,
    Visual,
    Audio,
    Emotional,
    User,
    Personality,
    WorldModel,
    SystemState,
    ExternalService(String),
}

impl fmt::Display for ContextSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment => write!(f, "Environment"),
            Self::Conversation => write!(f, "Conversation"),
            Self::Memory => write!(f, "Memory"),
            Self::Activity => write!(f, "Activity"),
            Self::Device => write!(f, "Device"),
            Self::Visual => write!(f, "Visual"),
            Self::Audio => write!(f, "Audio"),
            Self::Emotional => write!(f, "Emotional"),
            Self::User => write!(f, "User"),
            Self::Personality => write!(f, "Personality"),
            Self::WorldModel => write!(f, "WorldModel"),
            Self::SystemState => write!(f, "SystemState"),
            Self::ExternalService(name) => write!(f, "ExternalService({})", name),
        }
    }
}

/// Priority level for a context source.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum ContextPriority {
    Critical = 4,
    High = 3,
    #[default]
    Medium = 2,
    Low = 1,
    Background = 0,
}

/// A point-in-time snapshot of context from a single source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Unique identifier for this snapshot.
    pub id: ContextId,

    /// Which source produced this snapshot.
    pub source: ContextSource,

    /// Priority of this context source.
    pub priority: ContextPriority,

    /// Confidence in this snapshot (0.0 - 1.0).
    pub confidence: f64,

    /// Seconds since this snapshot was captured.
    pub freshness: u64,

    /// Relevance to the current intent (0.0 - 1.0).
    pub relevance: f64,

    /// When this snapshot was captured.
    pub captured_at: DateTime<Utc>,

    /// The actual context data.
    pub data: serde_json::Value,

    /// Estimated size in bytes.
    pub size_bytes: usize,
}

impl ContextSnapshot {
    /// Create a new context snapshot.
    pub fn new(source: ContextSource, data: serde_json::Value) -> Self {
        let size = serde_json::to_string(&data).map(|s| s.len()).unwrap_or(0);
        Self {
            id: ContextId::new(),
            source,
            priority: ContextPriority::default(),
            confidence: 1.0,
            freshness: 0,
            relevance: 0.5,
            captured_at: Utc::now(),
            data,
            size_bytes: size,
        }
    }

    /// Set priority (builder pattern).
    pub fn with_priority(mut self, priority: ContextPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set confidence (builder pattern).
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set relevance (builder pattern).
    pub fn with_relevance(mut self, relevance: f64) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }

    /// Calculate a composite score combining priority, confidence, freshness, and relevance.
    pub fn score(&self) -> f64 {
        let priority_score = match self.priority {
            ContextPriority::Critical => 4.0,
            ContextPriority::High => 3.0,
            ContextPriority::Medium => 2.0,
            ContextPriority::Low => 1.0,
            ContextPriority::Background => 0.5,
        };

        let freshness_score = 1.0 / (1.0 + self.freshness as f64 / 60.0);

        (priority_score * 0.3)
            + (self.confidence * 0.3)
            + (freshness_score * 0.2)
            + (self.relevance * 0.2)
    }

    /// Check if this snapshot is stale (older than max_age_seconds).
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        self.freshness > max_age_seconds
    }
}

/// An event representing a change to a context source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdate {
    /// Which source produced this update.
    pub source: ContextSource,

    /// The new snapshot.
    pub snapshot: ContextSnapshot,

    /// What changed (optional description).
    pub change_description: Option<String>,

    /// When this update occurred.
    pub timestamp: DateTime<Utc>,
}

impl ContextUpdate {
    pub fn new(source: ContextSource, snapshot: ContextSnapshot) -> Self {
        Self {
            source,
            snapshot,
            change_description: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.change_description = Some(description.into());
        self
    }
}

/// Configuration for context freshness thresholds per source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessConfig {
    /// Maximum age in seconds before environment context is considered stale.
    pub environment_max_age: u64,

    /// Maximum age in seconds before conversation context is considered stale.
    pub conversation_max_age: u64,

    /// Maximum age in seconds before memory context is considered stale.
    pub memory_max_age: u64,

    /// Maximum age in seconds before activity context is considered stale.
    pub activity_max_age: u64,

    /// Maximum age in seconds before device context is considered stale.
    pub device_max_age: u64,

    /// Maximum age in seconds before visual context is considered stale.
    pub visual_max_age: u64,

    /// Maximum age in seconds before audio context is considered stale.
    pub audio_max_age: u64,

    /// Maximum age in seconds before emotional context is considered stale.
    pub emotional_max_age: u64,
}

impl Default for FreshnessConfig {
    fn default() -> Self {
        Self {
            environment_max_age: 60,
            conversation_max_age: 300,
            memory_max_age: 600,
            activity_max_age: 120,
            device_max_age: 300,
            visual_max_age: 30,
            audio_max_age: 10,
            emotional_max_age: 300,
        }
    }
}

impl FreshnessConfig {
    /// Get the max age for a given source.
    pub fn max_age_for(&self, source: &ContextSource) -> u64 {
        match source {
            ContextSource::Environment => self.environment_max_age,
            ContextSource::Conversation => self.conversation_max_age,
            ContextSource::Memory => self.memory_max_age,
            ContextSource::Activity => self.activity_max_age,
            ContextSource::Device => self.device_max_age,
            ContextSource::Visual => self.visual_max_age,
            ContextSource::Audio => self.audio_max_age,
            ContextSource::Emotional => self.emotional_max_age,
            _ => 300,
        }
    }
}
