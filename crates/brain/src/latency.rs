use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencySnapshot {
    pub total_us: u64,
    pub context_us: u64,
    pub companion_us: u64,
    pub hdr_us: u64,
    pub cognition_us: u64,
    pub overhead_us: u64,
}

#[derive(Debug, Clone, Default)]
pub struct LatencyTracker {
    total: LatencySnapshot,
    history: Vec<LatencySnapshot>,
    max_history: usize,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            total: LatencySnapshot::default(),
            history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn start_turn(&mut self) -> TurnTimer {
        TurnTimer {
            start: Instant::now(),
            context_start: None,
            companion_start: None,
            hdr_start: None,
            cognition_start: None,
        }
    }

    pub fn record(&mut self, snapshot: LatencySnapshot) {
        self.total = snapshot.clone();
        self.history.push(snapshot);
        if self.history.len() > self.max_history {
            self.history.swap_remove(0);
        }
    }

    pub fn current(&self) -> &LatencySnapshot {
        &self.total
    }

    pub fn average(&self) -> LatencySnapshot {
        if self.history.is_empty() {
            return LatencySnapshot::default();
        }
        let count = self.history.len() as u64;
        LatencySnapshot {
            total_us: self.history.iter().map(|s| s.total_us).sum::<u64>() / count,
            context_us: self.history.iter().map(|s| s.context_us).sum::<u64>() / count,
            companion_us: self.history.iter().map(|s| s.companion_us).sum::<u64>() / count,
            hdr_us: self.history.iter().map(|s| s.hdr_us).sum::<u64>() / count,
            cognition_us: self.history.iter().map(|s| s.cognition_us).sum::<u64>() / count,
            overhead_us: self.history.iter().map(|s| s.overhead_us).sum::<u64>() / count,
        }
    }

    pub fn p99(&self) -> LatencySnapshot {
        if self.history.is_empty() {
            return LatencySnapshot::default();
        }
        let mut sorted = self.history.clone();
        sorted.sort_by_key(|s| s.total_us);
        let p99_idx = (sorted.len() as f64 * 0.99) as usize;
        sorted[p99_idx.min(sorted.len() - 1)].clone()
    }

    pub fn history(&self) -> &[LatencySnapshot] {
        &self.history
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.total = LatencySnapshot::default();
    }
}

pub struct TurnTimer {
    start: Instant,
    context_start: Option<Instant>,
    companion_start: Option<Instant>,
    hdr_start: Option<Instant>,
    cognition_start: Option<Instant>,
}

impl TurnTimer {
    pub fn begin_context(&mut self) {
        self.context_start = Some(Instant::now());
    }

    pub fn end_context(&mut self) -> u64 {
        self.context_start
            .take()
            .map(|s| s.elapsed().as_micros() as u64)
            .unwrap_or(0)
    }

    pub fn begin_companion(&mut self) {
        self.companion_start = Some(Instant::now());
    }

    pub fn end_companion(&mut self) -> u64 {
        self.companion_start
            .take()
            .map(|s| s.elapsed().as_micros() as u64)
            .unwrap_or(0)
    }

    pub fn begin_hdr(&mut self) {
        self.hdr_start = Some(Instant::now());
    }

    pub fn end_hdr(&mut self) -> u64 {
        self.hdr_start
            .take()
            .map(|s| s.elapsed().as_micros() as u64)
            .unwrap_or(0)
    }

    pub fn begin_cognition(&mut self) {
        self.cognition_start = Some(Instant::now());
    }

    pub fn end_cognition(&mut self) -> u64 {
        self.cognition_start
            .take()
            .map(|s| s.elapsed().as_micros() as u64)
            .unwrap_or(0)
    }

    pub fn finish(self) -> LatencySnapshot {
        let total_us = self.start.elapsed().as_micros() as u64;
        LatencySnapshot {
            total_us,
            context_us: 0,
            companion_us: 0,
            hdr_us: 0,
            cognition_us: 0,
            overhead_us: 0,
        }
    }
}
