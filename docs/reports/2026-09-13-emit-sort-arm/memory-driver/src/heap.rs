use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
static CALLS: AtomicU64 = AtomicU64::new(0);
static REQUESTED: AtomicU64 = AtomicU64::new(0);
static ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct Counting;

fn allocated(old_size: usize, new_size: usize) {
    let live = if new_size >= old_size {
        LIVE.fetch_add((new_size - old_size) as u64, Relaxed)
            + (new_size - old_size) as u64
    } else {
        LIVE.fetch_sub((old_size - new_size) as u64, Relaxed)
            - (old_size - new_size) as u64
    };
    if ACTIVE.load(Relaxed) {
        PEAK.fetch_max(live, Relaxed);
        CALLS.fetch_add(1, Relaxed);
        REQUESTED.fetch_add(new_size as u64, Relaxed);
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            allocated(0, layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            allocated(0, layout.size());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, layout, new_size) };
        if !result.is_null() {
            // Logical requested live bytes: allocator-internal copying,
            // capacity rounding and transient old+new storage are not visible.
            allocated(layout.size(), new_size);
        }
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size() as u64, Relaxed);
    }
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
pub struct Stats {
    pub peak_live_bytes: u64,
    pub live_after_render: u64,
    pub requested_bytes: u64,
    pub allocation_calls: u64,
}

pub fn begin() -> u64 {
    assert!(!ACTIVE.load(Relaxed));
    let base = LIVE.load(Relaxed);
    PEAK.store(base, Relaxed);
    CALLS.store(0, Relaxed);
    REQUESTED.store(0, Relaxed);
    ACTIVE.store(true, Relaxed);
    base
}

pub fn end(base: u64) -> Stats {
    ACTIVE.store(false, Relaxed);
    Stats {
        peak_live_bytes: PEAK.load(Relaxed).checked_sub(base).unwrap(),
        live_after_render: remaining(base),
        requested_bytes: REQUESTED.load(Relaxed),
        allocation_calls: CALLS.load(Relaxed),
    }
}

pub fn remaining(base: u64) -> u64 {
    LIVE.load(Relaxed).checked_sub(base).unwrap()
}

pub fn self_test() {
    use std::alloc::{alloc, alloc_zeroed, dealloc, realloc};
    use std::hint::black_box;
    unsafe {
        // Keep an allocation alive outside the measured scope. It must be
        // excluded without losing correct accounting when later freed.
        let outside = alloc(Layout::from_size_align(64, 8).unwrap());
        assert!(!outside.is_null());
        let base = begin();
        let a = black_box(alloc(Layout::from_size_align(32, 8).unwrap()));
        let mut b = black_box(alloc_zeroed(Layout::from_size_align(16, 8).unwrap()));
        assert!(!a.is_null() && !b.is_null());
        dealloc(a, Layout::from_size_align(32, 8).unwrap());
        b = black_box(realloc(b, Layout::from_size_align(16, 8).unwrap(), 80));
        assert!(!b.is_null());
        assert_eq!(end(base), Stats {
            peak_live_bytes: 80, live_after_render: 80,
            requested_bytes: 128, allocation_calls: 3,
        });
        dealloc(b, Layout::from_size_align(80, 8).unwrap());
        assert_eq!(remaining(base), 0);
        let base = begin();
        let mut c = black_box(alloc(Layout::from_size_align(128, 8).unwrap()));
        assert!(!c.is_null());
        c = black_box(realloc(c, Layout::from_size_align(128, 8).unwrap(), 64));
        assert!(!c.is_null());
        assert_eq!(end(base), Stats {
            peak_live_bytes: 128, live_after_render: 64,
            requested_bytes: 192, allocation_calls: 2,
        });
        dealloc(c, Layout::from_size_align(64, 8).unwrap());
        assert_eq!(remaining(base), 0);
        dealloc(outside, Layout::from_size_align(64, 8).unwrap());
    }
}
