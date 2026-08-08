use std::collections::HashMap;

use async_trait::async_trait;

use crate::capabilities::{
    CapabilityDescriptor, CapabilityId, CapabilityRegistry, InMemoryCapabilityRegistry,
};
use crate::config::SkillsConfig;
use crate::error::SkillsError;
use crate::event::SkillsEvent;
use crate::traits::SkillContext;
use crate::types::{InvocationId, SkillId};

#[test]
fn test_skill_id_creation() {
    let id = SkillId("skill-1".into());
    assert_eq!(id.as_str(), "skill-1");
    assert_eq!(id.to_string(), "skill-1");
    assert_eq!(id, SkillId("skill-1".into()));
}

#[test]
fn test_invocation_id_creation() {
    let id = InvocationId("inv-42".into());
    assert_eq!(id.as_str(), "inv-42");
    assert_eq!(id.to_string(), "inv-42");
}

#[test]
fn test_capability_id_creation() {
    let id = CapabilityId("cap-1".into());
    assert_eq!(id.as_str(), "cap-1");
    assert_eq!(id.to_string(), "cap-1");
}

#[test]
fn test_capability_descriptor_creation() {
    let desc = CapabilityDescriptor {
        id: CapabilityId("tts".into()),
        name: "Text-to-Speech".into(),
        description: "Convert text to spoken audio".into(),
        version: "1.0.0".into(),
        provider_hint: Some("kokoro".into()),
    };
    assert_eq!(desc.id.to_string(), "tts");
    assert_eq!(desc.name, "Text-to-Speech");
    assert_eq!(desc.version, "1.0.0");
    assert_eq!(desc.provider_hint, Some("kokoro".into()));
}

#[test]
fn test_skills_config_default() {
    let config = SkillsConfig::default();
    assert_eq!(config.max_concurrent_skills, 10);
    assert_eq!(config.skill_timeout_seconds, 30);
    assert!(config.enable_capability_discovery);
    assert!(!config.enable_skill_caching);
}

#[test]
fn test_skills_error_display() {
    let err = SkillsError::InvalidConfig("bad config".into());
    assert_eq!(err.to_string(), "Invalid configuration: bad config");

    let err = SkillsError::SkillNotFound("skill_1".into());
    assert_eq!(err.to_string(), "Skill not found: skill_1");

    let err = SkillsError::SkillExecutionFailed("timeout".into());
    assert_eq!(err.to_string(), "Skill execution failed: timeout");

    let err = SkillsError::CapabilityNotFound("cap_1".into());
    assert_eq!(err.to_string(), "Capability not found: cap_1");

    let err = SkillsError::InvalidInput("missing param".into());
    assert_eq!(err.to_string(), "Invalid input: missing param");

    let err = SkillsError::Timeout("skill_1".into());
    assert_eq!(err.to_string(), "Skill execution timed out: skill_1");

    let err = SkillsError::ExecutionCancelled("user abort".into());
    assert_eq!(err.to_string(), "Skill execution cancelled: user abort");
}

#[test]
fn test_skills_event_display() {
    let event = SkillsEvent::SkillRegistered {
        skill_id: "s1".into(),
        skill_name: "Weather".into(),
    };
    assert_eq!(event.to_string(), "Skill registered: s1 (Weather)");

    let event = SkillsEvent::SkillUnregistered {
        skill_id: "s1".into(),
    };
    assert_eq!(event.to_string(), "Skill unregistered: s1");

    let event = SkillsEvent::SkillExecutionStarted {
        skill_id: "s1".into(),
        invocation_id: "inv-1".into(),
    };
    assert_eq!(event.to_string(), "Skill execution started: s1 (inv-1)");

    let event = SkillsEvent::SkillExecutionCompleted {
        skill_id: "s1".into(),
        invocation_id: "inv-1".into(),
        duration_ms: 150,
    };
    assert_eq!(
        event.to_string(),
        "Skill execution completed: s1 (inv-1) in 150ms"
    );

    let event = SkillsEvent::SkillExecutionFailed {
        skill_id: "s1".into(),
        invocation_id: "inv-1".into(),
        error: "crash".into(),
    };
    assert_eq!(
        event.to_string(),
        "Skill execution failed: s1 (inv-1) - crash"
    );

    let event = SkillsEvent::CapabilityDiscovered {
        capability_id: "tts".into(),
        description: "Text-to-Speech".into(),
    };
    assert_eq!(
        event.to_string(),
        "Capability discovered: tts - Text-to-Speech"
    );
}

#[test]
fn test_skill_input_creation() {
    struct TestContext {
        config: SkillsConfig,
    }

    #[async_trait]
    impl SkillContext for TestContext {
        fn world_model(&self) -> Option<&dyn voxy_world_model::traits::WorldModelProvider> {
            None
        }
        fn personality(&self) -> Option<&dyn voxy_personality::traits::PersonalityProfile> {
            None
        }
        fn config(&self) -> &SkillsConfig {
            &self.config
        }
    }

    let ctx = TestContext {
        config: SkillsConfig::default(),
    };
    let mut parameters = HashMap::new();
    parameters.insert("query".into(), serde_json::Value::String("hello".into()));

    let input = crate::traits::SkillInput {
        parameters,
        context: Box::new(ctx),
    };

    assert_eq!(
        input.parameters.get("query"),
        Some(&serde_json::Value::String("hello".into()))
    );
}

#[test]
fn test_skill_output_creation() {
    let output = crate::traits::SkillOutput {
        result: serde_json::json!({"response": "hello world"}),
        confidence: Some(0.95),
        duration_ms: 42,
    };
    assert_eq!(output.result["response"], "hello world");
    assert_eq!(output.confidence, Some(0.95));
    assert_eq!(output.duration_ms, 42);
}

#[tokio::test]
async fn test_registry_register_and_has() {
    let registry = InMemoryCapabilityRegistry::new();
    let cap = CapabilityDescriptor {
        id: CapabilityId("tts".into()),
        name: "TTS".into(),
        description: "Text to speech".into(),
        version: "1.0".into(),
        provider_hint: None,
    };
    registry.register_capability(cap).await.unwrap();
    assert!(registry.has_capability(&CapabilityId("tts".into())).await);
    assert!(
        !registry
            .has_capability(&CapabilityId("nonexistent".into()))
            .await
    );
}

#[tokio::test]
async fn test_registry_list() {
    let registry = InMemoryCapabilityRegistry::new();
    registry
        .register_capability(CapabilityDescriptor {
            id: CapabilityId("llm".into()),
            name: "LLM".into(),
            description: "Language model".into(),
            version: "1.0".into(),
            provider_hint: None,
        })
        .await
        .unwrap();
    registry
        .register_capability(CapabilityDescriptor {
            id: CapabilityId("stt".into()),
            name: "STT".into(),
            description: "Speech to text".into(),
            version: "2.0".into(),
            provider_hint: Some("whisper".into()),
        })
        .await
        .unwrap();

    let list = registry.list_capabilities().await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn test_registry_unregister() {
    let registry = InMemoryCapabilityRegistry::new();
    let cap_id = CapabilityId("tts".into());
    registry
        .register_capability(CapabilityDescriptor {
            id: cap_id.clone(),
            name: "TTS".into(),
            description: "Text to speech".into(),
            version: "1.0".into(),
            provider_hint: None,
        })
        .await
        .unwrap();
    assert!(registry.has_capability(&cap_id).await);
    registry.unregister_capability(&cap_id).await.unwrap();
    assert!(!registry.has_capability(&cap_id).await);
}

#[tokio::test]
async fn test_registry_find() {
    let registry = InMemoryCapabilityRegistry::new();
    registry
        .register_capability(CapabilityDescriptor {
            id: CapabilityId("llm".into()),
            name: "Large Language Model".into(),
            description: "Advanced text generation".into(),
            version: "1.0".into(),
            provider_hint: None,
        })
        .await
        .unwrap();
    registry
        .register_capability(CapabilityDescriptor {
            id: CapabilityId("stt".into()),
            name: "Speech Recognition".into(),
            description: "Convert audio to text".into(),
            version: "2.0".into(),
            provider_hint: Some("whisper".into()),
        })
        .await
        .unwrap();

    let results = registry.find_capabilities("speech").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id.to_string(), "stt");

    let results = registry.find_capabilities("language").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id.to_string(), "llm");

    let results = registry.find_capabilities("nonexistent").await.unwrap();
    assert!(results.is_empty());
}
