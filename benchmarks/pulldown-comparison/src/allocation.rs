use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use serde::Serialize;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static REALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

/// Allocation counters captured inside one explicit measurement window.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct AllocationSnapshot {
    /// Successful allocation calls.
    pub allocations: u64,
    /// Successful reallocation calls.
    pub reallocations: u64,
    /// Deallocation calls.
    pub deallocations: u64,
    /// Requested bytes across allocations and reallocations.
    pub allocated_bytes: u64,
    /// Released bytes reported by deallocation layouts.
    pub deallocated_bytes: u64,
}

/// System allocator wrapper used only by the diagnostic binary.
pub struct CountingAllocator;

// SAFETY: Every operation delegates to `System` with the original pointer and
// layout. The additional atomics only observe successful operations.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forwarded unchanged to the system allocator.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && ACTIVE.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if ACTIVE.load(Ordering::Relaxed) {
            DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        // SAFETY: Forwarded unchanged to the system allocator.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: Forwarded unchanged to the system allocator.
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() && ACTIVE.load(Ordering::Relaxed) {
            REALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        new_pointer
    }
}

/// RAII guard that enables counters until it is dropped.
pub struct MeasurementWindow {
    _private: (),
}

impl MeasurementWindow {
    /// Reset all counters and open a measurement window.
    pub fn start() -> Self {
        ACTIVE.store(false, Ordering::SeqCst);
        reset();
        ACTIVE.store(true, Ordering::SeqCst);
        Self { _private: () }
    }

    /// Close the window and return its counters.
    pub fn finish(self) -> AllocationSnapshot {
        ACTIVE.store(false, Ordering::SeqCst);
        let snapshot = snapshot();
        std::mem::forget(self);
        snapshot
    }
}

impl Drop for MeasurementWindow {
    fn drop(&mut self) {
        ACTIVE.store(false, Ordering::SeqCst);
    }
}

fn reset() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    REALLOCATIONS.store(0, Ordering::Relaxed);
    DEALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    DEALLOCATED_BYTES.store(0, Ordering::Relaxed);
}

fn snapshot() -> AllocationSnapshot {
    AllocationSnapshot {
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        reallocations: REALLOCATIONS.load(Ordering::Relaxed),
        deallocations: DEALLOCATIONS.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        deallocated_bytes: DEALLOCATED_BYTES.load(Ordering::Relaxed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_window_should_count_direct_allocator_calls() {
        let layout = Layout::from_size_align(64, 8).expect("valid layout");
        let window = MeasurementWindow::start();
        // SAFETY: The pointer is deallocated with the same allocator and layout.
        let pointer = unsafe { CountingAllocator.alloc(layout) };
        assert!(!pointer.is_null());
        // SAFETY: The successful allocation used the same allocator and layout.
        unsafe { CountingAllocator.dealloc(pointer, layout) };
        let result = window.finish();

        assert_eq!(result.allocations, 1);
    }

    #[test]
    fn disabled_window_should_ignore_direct_allocator_calls() {
        let layout = Layout::from_size_align(64, 8).expect("valid layout");
        let initial = MeasurementWindow::start();
        let _ = initial.finish();
        // SAFETY: The pointer is deallocated with the same allocator and layout.
        let pointer = unsafe { CountingAllocator.alloc(layout) };
        assert!(!pointer.is_null());
        // SAFETY: The successful allocation used the same allocator and layout.
        unsafe { CountingAllocator.dealloc(pointer, layout) };

        let verification = MeasurementWindow::start().finish();
        assert_eq!(verification.allocations, 0);
    }
}
