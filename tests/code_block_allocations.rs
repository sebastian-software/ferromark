//! The plain fenced-code path must not allocate per block.
//!
//! `render_code_block`'s default route resolves a fence's language into a
//! borrow of the source and reserves the output once before writing, so a
//! renderer whose buffer is already warm should emit a fence-heavy document
//! without touching the allocator at all. A regression that reintroduces an
//! owned language string, a joined class list, or an unreserved growth step
//! would show up here as a non-zero count.
//!
//! The allocator is process-global, but its counter is scoped to the measuring
//! thread so unrelated allocations from the test harness cannot affect a
//! measurement.

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use ferromark::{Allocator, HtmlRenderer, Parser};

struct CountingSystem;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

impl CountingSystem {
    /// Records an allocation made by the thread whose work is being measured.
    fn record() {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
    }
}

// SAFETY: every operation forwards its original pointer, layout and size to
// the system allocator unchanged; the counter is the only added effect.
unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::record();
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::record();
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarding the allocation and its original layout unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A `String` grow arrives here rather than at `alloc`, so buffer growth
        // has to count too.
        Self::record();
        // SAFETY: forwarding the allocation, original layout, and requested size unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingSystem = CountingSystem;

/// Runs `work` and reports the allocations it made on this thread.
fn allocations<T>(work: impl FnOnce() -> T) -> (T, u64) {
    ALLOCATIONS.with(|count| count.set(0));
    COUNTING.with(|flag| flag.set(true));
    let value = work();
    COUNTING.with(|flag| flag.set(false));
    (value, ALLOCATIONS.with(Cell::get))
}

/// A document of nothing but bare-language fences, so every allocation the
/// measurement sees belongs to the plain fenced-code path.
fn bare_language_fences(count: usize) -> String {
    const LANGUAGES: [&str; 5] = ["ts", "js", "bash", "rust", "json"];

    let mut source = String::new();
    for index in 0..count {
        source.push_str("```");
        source.push_str(LANGUAGES[index % LANGUAGES.len()]);
        source.push_str("\nconst value = ");
        source.push_str(&index.to_string());
        source.push_str(";\nconst other = value < 2 && value > 0;\n```\n\n");
    }
    source
}

/// Allocation count of one render by a renderer that has already rendered the
/// same document, so its output buffer and per-render maps are warm.
fn steady_state_allocations(source: &str) -> u64 {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().expect("parses");
    let mut renderer = HtmlRenderer::new();

    // `render_borrowed` keeps the output buffer instead of handing it away, so
    // repeated renders reach a steady state. Four warm-ups are more than the
    // buffer needs to stop growing.
    let mut warm = 0;
    for _ in 0..4 {
        warm = renderer.render_borrowed(&document).len();
    }
    assert!(warm > 0, "the fixture should render to something");

    let (rendered, allocations) = allocations(|| renderer.render_borrowed(&document).len());

    assert_eq!(rendered, warm, "steady-state renders must agree");
    allocations
}

/// Guards the harness itself: it must count a local allocation while ignoring
/// an allocation made on a concurrent thread during the same measurement.
fn assert_the_counter_is_thread_local() {
    let ready = std::sync::Arc::new(AtomicBool::new(false));
    let start = std::sync::Arc::new(AtomicBool::new(false));
    let done = std::sync::Arc::new(AtomicBool::new(false));
    let worker_ready = std::sync::Arc::clone(&ready);
    let worker_start = std::sync::Arc::clone(&start);
    let worker_done = std::sync::Arc::clone(&done);

    let worker = thread::spawn(move || {
        worker_ready.store(true, Ordering::Release);
        while !worker_start.load(Ordering::Acquire) {
            thread::yield_now();
        }
        black_box(Vec::<u8>::with_capacity(4096));
        worker_done.store(true, Ordering::Release);
    });

    while !ready.load(Ordering::Acquire) {
        thread::yield_now();
    }
    let (probe, allocations) = allocations(|| {
        start.store(true, Ordering::Release);
        let local_probe = Vec::<u8>::with_capacity(4096);
        while !done.load(Ordering::Acquire) {
            thread::yield_now();
        }
        local_probe
    });
    worker
        .join()
        .expect("the allocator probe thread should finish");

    assert!(probe.capacity() >= 4096);
    assert_eq!(
        allocations, 1,
        "the counter should count the local allocation and ignore the foreign thread"
    );
}

#[test]
fn a_warm_renderer_emits_bare_language_fences_without_allocating() {
    assert_the_counter_is_thread_local();

    for count in [1, 32, 256] {
        let source = bare_language_fences(count);
        let allocations = steady_state_allocations(&source);
        assert_eq!(
            allocations, 0,
            "rendering {count} bare-language fences allocated {allocations} times"
        );
    }
}
