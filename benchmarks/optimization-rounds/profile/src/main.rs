//! Profiling driver: loops one stage over a corpus suite for a fixed time so a
//! sampling profiler (perf, `sample`, Instruments) can attribute where it goes.
//!
//! usage: profile <corpus.json> <parse|render|reuse> <seconds> [suite]
//!
//! Every document is repeated in proportion to 1 / its size, so each gets
//! about the same byte volume per sweep and small documents are not drowned
//! out by large ones. Options follow `worker.rs` for the `commonmark`, `gfm`
//! (footnotes off) and `autolink` profiles.

use std::hint::black_box;
use std::time::Instant;

use ferromark::ast::Document;
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

/// Bytes of input each document contributes to one sweep.
const BYTES_PER_DOCUMENT: usize = 400_000;

fn options(profile: &str) -> ParserOptions {
    match profile {
        "commonmark" | "autolink" => ParserOptions::default(),
        "gfm" => ParserOptions {
            footnotes: false,
            ..ParserOptions::gfm()
        },
        other => panic!("unsupported profile: {other}"),
    }
}

fn html_options(profile: &str) -> HtmlRendererOptions {
    HtmlRendererOptions {
        xhtml: true,
        hard_break: "<br />\n".into(),
        autolink_urls: profile == "autolink",
        autolink_target_blank: false,
        link_target_blank: false,
        disallow_raw_html: profile == "gfm",
        ..HtmlRendererOptions::new()
    }
}

struct Case {
    source: &'static str,
    parser: ParserOptions,
    renderer: HtmlRenderer,
    repeat: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, corpus, stage, seconds, rest @ ..] = args.as_slice() else {
        panic!("usage: profile <corpus.json> <parse|render|reuse> <seconds> [suite]");
    };
    let seconds: f64 = seconds.parse().expect("seconds");
    let suite = rest.first().map_or("broad", String::as_str);
    let corpus: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(corpus).expect("read corpus"))
            .expect("corpus JSON");
    let mut cases: Vec<Case> = corpus["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .filter(|case| case["suite"] == suite)
        .map(|case| {
            let profile = case["profile"].as_str().expect("profile");
            let source: &'static str = Box::leak(
                case["input"]
                    .as_str()
                    .expect("input")
                    .to_owned()
                    .into_boxed_str(),
            );
            Case {
                source,
                parser: options(profile),
                renderer: HtmlRenderer::with_options(html_options(profile)),
                repeat: (BYTES_PER_DOCUMENT / source.len().max(1)).max(1),
            }
        })
        .collect();
    assert!(!cases.is_empty(), "suite {suite} selected no cases");

    // The render stage renders documents parsed once up front, as the
    // harness's render lane does.
    let arena: &'static Allocator = Box::leak(Box::new(Allocator::new()));
    let documents: Vec<Document<'static>> = if stage == "render" {
        cases
            .iter()
            .map(|case| {
                Parser::with_options(arena, case.source, case.parser.clone())
                    .parse()
                    .expect("parse")
            })
            .collect()
    } else {
        Vec::new()
    };

    let mut allocator = Allocator::new();
    let start = Instant::now();
    let mut sweeps = 0u64;
    while start.elapsed().as_secs_f64() < seconds {
        for (index, case) in cases.iter_mut().enumerate() {
            for _ in 0..case.repeat {
                match stage.as_str() {
                    "render" => {
                        black_box(case.renderer.render_borrowed(black_box(&documents[index])));
                    }
                    "parse" => {
                        let document =
                            Parser::with_options(&allocator, case.source, case.parser.clone())
                                .parse()
                                .expect("parse");
                        black_box(&document);
                        drop(document);
                        allocator.reset();
                    }
                    "reuse" => {
                        let document =
                            Parser::with_options(&allocator, case.source, case.parser.clone())
                                .parse()
                                .expect("parse");
                        black_box(case.renderer.render_borrowed(&document));
                        drop(document);
                        allocator.reset();
                    }
                    other => panic!("unsupported stage: {other}"),
                }
            }
        }
        sweeps += 1;
    }
    eprintln!(
        "{} {suite} documents, {stage}: {sweeps} sweeps in {:.2} s",
        cases.len(),
        start.elapsed().as_secs_f64()
    );
}
