use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub threat_type: String,
    pub severity: ThreatSeverity,
    pub description: String,
    pub details: std::collections::HashMap<String, String>,
    pub is_handled: bool,
}

pub struct ThreatDetector {
    events: Vec<ThreatEvent>,
}

impl ThreatDetector {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn record_event(
        &mut self,
        source: &str,
        threat_type: &str,
        severity: ThreatSeverity,
        description: &str,
    ) -> Uuid {
        let event = ThreatEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source: source.to_string(),
            threat_type: threat_type.to_string(),
            severity,
            description: description.to_string(),
            details: std::collections::HashMap::new(),
            is_handled: false,
        };
        let id = event.id;
        self.events.push(event);
        id
    }

    pub fn record_event_with_details(
        &mut self,
        source: &str,
        threat_type: &str,
        severity: ThreatSeverity,
        description: &str,
        details: std::collections::HashMap<String, String>,
    ) -> Uuid {
        let event = ThreatEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source: source.to_string(),
            threat_type: threat_type.to_string(),
            severity,
            description: description.to_string(),
            details,
            is_handled: false,
        };
        let id = event.id;
        self.events.push(event);
        id
    }

    pub fn mark_handled(&mut self, id: Uuid) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.is_handled = true;
        }
    }

    pub fn unhandled_events(&self) -> Vec<&ThreatEvent> {
        self.events.iter().filter(|e| !e.is_handled).collect()
    }

    pub fn events_by_severity(&self, min_severity: ThreatSeverity) -> Vec<&ThreatEvent> {
        self.events
            .iter()
            .filter(|e| e.severity as u8 >= min_severity as u8)
            .collect()
    }

    pub fn events_by_type(&self, threat_type: &str) -> Vec<&ThreatEvent> {
        self.events
            .iter()
            .filter(|e| e.threat_type == threat_type)
            .collect()
    }

    pub fn all_events(&self) -> &[ThreatEvent] {
        &self.events
    }

    pub fn rapid_failure_detected(&self, source: &str, window_secs: i64, threshold: usize) -> bool {
        let cutoff = Utc::now() - chrono::Duration::seconds(window_secs);
        let count = self
            .events
            .iter()
            .filter(|e| e.source == source && e.timestamp >= cutoff && !e.is_handled)
            .count();
        count >= threshold
    }

    pub fn clear_old_events(&mut self, max_age_secs: i64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs);
        self.events.retain(|e| e.timestamp >= cutoff);
    }
}

impl Default for ThreatDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threat_event_recording() {
        let mut detector = ThreatDetector::new();
        let id = detector.record_event(
            "auth",
            "brute_force",
            ThreatSeverity::High,
            "Multiple failed logins",
        );
        assert!(!id.is_nil());
        assert_eq!(detector.events_by_type("brute_force").len(), 1);
    }

    #[test]
    fn threat_unhandled_tracking() {
        let mut detector = ThreatDetector::new();
        let id = detector.record_event(
            "network",
            "port_scan",
            ThreatSeverity::Medium,
            "Port scan detected",
        );
        assert_eq!(detector.unhandled_events().len(), 1);
        detector.mark_handled(id);
        assert_eq!(detector.unhandled_events().len(), 0);
    }

    #[test]
    fn threat_severity_filter() {
        let mut detector = ThreatDetector::new();
        detector.record_event("sys", "warning", ThreatSeverity::Low, "Low priority");
        detector.record_event("sys", "alert", ThreatSeverity::High, "High priority");
        let high_or_critical = detector.events_by_severity(ThreatSeverity::High);
        assert_eq!(high_or_critical.len(), 1);
    }

    #[test]
    fn threat_rapid_failure_detection() {
        let mut detector = ThreatDetector::new();
        for _ in 0..5 {
            detector.record_event("auth", "login_failure", ThreatSeverity::Low, "Failed login");
        }
        assert!(detector.rapid_failure_detected("auth", 60, 5));
        assert!(!detector.rapid_failure_detected("auth", 60, 10));
    }

    #[test]
    fn threat_clear_old_events() {
        let mut detector = ThreatDetector::new();
        detector.record_event("sys", "test", ThreatSeverity::Info, "Old event");
        detector.clear_old_events(0);
        assert_eq!(detector.all_events().len(), 0);
    }

    #[test]
    fn threat_severity_ordering() {
        assert!(ThreatSeverity::Info < ThreatSeverity::Low);
        assert!(ThreatSeverity::Low < ThreatSeverity::Medium);
        assert!(ThreatSeverity::Medium < ThreatSeverity::High);
        assert!(ThreatSeverity::High < ThreatSeverity::Critical);
    }
}
