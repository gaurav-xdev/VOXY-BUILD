//! Counters, histograms, gauges, timers, and metric registry.
//!
//! Thread-safe metrics collection with Prometheus export.
//! All metric types use `Arc` for cheap cloning and atomic operations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// A simple counter metric.
///
/// Thread-safe via `Arc<AtomicU64>`. Clone is cheap (clones the Arc).
#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    value: Arc<AtomicU64>,
}

impl Counter {
    /// Create a new counter.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment the counter by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Add a value to the counter.
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get the current value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the counter name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reset the counter to zero.
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

/// A gauge metric that can go up and down.
///
/// Thread-safe via `Arc<AtomicU64>`.
#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    value: Arc<AtomicU64>,
}

impl Gauge {
    /// Create a new gauge.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Set the gauge value.
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge by 1.
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Add a value to the gauge.
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Subtract a value from the gauge.
    pub fn sub(&self, value: u64) {
        self.value.fetch_sub(value, Ordering::Relaxed);
    }

    /// Get the current value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the gauge name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A histogram for tracking distributions of values.
///
/// Uses `parking_lot::Mutex` for bucket updates (short critical sections).
/// The `count` and `sum` are tracked separately for lock-free reads.
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    buckets: Arc<parking_lot::Mutex<Vec<(f64, u64)>>>,
    count: Arc<AtomicU64>,
    sum: Arc<parking_lot::Mutex<f64>>,
}

impl Histogram {
    /// Create a new histogram with default buckets.
    pub fn new(name: impl Into<String>) -> Self {
        let default_buckets = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];
        Self::with_buckets(name, default_buckets)
    }

    /// Create a new histogram with custom buckets.
    pub fn with_buckets(name: impl Into<String>, boundaries: Vec<f64>) -> Self {
        let buckets = boundaries.into_iter().map(|b| (b, 0u64)).collect();
        Self {
            name: name.into(),
            buckets: Arc::new(parking_lot::Mutex::new(buckets)),
            count: Arc::new(AtomicU64::new(0)),
            sum: Arc::new(parking_lot::Mutex::new(0.0)),
        }
    }

    /// Record a value.
    pub fn record(&self, value: f64) {
        // Lock buckets first, then sum — consistent ordering prevents deadlocks
        {
            let mut buckets = self.buckets.lock();
            for (boundary, count) in buckets.iter_mut() {
                if value <= *boundary {
                    *count += 1;
                }
            }
        }
        self.count.fetch_add(1, Ordering::Relaxed);
        {
            let mut sum = self.sum.lock();
            *sum += value;
        }
    }

    /// Get the count of observations.
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the sum of all observations.
    pub fn sum(&self) -> f64 {
        *self.sum.lock()
    }

    /// Get the mean value.
    pub fn mean(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            return 0.0;
        }
        self.sum() / count as f64
    }

    /// Get the histogram name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the bucket counts.
    pub fn buckets(&self) -> Vec<(f64, u64)> {
        self.buckets.lock().clone()
    }

    /// Reset the histogram.
    pub fn reset(&self) {
        self.buckets.lock().iter_mut().for_each(|(_, c)| *c = 0);
        self.count.store(0, Ordering::Relaxed);
        *self.sum.lock() = 0.0;
    }
}

/// A timer for measuring durations.
///
/// Use the RAII `TimerGuard` for automatic duration recording.
#[derive(Debug, Clone)]
pub struct Timer {
    name: String,
    histogram: Histogram,
}

impl Timer {
    /// Create a new timer.
    pub fn new(name: impl Into<String>) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            histogram: Histogram::new(name_str),
        }
    }

    /// Start the timer and return a guard that records on drop.
    pub fn start(&self) -> TimerGuard<'_> {
        TimerGuard {
            timer: self,
            start: Instant::now(),
        }
    }

    /// Record a duration in seconds.
    pub fn record_seconds(&self, seconds: f64) {
        self.histogram.record(seconds);
    }

    /// Record a duration in milliseconds.
    pub fn record_millis(&self, millis: f64) {
        self.histogram.record(millis / 1000.0);
    }

    /// Get the histogram reference.
    pub fn histogram(&self) -> &Histogram {
        &self.histogram
    }

    /// Get the timer name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Guard that records elapsed time when dropped.
pub struct TimerGuard<'a> {
    timer: &'a Timer,
    start: Instant,
}

impl Drop for TimerGuard<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        self.timer.record_seconds(elapsed);
    }
}

/// Registry for all metrics.
///
/// Thread-safe via `parking_lot::RwLock`. All metric getters use read locks.
/// Write locks are only taken when creating new metrics.
pub struct MetricsRegistry {
    counters: parking_lot::RwLock<HashMap<String, Counter>>,
    gauges: parking_lot::RwLock<HashMap<String, Gauge>>,
    histograms: parking_lot::RwLock<HashMap<String, Histogram>>,
    timers: parking_lot::RwLock<HashMap<String, Timer>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry.
    pub fn new() -> Self {
        Self {
            counters: parking_lot::RwLock::new(HashMap::new()),
            gauges: parking_lot::RwLock::new(HashMap::new()),
            histograms: parking_lot::RwLock::new(HashMap::new()),
            timers: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a counter.
    pub fn counter(&self, name: &str) -> Counter {
        let mut counters = self.counters.write();
        counters
            .entry(name.to_string())
            .or_insert_with(|| Counter::new(name))
            .clone()
    }

    /// Get or create a gauge.
    pub fn gauge(&self, name: &str) -> Gauge {
        let mut gauges = self.gauges.write();
        gauges
            .entry(name.to_string())
            .or_insert_with(|| Gauge::new(name))
            .clone()
    }

    /// Get or create a histogram.
    pub fn histogram(&self, name: &str) -> Histogram {
        let mut histograms = self.histograms.write();
        histograms
            .entry(name.to_string())
            .or_insert_with(|| Histogram::new(name))
            .clone()
    }

    /// Get or create a timer.
    pub fn timer(&self, name: &str) -> Timer {
        let mut timers = self.timers.write();
        timers
            .entry(name.to_string())
            .or_insert_with(|| Timer::new(name))
            .clone()
    }

    /// Get the value of a counter.
    pub fn get_counter_value(&self, name: &str) -> Option<u64> {
        self.counters.read().get(name).map(|c| c.get())
    }

    /// Get the value of a gauge.
    pub fn get_gauge_value(&self, name: &str) -> Option<u64> {
        self.gauges.read().get(name).map(|g| g.get())
    }

    /// Get all counter names.
    pub fn counter_names(&self) -> Vec<String> {
        self.counters.read().keys().cloned().collect()
    }

    /// Get all gauge names.
    pub fn gauge_names(&self) -> Vec<String> {
        self.gauges.read().keys().cloned().collect()
    }

    /// Get all histogram names.
    pub fn histogram_names(&self) -> Vec<String> {
        self.histograms.read().keys().cloned().collect()
    }

    /// Get all timer names.
    pub fn timer_names(&self) -> Vec<String> {
        self.timers.read().keys().cloned().collect()
    }

    /// Export all metrics as Prometheus text format.
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();

        for (name, counter) in self.counters.read().iter() {
            let sanitized = sanitize_name(name);
            output.push_str(&format!("# TYPE {} counter\n", sanitized));
            output.push_str(&format!("{} {}\n", sanitized, counter.get()));
        }

        for (name, gauge) in self.gauges.read().iter() {
            let sanitized = sanitize_name(name);
            output.push_str(&format!("# TYPE {} gauge\n", sanitized));
            output.push_str(&format!("{} {}\n", sanitized, gauge.get()));
        }

        for (name, histogram) in self.histograms.read().iter() {
            let sanitized = sanitize_name(name);
            output.push_str(&format!("# TYPE {} histogram\n", sanitized));
            let buckets = histogram.buckets();
            for (boundary, count) in &buckets {
                output.push_str(&format!(
                    "{}_bucket{{le=\"{}\"}} {}\n",
                    sanitized, boundary, count
                ));
            }
            output.push_str(&format!("{}_count {}\n", sanitized, histogram.count()));
            output.push_str(&format!("{}_sum {}\n", sanitized, histogram.sum()));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize a metric name for Prometheus.
fn sanitize_name(name: &str) -> String {
    name.replace(['.', '-'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_increments() {
        let counter = Counter::new("test");
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn counter_reset() {
        let counter = Counter::new("test");
        counter.add(10);
        assert_eq!(counter.get(), 10);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn gauge_operations() {
        let gauge = Gauge::new("test");
        assert_eq!(gauge.get(), 0);
        gauge.set(5);
        assert_eq!(gauge.get(), 5);
        gauge.inc();
        assert_eq!(gauge.get(), 6);
        gauge.dec();
        assert_eq!(gauge.get(), 5);
        gauge.sub(3);
        assert_eq!(gauge.get(), 2);
    }

    #[test]
    fn histogram_records() {
        let hist = Histogram::new("test");
        hist.record(0.5);
        hist.record(1.5);
        assert_eq!(hist.count(), 2);
        assert!((hist.sum() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn histogram_buckets() {
        let hist = Histogram::new("test");
        hist.record(0.01);
        hist.record(0.1);
        hist.record(1.0);

        let buckets = hist.buckets();
        assert!(!buckets.is_empty());
        let bucket_1 = buckets.iter().find(|(b, _)| *b >= 1.0).unwrap();
        assert_eq!(bucket_1.1, 3);
    }

    #[test]
    fn timer_records_duration() {
        let timer = Timer::new("test");
        {
            let _guard = timer.start();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(timer.histogram().count() > 0);
    }

    #[test]
    fn registry_returns_same_metrics() {
        let registry = MetricsRegistry::new();
        let c1 = registry.counter("test");
        let c2 = registry.counter("test");
        c1.inc();
        assert_eq!(c2.get(), 1);

        let g1 = registry.gauge("test_gauge");
        let g2 = registry.gauge("test_gauge");
        g1.set(42);
        assert_eq!(g2.get(), 42);
    }

    #[test]
    fn prometheus_export() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("test_counter");
        counter.add(10);

        let output = registry.to_prometheus();
        assert!(output.contains("test_counter"));
        assert!(output.contains("# TYPE test_counter counter"));
    }
}
