use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::PresenceConfig;
use crate::types::UserPresence;

/// Presence animation states for the orb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceState {
    /// Normal breathing — calm, steady pulse.
    Breathing,
    /// Blinking — brief visual event.
    Blinking,
    /// Looking around — subtle directional shift.
    LookingAround,
    /// Pulse — energy burst, e.g. greeting.
    Pulsing,
    /// Resting — very slow breathing, low energy.
    Resting,
    /// Alert — faster breathing, attentive.
    Alert,
    /// Thinking — slower, contemplative movement.
    Thinking,
    /// Focused — steady, attentive presence.
    Focused,
}

impl PresenceState {
    pub fn to_breathing_speed(&self) -> f64 {
        match self {
            Self::Breathing => 0.5,
            Self::Blinking => 0.5,
            Self::LookingAround => 0.5,
            Self::Pulsing => 0.8,
            Self::Resting => 0.3,
            Self::Alert => 0.8,
            Self::Thinking => 0.4,
            Self::Focused => 0.6,
        }
    }
}

/// Presence system state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceSnapshot {
    pub state: PresenceState,
    pub breathing_phase: f64,
    pub blink_progress: f64,
    pub pulse_intensity: f64,
    pub movement_offset_x: f64,
    pub movement_offset_y: f64,
    pub energy: f64,
}

/// Controls the orb's visual presence.
#[derive(Debug)]
pub struct PresenceSystem {
    config: PresenceConfig,
    state: PresenceState,
    breathing_phase: f64,
    blink_timer: Duration,
    next_blink: Duration,
    blink_progress: f64,
    is_blinking: bool,
    pulse_timer: Option<Duration>,
    pulse_intensity: f64,
    look_timer: Duration,
    movement_x: f64,
    movement_y: f64,
    energy: f64,
    elapsed: Duration,
}

impl PresenceSystem {
    pub fn new(config: PresenceConfig) -> Self {
        let next_blink = Self::random_blink_interval(&config);
        Self {
            config,
            state: PresenceState::Breathing,
            breathing_phase: 0.0,
            blink_timer: Duration::ZERO,
            next_blink,
            blink_progress: 0.0,
            is_blinking: false,
            pulse_timer: None,
            pulse_intensity: 0.0,
            look_timer: Duration::ZERO,
            movement_x: 0.0,
            movement_y: 0.0,
            energy: 0.5,
            elapsed: Duration::ZERO,
        }
    }

    /// Advance presence by one tick. Call at `update_interval`.
    pub fn tick(&mut self, dt: Duration, presence: &UserPresence) -> PresenceSnapshot {
        self.elapsed += dt;

        self.update_energy(presence);
        self.update_breathing(dt);
        self.update_blink(dt);
        self.update_pulse(dt);
        self.update_look_around(dt);
        self.update_state(presence);

        PresenceSnapshot {
            state: self.state,
            breathing_phase: self.breathing_phase,
            blink_progress: self.blink_progress,
            pulse_intensity: self.pulse_intensity,
            movement_offset_x: self.movement_x,
            movement_offset_y: self.movement_y,
            energy: self.energy,
        }
    }

    /// Trigger a pulse (e.g. on greeting or micro interaction).
    pub fn trigger_pulse(&mut self, intensity: f64) {
        self.pulse_timer = Some(Duration::ZERO);
        self.pulse_intensity = intensity.clamp(
            self.config.pulse_intensity_min,
            self.config.pulse_intensity_max,
        );
    }

    pub fn state(&self) -> PresenceState {
        self.state
    }

    pub fn energy(&self) -> f64 {
        self.energy
    }

    fn update_energy(&mut self, presence: &UserPresence) {
        let target = match presence {
            UserPresence::Active => 0.7,
            UserPresence::Focused => 0.6,
            UserPresence::Idle { .. } => 0.3,
            UserPresence::Away { .. } => 0.15,
            UserPresence::Sleeping { .. } => 0.1,
            UserPresence::InMeeting => 0.5,
            UserPresence::Gaming => 0.6,
            UserPresence::Browsing => 0.4,
        };
        self.energy += (target - self.energy) * 0.05;
    }

    fn update_breathing(&mut self, dt: Duration) {
        let period = self.config.breathing_period.as_secs_f64();
        let speed = self.state.to_breathing_speed();
        let increment = (dt.as_secs_f64() / period) * std::f64::consts::TAU * speed;
        self.breathing_phase = (self.breathing_phase + increment) % std::f64::consts::TAU;
    }

    fn update_blink(&mut self, dt: Duration) {
        if self.is_blinking {
            self.blink_timer += dt;
            let blink_duration = self.config.blink_duration;
            self.blink_progress =
                (self.blink_timer.as_secs_f64() / blink_duration.as_secs_f64()).min(1.0);
            if self.blink_timer >= blink_duration {
                self.is_blinking = false;
                self.blink_timer = Duration::ZERO;
                self.blink_progress = 0.0;
                self.next_blink = Self::random_blink_interval(&self.config);
            }
        } else {
            self.blink_timer += dt;
            if self.blink_timer >= self.next_blink {
                self.is_blinking = true;
                self.blink_timer = Duration::ZERO;
            }
        }
    }

    fn update_pulse(&mut self, dt: Duration) {
        if let Some(ref mut timer) = self.pulse_timer {
            *timer += dt;
            let total = Duration::from_secs(1);
            if *timer >= total {
                self.pulse_timer = None;
                self.pulse_intensity = 0.0;
            }
        }
    }

    fn update_look_around(&mut self, dt: Duration) {
        self.look_timer += dt;
        let look_interval = Duration::from_secs(8);
        if self.look_timer >= look_interval {
            self.look_timer = Duration::ZERO;
            if self.energy > 0.3 && !self.is_blinking {
                let seed = self.elapsed.as_millis() as f64;
                let hash = (seed * std::f64::consts::FRAC_PI_4).sin();
                if hash.abs() < self.config.look_around_probability {
                    self.movement_x = (hash * 2.0).clamp(-1.0, 1.0);
                    self.movement_y = ((seed * 0.321) * std::f64::consts::FRAC_PI_4)
                        .sin()
                        .clamp(-1.0, 1.0);
                    return;
                }
            }
        }
        self.movement_x *= 0.95;
        self.movement_y *= 0.95;
    }

    fn update_state(&mut self, presence: &UserPresence) {
        self.state = match presence {
            UserPresence::Sleeping { .. } | UserPresence::Away { .. } => PresenceState::Resting,
            UserPresence::Focused => {
                if self.is_blinking {
                    PresenceState::Blinking
                } else {
                    PresenceState::Focused
                }
            }
            UserPresence::InMeeting => PresenceState::Alert,
            _ => {
                if self.is_blinking {
                    PresenceState::Blinking
                } else if self.pulse_timer.is_some() {
                    PresenceState::Pulsing
                } else if self.energy < 0.25 {
                    PresenceState::Resting
                } else {
                    PresenceState::Breathing
                }
            }
        };
    }

    fn random_blink_interval(config: &PresenceConfig) -> Duration {
        let min = config.blink_interval_min.as_secs_f64();
        let max = config.blink_interval_max.as_secs_f64();
        let seed = std::time::Instant::now().elapsed().as_nanos() as f64;
        let ratio = ((seed * std::f64::consts::FRAC_PI_4).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        Duration::from_secs_f64(min + ratio * (max - min))
    }
}

impl Default for PresenceSystem {
    fn default() -> Self {
        Self::new(PresenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_breathing() {
        let mut system = PresenceSystem::new(PresenceConfig::default());
        let snap = system.tick(Duration::from_millis(100), &UserPresence::Active);
        assert!(snap.breathing_phase >= 0.0);
    }

    #[test]
    fn test_presence_resting_when_away() {
        let mut system = PresenceSystem::new(PresenceConfig::default());
        let snap = system.tick(
            Duration::from_millis(100),
            &UserPresence::Away {
                since: chrono::Utc::now(),
            },
        );
        assert_eq!(snap.state, PresenceState::Resting);
    }

    #[test]
    fn test_presence_pulse_trigger() {
        let mut system = PresenceSystem::new(PresenceConfig::default());
        system.trigger_pulse(0.6);
        let snap = system.tick(Duration::from_millis(50), &UserPresence::Active);
        assert!(snap.pulse_intensity > 0.0);
    }

    #[test]
    fn test_energy_tracks_presence() {
        let mut system = PresenceSystem::new(PresenceConfig::default());
        system.tick(Duration::from_millis(100), &UserPresence::Active);
        assert!(system.energy() > 0.5);
    }

    #[test]
    fn test_blink_occurs() {
        let mut system = PresenceSystem::new(PresenceConfig::default());
        for _ in 0..1000 {
            system.tick(Duration::from_millis(50), &UserPresence::Active);
        }
        assert!(system.is_blinking || system.blink_progress == 0.0);
    }
}
