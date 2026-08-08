use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Tracks heartbeats from subsystems. If a heartbeat is missed
/// beyond the configured timeout, the subsystem is considered dead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Expected interval between heartbeats (ms).
    pub expected_interval_ms: u64,
    /// Max allowed drift before declaring missed (multiplier).
    pub tolerance_multiplier: f64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            expected_interval_ms: 5000,
            tolerance_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
struct HeartbeatEntry {
    last_seen: DateTime<Utc>,
    total_beats: u64,
    missed_count: u32,
}

pub struct HeartbeatTracker {
    entries: parking_lot::RwLock<HashMap<String, HeartbeatEntry>>,
    config: HeartbeatConfig,
}

impl HeartbeatTracker {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            entries: parking_lot::RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Register a subsystem for heartbeat tracking.
    pub fn register(&self, name: &str) {
        let mut entries = self.entries.write();
        entries
            .entry(name.to_string())
            .or_insert_with(|| HeartbeatEntry {
                last_seen: Utc::now(),
                total_beats: 0,
                missed_count: 0,
            });
        debug!("Heartbeat registered: {}", name);
    }

    /// Record a heartbeat from a subsystem.
    pub fn beat(&self, name: &str) {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(name) {
            entry.last_seen = Utc::now();
            entry.total_beats += 1;
            entry.missed_count = 0;
        }
    }

    /// Check if a subsystem has missed its heartbeat.
    pub fn is_alive(&self, name: &str) -> bool {
        let entries = self.entries.read();
        if let Some(entry) = entries.get(name) {
            let now = Utc::now();
            let elapsed = (now - entry.last_seen).num_milliseconds() as u64;
            let max_interval =
                (self.config.expected_interval_ms as f64 * self.config.tolerance_multiplier) as u64;
            elapsed <= max_interval
        } else {
            false
        }
    }

    /// Get the number of missed heartbeats for a subsystem.
    pub fn missed_count(&self, name: &str) -> u32 {
        let entries = self.entries.read();
        entries.get(name).map(|e| e.missed_count).unwrap_or(0)
    }

    /// Get total heartbeat count for a subsystem.
    pub fn total_beats(&self, name: &str) -> u64 {
        let entries = self.entries.read();
        entries.get(name).map(|e| e.total_beats).unwrap_or(0)
    }

    /// Increment missed heartbeat count (called by watchdog when beat is missed).
    pub fn increment_missed(&self, name: &str) -> u32 {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(name) {
            entry.missed_count += 1;
            entry.missed_count
        } else {
            0
        }
    }

    /// Check all subsystems and return list of dead ones.
    pub fn check_all_alive(&self) -> Vec<String> {
        let entries = self.entries.read();
        let now = Utc::now();
        let max_interval =
            (self.config.expected_interval_ms as f64 * self.config.tolerance_multiplier) as u64;

        entries
            .iter()
            .filter(|(_, entry)| {
                let elapsed = (now - entry.last_seen).num_milliseconds() as u64;
                elapsed > max_interval
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get last heartbeat time for a subsystem.
    pub fn last_seen(&self, name: &str) -> Option<DateTime<Utc>> {
        let entries = self.entries.read();
        entries.get(name).map(|e| e.last_seen)
    }

    /// Remove a subsystem from tracking.
    pub fn unregister(&self, name: &str) {
        self.entries.write().remove(name);
    }

    /// Get count of registered subsystems.
    pub fn count(&self) -> usize {
        self.entries.read().len()
    }
}

impl Default for HeartbeatTracker {
    fn default() -> Self {
        Self::new(HeartbeatConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_creation() {
        let tracker = HeartbeatTracker::new(HeartbeatConfig::default());
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn register_and_beat() {
        let tracker = HeartbeatTracker::new(HeartbeatConfig::default());
        tracker.register("audio");
        assert_eq!(tracker.count(), 1);
        assert!(tracker.is_alive("audio"));
        tracker.beat("audio");
        assert_eq!(tracker.total_beats("audio"), 1);
    }

    #[test]
    fn unregistered_is_dead() {
        let tracker = HeartbeatTracker::new(HeartbeatConfig::default());
        assert!(!tracker.is_alive("nonexistent"));
    }

    #[test]
    fn check_all_alive_empty() {
        let tracker = HeartbeatTracker::new(HeartbeatConfig::default());
        assert!(tracker.check_all_alive().is_empty());
    }

    #[test]
    fn unregister() {
        let tracker = HeartbeatTracker::new(HeartbeatConfig::default());
        tracker.register("svc");
        assert_eq!(tracker.count(), 1);
        tracker.unregister("svc");
        assert_eq!(tracker.count(), 0);
    }
}
