use std::collections::VecDeque;

/// Predicts latency for each pipeline stage using rolling averages.
///
/// Used to:
/// - Show estimated wait time to user
/// - Adapt quality mode when latency is predicted to be high
/// - Detect latency degradation early
pub struct LatencyPredictor {
    /// Recent STT latencies (ms).
    stt_history: VecDeque<f32>,
    /// Recent LLM latencies (ms).
    llm_history: VecDeque<f32>,
    /// Recent TTS latencies (ms).
    tts_history: VecDeque<f32>,
    /// Maximum history length.
    max_history: usize,
    /// Percentile to use for prediction (0.0-1.0). 0.9 = 90th percentile.
    percentile: f32,
}

impl LatencyPredictor {
    pub fn new() -> Self {
        Self {
            stt_history: VecDeque::with_capacity(50),
            llm_history: VecDeque::with_capacity(50),
            tts_history: VecDeque::with_capacity(50),
            max_history: 50,
            percentile: 0.9,
        }
    }

    pub fn with_percentile(mut self, percentile: f32) -> Self {
        self.percentile = percentile.clamp(0.5, 0.99);
        self
    }

    /// Record a latency sample.
    pub fn record_stt(&mut self, ms: f32) {
        self.stt_history.push_back(ms);
        if self.stt_history.len() > self.max_history {
            self.stt_history.pop_front();
        }
    }

    pub fn record_llm(&mut self, ms: f32) {
        self.llm_history.push_back(ms);
        if self.llm_history.len() > self.max_history {
            self.llm_history.pop_front();
        }
    }

    pub fn record_tts(&mut self, ms: f32) {
        self.tts_history.push_back(ms);
        if self.tts_history.len() > self.max_history {
            self.tts_history.pop_front();
        }
    }

    /// Predict next latency for a stage (percentile of recent history).
    pub fn predict_stt(&self) -> f32 {
        self.predict_from_history(&self.stt_history)
    }

    pub fn predict_llm(&self) -> f32 {
        self.predict_from_history(&self.llm_history)
    }

    pub fn predict_tts(&self) -> f32 {
        self.predict_from_history(&self.tts_history)
    }

    /// Predict total end-to-end latency.
    pub fn predict_e2e(&self) -> f32 {
        self.predict_stt() + self.predict_llm() + self.predict_tts()
    }

    /// Get average latency for each stage.
    pub fn averages(&self) -> LatencyAverages {
        LatencyAverages {
            stt_avg: self.average(&self.stt_history),
            llm_avg: self.average(&self.llm_history),
            tts_avg: self.average(&self.tts_history),
            e2e_avg: self.average(&self.stt_history)
                + self.average(&self.llm_history)
                + self.average(&self.tts_history),
        }
    }

    /// Check if latency is degrading (recent > 1.5x historical average).
    pub fn is_degrading(&self) -> LatencyDegradation {
        LatencyDegradation {
            stt: self.is_stage_degrading(&self.stt_history),
            llm: self.is_stage_degrading(&self.llm_history),
            tts: self.is_stage_degrading(&self.tts_history),
        }
    }

    fn predict_from_history(&self, history: &VecDeque<f32>) -> f32 {
        if history.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f32> = history.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((sorted.len() - 1) as f32 * self.percentile) as usize;
        sorted[idx]
    }

    fn average(&self, history: &VecDeque<f32>) -> f32 {
        if history.is_empty() {
            return 0.0;
        }
        history.iter().sum::<f32>() / history.len() as f32
    }

    fn is_stage_degrading(&self, history: &VecDeque<f32>) -> bool {
        if history.len() < 10 {
            return false;
        }
        let recent_avg: f32 = history.iter().rev().take(5).sum::<f32>() / 5.0;
        let historical_avg: f32 =
            history.iter().take(history.len() - 5).sum::<f32>() / (history.len() - 5).max(1) as f32;
        recent_avg > historical_avg * 1.5
    }

    pub fn reset(&mut self) {
        self.stt_history.clear();
        self.llm_history.clear();
        self.tts_history.clear();
    }
}

impl Default for LatencyPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct LatencyAverages {
    pub stt_avg: f32,
    pub llm_avg: f32,
    pub tts_avg: f32,
    pub e2e_avg: f32,
}

#[derive(Debug, Clone, Default)]
pub struct LatencyDegradation {
    pub stt: bool,
    pub llm: bool,
    pub tts: bool,
}

impl LatencyDegradation {
    pub fn any(&self) -> bool {
        self.stt || self.llm || self.tts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predictor_creation() {
        let p = LatencyPredictor::new();
        assert_eq!(p.predict_stt(), 0.0);
    }

    #[test]
    fn predictor_record_and_predict() {
        let mut p = LatencyPredictor::new();
        for i in 0..20 {
            p.record_stt(i as f32 * 10.0);
        }
        let predicted = p.predict_stt();
        assert!(predicted > 0.0);
    }

    #[test]
    fn predictor_averages() {
        let mut p = LatencyPredictor::new();
        p.record_stt(100.0);
        p.record_stt(200.0);
        let avg = p.averages();
        assert!((avg.stt_avg - 150.0).abs() < 1.0);
    }

    #[test]
    fn predictor_e2e() {
        let mut p = LatencyPredictor::new();
        p.record_stt(100.0);
        p.record_llm(500.0);
        p.record_tts(200.0);
        let e2e = p.predict_e2e();
        assert!(e2e > 0.0);
    }

    #[test]
    fn predictor_degradation() {
        let mut p = LatencyPredictor::new();
        // Historical low latency
        for _ in 0..15 {
            p.record_stt(50.0);
        }
        // Recent high latency
        for _ in 0..5 {
            p.record_stt(200.0);
        }
        let degradation = p.is_degrading();
        assert!(degradation.stt);
    }

    #[test]
    fn predictor_no_degradation() {
        let mut p = LatencyPredictor::new();
        for _ in 0..20 {
            p.record_stt(100.0);
        }
        let degradation = p.is_degrading();
        assert!(!degradation.stt);
    }

    #[test]
    fn predictor_reset() {
        let mut p = LatencyPredictor::new();
        p.record_stt(100.0);
        p.record_llm(200.0);
        p.reset();
        assert_eq!(p.predict_stt(), 0.0);
    }

    #[test]
    fn predictor_percentile() {
        let mut p = LatencyPredictor::new().with_percentile(0.5);
        for i in 0..30 {
            p.record_stt(i as f32);
        }
        let predicted = p.predict_stt();
        // 50th percentile of 0..29 should be around 14-15
        assert!(
            predicted > 10.0 && predicted < 20.0,
            "predicted: {predicted}"
        );
    }
}
