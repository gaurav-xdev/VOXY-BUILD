use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Quality mode for the voice engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityMode {
    /// Highest quality, highest latency.
    UltraQuality,
    /// Balanced quality and latency.
    Balanced,
    /// Lowest latency, lower quality.
    UltraLowLatency,
    /// Optimized for gaming (minimal CPU usage).
    Gaming,
    /// Minimal resource usage on battery.
    BatterySaver,
}

impl QualityMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UltraQuality => "ultra_quality",
            Self::Balanced => "balanced",
            Self::UltraLowLatency => "ultra_low_latency",
            Self::Gaming => "gaming",
            Self::BatterySaver => "battery_saver",
        }
    }

    /// Noise suppression level (0.0 = off, 1.0 = maximum).
    pub fn noise_suppression(&self) -> f32 {
        match self {
            Self::UltraQuality => 1.0,
            Self::Balanced => 0.7,
            Self::UltraLowLatency => 0.3,
            Self::Gaming => 0.5,
            Self::BatterySaver => 0.3,
        }
    }

    /// Echo cancellation enabled.
    pub fn echo_cancellation(&self) -> bool {
        matches!(self, Self::UltraQuality | Self::Balanced)
    }

    /// DSP processing enabled.
    pub fn dsp_enabled(&self) -> bool {
        !matches!(self, Self::BatterySaver)
    }
}

impl std::fmt::Display for QualityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// System resource snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub memory_total_mb: f64,
    pub gpu_percent: f64,
    pub vram_mb: f64,
    pub vram_total_mb: f64,
    pub battery_percent: Option<f64>,
    pub temperature_c: Option<f64>,
    pub is_on_battery: bool,
}

/// Thresholds for quality mode switching.
#[derive(Debug, Clone)]
pub struct SchedulerThresholds {
    /// CPU percent above which we downgrade.
    pub cpu_high: f64,
    /// Memory percent above which we downgrade.
    pub memory_high: f64,
    /// GPU percent above which we downgrade.
    pub gpu_high: f64,
    /// Battery percent below which we enter battery saver.
    pub battery_low: f64,
    /// Temperature above which we throttle.
    pub temperature_high: f64,
}

impl Default for SchedulerThresholds {
    fn default() -> Self {
        Self {
            cpu_high: 80.0,
            memory_high: 85.0,
            gpu_high: 90.0,
            battery_low: 20.0,
            temperature_high: 85.0,
        }
    }
}

/// AI Audio Scheduler that monitors system resources and switches quality modes.
pub struct AiAudioScheduler {
    current_mode: RwLock<QualityMode>,
    thresholds: SchedulerSnapshot,
    last_switch: RwLock<Instant>,
    min_switch_interval: Duration,
    is_running: Arc<AtomicBool>,
    history: RwLock<Vec<(Instant, QualityMode, SystemSnapshot)>>,
    max_history: usize,
}

struct SchedulerSnapshot {
    thresholds: SchedulerThresholds,
}

impl AiAudioScheduler {
    pub fn new() -> Self {
        Self {
            current_mode: RwLock::new(QualityMode::Balanced),
            thresholds: SchedulerSnapshot {
                thresholds: SchedulerThresholds::default(),
            },
            last_switch: RwLock::new(Instant::now() - Duration::from_secs(300)),
            min_switch_interval: Duration::from_secs(5),
            is_running: Arc::new(AtomicBool::new(false)),
            history: RwLock::new(Vec::with_capacity(100)),
            max_history: 100,
        }
    }

    pub fn with_thresholds(mut self, thresholds: SchedulerThresholds) -> Self {
        self.thresholds.thresholds = thresholds;
        self
    }

    pub fn with_min_switch_interval(mut self, interval: Duration) -> Self {
        self.min_switch_interval = interval;
        self
    }

    /// Evaluate system snapshot and determine the optimal quality mode.
    pub fn evaluate(&self, snapshot: &SystemSnapshot) -> QualityMode {
        let t = &self.thresholds.thresholds;

        // Priority 1: Battery saver
        if snapshot.is_on_battery {
            if let Some(bat) = snapshot.battery_percent {
                if bat < t.battery_low {
                    return QualityMode::BatterySaver;
                }
            }
        }

        // Priority 2: Temperature throttle
        if let Some(temp) = snapshot.temperature_c {
            if temp > t.temperature_high {
                return QualityMode::BatterySaver;
            }
        }

        // Priority 3: CPU overload
        if snapshot.cpu_percent > t.cpu_high {
            return QualityMode::Gaming;
        }

        // Priority 4: Memory pressure
        let mem_percent = if snapshot.memory_total_mb > 0.0 {
            (snapshot.memory_mb / snapshot.memory_total_mb) * 100.0
        } else {
            0.0
        };
        if mem_percent > t.memory_high {
            return QualityMode::UltraLowLatency;
        }

        // Priority 5: GPU pressure
        if snapshot.gpu_percent > t.gpu_high {
            return QualityMode::Gaming;
        }

        // Default: balanced
        QualityMode::Balanced
    }

    /// Update the scheduler with a new system snapshot.
    /// Returns (new_mode, changed).
    pub fn update(&self, snapshot: &SystemSnapshot) -> (QualityMode, bool) {
        let new_mode = self.evaluate(snapshot);
        let current = *self.current_mode.read();
        let now = Instant::now();

        if new_mode != current
            && now.duration_since(*self.last_switch.read()) >= self.min_switch_interval
        {
            *self.current_mode.write() = new_mode;
            *self.last_switch.write() = now;

            let mut hist = self.history.write();
            if hist.len() >= self.max_history {
                hist.remove(0);
            }
            hist.push((now, new_mode, snapshot.clone()));

            tracing::info!(
                from = %current,
                to = %new_mode,
                cpu = snapshot.cpu_percent,
                mem = snapshot.memory_mb,
                "Quality mode switched"
            );

            (new_mode, true)
        } else {
            (current, false)
        }
    }

    /// Get the current quality mode.
    pub fn current_mode(&self) -> QualityMode {
        *self.current_mode.read()
    }

    /// Get the mode history.
    pub fn history(&self) -> Vec<(Instant, QualityMode, SystemSnapshot)> {
        self.history.read().clone()
    }

    /// Force a specific quality mode.
    pub fn force_mode(&self, mode: QualityMode) {
        let current = *self.current_mode.read();
        if current != mode {
            *self.current_mode.write() = mode;
            *self.last_switch.write() = Instant::now();
            tracing::info!(mode = %mode, "Quality mode forced");
        }
    }

    /// Start background monitoring.
    pub fn start(self: &Arc<Self>) {
        self.is_running.store(true, Ordering::SeqCst);
    }

    /// Stop monitoring.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Default for AiAudioScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_for(cpu: f64, mem_mb: f64, total_mb: f64, gpu: f64) -> SystemSnapshot {
        SystemSnapshot {
            cpu_percent: cpu,
            memory_mb: mem_mb,
            memory_total_mb: total_mb,
            gpu_percent: gpu,
            vram_mb: 0.0,
            vram_total_mb: 0.0,
            battery_percent: None,
            temperature_c: None,
            is_on_battery: false,
        }
    }

    #[test]
    fn scheduler_creation() {
        let s = AiAudioScheduler::new();
        assert_eq!(s.current_mode(), QualityMode::Balanced);
    }

    #[test]
    fn scheduler_default_mode_balanced() {
        let s = AiAudioScheduler::new();
        let snap = snapshot_for(50.0, 4000.0, 16000.0, 50.0);
        let (mode, changed) = s.update(&snap);
        assert_eq!(mode, QualityMode::Balanced);
        assert!(!changed); // Already balanced
    }

    #[test]
    fn scheduler_cpu_high_gaming() {
        let s = AiAudioScheduler::new();
        let snap = snapshot_for(95.0, 4000.0, 16000.0, 50.0);
        let (mode, changed) = s.update(&snap);
        assert_eq!(mode, QualityMode::Gaming);
        assert!(changed);
    }

    #[test]
    fn scheduler_memory_high_low_latency() {
        let s = AiAudioScheduler::new();
        // 15GB used out of 16GB = 93.75% > 85% threshold
        let snap = snapshot_for(50.0, 15000.0, 16000.0, 50.0);
        let (mode, changed) = s.update(&snap);
        assert_eq!(mode, QualityMode::UltraLowLatency);
        assert!(changed);
    }

    #[test]
    fn scheduler_gpu_high_gaming() {
        let s = AiAudioScheduler::new();
        let snap = snapshot_for(50.0, 4000.0, 16000.0, 95.0);
        let (mode, changed) = s.update(&snap);
        assert_eq!(mode, QualityMode::Gaming);
        assert!(changed);
    }

    #[test]
    fn scheduler_battery_low_battery_saver() {
        let s = AiAudioScheduler::new();
        let mut snap = snapshot_for(30.0, 4000.0, 16000.0, 30.0);
        snap.is_on_battery = true;
        snap.battery_percent = Some(10.0);
        let (mode, changed) = s.update(&snap);
        assert_eq!(mode, QualityMode::BatterySaver);
        assert!(changed);
    }

    #[test]
    fn scheduler_temperature_high_battery_saver() {
        let s = AiAudioScheduler::new();
        let mut snap = snapshot_for(50.0, 4000.0, 16000.0, 50.0);
        snap.temperature_c = Some(90.0);
        let (mode, changed) = s.update(&snap);
        assert_eq!(mode, QualityMode::BatterySaver);
        assert!(changed);
    }

    #[test]
    fn scheduler_min_switch_interval() {
        let s = AiAudioScheduler::new().with_min_switch_interval(Duration::from_secs(60));
        let snap1 = snapshot_for(95.0, 4000.0, 16000.0, 50.0);
        let (mode1, changed1) = s.update(&snap1);
        assert_eq!(mode1, QualityMode::Gaming);
        assert!(changed1);

        // Second update within interval should not change
        let snap2 = snapshot_for(30.0, 4000.0, 16000.0, 30.0);
        let (mode2, changed2) = s.update(&snap2);
        assert_eq!(mode2, QualityMode::Gaming); // Still gaming
        assert!(!changed2);
    }

    #[test]
    fn scheduler_force_mode() {
        let s = AiAudioScheduler::new();
        s.force_mode(QualityMode::UltraQuality);
        assert_eq!(s.current_mode(), QualityMode::UltraQuality);
    }

    #[test]
    fn quality_mode_properties() {
        assert!(QualityMode::UltraQuality.echo_cancellation());
        assert!(QualityMode::Balanced.echo_cancellation());
        assert!(!QualityMode::UltraLowLatency.echo_cancellation());
        assert!(!QualityMode::Gaming.echo_cancellation());
        assert!(!QualityMode::BatterySaver.echo_cancellation());

        assert!(QualityMode::UltraQuality.dsp_enabled());
        assert!(QualityMode::Balanced.dsp_enabled());
        assert!(!QualityMode::BatterySaver.dsp_enabled());

        assert!((QualityMode::UltraQuality.noise_suppression() - 1.0).abs() < f32::EPSILON);
        assert!((QualityMode::BatterySaver.noise_suppression() - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn scheduler_history_tracking() {
        let s = AiAudioScheduler::new();
        let snap = snapshot_for(95.0, 4000.0, 16000.0, 50.0);
        s.update(&snap);
        assert_eq!(s.history().len(), 1);
    }

    #[test]
    fn scheduler_default_trait() {
        let s = AiAudioScheduler::default();
        assert_eq!(s.current_mode(), QualityMode::Balanced);
    }
}
