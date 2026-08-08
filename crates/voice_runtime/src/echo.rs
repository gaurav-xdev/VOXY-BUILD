use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

pub struct EchoCanceller {
    enabled: bool,
    tail_length: usize,
    reference_buffer: Arc<Mutex<VecDeque<f32>>>,
    is_active: Arc<AtomicBool>,
    suppression_factor: f32,
    frame_count: Arc<AtomicU64>,
}

impl EchoCanceller {
    pub fn new(enabled: bool, tail_ms: u32, sample_rate: u32) -> Self {
        let tail_samples = (sample_rate as usize * tail_ms as usize / 1000).max(256);
        Self {
            enabled,
            tail_length: tail_samples,
            reference_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(tail_samples * 2))),
            is_active: Arc::new(AtomicBool::new(false)),
            suppression_factor: 0.85,
            frame_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_suppression_factor(mut self, factor: f32) -> Self {
        self.suppression_factor = factor.clamp(0.0, 1.0);
        self
    }

    pub fn process_capture(&self, input: &mut [f32]) {
        if !self.enabled {
            return;
        }

        self.frame_count.fetch_add(1, Ordering::Relaxed);

        {
            let mut buf = self.reference_buffer.lock();
            for &sample in input.iter() {
                if buf.len() >= self.tail_length {
                    buf.pop_front();
                }
                buf.push_back(sample);
            }
        }
    }

    pub fn process_playback(&self, output: &[f32]) {
        if !self.enabled {
            return;
        }

        {
            let mut buf = self.reference_buffer.lock();
            for &sample in output.iter() {
                if buf.len() >= self.tail_length {
                    buf.pop_front();
                }
                buf.push_back(sample);
            }
        }
    }

    pub fn process_input(&self, input: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let reference: Vec<f32> = {
            let buf = self.reference_buffer.lock();
            buf.iter().cloned().collect()
        };

        if reference.is_empty() {
            return;
        }

        for sample in input.iter_mut() {
            let echo_estimate = self.estimate_echo(*sample, &reference);
            *sample = (*sample - echo_estimate * self.suppression_factor).clamp(-1.0, 1.0);
        }
    }

    fn estimate_echo(&self, sample: f32, reference: &[f32]) -> f32 {
        if reference.is_empty() {
            return 0.0;
        }

        let mut correlation = 0.0f32;
        let mut energy = 0.0f32;

        let tap_count = reference.len().min(64);
        let step = if reference.len() > tap_count {
            reference.len() / tap_count
        } else {
            1
        };

        for (i, &ref_sample) in reference.iter().step_by(step).take(tap_count).enumerate() {
            let weight = 1.0 - (i as f32 / tap_count as f32);
            correlation += sample * ref_sample * weight;
            energy += ref_sample * ref_sample * weight;
        }

        if energy > 1e-10 {
            correlation / energy
        } else {
            0.0
        }
    }

    pub fn reset(&self) {
        self.reference_buffer.lock().clear();
        self.is_active.store(false, Ordering::SeqCst);
    }

    pub fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(Ordering::Relaxed)
    }
}
