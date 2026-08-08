use crate::error::Result;

/// GPU compute backend for DSP operations.
/// Falls back to CPU when GPU is unavailable.
pub enum GpuDspBackend {
    /// CPU-only fallback (always available).
    Cpu,
    /// WGPU compute shader backend (when GPU is available).
    #[cfg(feature = "gpu-dsp")]
    Wgpu(wgpu::Device, wgpu::Queue),
}

impl GpuDspBackend {
    /// Create the best available backend.
    pub async fn create() -> Self {
        #[cfg(feature = "gpu-dsp")]
        {
            match Self::try_create_wgpu().await {
                Some((device, queue)) => {
                    tracing::info!("GPU DSP backend initialized via wgpu");
                    Self::Wgpu(device, queue)
                }
                None => {
                    tracing::info!("GPU DSP not available, using CPU fallback");
                    Self::Cpu
                }
            }
        }
        #[cfg(not(feature = "gpu-dsp"))]
        {
            Self::Cpu
        }
    }

    #[cfg(feature = "gpu-dsp")]
    async fn try_create_wgpu() -> Option<(wgpu::Device, wgpu::Queue)> {
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("voxy-dsp"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            })
            .ok()?;

        Some((device, queue))
    }

    pub fn is_gpu(&self) -> bool {
        #[cfg(feature = "gpu-dsp")]
        {
            matches!(self, Self::Wgpu(..))
        }
        #[cfg(not(feature = "gpu-dsp"))]
        {
            false
        }
    }

    pub fn name(&self) -> &'static str {
        #[cfg(feature = "gpu-dsp")]
        {
            match self {
                Self::Cpu => "cpu",
                Self::Wgpu(..) => "wgpu",
            }
        }
        #[cfg(not(feature = "gpu-dsp"))]
        {
            "cpu"
        }
    }
}

/// Noise suppression processor with optional GPU acceleration.
pub struct AdaptiveNoiseSuppressor {
    /// Noise floor estimate in linear scale.
    noise_floor: f32,
    /// Smoothing factor for noise floor adaptation (0.0-1.0).
    adaptation_rate: f32,
    /// Current estimated noise spectrum (simplified: energy per band).
    noise_estimate: Vec<f32>,
    /// Number of frequency bands.
    num_bands: usize,
    /// Minimum suppression in dB.
    min_suppression_db: f32,
    /// Maximum suppression in dB.
    max_suppression_db: f32,
}

impl AdaptiveNoiseSuppressor {
    pub fn new(_sample_rate: u32) -> Self {
        let num_bands = 32;
        Self {
            noise_floor: 0.001,
            adaptation_rate: 0.01,
            noise_estimate: vec![0.001; num_bands],
            num_bands,
            min_suppression_db: 6.0,
            max_suppression_db: 30.0,
        }
    }

    pub fn with_adaptation_rate(mut self, rate: f32) -> Self {
        self.adaptation_rate = rate.clamp(0.001, 0.5);
        self
    }

    pub fn with_suppression_range(mut self, min_db: f32, max_db: f32) -> Self {
        self.min_suppression_db = min_db;
        self.max_suppression_db = max_db;
        self
    }

    /// Process audio with adaptive noise suppression.
    /// Uses a simplified sub-band approach: estimates noise per band,
    /// applies spectral subtraction.
    pub fn process(&mut self, input: &[f32], output: &mut Vec<f32>) -> Result<()> {
        if input.is_empty() {
            output.clear();
            return Ok(());
        }

        output.clear();
        output.reserve(input.len());

        // Simple energy-based noise gate with smoothing
        // In production, this would use FFT for real spectral subtraction
        let block_size = (input.len() / self.num_bands).max(1);

        for (i, &sample) in input.iter().enumerate() {
            let band_idx = (i / block_size).min(self.num_bands - 1);
            let abs_sample = sample.abs();

            // Update noise estimate for this band (adapt to stationary noise)
            if abs_sample < self.noise_estimate[band_idx] * 2.0 {
                self.noise_estimate[band_idx] = self.noise_estimate[band_idx]
                    * (1.0 - self.adaptation_rate)
                    + abs_sample * self.adaptation_rate;
            }

            // Compute suppression needed
            let noise_level = self.noise_estimate[band_idx];
            let signal_to_noise = if noise_level > 0.0 {
                abs_sample / noise_level
            } else {
                100.0
            };

            // Apply suppression: more suppression when SNR is low
            let suppression_linear = if signal_to_noise < 2.0 {
                let suppression_db = self.min_suppression_db
                    + (self.max_suppression_db - self.min_suppression_db)
                        * (1.0 - signal_to_noise / 2.0);
                10.0f32.powf(-suppression_db / 20.0)
            } else {
                1.0
            };

            output.push(sample * suppression_linear);
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.noise_estimate.fill(self.noise_floor);
    }

    pub fn noise_floor(&self) -> f32 {
        self.noise_floor
    }

    pub fn noise_estimate(&self) -> &[f32] {
        &self.noise_estimate
    }
}

/// Spectral echo canceller using normalized cross-correlation.
pub struct SpectralEchoCanceller {
    /// Delay line: stores recent reference signal.
    delay_line: Vec<f32>,
    /// Maximum echo delay in samples.
    max_delay_samples: usize,
    /// Adaptive filter coefficients.
    filter_coeffs: Vec<f32>,
    /// Filter length.
    filter_length: usize,
    /// Step size for LMS adaptation.
    step_size: f32,
    /// Leak factor for coefficient update.
    leak_factor: f32,
    /// Current write position in delay line.
    write_pos: usize,
}

impl SpectralEchoCanceller {
    pub fn new(sample_rate: u32, max_delay_ms: u32) -> Self {
        let max_delay_samples = (sample_rate as u64 * max_delay_ms as u64 / 1000) as usize;
        let filter_length = (max_delay_samples / 4).max(64);
        Self {
            delay_line: vec![0.0; max_delay_samples + filter_length],
            max_delay_samples,
            filter_coeffs: vec![0.0; filter_length],
            filter_length,
            step_size: 0.01,
            leak_factor: 0.9999,
            write_pos: 0,
        }
    }

    pub fn with_step_size(mut self, step: f32) -> Self {
        self.step_size = step.clamp(0.0001, 0.1);
        self
    }

    /// Process input (mic signal) with echo cancellation.
    /// `reference` is the signal being played through speakers.
    pub fn process(
        &mut self,
        input: &[f32],
        reference: &[f32],
        output: &mut Vec<f32>,
    ) -> Result<()> {
        if input.is_empty() {
            output.clear();
            return Ok(());
        }

        output.clear();
        output.reserve(input.len());

        // Write reference into delay line
        for &sample in reference.iter().take(self.delay_line.len()) {
            self.delay_line[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.delay_line.len();
        }

        // Process each input sample
        for &mic_sample in input.iter() {
            // Compute echo estimate using adaptive filter
            let mut echo_estimate = 0.0f32;
            for j in 0..self.filter_length {
                let delay_pos =
                    (self.write_pos + self.delay_line.len() - self.max_delay_samples - j
                        + self.delay_line.len())
                        % self.delay_line.len();
                echo_estimate += self.filter_coeffs[j] * self.delay_line[delay_pos];
            }

            // Error signal = mic - echo estimate
            let error = mic_sample - echo_estimate;

            // Update filter coefficients (NLMS)
            let power: f32 = (0..self.filter_length)
                .map(|j| {
                    let delay_pos =
                        (self.write_pos + self.delay_line.len() - self.max_delay_samples - j
                            + self.delay_line.len())
                            % self.delay_line.len();
                    self.delay_line[delay_pos].powi(2)
                })
                .sum();
            let norm = power + 1e-10;

            for j in 0..self.filter_length {
                let delay_pos =
                    (self.write_pos + self.delay_line.len() - self.max_delay_samples - j
                        + self.delay_line.len())
                        % self.delay_line.len();
                self.filter_coeffs[j] = self.filter_coeffs[j] * self.leak_factor
                    + self.step_size * error * self.delay_line[delay_pos] / norm;
            }

            output.push(error);
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.delay_line.fill(0.0);
        self.filter_coeffs.fill(0.0);
        self.write_pos = 0;
    }

    pub fn echo_level_db(&self) -> f32 {
        let power: f32 = self.filter_coeffs.iter().map(|c| c * c).sum();
        if power > 0.0 {
            10.0 * power.log10()
        } else {
            -100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_backend_creation_cpu() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let backend = rt.block_on(GpuDspBackend::create());
        assert_eq!(backend.name(), "cpu");
        assert!(!backend.is_gpu());
    }

    #[test]
    fn noise_suppressor_creation() {
        let ns = AdaptiveNoiseSuppressor::new(16000);
        assert_eq!(ns.noise_floor(), 0.001);
    }

    #[test]
    fn noise_suppressor_passthrough_loud_signal() {
        let mut ns = AdaptiveNoiseSuppressor::new(16000);
        let input: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let mut output = Vec::new();
        ns.process(&input, &mut output).unwrap();
        assert_eq!(output.len(), input.len());
        // Loud signal should pass through mostly unchanged
        let input_rms = compute_rms(&input);
        let output_rms = compute_rms(&output);
        assert!(output_rms > input_rms * 0.3);
    }

    #[test]
    fn noise_suppressor_reduces_quiet_signal() {
        let mut ns = AdaptiveNoiseSuppressor::new(16000).with_suppression_range(10.0, 30.0);
        // Feed noise floor first
        for _ in 0..100 {
            let noise: Vec<f32> = (0..480).map(|_| rand_f32() * 0.001).collect();
            let mut out = Vec::new();
            ns.process(&noise, &mut out).unwrap();
        }
        // Now very quiet signal should be suppressed
        let quiet: Vec<f32> = vec![0.0001; 480];
        let mut output = Vec::new();
        ns.process(&quiet, &mut output).unwrap();
        let output_rms = compute_rms(&output);
        assert!(output_rms < 0.0001);
    }

    #[test]
    fn noise_suppressor_reset() {
        let mut ns = AdaptiveNoiseSuppressor::new(16000);
        let input = vec![0.5; 480];
        let mut output = Vec::new();
        ns.process(&input, &mut output).unwrap();
        ns.reset();
        assert!(ns
            .noise_estimate()
            .iter()
            .all(|&x| (x - 0.001).abs() < 1e-6));
    }

    #[test]
    fn echo_canceller_creation() {
        let ec = SpectralEchoCanceller::new(16000, 200);
        assert_eq!(ec.echo_level_db(), -100.0);
    }

    #[test]
    fn echo_canceller_removes_echo() {
        let mut ec = SpectralEchoCanceller::new(16000, 200);
        // Run multiple blocks so NLMS filter converges
        for _ in 0..20 {
            let reference: Vec<f32> = (0..480).map(|i| (i as f32 * 0.05).sin() * 0.3).collect();
            let mic: Vec<f32> = reference.iter().map(|&r| r * 0.8 + 0.1).collect();
            let mut output = Vec::new();
            ec.process(&mic, &reference, &mut output).unwrap();
            assert_eq!(output.len(), mic.len());
        }

        // After convergence, measure on a fresh block
        let reference: Vec<f32> = (0..480).map(|i| (i as f32 * 0.05).sin() * 0.3).collect();
        let mic: Vec<f32> = reference.iter().map(|&r| r * 0.8 + 0.1).collect();
        let mut output = Vec::new();
        ec.process(&mic, &reference, &mut output).unwrap();

        let input_rms = compute_rms(&mic);
        let output_rms = compute_rms(&output);
        // Output should differ from input (echo removed)
        assert!(input_rms > 0.1, "input_rms too low: {input_rms}");
        assert!(
            output_rms < input_rms,
            "output_rms {output_rms} should be < input_rms {input_rms}"
        );
    }

    #[test]
    fn echo_canceller_reset() {
        let mut ec = SpectralEchoCanceller::new(16000, 100);
        let reference = vec![0.1; 480];
        let mic = vec![0.2; 480];
        let mut output = Vec::new();
        ec.process(&mic, &reference, &mut output).unwrap();
        ec.reset();
        assert_eq!(ec.echo_level_db(), -100.0);
    }

    fn compute_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = samples.iter().map(|&s| s as f64 * s as f64).sum();
        (sum / samples.len() as f64).sqrt() as f32
    }

    fn rand_f32() -> f32 {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let s = RandomState::new();
        let mut hasher = s.build_hasher();
        hasher.write_u64(0);
        (hasher.finish() % 10000) as f32 / 10000.0
    }
}
