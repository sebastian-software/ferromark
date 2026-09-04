// Reproduce: copy into examples/memory_probe.rs in the revision under test, then
// cargo run --locked --profile release-debug --features mdx --example memory_probe -- 8192
// Measures live allocator requests, not OS RSS. Run without other allocations in this thread.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
struct Counter;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn add(size: usize) {
    let total = LIVE.fetch_add(size, Relaxed) + size;
    PEAK.fetch_max(total, Relaxed);
}
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            add(layout.size());
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, layout, size) };
        if !result.is_null() {
            LIVE.fetch_sub(layout.size(), Relaxed);
            add(size);
        }
        result
    }
}
#[global_allocator]
static ALLOC: Counter = Counter;
fn main() {
    let n: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let source = "Text with **emphasis**.\n\n<Widget />\n\n".repeat(n);
    let baseline = LIVE.load(Relaxed);
    PEAK.store(baseline, Relaxed);
    let result = ferromark::mdx::render_with_options(&source, &ferromark::Options::default());
    let peak = PEAK.load(Relaxed) - baseline;
    println!(
        "n={n} output_bytes={} peak_additional_live_requested_bytes={peak}",
        result.body.len()
    );
}
