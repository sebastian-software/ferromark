//! The container line facts must not change what the list and block quote
//! walks produce.
//!
//! Every input here is parsed twice: once through the production walks,
//! which measure each line once through [`LineFacts`](super::cursor::LineFacts),
//! and once through the per-line walks they replaced, kept verbatim in
//! `list/per_line.rs` and `block_quote/per_line.rs`. The two documents have to
//! agree in their full AST, source spans included, and in their HTML. The
//! inputs are generated container soup (nested lists and quotes, blank lines,
//! tabs, mixed LF/CRLF/CR endings, lazy continuation, no-break-space and
//! form-feed lines, line comments), the bundled specification fixtures in all
//! three line endings, and the frozen measurement corpus.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::print_stdout
)]

use std::cell::Cell;
use std::path::Path;

use crate::allocator::Allocator;
use crate::parser::{Parser, ParserOptions};
use crate::renderer::HtmlRenderer;

thread_local! {
    /// Whether the container walks on this thread take the per-line path.
    static PER_LINE_WALK: Cell<bool> = const { Cell::new(false) };
    /// How many container walks took the per-line path, so a sweep can
    /// prove it really compared two different walks.
    static PER_LINE_WALKS: Cell<usize> = const { Cell::new(0) };
}

/// Whether the list and block quote walks should run their per-line
/// reference versions. Asked once per walk.
pub(super) fn per_line_walk() -> bool {
    let per_line = PER_LINE_WALK.with(Cell::get);
    if per_line {
        PER_LINE_WALKS.with(|walks| walks.set(walks.get() + 1));
    }
    per_line
}

impl Parser<'_> {
    /// `line_starts_block` as the per-line walks called it: for the line at
    /// the current position, walking its indentation once more to find the
    /// first content byte.
    pub(super) fn line_starts_block_per_line(&self) -> bool {
        let line_start = self.position;
        let Some(trimmed_start) = self.first_non_whitespace_in_line(line_start) else {
            return false;
        };
        self.probe_line(line_start, trimmed_start).starts_block
    }
}

/// Resets the walk selection even when a parse panics.
struct PerLineWalk;

impl PerLineWalk {
    fn select(per_line: bool) -> Self {
        PER_LINE_WALK.with(|flag| flag.set(per_line));
        Self
    }
}

impl Drop for PerLineWalk {
    fn drop(&mut self) {
        PER_LINE_WALK.with(|flag| flag.set(false));
    }
}

/// The full AST (source spans included) and the HTML of one parse, or the
/// error it reported.
fn output(source: &str, options: &ParserOptions, per_line: bool) -> String {
    let _walk = PerLineWalk::select(per_line);
    let allocator = Allocator::new();
    match Parser::with_options(&allocator, source, options.clone()).parse() {
        Ok(document) => {
            let html = HtmlRenderer::new().render(&document);
            format!(
                "{:?}\n{:?}\n{:?}\n{html}",
                document.span, document.front_matter, document.children
            )
        }
        Err(error) => format!("error: {error:?}"),
    }
}

#[track_caller]
fn check(source: &str, options: &ParserOptions, label: &str) {
    let measured = output(source, options, false);
    let per_line = output(source, options, true);
    assert!(
        measured == per_line,
        "line facts changed the output for {label} {source:?}\n\
         --- per-line walk ---\n{per_line}\n--- line facts ---\n{measured}"
    );
}

fn option_matrix() -> Vec<(&'static str, ParserOptions)> {
    vec![
        ("commonmark", ParserOptions::commonmark()),
        ("gfm", ParserOptions::gfm()),
        ("gfm_spec", ParserOptions::gfm_spec()),
        ("mdx", ParserOptions::mdx()),
        (
            "line_comments",
            ParserOptions {
                line_comments: true,
                ..ParserOptions::gfm()
            },
        ),
        (
            "extensions",
            ParserOptions {
                footnotes: true,
                superscript: true,
                subscript: true,
                math: true,
                definition_lists: true,
                heading_attributes: true,
                wiki_links: true,
                cjk_emphasis: true,
                line_comments: true,
                ..ParserOptions::gfm()
            },
        ),
    ]
}

/// `source` as written and with every LF turned into CRLF and into CR.
fn line_ending_variants(source: &str) -> [String; 3] {
    [
        source.to_string(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
    ]
}

#[test]
fn container_edge_cases_parse_the_same() {
    let cases = [
        // The blank line that closes a quote, in every shape the quote's
        // own test and the list's test disagree on.
        "> quote\n\nafter\n",
        "> quote\n   \nafter\n",
        "> quote\n\t \nafter\n",
        "> quote\n\u{c}\nafter\n",
        "> quote\n \u{b} \nafter\n",
        "> quote\n\u{a0}\nafter\n",
        "> quote",
        "> quote\n",
        "> quote\n  ",
        // Lazy continuation and what interrupts it.
        "> a\nlazy\n> b\n    lazy code?\n- item\n",
        "> a\n  - x\n> b\n-\n",
        "- a\nlazy\n  more\n\n  para\n- b\n",
        "- a\n\u{a0}\n- b\n",
        "- a\n\u{a0}\u{a0}\n\n- b\n",
        "- a\n \u{c} \n- b\n",
        "- a\n\u{c}- b\n",
        "- a\n\n\u{a0}- b\n",
        "* a\n*\n\n* c\n",
        // Tabs in the indentation, after markers and in the dedent.
        "-\tfoo\n\n\tbar\n",
        "- foo\n\n\t\tbar\n",
        "  - foo\n \t  bar\n\t- baz\n",
        ">\t\tfoo\n>  \tbar\n \t> baz\n",
        // A tab before the marker puts the true and the flat column count
        // apart; the tabs after it expand from the true one.
        "> a\n \t>\t\tb\n  \t> \tc\n",
        "1.\ta\n\n   \tb\n  \t2. c\n",
        // Nested containers with blank lines between and inside.
        "- a\n  > b\n  >\n  > c\n\n  d\n- e\n",
        "> - a\n>\n>   b\n> - c\n",
        "1. a\n\n   1. b\n\n      c\n\n2. d\n",
        "- a\n\n\n\n- b\n",
        "- a\n  ```\n  code\n\n  ```\n- b\n",
        "- [ ] task\n- [x] done\n\n  text\n",
        // Line comments inside and around containers.
        "- a\n// note\n  b\n",
        "- a\n\n// note\n\n  b\n",
        "> a\n// note\n> b\n",
    ];
    for case in cases {
        for source in line_ending_variants(case) {
            for (label, options) in option_matrix() {
                check(&source, &options, label);
            }
        }
    }
}

#[test]
fn generated_container_soup_parses_the_same() {
    const PREFIXES: &[&str] = &[
        "", "", "", "> ", ">", "- ", "* ", "+ ", "1. ", "2) ", "10. ", "-\t", ">\t", "-   ",
        "-      ", " ", "  ", "   ", "    ", "\t", " \t", "  \t", "  > ", "   - ", "> - ", "- > ",
        ">> ", "> > ", "\u{a0}",
    ];
    const CONTENTS: &[&str] = &[
        "text",
        "lazy line",
        "more *emphasis*",
        "",
        "\u{a0}",
        "\u{a0}\u{a0} ",
        "\u{c}",
        " \u{b} ",
        "\u{c}- x",
        "```",
        "~~~ js",
        "# heading",
        "---",
        "***",
        "- ",
        "-",
        "1.",
        "2. two",
        "[ ] task",
        "[x] done",
        "<div>",
        "</div>",
        "| a | b |",
        "| - | - |",
        "// note",
        "    code",
        "\tcode",
        "===",
        "$$",
        ": definition",
        "term",
        "[^1]: note",
        "[ref]: /url",
        "> nested",
        "<!-- c -->",
        "{expr}",
        "  two  spaces",
        "é中🙂",
    ];
    const BLANKS: &[&str] = &["", "", " ", "  ", "\t", " \t ", "\u{c}", "\u{b} "];
    const ENDINGS: &[&str] = &["\n", "\n", "\n", "\n", "\r\n", "\r"];

    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    let matrix = option_matrix();
    PER_LINE_WALKS.with(|walks| walks.set(0));
    let mut documents = 0usize;
    for _ in 0..2_500 {
        let lines = 1 + next() % 16;
        let mut source = String::new();
        for line in 0..lines {
            if next() % 5 == 0 {
                source.push_str(BLANKS[next() % BLANKS.len()]);
            } else {
                for _ in 0..next() % 3 {
                    source.push_str(PREFIXES[next() % PREFIXES.len()]);
                }
                source.push_str(PREFIXES[next() % PREFIXES.len()]);
                source.push_str(CONTENTS[next() % CONTENTS.len()]);
            }
            // The last line is unterminated now and then.
            if line + 1 < lines || next() % 4 != 0 {
                source.push_str(ENDINGS[next() % ENDINGS.len()]);
            }
        }
        for (label, options) in &matrix {
            check(&source, options, label);
        }
        documents += 1;
    }
    let walks = PER_LINE_WALKS.with(Cell::get);
    assert!(
        walks > 10 * documents,
        "expected the soup to exercise the container walks, got {walks} walks over {documents} documents"
    );
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
fn specification_fixtures_parse_the_same() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut examples = 0usize;
    for fixture in [
        "tests/spec_fixtures/commonmark-0.31.2-spec.txt",
        "tests/spec_fixtures/gfm-extensions-spec.txt",
    ] {
        let path = root.join(fixture);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for markdown in spec_inputs(&text) {
            for source in line_ending_variants(&markdown) {
                for (label, options) in option_matrix() {
                    check(&source, &options, &format!("{fixture} ({label})"));
                }
            }
            examples += 1;
        }
    }
    assert!(
        examples > 600,
        "expected the full spec suite, got {examples}"
    );
}

#[test]
fn frozen_corpus_parses_the_same() {
    const CORPUS: &str = "docs/reports/2026-09-14-simd-round/corpus.json.gz";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS);
    let Ok(raw) = std::fs::read(&path) else {
        // Published crate archives carry `tests/` but not `docs/`.
        println!("{CORPUS}: not present, skipped");
        return;
    };
    let json = super::prepass::gzip::gunzip(&raw).unwrap_or_else(|error| panic!("{error}"));
    let corpus: serde_json::Value =
        serde_json::from_slice(&json).unwrap_or_else(|error| panic!("{error}"));
    let cases = corpus["cases"].as_array().expect("cases array");
    assert_eq!(cases.len(), 72, "{CORPUS}: case count changed");
    let matrix = option_matrix();
    for case in cases {
        let name = case["name"].as_str().expect("case name");
        let input = case["input"].as_str().expect("case input");
        let profile = case["profile"].as_str().expect("case profile");
        assert!(
            matrix.iter().any(|(label, _)| *label == profile),
            "{name}: unknown profile {profile}"
        );
        // The corpus profile's own options, plus the line-comment profile
        // for the branches only it reaches.
        let options: Vec<_> = matrix
            .iter()
            .filter(|(label, _)| *label == profile || *label == "line_comments")
            .collect();
        for source in line_ending_variants(input) {
            for (label, options) in &options {
                check(&source, options, &format!("{name} ({label})"));
            }
        }
    }
}
