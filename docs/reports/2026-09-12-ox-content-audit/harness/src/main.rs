use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
use serde_json::json;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
#[path = "../../ox-current/crates/ox_content_renderer/tests/spec_support/normalize.rs"]
mod normalize;
fn ferro_options(mode: &str) -> ferro::Options {
    let mut o = if mode.contains("default") {
        ferro::Options::default()
    } else {
        ferro::Options::commonmark()
    };
    if !mode.contains("default") {
        o.render_policy = ferro::RenderPolicy::Trusted;
        o.heading_ids = true;
        o.tables = mode.contains("tables");
    }
    if mode.contains("noids") {
        o.heading_ids = false;
    }
    o
}
fn old_options(mode: &str) -> ferro_old::Options {
    let mut o = if mode.contains("default") {
        ferro_old::Options::default()
    } else {
        ferro_old::Options::commonmark()
    };
    if !mode.contains("default") {
        o.render_policy = ferro_old::RenderPolicy::Trusted;
        o.heading_ids = true;
        o.tables = mode.contains("tables");
    }
    if mode.contains("noids") {
        o.heading_ids = false;
    }
    o
}
fn ox_options(mode: &str) -> (ParserOptions, HtmlRendererOptions) {
    let p = ParserOptions {
        tables: mode.contains("tables"),
        ..Default::default()
    };
    let mut h = HtmlRendererOptions::default();
    if !mode.contains("default") {
        h.autolink_urls = false;
        h.autolink_target_blank = false;
        h.link_target_blank = false;
    }
    (p, h)
}
fn operation(mode: &str) -> Box<dyn FnMut(&str) -> String> {
    if mode.starts_with("old") {
        let o = old_options(mode);
        return Box::new(move |s| ferro_old::to_html_with_options(s, &o));
    }
    if mode.starts_with("ferro") {
        let o = ferro_options(mode);
        if mode.contains("reuse") {
            let mut renderer = ferro::Renderer::with_options(o);
            return Box::new(move |s| renderer.render(s));
        }
        return Box::new(move |s| ferro::to_html_with_options(s, &o));
    }
    let (p, h) = ox_options(mode);
    let presize = !mode.contains("grow");
    let reuse = mode.contains("reuse");
    let mut renderer = HtmlRenderer::with_options(h.clone());
    Box::new(move |s| {
        let arena = if presize {
            Allocator::for_source_len(s.len())
        } else {
            Allocator::new()
        };
        let doc = Parser::with_options(&arena, s, p.clone()).parse().unwrap();
        if reuse {
            renderer.render(&doc)
        } else {
            HtmlRenderer::with_options(h.clone()).render(&doc)
        }
    })
}
fn window(op: &mut dyn FnMut(&str) -> String, s: &str, ms: u64) -> f64 {
    let start = Instant::now();
    let mut n = 0;
    while start.elapsed() < Duration::from_millis(ms) {
        for _ in 0..16 {
            black_box(op(black_box(s)));
            n += 1;
        }
    }
    start.elapsed().as_secs_f64() * 1e9 / n as f64
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "profile" {
        let sample = include_str!("../sample.md");
        let s = if args[3] == "short" {
            "Hello **world**!\n".into()
        } else {
            vec![sample; 100].join("\n\n")
        };
        let mut op = operation(&args[2]);
        window(&mut op, &s, 8000);
        return;
    }
    if args[1] == "conformance" {
        let examples: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
        for mode in [
            "old-default",
            "old-cm-noids",
            "ferro-default",
            "ferro-cm-noids",
            "ox-cm",
        ] {
            let mut op = operation(mode);
            let mut failures = vec![];
            for x in examples.as_array().unwrap() {
                let actual = op(x["markdown"].as_str().unwrap());
                if normalize::normalize_html(&actual)
                    != normalize::normalize_html(x["html"].as_str().unwrap())
                {
                    failures.push(json!({"example":x["example"],"markdown":x["markdown"],"expected":x["html"],"actual":actual}));
                }
            }
            println!(
                "{}",
                json!({"mode":mode,"total":examples.as_array().unwrap().len(),"failed":failures.len(),"failures":failures})
            );
        }
        return;
    }
    let sample = include_str!("../sample.md");
    let cases = if args[1] == "reuse" {
        vec![
            ("short", "Hello **world**!\n".into()),
            ("upstream-large", vec![sample; 100].join("\n\n")),
        ]
    } else if args[1] == "baseline" {
        vec![
            ("upstream-large", vec![sample; 100].join("\n\n")),
            ("upstream-huge", vec![sample; 2150].join("\n\n")),
        ]
    } else {
        vec![("upstream-large",vec![sample;100].join("\n\n")),("upstream-huge",vec![sample;2150].join("\n\n")),("short","Hello **world**!\n".into()),("plain","A plain paragraph with some ordinary words and no formatting.\n\n".repeat(100)),("inline","A **bold** paragraph with *emphasis*, `code` and [a link](https://example.com).\n\n".repeat(100)),("lists","- First item\n- Second **bold** item\n  - Nested item\n\n".repeat(100)),("code","```javascript\nfunction hello() { return 1 < 2 && true; }\n```\n\n".repeat(100)),("tables","| Header 1 | Header 2 |\n|---|---|\n| Cell 1 | Cell 2 |\n| Cell 3 | Cell 4 |\n\n".repeat(100)),("headings","# Heading 1\n\n## Heading 2\n\n### Heading 3\n\n".repeat(100))]
    };
    let modes: Vec<&str> = if args[1] == "reuse" {
        vec!["ferro-cm", "ferro-cm-reuse", "ox-cm", "ox-cm-reuse"]
    } else if args[1] == "baseline" {
        vec!["old-default", "ferro-default", "ox-default"]
    } else {
        vec![
            "old-default",
            "old-cm",
            "ferro-default",
            "ferro-cm",
            "ferro-cm-noids",
            "ferro-tables",
            "ox-default",
            "ox-cm",
            "ox-tables",
            "ox-cm-grow",
            "ox-cm-reuse",
        ]
    };
    for (case, s) in cases {
        if args[1] == "verify" {
            for (a, b) in [("ferro-cm", "ox-cm"), ("ferro-tables", "ox-tables")] {
                let x = operation(a)(&s);
                let y = operation(b)(&s);
                println!(
                    "{}",
                    json!({"case":case,"a":a,"b":b,"normalized_equal":normalize::normalize_html(&x)==normalize::normalize_html(&y)})
                );
            }
            continue;
        }
        #[cfg(feature = "allocations")]
        {
            for mode in &modes {
                let mut op = operation(mode);
                black_box(op(&s));
                allocation::reset();
                for _ in 0..10 {
                    black_box(op(&s));
                }
                let (calls, bytes) = allocation::snapshot();
                println!(
                    "{}",
                    json!({"case":case,"mode":mode,"allocation_calls":calls/10,"requested_bytes":bytes/10})
                );
            }
            continue;
        }
        #[cfg(not(feature = "allocations"))]
        {
            let mut ops: Vec<_> = modes.iter().map(|m| operation(m)).collect();
            for op in &mut ops {
                window(op, &s, 150);
            }
            let mut windows = vec![vec![]; modes.len()];
            for round in 0..11 {
                for j in 0..modes.len() {
                    let i = (j + round) % modes.len();
                    windows[i].push(window(&mut ops[i], &s, 50));
                }
            }
            for (i, mode) in modes.iter().enumerate() {
                let output = ops[i](&s);
                println!(
                    "{}",
                    json!({"case":case,"mode":mode,"bytes":s.len(),"windows_ns":windows[i],"output_bytes":output.len(),"tables":output.matches("<table").count(),"normalized":if case=="upstream-large" {Some(normalize::normalize_html(&output))}else{None}})
                );
            }
        }
    }
}

#[cfg(feature = "allocations")]
mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    static CALLS: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    pub struct Counting;
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
            unsafe { System.alloc(l) }
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
            unsafe { System.alloc_zeroed(l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n as u64, Relaxed);
            unsafe { System.realloc(p, l, n) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            unsafe { System.dealloc(p, l) }
        }
    }
    pub fn reset() {
        CALLS.store(0, Relaxed);
        BYTES.store(0, Relaxed);
    }
    pub fn snapshot() -> (u64, u64) {
        (CALLS.load(Relaxed), BYTES.load(Relaxed))
    }
}
#[cfg(feature = "allocations")]
#[global_allocator]
static GLOBAL: allocation::Counting = allocation::Counting;
