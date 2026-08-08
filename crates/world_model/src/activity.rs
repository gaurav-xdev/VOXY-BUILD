use crate::desktop::DesktopState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityType {
    Coding,
    Browsing,
    Communication,
    Media,
    Productivity,
    Gaming,
    System,
    Idle,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityClassification {
    pub activity_type: ActivityType,
    pub confidence: f64,
    pub app_name: String,
    pub details: String,
}

pub struct ActivityClassifier {
    app_categories: HashMap<String, ActivityType>,
    title_patterns: Vec<(String, ActivityType)>,
}

impl ActivityClassifier {
    pub fn new() -> Self {
        let mut app_categories = HashMap::new();

        // Coding
        for app in &[
            "code.exe",
            "devenv.exe",
            "rust-analyzer.exe",
            "clion.exe",
            "idea64.exe",
            "pycharm64.exe",
            "webstorm64.exe",
            "sublime_text.exe",
            "notepad++.exe",
        ] {
            app_categories.insert(app.to_lowercase(), ActivityType::Coding);
        }

        // Browsing
        for app in &[
            "chrome.exe",
            "firefox.exe",
            "msedge.exe",
            "brave.exe",
            "opera.exe",
        ] {
            app_categories.insert(app.to_lowercase(), ActivityType::Browsing);
        }

        // Communication
        for app in &[
            "slack.exe",
            "discord.exe",
            "teams.exe",
            "zoom.exe",
            "telegram.exe",
            "whatsapp.exe",
            "skype.exe",
        ] {
            app_categories.insert(app.to_lowercase(), ActivityType::Communication);
        }

        // Media
        for app in &[
            "spotify.exe",
            "vlc.exe",
            "mpv.exe",
            "foobar2000.exe",
            "AIMP.exe",
        ] {
            app_categories.insert(app.to_lowercase(), ActivityType::Media);
        }

        // Productivity
        for app in &[
            "notepad.exe",
            "wordpad.exe",
            "winword.exe",
            "excel.exe",
            "powerpnt.exe",
            "outlook.exe",
            "explorer.exe",
        ] {
            app_categories.insert(app.to_lowercase(), ActivityType::Productivity);
        }

        // Gaming
        for app in &[
            "steam.exe",
            "epicgameslauncher.exe",
            "battle.net.exe",
            "origin.exe",
        ] {
            app_categories.insert(app.to_lowercase(), ActivityType::Gaming);
        }

        let title_patterns = vec![
            ("inbox".to_string(), ActivityType::Communication),
            ("mail".to_string(), ActivityType::Communication),
            ("chat".to_string(), ActivityType::Communication),
            ("call".to_string(), ActivityType::Communication),
            ("meeting".to_string(), ActivityType::Communication),
            ("youtube".to_string(), ActivityType::Media),
            ("music".to_string(), ActivityType::Media),
            ("spotify".to_string(), ActivityType::Media),
            ("docs.google".to_string(), ActivityType::Productivity),
            ("github.com".to_string(), ActivityType::Coding),
            ("stackoverflow".to_string(), ActivityType::Coding),
        ];

        Self {
            app_categories,
            title_patterns,
        }
    }

    pub fn classify(&self, state: &DesktopState) -> ActivityClassification {
        if let Some(ref focused_app) = state.focused_app {
            let app_lower = focused_app.to_lowercase();

            // Check app-based classification
            if let Some(&activity_type) = self.app_categories.get(&app_lower) {
                return ActivityClassification {
                    activity_type,
                    confidence: 0.9,
                    app_name: focused_app.clone(),
                    details: format!("App-based classification: {}", app_lower),
                };
            }

            // Check title-based classification
            if let Some(window) = state.windows.iter().find(|w| w.is_focused) {
                let title_lower = window.title.to_lowercase();
                for (pattern, activity_type) in &self.title_patterns {
                    if title_lower.contains(pattern) {
                        return ActivityClassification {
                            activity_type: *activity_type,
                            confidence: 0.7,
                            app_name: focused_app.clone(),
                            details: format!("Title pattern match: {}", pattern),
                        };
                    }
                }
            }

            // If app is running, classify as unknown with lower confidence
            ActivityClassification {
                activity_type: ActivityType::Unknown,
                confidence: 0.3,
                app_name: focused_app.clone(),
                details: format!("No classification for app: {}", focused_app),
            }
        } else {
            ActivityClassification {
                activity_type: ActivityType::Idle,
                confidence: 0.8,
                app_name: String::new(),
                details: "No focused application".to_string(),
            }
        }
    }
}

impl Default for ActivityClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::WindowInfo;

    #[test]
    fn test_classifier_creation() {
        let _classifier = ActivityClassifier::new();
    }

    #[test]
    fn test_classify_idle() {
        let classifier = ActivityClassifier::new();
        let state = DesktopState {
            windows: vec![],
            active_window_id: None,
            workspaces: vec![],
            focused_app: None,
        };
        let result = classifier.classify(&state);
        assert_eq!(result.activity_type, ActivityType::Idle);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_classify_coding() {
        let classifier = ActivityClassifier::new();
        let state = DesktopState {
            windows: vec![WindowInfo {
                id: "w1".to_string(),
                title: "main.rs - VS Code".to_string(),
                application_id: "code.exe".to_string(),
                application_name: "VS Code".to_string(),
                bounds: None,
                is_focused: true,
                is_minimized: false,
                process_id: Some(1234),
            }],
            active_window_id: Some("w1".to_string()),
            workspaces: vec![],
            focused_app: Some("code.exe".to_string()),
        };
        let result = classifier.classify(&state);
        assert_eq!(result.activity_type, ActivityType::Coding);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_classify_browsing() {
        let classifier = ActivityClassifier::new();
        let state = DesktopState {
            windows: vec![WindowInfo {
                id: "w1".to_string(),
                title: "Google - Chrome".to_string(),
                application_id: "chrome.exe".to_string(),
                application_name: "Chrome".to_string(),
                bounds: None,
                is_focused: true,
                is_minimized: false,
                process_id: Some(1234),
            }],
            active_window_id: Some("w1".to_string()),
            workspaces: vec![],
            focused_app: Some("chrome.exe".to_string()),
        };
        let result = classifier.classify(&state);
        assert_eq!(result.activity_type, ActivityType::Browsing);
        assert!(result.confidence >= 0.8);
    }
}
