//! The indexed autolink pre-flight must produce exactly the documents the
//! full pass produces.
//!
//! Every input is parsed twice — once with the root parser recording trigger
//! offsets and once without — and the AST `Debug` output and the rendered HTML
//! are compared. On top of that, every block the indexed parse answers from
//! the offsets asserts that its gate value equals the full pass's
//! (`assert_same_preflight`), so these inputs check the gate itself and not
//! only its effect.
//!
//! The inputs are the bundled specification fixtures, two frozen measurement
//! corpora, and generated documents built from the shapes that decide the
//! gate: the needles and their near misses at block boundaries, destinations,
//! code, containers that copy their content, line endings, NUL bytes and front
//! matter.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::print_stdout
)]

use std::cell::Cell;
use std::path::{Path, PathBuf};

use crate::allocator::Allocator;
use crate::parser::prepass::gzip::gunzip;
use crate::parser::{Parser, ParserOptions};
use crate::renderer::{HtmlRenderer, HtmlRendererOptions};

use super::triggers::{BYTES_PER_OFFSET, CHUNK, OFFSET_ALLOWANCE, with_triggers};

thread_local! {
    /// Blocks the offsets answered on this thread, so a test can show that
    /// its inputs reach the indexed path and not only the fallback.
    static INDEXED: Cell<usize> = const { Cell::new(0) };
}

/// Counts one block answered from the offsets.
pub(super) fn count_indexed() {
    INDEXED.with(|count| count.set(count.get() + 1));
}

fn indexed_so_far() -> usize {
    INDEXED.with(Cell::get)
}

/// The AST and the HTML of one parse, or its error.
fn output(source: &str, options: &ParserOptions, collect: bool) -> String {
    with_triggers(collect, || {
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
    let full = output(source, options, false);
    let indexed = output(source, options, true);
    if full != indexed {
        let at = full
            .bytes()
            .zip(indexed.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| full.len().min(indexed.len()));
        let context = |text: &str| {
            let start = text.floor_char_boundary(at.saturating_sub(80));
            let end = text.ceil_char_boundary((at + 80).min(text.len()));
            text[start..end].to_owned()
        };
        panic!(
            "outputs differ at byte {at} for {source:?}\nfull:    {:?}\nindexed: {:?}",
            context(&full),
            context(&indexed)
        );
    }
    full.contains("Link(")
}

/// Profiles with autolinks on: with and without link references and
/// footnotes, with front matter and line comments, with every optional
/// inline construct, and MDX.
fn option_matrix() -> Vec<ParserOptions> {
    vec![
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions {
            footnotes: false,
            ..ParserOptions::gfm()
        },
        ParserOptions {
            allow_link_refs: false,
            ..ParserOptions::gfm()
        },
        ParserOptions {
            front_matter: true,
            line_comments: true,
            ..ParserOptions::gfm()
        },
        ParserOptions {
            highlight: true,
            inline_footnotes: true,
            superscript: true,
            subscript: true,
            math: true,
            definition_lists: true,
            heading_attributes: true,
            wiki_links: true,
            table_attributes: true,
            ..ParserOptions::gfm()
        },
        ParserOptions::mdx(),
        ParserOptions {
            allow_link_refs: false,
            footnotes: false,
            ..ParserOptions::gfm()
        },
    ]
    .into_iter()
    .map(|options| ParserOptions {
        autolinks: true,
        ..options
    })
    .collect()
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
    let before = indexed_so_far();
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
            // Line-ending variants move every trigger against the line ends.
            for ending in ["\r\n", "\r"] {
                examples += 1;
                check(&markdown.replace('\n', ending), &matrix[0]);
            }
        }
    }
    let indexed = indexed_so_far() - before;
    assert!(
        examples > 5_000,
        "expected the full spec suites, got {examples}"
    );
    assert!(linked > 800, "expected links in the fixtures, got {linked}");
    assert!(
        indexed > 4_000,
        "expected blocks answered from the offsets, got {indexed}"
    );
    println!("spec fixtures: {examples} checks, {linked} with links, {indexed} indexed blocks");
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
    let matrix = option_matrix();
    let before = indexed_so_far();
    for case in cases {
        let input = case["input"].as_str().expect("case input");
        // The benchmark's own GFM profile, and the broader ones.
        check(
            input,
            &ParserOptions {
                footnotes: false,
                ..ParserOptions::gfm()
            },
        );
        for options in &matrix[..2] {
            check(input, options);
        }
        check(input, &matrix[5]);
    }
    println!(
        "{relative}: {} cases checked, {} indexed blocks",
        cases.len(),
        indexed_so_far() - before
    );
}

#[test]
fn frozen_corpora_render_the_same() {
    // The 57 broad documents with the authored table, autolink and scanner
    // diagnostics, and the first SIMD round's corpus.
    check_corpus("docs/reports/2026-09-16-arm-round-3/corpus.json.gz", 207);
    check_corpus("docs/reports/2026-09-14-simd-round/corpus.json.gz", 72);
}

/// Deterministic generator for the documents below.
fn generator(seed: u64) -> impl FnMut() -> usize {
    let mut state = seed;
    move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    }
}

#[test]
fn needles_at_block_boundaries_render_the_same() {
    // Each needle split at every point across the start or end of a block,
    // behind every block opener, so the pre-flight's content-relative bounds
    // decide the answer: a `mailto:`/`xmpp:` whose name ends the line before,
    // a `://` whose colon ends a paragraph, a `www` cut from its `.`, a `](`
    // left in the block above.
    let needles = [
        "mailto:a@b.co",
        "xmpp:a@b.co/r",
        "http://x.y",
        "https://x.y",
        "ftp://x.y",
        "www.x.y",
        "](https://x.y)",
        "[a](http://x.y) http://z.w",
        "a@b.co",
    ];
    let openers = [
        "", "> ", "- ", "1. ", "# ", "    ", "  ", "| ", "[^n]: ", "- [ ] ", "> - ",
    ];
    let matrix = option_matrix();
    let before = indexed_so_far();
    let mut checks = 0usize;
    for needle in needles {
        for split in 0..=needle.len() {
            let (head, tail) = needle.split_at(split);
            for opener in openers {
                for separator in ["\n", "\n\n", "\r\n", "\r\n\r\n", "\n```\n", "`\n`"] {
                    let source = format!("text {head}{separator}{opener}{tail} text\n");
                    for options in &matrix[..3] {
                        check(&source, options);
                        checks += 1;
                    }
                    let table = format!("| a | b |\n|---|---|\n| {head}|{tail} |\n");
                    check(&table, &matrix[0]);
                    checks += 1;
                }
            }
        }
    }
    println!(
        "block boundaries: {checks} checks, {} indexed blocks",
        indexed_so_far() - before
    );
}

/// Blocks the offsets answered while parsing `source` with the record on.
fn indexed_blocks(source: &str, options: &ParserOptions) -> usize {
    let before = indexed_so_far();
    output(source, options, true);
    indexed_so_far() - before
}

#[test]
fn the_density_bound_renders_the_same_around_it() {
    // A first paragraph whose chunk holds exactly as many triggers as the
    // record allows for one chunk, or one more, then filler to the end of
    // that chunk and sparse paragraphs with a URL each. The first block
    // records the first chunk, so the record stops right there or never.
    let bound = OFFSET_ALLOWANCE + CHUNK / BYTES_PER_OFFSET;
    let options = ParserOptions {
        footnotes: false,
        ..ParserOptions::gfm()
    };
    for (count, trigger) in ["w. ", "a@b ", "x:/ "]
        .into_iter()
        .flat_map(|piece| [(bound, piece), (bound + 1, piece)])
    {
        let mut source = trigger.repeat(count);
        source.push_str("\n\n");
        while source.len() < CHUNK + 40 {
            source.push_str("plain filler prose.\n\n");
        }
        let sparse = 12;
        for _ in 0..sparse {
            source.push_str(
                "Longer prose that mentions http://example.com once, and then keeps \
                 going for a while without any other needle in it.\n\n",
            );
        }
        check(&source, &options);
        let indexed = indexed_blocks(&source, &options);
        if count > bound {
            assert_eq!(indexed, 0, "{count} × {trigger:?}: the record must stop");
        } else {
            // Every paragraph: the dense one, the filler and the sparse ones.
            assert!(indexed > sparse, "{count} × {trigger:?}: {indexed} indexed");
        }
    }
}

#[test]
fn dense_prefixes_render_the_same() {
    // Dense stretches of every trigger kind in front of sparse prose, in
    // lists, quotes, tables and code as well as paragraphs, at lengths
    // around the bound, so the record stops at every kind of block.
    let bound = OFFSET_ALLOWANCE + CHUNK / BYTES_PER_OFFSET;
    let dense = [
        "w. ",
        "www.a ",
        "a@b.c ",
        "http://x ",
        "[l](https://x) ",
        ":/ ",
    ];
    let wrappers = [
        ("", "\n\n"),
        ("- ", "\n"),
        ("> ", "\n"),
        ("| a |\n|---|\n| ", " |\n"),
        ("```\n", "\n```\n"),
    ];
    let tail = "Sparse prose with www.example.org, http://example.com and \
                user@example.com, then a [link](https://example.net).\n\n"
        .repeat(10);
    let matrix = option_matrix();
    for piece in dense {
        for count in [bound - 1, bound, bound + 1, 2 * bound, 6 * bound] {
            for (open, close) in wrappers {
                let source = format!("{open}{}{close}\n{tail}", piece.repeat(count));
                for options in &matrix[..3] {
                    check(&source, options);
                }
            }
        }
    }
}

#[test]
fn generated_documents_render_the_same() {
    // The needles, their near misses, the escapes and references that decode
    // into them, and every construct around them: code spans and blocks, link
    // destinations and titles, autolinks, raw HTML, containers that copy
    // their content, line comments, NUL bytes, front matter. The long filler
    // crosses the recorder's 16- and 64-byte steps, its last vector and the
    // 512-byte chunks.
    let tokens = [
        "www.",
        "WWW.",
        "www",
        "ww",
        "w",
        ".",
        "w.",
        "w/",
        "://",
        ":",
        ":.",
        ":/",
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
        "&#58;",
        "&#64;",
        "&amp;",
        "`",
        "``",
        "`www.x.y`",
        "`a@b.c`",
        "\n```\nhttp://code.example\n```\n",
        "\n    www.code.example\n",
        "*",
        "**",
        "_",
        "~~",
        "<",
        ">",
        "<http://a.b>",
        "<a@b.c>",
        "<span title=\"www.x.y\">",
        "\n",
        "\r\n",
        "\r",
        "  \n",
        "\\\n",
        "\n\n",
        " ",
        "a",
        "text ",
        "$",
        "^[",
        "{",
        "}",
        "| a | b |\n|---|---|\n| ",
        " | ",
        "\n- ",
        "\n> ",
        "\n# ",
        "\n1. ",
        "\n> - ",
        "\t",
        "// note\n",
        "\0",
        "é",
        "中",
        "🙂",
        "abcdefghijklmnopqrstuvwxyz0123",
        "a long run of ordinary prose without any trigger in it ",
    ];
    let mut next = generator(0x2545_f491_4f6c_dd1d);
    let matrix = option_matrix();
    let before = indexed_so_far();
    let mut checks = 0usize;
    let mut linked = 0usize;
    for round in 0..3_000 {
        let len = next() % 48;
        let mut source = String::new();
        if round % 5 == 0 {
            source.push_str("---\ntitle: www.front.example a@b.c\n---\n");
        }
        for _ in 0..len {
            let token = tokens[next() % tokens.len()];
            // A NUL leaves the whole document to the full pass.
            if token != "\0" || round % 8 == 0 {
                source.push_str(token);
            }
        }
        linked += usize::from(check(&source, &matrix[0]));
        check(&source, &matrix[1 + round % (matrix.len() - 1)]);
        checks += 2;
    }
    let indexed = indexed_so_far() - before;
    assert!(checks > 5_000, "expected a broad sweep, got {checks}");
    assert!(linked > 500, "expected generated links, got {linked}");
    assert!(
        indexed > 5_000,
        "expected blocks answered from the offsets, got {indexed}"
    );
    println!("generated documents: {checks} checks, {linked} with links, {indexed} indexed blocks");
}
