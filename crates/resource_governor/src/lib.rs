//! Resource budgets: CPU, memory, concurrency, timeouts, cancellation.

pub mod error;

pub use error::{GovernorError, Result};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Resource governor enforcing budgets.
pub struct ResourceGovernor {
    memory_budget_bytes: u64,
    cpu_budget_percent: f64,
    max_concurrency: usize,
    timeout_ms: u64,
    memory_used: Arc<AtomicU64>,
    active_tasks: Arc<AtomicU64>,
}

impl ResourceGovernor {
    /// Create a new resource governor.
    pub fn new(
        memory_budget_mb: u64,
        cpu_budget_percent: f64,
        max_concurrency: usize,
        timeout_ms: u64,
    ) -> Self {
        Self {
            memory_budget_bytes: memory_budget_mb * 1024 * 1024,
            cpu_budget_percent,
            max_concurrency,
            timeout_ms,
            memory_used: Arc::new(AtomicU64::new(0)),
            active_tasks: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get the CPU budget.
    pub fn cpu_budget_percent(&self) -> f64 {
        self.cpu_budget_percent
    }
    pub fn within_memory_budget(&self) -> bool {
        self.memory_used.load(Ordering::Relaxed) < self.memory_budget_bytes
    }

    /// Check if we're within concurrency budget.
    pub fn within_concurrency_budget(&self) -> bool {
        self.active_tasks.load(Ordering::Relaxed) < self.max_concurrency as u64
    }

    /// Record memory allocation.
    pub fn allocate_memory(&self, bytes: u64) {
        self.memory_used.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record memory deallocation (saturating to prevent underflow).
    pub fn deallocate_memory(&self, bytes: u64) {
        let mut prev = self.memory_used.load(Ordering::Relaxed);
        loop {
            let new = prev.saturating_sub(bytes);
            match self.memory_used.compare_exchange_weak(
                prev,
                new,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => {
                    if actual == prev {
                        break;
                    }
                    prev = actual;
                }
            }
        }
    }

    /// Acquire a task slot (CAS loop to prevent TOCTOU race).
    pub fn acquire_task(&self) -> bool {
        let current = self.active_tasks.load(Ordering::Relaxed);
        loop {
            if current >= self.max_concurrency as u64 {
                return false;
            }
            match self.active_tasks.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => {
                    if actual >= self.max_concurrency as u64 {
                        return false;
                    }
                    // retry with actual value
                }
            }
        }
    }

    /// Release a task slot.
    /// SECURITY: Uses saturating subtraction to prevent underflow wrapping
    /// to u64::MAX which would permanently block all task acquisitions.
    pub fn release_task(&self) {
        let mut prev = self.active_tasks.load(Ordering::Relaxed);
        loop {
            if prev == 0 {
                // Already at zero — avoid underflow
                break;
            }
            let new = prev - 1;
            match self.active_tasks.compare_exchange_weak(
                prev,
                new,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => {
                    if actual == 0 {
                        break;
                    }
                    prev = actual;
                }
            }
        }
    }

    /// Get the timeout.
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// Get current memory usage.
    pub fn memory_used_bytes(&self) -> u64 {
        self.memory_used.load(Ordering::Relaxed)
    }

    /// Get current task count.
    pub fn active_task_count(&self) -> u64 {
        self.active_tasks.load(Ordering::Relaxed)
    }
}

impl Default for ResourceGovernor {
    fn default() -> Self {
        Self::new(2048, 80.0, 64, 30000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governor_creation() {
        let gov = ResourceGovernor::new(1024, 50.0, 32, 10000);
        assert!(gov.within_memory_budget());
        assert!(gov.within_concurrency_budget());
    }

    #[test]
    fn memory_tracking() {
        let gov = ResourceGovernor::new(1, 50.0, 32, 10000); // 1MB budget
        gov.allocate_memory(512 * 1024);
        assert!(gov.within_memory_budget());

        gov.allocate_memory(600 * 1024);
        assert!(!gov.within_memory_budget());
    }

    #[test]
    fn concurrency_tracking() {
        let gov = ResourceGovernor::new(1024, 50.0, 2, 10000);
        assert!(gov.acquire_task());
        assert!(gov.acquire_task());
        assert!(!gov.acquire_task());

        gov.release_task();
        assert!(gov.acquire_task());
    }
}
