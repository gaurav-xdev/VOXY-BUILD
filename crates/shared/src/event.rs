//! Event types for inter-crate communication.
//!
//! Events are the primary communication mechanism between subsystems.
//! All events carry trace context for distributed tracing correlation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority levels for events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Event type for inter-crate communication.
///
/// Events are immutable once created. Use the builder pattern to construct them.
/// Fields are private to allow future additions without breaking the public API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    id: Uuid,
    topic: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    source: String,
    payload: Vec<u8>,
    correlation_id: Option<Uuid>,
    priority: Priority,
    schema_version: String,
    /// Distributed tracing: trace ID for correlation across services.
    trace_id: Option<String>,
    /// Distributed tracing: span ID for parent-child relationship.
    span_id: Option<String>,
    /// Cancellation token ID. If set, subscribers should check this token
    /// before processing and abort if cancelled.
    cancellation_token_id: Option<Uuid>,
    /// Custom metadata for extensibility.
    metadata: std::collections::HashMap<String, String>,
}

impl Event {
    /// Create a new event.
    pub fn new(topic: impl Into<String>, source: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            topic: topic.into(),
            timestamp: chrono::Utc::now(),
            source: source.into(),
            payload,
            correlation_id: None,
            priority: Priority::default(),
            schema_version: "1.0.0".to_string(),
            trace_id: None,
            span_id: None,
            cancellation_token_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get the event ID.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the topic path.
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get the event timestamp.
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    /// Get the source subsystem.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the payload bytes.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Get the correlation ID.
    pub fn correlation_id(&self) -> Option<Uuid> {
        self.correlation_id
    }

    /// Get the priority.
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Get the schema version.
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Get the trace ID.
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// Get the span ID.
    pub fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }

    /// Get the cancellation token ID.
    pub fn cancellation_token_id(&self) -> Option<Uuid> {
        self.cancellation_token_id
    }

    /// Get a metadata value by key.
    pub fn metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Get all metadata.
    pub fn metadata_map(&self) -> &std::collections::HashMap<String, String> {
        &self.metadata
    }

    /// Set the correlation ID for request/response.
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the schema version.
    pub fn with_schema_version(mut self, version: impl Into<String>) -> Self {
        self.schema_version = version.into();
        self
    }

    /// Set the trace ID for distributed tracing.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set the span ID for distributed tracing.
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Set the cancellation token ID.
    pub fn with_cancellation_token(mut self, token_id: Uuid) -> Self {
        self.cancellation_token_id = Some(token_id);
        self
    }

    /// Add a metadata entry.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if this event has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token_id.is_some()
    }

    /// Serialize the payload to JSON.
    pub fn from_json<T: Serialize>(topic: &str, source: &str, data: &T) -> crate::Result<Self> {
        let payload = serde_json::to_vec(data)?;
        Ok(Self::new(topic, source, payload))
    }

    /// Deserialize the payload from JSON.
    pub fn to_json<T: for<'de> Deserialize<'de>>(&self) -> crate::Result<T> {
        Ok(serde_json::from_slice(&self.payload)?)
    }
}

/// Trait for typed events.
pub trait TypedEvent: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Get the topic for this event type.
    fn topic() -> &'static str;

    /// Get the source subsystem.
    fn source() -> &'static str;

    /// Get the schema version for this event type.
    fn schema_version() -> &'static str {
        "1.0.0"
    }
}

/// Create an event from a typed event.
pub fn create_event<T: TypedEvent>(event: &T) -> crate::Result<Event> {
    let mut e = Event::from_json(T::topic(), T::source(), event)?;
    e = e.with_schema_version(T::schema_version());
    Ok(e)
}

/// Deserialize a typed event from an event.
pub fn deserialize_event<T: TypedEvent>(event: &Event) -> crate::Result<T> {
    event.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
    }

    impl TypedEvent for TestEvent {
        fn topic() -> &'static str {
            "test.event"
        }

        fn source() -> &'static str {
            "test"
        }
    }

    #[test]
    fn event_creation() {
        let event = Event::new("test.topic", "test", vec![1, 2, 3]);
        assert_eq!(event.topic(), "test.topic");
        assert_eq!(event.source(), "test");
        assert_eq!(event.payload(), &[1, 2, 3]);
        assert!(event.correlation_id().is_none());
    }

    #[test]
    fn event_with_correlation() {
        let correlation_id = Uuid::new_v4();
        let event = Event::new("test.topic", "test", vec![]).with_correlation_id(correlation_id);
        assert_eq!(event.correlation_id(), Some(correlation_id));
    }

    #[test]
    fn event_with_trace_context() {
        let event = Event::new("test.topic", "test", vec![])
            .with_trace_id("trace-123")
            .with_span_id("span-456");
        assert_eq!(event.trace_id(), Some("trace-123"));
        assert_eq!(event.span_id(), Some("span-456"));
    }

    #[test]
    fn event_with_cancellation() {
        let token_id = Uuid::new_v4();
        let event = Event::new("test.topic", "test", vec![]).with_cancellation_token(token_id);
        assert!(event.is_cancelled());
        assert_eq!(event.cancellation_token_id(), Some(token_id));
    }

    #[test]
    fn event_with_metadata() {
        let event = Event::new("test.topic", "test", vec![])
            .with_metadata("version", "2")
            .with_metadata("region", "us-east-1");
        assert_eq!(event.metadata("version"), Some("2"));
        assert_eq!(event.metadata("region"), Some("us-east-1"));
        assert!(event.metadata("missing").is_none());
    }

    #[test]
    fn event_json_roundtrip() {
        let test_event = TestEvent {
            message: "hello".to_string(),
        };
        let event = create_event(&test_event).unwrap();
        assert_eq!(event.schema_version(), "1.0.0");
        let deserialized: TestEvent = deserialize_event(&event).unwrap();
        assert_eq!(deserialized.message, "hello");
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn event_builder_chain() {
        let event = Event::new("topic", "src", vec![])
            .with_priority(Priority::High)
            .with_trace_id("t1")
            .with_metadata("key", "val");

        assert_eq!(event.priority(), Priority::High);
        assert_eq!(event.trace_id(), Some("t1"));
        assert_eq!(event.metadata("key"), Some("val"));
    }
}
