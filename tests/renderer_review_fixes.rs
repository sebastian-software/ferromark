//! Regression tests for the renderer findings of the final v2 review.
//!
//! Each test pins one corrected behavior: bounded code-annotation ranges,
//! representable line-number starts, base-URL handling for root-absolute
//! links, and code-block state that does not leak across incremental
//! fragments.

use std::time::{Duration, Instant};

use ferromark::ast::Document;
use ferromark::{
    Allocator, CodeAnnotationSyntax, HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks, Parser,
    ParserOptions,
};

fn annotation_options(syntax: CodeAnnotationSyntax) -> HtmlRendererOptions {
    HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: syntax,
        ..HtmlRendererOptions::new()
    }
}

fn render(source: &str, options: HtmlRendererOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(options);
    renderer.render(&document)
}

fn render_timed(source: &str, options: HtmlRendererOptions) -> (String, Duration) {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(options);
    let started = Instant::now();
    let html = renderer.render(&document);
    (html, started.elapsed())
}

/// The guard budget for a range whose end is far past the block.
///
/// The fixed parser clamps the range to the block's own lines, so the render
/// takes microseconds; the unbounded expansion took 167 ms at 40,000 lines and
/// grew fourfold per doubling, and the VitePress form never returned. A one
/// second budget separates the two by orders of magnitude without making the
/// test sensitive to machine speed.
const RANGE_BUDGET: Duration = Duration::from_secs(1);

/// A block of two code lines.
///
/// A fence's value keeps its final newline, so this block has three line
/// states: the two code lines and the empty line the newline ends. A range
/// covering the whole block is therefore `1-3`.
const TWO_CODE_LINES: &str = "let x = 1;\nlet y = 2;\n";

/// A block of three code lines, four line states (see [`TWO_CODE_LINES`]).
const THREE_CODE_LINES: &str = "a;\nb;\nc;\n";

fn fence(meta: &str, value: &str) -> String {
    format!("```js {meta}\n{value}```")
}

#[test]
fn attribute_annotation_range_is_bounded_by_the_code_block() {
    let (huge, elapsed) = render_timed(
        &fence("annotate=\"highlight:1-10000000\"", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );
    let whole_block = render(
        &fence("annotate=\"highlight:1-3\"", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );

    assert_eq!(huge, whole_block);
    assert!(
        elapsed < RANGE_BUDGET,
        "clamped annotation range took {elapsed:?}"
    );
}

#[test]
fn vitepress_annotation_range_is_bounded_by_the_code_block() {
    let (huge, elapsed) = render_timed(
        &fence("{1-999999999999}", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::VitePress),
    );
    let whole_block = render(
        &fence("{1-3}", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::VitePress),
    );

    assert_eq!(huge, whole_block);
    assert!(
        elapsed < RANGE_BUDGET,
        "clamped VitePress range took {elapsed:?}"
    );
}

#[test]
fn annotation_range_inside_the_block_is_unchanged() {
    let html = render(
        &fence("annotate=\"highlight:1-3\"", THREE_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );

    assert_eq!(html.matches("ox-code-line--highlight").count(), 3, "{html}");
}

#[test]
fn annotation_range_past_the_end_annotates_the_lines_that_exist() {
    let clamped = render(
        &fence("annotate=\"highlight:2-9\"", THREE_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );
    let whole_tail = render(
        &fence("annotate=\"highlight:2-4\"", THREE_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );

    assert_eq!(clamped, whole_tail);
    assert_eq!(
        clamped.matches("ox-code-line--highlight").count(),
        3,
        "{clamped}"
    );
}

#[test]
fn annotation_range_starting_past_the_end_annotates_nothing() {
    // Clamping the end must not drag the start onto the final line: no line
    // named by `9-12` exists, so the block stays unannotated.
    let html = render(
        &fence("annotate=\"highlight:9-12\"", THREE_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );

    assert!(!html.contains("ox-code-line--highlight"), "{html}");
    assert!(!html.contains("ox-code-block--annotated"), "{html}");
}

#[test]
fn annotation_line_list_keeps_order_and_deduplicates() {
    // The parser sorts and de-duplicates; a shuffled list with a repeat and an
    // overlapping range renders exactly like the ascending unique list.
    let shuffled = render(
        &fence("annotate=\"highlight:3,1,2-3,1\"", THREE_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );
    let ascending = render(
        &fence("annotate=\"highlight:1,2,3\"", THREE_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::Attribute),
    );

    assert_eq!(shuffled, ascending);
}

#[test]
fn line_number_start_beyond_usize_saturates_instead_of_overflowing() {
    let html = render(
        &fence(":line-numbers=18446744073709551615", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::VitePress),
    );

    assert!(
        html.contains("data-line-number-start=\"18446744073709551615\""),
        "{html}"
    );
    assert_eq!(
        html.matches("data-line-number=\"18446744073709551615\"")
            .count(),
        3,
        "{html}"
    );
    assert!(!html.contains("data-line-number=\"0\""), "{html}");
}

#[test]
fn line_number_start_near_the_maximum_still_counts_up() {
    let html = render(
        &fence(":line-numbers=18446744073709551614", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::VitePress),
    );

    assert!(
        html.contains("data-line-number=\"18446744073709551614\""),
        "{html}"
    );
    assert_eq!(
        html.matches("data-line-number=\"18446744073709551615\"")
            .count(),
        2,
        "{html}"
    );
}

#[test]
fn ordinary_line_number_starts_are_unaffected_by_saturation() {
    let html = render(
        &fence(":line-numbers=18", TWO_CODE_LINES),
        annotation_options(CodeAnnotationSyntax::VitePress),
    );

    assert!(html.contains("data-line-number=\"18\""), "{html}");
    assert!(html.contains("data-line-number=\"19\""), "{html}");
    assert!(html.contains("data-line-number=\"20\""), "{html}");
}

#[test]
fn line_links_use_the_saturated_visible_line_number() {
    let html = render(
        &fence(
            ":line-numbers=18446744073709551615 :line-links",
            TWO_CODE_LINES,
        ),
        annotation_options(CodeAnnotationSyntax::VitePress),
    );

    // Each of the three line states carries the number twice, in `id` and in
    // `data-line-anchor`.
    assert_eq!(
        html.matches("-L18446744073709551615\"").count(),
        6,
        "{html}"
    );
}

fn link_options(base_url: &'static str) -> HtmlRendererOptions {
    HtmlRendererOptions {
        convert_md_links: true,
        base_url: base_url.into(),
        ..HtmlRendererOptions::new()
    }
}

#[test]
fn empty_base_keeps_root_absolute_links_root_absolute() {
    let html = render("[a](/b.md) [c](/d) [e](./f.md)", link_options(""));

    assert_eq!(
        html,
        "<p><a href=\"/b/index.html\">a</a> <a href=\"/d\">c</a> <a href=\"../f/index.html\">e</a></p>\n"
    );
}

#[test]
fn root_base_keeps_root_absolute_links_root_absolute() {
    let html = render("[a](/b.md) [c](/d) [e](./f.md)", link_options("/"));

    assert_eq!(
        html,
        "<p><a href=\"/b/index.html\">a</a> <a href=\"/d\">c</a> <a href=\"../f/index.html\">e</a></p>\n"
    );
}

#[test]
fn configured_base_prefixes_root_absolute_links() {
    for base_url in ["/docs", "/docs/"] {
        let html = render("[a](/b.md) [c](/d) [e](./f.md)", link_options(base_url));

        assert_eq!(
            html,
            "<p><a href=\"/docs/b/index.html\">a</a> <a href=\"/docs/d\">c</a> <a href=\"../f/index.html\">e</a></p>\n",
            "base {base_url}"
        );
    }
}

#[test]
fn empty_base_keeps_root_absolute_index_and_directory_links_root_absolute() {
    let html = render(
        "[root](/index.md) [dir](/lib/index.md) [page](/lib/guide.md?q=1#top)",
        link_options(""),
    );

    assert_eq!(
        html,
        concat!(
            "<p><a href=\"/index.html\">root</a> ",
            "<a href=\"/lib/index.html\">dir</a> ",
            "<a href=\"/lib/guide/index.html?q=1#top\">page</a></p>\n"
        )
    );
}

#[test]
fn empty_base_keeps_raw_html_root_absolute_links_root_absolute() {
    let html = render("<a href=\"/b.md\">a</a>\n", link_options(""));

    assert!(html.contains("href=\"/b/index.html\""), "{html}");
}

fn line_link_prefixes(html: &str) -> Vec<String> {
    html.match_indices("data-line-link-prefix=\"")
        .map(|(index, marker)| {
            let rest = &html[index + marker.len()..];
            let end = rest.find('"').unwrap();
            rest[..end].to_string()
        })
        .collect()
}

fn parse_into<'a>(allocator: &'a Allocator, source: &'a str) -> Document<'a> {
    Parser::with_options(allocator, source, ParserOptions::default())
        .parse()
        .unwrap()
}

#[test]
fn provisional_fragments_do_not_advance_the_code_block_index() {
    const SOURCE: &str = "```js :line-links\nlet x = 1;\n```";

    let allocator = Allocator::new();
    let document = parse_into(&allocator, SOURCE);

    let mut fresh = HtmlRenderer::with_options(annotation_options(CodeAnnotationSyntax::VitePress));
    let expected = fresh.render_incremental_fragment(&document);
    assert_eq!(line_link_prefixes(&expected), vec!["code-js-1".to_string()]);

    let mut renderer =
        HtmlRenderer::with_options(annotation_options(CodeAnnotationSyntax::VitePress));
    let provisional = renderer.render_provisional_fragment(&document);
    assert_eq!(
        line_link_prefixes(&provisional),
        vec!["code-js-1".to_string()]
    );

    let committed = renderer.render_incremental_fragment(&document);
    assert_eq!(committed, expected);
}

#[test]
fn committed_fragments_keep_advancing_the_code_block_index() {
    const SOURCE: &str = "```js :line-links\nlet x = 1;\n```";

    let allocator = Allocator::new();
    let document = parse_into(&allocator, SOURCE);

    let mut renderer =
        HtmlRenderer::with_options(annotation_options(CodeAnnotationSyntax::VitePress));
    let first = renderer.render_incremental_fragment(&document);
    let second = renderer.render_incremental_fragment(&document);

    assert_eq!(line_link_prefixes(&first), vec!["code-js-1".to_string()]);
    assert_eq!(line_link_prefixes(&second), vec!["code-js-2".to_string()]);
}

#[test]
fn resetting_incremental_state_restarts_the_code_block_index() {
    const SOURCE: &str = "```js :line-links\nlet x = 1;\n```";

    let allocator = Allocator::new();
    let document = parse_into(&allocator, SOURCE);

    let mut renderer =
        HtmlRenderer::with_options(annotation_options(CodeAnnotationSyntax::VitePress));
    let first = renderer.render_incremental_fragment(&document);
    let _ = renderer.render_incremental_fragment(&document);
    renderer.reset_incremental_state();
    let restarted = renderer.render_incremental_fragment(&document);

    assert_eq!(restarted, first);
    assert_eq!(
        line_link_prefixes(&restarted),
        vec!["code-js-1".to_string()]
    );
}

#[test]
fn provisional_fragments_with_hooks_do_not_advance_the_code_block_index() {
    const SOURCE: &str = "```js :line-links\nlet x = 1;\n```";

    let allocator = Allocator::new();
    let document = parse_into(&allocator, SOURCE);

    let mut renderer =
        HtmlRenderer::with_options(annotation_options(CodeAnnotationSyntax::VitePress));
    let mut hooks = NoHtmlRenderHooks;
    let provisional = renderer.render_provisional_fragment_with_hooks(&document, &mut hooks);
    assert_eq!(
        line_link_prefixes(&provisional),
        vec!["code-js-1".to_string()]
    );

    let committed = renderer.render_incremental_fragment(&document);
    assert_eq!(
        line_link_prefixes(&committed),
        vec!["code-js-1".to_string()]
    );
}
