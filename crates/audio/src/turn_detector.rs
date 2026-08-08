use std::collections::VecDeque;

/// Detects when a speaker has finished their turn by analyzing
/// energy patterns, speech rate, and prosodic cues.
///
/// This replaces the fixed silence timeout with an adaptive model
/// that adjusts based on the speaker's recent speech characteristics.
pub struct TurnDetector {
    /// Recent energy levels (rolling window).
    energy_history: VecDeque<f32>,
    /// Maximum history length.
    max_history: usize,
    /// Current estimated speech rate (frames of speech per second).
    #[allow(dead_code)]
    speech_rate: f32,
    /// Recent speech burst durations (frames).
    speech_bursts: VecDeque<usize>,
    /// Maximum burst history.
    max_bursts: usize,
    /// Minimum silence frames before turn-end (adaptive).
    adaptive_silence_threshold: usize,
    /// Base silence threshold (frames).
    base_silence_threshold: usize,
    /// Current consecutive silence frames.
    consecutive_silence: usize,
    /// Whether currently in speech.
    in_speech: bool,
    /// Current speech burst length.
    current_burst: usize,
    /// Energy threshold for voice detection.
    energy_threshold: f32,
    /// Minimum speech frames to confirm speech start.
    min_speech_frames: usize,
    /// Consecutive frames above threshold needed.
    speech_confirm_count: usize,
}

impl TurnDetector {
    pub fn new(
        energy_threshold: f32,
        base_silence_ms: u32,
        frame_size_samples: usize,
        sample_rate: u32,
    ) -> Self {
        let frames_per_sec = sample_rate as usize / frame_size_samples.max(1);
        let base_silence_frames = (base_silence_ms as usize * frames_per_sec / 1000).max(1);

        Self {
            energy_history: VecDeque::with_capacity(100),
            max_history: 100,
            speech_rate: 0.0,
            speech_bursts: VecDeque::with_capacity(20),
            max_bursts: 20,
            adaptive_silence_threshold: base_silence_frames,
            base_silence_threshold: base_silence_frames,
            consecutive_silence: 0,
            in_speech: false,
            current_burst: 0,
            energy_threshold,
            min_speech_frames: 3,
            speech_confirm_count: 0,
        }
    }

    pub fn with_min_speech_frames(mut self, frames: usize) -> Self {
        self.min_speech_frames = frames;
        self
    }

    /// Process a new audio frame and return whether the turn has ended.
    /// Returns `Some(true)` if turn ended, `Some(false)` if still in speech,
    /// `None` if undetermined.
    pub fn process_frame(&mut self, energy: f32) -> Option<bool> {
        self.energy_history.push_back(energy);
        if self.energy_history.len() > self.max_history {
            self.energy_history.pop_front();
        }

        let is_voice = energy >= self.energy_threshold;

        if is_voice {
            self.consecutive_silence = 0;
            self.current_burst += 1;
            self.speech_confirm_count += 1;

            if !self.in_speech && self.speech_confirm_count >= self.min_speech_frames {
                self.in_speech = true;
            }
            None
        } else {
            self.speech_confirm_count = 0;

            if self.in_speech {
                self.consecutive_silence += 1;

                // Update speech rate from completed burst
                if self.current_burst > 0 {
                    self.speech_bursts.push_back(self.current_burst);
                    if self.speech_bursts.len() > self.max_bursts {
                        self.speech_bursts.pop_front();
                    }
                    self.update_adaptive_threshold();
                    self.current_burst = 0;
                }

                if self.consecutive_silence >= self.adaptive_silence_threshold {
                    self.in_speech = false;
                    self.consecutive_silence = 0;
                    return Some(true); // Turn ended
                }
                None
            } else {
                None
            }
        }
    }

    /// Update the adaptive silence threshold based on recent speech patterns.
    fn update_adaptive_threshold(&mut self) {
        if self.speech_bursts.is_empty() {
            return;
        }

        // Calculate average burst length
        let avg_burst: f32 =
            self.speech_bursts.iter().sum::<usize>() as f32 / self.speech_bursts.len() as f32;

        // Fast talkers (short bursts) get shorter silence timeouts
        // Slow talkers (long bursts) get longer silence timeouts
        let rate_factor = if avg_burst > 0.0 {
            (avg_burst / 10.0).clamp(0.5, 2.0)
        } else {
            1.0
        };

        // Also consider recent energy variance — low variance = trailing off
        let energy_variance = self.compute_energy_variance();
        let variance_factor = if energy_variance < 0.001 {
            0.7 // Trailing off — shorter timeout
        } else if energy_variance > 0.01 {
            1.3 // Active speech — longer timeout
        } else {
            1.0
        };

        self.adaptive_silence_threshold =
            (self.base_silence_threshold as f32 * rate_factor * variance_factor) as usize;
        self.adaptive_silence_threshold = self.adaptive_silence_threshold.max(3);
        // Minimum 3 frames
    }

    /// Compute variance of recent energy levels.
    fn compute_energy_variance(&self) -> f32 {
        if self.energy_history.len() < 2 {
            return 0.0;
        }
        let mean: f32 = self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;
        let variance: f32 = self
            .energy_history
            .iter()
            .map(|&e| (e - mean).powi(2))
            .sum::<f32>()
            / self.energy_history.len() as f32;
        variance
    }

    /// Get the current adaptive silence threshold in frames.
    pub fn adaptive_threshold(&self) -> usize {
        self.adaptive_silence_threshold
    }

    /// Whether currently in speech.
    pub fn in_speech(&self) -> bool {
        self.in_speech
    }

    /// Estimated speech rate (avg frames per burst).
    pub fn speech_rate(&self) -> f32 {
        if self.speech_bursts.is_empty() {
            return 0.0;
        }
        self.speech_bursts.iter().sum::<usize>() as f32 / self.speech_bursts.len() as f32
    }

    /// Reset the detector state.
    pub fn reset(&mut self) {
        self.energy_history.clear();
        self.speech_bursts.clear();
        self.consecutive_silence = 0;
        self.in_speech = false;
        self.current_burst = 0;
        self.speech_confirm_count = 0;
        self.adaptive_silence_threshold = self.base_silence_threshold;
    }
}

/// Analyzes pitch contour for prosodic turn-taking cues.
///
/// Detects:
/// - Rising intonation (question) → longer expected pause
/// - Falling intonation (statement) → shorter expected pause
/// - Trailing off (energy decay) → turn likely ending
/// - Abrupt stop (high energy → silence) → may resume
pub struct ProsodyAnalyzer {
    /// Recent F0 (pitch) estimates.
    pitch_history: VecDeque<f32>,
    /// Recent energy levels.
    energy_history: VecDeque<f32>,
    /// Window size for analysis.
    window_size: usize,
}

impl ProsodyAnalyzer {
    pub fn new(window_size: usize) -> Self {
        Self {
            pitch_history: VecDeque::with_capacity(window_size),
            energy_history: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Add a frame's pitch and energy.
    pub fn push_frame(&mut self, pitch: f32, energy: f32) {
        self.pitch_history.push_back(pitch);
        self.energy_history.push_back(energy);
        if self.pitch_history.len() > self.window_size {
            self.pitch_history.pop_front();
        }
        if self.energy_history.len() > self.window_size {
            self.energy_history.pop_front();
        }
    }

    /// Analyze the current prosodic state.
    pub fn analyze(&self) -> ProsodyState {
        if self.energy_history.len() < 3 {
            return ProsodyState::Neutral;
        }

        let energy_trend = self.compute_trend(&self.energy_history);
        let pitch_trend = self.compute_trend(&self.pitch_history);

        // Trailing off: energy decreasing + pitch dropping
        if energy_trend < -0.001 && pitch_trend < -0.5 {
            return ProsodyState::TrailingOff;
        }

        // Rising intonation (question): pitch going up
        if pitch_trend > 2.0 {
            return ProsodyState::RisingIntonation;
        }

        // Falling intonation (statement end): pitch dropping sharply
        if pitch_trend < -3.0 {
            return ProsodyState::FallingIntonation;
        }

        // Abrupt stop: high energy then sudden silence
        let recent_energy: f32 = self.energy_history.iter().rev().take(3).sum::<f32>() / 3.0;
        let prev_energy: f32 = self
            .energy_history
            .iter()
            .rev()
            .skip(3)
            .take(3)
            .sum::<f32>()
            / 3.0;
        if prev_energy > 0.05 && recent_energy < 0.01 {
            return ProsodyState::AbruptStop;
        }

        ProsodyState::Neutral
    }

    /// Compute linear trend (slope) of a value series.
    fn compute_trend(&self, series: &VecDeque<f32>) -> f32 {
        if series.len() < 2 {
            return 0.0;
        }
        let n = series.len() as f32;
        let sum_x: f32 = (0..series.len()).map(|i| i as f32).sum();
        let sum_y: f32 = series.iter().sum();
        let sum_xy: f32 = series.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
        let sum_x2: f32 = (0..series.len()).map(|i| (i as f32).powi(2)).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-10 {
            return 0.0;
        }
        (n * sum_xy - sum_x * sum_y) / denom
    }
}

/// Prosodic state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProsodyState {
    /// Neutral speech.
    Neutral,
    /// Speaker trailing off (energy + pitch declining).
    TrailingOff,
    /// Rising intonation (question).
    RisingIntonation,
    /// Falling intonation (statement end).
    FallingIntonation,
    /// Abrupt stop (high energy → silence).
    AbruptStop,
}

impl std::fmt::Display for ProsodyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neutral => write!(f, "neutral"),
            Self::TrailingOff => write!(f, "trailing-off"),
            Self::RisingIntonation => write!(f, "rising-intonation"),
            Self::FallingIntonation => write!(f, "falling-intonation"),
            Self::AbruptStop => write!(f, "abrupt-stop"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_detector_silence() {
        let mut td = TurnDetector::new(0.01, 500, 480, 16000);
        // Feed silence frames
        for _ in 0..100 {
            let result = td.process_frame(0.001);
            assert_eq!(result, None);
        }
        assert!(!td.in_speech());
    }

    #[test]
    fn turn_detector_speech_detection() {
        let mut td = TurnDetector::new(0.01, 500, 480, 16000).with_min_speech_frames(2);
        // Feed voice frames
        for _ in 0..5 {
            td.process_frame(0.5);
        }
        assert!(td.in_speech());
    }

    #[test]
    fn turn_detector_turn_end() {
        let mut td = TurnDetector::new(0.01, 100, 480, 16000).with_min_speech_frames(1);
        // Speech
        for _ in 0..10 {
            td.process_frame(0.5);
        }
        assert!(td.in_speech());
        // Silence — should detect turn end
        let mut turned = false;
        for _ in 0..200 {
            if let Some(true) = td.process_frame(0.001) {
                turned = true;
                break;
            }
        }
        assert!(turned);
        assert!(!td.in_speech());
    }

    #[test]
    fn turn_detector_adaptive_threshold() {
        let mut td = TurnDetector::new(0.01, 200, 480, 16000).with_min_speech_frames(1);
        // Multiple speech bursts
        for _ in 0..3 {
            for _ in 0..5 {
                td.process_frame(0.5);
            }
            for _ in 0..100 {
                td.process_frame(0.001);
            }
        }
        // Threshold should have adapted
        assert!(td.adaptive_threshold() > 0);
    }

    #[test]
    fn turn_detector_reset() {
        let mut td = TurnDetector::new(0.01, 500, 480, 16000);
        for _ in 0..10 {
            td.process_frame(0.5);
        }
        td.reset();
        assert!(!td.in_speech());
        assert_eq!(td.speech_rate(), 0.0);
    }

    #[test]
    fn prosody_analyzer_neutral() {
        let pa = ProsodyAnalyzer::new(10);
        let state = pa.analyze();
        assert_eq!(state, ProsodyState::Neutral);
    }

    #[test]
    fn prosody_analyzer_trailing_off() {
        let mut pa = ProsodyAnalyzer::new(10);
        // Energy declining
        for i in 0..10 {
            let energy = 0.5 - i as f32 * 0.05;
            let pitch = 200.0 - i as f32 * 5.0;
            pa.push_frame(pitch, energy);
        }
        let state = pa.analyze();
        assert_eq!(state, ProsodyState::TrailingOff);
    }

    #[test]
    fn prosody_analyzer_rising_intonation() {
        let mut pa = ProsodyAnalyzer::new(10);
        for i in 0..10 {
            let pitch = 150.0 + i as f32 * 10.0;
            pa.push_frame(pitch, 0.3);
        }
        let state = pa.analyze();
        assert_eq!(state, ProsodyState::RisingIntonation);
    }

    #[test]
    fn prosody_analyzer_abrupt_stop() {
        let mut pa = ProsodyAnalyzer::new(10);
        // High energy then sudden silence
        for _ in 0..5 {
            pa.push_frame(200.0, 0.5);
        }
        for _ in 0..5 {
            pa.push_frame(200.0, 0.001);
        }
        let state = pa.analyze();
        assert_eq!(state, ProsodyState::AbruptStop);
    }

    #[test]
    fn prosody_analyzer_display() {
        assert_eq!(format!("{}", ProsodyState::Neutral), "neutral");
        assert_eq!(format!("{}", ProsodyState::TrailingOff), "trailing-off");
    }
}
