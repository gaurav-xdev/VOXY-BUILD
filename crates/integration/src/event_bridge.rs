//! Event Bridge — connects all subsystems through the EventBus.
//!
//! Defines the standard event topics for inter-subsystem communication.
//! Every subsystem publishes and subscribes through this bridge.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use voxy_event_bus::EventBus;
use voxy_shared::Event;

use crate::error::{IntegrationError, Result};

// ============================================================================
// Standard Topics
// ============================================================================

/// All standard event topics in the VOXY system.
pub struct Topics;

impl Topics {
    // Voice
    pub const VOICE_WAKE: &'static str = "voice.wake";
    pub const VOICE_INPUT: &'static str = "voice.input";
    pub const VOICE_OUTPUT: &'static str = "voice.output";
    pub const VOICE_BARGE_IN: &'static str = "voice.barge_in";

    // STT
    pub const STT_PARTIAL: &'static str = "stt.partial";
    pub const STT_FINAL: &'static str = "stt.final";

    // LLM
    pub const LLM_REQUEST: &'static str = "llm.request";
    pub const LLM_RESPONSE: &'static str = "llm.response";
    pub const LLM_TOKEN: &'static str = "llm.token";

    // TTS
    pub const TTS_REQUEST: &'static str = "tts.request";
    pub const TTS_AUDIO: &'static str = "tts.audio";

    // Planner
    pub const PLAN_CREATE: &'static str = "plan.create";
    pub const PLAN_STEP_DONE: &'static str = "plan.step_done";
    pub const PLAN_COMPLETE: &'static str = "plan.complete";

    // Task Graph
    pub const TASK_CREATED: &'static str = "task.created";
    pub const TASK_RUNNING: &'static str = "task.running";
    pub const TASK_COMPLETED: &'static str = "task.completed";
    pub const TASK_FAILED: &'static str = "task.failed";

    // Decision Engine
    pub const DECISION_REQUEST: &'static str = "decision.request";
    pub const DECISION_RESULT: &'static str = "decision.result";

    // Memory
    pub const MEMORY_STORED: &'static str = "memory.stored";
    pub const MEMORY_RETRIEVED: &'static str = "memory.retrieved";
    pub const MEMORY_FORGOTTEN: &'static str = "memory.forgotten";

    // Goals
    pub const GOAL_CREATED: &'static str = "goal.created";
    pub const GOAL_COMPLETED: &'static str = "goal.completed";
    pub const GOAL_FAILED: &'static str = "goal.failed";

    // Projects
    pub const PROJECT_UPDATED: &'static str = "project.updated";
    pub const PROJECT_MILESTONE: &'static str = "project.milestone";

    // Agents
    pub const AGENT_ASSIGNED: &'static str = "agent.assigned";
    pub const AGENT_COMPLETED: &'static str = "agent.completed";
    pub const AGENT_FAILED: &'static str = "agent.failed";
    pub const AGENT_MESSAGE: &'static str = "agent.message";

    // Self-Improvement
    pub const IMPROVEMENT_INSIGHT: &'static str = "improvement.insight";
    pub const IMPROVEMENT_CORRECTION: &'static str = "improvement.correction";

    // System
    pub const SYSTEM_HEALTH: &'static str = "system.health";
    pub const SYSTEM_METRICS: &'static str = "system.metrics";
    pub const SYSTEM_ALERT: &'static str = "system.alert";
    pub const SYSTEM_ERROR: &'static str = "system.error";

    // Automation
    pub const AUTOMATION_TRIGGER: &'static str = "automation.trigger";
    pub const AUTOMATION_COMPLETE: &'static str = "automation.complete";

    // World Model
    pub const WORLD_CONTEXT: &'static str = "world.context";
    pub const WORLD_DESKTOP: &'static str = "world.desktop";

    // Conversation
    pub const CONVERSATION_TURN: &'static str = "conversation.turn";
    pub const CONVERSATION_CONTEXT: &'static str = "conversation.context";
}

// ============================================================================
// Typed Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceWakeEvent {
    pub confidence: f32,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttFinalEvent {
    pub text: String,
    pub language: String,
    pub confidence: f32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponseEvent {
    pub text: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedEvent {
    pub task_id: String,
    pub result: String,
    pub duration_ms: u64,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStoredEvent {
    pub memory_id: String,
    pub memory_type: String,
    pub importance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCompletedEvent {
    pub goal_id: String,
    pub title: String,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthEvent {
    pub component: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResultEvent {
    pub action: String,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompletedEvent {
    pub agent_id: String,
    pub task_id: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStepDoneEvent {
    pub plan_id: String,
    pub step_id: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementInsightEvent {
    pub insight_id: String,
    pub category: String,
    pub description: String,
    pub impact: f32,
}

// ============================================================================
// Event Bridge
// ============================================================================

/// Bridges subsystems through the EventBus.
///
/// Provides typed helpers for publishing and subscribing to standard events.
pub struct EventBridge {
    bus: Arc<EventBus>,
    /// Topic -> subscriber count for monitoring.
    subscriber_counts: RwLock<HashMap<String, usize>>,
}

impl EventBridge {
    /// Create a new event bridge.
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self {
            bus,
            subscriber_counts: RwLock::new(HashMap::new()),
        }
    }

    /// Get the underlying event bus.
    pub fn bus(&self) -> Arc<EventBus> {
        self.bus.clone()
    }

    /// Publish a typed event to a topic.
    pub async fn publish<T: Serialize>(&self, topic: &str, source: &str, data: &T) -> Result<()> {
        let event = Event::from_json(topic, source, data)
            .map_err(|e| IntegrationError::EventBridge(e.to_string()))?;
        self.bus
            .publish(topic, event)
            .await
            .map_err(|e| IntegrationError::EventBridge(e.to_string()))
    }

    /// Publish a raw event.
    pub async fn publish_raw(&self, event: Event) -> Result<()> {
        let topic = event.topic().to_string();
        self.bus
            .publish(&topic, event)
            .await
            .map_err(|e| IntegrationError::EventBridge(e.to_string()))
    }

    /// Subscribe to a topic.
    pub async fn subscribe(&self, topic: &str) -> Result<tokio::sync::broadcast::Receiver<Event>> {
        let rx = self
            .bus
            .subscribe(topic)
            .await
            .map_err(|e| IntegrationError::EventBridge(e.to_string()))?;

        let mut counts = self.subscriber_counts.write();
        *counts.entry(topic.to_string()).or_insert(0) += 1;

        Ok(rx)
    }

    /// Get subscriber count for a topic.
    pub fn subscriber_count(&self, topic: &str) -> usize {
        self.subscriber_counts
            .read()
            .get(topic)
            .copied()
            .unwrap_or(0)
    }

    /// Get all tracked topics.
    pub fn tracked_topics(&self) -> Vec<String> {
        self.subscriber_counts.read().keys().cloned().collect()
    }

    // Convenience publishers

    pub async fn publish_voice_wake(&self, confidence: f32, timestamp_ms: u64) -> Result<()> {
        self.publish(
            Topics::VOICE_WAKE,
            "voice",
            &VoiceWakeEvent {
                confidence,
                timestamp_ms,
            },
        )
        .await
    }

    pub async fn publish_stt_final(
        &self,
        text: &str,
        language: &str,
        confidence: f32,
        duration_ms: u64,
    ) -> Result<()> {
        self.publish(
            Topics::STT_FINAL,
            "stt",
            &SttFinalEvent {
                text: text.to_string(),
                language: language.to_string(),
                confidence,
                duration_ms,
            },
        )
        .await
    }

    pub async fn publish_llm_response(
        &self,
        text: &str,
        tokens_used: u32,
        latency_ms: u64,
        model: &str,
    ) -> Result<()> {
        self.publish(
            Topics::LLM_RESPONSE,
            "llm",
            &LlmResponseEvent {
                text: text.to_string(),
                tokens_used,
                latency_ms,
                model: model.to_string(),
            },
        )
        .await
    }

    pub async fn publish_task_completed(
        &self,
        task_id: &str,
        result: &str,
        duration_ms: u64,
        agent_id: Option<String>,
    ) -> Result<()> {
        self.publish(
            Topics::TASK_COMPLETED,
            "task_graph",
            &TaskCompletedEvent {
                task_id: task_id.to_string(),
                result: result.to_string(),
                duration_ms,
                agent_id,
            },
        )
        .await
    }

    pub async fn publish_memory_stored(
        &self,
        memory_id: &str,
        memory_type: &str,
        importance: f64,
    ) -> Result<()> {
        self.publish(
            Topics::MEMORY_STORED,
            "memory",
            &MemoryStoredEvent {
                memory_id: memory_id.to_string(),
                memory_type: memory_type.to_string(),
                importance,
            },
        )
        .await
    }

    pub async fn publish_goal_completed(
        &self,
        goal_id: &str,
        title: &str,
        progress: f32,
    ) -> Result<()> {
        self.publish(
            Topics::GOAL_COMPLETED,
            "goal_engine",
            &GoalCompletedEvent {
                goal_id: goal_id.to_string(),
                title: title.to_string(),
                progress,
            },
        )
        .await
    }

    pub async fn publish_system_health(
        &self,
        component: &str,
        status: &str,
        message: Option<String>,
    ) -> Result<()> {
        self.publish(
            Topics::SYSTEM_HEALTH,
            "system",
            &SystemHealthEvent {
                component: component.to_string(),
                status: status.to_string(),
                message,
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_are_distinct() {
        let mut topics = vec![
            Topics::VOICE_WAKE,
            Topics::VOICE_INPUT,
            Topics::STT_FINAL,
            Topics::LLM_REQUEST,
            Topics::LLM_RESPONSE,
            Topics::TTS_REQUEST,
            Topics::PLAN_CREATE,
            Topics::TASK_CREATED,
            Topics::TASK_COMPLETED,
            Topics::DECISION_REQUEST,
            Topics::DECISION_RESULT,
            Topics::MEMORY_STORED,
            Topics::MEMORY_RETRIEVED,
            Topics::GOAL_CREATED,
            Topics::GOAL_COMPLETED,
            Topics::PROJECT_UPDATED,
            Topics::AGENT_ASSIGNED,
            Topics::AGENT_COMPLETED,
            Topics::IMPROVEMENT_INSIGHT,
            Topics::SYSTEM_HEALTH,
            Topics::SYSTEM_METRICS,
            Topics::AUTOMATION_TRIGGER,
            Topics::WORLD_CONTEXT,
            Topics::CONVERSATION_TURN,
        ];
        topics.sort();
        topics.dedup();
        assert_eq!(topics.len(), 24, "All topics must be unique");
    }

    #[test]
    fn event_bridge_creation() {
        let bus = Arc::new(EventBus::new(64));
        let bridge = EventBridge::new(bus);
        assert_eq!(bridge.tracked_topics().len(), 0);
    }

    #[test]
    fn typed_events_serialize() {
        let wake = VoiceWakeEvent {
            confidence: 0.95,
            timestamp_ms: 12345,
        };
        let json = serde_json::to_string(&wake).unwrap();
        assert!(json.contains("0.95"));

        let task = TaskCompletedEvent {
            task_id: "t1".to_string(),
            result: "done".to_string(),
            duration_ms: 500,
            agent_id: None,
        };
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("t1"));
    }

    #[tokio::test]
    async fn publish_and_subscribe() {
        let bus = Arc::new(EventBus::new(64));
        let bridge = EventBridge::new(bus);

        let mut rx = bridge.subscribe(Topics::VOICE_WAKE).await.unwrap();

        bridge.publish_voice_wake(0.9, 100).await.unwrap();

        let event = rx.recv().await.unwrap();
        assert_eq!(event.topic(), Topics::VOICE_WAKE);
    }

    #[tokio::test]
    async fn subscriber_count_tracking() {
        let bus = Arc::new(EventBus::new(64));
        let bridge = EventBridge::new(bus);

        let _rx1 = bridge.subscribe(Topics::STT_FINAL).await.unwrap();
        let _rx2 = bridge.subscribe(Topics::STT_FINAL).await.unwrap();

        assert_eq!(bridge.subscriber_count(Topics::STT_FINAL), 2);
        assert_eq!(bridge.subscriber_count(Topics::VOICE_WAKE), 0);
    }
}
