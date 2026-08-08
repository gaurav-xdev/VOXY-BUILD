use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Thread priority levels for audio processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioThreadPriority {
    /// For audio capture/playback threads — highest priority.
    TimeCritical,
    /// For DSP processing — high priority.
    Highest,
    /// For monitoring/health checks — above normal.
    AboveNormal,
    /// Default priority.
    Normal,
}

/// Handle for a pinned audio thread.
pub struct AudioThreadHandle {
    is_active: Arc<AtomicBool>,
}

impl AudioThreadHandle {
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }
}

/// Set the current thread's priority on Windows.
///
/// # Safety
/// This calls `SetThreadPriority` which is safe on the calling thread.
pub fn set_thread_priority(priority: AudioThreadPriority) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;

        // THREAD_PRIORITY constants
        const THREAD_PRIORITY_TIME_CRITICAL: i32 = 15;
        const THREAD_PRIORITY_HIGHEST: i32 = 2;
        const THREAD_PRIORITY_ABOVE_NORMAL: i32 = 1;
        const THREAD_PRIORITY_NORMAL: i32 = 0;

        let win_priority = match priority {
            AudioThreadPriority::TimeCritical => THREAD_PRIORITY_TIME_CRITICAL,
            AudioThreadPriority::Highest => THREAD_PRIORITY_HIGHEST,
            AudioThreadPriority::AboveNormal => THREAD_PRIORITY_ABOVE_NORMAL,
            AudioThreadPriority::Normal => THREAD_PRIORITY_NORMAL,
        };

        extern "system" {
            fn SetThreadPriority(hthread: *mut c_void, npriority: i32) -> i32;
            fn GetCurrentThread() -> *mut c_void;
        }

        unsafe {
            let handle = GetCurrentThread();
            SetThreadPriority(handle, win_priority) != 0
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = priority;
        false
    }
}

/// Pin the current thread to a specific CPU core.
///
/// # Safety
/// This calls `SetThreadAffinityMask` which is safe on the calling thread.
pub fn pin_to_core(core_index: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;

        extern "system" {
            fn SetThreadAffinityMask(hthread: *mut c_void, dwthreadaffinitymask: usize) -> usize;
            fn GetCurrentThread() -> *mut c_void;
        }

        let mask = 1usize << core_index;
        unsafe {
            let handle = GetCurrentThread();
            SetThreadAffinityMask(handle, mask) != 0
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = core_index;
        false
    }
}

/// Set thread priority and optionally pin to a core.
/// Returns a handle for monitoring.
pub fn configure_audio_thread(
    priority: AudioThreadPriority,
    core_affinity: Option<u32>,
) -> AudioThreadHandle {
    let is_active = Arc::new(AtomicBool::new(true));

    set_thread_priority(priority);
    if let Some(core) = core_affinity {
        pin_to_core(core);
    }

    AudioThreadHandle { is_active }
}

/// Boost thread priority for a critical section, then restore.
/// Returns the restore function.
pub fn temporary_priority_boost(priority: AudioThreadPriority) -> impl FnOnce() {
    set_thread_priority(priority);
    || {
        set_thread_priority(AudioThreadPriority::Normal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_thread_priority_normal() {
        let result = set_thread_priority(AudioThreadPriority::Normal);
        assert!(result);
    }

    #[test]
    fn test_set_thread_priority_highest() {
        let result = set_thread_priority(AudioThreadPriority::Highest);
        assert!(result);
    }

    #[test]
    fn test_set_thread_priority_time_critical() {
        let result = set_thread_priority(AudioThreadPriority::TimeCritical);
        assert!(result);
    }

    #[test]
    fn test_pin_to_core() {
        let result = pin_to_core(0);
        assert!(result);
    }

    #[test]
    fn test_configure_audio_thread() {
        let handle = configure_audio_thread(AudioThreadPriority::Highest, Some(0));
        assert!(handle.is_active());
    }

    #[test]
    fn test_temporary_priority_boost() {
        let restore = temporary_priority_boost(AudioThreadPriority::TimeCritical);
        restore();
        let result = set_thread_priority(AudioThreadPriority::Normal);
        assert!(result);
    }
}
