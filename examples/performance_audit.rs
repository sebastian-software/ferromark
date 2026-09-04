//! Reproducible scaling and allocation probes for the September 2026 audit.
//! Run with `cargo run --profile release-debug --features mdx --example performance_audit`.
//! Pass `check` for small correctness reproducers, `alloc` for allocation counts,
//! or `spin <case> <size> <seconds>` for a sampling-profiler workload.

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::time::{Duration, Instant};

struct CountingAllocator;
static COUNTING: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(size, Relaxed);
        }
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn input(case: &str, n: usize) -> String {
    match case {
        "trailing_parens" => format!("https://example.com/{}", ")".repeat(n)),
        "angle_starts" => format!("a {}", "<".repeat(n)),
        "autolink_ranges" => "<https://example.com> ".repeat(n),
        "mdx_segments" => "Text with **emphasis**.\n\n<Widget />\n\n".repeat(n),
        "plain_segments" => "Text with **emphasis**.\n\n".repeat(n),
        "commonmark" => include_str!("../benches/fixtures/commonmark-50k.md").into(),
        "tables" => include_str!("../benches/fixtures/tables-5k.md").into(),
        "footnotes" => (0..n)
            .map(|i| format!("Text[^n{i}].\n\n[^n{i}]: Note {i}.\n\n"))
            .collect(),
        _ => panic!("unknown case {case}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "check") {
        for source in ["# foo\n\n# foo\n\n# foo-1\n", "# foo-1\n\n# foo\n\n# foo\n"] {
            println!("heading source={source:?}\n{}", ferromark::to_html(source));
        }
        #[cfg(feature = "mdx")]
        for source in [
            "# foo\n\n<X />\n\n# foo\n",
            "a[^x]\n\n<X />\n\n[^x]: note\n",
            "^[first]\n\n<X />\n\n^[second]\n",
        ] {
            let options = ferromark::options!(ferromark::Options::default(); footnotes: true, inline_footnotes: true,);
            println!(
                "MDX source={source:?}\n{}",
                ferromark::mdx::render_with_options(source, &options).body
            );
        }
        return;
    }
    let spin = args.first().is_some_and(|a| a == "spin");
    let allocations = args.first().is_some_and(|a| a == "alloc");
    let selected = args.get(1).map(String::as_str);
    let cases = [
        "trailing_parens",
        "angle_starts",
        "autolink_ranges",
        "mdx_segments",
        "plain_segments",
        "commonmark",
        "tables",
        "footnotes",
    ];
    for case in cases {
        if selected.is_some_and(|s| s != case) {
            continue;
        }
        let sizes = if spin || allocations {
            vec![args.get(2).map_or(128, |s| s.parse().unwrap())]
        } else {
            vec![1024, 2048, 4096, 8192]
        };
        for n in sizes {
            if matches!(case, "commonmark" | "tables") && n != sizes_first(spin, allocations, &args)
            {
                continue;
            }
            let source = input(case, n);
            let options = ferromark::options!(ferromark::Options::default();
                footnotes: case == "footnotes",
                autolink_literals: matches!(case, "trailing_parens" | "autolink_ranges"),);
            let mut renderer = ferromark::Renderer::with_options(options.clone());
            let mut out = Vec::new();
            for reuse in [false, true] {
                if case == "mdx_segments" && reuse {
                    continue;
                }
                let mut render = || {
                    #[cfg(feature = "mdx")]
                    if case == "mdx_segments" {
                        return black_box(
                            ferromark::mdx::render_with_options(black_box(&source), &options).body,
                        )
                        .len();
                    }
                    if reuse {
                        renderer.render_into(black_box(&source), &mut out);
                    } else {
                        ferromark::to_html_into_with_options(
                            black_box(&source),
                            &mut out,
                            &options,
                        );
                    }
                    black_box(&out).len()
                };
                let output_bytes = render();
                if allocations {
                    CALLS.store(0, Relaxed);
                    BYTES.store(0, Relaxed);
                    COUNTING.store(true, Relaxed);
                    render();
                    COUNTING.store(false, Relaxed);
                    println!(
                        "{}",
                        serde_json::json!({"case":case,"n":n,"input_bytes":source.len(),"reuse":reuse,"alloc_realloc_calls":CALLS.load(Relaxed),"requested_bytes":BYTES.load(Relaxed)})
                    );
                    continue;
                }
                let duration = if spin {
                    Duration::from_secs(args[3].parse().unwrap())
                } else {
                    Duration::from_millis(150)
                };
                let mut samples = Vec::new();
                for _ in 0..if spin { 1 } else { 5 } {
                    let start = Instant::now();
                    let mut iterations = 0;
                    while start.elapsed() < duration {
                        render();
                        iterations += 1;
                    }
                    samples.push(start.elapsed().as_secs_f64() * 1e9 / iterations as f64);
                }
                samples.sort_by(f64::total_cmp);
                println!(
                    "{}",
                    serde_json::json!({"case":case,"n":n,"input_bytes":source.len(),"output_bytes":output_bytes,"reuse":reuse,"median_ns":samples[samples.len()/2],"min_ns":samples[0],"max_ns":samples[samples.len()-1]})
                );
                if spin {
                    break;
                }
            }
        }
    }
}

fn sizes_first(spin: bool, allocations: bool, args: &[String]) -> usize {
    if spin || allocations {
        args.get(2).map_or(128, |s| s.parse().unwrap())
    } else {
        1024
    }
}
