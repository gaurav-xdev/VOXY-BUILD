use crate::error::Result;
use crate::provider::ContextProvider;
use crate::types::{ContextPriority, ContextSnapshot, ContextSource};
use async_trait::async_trait;

/// Provides conversation context (history, topic, sentiment).
pub struct ConversationContextProvider {
    conversation_id: Option<String>,
    turn_count: usize,
    current_topic: Option<String>,
    sentiment: Option<f64>,
}

impl ConversationContextProvider {
    /// Create a new conversation context provider.
    pub fn new() -> Self {
        Self {
            conversation_id: None,
            turn_count: 0,
            current_topic: None,
            sentiment: None,
        }
    }

    /// Set the conversation ID.
    pub fn set_conversation_id(&mut self, id: String) {
        self.conversation_id = Some(id);
    }

    /// Update conversation state.
    pub fn update_state(
        &mut self,
        turn_count: usize,
        current_topic: Option<String>,
        sentiment: Option<f64>,
    ) {
        self.turn_count = turn_count;
        self.current_topic = current_topic;
        self.sentiment = sentiment;
    }
}

impl Default for ConversationContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextProvider for ConversationContextProvider {
    fn name(&self) -> &str {
        "conversation"
    }

    fn source(&self) -> ContextSource {
        ContextSource::Conversation
    }

    fn default_priority(&self) -> ContextPriority {
        ContextPriority::High
    }

    async fn collect(&self) -> Result<ContextSnapshot> {
        let data = serde_json::json!({
            "conversation_id": self.conversation_id,
            "turn_count": self.turn_count,
            "current_topic": self.current_topic,
            "sentiment": self.sentiment,
        });

        Ok(ContextSnapshot::new(ContextSource::Conversation, data))
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_conversation() {
        let mut provider = ConversationContextProvider::new();
        provider.set_conversation_id("test-conv-1".to_string());
        provider.update_state(5, Some("programming".to_string()), Some(0.8));

        let snapshot = provider.collect().await.unwrap();
        assert_eq!(snapshot.source, ContextSource::Conversation);
        assert_eq!(
            snapshot.data["conversation_id"].as_str(),
            Some("test-conv-1")
        );
        assert_eq!(snapshot.data["turn_count"].as_u64(), Some(5));
    }

    #[test]
    fn test_default_state() {
        let provider = ConversationContextProvider::new();
        assert!(provider.conversation_id.is_none());
        assert_eq!(provider.turn_count, 0);
    }
}
