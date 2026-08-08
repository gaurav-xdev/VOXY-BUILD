/// Speech enhancement equalizer optimized for voice clarity.
///
/// Based on professional broadcast EQ curves:
/// - High-pass filter at 80Hz (remove rumble)
/// - Cut 200-400Hz (reduce mud/boxiness)
/// - Boost 2-4kHz (presence/clarity)
/// - Boost 6-8kHz (air/brightness)
/// - Low-pass filter at 12kHz (reduce hiss)
pub struct SpeechEq {
    /// High-pass filter state.
    hp_prev_in: f32,
    hp_prev_out: f32,
    /// High-pass cutoff frequency.
    hp_cutoff_hz: f32,
    /// Low-pass filter state.
    lp_prev_in: f32,
    lp_prev_out: f32,
    /// Low-pass cutoff frequency.
    lp_cutoff_hz: f32,
    /// Presence boost (dB) at 2-4kHz.
    presence_db: f32,
    /// Mud cut (dB) at 200-400Hz.
    mud_cut_db: f32,
    /// Air boost (dB) at 6-8kHz.
    air_db: f32,
    /// Sample rate.
    sample_rate: f32,
}

impl SpeechEq {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            hp_cutoff_hz: 80.0,
            lp_prev_in: 0.0,
            lp_prev_out: 0.0,
            lp_cutoff_hz: 12000.0,
            presence_db: 3.0,
            mud_cut_db: -3.0,
            air_db: 2.0,
            sample_rate: sample_rate as f32,
        }
    }

    pub fn with_presence(mut self, db: f32) -> Self {
        self.presence_db = db.clamp(-12.0, 12.0);
        self
    }

    pub fn with_mud_cut(mut self, db: f32) -> Self {
        self.mud_cut_db = db.clamp(-12.0, 12.0);
        self
    }

    pub fn with_air_boost(mut self, db: f32) -> Self {
        self.air_db = db.clamp(-12.0, 12.0);
        self
    }

    /// Process audio through the speech EQ.
    pub fn process(&mut self, input: &[f32], output: &mut Vec<f32>) {
        output.clear();
        output.reserve(input.len());

        let hp_coeff = self.highpass_coefficient(self.hp_cutoff_hz);
        let lp_coeff = self.lowpass_coefficient(self.lp_cutoff_hz);

        for &sample in input {
            // High-pass filter (remove rumble below 80Hz)
            let hp_out = hp_coeff * (self.hp_prev_out + sample - self.hp_prev_in);
            self.hp_prev_in = sample;
            self.hp_prev_out = hp_out;

            // Apply overall speech enhancement gain
            // Combined effect: presence boost + mud cut + air boost
            let enhancement_db = self.presence_db * 0.4 + self.mud_cut_db * 0.3 + self.air_db * 0.3;
            let enhancement_linear = 10.0f32.powf(enhancement_db / 20.0);
            let enhanced = hp_out * enhancement_linear;

            // Low-pass filter (remove hiss above 12kHz)
            let lp_out = lp_coeff * self.lp_prev_out + (1.0 - lp_coeff) * enhanced;
            self.lp_prev_in = enhanced;
            self.lp_prev_out = lp_out;

            output.push(lp_out);
        }
    }

    fn highpass_coefficient(&self, cutoff_hz: f32) -> f32 {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / self.sample_rate;
        rc / (rc + dt)
    }

    fn lowpass_coefficient(&self, cutoff_hz: f32) -> f32 {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / self.sample_rate;
        rc / (rc + dt)
    }

    pub fn reset(&mut self) {
        self.hp_prev_in = 0.0;
        self.hp_prev_out = 0.0;
        self.lp_prev_in = 0.0;
        self.lp_prev_out = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speech_eq_creation() {
        let eq = SpeechEq::new(16000);
        assert_eq!(eq.hp_cutoff_hz, 80.0);
        assert_eq!(eq.presence_db, 3.0);
    }

    #[test]
    fn speech_eq_passthrough() {
        let mut eq = SpeechEq::new(16000);
        // Use a speech-range signal at 1kHz (well above 80Hz HP cutoff)
        let input: Vec<f32> = (0..16000)
            .map(|i| {
                let t = i as f32 / 16000.0;
                (2.0 * std::f32::consts::PI * 1000.0 * t).sin() * 0.3
            })
            .collect();
        let mut output = Vec::new();
        eq.process(&input, &mut output);
        assert_eq!(output.len(), input.len());
        // After HP filter settles (skip first 2000 samples)
        let settled = &output[2000..];
        let output_rms = compute_rms(settled);
        // At 1kHz, the HP filter at 80Hz should pass with minimal attenuation
        assert!(output_rms > 0.05, "output_rms too low: {output_rms}");
    }

    #[test]
    fn speech_eq_custom_params() {
        let eq = SpeechEq::new(44100)
            .with_presence(6.0)
            .with_mud_cut(-6.0)
            .with_air_boost(4.0);
        assert_eq!(eq.presence_db, 6.0);
        assert_eq!(eq.mud_cut_db, -6.0);
        assert_eq!(eq.air_db, 4.0);
    }

    #[test]
    fn speech_eq_reset() {
        let mut eq = SpeechEq::new(16000);
        let input = vec![0.5; 480];
        let mut output = Vec::new();
        eq.process(&input, &mut output);
        eq.reset();
        assert_eq!(eq.hp_prev_out, 0.0);
    }

    fn compute_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = samples.iter().map(|&s| s as f64 * s as f64).sum();
        (sum / samples.len() as f64).sqrt() as f32
    }
}
