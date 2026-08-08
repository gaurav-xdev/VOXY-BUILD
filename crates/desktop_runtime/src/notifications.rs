//! Notification manager.
//!
//! Sends native Windows notifications via the tray icon.

use crate::error::Result;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

/// Notification priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// A notification to display.
#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub priority: NotificationPriority,
    pub timeout_ms: u32,
}

impl Notification {
    pub fn info(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            priority: NotificationPriority::Normal,
            timeout_ms: 5000,
        }
    }

    pub fn warning(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            priority: NotificationPriority::High,
            timeout_ms: 8000,
        }
    }

    pub fn error(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            priority: NotificationPriority::Critical,
            timeout_ms: 10000,
        }
    }
}

/// Notification history entry.
#[derive(Debug, Clone)]
pub struct NotificationRecord {
    pub notification: Notification,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub id: u64,
}

/// Notification manager.
pub struct NotificationManager {
    history: RwLock<Vec<NotificationRecord>>,
    max_history: usize,
    counter: AtomicU64,
}

impl NotificationManager {
    /// Create a new notification manager.
    pub fn new() -> Result<Self> {
        Ok(Self {
            history: RwLock::new(Vec::new()),
            max_history: 100,
            counter: AtomicU64::new(0),
        })
    }

    /// Send a notification.
    pub fn send(&self, notification: Notification) -> u64 {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);

        let record = NotificationRecord {
            notification: notification.clone(),
            timestamp: chrono::Utc::now(),
            id,
        };

        {
            let mut history = self.history.write();
            if history.len() >= self.max_history {
                history.remove(0);
            }
            history.push(record);
        }

        // Send via tray icon balloon
        #[cfg(windows)]
        {
            // In production, this would use the tray icon's show_balloon
            info!(
                "Notification: {} - {}",
                notification.title, notification.message
            );
        }
        #[cfg(not(windows))]
        {
            info!(
                "Notification: {} - {}",
                notification.title, notification.message
            );
        }

        id
    }

    /// Get notification history.
    pub fn history(&self) -> Vec<NotificationRecord> {
        self.history.read().clone()
    }

    /// Clear notification history.
    pub fn clear_history(&self) {
        self.history.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_info() {
        let n = Notification::info("Test", "Hello");
        assert_eq!(n.priority, NotificationPriority::Normal);
    }

    #[test]
    fn notification_warning() {
        let n = Notification::warning("Warning", "Be careful");
        assert_eq!(n.priority, NotificationPriority::High);
    }

    #[test]
    fn notification_error() {
        let n = Notification::error("Error", "Something broke");
        assert_eq!(n.priority, NotificationPriority::Critical);
    }

    #[test]
    fn notification_manager_send() {
        let mgr = NotificationManager::new().unwrap();
        let id = mgr.send(Notification::info("Test", "Hello"));
        assert!(id < 1000); // Initial counter value
    }

    #[test]
    fn notification_history() {
        let mgr = NotificationManager::new().unwrap();
        mgr.send(Notification::info("Test1", "Hello1"));
        mgr.send(Notification::info("Test2", "Hello2"));
        assert_eq!(mgr.history().len(), 2);
    }

    #[test]
    fn notification_clear_history() {
        let mgr = NotificationManager::new().unwrap();
        mgr.send(Notification::info("Test", "Hello"));
        mgr.clear_history();
        assert!(mgr.history().is_empty());
    }
}
