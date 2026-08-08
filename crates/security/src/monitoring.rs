use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Security event severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityEventSeverity {
    Info,
    Warning,
    Critical,
}

/// A forensics event for admin monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicsEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub severity: SecurityEventSeverity,
    pub source: String,
    pub description: String,
    pub details: std::collections::HashMap<String, String>,
    pub mitigated: bool,
}

/// Live threat feed providing real-time security visibility.
pub struct SecurityMonitor {
    events: Vec<ForensicsEvent>,
    max_events: usize,
    alert_threshold: SecurityEventSeverity,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_events: 1000,
            alert_threshold: SecurityEventSeverity::Warning,
        }
    }

    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    pub fn with_alert_threshold(mut self, threshold: SecurityEventSeverity) -> Self {
        self.alert_threshold = threshold;
        self
    }

    pub fn record_event(
        &mut self,
        category: &str,
        severity: SecurityEventSeverity,
        source: &str,
        description: &str,
    ) -> Uuid {
        self.record_event_with_details(
            category,
            severity,
            source,
            description,
            std::collections::HashMap::new(),
        )
    }

    pub fn record_event_with_details(
        &mut self,
        category: &str,
        severity: SecurityEventSeverity,
        source: &str,
        description: &str,
        details: std::collections::HashMap<String, String>,
    ) -> Uuid {
        let event = ForensicsEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: category.to_string(),
            severity,
            source: source.to_string(),
            description: description.to_string(),
            details,
            mitigated: false,
        };
        let id = event.id;
        self.events.push(event);

        // Evict oldest if over limit
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }

        id
    }

    pub fn mark_mitigated(&mut self, id: Uuid) -> bool {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.mitigated = true;
            true
        } else {
            false
        }
    }

    /// Get events at or above the alert threshold.
    pub fn alerts(&self) -> Vec<&ForensicsEvent> {
        self.events
            .iter()
            .filter(|e| e.severity >= self.alert_threshold)
            .collect()
    }

    /// Get recent unmitigated critical events.
    pub fn critical_unmitigated(&self) -> Vec<&ForensicsEvent> {
        self.events
            .iter()
            .filter(|e| e.severity == SecurityEventSeverity::Critical && !e.mitigated)
            .collect()
    }

    pub fn events_by_category(&self, category: &str) -> Vec<&ForensicsEvent> {
        self.events
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn events_by_source(&self, source: &str) -> Vec<&ForensicsEvent> {
        self.events.iter().filter(|e| e.source == source).collect()
    }

    pub fn recent_events(&self, count: usize) -> Vec<&ForensicsEvent> {
        self.events.iter().rev().take(count).collect()
    }

    pub fn all_events(&self) -> &[ForensicsEvent] {
        &self.events
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn clear_old_events(&mut self, max_age_secs: i64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs);
        self.events.retain(|e| e.timestamp >= cutoff);
    }

    pub fn unmitigated_count(&self) -> usize {
        self.events.iter().filter(|e| !e.mitigated).count()
    }
}

impl Default for SecurityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_retrieve() {
        let mut monitor = SecurityMonitor::new();
        let id = monitor.record_event(
            "auth",
            SecurityEventSeverity::Warning,
            "login",
            "Failed login attempt",
        );
        assert!(!id.is_nil());
        assert_eq!(monitor.event_count(), 1);
    }

    #[test]
    fn critical_unmitigated_filter() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_event(
            "auth",
            SecurityEventSeverity::Critical,
            "system",
            "Brute force detected",
        );
        monitor.record_event(
            "net",
            SecurityEventSeverity::Warning,
            "system",
            "Connection refused",
        );
        assert_eq!(monitor.critical_unmitigated().len(), 1);
    }

    #[test]
    fn mitigate_event() {
        let mut monitor = SecurityMonitor::new();
        let id = monitor.record_event(
            "auth",
            SecurityEventSeverity::Critical,
            "system",
            "Attack detected",
        );
        assert_eq!(monitor.unmitigated_count(), 1);
        assert!(monitor.mark_mitigated(id));
        assert_eq!(monitor.unmitigated_count(), 0);
        assert!(!monitor.mark_mitigated(Uuid::new_v4()));
    }

    #[test]
    fn max_events_eviction() {
        let mut monitor = SecurityMonitor::new().with_max_events(3);
        for i in 0..5 {
            monitor.record_event(
                &format!("cat-{i}"),
                SecurityEventSeverity::Info,
                "sys",
                "event",
            );
        }
        assert_eq!(monitor.event_count(), 3);
    }

    #[test]
    fn alerts_filter_by_threshold() {
        let mut monitor =
            SecurityMonitor::new().with_alert_threshold(SecurityEventSeverity::Critical);
        monitor.record_event("cat", SecurityEventSeverity::Info, "sys", "info");
        monitor.record_event("cat", SecurityEventSeverity::Warning, "sys", "warn");
        monitor.record_event("cat", SecurityEventSeverity::Critical, "sys", "critical");
        assert_eq!(monitor.alerts().len(), 1);
    }

    #[test]
    fn category_filter() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_event("auth", SecurityEventSeverity::Info, "s", "e1");
        monitor.record_event("net", SecurityEventSeverity::Info, "s", "e2");
        monitor.record_event("auth", SecurityEventSeverity::Info, "s", "e3");
        assert_eq!(monitor.events_by_category("auth").len(), 2);
        assert_eq!(monitor.events_by_category("net").len(), 1);
    }

    #[test]
    fn recent_events_ordering() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_event("a", SecurityEventSeverity::Info, "s", "first");
        monitor.record_event("b", SecurityEventSeverity::Info, "s", "second");
        let recent = monitor.recent_events(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].category, "b");
    }
}
