use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::error::AudioError;
use crate::stream::AudioPacket;

#[derive(Debug, Clone)]
pub struct AudioMetrics {
    pub total_packets_captured: u64,
    pub total_packets_played: u64,
    pub total_bytes_captured: u64,
    pub total_bytes_played: u64,
    pub dropped_packets: u64,
    pub buffer_reuse_count: u64,
    pub lock_contention_count: u64,
    latency_sample_count: u64,
    pub current_latency_ms: f64,
    pub average_latency_ms: f64,
    pub peak_latency_ms: f64,
    pub min_latency_ms: f64,
    pub current_peak_level: f32,
    pub current_rms_level: f32,
    pub clock_drift_ppm: f64,
    pub uptime_seconds: u64,
}

impl Default for AudioMetrics {
    fn default() -> Self {
        Self {
            total_packets_captured: 0,
            total_packets_played: 0,
            total_bytes_captured: 0,
            total_bytes_played: 0,
            dropped_packets: 0,
            buffer_reuse_count: 0,
            lock_contention_count: 0,
            latency_sample_count: 0,
            current_latency_ms: 0.0,
            average_latency_ms: 0.0,
            peak_latency_ms: 0.0,
            min_latency_ms: f64::MAX,
            current_peak_level: 0.0,
            current_rms_level: 0.0,
            clock_drift_ppm: 0.0,
            uptime_seconds: 0,
        }
    }
}

#[async_trait::async_trait]
pub trait AudioDiagnostics: Send + Sync {
    async fn metrics(&self) -> AudioMetrics;
    async fn record_packet_captured(&self, packet: &AudioPacket);
    async fn record_packet_played(&self, packet: &AudioPacket);
    async fn record_error(&self, error: &AudioError);
    async fn record_latency(&self, latency_ms: f64);
    async fn recent_errors(&self, count: usize) -> Vec<String>;
    async fn reset(&self);
    async fn record_buffer_reuse(&self) {}
    async fn record_lock_contention(&self) {}
}

pub struct InMemoryDiagnostics {
    metrics: Arc<RwLock<AudioMetrics>>,
    errors: Arc<RwLock<VecDeque<String>>>,
}

impl Default for InMemoryDiagnostics {
    fn default() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(AudioMetrics::default())),
            errors: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
}

impl InMemoryDiagnostics {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl AudioDiagnostics for InMemoryDiagnostics {
    async fn metrics(&self) -> AudioMetrics {
        self.metrics.read().clone()
    }

    async fn record_packet_captured(&self, packet: &AudioPacket) {
        let mut metrics = self.metrics.write();
        metrics.total_packets_captured += 1;
        metrics.total_bytes_captured += (packet.data.len() * 4) as u64;
        metrics.current_peak_level = packet.peak_level;
        metrics.current_rms_level = packet.rms_level;
    }

    async fn record_packet_played(&self, packet: &AudioPacket) {
        let mut metrics = self.metrics.write();
        metrics.total_packets_played += 1;
        metrics.total_bytes_played += (packet.data.len() * 4) as u64;
        metrics.current_peak_level = packet.peak_level;
        metrics.current_rms_level = packet.rms_level;
    }

    async fn record_error(&self, error: &AudioError) {
        let mut errors = self.errors.write();
        if errors.len() >= 1000 {
            errors.pop_front();
        }
        errors.push_back(error.to_string());
    }

    async fn record_latency(&self, latency_ms: f64) {
        let mut metrics = self.metrics.write();
        metrics.current_latency_ms = latency_ms;

        let count = metrics.latency_sample_count;
        if count > 0 {
            let prev_total = metrics.average_latency_ms * (count as f64);
            metrics.average_latency_ms = (prev_total + latency_ms) / ((count + 1) as f64);
        } else {
            metrics.average_latency_ms = latency_ms;
        }
        metrics.latency_sample_count += 1;

        if latency_ms > metrics.peak_latency_ms {
            metrics.peak_latency_ms = latency_ms;
        }
        if latency_ms < metrics.min_latency_ms {
            metrics.min_latency_ms = latency_ms;
        }
    }

    async fn recent_errors(&self, count: usize) -> Vec<String> {
        let errors = self.errors.read();
        let len = errors.len();
        let start = len.saturating_sub(count);
        errors.iter().skip(start).cloned().collect()
    }

    async fn record_buffer_reuse(&self) {
        self.metrics.write().buffer_reuse_count += 1;
    }

    async fn record_lock_contention(&self) {
        self.metrics.write().lock_contention_count += 1;
    }

    async fn reset(&self) {
        let mut metrics = self.metrics.write();
        *metrics = AudioMetrics::default();
        let mut errors = self.errors.write();
        errors.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Histogram {
    total: u64,
    min: f64,
    max: f64,
    sum: f64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            total: 0,
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
        }
    }

    pub fn record(&mut self, value_ms: f64) {
        self.total += 1;
        self.sum += value_ms;
        if value_ms < self.min {
            self.min = value_ms;
        }
        if value_ms > self.max {
            self.max = value_ms;
        }
    }

    pub fn count(&self) -> u64 {
        self.total
    }

    pub fn min_ms(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.min
        }
    }

    pub fn max_ms(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.max
        }
    }

    pub fn avg_ms(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.sum / self.total as f64
        }
    }

    pub fn p50_ms(&self) -> f64 {
        self.percentile(50.0)
    }

    pub fn p95_ms(&self) -> f64 {
        self.percentile(95.0)
    }

    pub fn p99_ms(&self) -> f64 {
        self.percentile(99.0)
    }

    fn percentile(&self, _p: f64) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.avg_ms()
    }
}

pub struct StageTimingCollector {
    stages: RwLock<Vec<StageTiming>>,
}

impl Default for StageTimingCollector {
    fn default() -> Self {
        Self::new()
    }
}

struct StageTiming {
    name: &'static str,
    samples: Vec<f64>,
}

impl StageTimingCollector {
    pub fn new() -> Self {
        Self {
            stages: RwLock::new(Vec::new()),
        }
    }

    pub fn record_stage(&self, name: &'static str, duration_ms: f64) {
        let mut stages = self.stages.write();
        if let Some(s) = stages
            .iter_mut()
            .find(|s: &&mut StageTiming| s.name == name)
        {
            s.samples.push(duration_ms);
            // Keep only the last 1000 samples per stage to prevent unbounded growth
            if s.samples.len() > 1000 {
                s.samples.drain(0..s.samples.len() - 1000);
            }
        } else {
            stages.push(StageTiming {
                name,
                samples: vec![duration_ms],
            });
        }
    }

    pub fn stage_report(&self, name: &str) -> Option<Histogram> {
        let stages = self.stages.read();
        let s = stages.iter().find(|s| s.name == name)?;
        let mut h = Histogram::new();
        for &v in &s.samples {
            h.record(v);
        }
        Some(h)
    }

    pub fn all_reports(&self) -> Vec<(&'static str, Histogram)> {
        let stages = self.stages.read();
        stages
            .iter()
            .map(|s| {
                let mut h = Histogram::new();
                for &v in &s.samples {
                    h.record(v);
                }
                (s.name, h)
            })
            .collect()
    }

    pub fn reset(&self) {
        self.stages.write().clear();
    }
}

pub struct ResourceTracker {
    cpu_samples: RwLock<VecDeque<f64>>,
    ram_samples: RwLock<VecDeque<u64>>,
    thread_samples: RwLock<VecDeque<usize>>,
    audio_buffer_samples: RwLock<VecDeque<u64>>,
}

const MAX_RESOURCE_SAMPLES: usize = 1000;

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            cpu_samples: RwLock::new(VecDeque::with_capacity(MAX_RESOURCE_SAMPLES)),
            ram_samples: RwLock::new(VecDeque::with_capacity(MAX_RESOURCE_SAMPLES)),
            thread_samples: RwLock::new(VecDeque::with_capacity(MAX_RESOURCE_SAMPLES)),
            audio_buffer_samples: RwLock::new(VecDeque::with_capacity(MAX_RESOURCE_SAMPLES)),
        }
    }

    pub fn record_cpu(&self, percent: f64) {
        let mut s = self.cpu_samples.write();
        if s.len() >= MAX_RESOURCE_SAMPLES {
            s.pop_front();
        }
        s.push_back(percent);
    }

    pub fn record_ram(&self, bytes: u64) {
        let mut s = self.ram_samples.write();
        if s.len() >= MAX_RESOURCE_SAMPLES {
            s.pop_front();
        }
        s.push_back(bytes);
    }

    pub fn record_threads(&self, count: usize) {
        let mut s = self.thread_samples.write();
        if s.len() >= MAX_RESOURCE_SAMPLES {
            s.pop_front();
        }
        s.push_back(count);
    }

    pub fn record_audio_buffer(&self, packets: u64) {
        let mut s = self.audio_buffer_samples.write();
        if s.len() >= MAX_RESOURCE_SAMPLES {
            s.pop_front();
        }
        s.push_back(packets);
    }

    pub fn avg_cpu(&self) -> f64 {
        let s = self.cpu_samples.read();
        if s.is_empty() {
            return 0.0;
        }
        s.iter().sum::<f64>() / s.len() as f64
    }

    pub fn avg_ram_mb(&self) -> f64 {
        let s = self.ram_samples.read();
        if s.is_empty() {
            return 0.0;
        }
        (s.iter().sum::<u64>() as f64 / s.len() as f64) / (1024.0 * 1024.0)
    }

    pub fn avg_threads(&self) -> f64 {
        let s = self.thread_samples.read();
        if s.is_empty() {
            return 0.0;
        }
        s.iter().sum::<usize>() as f64 / s.len() as f64
    }

    pub fn reset(&self) {
        self.cpu_samples.write().clear();
        self.ram_samples.write().clear();
        self.thread_samples.write().clear();
        self.audio_buffer_samples.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_diagnostics_default_metrics() {
        let diag = InMemoryDiagnostics::new();
        let metrics = diag.metrics().await;
        assert_eq!(metrics.total_packets_captured, 0);
        assert_eq!(metrics.total_packets_played, 0);
        assert_eq!(metrics.dropped_packets, 0);
    }

    #[tokio::test]
    async fn test_record_packet_captured() {
        let diag = InMemoryDiagnostics::new();
        let packet = AudioPacket::new(vec![0.5, -0.3, 0.1], 16000, 1);
        diag.record_packet_captured(&packet).await;
        let metrics = diag.metrics().await;
        assert_eq!(metrics.total_packets_captured, 1);
        assert_eq!(metrics.total_bytes_captured, 12);
        assert_eq!(metrics.current_peak_level, 0.5);
    }

    #[tokio::test]
    async fn test_record_packet_played() {
        let diag = InMemoryDiagnostics::new();
        let packet = AudioPacket::new(vec![0.2, -0.1], 16000, 1);
        diag.record_packet_played(&packet).await;
        let metrics = diag.metrics().await;
        assert_eq!(metrics.total_packets_played, 1);
        assert_eq!(metrics.total_bytes_played, 8);
    }

    #[tokio::test]
    async fn test_record_error() {
        let diag = InMemoryDiagnostics::new();
        let error = AudioError::BufferOverflow;
        diag.record_error(&error).await;
        let errors = diag.recent_errors(10).await;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Buffer overflow"));
    }

    #[tokio::test]
    async fn test_record_latency() {
        let diag = InMemoryDiagnostics::new();
        diag.record_latency(10.0).await;
        diag.record_latency(20.0).await;
        diag.record_latency(30.0).await;
        let metrics = diag.metrics().await;
        assert_eq!(metrics.current_latency_ms, 30.0);
        assert!((metrics.average_latency_ms - 20.0).abs() < 1e-6);
        assert_eq!(metrics.peak_latency_ms, 30.0);
        assert_eq!(metrics.min_latency_ms, 10.0);
    }

    #[tokio::test]
    async fn test_recent_errors_count() {
        let diag = InMemoryDiagnostics::new();
        for i in 0..5 {
            let err = AudioError::StreamError(format!("error {i}"));
            diag.record_error(&err).await;
        }
        let errors = diag.recent_errors(3).await;
        assert_eq!(errors.len(), 3);
    }

    #[tokio::test]
    async fn test_reset() {
        let diag = InMemoryDiagnostics::new();
        let packet = AudioPacket::new(vec![0.5], 16000, 1);
        diag.record_packet_captured(&packet).await;
        diag.reset().await;
        let metrics = diag.metrics().await;
        assert_eq!(metrics.total_packets_captured, 0);
        let errors = diag.recent_errors(10).await;
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_packets() {
        let diag = InMemoryDiagnostics::new();
        for _ in 0..10 {
            let packet = AudioPacket::new(vec![0.1, 0.2], 16000, 1);
            diag.record_packet_captured(&packet).await;
        }
        let metrics = diag.metrics().await;
        assert_eq!(metrics.total_packets_captured, 10);
    }

    #[test]
    fn test_histogram_record() {
        let mut h = Histogram::new();
        h.record(10.0);
        h.record(20.0);
        h.record(30.0);
        assert_eq!(h.count(), 3);
        assert!((h.avg_ms() - 20.0).abs() < 1e-6);
        assert!((h.min_ms() - 10.0).abs() < 1e-6);
        assert!((h.max_ms() - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_histogram_empty() {
        let h = Histogram::new();
        assert_eq!(h.count(), 0);
        assert!((h.avg_ms() - 0.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_stage_timing_collector() {
        let c = StageTimingCollector::new();
        c.record_stage("stt", 150.0);
        c.record_stage("stt", 250.0);
        c.record_stage("tts", 300.0);
        let report = c.stage_report("stt").unwrap();
        assert_eq!(report.count(), 2);
        assert!((report.avg_ms() - 200.0).abs() < 1e-6);
        let all = c.all_reports();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_stage_timing_collector_reset() {
        let c = StageTimingCollector::new();
        c.record_stage("test", 100.0);
        assert!(c.stage_report("test").is_some());
        c.reset();
        assert!(c.stage_report("test").is_none());
    }
}
