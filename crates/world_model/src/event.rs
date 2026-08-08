use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone)]
pub enum WorldModelEvent {
    DesktopUpdated {
        window_count: usize,
        focused_app: Option<String>,
    },
    ApplicationLaunched {
        app_id: String,
        app_name: String,
    },
    ApplicationClosed {
        app_id: String,
    },
    DeviceConnected {
        device_id: String,
        device_type: String,
    },
    DeviceDisconnected {
        device_id: String,
    },
    TaskCreated {
        task_id: String,
        description: String,
    },
    TaskUpdated {
        task_id: String,
        status: String,
    },
    TaskCompleted {
        task_id: String,
    },
    EnvironmentChanged {
        description: String,
    },
    WindowChanged {
        app_id: String,
        window_title: String,
        timestamp: DateTime<Utc>,
    },
    ActivityChanged {
        app_id: String,
        activity_type: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    ProjectDetected {
        project_name: String,
        project_path: Option<String>,
        language: Option<String>,
        timestamp: DateTime<Utc>,
    },
    PreferenceLearned {
        category: String,
        key: String,
        value: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    ContextUpdated {
        focused_app: Option<String>,
        activity_type: Option<String>,
        window_title: Option<String>,
        timestamp: DateTime<Utc>,
    },
    ApplicationFocused {
        app_id: String,
        app_name: String,
        timestamp: DateTime<Utc>,
    },
    IdleStarted {
        last_active_app: Option<String>,
        timestamp: DateTime<Utc>,
    },
    IdleEnded {
        new_app: String,
        timestamp: DateTime<Utc>,
    },
}

impl fmt::Display for WorldModelEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DesktopUpdated {
                window_count,
                focused_app,
            } => {
                write!(
                    f,
                    "Desktop updated: {} windows, focused app: {:?}",
                    window_count, focused_app
                )
            }
            Self::ApplicationLaunched { app_id, app_name } => {
                write!(f, "Application launched: {} ({})", app_name, app_id)
            }
            Self::ApplicationClosed { app_id } => {
                write!(f, "Application closed: {}", app_id)
            }
            Self::DeviceConnected {
                device_id,
                device_type,
            } => {
                write!(f, "Device connected: {} ({})", device_id, device_type)
            }
            Self::DeviceDisconnected { device_id } => {
                write!(f, "Device disconnected: {}", device_id)
            }
            Self::TaskCreated {
                task_id,
                description,
            } => {
                write!(f, "Task created: {} - {}", task_id, description)
            }
            Self::TaskUpdated { task_id, status } => {
                write!(f, "Task updated: {} -> {}", task_id, status)
            }
            Self::TaskCompleted { task_id } => {
                write!(f, "Task completed: {}", task_id)
            }
            Self::EnvironmentChanged { description } => {
                write!(f, "Environment changed: {}", description)
            }
            Self::WindowChanged {
                app_id,
                window_title,
                ..
            } => {
                write!(f, "Window changed: {} -> {}", app_id, window_title)
            }
            Self::ActivityChanged {
                app_id,
                activity_type,
                confidence,
                ..
            } => {
                write!(
                    f,
                    "Activity changed: {} -> {} ({:.0}%)",
                    app_id,
                    activity_type,
                    confidence * 100.0
                )
            }
            Self::ProjectDetected {
                project_name,
                language,
                ..
            } => {
                write!(
                    f,
                    "Project detected: {} (lang: {:?})",
                    project_name, language
                )
            }
            Self::PreferenceLearned {
                category,
                key,
                value,
                ..
            } => {
                write!(f, "Preference learned: {}.{} = {}", category, key, value)
            }
            Self::ContextUpdated {
                focused_app,
                activity_type,
                ..
            } => {
                write!(
                    f,
                    "Context updated: app={:?}, activity={:?}",
                    focused_app, activity_type
                )
            }
            Self::ApplicationFocused {
                app_id, app_name, ..
            } => {
                write!(f, "Application focused: {} ({})", app_name, app_id)
            }
            Self::IdleStarted {
                last_active_app, ..
            } => {
                write!(f, "Idle started: last app was {:?}", last_active_app)
            }
            Self::IdleEnded { new_app, .. } => {
                write!(f, "Idle ended: new app is {}", new_app)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_model_event_display() {
        let event = WorldModelEvent::DesktopUpdated {
            window_count: 5,
            focused_app: Some("Code".to_string()),
        };
        let s = format!("{}", event);
        assert!(s.contains("Desktop updated"));
        assert!(s.contains("5"));

        let event = WorldModelEvent::ApplicationLaunched {
            app_id: "code1".to_string(),
            app_name: "VS Code".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Application launched"));
        assert!(s.contains("VS Code"));

        let event = WorldModelEvent::ApplicationClosed {
            app_id: "code1".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Application closed"));

        let event = WorldModelEvent::DeviceConnected {
            device_id: "cam1".to_string(),
            device_type: "Camera".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Device connected"));

        let event = WorldModelEvent::DeviceDisconnected {
            device_id: "cam1".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Device disconnected"));

        let event = WorldModelEvent::TaskCreated {
            task_id: "t1".to_string(),
            description: "Test task".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Task created"));

        let event = WorldModelEvent::TaskUpdated {
            task_id: "t1".to_string(),
            status: "InProgress".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Task updated"));

        let event = WorldModelEvent::TaskCompleted {
            task_id: "t1".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Task completed"));

        let event = WorldModelEvent::EnvironmentChanged {
            description: "moved to office".to_string(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Environment changed"));
    }

    #[test]
    fn test_new_event_types_display() {
        let event = WorldModelEvent::WindowChanged {
            app_id: "code.exe".to_string(),
            window_title: "main.rs - VS Code".to_string(),
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Window changed"));
        assert!(s.contains("main.rs"));

        let event = WorldModelEvent::ActivityChanged {
            app_id: "code.exe".to_string(),
            activity_type: "Coding".to_string(),
            confidence: 0.9,
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Activity changed"));
        assert!(s.contains("Coding"));
        assert!(s.contains("90%"));

        let event = WorldModelEvent::ProjectDetected {
            project_name: "voxy".to_string(),
            project_path: Some("/projects/voxy".to_string()),
            language: Some("Rust".to_string()),
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Project detected"));
        assert!(s.contains("voxy"));

        let event = WorldModelEvent::PreferenceLearned {
            category: "app_usage".to_string(),
            key: "code.exe".to_string(),
            value: "500".to_string(),
            confidence: 0.8,
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Preference learned"));
        assert!(s.contains("code.exe"));

        let event = WorldModelEvent::ContextUpdated {
            focused_app: Some("code.exe".to_string()),
            activity_type: Some("Coding".to_string()),
            window_title: Some("main.rs".to_string()),
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Context updated"));

        let event = WorldModelEvent::ApplicationFocused {
            app_id: "code.exe".to_string(),
            app_name: "VS Code".to_string(),
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Application focused"));
        assert!(s.contains("VS Code"));

        let event = WorldModelEvent::IdleStarted {
            last_active_app: Some("chrome.exe".to_string()),
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Idle started"));

        let event = WorldModelEvent::IdleEnded {
            new_app: "code.exe".to_string(),
            timestamp: Utc::now(),
        };
        let s = format!("{}", event);
        assert!(s.contains("Idle ended"));
    }
}
