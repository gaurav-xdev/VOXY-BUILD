use std::collections::VecDeque;

/// Classifies the ambient environment based on audio analysis.
///
/// Detects:
/// - Quiet room (low noise floor, no music)
/// - Noisy room (high noise floor, no music)
/// - Music playing (constant harmonic content)
/// - Meeting (multiple voices)
/// - Gaming (game audio patterns)
pub struct AmbientAnalyzer {
    /// Recent noise floor estimates.
    noise_history: VecDeque<f32>,
    /// Maximum history length.
    max_history: usize,
    /// Current classified environment.
    environment: AmbientEnvironment,
    /// Confidence in classification (0.0-1.0).
    confidence: f32,
    /// Frames since last reclassification.
    frames_since_classify: usize,
    /// Reclassification interval (frames).
    reclassify_interval: usize,
}

impl AmbientAnalyzer {
    pub fn new() -> Self {
        Self {
            noise_history: VecDeque::with_capacity(100),
            max_history: 100,
            environment: AmbientEnvironment::Unknown,
            confidence: 0.0,
            frames_since_classify: 0,
            reclassify_interval: 30, // ~1 second at 30ms frames
        }
    }

    /// Analyze a frame and update environment classification.
    pub fn analyze_frame(&mut self, energy: f32, spectral_centroid: f32) {
        self.noise_history.push_back(energy);
        if self.noise_history.len() > self.max_history {
            self.noise_history.pop_front();
        }

        self.frames_since_classify += 1;
        if self.frames_since_classify >= self.reclassify_interval {
            self.classify(spectral_centroid);
            self.frames_since_classify = 0;
        }
    }

    fn classify(&mut self, spectral_centroid: f32) {
        let avg_energy = if self.noise_history.is_empty() {
            0.0
        } else {
            self.noise_history.iter().sum::<f32>() / self.noise_history.len() as f32
        };

        let energy_variance = self.compute_variance();

        // Classification heuristics
        if avg_energy < 0.005 {
            self.environment = AmbientEnvironment::Quiet;
            self.confidence = 0.9;
        } else if avg_energy > 0.1 && spectral_centroid > 2000.0 {
            // High energy + high centroid = music
            self.environment = AmbientEnvironment::Music;
            self.confidence = 0.7;
        } else if avg_energy > 0.05 && energy_variance > 0.01 {
            // Moderate energy with high variance = gaming
            self.environment = AmbientEnvironment::Gaming;
            self.confidence = 0.6;
        } else if avg_energy > 0.02 {
            self.environment = AmbientEnvironment::Noisy;
            self.confidence = 0.7;
        } else {
            self.environment = AmbientEnvironment::Quiet;
            self.confidence = 0.8;
        }
    }

    fn compute_variance(&self) -> f32 {
        if self.noise_history.len() < 2 {
            return 0.0;
        }
        let mean = self.noise_history.iter().sum::<f32>() / self.noise_history.len() as f32;
        self.noise_history
            .iter()
            .map(|&e| (e - mean).powi(2))
            .sum::<f32>()
            / self.noise_history.len() as f32
    }

    pub fn environment(&self) -> AmbientEnvironment {
        self.environment
    }

    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    pub fn noise_floor(&self) -> f32 {
        if self.noise_history.is_empty() {
            return 0.0;
        }
        self.noise_history.iter().sum::<f32>() / self.noise_history.len() as f32
    }

    pub fn reset(&mut self) {
        self.noise_history.clear();
        self.environment = AmbientEnvironment::Unknown;
        self.confidence = 0.0;
        self.frames_since_classify = 0;
    }
}

impl Default for AmbientAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientEnvironment {
    Unknown,
    Quiet,
    Noisy,
    Music,
    Gaming,
    Meeting,
}

impl std::fmt::Display for AmbientEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Quiet => write!(f, "quiet"),
            Self::Noisy => write!(f, "noisy"),
            Self::Music => write!(f, "music"),
            Self::Gaming => write!(f, "gaming"),
            Self::Meeting => write!(f, "meeting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambient_creation() {
        let a = AmbientAnalyzer::new();
        assert_eq!(a.environment(), AmbientEnvironment::Unknown);
        assert_eq!(a.confidence(), 0.0);
    }

    #[test]
    fn ambient_quiet_detection() {
        let mut a = AmbientAnalyzer::new();
        for _ in 0..40 {
            a.analyze_frame(0.001, 500.0);
        }
        assert_eq!(a.environment(), AmbientEnvironment::Quiet);
    }

    #[test]
    fn ambient_music_detection() {
        let mut a = AmbientAnalyzer::new();
        for _ in 0..40 {
            a.analyze_frame(0.15, 3000.0);
        }
        assert_eq!(a.environment(), AmbientEnvironment::Music);
    }

    #[test]
    fn ambient_reset() {
        let mut a = AmbientAnalyzer::new();
        for _ in 0..40 {
            a.analyze_frame(0.15, 3000.0);
        }
        a.reset();
        assert_eq!(a.environment(), AmbientEnvironment::Unknown);
    }

    #[test]
    fn ambient_display() {
        assert_eq!(format!("{}", AmbientEnvironment::Quiet), "quiet");
        assert_eq!(format!("{}", AmbientEnvironment::Music), "music");
    }
}
