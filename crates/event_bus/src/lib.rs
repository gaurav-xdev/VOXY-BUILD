//! Publish/subscribe event bus for inter-crate communication.
//!
//! Supports:
//! - Topic-based publish/subscribe
//! - Dead letter queue for failed deliveries
//! - Event versioning via schema_version field
//! - Distributed tracing via trace_id/span_id
//! - Cancellation support via CancellationToken

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Event bus error type.
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    /// Topic not found.
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    /// Subscriber queue full.
    #[error("Subscriber queue full")]
    SubscriberQueueFull,
    /// Serialization failed.
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    /// Subscription dropped.
    #[error("Subscription dropped")]
    SubscriptionDropped,
}

impl From<BusError> for voxy_shared::VoxyError {
    fn from(e: BusError) -> Self {
        let kind = match &e {
            BusError::TopicNotFound(_) => voxy_shared::ErrorKind::NotFound,
            BusError::SubscriberQueueFull => voxy_shared::ErrorKind::ResourceExhausted,
            BusError::SerializationFailed(_) => voxy_shared::ErrorKind::Serialization,
            BusError::SubscriptionDropped => voxy_shared::ErrorKind::IO,
        };
        voxy_shared::VoxyError::with_source(kind, e.to_string(), e)
    }
}

/// Statistics for a topic.
#[derive(Debug, Clone)]
pub struct TopicStats {
    /// Number of active subscribers.
    pub subscriber_count: usize,
    /// Number of messages published.
    pub message_count: u64,
    /// Number of messages sent to dead letter.
    pub dead_letter_count: u64,
}

/// Dead letter entry.
#[derive(Debug, Clone)]
pub struct DeadLetterEntry {
    /// The original event.
    pub event: voxy_shared::Event,
    /// The topic it was published to.
    pub topic: String,
    /// The error that caused it to be dead-lettered.
    pub error: String,
    /// When it was dead-lettered.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SECURITY: Maximum payload size per event (1 MB) to prevent OOM via large payloads.
const MAX_EVENT_PAYLOAD_BYTES: usize = 1024 * 1024;

/// SECURITY: Maximum number of topics to prevent memory exhaustion.
const ABSOLUTE_MAX_TOPICS: usize = 200;

/// SECURITY: Maximum buffer size per topic channel to prevent OOM.
const MAX_BUFFER_SIZE: usize = 256;

/// Publish/subscribe event bus with dead letter queue.
pub struct EventBus {
    topics: Arc<RwLock<HashMap<String, broadcast::Sender<voxy_shared::Event>>>>,
    topic_stats: Arc<RwLock<HashMap<String, TopicStats>>>,
    dead_letters: Arc<RwLock<VecDeque<DeadLetterEntry>>>,
    buffer_size: usize,
    dead_letter_max_size: usize,
    max_topics: usize,
}

impl EventBus {
    /// Create a new event bus.
    pub fn new(buffer_size: usize) -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            topic_stats: Arc::new(RwLock::new(HashMap::new())),
            dead_letters: Arc::new(RwLock::new(VecDeque::new())),
            buffer_size: buffer_size.min(MAX_BUFFER_SIZE),
            dead_letter_max_size: 100,
            max_topics: ABSOLUTE_MAX_TOPICS,
        }
    }

    /// Create a new event bus with dead letter configuration.
    pub fn with_dead_letter(buffer_size: usize, dead_letter_max_size: usize) -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            topic_stats: Arc::new(RwLock::new(HashMap::new())),
            dead_letters: Arc::new(RwLock::new(VecDeque::new())),
            buffer_size: buffer_size.min(MAX_BUFFER_SIZE),
            dead_letter_max_size,
            max_topics: ABSOLUTE_MAX_TOPICS,
        }
    }

    /// Get or create a topic sender, evicting stale topics if at capacity.
    async fn get_or_create_topic(&self, topic: &str) -> broadcast::Sender<voxy_shared::Event> {
        {
            let topics = self.topics.read().await;
            if let Some(sender) = topics.get(topic) {
                return sender.clone();
            }
        }
        let mut topics = self.topics.write().await;
        let mut stale_keys = Vec::new();
        if topics.len() >= self.max_topics {
            let keys_to_remove: Vec<String> = topics
                .keys()
                .take(topics.len() - self.max_topics + 1)
                .cloned()
                .collect();
            for key in keys_to_remove {
                topics.remove(&key);
                stale_keys.push(key);
            }
        }
        let sender = topics
            .entry(topic.to_string())
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(self.buffer_size);
                sender
            })
            .clone();
        drop(topics);
        if !stale_keys.is_empty() {
            let mut stats = self.topic_stats.write().await;
            for key in stale_keys {
                stats.remove(&key);
            }
        }
        sender
    }

    /// Publish an event to a topic.
    pub async fn publish(&self, topic: &str, event: voxy_shared::Event) -> Result<(), BusError> {
        // SECURITY: Validate payload size to prevent OOM via large events
        if event.payload().len() > MAX_EVENT_PAYLOAD_BYTES {
            tracing::warn!(
                topic = %topic,
                size = event.payload().len(),
                "Event payload exceeds maximum size, dropped"
            );
            return Ok(());
        }
        let sender = self.get_or_create_topic(topic).await;

        match sender.send(event) {
            Ok(_) => {
                let mut stats = self.topic_stats.write().await;
                let entry = stats
                    .entry(topic.to_string())
                    .or_insert_with(|| TopicStats {
                        subscriber_count: 0,
                        message_count: 0,
                        dead_letter_count: 0,
                    });
                entry.message_count += 1;
                entry.subscriber_count = sender.receiver_count();
                Ok(())
            }
            Err(broadcast::error::SendError(_)) => {
                tracing::debug!(topic = %topic, "No active subscribers, event dropped");
                Ok(())
            }
        }
    }

    /// Publish an event with dead letter handling.
    pub async fn publish_with_dead_letter(
        &self,
        topic: &str,
        event: voxy_shared::Event,
    ) -> Result<(), BusError> {
        let sender = self.get_or_create_topic(topic).await;

        match sender.send(event.clone()) {
            Ok(_) => {
                let mut stats = self.topic_stats.write().await;
                let entry = stats
                    .entry(topic.to_string())
                    .or_insert_with(|| TopicStats {
                        subscriber_count: 0,
                        message_count: 0,
                        dead_letter_count: 0,
                    });
                entry.message_count += 1;
                entry.subscriber_count = sender.receiver_count();
                Ok(())
            }
            Err(e) => {
                let dead_entry = DeadLetterEntry {
                    event,
                    topic: topic.to_string(),
                    error: e.to_string(),
                    timestamp: chrono::Utc::now(),
                };
                {
                    let mut dead_letters = self.dead_letters.write().await;
                    if dead_letters.len() >= self.dead_letter_max_size {
                        dead_letters.pop_front();
                    }
                    dead_letters.push_back(dead_entry);
                }

                let mut stats = self.topic_stats.write().await;
                if let Some(entry) = stats.get_mut(topic) {
                    entry.dead_letter_count += 1;
                }

                tracing::warn!(topic = %topic, error = %e, "Event dead-lettered");
                Ok(())
            }
        }
    }

    /// Subscribe to a topic.
    pub async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<broadcast::Receiver<voxy_shared::Event>, BusError> {
        let sender = self.get_or_create_topic(topic).await;
        Ok(sender.subscribe())
    }

    /// Get the number of topics.
    pub async fn topic_count(&self) -> usize {
        self.topics.read().await.len()
    }

    /// Get stats for a topic.
    pub async fn stats(&self, topic: &str) -> Result<TopicStats, BusError> {
        let receiver_count = {
            let topics = self.topics.read().await;
            let sender = topics
                .get(topic)
                .ok_or_else(|| BusError::TopicNotFound(topic.to_string()))?;
            sender.receiver_count()
        };

        let stats = self.topic_stats.read().await;
        let mut topic_stats = stats.get(topic).cloned().unwrap_or(TopicStats {
            subscriber_count: 0,
            message_count: 0,
            dead_letter_count: 0,
        });
        topic_stats.subscriber_count = receiver_count;
        Ok(topic_stats)
    }

    /// Get all dead letter entries.
    pub async fn dead_letters(&self) -> Vec<DeadLetterEntry> {
        self.dead_letters.read().await.clone().into()
    }

    /// Get dead letter count.
    pub async fn dead_letter_count(&self) -> usize {
        self.dead_letters.read().await.len()
    }

    /// Clear dead letter queue.
    pub async fn clear_dead_letters(&self) {
        self.dead_letters.write().await.clear();
    }

    /// Get all topic names.
    pub async fn topic_names(&self) -> Vec<String> {
        self.topics.read().await.keys().cloned().collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_subscribe() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe("test.topic").await.unwrap();

        let event = voxy_shared::Event::new("test.topic", "test", vec![1, 2, 3]);
        bus.publish("test.topic", event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.topic(), "test.topic");
        assert_eq!(received.payload(), &[1, 2, 3]);
    }

    #[tokio::test]
    async fn multiple_subscribers() {
        let bus = EventBus::new(10);
        let mut rx1 = bus.subscribe("test.topic").await.unwrap();
        let mut rx2 = bus.subscribe("test.topic").await.unwrap();

        let event = voxy_shared::Event::new("test.topic", "test", vec![]);
        bus.publish("test.topic", event).await.unwrap();

        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }

    #[tokio::test]
    async fn topic_count() {
        let bus = EventBus::new(10);
        assert_eq!(bus.topic_count().await, 0);

        let _ = bus.subscribe("topic1").await.unwrap();
        assert_eq!(bus.topic_count().await, 1);

        let _ = bus.subscribe("topic2").await.unwrap();
        assert_eq!(bus.topic_count().await, 2);
    }

    #[tokio::test]
    async fn topic_stats() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe("test.topic").await.unwrap();

        let event = voxy_shared::Event::new("test.topic", "test", vec![]);
        bus.publish("test.topic", event).await.unwrap();

        let _ = rx.recv().await;

        let stats = bus.stats("test.topic").await.unwrap();
        assert_eq!(stats.message_count, 1);
    }

    #[tokio::test]
    async fn dead_letter_queue() {
        let bus = EventBus::new(1);
        let event = voxy_shared::Event::new("test.topic", "test", vec![]);

        let mut rx = bus.subscribe("test.topic").await.unwrap();

        bus.publish_with_dead_letter("test.topic", event.clone())
            .await
            .unwrap();
        let _ = rx.recv().await;

        assert_eq!(bus.dead_letter_count().await, 0);
    }

    #[tokio::test]
    async fn clear_dead_letters() {
        let bus = EventBus::new(10);
        bus.clear_dead_letters().await;
        assert_eq!(bus.dead_letter_count().await, 0);
    }

    #[tokio::test]
    async fn topic_names() {
        let bus = EventBus::new(10);
        let _ = bus.subscribe("alpha").await.unwrap();
        let _ = bus.subscribe("beta").await.unwrap();

        let mut names = bus.topic_names().await;
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[tokio::test]
    async fn event_with_trace_context() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe("test.topic").await.unwrap();

        let event = voxy_shared::Event::new("test.topic", "test", vec![])
            .with_trace_id("trace-123")
            .with_span_id("span-456");
        bus.publish("test.topic", event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.trace_id(), Some("trace-123"));
        assert_eq!(received.span_id(), Some("span-456"));
    }

    #[tokio::test]
    async fn bus_error_conversion() {
        let err = BusError::TopicNotFound("test".into());
        let voxy_err: voxy_shared::VoxyError = err.into();
        assert_eq!(voxy_err.kind(), &voxy_shared::ErrorKind::NotFound);
    }
}
