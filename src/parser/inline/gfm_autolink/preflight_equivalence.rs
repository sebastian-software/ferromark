//! The tracked autolink pre-flight must produce exactly the documents the
//! separate pre-flight produces.
//!
//! Every input is parsed twice — once with each pre-flight selected on this
//! thread — and the AST `Debug` output and the rendered HTML are compared.
//! On top of that, every block parsed with the tracked pre-flight asserts
//! that its gate value equals the separate pass's (`assert_same_preflight`),
//! so these inputs check the gate itself and not only its effect.
//!
//! The inputs are the bundled specification fixtures, two frozen
//! measurement corpora, and generated token soup built from the shapes that
//! decide the gate: the needles, their near misses, and every construct that
//! consumes bytes the marker scan never classifies.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::print_stdout
)]

use std::path::{Path, PathBuf};

use crate::allocator::Allocator;
use crate::parser::inline::AutolinkPreflight;
use crate::parser::prepass::gzip::gunzip;
use crate::parser::{Parser, ParserOptions};
use crate::renderer::{HtmlRenderer, HtmlRendererOptions};

/// The AST and the HTML of one parse, or its error.
fn output(source: &str, options: &ParserOptions, preflight: AutolinkPreflight) -> String {
    preflight.with(|| {
        let arena = Allocator::new();
        match Parser::with_options(&arena, source, options.clone()).parse() {
            Ok(document) => {
                // `gfm_spec` renders without the renderer's own URL
                // autolinking, which could otherwise hide a parser difference.
                let html =
                    HtmlRenderer::with_options(HtmlRendererOptions::gfm_spec()).render(&document);
                format!("{document:?}\n{html}")
            }
            Err(error) => format!("error: {error:?}"),
        }
    })
}

/// Asserts identical output and reports whether the parse made a link.
#[track_caller]
fn check(source: &str, options: &ParserOptions) -> bool {
    let separate = output(source, options, AutolinkPreflight::Separate);
    let tracked = output(source, options, AutolinkPreflight::Tracked);
    if separate != tracked {
        let at = separate
            .bytes()
            .zip(tracked.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| separate.len().min(tracked.len()));
        let context = |text: &str| {
            let start = text.floor_char_boundary(at.saturating_sub(80));
            let end = text.ceil_char_boundary((at + 80).min(text.len()));
            text[start..end].to_owned()
        };
        panic!(
            "outputs differ at byte {at} for {source:?}\nseparate: {:?}\ntracked:  {:?}",
            context(&separate),
            context(&tracked)
        );
    }
    separate.contains("Link(")
}

/// Every profile with autolinks on: GFM, strict GFM, and GFM with each
/// combination of the four optional inline markers, which select the
/// classifier tables the tracked scan runs.
fn option_matrix() -> Vec<ParserOptions> {
    let mut matrix = vec![ParserOptions::gfm(), ParserOptions::gfm_spec()];
    for markers in 0..16u8 {
        matrix.push(ParserOptions {
            mdx: markers & 1 != 0,
            superscript: markers & 2 != 0,
            math: markers & 4 != 0,
            highlight: markers & 8 != 0,
            // A caret that cannot open a note stays in the text run.
            inline_footnotes: markers == 0,
            subscript: markers & 2 != 0,
            wiki_links: markers & 4 != 0,
            ..ParserOptions::gfm()
        });
    }
    matrix
}

/// The smaller matrix for the long corpus documents.
fn corpus_matrix() -> Vec<ParserOptions> {
    vec![
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions {
            highlight: true,
            inline_footnotes: true,
            superscript: true,
            subscript: true,
            math: true,
            definition_lists: true,
            heading_attributes: true,
            wiki_links: true,
            ..ParserOptions::gfm()
        },
    ]
}

fn repository_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Markdown inputs of every example in a CommonMark-style `spec.txt` fixture.
fn spec_inputs(text: &str) -> Vec<String> {
    const FENCE: &str = "````````````````````````````````";
    let mut inputs = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let is_example = line
            .strip_prefix(FENCE)
            .is_some_and(|rest| rest.trim().starts_with("example"));
        if !is_example {
            continue;
        }
        let mut markdown = String::new();
        for body in lines.by_ref() {
            if body.starts_with(FENCE) || body == "." {
                break;
            }
            markdown.push_str(&body.replace('→', "\t"));
            markdown.push('\n');
        }
        inputs.push(markdown);
    }
    inputs
}

#[test]
fn specification_fixtures_render_the_same() {
    let matrix = option_matrix();
    let mut examples = 0usize;
    let mut linked = 0usize;
    for fixture in [
        "tests/spec_fixtures/commonmark-0.31.2-spec.txt",
        "tests/spec_fixtures/gfm-extensions-spec.txt",
    ] {
        let path = repository_path(fixture);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for markdown in spec_inputs(&text) {
            for options in &matrix {
                examples += 1;
                linked += usize::from(check(&markdown, options));
            }
        }
    }
    assert!(
        examples > 10_000,
        "expected the full spec suites, got {examples}"
    );
    assert!(
        linked > 1_000,
        "expected links in the fixtures, got {linked}"
    );
    println!("spec fixtures: {examples} checks, {linked} with links");
}

/// Every case of one frozen measurement corpus.
fn check_corpus(relative: &str, expected_cases: usize) {
    let path = repository_path(relative);
    let Ok(raw) = std::fs::read(&path) else {
        // Published crate archives carry `tests/` but not `docs/`.
        println!("{relative}: not present, skipped");
        return;
    };
    let json = gunzip(&raw).unwrap_or_else(|error| panic!("{relative}: {error}"));
    let corpus: serde_json::Value =
        serde_json::from_slice(&json).unwrap_or_else(|error| panic!("{relative}: {error}"));
    let cases = corpus["cases"]
        .as_array()
        .unwrap_or_else(|| panic!("{relative}: no cases array"));
    assert_eq!(
        cases.len(),
        expected_cases,
        "{relative}: case count changed"
    );
    let matrix = corpus_matrix();
    for case in cases {
        let input = case["input"].as_str().expect("case input");
        for options in &matrix {
            check(input, options);
        }
    }
    println!("{relative}: {} cases checked", cases.len());
}

#[test]
fn frozen_corpora_render_the_same() {
    // The 57 broad documents with the authored table, autolink and scanner
    // diagnostics, and the first SIMD round's corpus.
    check_corpus("docs/reports/2026-09-16-arm-round-3/corpus.json.gz", 207);
    check_corpus("docs/reports/2026-09-14-simd-round/corpus.json.gz", 72);
}

#[test]
fn generated_token_soup_renders_the_same() {
    // The needles, their near misses, the escapes and references that
    // decode into them, and every construct whose bytes the marker scan
    // skips: code spans, link destinations and titles, autolinks, raw HTML,
    // math, expressions, notes. The long filler crosses the 16- and 32-byte
    // vector steps and the overlapping tail.
    let tokens = [
        "www.",
        "WWW.",
        "www",
        "ww",
        "w",
        ".",
        "w.",
        "://",
        ":",
        "//",
        "/",
        "http",
        "https",
        "HTTP",
        "ftp",
        "](",
        "](http://",
        "](https://",
        "](ftp://",
        "[",
        "]",
        "(",
        ")",
        "![",
        "[x](",
        "[^1]",
        "[^1]: note\n",
        "[ref]: http://ref.example\n",
        "mailto:",
        "mailto",
        "xmpp:",
        "xmpp",
        "@",
        "user@example.com",
        "a@b.c",
        "example.com",
        ".org/path?q=1&x=2",
        "\\",
        "\\.",
        "\\:",
        "\\@",
        "\\/",
        "\\w",
        "&#58;",
        "&#64;",
        "&#46;",
        "&#119;",
        "&amp;",
        "&",
        "`",
        "``",
        "`www.x.y`",
        "`a@b.c`",
        "*",
        "**",
        "_",
        "~",
        "~~",
        "<",
        ">",
        "<http://a.b>",
        "<a@b.c>",
        "<span title=\"www.x.y\">",
        "\n",
        "  \n",
        "\\\n",
        "\n\n",
        " ",
        "a",
        "text ",
        "$",
        "$x@y.z$",
        "^",
        "^[",
        "=",
        "==",
        "{",
        "}",
        "| a | b |\n|---|---|\n| ",
        " | ",
        "- ",
        "> ",
        "# ",
        "\t",
        "é",
        "中",
        "abcdefghijklmnopqrstuvwxyz0123",
        "a long run of ordinary prose without any trigger in it ",
    ];
    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    let matrix = option_matrix();
    let mut checks = 0usize;
    let mut linked = 0usize;
    for round in 0..2_500 {
        let len = next() % 48;
        let mut source = String::new();
        for _ in 0..len {
            source.push_str(tokens[next() % tokens.len()]);
        }
        // Every document under GFM; a rotating slice of the matrix on top.
        linked += usize::from(check(&source, &matrix[0]));
        check(&source, &matrix[2 + round % (matrix.len() - 2)]);
        check(&source, &matrix[1]);
        checks += 3;
    }
    assert!(checks > 7_000, "expected a broad sweep, got {checks}");
    assert!(linked > 500, "expected generated links, got {linked}");
    println!("token soup: {checks} checks, {linked} with links");
}
