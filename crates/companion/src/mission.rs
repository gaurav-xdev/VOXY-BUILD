use chrono::{DateTime, Utc};

use crate::attention::ActivityKind;
use crate::types::MissionState;

/// Mission companion — tracks background work.
pub struct MissionCompanion {
    current: MissionState,
    completed_missions: Vec<MissionState>,
}

impl MissionCompanion {
    pub fn new() -> Self {
        Self {
            current: MissionState::Idle,
            completed_missions: Vec::new(),
        }
    }

    /// Start a mission when user is detected doing eligible work.
    pub fn start_mission(&mut self, kind: ActivityKind, description: &str, now: DateTime<Utc>) {
        self.current = MissionState::Active {
            kind,
            started_at: now,
            description: description.to_string(),
        };
    }

    /// Complete the current mission.
    pub fn complete_mission(&mut self, summary: &str, now: DateTime<Utc>) {
        if let MissionState::Active {
            kind, started_at, ..
        } = self.current.clone()
        {
            let completed = MissionState::Completed {
                kind,
                started_at,
                completed_at: now,
                summary: summary.to_string(),
            };
            self.completed_missions.push(completed);
            self.current = MissionState::Idle;
        }
    }

    /// Check if a mission should be auto-completed (activity changed).
    pub fn check_activity_change(
        &mut self,
        new_activity: Option<ActivityKind>,
        now: DateTime<Utc>,
    ) -> Option<String> {
        if let MissionState::Active { kind, .. } = &self.current {
            if let Some(new) = new_activity {
                if new != *kind {
                    let summary = format!("Auto-completed: {:?}", kind);
                    self.complete_mission(&summary, now);
                    return Some(summary);
                }
            }
        }
        None
    }

    /// Generate a natural summary of what happened while user was away.
    pub fn generate_return_summary(&self) -> Option<String> {
        self.completed_missions.last().map(|m| match m {
            MissionState::Completed {
                kind,
                summary,
                started_at,
                completed_at,
            } => {
                let duration = completed_at.signed_duration_since(*started_at);
                let minutes = duration.num_minutes();
                if minutes > 0 {
                    format!(
                        "While you were away: {:?} completed in {} minutes. {}",
                        kind, minutes, summary
                    )
                } else {
                    format!("While you were away: {:?}. {}", kind, summary)
                }
            }
            _ => "Mission summary unavailable.".to_string(),
        })
    }

    pub fn current_state(&self) -> &MissionState {
        &self.current
    }

    pub fn is_active(&self) -> bool {
        matches!(self.current, MissionState::Active { .. })
    }

    pub fn completed_count(&self) -> usize {
        self.completed_missions.len()
    }
}

impl Default for MissionCompanion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_lifecycle() {
        let mut mc = MissionCompanion::new();
        assert!(!mc.is_active());

        mc.start_mission(ActivityKind::Coding, "Implement feature X", Utc::now());
        assert!(mc.is_active());

        mc.complete_mission("Feature X done", Utc::now());
        assert!(!mc.is_active());
        assert_eq!(mc.completed_count(), 1);
    }

    #[test]
    fn test_return_summary() {
        let mut mc = MissionCompanion::new();
        mc.start_mission(ActivityKind::Coding, "Work", Utc::now());
        mc.complete_mission("Done", Utc::now());
        let summary = mc.generate_return_summary();
        assert!(summary.is_some());
        assert!(summary.unwrap().contains("While you were away"));
    }

    #[test]
    fn test_auto_complete_on_activity_change() {
        let mut mc = MissionCompanion::new();
        mc.start_mission(ActivityKind::Coding, "Code", Utc::now());
        let result = mc.check_activity_change(Some(ActivityKind::Browsing), Utc::now());
        assert!(result.is_some());
        assert!(!mc.is_active());
    }
}
