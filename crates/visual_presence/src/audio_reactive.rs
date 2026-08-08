use crate::config::AudioReactiveConfig;

pub struct AudioReactive {
    config: AudioReactiveConfig,
    current_rms: f32,
    smoothed_rms: f32,
    peak_rms: f32,
    pulse_phase: f32,
}

impl AudioReactive {
    pub fn new(config: AudioReactiveConfig) -> Self {
        Self {
            config,
            current_rms: 0.0,
            smoothed_rms: 0.0,
            peak_rms: 0.0,
            pulse_phase: 0.0,
        }
    }

    pub fn update_rms(&mut self, rms: f32) {
        self.current_rms = rms;
        self.smoothed_rms =
            self.smoothed_rms * self.config.rms_smoothing + rms * (1.0 - self.config.rms_smoothing);
        self.peak_rms = self.peak_rms.max(rms);
    }

    pub fn update(&mut self, delta_time: f32) {
        self.pulse_phase += delta_time * self.config.pulse_speed;
    }

    pub fn glow_intensity(&self) -> f32 {
        1.0 + self.smoothed_rms * self.config.glow_boost
    }

    pub fn expansion(&self) -> f32 {
        1.0 + self.smoothed_rms * self.config.expansion_factor
    }

    pub fn pulse(&self) -> f32 {
        if self.smoothed_rms > 0.1 {
            self.pulse_phase.sin() * self.smoothed_rms
        } else {
            0.0
        }
    }

    pub fn rms(&self) -> f32 {
        self.current_rms
    }

    pub fn smoothed_rms(&self) -> f32 {
        self.smoothed_rms
    }

    pub fn peak_rms(&self) -> f32 {
        self.peak_rms
    }

    pub fn reset_peak(&mut self) {
        self.peak_rms = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_reactive_creation() {
        let config = AudioReactiveConfig::default();
        let audio = AudioReactive::new(config);
        assert_eq!(audio.rms(), 0.0);
        assert_eq!(audio.smoothed_rms(), 0.0);
    }

    #[test]
    fn test_rms_update() {
        let config = AudioReactiveConfig::default();
        let mut audio = AudioReactive::new(config);
        audio.update_rms(0.5);
        assert_eq!(audio.rms(), 0.5);
        assert!(audio.smoothed_rms() > 0.0);
    }

    #[test]
    fn test_glow_intensity() {
        let config = AudioReactiveConfig::default();
        let mut audio = AudioReactive::new(config);
        audio.update_rms(0.5);
        assert!(audio.glow_intensity() > 1.0);
    }

    #[test]
    fn test_peak_tracking() {
        let config = AudioReactiveConfig::default();
        let mut audio = AudioReactive::new(config);
        audio.update_rms(0.3);
        audio.update_rms(0.7);
        assert_eq!(audio.peak_rms(), 0.7);
        audio.reset_peak();
        assert_eq!(audio.peak_rms(), 0.0);
    }

    #[test]
    fn test_pulse() {
        let config = AudioReactiveConfig::default();
        let mut audio = AudioReactive::new(config);
        audio.update_rms(0.5);
        audio.update(0.016);
        assert!(audio.pulse().abs() < 1.0);
    }
}
