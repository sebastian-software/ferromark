//! The segmented definition pass must collect exactly what the full pass
//! collects.
//!
//! Every test here compares the two collections directly rather than through
//! rendered HTML: the segment planner is only allowed to make the structural
//! pass cheaper, never to change which definitions or footnote labels reach
//! the document parse. The inputs are the bundled specification fixtures, the
//! frozen measurement corpora, and generated token soup in the shape of the
//! candidate-scan property test.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::print_stdout
)]

use std::path::{Path, PathBuf};

use crate::allocator::Allocator;
use crate::parser::{Parser, ParserOptions};

use super::gzip::gunzip;
use super::segments::{DefinitionPlan, plan_with_survivors};
use super::{CandidateOpeners, scan_definition_candidates};

/// Reference definitions and footnote labels in a comparable, ordered form.
type Collected = (Vec<(String, String, Option<String>)>, Vec<String>);

fn collect(parser: &Parser<'_>, segments: &[(usize, usize)]) -> Collected {
    let (definitions, labels) = parser.collect_definitions(segments);
    let mut references: Vec<_> = definitions
        .iter()
        .map(|(identifier, definition)| {
            (
                identifier.to_string(),
                definition.url.to_string(),
                definition.title.map(str::to_string),
            )
        })
        .collect();
    references.sort();
    let mut labels: Vec<_> = labels.iter().map(ToString::to_string).collect();
    labels.sort();
    (references, labels)
}

/// What the planner decided, and how much of the body it left to parse.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// Bytes the structural pass has to parse, out of the whole body.
    Segmented {
        planned: usize,
        body: usize,
    },
    FellBack,
}

impl Outcome {
    fn fell_back(self) -> bool {
        self == Self::FellBack
    }
}

/// Asserts that the planner's segments collect what the whole body collects.
///
/// A document the block grammar rejects (nesting depth) is skipped: the real
/// parse reports that error and no successful output reads the map, so the
/// two passes are free to stop at different points.
#[track_caller]
fn check(source: &str, options: &ParserOptions) -> Outcome {
    let allocator = Allocator::new();
    if Parser::with_options(&allocator, source, options.clone())
        .parse()
        .is_err()
    {
        return Outcome::FellBack;
    }
    let parser = Parser::with_options(&allocator, source, options.clone());
    let body = parser.source;

    let mut openers = CandidateOpeners::new();
    scan_definition_candidates(body, options.footnotes, options.mdx, &mut openers);
    let (plan, survivors) = plan_with_survivors(body, &parser.options, &openers);

    let DefinitionPlan::Segments(segments) = &plan else {
        return Outcome::FellBack;
    };
    for &survivor in &survivors {
        assert!(
            segments
                .iter()
                .any(|&(start, end)| (start..end).contains(&(survivor as usize))),
            "candidate at {survivor} is outside {segments:?} for {source:?}"
        );
    }
    assert!(
        segments
            .windows(2)
            .all(|pair| pair[0].1 < pair[1].0 && pair[0].0 < pair[0].1),
        "segments must be disjoint and ascending: {segments:?} for {source:?}"
    );
    assert!(
        segments.iter().all(|&(_, end)| end <= body.len()),
        "segments must stay inside the body: {segments:?} for {source:?}"
    );

    assert_eq!(
        collect(&parser, segments),
        collect(&parser, &[(0, body.len())]),
        "segmented collection differs from the full pass for {source:?}"
    );
    Outcome::Segmented {
        planned: segments.iter().map(|&(start, end)| end - start).sum(),
        body: body.len(),
    }
}

fn option_matrix() -> Vec<(&'static str, ParserOptions)> {
    let with = |name: &'static str, apply: fn(&mut ParserOptions)| {
        let mut options = ParserOptions::default();
        apply(&mut options);
        (name, options)
    };
    vec![
        ("default", ParserOptions::default()),
        ("gfm", ParserOptions::gfm()),
        with("footnotes", |options| options.footnotes = true),
        with("definition_lists", |options| {
            options.definition_lists = true;
        }),
        with("math", |options| options.math = true),
        with("tables", |options| options.tables = true),
        (
            "gfm_without_tables",
            ParserOptions {
                tables: false,
                ..ParserOptions::gfm()
            },
        ),
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
fn specification_fixtures_collect_the_same_definitions() {
    let mut examples = 0usize;
    let mut fell_back = 0usize;
    for fixture in [
        "tests/spec_fixtures/commonmark-0.31.2-spec.txt",
        "tests/spec_fixtures/gfm-extensions-spec.txt",
    ] {
        let path = repository_path(fixture);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for markdown in spec_inputs(&text) {
            for (_, options) in option_matrix() {
                examples += 1;
                if check(&markdown, &options).fell_back() {
                    fell_back += 1;
                }
            }
            // Line-ending variants exercise the boundary scan's CR handling.
            for replacement in ["\r\n", "\r"] {
                let converted = markdown.replace('\n', replacement);
                examples += 1;
                if check(&converted, &ParserOptions::gfm()).fell_back() {
                    fell_back += 1;
                }
            }
        }
    }
    assert!(
        examples > 3_000,
        "expected the full spec suite, got {examples}"
    );
    println!("spec fixtures: {examples} checks, {fell_back} fell back");
}

/// Every case of one frozen measurement corpus, in the corpus's own profile.
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

    let mut fell_back = 0usize;
    let mut planned_bytes = 0usize;
    let mut body_bytes = 0usize;
    for case in cases {
        let name = case["name"].as_str().expect("case name");
        let input = case["input"].as_str().expect("case input");
        if let Some(bytes) = case["byte_count"].as_u64() {
            assert_eq!(
                input.len() as u64,
                bytes,
                "{relative}/{name}: decompressed input does not match byte_count"
            );
        }
        let profile = match case["profile"].as_str() {
            Some("commonmark") => ParserOptions::commonmark(),
            Some("gfm_spec") => ParserOptions::gfm_spec(),
            // MDX always falls back; checking it keeps that honest.
            Some("mdx") => ParserOptions::mdx(),
            // Diagnostic profiles name feature bundles the corpus does not
            // spell out; the option matrix above already covers each feature.
            _ => ParserOptions::gfm(),
        };
        for (label, options) in option_matrix() {
            if check(input, &options).fell_back() {
                fell_back += 1;
                println!("{relative}/{name}: falls back ({label})");
            }
        }
        match check(input, &profile) {
            Outcome::FellBack => println!("{relative}/{name}: falls back (corpus profile)"),
            Outcome::Segmented { planned, body } => {
                if planned > 0 {
                    println!("{relative}/{name}: plans {planned} of {body} bytes");
                }
                planned_bytes += planned;
                body_bytes += body;
            }
        }
    }
    // A structural fact, not a timing: how much of each document the pass is
    // still asked to parse in its own profile.
    println!(
        "{relative}: {} cases, {fell_back} fallbacks, {planned_bytes}/{body_bytes} planned bytes",
        cases.len()
    );
}

#[test]
fn broad_corpus_documents_collect_the_same_definitions() {
    check_corpus("docs/reports/2026-09-15-release-native/corpus.json.gz", 57);
}

#[test]
fn additional_corpora_collect_the_same_definitions() {
    for (relative, cases) in [
        ("docs/reports/2026-09-14-broad-markdown/corpus.json.gz", 57),
        // The third Apple Silicon round adds the authored table, autolink and
        // scanner diagnostics to the same 57 broad documents.
        ("docs/reports/2026-09-16-arm-round-3/corpus.json.gz", 207),
    ] {
        check_corpus(relative, cases);
    }
}

#[test]
fn generated_token_soup_collects_the_same_definitions() {
    // The shapes that decide a boundary, a candidate, or an opaque region,
    // dense enough that regions nest, overlap and run off the end.
    let tokens = [
        "[",
        "]:",
        "[^",
        "\n",
        "\r\n",
        "\r",
        " ",
        "  ",
        "   ",
        "    ",
        ">",
        "- ",
        "1. ",
        "```",
        "~~~",
        "`````",
        "<div>",
        "<!--",
        "-->",
        "<script>",
        "</script>",
        "<custom>",
        "$$",
        ":",
        "word ",
        "a",
        "</div>",
        "<pre>",
        "|",
        "#",
    ];
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (state >> 33) as usize
    };
    let matrix = option_matrix();
    let mut checks = 0usize;
    for _ in 0..1_500 {
        let len = next() % 40;
        let mut source = String::new();
        for _ in 0..len {
            source.push_str(tokens[next() % tokens.len()]);
        }
        for (_, options) in &matrix {
            check(&source, options);
            checks += 1;
        }
    }
    assert!(checks > 10_000, "expected a broad sweep, got {checks}");
}
