use crate::ambient::{AmbientAnalyzer, AmbientEnvironment};

/// Adapts output volume based on ambient noise level and environment.
///
/// Quiet room → reduce volume (-6dB)
/// Noisy room → increase volume (+6dB)
/// Music playing → duck music, keep voice at normal level
/// Gaming → reduce game audio slightly when speaking
pub struct VolumeAdapter {
    /// Target output gain in dB.
    target_gain_db: f32,
    /// Current smoothed gain in dB.
    current_gain_db: f32,
    /// Smoothing factor (0.0-1.0). Lower = smoother.
    smoothing: f32,
    /// Minimum gain (dB).
    min_gain_db: f32,
    /// Maximum gain (dB).
    max_gain_db: f32,
    /// Noise floor threshold for quiet/noisy boundary.
    quiet_threshold: f32,
    /// Noise floor threshold for noisy/very-noisy boundary.
    noisy_threshold: f32,
}

impl VolumeAdapter {
    pub fn new() -> Self {
        Self {
            target_gain_db: 0.0,
            current_gain_db: 0.0,
            smoothing: 0.1,
            min_gain_db: -12.0,
            max_gain_db: 12.0,
            quiet_threshold: 0.005,
            noisy_threshold: 0.05,
        }
    }

    pub fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing.clamp(0.01, 1.0);
        self
    }

    pub fn with_gain_range(mut self, min_db: f32, max_db: f32) -> Self {
        self.min_gain_db = min_db;
        self.max_gain_db = max_db;
        self
    }

    /// Update target volume based on ambient analysis.
    pub fn update(&mut self, ambient: &AmbientAnalyzer) {
        let noise_floor = ambient.noise_floor();
        let environment = ambient.environment();

        self.target_gain_db = match environment {
            AmbientEnvironment::Quiet => -3.0, // Quiet room: reduce slightly
            AmbientEnvironment::Noisy => {
                // Scale gain with noise level
                let noise_factor = ((noise_floor - self.quiet_threshold)
                    / (self.noisy_threshold - self.quiet_threshold))
                    .clamp(0.0, 1.0);
                noise_factor * 9.0 // 0 to +9dB
            }
            AmbientEnvironment::Music => 0.0, // Normal level, ducking handles music
            AmbientEnvironment::Gaming => 2.0, // Slightly louder for gaming
            AmbientEnvironment::Meeting => 0.0, // Normal for meetings
            AmbientEnvironment::Unknown => 0.0,
        };

        self.target_gain_db = self
            .target_gain_db
            .clamp(self.min_gain_db, self.max_gain_db);
    }

    /// Get current gain (smoothed). Call this per-frame.
    pub fn current_gain_db(&mut self) -> f32 {
        self.current_gain_db += (self.target_gain_db - self.current_gain_db) * self.smoothing;
        self.current_gain_db
    }

    /// Get the linear gain multiplier.
    pub fn gain_linear(&mut self) -> f32 {
        10.0f32.powf(self.current_gain_db() / 20.0)
    }

    pub fn reset(&mut self) {
        self.target_gain_db = 0.0;
        self.current_gain_db = 0.0;
    }
}

impl Default for VolumeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_adapter_creation() {
        let va = VolumeAdapter::new();
        assert_eq!(va.target_gain_db, 0.0);
        assert_eq!(va.current_gain_db, 0.0);
    }

    #[test]
    fn volume_adapter_quiet_room() {
        let mut va = VolumeAdapter::new();
        let mut ambient = AmbientAnalyzer::new();
        for _ in 0..40 {
            ambient.analyze_frame(0.001, 500.0);
        }
        va.update(&ambient);
        assert!(va.target_gain_db < 0.0);
    }

    #[test]
    fn volume_adapter_noisy_room() {
        let mut va = VolumeAdapter::new();
        let mut ambient = AmbientAnalyzer::new();
        for _ in 0..40 {
            ambient.analyze_frame(0.08, 1000.0);
        }
        va.update(&ambient);
        assert!(va.target_gain_db > 0.0);
    }

    #[test]
    fn volume_adapter_smooth_transition() {
        let mut va = VolumeAdapter::new();
        let gain1 = va.current_gain_db();
        va.target_gain_db = 6.0;
        let gain2 = va.current_gain_db();
        // Smooth transition: gain2 should be between gain1 and target
        assert!(gain2 > gain1);
        assert!(gain2 < 6.0);
    }

    #[test]
    fn volume_adapter_gain_range() {
        let va = VolumeAdapter::new().with_gain_range(-6.0, 6.0);
        assert_eq!(va.min_gain_db, -6.0);
        assert_eq!(va.max_gain_db, 6.0);
    }

    #[test]
    fn volume_adapter_reset() {
        let mut va = VolumeAdapter::new();
        va.target_gain_db = 6.0;
        va.current_gain_db = 3.0;
        va.reset();
        assert_eq!(va.target_gain_db, 0.0);
        assert_eq!(va.current_gain_db, 0.0);
    }
}
