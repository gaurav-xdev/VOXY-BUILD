use std::collections::HashMap;
use std::time::Instant;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::calibration::CalibrationProfile;
use crate::diagnostics::AudioMetrics;
use crate::mixer::{ChannelState, MixerChannel};

/// Real-time metrics for the voice engine owner dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceEngineMetrics {
    pub latency: LatencyMetrics,
    pub audio: AudioQualityMetrics,
    pub system: SystemMetrics,
    pub detection: DetectionMetrics,
    pub channels: HashMap<String, ChannelMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub wake_detection_ms: f64,
    pub stt_first_token_ms: f64,
    pub llm_first_token_ms: f64,
    pub tts_first_chunk_ms: f64,
    pub end_to_end_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityMetrics {
    pub noise_percent: f64,
    pub echo_reduction_db: f64,
    pub dropped_frames: u64,
    pub total_frames: u64,
    pub clipping_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub gpu_percent: f64,
    pub thread_count: usize,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionMetrics {
    pub wake_word_accuracy: f64,
    pub false_positives: u64,
    pub true_positives: u64,
    pub false_negatives: u64,
    pub interruption_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetrics {
    pub gain_db: f32,
    pub duck_amount_db: f32,
    pub is_muted: bool,
    pub priority: String,
}

/// Voice engine metrics collector.
pub struct MetricsCollector {
    start_time: Instant,
    wake_detection_latencies: RwLock<Vec<f64>>,
    stt_latencies: RwLock<Vec<f64>>,
    llm_latencies: RwLock<Vec<f64>>,
    tts_latencies: RwLock<Vec<f64>>,
    false_positives: RwLock<u64>,
    true_positives: RwLock<u64>,
    false_negatives: RwLock<u64>,
    interruptions: RwLock<u64>,
    dropped_frames: RwLock<u64>,
    clipping_count: RwLock<u64>,
    cpu_samples: RwLock<Vec<f64>>,
    memory_samples: RwLock<Vec<f64>>,
    gpu_samples: RwLock<Vec<f64>>,
    noise_samples: RwLock<Vec<f64>>,
    max_samples: usize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            wake_detection_latencies: RwLock::new(Vec::with_capacity(1000)),
            stt_latencies: RwLock::new(Vec::with_capacity(1000)),
            llm_latencies: RwLock::new(Vec::with_capacity(1000)),
            tts_latencies: RwLock::new(Vec::with_capacity(1000)),
            false_positives: RwLock::new(0),
            true_positives: RwLock::new(0),
            false_negatives: RwLock::new(0),
            interruptions: RwLock::new(0),
            dropped_frames: RwLock::new(0),
            clipping_count: RwLock::new(0),
            cpu_samples: RwLock::new(Vec::with_capacity(1000)),
            memory_samples: RwLock::new(Vec::with_capacity(1000)),
            gpu_samples: RwLock::new(Vec::with_capacity(1000)),
            noise_samples: RwLock::new(Vec::with_capacity(1000)),
            max_samples: 1000,
        }
    }

    pub fn record_wake_latency(&self, ms: f64) {
        let mut samples = self.wake_detection_latencies.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(ms);
    }

    pub fn record_stt_latency(&self, ms: f64) {
        let mut samples = self.stt_latencies.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(ms);
    }

    pub fn record_llm_latency(&self, ms: f64) {
        let mut samples = self.llm_latencies.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(ms);
    }

    pub fn record_tts_latency(&self, ms: f64) {
        let mut samples = self.tts_latencies.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(ms);
    }

    pub fn record_false_positive(&self) {
        *self.false_positives.write() += 1;
    }

    pub fn record_true_positive(&self) {
        *self.true_positives.write() += 1;
    }

    pub fn record_false_negative(&self) {
        *self.false_negatives.write() += 1;
    }

    pub fn record_interruption(&self) {
        *self.interruptions.write() += 1;
    }

    pub fn record_dropped_frame(&self) {
        *self.dropped_frames.write() += 1;
    }

    pub fn record_clipping(&self) {
        *self.clipping_count.write() += 1;
    }

    pub fn record_cpu(&self, percent: f64) {
        let mut samples = self.cpu_samples.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(percent);
    }

    pub fn record_memory(&self, mb: f64) {
        let mut samples = self.memory_samples.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(mb);
    }

    pub fn record_gpu(&self, percent: f64) {
        let mut samples = self.gpu_samples.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(percent);
    }

    pub fn record_noise(&self, percent: f64) {
        let mut samples = self.noise_samples.write();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(percent);
    }

    /// Quick snapshot of just latency metrics (no full collect needed).
    pub fn latency_snapshot(&self) -> LatencyMetrics {
        LatencyMetrics {
            wake_detection_ms: avg_or_zero(&self.wake_detection_latencies.read()),
            stt_first_token_ms: avg_or_zero(&self.stt_latencies.read()),
            llm_first_token_ms: avg_or_zero(&self.llm_latencies.read()),
            tts_first_chunk_ms: avg_or_zero(&self.tts_latencies.read()),
            end_to_end_ms: avg_or_zero(&self.wake_detection_latencies.read())
                + avg_or_zero(&self.stt_latencies.read())
                + avg_or_zero(&self.llm_latencies.read())
                + avg_or_zero(&self.tts_latencies.read()),
        }
    }

    /// Uptime in seconds.
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Collect all metrics into a snapshot.
    pub fn collect(
        &self,
        audio_metrics: &AudioMetrics,
        mixer_channels: &HashMap<MixerChannel, ChannelState>,
        calibration: &CalibrationProfile,
    ) -> VoiceEngineMetrics {
        let wake = avg_or_zero(&self.wake_detection_latencies.read());
        let stt = avg_or_zero(&self.stt_latencies.read());
        let llm = avg_or_zero(&self.llm_latencies.read());
        let tts = avg_or_zero(&self.tts_latencies.read());

        let cpu = avg_or_zero(&self.cpu_samples.read());
        let mem = avg_or_zero(&self.memory_samples.read());
        let gpu = avg_or_zero(&self.gpu_samples.read());
        let noise = avg_or_zero(&self.noise_samples.read());

        let tp = *self.true_positives.read();
        let fp = *self.false_positives.read();
        let fn_ = *self.false_negatives.read();
        let accuracy = if tp + fp + fn_ > 0 {
            tp as f64 / (tp + fp + fn_) as f64
        } else {
            1.0
        };

        let channels: HashMap<String, ChannelMetrics> = mixer_channels
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    ChannelMetrics {
                        gain_db: v.gain_db,
                        duck_amount_db: v.duck_amount_db,
                        is_muted: v.is_muted,
                        priority: format!("{:?}", v.priority),
                    },
                )
            })
            .collect();

        VoiceEngineMetrics {
            latency: LatencyMetrics {
                wake_detection_ms: wake,
                stt_first_token_ms: stt,
                llm_first_token_ms: llm,
                tts_first_chunk_ms: tts,
                end_to_end_ms: wake + stt + llm + tts,
            },
            audio: AudioQualityMetrics {
                noise_percent: noise,
                echo_reduction_db: calibration.echo_level_db.abs() as f64,
                dropped_frames: *self.dropped_frames.read(),
                total_frames: audio_metrics.total_packets_captured,
                clipping_count: *self.clipping_count.read(),
            },
            system: SystemMetrics {
                cpu_percent: cpu,
                memory_mb: mem,
                gpu_percent: gpu,
                thread_count: std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1),
                uptime_seconds: self.start_time.elapsed().as_secs(),
            },
            detection: DetectionMetrics {
                wake_word_accuracy: accuracy,
                false_positives: fp,
                true_positives: tp,
                false_negatives: fn_,
                interruption_count: *self.interruptions.read(),
            },
            channels,
        }
    }

    pub fn reset(&self) {
        self.wake_detection_latencies.write().clear();
        self.stt_latencies.write().clear();
        self.llm_latencies.write().clear();
        self.tts_latencies.write().clear();
        *self.false_positives.write() = 0;
        *self.true_positives.write() = 0;
        *self.false_negatives.write() = 0;
        *self.interruptions.write() = 0;
        *self.dropped_frames.write() = 0;
        *self.clipping_count.write() = 0;
        self.cpu_samples.write().clear();
        self.memory_samples.write().clear();
        self.gpu_samples.write().clear();
        self.noise_samples.write().clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn avg_or_zero(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        0.0
    } else {
        samples.iter().sum::<f64>() / samples.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::AudioMetrics;
    use crate::mixer::MixerChannel;

    #[test]
    fn metrics_collector_creation() {
        let mc = MetricsCollector::new();
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &HashMap::new(),
            &CalibrationProfile::default(),
        );
        assert_eq!(metrics.latency.end_to_end_ms, 0.0);
        assert_eq!(metrics.system.uptime_seconds, 0);
    }

    #[test]
    fn metrics_record_latencies() {
        let mc = MetricsCollector::new();
        mc.record_wake_latency(50.0);
        mc.record_stt_latency(200.0);
        mc.record_llm_latency(300.0);
        mc.record_tts_latency(150.0);
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &HashMap::new(),
            &CalibrationProfile::default(),
        );
        assert!((metrics.latency.wake_detection_ms - 50.0).abs() < 1.0);
        assert!((metrics.latency.stt_first_token_ms - 200.0).abs() < 1.0);
        assert!((metrics.latency.end_to_end_ms - 700.0).abs() < 1.0);
    }

    #[test]
    fn metrics_detection_accuracy() {
        let mc = MetricsCollector::new();
        mc.record_true_positive();
        mc.record_true_positive();
        mc.record_false_positive();
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &HashMap::new(),
            &CalibrationProfile::default(),
        );
        let accuracy = metrics.detection.wake_word_accuracy;
        assert!((accuracy - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn metrics_channel_state() {
        let mc = MetricsCollector::new();
        let mut channels = HashMap::new();
        channels.insert(
            MixerChannel::Voxy,
            ChannelState {
                gain_db: 3.0,
                duck_amount_db: 0.0,
                is_muted: false,
                priority: crate::mixer::DuckingPriority::Never,
                ..Default::default()
            },
        );
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &channels,
            &CalibrationProfile::default(),
        );
        assert!(metrics.channels.contains_key("voxy"));
    }

    #[test]
    fn metrics_reset() {
        let mc = MetricsCollector::new();
        mc.record_wake_latency(50.0);
        mc.record_false_positive();
        mc.reset();
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &HashMap::new(),
            &CalibrationProfile::default(),
        );
        assert_eq!(metrics.latency.wake_detection_ms, 0.0);
        assert_eq!(metrics.detection.false_positives, 0);
    }

    #[test]
    fn metrics_clipping_count() {
        let mc = MetricsCollector::new();
        mc.record_clipping();
        mc.record_clipping();
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &HashMap::new(),
            &CalibrationProfile::default(),
        );
        assert_eq!(metrics.audio.clipping_count, 2);
    }

    #[test]
    fn metrics_avg_or_zero() {
        assert_eq!(avg_or_zero(&[]), 0.0);
        assert!((avg_or_zero(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn metrics_serialization() {
        let mc = MetricsCollector::new();
        let metrics = mc.collect(
            &AudioMetrics::default(),
            &HashMap::new(),
            &CalibrationProfile::default(),
        );
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("wake_detection_ms"));
        assert!(json.contains("cpu_percent"));
    }
}
