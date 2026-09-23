//! Heading IDs planned in the planner's claim storage must render exactly as
//! the map-keyed planner and scratch-buffer write path they replaced.
//!
//! [`LegacyHeadings`] is a render hook that takes over every heading and
//! writes it the way `render_heading_with_hooks` did before claim storage:
//! the text and slug scratch buffers, the map-keyed planner, the prefix
//! inserted in front of the planned ID, and the permalink read from that
//! concatenated ID. The tests render the same documents through the built-in
//! renderer and through that hook and require byte-identical HTML, for
//! one-shot rendering, a renderer reused across documents, and committed and
//! provisional incremental fragments, under permalinks, ID prefixes, level
//! offsets, explicit IDs, semantic footnotes, and source spans.
//!
//! The inputs are the bundled specification examples, generated documents
//! dense with colliding headings, and the frozen measurement corpora.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::print_stdout
)]

use std::fmt::Write as _;
use std::path::Path;

use crate::allocator::Allocator;
use crate::ast::{Document, Heading, Node};
use crate::parser::{Parser, ParserOptions, gunzip};
use crate::renderer::html::heading::planner_tests::MapKeyedPlanner;
use crate::renderer::html::heading::{
    collect_heading_text_into, heading_has_permalink_marker, slugify_heading_into,
};
use crate::renderer::html::{
    HEADING_PERMALINK_CLASS, HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer,
    HtmlRendererOptions, map_heading_level,
};

/// One renderer configuration of the comparison matrix.
#[derive(Debug, Clone, Copy)]
struct Config {
    name: &'static str,
    permalinks: bool,
    prefix: &'static str,
    offset: i32,
    source_spans: bool,
    heading_attributes: bool,
    semantic_footnotes: bool,
}

const PLAIN: Config = Config {
    name: "default",
    permalinks: false,
    prefix: "",
    offset: 0,
    source_spans: false,
    heading_attributes: false,
    semantic_footnotes: false,
};

const CONFIGS: [Config; 5] = [
    PLAIN,
    Config {
        name: "permalinks",
        permalinks: true,
        ..PLAIN
    },
    Config {
        name: "permalinks, prefix, offset",
        permalinks: true,
        prefix: "docs-",
        offset: 1,
        ..PLAIN
    },
    Config {
        name: "explicit ids, permalinks, prefix, offset, spans",
        permalinks: true,
        prefix: "p_",
        offset: -1,
        source_spans: true,
        heading_attributes: true,
        ..PLAIN
    },
    Config {
        name: "explicit ids, permalinks, semantic footnotes",
        permalinks: true,
        heading_attributes: true,
        semantic_footnotes: true,
        ..PLAIN
    },
];

fn renderer_for(config: Config) -> HtmlRenderer {
    HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: config.permalinks,
        semantic_footnotes: config.semantic_footnotes,
        source_spans: config.source_spans,
        ..Default::default()
    })
    .with_heading_level_offset(config.offset)
    .try_with_heading_id_prefix(config.prefix)
    .unwrap()
}

/// Renders headings with the routine claim storage replaced.
///
/// Everything but the ID and the permalink is the hook renderer's own heading
/// markup, so the comparison isolates the heading ID path.
#[derive(Clone)]
struct LegacyHeadings {
    config: Config,
    planner: MapKeyedPlanner,
    heading_text_scratch: String,
    heading_slug_scratch: String,
    heading_id_scratch: String,
    heading_id_is_explicit: bool,
}

impl LegacyHeadings {
    fn new(config: Config) -> Self {
        Self {
            config,
            planner: MapKeyedPlanner::default(),
            heading_text_scratch: String::new(),
            heading_slug_scratch: String::new(),
            heading_id_scratch: String::new(),
            heading_id_is_explicit: false,
        }
    }

    fn render_heading(&mut self, heading: &Heading<'_>, cx: &mut HtmlRenderContext<'_>) {
        let depth = map_heading_level(heading.depth, self.config.offset);
        cx.write("<h");
        cx.write_display(depth);
        cx.write(" id=\"");
        self.prepare_heading_id(heading);
        self.write_prepared_heading_id(cx);
        cx.write("\"");
        if !heading.classes.is_empty() {
            cx.write(" class=\"");
            for (index, class_name) in heading.classes.iter().enumerate() {
                if index > 0 {
                    cx.write(" ");
                }
                cx.write_attribute_escaped(class_name);
            }
            cx.write("\"");
        }
        if self.config.source_spans && heading.span.start != heading.span.end {
            cx.write(" data-source-span=\"");
            cx.write_display(heading.span.start);
            cx.write("-");
            cx.write_display(heading.span.end);
            cx.write("\"");
        }
        cx.write(">");
        cx.render_nodes(&heading.children, self);
        self.write_heading_permalink_if_needed(heading, cx);
        cx.write("</h");
        cx.write_display(depth);
        cx.write(">\n");
    }

    // The three methods below are the previous `write.rs` routines, with the
    // renderer's fields moved onto the hook.

    fn write_prepared_heading_id(&self, cx: &mut HtmlRenderContext<'_>) {
        if self.heading_id_is_explicit {
            cx.write_attribute_escaped(&self.heading_id_scratch);
        } else {
            cx.write(&self.heading_id_scratch);
        }
    }

    fn write_heading_permalink_if_needed(
        &self,
        heading: &Heading<'_>,
        cx: &mut HtmlRenderContext<'_>,
    ) {
        if !self.config.permalinks {
            return;
        }
        // The previous check compared links against the whole prefixed ID;
        // an empty prefix is that comparison (see `heading::tests`).
        if heading_has_permalink_marker(&heading.children, "", &self.heading_id_scratch) {
            return;
        }
        cx.write("<a class=\"");
        cx.write(HEADING_PERMALINK_CLASS);
        cx.write("\" href=\"#");
        self.write_prepared_heading_id(cx);
        if self.heading_text_scratch.is_empty() {
            cx.write("\" aria-label=\"Permalink to this section\">#</a>");
            return;
        }
        cx.write("\" aria-label=\"Permalink to &quot;");
        cx.write_escaped(&self.heading_text_scratch);
        cx.write("&quot;\">#</a>");
    }

    fn prepare_heading_id(&mut self, heading: &Heading<'_>) {
        let single_text = match &heading.children[..] {
            [Node::Text(text)] => Some(text.value),
            _ => None,
        };
        let permalink_reads_text = self.config.permalinks;
        self.heading_text_scratch.clear();
        if permalink_reads_text || (heading.id.is_none() && single_text.is_none()) {
            collect_heading_text_into(&heading.children, &mut self.heading_text_scratch);
        }

        if let Some(id) = heading.id {
            self.heading_id_is_explicit = true;
            self.planner.plan_into(id, &mut self.heading_id_scratch);
            self.apply_heading_id_prefix();
            return;
        }
        self.heading_id_is_explicit = false;
        self.heading_slug_scratch.clear();
        if let Some(text) = single_text {
            slugify_heading_into(text, &mut self.heading_slug_scratch);
        } else {
            slugify_heading_into(&self.heading_text_scratch, &mut self.heading_slug_scratch);
        }

        self.planner
            .plan_into(&self.heading_slug_scratch, &mut self.heading_id_scratch);
        self.apply_heading_id_prefix();
    }

    fn apply_heading_id_prefix(&mut self) {
        let prefix = self.config.prefix;
        if prefix.is_empty() {
            return;
        }
        self.heading_id_scratch.insert_str(0, prefix);
    }
}

impl HtmlRenderHooks for LegacyHeadings {
    fn render_node(
        &mut self,
        node: &Node<'_>,
        cx: &mut HtmlRenderContext<'_>,
    ) -> HtmlRenderControl {
        let Node::Heading(heading) = node else {
            return HtmlRenderControl::Default;
        };
        self.render_heading(heading, cx);
        HtmlRenderControl::Handled
    }
}

/// Renderers that live across the documents of one input set.
struct Lane {
    config: Config,
    /// Built-in renderer reused for every document, one-shot each time.
    reused: HtmlRenderer,
    /// Built-in renderer that sees every document as a committed fragment.
    incremental: HtmlRenderer,
    /// Hook renderer and legacy state for the same fragments.
    legacy_incremental: HtmlRenderer,
    legacy_state: LegacyHeadings,
}

impl Lane {
    fn new(config: Config) -> Self {
        Self {
            config,
            reused: renderer_for(config),
            incremental: renderer_for(config),
            legacy_incremental: renderer_for(config),
            legacy_state: LegacyHeadings::new(config),
        }
    }

    fn check(&mut self, name: &str, document: &Document<'_>) {
        let label = format!("{name} [{}]", self.config.name);
        let expected = renderer_for(self.config)
            .render_with_hooks(document, &mut LegacyHeadings::new(self.config));
        assert_same(
            &format!("{label} fresh"),
            &renderer_for(self.config).render(document),
            &expected,
        );
        assert_same(
            &format!("{label} reused"),
            self.reused.render_borrowed(document),
            &expected,
        );

        // A provisional fragment sees the committed state and leaves it
        // untouched; the legacy side gets a throwaway copy of its state.
        let expected_provisional = self
            .legacy_incremental
            .render_provisional_fragment_with_hooks(document, &mut self.legacy_state.clone());
        assert_same(
            &format!("{label} provisional"),
            &self.incremental.render_provisional_fragment(document),
            &expected_provisional,
        );
        let expected_committed = self
            .legacy_incremental
            .render_incremental_fragment_with_hooks(document, &mut self.legacy_state);
        assert_same(
            &format!("{label} committed"),
            &self.incremental.render_incremental_fragment(document),
            &expected_committed,
        );
    }
}

/// Compares two renders, reporting the first difference with some context
/// rather than two whole documents.
#[track_caller]
fn assert_same(label: &str, actual: &str, expected: &str) {
    if actual == expected {
        return;
    }
    let at = actual
        .bytes()
        .zip(expected.bytes())
        .take_while(|(left, right)| left == right)
        .count();
    let context = |text: &str| {
        let mut start = at.saturating_sub(120);
        while !text.is_char_boundary(start) {
            start -= 1;
        }
        let mut end = (at + 120).min(text.len());
        while !text.is_char_boundary(end) {
            end += 1;
        }
        text[start..end].to_owned()
    };
    panic!(
        "{label}: renders differ at byte {at}\n--- claim storage\n{}\n--- previous routine\n{}",
        context(actual),
        context(expected)
    );
}

/// Runs every configuration over `inputs`, keeping one lane per configuration
/// across the whole set so reuse and incremental state accumulate.
fn check_inputs<'a>(inputs: impl IntoIterator<Item = (String, ParserOptions, &'a str)>) -> usize {
    let mut lanes: Vec<Lane> = CONFIGS.into_iter().map(Lane::new).collect();
    let mut documents = 0usize;
    for (name, options, source) in inputs {
        for heading_attributes in [false, true] {
            let options = ParserOptions {
                heading_attributes,
                ..options.clone()
            };
            let allocator = Allocator::new();
            let Ok(document) = Parser::with_options(&allocator, source, options).parse() else {
                continue;
            };
            documents += 1;
            for lane in &mut lanes {
                if lane.config.heading_attributes == heading_attributes {
                    lane.check(&name, &document);
                }
            }
        }
    }
    documents
}

fn repository_path(relative: &str) -> std::path::PathBuf {
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
fn specification_examples_render_heading_ids_like_the_previous_routine() {
    let mut sources = Vec::new();
    for fixture in [
        "tests/spec_fixtures/commonmark-0.31.2-spec.txt",
        "tests/spec_fixtures/gfm-extensions-spec.txt",
    ] {
        let path = repository_path(fixture);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for (index, markdown) in spec_inputs(&text).into_iter().enumerate() {
            sources.push((format!("{fixture} example {}", index + 1), markdown));
        }
    }
    let documents = check_inputs(
        sources
            .iter()
            .map(|(name, source)| (name.clone(), ParserOptions::gfm(), source.as_str())),
    );
    assert!(
        documents > 1_000,
        "expected the spec suite, got {documents}"
    );
}

/// Deterministic xorshift, so a failure is reproducible from the test alone.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[(self.next() % items.len() as u64) as usize]
    }
}

/// Documents dense with headings whose generated, suffixed, explicit, and
/// prefixed IDs collide, in containers, footnotes, and setext form, some
/// carrying their own permalink markers.
fn colliding_heading_documents(seed: u64, count: usize) -> Vec<String> {
    const TEXTS: [&str; 30] = [
        "a",
        "A",
        "a 1",
        "a-1",
        "a-1-1",
        "a 2",
        "Section",
        "",
        "!!!",
        "section-1",
        "section",
        "はじめに",
        "*a*",
        "**A** `b`",
        "[a](./x.md)",
        "a [#](#a)",
        "a [#](#docs-a)",
        "a [#](#p_a-1)",
        "a <a class=\"header-anchor\" href=\"#a\">#</a>",
        "Über Größe",
        "\u{0130}stanbul",
        "a & b",
        "a&b",
        "x\"y",
        "Options & Defaults",
        "An extended API reference heading describing configuration",
        "An extended API reference heading describing configuration 1",
        "b",
        "b-1",
        "b-2",
    ];
    const ATTRIBUTES: [&str; 14] = [
        "",
        "",
        "",
        "",
        " {#a}",
        " {#a-1}",
        " {#b-1}",
        " {#x\"y}",
        " {#section}",
        " {#a .c}",
        " {.cls}",
        " {#docs-a}",
        " {#a-1-1}",
        " {#日本}",
    ];
    let mut rng = Rng(seed);
    let mut documents = Vec::with_capacity(count);
    for _ in 0..count {
        let mut document = String::new();
        let headings = 1 + rng.next() % 40;
        for _ in 0..headings {
            let container = match rng.next() % 8 {
                0 => "> ",
                1 => "- ",
                _ => "",
            };
            let text = rng.pick(&TEXTS);
            let attributes = rng.pick(&ATTRIBUTES);
            if rng.next().is_multiple_of(5) {
                let _ = write!(
                    document,
                    "{container}{text}{attributes}\n{container}===\n\n"
                );
            } else {
                let level = 1 + rng.next() % 6;
                let marker = "#".repeat(level as usize);
                let _ = write!(document, "{container}{marker} {text}{attributes}\n\n");
            }
            if rng.next().is_multiple_of(6) {
                document.push_str("Text with a footnote[^n].\n\n[^n]: # a\n\n");
            }
        }
        documents.push(document);
    }
    documents
}

#[test]
fn colliding_headings_render_heading_ids_like_the_previous_routine() {
    let documents = colliding_heading_documents(0x5EED_0001, 600);
    let checked = check_inputs(documents.iter().enumerate().map(|(index, source)| {
        (
            format!("generated document {index}"),
            ParserOptions::gfm(),
            source.as_str(),
        )
    }));
    assert_eq!(checked, 2 * documents.len());
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
    check_inputs(cases.iter().map(|case| {
        let name = case["name"].as_str().expect("case name");
        let input = case["input"].as_str().expect("case input");
        let options = match case["profile"].as_str() {
            Some("commonmark") => ParserOptions::commonmark(),
            Some("gfm_spec") => ParserOptions::gfm_spec(),
            Some("mdx") => ParserOptions::mdx(),
            _ => ParserOptions::gfm(),
        };
        (format!("{relative}/{name}"), options, input)
    }));
}

#[test]
fn frozen_corpora_render_heading_ids_like_the_previous_routine() {
    check_corpus("docs/reports/2026-09-14-simd-round/corpus.json.gz", 72);
    check_corpus("docs/reports/2026-09-16-arm-round-3/corpus.json.gz", 207);
}
