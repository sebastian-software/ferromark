//! Training driver for the published Node addon's profile-guided build.
//!
//! `node/ferromark/scripts/build-native.mjs` builds this binary with
//! `-Cprofile-generate`, runs it over the frozen benchmark corpus and merges the
//! raw counters into a profile that the real `napi build` consumes through
//! `-Cprofile-use`. `ferromark-node` is `publish = false`, so this target is a
//! build tool and never ships.
//!
//! The binary links `ferromark` directly: a `cdylib` cannot be used as a library
//! target, and the addon's own N-API surface is a thin wrapper around the parser
//! and renderer that the training run exercises here.
//!
//! Usage: `pgo_train <corpus-directory> <milliseconds-per-file-and-lifecycle>`.
//! The directory holds one `.md` file per corpus case. Every file is run through
//! the four option combinations the addon reaches, each in the addon's two
//! lifecycles: a fresh allocator/renderer per document, and a retained allocator
//! and renderer reused across documents.

// This is a build tool driven by a Node script; its progress summary is the only
// record of what the training run covered in a CI log.
#![allow(clippy::print_stdout)]

use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, ParseError, Parser, ParserOptions};

/// The option combinations the addon's entry points reach: the product defaults,
/// the GFM parser with default rendering, the full GFM profile, and MDX.
fn configurations() -> [(&'static str, ParserOptions, HtmlRendererOptions); 4] {
    [
        (
            "default",
            ParserOptions::default(),
            HtmlRendererOptions::new(),
        ),
        (
            "gfm-parse",
            ParserOptions::gfm(),
            HtmlRendererOptions::new(),
        ),
        ("gfm", ParserOptions::gfm(), HtmlRendererOptions::gfm()),
        ("mdx", ParserOptions::mdx(), HtmlRendererOptions::new()),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let (Some(directory), Some(budget), None) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err(
            "usage: pgo_train <corpus-directory> <milliseconds-per-file-and-lifecycle>".into(),
        );
    };
    let budget = Duration::from_millis(budget.parse::<u64>()?);
    let directory = Path::new(&directory);
    let sources = read_corpus(directory)?;
    if sources.is_empty() {
        return Err(format!("no .md files in {}", directory.display()).into());
    }

    let configurations = configurations();
    let start = Instant::now();
    let mut iterations: u64 = 0;
    for (name, source) in &sources {
        for (label, parser, html) in &configurations {
            iterations += train_fresh(source, parser, html, budget)
                .map_err(|error| format!("{name} [{label}] fresh: {error}"))?;
            iterations += train_reuse(source, parser, html, budget)
                .map_err(|error| format!("{name} [{label}] reuse: {error}"))?;
        }
    }

    println!(
        "pgo_train: {} files x {} configurations x 2 lifecycles, {iterations} iterations in {:.1}s",
        sources.len(),
        configurations.len(),
        start.elapsed().as_secs_f64(),
    );
    Ok(())
}

/// Reads every `.md` file in `directory` in a stable order.
fn read_corpus(directory: &Path) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut sources = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension() == Some(OsStr::new("md")) {
            let name = path
                .file_stem()
                .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
            sources.push((name, fs::read_to_string(&path)?));
        }
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(sources)
}

/// One arena and one renderer per document, as `toHtml` uses them.
fn train_fresh(
    source: &str,
    parser: &ParserOptions,
    html: &HtmlRendererOptions,
    budget: Duration,
) -> Result<u64, ParseError> {
    let start = Instant::now();
    let mut iterations = 0;
    loop {
        let allocator = Allocator::for_source_len(source.len());
        let document =
            Parser::with_options(&allocator, black_box(source), parser.clone()).parse()?;
        let mut renderer = HtmlRenderer::with_options(html.clone());
        black_box(renderer.render(&document));
        iterations += 1;
        if start.elapsed() >= budget {
            return Ok(iterations);
        }
    }
}

/// A retained arena reset per document and a retained renderer writing into its
/// own buffer, as the `Renderer` class uses them.
fn train_reuse(
    source: &str,
    parser: &ParserOptions,
    html: &HtmlRendererOptions,
    budget: Duration,
) -> Result<u64, ParseError> {
    let mut allocator = Allocator::for_source_len(source.len());
    let mut renderer = HtmlRenderer::with_options(html.clone());
    let start = Instant::now();
    let mut iterations = 0;
    loop {
        allocator.reset();
        {
            let document =
                Parser::with_options(&allocator, black_box(source), parser.clone()).parse()?;
            black_box(renderer.render_borrowed(&document).len());
        }
        iterations += 1;
        if start.elapsed() >= budget {
            return Ok(iterations);
        }
    }
}
