use crate::config::ConfidenceConfig;
use crate::types::{ConfidenceLevel, ConfidenceOutput};

/// Confidence engine — calculates internal confidence for responses.
pub struct ConfidenceEngine {
    config: ConfidenceConfig,
    history: Vec<f64>,
    max_history: usize,
}

impl ConfidenceEngine {
    pub fn new(config: ConfidenceConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Calculate confidence output from raw score.
    pub fn calculate(
        &mut self,
        raw_score: f64,
        context_clarity: f64,
        data_quality: f64,
    ) -> ConfidenceOutput {
        let adjusted =
            (raw_score * 0.5 + context_clarity * 0.3 + data_quality * 0.2).clamp(0.0, 1.0);

        let level = ConfidenceLevel::from_score(adjusted);
        let should_explain = level.should_explain();
        let explanation_depth = if should_explain {
            (1.0 - adjusted) * self.config.max_explanation_depth
        } else {
            0.0
        };

        self.history.push(adjusted);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        ConfidenceOutput {
            score: adjusted,
            level,
            should_explain,
            explanation_depth,
        }
    }

    /// Get average confidence over history.
    pub fn average_confidence(&self) -> f64 {
        if self.history.is_empty() {
            return 0.5;
        }
        let sum: f64 = self.history.iter().sum();
        sum / self.history.len() as f64
    }

    /// Check if confidence trend is improving.
    pub fn trend(&self) -> f64 {
        if self.history.len() < 10 {
            return 0.0;
        }
        let mid = self.history.len() / 2;
        let first_half: f64 = self.history[..mid].iter().sum::<f64>() / mid as f64;
        let second_half: f64 =
            self.history[mid..].iter().sum::<f64>() / (self.history.len() - mid) as f64;
        second_half - first_half
    }
}

impl Default for ConfidenceEngine {
    fn default() -> Self {
        Self::new(ConfidenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_confidence() {
        let mut engine = ConfidenceEngine::new(ConfidenceConfig::default());
        let output = engine.calculate(0.9, 0.9, 0.9);
        assert_eq!(output.level, ConfidenceLevel::VeryHigh);
        assert!(!output.should_explain);
    }

    #[test]
    fn test_low_confidence() {
        let mut engine = ConfidenceEngine::new(ConfidenceConfig::default());
        let output = engine.calculate(0.2, 0.3, 0.2);
        assert!(output.level as u8 <= ConfidenceLevel::Low as u8);
        assert!(output.should_explain);
    }

    #[test]
    fn test_average_confidence() {
        let mut engine = ConfidenceEngine::new(ConfidenceConfig::default());
        for _ in 0..10 {
            engine.calculate(0.8, 0.8, 0.8);
        }
        assert!(engine.average_confidence() > 0.7);
    }

    #[test]
    fn test_trend() {
        let mut engine = ConfidenceEngine::new(ConfidenceConfig::default());
        for _ in 0..20 {
            engine.calculate(0.3, 0.3, 0.3);
        }
        let trend = engine.trend();
        assert!(trend.abs() < 0.1);
    }
}
