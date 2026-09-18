
//! Timing harness for the nested-link shapes of issue #350.
//!
//! usage: probe <shape> <n> <groups> <cap> [profile]
//!        probe stack <shape> <n> <stack_kib>   parse on a fixed stack
//!        probe file <path>                      parse and render one file
//!        probe one <profile> <source>           render one source
//!        probe fuzz <seed> <count> <max_tokens> the differential corpus
//! shape: nest-inline | nest-ref | nest-plain | one-closer | nest-image | mdx-open
//! cap: max_nesting_depth (0 = unlimited)
//! profile: default | gfm | wiki | mdx

mod fuzz;
use std::time::Instant;

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

fn build(shape: &str, n: usize, groups: usize) -> String {
    let unit = match shape {
        "nest-inline" => "[".repeat(n) + "a" + &"](u)".repeat(n),
        "nest-ref" => "[".repeat(n) + "a" + &"][r]".repeat(n),
        "nest-plain" => "[".repeat(n) + "a" + &"]".repeat(n),
        "one-closer" => "[".repeat(n) + "a](u)",
        "nest-image" => "[![".repeat(n) + "a" + &"](i)](u)".repeat(n),
        "mdx-open" => "<A>".repeat(n),
        other => panic!("unknown shape {other}"),
    };
    let mut out = String::new();
    for _ in 0..groups {
        out.push_str(&unit);
    }
    if shape == "nest-ref" {
        out.push_str("\n\n[r]: /r\n");
    }
    out
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "stack" {
        // probe stack <shape> <n> <stack_kib>
        let shape = args[2].clone();
        let n: usize = args[3].parse().unwrap();
        let kib: usize = args[4].parse().unwrap();
        let source = build(&shape, n, 1);
        let handle = std::thread::Builder::new()
            .stack_size(kib * 1024)
            .spawn(move || {
                let mut options = ParserOptions::default();
                options.max_nesting_depth = 0;
                let allocator = Allocator::for_source_len(source.len());
                Parser::with_options(&allocator, &source, options)
                    .parse()
                    .is_ok()
            })
            .unwrap();
        println!("stack {shape} n={n} kib={kib} parsed={:?}", handle.join());
        return;
    }
    if args[1] == "file" {
        let source = std::fs::read_to_string(&args[2]).unwrap();
        let mut best = f64::MAX;
        let started = Instant::now();
        let mut runs = 0;
        let mut hash = 0;
        while runs < 5 || (runs < 20_000 && started.elapsed().as_secs_f64() < 2.0) {
            runs += 1;
            let at = Instant::now();
            let allocator = Allocator::for_source_len(source.len());
            let document = Parser::with_options(&allocator, &source, ParserOptions::gfm())
                .parse()
                .unwrap();
            let html = HtmlRenderer::with_options(HtmlRendererOptions::new()).render(&document);
            best = best.min(at.elapsed().as_secs_f64());
            hash = fnv(html.as_bytes());
        }
        println!(
            "{} bytes={} runs={runs} best={:.9}s hash={hash:016x}",
            args[2],
            source.len(),
            best
        );
        return;
    }
    if args[1] == "one" {
        fuzz::one(args[2].parse().unwrap(), &args[3]);
        return;
    }
    if args[1] == "fuzz" {
        fuzz::run(
            args[2].parse().unwrap(),
            args[3].parse().unwrap(),
            args[4].parse().unwrap(),
        );
        return;
    }
    let shape = args[1].clone();
    let n: usize = args[2].parse().unwrap();
    let groups: usize = args[3].parse().unwrap();
    let cap: usize = args[4].parse().unwrap();
    let profile = args.get(5).map_or("default", String::as_str);
    let source = build(&shape, n, groups);

    let mut best = f64::MAX;
    let mut summary = String::new();
    let deadline = Instant::now();
    let mut runs = 0;
    while runs < 3 || (runs < 200 && deadline.elapsed().as_secs_f64() < 1.0) {
        runs += 1;
        let mut options = match profile {
            "gfm" => ParserOptions::gfm(),
            "wiki" => {
                let mut o = ParserOptions::gfm();
                o.wiki_links = true;
                o
            }
            "mdx" => {
                let mut o = ParserOptions::default();
                o.mdx = true;
                o
            }
            _ => ParserOptions::default(),
        };
        options.max_nesting_depth = cap;
        let started = Instant::now();
        let allocator = Allocator::for_source_len(source.len());
        let parsed = Parser::with_options(&allocator, &source, options).parse();
        let out = match parsed {
            Ok(document) => {
                let html = HtmlRenderer::with_options(HtmlRendererOptions::new()).render(&document);
                format!("ok len={} hash={:016x}", html.len(), fnv(html.as_bytes()))
            }
            Err(error) => format!("err {error}"),
        };
        let elapsed = started.elapsed().as_secs_f64();
        best = best.min(elapsed);
        summary = out;
    }
    println!(
        "{shape} n={n} groups={groups} cap={cap} profile={profile} bytes={} runs={runs} best={:.6}s {summary}",
        source.len(),
        best
    );
}
