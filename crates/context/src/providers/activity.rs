use crate::error::Result;
use crate::provider::ContextProvider;
use crate::types::{ContextPriority, ContextSnapshot, ContextSource};
use async_trait::async_trait;

/// User activity type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityType {
    Idle,
    Active,
    AppInteraction { app_name: String },
    Listening,
    Reading,
}

impl serde::Serialize for ActivityType {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        match self {
            ActivityType::Idle => {
                let mut s = serializer.serialize_struct("ActivityType", 1)?;
                s.serialize_field("type", "idle")?;
                s.end()
            }
            ActivityType::Active => {
                let mut s = serializer.serialize_struct("ActivityType", 1)?;
                s.serialize_field("type", "active")?;
                s.end()
            }
            ActivityType::AppInteraction { app_name } => {
                let mut s = serializer.serialize_struct("ActivityType", 2)?;
                s.serialize_field("type", "app_interaction")?;
                s.serialize_field("app_name", app_name)?;
                s.end()
            }
            ActivityType::Listening => {
                let mut s = serializer.serialize_struct("ActivityType", 1)?;
                s.serialize_field("type", "listening")?;
                s.end()
            }
            ActivityType::Reading => {
                let mut s = serializer.serialize_struct("ActivityType", 1)?;
                s.serialize_field("type", "reading")?;
                s.end()
            }
        }
    }
}

/// Provides activity context (current user activity, focus state).
pub struct ActivityContextProvider {
    current_activity: ActivityType,
    idle_duration_secs: u64,
    focus_app: Option<String>,
}

impl ActivityContextProvider {
    pub fn new() -> Self {
        Self {
            current_activity: ActivityType::Idle,
            idle_duration_secs: 0,
            focus_app: None,
        }
    }

    pub fn with_activity(activity: ActivityType) -> Self {
        Self {
            current_activity: activity,
            idle_duration_secs: 0,
            focus_app: None,
        }
    }

    pub fn set_activity(&mut self, activity: ActivityType) {
        self.current_activity = activity;
    }

    pub fn set_idle_duration(&mut self, secs: u64) {
        self.idle_duration_secs = secs;
    }

    pub fn set_focus_app(&mut self, app: Option<String>) {
        self.focus_app = app;
    }
}

impl Default for ActivityContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextProvider for ActivityContextProvider {
    fn name(&self) -> &str {
        "activity"
    }

    fn source(&self) -> ContextSource {
        ContextSource::Activity
    }

    fn default_priority(&self) -> ContextPriority {
        ContextPriority::Medium
    }

    async fn collect(&self) -> Result<ContextSnapshot> {
        let data = serde_json::json!({
            "activity": self.current_activity,
            "idle_duration_secs": self.idle_duration_secs,
            "focus_app": self.focus_app,
        });

        Ok(ContextSnapshot::new(ContextSource::Activity, data))
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_idle() {
        let provider = ActivityContextProvider::with_activity(ActivityType::Idle);
        let snapshot = provider.collect().await.unwrap();
        assert_eq!(snapshot.source, ContextSource::Activity);
        assert_eq!(snapshot.data["activity"]["type"], "idle");
    }

    #[tokio::test]
    async fn test_collect_app_interaction() {
        let provider = ActivityContextProvider::with_activity(ActivityType::AppInteraction {
            app_name: "vscode".to_string(),
        });
        let snapshot = provider.collect().await.unwrap();
        assert_eq!(snapshot.data["activity"]["type"], "app_interaction");
        assert_eq!(snapshot.data["activity"]["app_name"], "vscode");
    }

    #[test]
    fn test_setters() {
        let mut provider = ActivityContextProvider::new();
        provider.set_activity(ActivityType::Active);
        provider.set_idle_duration(30);
        provider.set_focus_app(Some("terminal".to_string()));

        assert_eq!(provider.idle_duration_secs, 30);
        assert_eq!(provider.focus_app.as_deref(), Some("terminal"));
    }
}
