#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

struct CountingSystem;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static ZEROED_COUNT: AtomicU64 = AtomicU64::new(0);
static ZEROED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static REALLOC_OLD_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_NEW_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            ZEROED_COUNT.fetch_add(1, Ordering::Relaxed);
            ZEROED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        // SAFETY: forwarding the caller's valid layout to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING.load(Ordering::Relaxed) {
            DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            DEALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        // SAFETY: forwarding the allocation and its original layout unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            REALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            REALLOC_OLD_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            REALLOC_NEW_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        }
        // SAFETY: forwarding the allocation, original layout, and requested size unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingSystem = CountingSystem;

#[derive(Clone, Copy)]
struct Counts {
    alloc_count: u64,
    alloc_bytes: u64,
    zeroed_count: u64,
    zeroed_bytes: u64,
    dealloc_count: u64,
    dealloc_bytes: u64,
    realloc_count: u64,
    realloc_old_bytes: u64,
    realloc_new_bytes: u64,
}

fn start_counting() {
    COUNTING.store(false, Ordering::SeqCst);
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    ZEROED_COUNT.store(0, Ordering::Relaxed);
    ZEROED_BYTES.store(0, Ordering::Relaxed);
    DEALLOC_COUNT.store(0, Ordering::Relaxed);
    DEALLOC_BYTES.store(0, Ordering::Relaxed);
    REALLOC_COUNT.store(0, Ordering::Relaxed);
    REALLOC_OLD_BYTES.store(0, Ordering::Relaxed);
    REALLOC_NEW_BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::SeqCst);
}

fn stop_counting() -> Counts {
    COUNTING.store(false, Ordering::SeqCst);
    Counts {
        alloc_count: ALLOC_COUNT.load(Ordering::Relaxed),
        alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
        zeroed_count: ZEROED_COUNT.load(Ordering::Relaxed),
        zeroed_bytes: ZEROED_BYTES.load(Ordering::Relaxed),
        dealloc_count: DEALLOC_COUNT.load(Ordering::Relaxed),
        dealloc_bytes: DEALLOC_BYTES.load(Ordering::Relaxed),
        realloc_count: REALLOC_COUNT.load(Ordering::Relaxed),
        realloc_old_bytes: REALLOC_OLD_BYTES.load(Ordering::Relaxed),
        realloc_new_bytes: REALLOC_NEW_BYTES.load(Ordering::Relaxed),
    }
}

#[derive(Clone, Copy)]
struct Observation {
    html_len: usize,
    html_checksum: u64,
    children: usize,
    arena_capacity_bytes: usize,
}

#[derive(Clone, Copy)]
struct Measurement {
    observation: Observation,
    counts: Counts,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Fresh,
    Reuse,
    Parse,
}

impl Mode {
    fn parse(value: &str) -> Self {
        match value {
            "fresh" => Self::Fresh,
            "reuse" => Self::Reuse,
            "parse" => Self::Parse,
            _ => panic!("unknown mode: {value}"),
        }
    }
}

fn checksum(bytes: &[u8]) -> u64 {
    // FNV-1a is deliberately kept allocation-free so it can run while counting.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

struct Engine {
    options: ParserOptions,
    html_options: HtmlRendererOptions,
    mode: Mode,
    source: &'static str,
    retained_allocator: Option<Allocator>,
    retained_renderer: Option<HtmlRenderer>,
}

impl Engine {
    fn options(profile: &str) -> ParserOptions {
        match profile {
            "commonmark" => ParserOptions::default(),
            "gfm" => ParserOptions::gfm(),
            "extensions" => ParserOptions {
                footnotes: true,
                superscript: true,
                subscript: true,
                smart_punctuation: true,
                math: true,
                definition_lists: true,
                heading_attributes: true,
                wiki_links: true,
                cjk_emphasis: true,
                ..ParserOptions::gfm()
            },
            "mdx" => ParserOptions::mdx(),
            _ => panic!("unknown profile: {profile}"),
        }
    }

    fn html_options(profile: &str) -> HtmlRendererOptions {
        HtmlRendererOptions {
            xhtml: true,
            hard_break: "<br />\n".into(),
            autolink_urls: false,
            autolink_target_blank: false,
            link_target_blank: false,
            disallow_raw_html: profile == "gfm",
            ..HtmlRendererOptions::new()
        }
    }

    fn new(profile: &str, mode: Mode, source: &'static str) -> Self {
        let mut options = Self::options(profile);
        if profile == "gfm" {
            options.footnotes = false;
        }
        let html_options = Self::html_options(profile);
        let retained_allocator = match mode {
            Mode::Reuse | Mode::Parse => Some(Allocator::for_source_len(source.len())),
            Mode::Fresh => None,
        };
        let retained_renderer = match mode {
            Mode::Reuse => Some(HtmlRenderer::with_options(html_options.clone())),
            Mode::Fresh | Mode::Parse => None,
        };
        Self { options, html_options, mode, source, retained_allocator, retained_renderer }
    }

    fn fresh_once(&self) -> Observation {
        let allocator = Allocator::for_source_len(self.source.len());
        let document = Parser::with_options(&allocator, self.source, self.options.clone())
            .parse()
            .expect("parse input");
        let children = document.children.len();
        let mut renderer = HtmlRenderer::with_options(self.html_options.clone());
        let html = renderer.render(&document);
        let observation = Observation {
            html_len: html.len(),
            html_checksum: checksum(html.as_bytes()),
            children,
            arena_capacity_bytes: allocator.allocated_bytes(),
        };
        // Keep every operation owned by this lifecycle inside the counting scope.
        drop(html);
        drop(renderer);
        drop(document);
        drop(allocator);
        observation
    }

    fn reuse_once(&mut self) -> Observation {
        let allocator = self.retained_allocator.as_mut().expect("retained allocator");
        let renderer = self.retained_renderer.as_mut().expect("retained renderer");
        let (observation, html_checksum, html_len) = {
            let document = Parser::with_options(allocator, self.source, self.options.clone())
                .parse()
                .expect("parse input");
            let children = document.children.len();
            let html = renderer.render_borrowed(&document);
            let html_len = html.len();
            let html_checksum = checksum(html.as_bytes());
            let arena_capacity_bytes = allocator.allocated_bytes();
            (
                Observation { html_len, html_checksum, children, arena_capacity_bytes },
                html_checksum,
                html_len,
            )
        };
        allocator.reset();
        // Keep these scalars explicit so the borrowed renderer output cannot escape reset.
        debug_assert_eq!(observation.html_checksum, html_checksum);
        debug_assert_eq!(observation.html_len, html_len);
        observation
    }

    fn parse_once(&mut self) -> Observation {
        let allocator = self.retained_allocator.as_mut().expect("retained allocator");
        let (children, arena_capacity_bytes) = {
            let document = Parser::with_options(allocator, self.source, self.options.clone())
                .parse()
                .expect("parse input");
            black_box(&document);
            (document.children.len(), allocator.allocated_bytes())
        };
        allocator.reset();
        Observation { html_len: 0, html_checksum: 0, children, arena_capacity_bytes }
    }

    fn once(&mut self) -> Observation {
        match self.mode {
            Mode::Fresh => self.fresh_once(),
            Mode::Reuse => self.reuse_once(),
            Mode::Parse => self.parse_once(),
        }
    }

    fn warmup(&mut self) {
        self.once();
    }

    fn measure(&mut self) -> Measurement {
        start_counting();
        let observation = self.once();
        let counts = stop_counting();
        Measurement { observation, counts }
    }
}

fn hex_u64(value: u64) -> String {
    format!("{value:016x}")
}

fn print_measurement(stdout: &mut impl Write, measurement: Measurement) {
    let o = measurement.observation;
    let c = measurement.counts;
    writeln!(
        stdout,
        "allocation {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        o.html_len,
        hex_u64(o.html_checksum),
        o.children,
        o.arena_capacity_bytes,
        c.alloc_count,
        c.alloc_bytes,
        c.zeroed_count,
        c.zeroed_bytes,
        c.dealloc_count,
        c.dealloc_bytes,
        c.realloc_count,
        c.realloc_old_bytes,
        c.realloc_new_bytes,
        // alloc_bytes includes alloc_zeroed; zeroed_bytes is its diagnostic subset.
        c.alloc_bytes,
    )
    .expect("write measurement");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() == 3, "allocation_worker PROFILE MODE INPUT");
    let profile = &args[0];
    let mode = Mode::parse(&args[1]);
    let bytes = std::fs::read(&args[2]).expect("read input");
    let source = Box::leak(String::from_utf8(bytes).expect("UTF-8 input").into_boxed_str());
    let mut engine = Engine::new(profile, mode, source);
    // Prime parser/renderer OnceLock state before the first counted operation.
    engine.warmup();

    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        match line.expect("read command").as_str() {
            "measure" => {
                print_measurement(&mut stdout, engine.measure());
                stdout.flush().expect("flush measurement");
            }
            "quit" => break,
            command => panic!("unknown command: {command}"),
        }
    }
}
