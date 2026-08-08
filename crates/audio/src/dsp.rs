use crate::error::{AudioError, Result};

pub trait DspProcessor: Send + Sync {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
    fn name(&self) -> &str;
    fn latency_frames(&self) -> usize;
}

pub struct GainProcessor {
    gain_db: f32,
}

impl GainProcessor {
    pub fn new(gain_db: f32) -> Self {
        Self { gain_db }
    }

    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    pub fn set_gain_db(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
    }
}

impl DspProcessor for GainProcessor {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        if input.is_empty() {
            output.clear();
            return Ok(());
        }
        let gain_linear = 10.0f32.powf(self.gain_db / 20.0);
        output.clear();
        output.reserve(input.len());
        for &sample in input {
            output.push(sample * gain_linear);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GainProcessor"
    }

    fn latency_frames(&self) -> usize {
        0
    }
}

pub struct Resampler {
    from_rate: u32,
    to_rate: u32,
}

impl Resampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        Self { from_rate, to_rate }
    }

    pub fn from_to(&self) -> (u32, u32) {
        (self.from_rate, self.to_rate)
    }
}

impl DspProcessor for Resampler {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        if input.is_empty() {
            output.clear();
            return Ok(());
        }
        if self.from_rate == 0 || self.to_rate == 0 {
            return Err(AudioError::DspError(
                "Invalid sample rate for resampling".to_string(),
            ));
        }
        if self.from_rate == self.to_rate {
            output.clear();
            output.extend_from_slice(input);
            return Ok(());
        }

        let ratio = self.to_rate as f64 / self.from_rate as f64;
        let output_len = (input.len() as f64 * ratio).round() as usize;
        output.clear();
        output.reserve(output_len);

        for i in 0..output_len {
            let src_pos_f64 = i as f64 / ratio;
            let src_pos = src_pos_f64.floor() as usize;
            let frac = src_pos_f64 - src_pos as f64;
            let sample = if frac > 0.0 {
                let i0 = src_pos.min(input.len().saturating_sub(2));
                let i1 = i0 + 1;
                input[i0] * (1.0 - frac as f32) + input[i1] * frac as f32
            } else {
                let i = src_pos.min(input.len().saturating_sub(1));
                input[i]
            };
            output.push(sample);
        }

        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Resampler"
    }

    fn latency_frames(&self) -> usize {
        0
    }
}

pub struct Normalizer {
    target_peak: f32,
}

impl Normalizer {
    pub fn new(target_peak: f32) -> Self {
        Self { target_peak }
    }
}

impl DspProcessor for Normalizer {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        if input.is_empty() {
            output.clear();
            return Ok(());
        }

        let peak = input
            .iter()
            .copied()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        if peak == 0.0 {
            output.clear();
            output.extend_from_slice(input);
            return Ok(());
        }

        let gain = self.target_peak / peak;
        output.clear();
        output.reserve(input.len());
        for &sample in input {
            output.push(sample * gain);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Normalizer"
    }

    fn latency_frames(&self) -> usize {
        0
    }
}

pub struct SilenceDetector {
    threshold: f32,
    min_silence_frames: usize,
}

impl SilenceDetector {
    pub fn new(threshold: f32, min_silence_frames: usize) -> Self {
        Self {
            threshold,
            min_silence_frames,
        }
    }

    pub fn is_silence(&self, audio: &[f32]) -> bool {
        let peak = audio
            .iter()
            .copied()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        peak < self.threshold
    }

    pub fn silence_ratio(&self, audio: &[f32]) -> f64 {
        if audio.is_empty() {
            return 1.0;
        }
        let silent_count = audio.iter().filter(|&&s| s.abs() < self.threshold).count();
        silent_count as f64 / audio.len() as f64
    }

    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    pub fn min_silence_frames(&self) -> usize {
        self.min_silence_frames
    }
}

pub struct DspChain {
    processors: Vec<Box<dyn DspProcessor>>,
}

impl DspChain {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add(&mut self, processor: Box<dyn DspProcessor>) {
        self.processors.push(processor);
    }

    pub fn process(&self, input: &[f32]) -> Result<Vec<f32>> {
        if self.processors.is_empty() {
            return Ok(input.to_vec());
        }

        let mut buf_a = input.to_vec();
        let mut buf_b = Vec::with_capacity(input.len());
        let mut cur = &mut buf_a;
        let mut nxt = &mut buf_b;
        for processor in &self.processors {
            nxt.clear();
            processor.process(cur, nxt)?;
            std::mem::swap(&mut cur, &mut nxt);
        }
        Ok(std::mem::take(cur))
    }

    pub fn reset_all(&mut self) -> Result<()> {
        for processor in &mut self.processors {
            processor.reset()?;
        }
        Ok(())
    }

    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }
}

pub struct NoiseGateProcessor {
    threshold: f32,
    _hold_frames: usize,
    hold_counter: usize,
    open: bool,
}

impl NoiseGateProcessor {
    pub fn new(threshold: f32, _hold_frames: usize) -> Self {
        Self {
            threshold,
            _hold_frames,
            hold_counter: 0,
            open: false,
        }
    }
}

impl DspProcessor for NoiseGateProcessor {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        if input.is_empty() {
            output.clear();
            return Ok(());
        }
        let peak = input
            .iter()
            .copied()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        output.clear();
        output.reserve(input.len());
        if peak >= self.threshold {
            output.extend_from_slice(input);
        } else {
            output.resize(input.len(), 0.0);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.hold_counter = 0;
        self.open = false;
        Ok(())
    }

    fn name(&self) -> &str {
        "NoiseGateProcessor"
    }

    fn latency_frames(&self) -> usize {
        0
    }
}

pub struct EchoCancellationProcessor;

impl EchoCancellationProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoCancellationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl DspProcessor for EchoCancellationProcessor {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        output.clear();
        output.extend_from_slice(input);
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "EchoCancellationProcessor"
    }

    fn latency_frames(&self) -> usize {
        0
    }
}

pub struct NoiseSuppressionProcessor {
    reduction_db: f32,
}

impl NoiseSuppressionProcessor {
    pub fn new(reduction_db: f32) -> Self {
        Self { reduction_db }
    }
}

impl DspProcessor for NoiseSuppressionProcessor {
    fn process(&self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        let gain = 10.0f32.powf(-self.reduction_db / 20.0);
        output.clear();
        output.reserve(input.len());
        for &sample in input {
            let abs = sample.abs();
            let reduced = if abs < 0.01 { sample * gain } else { sample };
            output.push(reduced);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "NoiseSuppressionProcessor"
    }

    fn latency_frames(&self) -> usize {
        0
    }
}

impl Default for DspChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_processor_identity() {
        let processor = GainProcessor::new(0.0);
        let input = vec![0.5, -0.5, 0.25, -0.75];
        let mut output = Vec::new();
        processor.process(&input, &mut output).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_gain_processor_positive_gain() {
        let processor = GainProcessor::new(6.0);
        let input = vec![0.5, -0.5];
        let mut output = Vec::new();
        processor.process(&input, &mut output).unwrap();
        let expected_gain = 10.0f32.powf(6.0 / 20.0);
        assert!((output[0] - 0.5 * expected_gain).abs() < 1e-6);
        assert!((output[1] - (-0.5 * expected_gain)).abs() < 1e-6);
    }

    #[test]
    fn test_gain_processor_negative_gain() {
        let processor = GainProcessor::new(-6.0);
        let input = vec![0.5, -0.5];
        let mut output = Vec::new();
        processor.process(&input, &mut output).unwrap();
        let expected_gain = 10.0f32.powf(-6.0 / 20.0);
        assert!((output[0] - 0.5 * expected_gain).abs() < 1e-6);
    }

    #[test]
    fn test_gain_processor_empty_input() {
        let processor = GainProcessor::new(3.0);
        let mut output = vec![1.0, 2.0];
        processor.process(&[], &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_gain_processor_name() {
        let processor = GainProcessor::new(0.0);
        assert_eq!(processor.name(), "GainProcessor");
    }

    #[test]
    fn test_gain_processor_latency() {
        let processor = GainProcessor::new(0.0);
        assert_eq!(processor.latency_frames(), 0);
    }

    #[test]
    fn test_normalizer_basic() {
        let normalizer = Normalizer::new(1.0);
        let input = vec![0.5, -0.25, 0.75, -0.1];
        let mut output = Vec::new();
        normalizer.process(&input, &mut output).unwrap();

        let peak = output
            .iter()
            .copied()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        assert!((peak - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalizer_silence() {
        let normalizer = Normalizer::new(1.0);
        let input = vec![0.0; 10];
        let mut output = Vec::new();
        normalizer.process(&input, &mut output).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_normalizer_empty() {
        let normalizer = Normalizer::new(1.0);
        let mut output = vec![1.0, 2.0];
        normalizer.process(&[], &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_resampler_identity() {
        let resampler = Resampler::new(44100, 44100);
        let input = vec![0.1, 0.2, 0.3];
        let mut output = Vec::new();
        resampler.process(&input, &mut output).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_resampler_upsample() {
        let resampler = Resampler::new(16000, 48000);
        let input = vec![0.1, 0.2, 0.3];
        let mut output = Vec::new();
        resampler.process(&input, &mut output).unwrap();
        assert_eq!(output.len(), 9);
        assert!((output[0] - 0.1).abs() < 1e-6);
        assert!((output[3] - 0.2).abs() < 1e-6);
        assert!((output[6] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_resampler_downsample() {
        let resampler = Resampler::new(48000, 16000);
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let mut output = Vec::new();
        resampler.process(&input, &mut output).unwrap();
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_resampler_empty() {
        let resampler = Resampler::new(16000, 48000);
        let mut output = vec![1.0, 2.0];
        resampler.process(&[], &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_resampler_invalid_rate() {
        let resampler = Resampler::new(0, 48000);
        let mut output = Vec::new();
        let result = resampler.process(&[0.1], &mut output);
        assert!(result.is_err());
    }

    #[test]
    fn test_silence_detector_is_silence() {
        let detector = SilenceDetector::new(0.01, 100);
        assert!(detector.is_silence(&[0.0, 0.0, 0.0]));
        assert!(!detector.is_silence(&[0.5, 0.0, 0.0]));
    }

    #[test]
    fn test_silence_detector_silence_ratio() {
        let detector = SilenceDetector::new(0.1, 100);
        let audio = vec![0.0, 0.05, 0.5, 0.0, 0.2];
        let ratio = detector.silence_ratio(&audio);
        assert!((ratio - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_silence_detector_empty() {
        let detector = SilenceDetector::new(0.01, 100);
        assert_eq!(detector.silence_ratio(&[]), 1.0);
    }

    #[test]
    fn test_dsp_chain_empty() {
        let chain = DspChain::new();
        let input = vec![0.5, -0.5];
        let output = chain.process(&input).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_dsp_chain_single_processor() {
        let mut chain = DspChain::new();
        chain.add(Box::new(GainProcessor::new(6.0)));
        let input = vec![0.5, -0.5];
        let output = chain.process(&input).unwrap();
        let expected_gain = 10.0f32.powf(6.0 / 20.0);
        assert!((output[0] - 0.5 * expected_gain).abs() < 1e-6);
    }

    #[test]
    fn test_dsp_chain_multiple_processors() {
        let mut chain = DspChain::new();
        chain.add(Box::new(GainProcessor::new(6.0)));
        chain.add(Box::new(Normalizer::new(1.0)));
        let input = vec![0.5, -0.25, 0.75];
        let output = chain.process(&input).unwrap();
        let peak = output
            .iter()
            .copied()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        assert!((peak - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dsp_chain_processor_count() {
        let mut chain = DspChain::new();
        assert_eq!(chain.processor_count(), 0);
        chain.add(Box::new(GainProcessor::new(0.0)));
        assert_eq!(chain.processor_count(), 1);
        chain.add(Box::new(Normalizer::new(1.0)));
        assert_eq!(chain.processor_count(), 2);
    }

    #[test]
    fn test_dsp_chain_reset_all() {
        let mut chain = DspChain::new();
        chain.add(Box::new(GainProcessor::new(3.0)));
        chain.reset_all().unwrap();
        assert_eq!(chain.processor_count(), 1);
    }
}
