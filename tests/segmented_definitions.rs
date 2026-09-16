//! Rendered-output regressions for the segmented definition pre-pass.
//!
//! The pre-pass no longer block-parses the whole document to find link
//! reference and footnote definitions; it parses only the segments that can
//! hold one, each bounded by a line start where the real parser is at the
//! document root (see `docs/decisions/2026-09-16-segmented-definition-pass.md`).
//! These tests fix the observable consequences of that boundary rule: which
//! `[label]:` lines become definitions and which stay text.

use std::time::{Duration, Instant};

use ferromark::parser::ParserOptions;
use ferromark::renderer::HtmlRendererOptions;
use ferromark::{to_html, to_html_with_options};

fn render(source: &str, options: ParserOptions) -> String {
    to_html_with_options(source, options, HtmlRendererOptions::new()).expect("source should parse")
}

fn gfm(source: &str) -> String {
    render(source, ParserOptions::gfm())
}

fn commonmark(source: &str) -> String {
    to_html(source).expect("source should parse")
}

fn with_definition_lists(source: &str) -> String {
    render(
        source,
        ParserOptions {
            definition_lists: true,
            ..ParserOptions::gfm()
        },
    )
}

/// The three line endings the boundary scan has to treat alike.
fn line_ending_variants(source: &str) -> [String; 3] {
    [
        source.to_owned(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
    ]
}

#[track_caller]
fn assert_defines(html: &str, url: &str) {
    assert!(html.contains(url), "expected a link to {url} in {html}");
    assert!(!html.contains("]:"), "definition leaked as text: {html}");
}

#[track_caller]
fn assert_does_not_define(html: &str, label: &str) {
    assert!(
        !html.contains("href=\"/hidden\""),
        "{label} must not define a reference: {html}"
    );
}

#[test]
fn an_unclosed_quoted_fence_does_not_hide_a_later_root_definition() {
    // CommonMark 0.31.2 example 128: the fence ends with its block quote, so
    // `bbb` is a root paragraph and the definition after it is real.
    for source in line_ending_variants("[ref]\n\n> ```\n> aaa\n\nbbb\n\n[ref]: /url\n") {
        assert_defines(&gfm(&source), "href=\"/url\"");
    }
    for source in line_ending_variants("[ref]\n\n> ~~~\n> aaa\n\nbbb\n\n[ref]: /url\n") {
        assert_defines(&gfm(&source), "href=\"/url\"");
    }
}

#[test]
fn a_fence_indented_inside_a_list_item_hides_only_its_own_content() {
    let source = "[ref]\n\n- item\n\n  ```\n  [ref]: /hidden\n  ```\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert_does_not_define(&html, "an indented fence");
}

#[test]
fn a_root_html_block_containing_a_fence_line_does_not_open_one() {
    // The fence run is HTML content, so the definition after the block is
    // still discovered at the root.
    let source = "[ref]\n\n<div>\n```\n</div>\n\n[ref]: /url\n";
    assert_defines(&gfm(source), "href=\"/url\"");
}

#[test]
fn an_html_comment_spanning_blank_lines_defines_nothing() {
    let source = "[ref]\n\n<!-- start\n\n[ref]: /hidden\n\n-->\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert_does_not_define(&html, "an HTML comment");
}

#[test]
fn a_script_block_containing_a_definition_shape_defines_nothing() {
    let source =
        "[ref]\n\n<script>\nconst map = { x: 1 }\n[ref]: /hidden\n</script>\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert_does_not_define(&html, "a script block");
}

#[test]
fn a_type_seven_tag_after_a_paragraph_line_keeps_its_document_definitions() {
    // `<custom>` cannot interrupt a paragraph, so the planner cannot place it;
    // whichever way it is resolved, the definitions must not change.
    let source = "[ref]\n\nparagraph\n<custom>\n\n```\n[ref]: /hidden\n```\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert_does_not_define(&html, "a fence after a type-7 line");
}

#[test]
fn a_definition_list_body_after_a_blank_line_still_holds_its_definition() {
    let source = "[ref]\n\nterm\n\n: body text\n\n  [ref]: /url\n";
    let html = with_definition_lists(source);
    assert!(html.contains("href=\"/url\""), "{html}");

    // With the option off the same lines are ordinary paragraphs, and the
    // definition is a root one.
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
}

#[test]
fn a_tilde_fence_hides_backtick_runs_and_the_definitions_between_them() {
    let source = "[ref]\n\n~~~\n```\n[ref]: /hidden\n```\n~~~\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert_does_not_define(&html, "a tilde-fenced block");
}

#[test]
fn a_closer_longer_than_its_opener_still_closes_the_fence() {
    for source in line_ending_variants("[ref]\n\n```\n[ref]: /hidden\n`````\n\n[ref]: /url\n") {
        let html = gfm(&source);
        assert!(html.contains("href=\"/url\""), "{html}");
        assert_does_not_define(&html, "a fenced block");
    }
}

#[test]
fn a_backtick_in_an_info_string_is_not_a_fence() {
    // ``` a ``` is an inline code span, so the line below it is a root
    // definition rather than fenced content.
    let source = "[ref]\n\n``` a ```\n\n[ref]: /url\n";
    assert_defines(&gfm(source), "href=\"/url\"");
}

#[test]
fn definitions_in_lists_and_block_quotes_are_still_collected() {
    for container in [
        "- [ref]: /url",
        "> [ref]: /url",
        "- item\n\n  [ref]: /url",
        "> - [ref]: /url",
        "1. [ref]: /url",
    ] {
        let html = commonmark(&format!("[ref]\n\n{container}\n"));
        assert!(html.contains("href=\"/url\""), "{container:?}: {html}");
    }
}

#[test]
fn the_first_definition_wins_across_two_segments() {
    // The candidates sit in different segments, separated by a fenced block
    // that holds no candidate at all.
    let source = "[ref]\n\n[ref]: /first\n\n```\ncode\n```\n\n[ref]: /second\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/first\""), "{html}");
    assert!(!html.contains("/second"), "{html}");
}

#[test]
fn a_footnote_and_a_link_definition_in_different_segments_both_resolve() {
    let source = "Text[^note] and [ref].\n\n[^note]: the note\n\n```\ncode\n```\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert!(html.contains("the note"), "{html}");
    assert!(html.contains("footnote"), "{html}");
}

#[test]
fn a_lazy_continuation_of_a_list_item_is_not_a_definition() {
    let source = "[ref]\n\n- item\n[ref]: /hidden\n";
    let html = gfm(source);
    assert_does_not_define(&html, "a lazy continuation");
    assert!(
        html.contains("[ref]:"),
        "the line stays paragraph text: {html}"
    );
}

#[test]
fn a_definition_right_after_a_heading_is_collected() {
    for source in line_ending_variants("[ref]\n\n# Heading\n[ref]: /url\n") {
        assert_defines(&gfm(&source), "href=\"/url\"");
    }
}

#[test]
fn definitions_on_the_first_and_last_line_are_collected() {
    let html = gfm("[a]: /first\n\ntext\n\n[a] [b]\n\n[b]: /last");
    assert!(html.contains("href=\"/first\""), "{html}");
    assert!(html.contains("href=\"/last\""), "{html}");
}

#[test]
fn a_candidate_inside_an_unterminated_fence_never_defines() {
    let source = "[ref]\n\n```\n[ref]: /hidden\n";
    let html = gfm(source);
    assert_does_not_define(&html, "an unterminated fence");
    assert!(html.contains("<code>"), "{html}");
}

#[test]
fn an_indented_fence_inside_a_list_item_keeps_a_following_root_definition() {
    // The shape that made real documentation fall back: every fence in the
    // item body is indented, and the item is followed by a root definition.
    for source in line_ending_variants(
        "[ref]\n\n- first\n  ```ts\n  const a: 1\n  ```\n- second\n  ```json\n  {}\n  ```\n\n[ref]: /url\n",
    ) {
        assert_defines(&gfm(&source), "href=\"/url\"");
    }
}

#[test]
fn a_root_fence_indented_one_column_still_hides_its_content() {
    let source = "[ref]\n\n ```\n[ref]: /hidden\n ```\n\n[ref]: /url\n";
    let html = gfm(source);
    assert!(html.contains("href=\"/url\""), "{html}");
    assert_does_not_define(&html, "a fence indented one column");
}

#[test]
fn an_unclosed_root_fence_indented_one_column_hides_everything_after_it() {
    let source = "[ref]\n\n ```\n[ref]: /hidden\n";
    assert_does_not_define(&gfm(source), "an unclosed indented fence");
}

#[test]
fn nested_footnote_definitions_keep_their_labels() {
    let source = "Text[^outer]\n\n[^outer]: body[^inner]\n\n    [^inner]: nested\n";
    let html = gfm(source);
    assert!(html.contains("footnote"), "{html}");
    assert!(!html.contains("[^inner]:"), "{html}");
}

#[test]
fn a_document_without_any_candidate_renders_unchanged() {
    // The pre-pass must still short-circuit: `]:` in prose is not a shape.
    let html = gfm("Earlier [link](https://example.com).\n\n- Skip `]:` in prose.\n");
    assert!(html.contains("https://example.com"), "{html}");
    assert!(html.contains("]:"), "{html}");
}

/// Shapes whose planning must not rescan the same bytes once per line:
/// indented fence-run lines that never close, and indented HTML openers that
/// never terminate. The planner either skips such a region or hands the
/// document to the full pass; it never walks the region line by line.
fn indented_opener_run(line: &str, count: usize) -> String {
    let mut source = String::with_capacity(count * (line.len() + 1) + 16);
    for _ in 0..count {
        source.push_str(line);
        source.push('\n');
    }
    source.push_str("\n[ref]: /url\n");
    source
}

fn parse_batch(source: &str, runs: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..runs {
        assert!(
            to_html_with_options(source, ParserOptions::gfm(), HtmlRendererOptions::new()).is_ok()
        );
    }
    start.elapsed()
}

#[track_caller]
fn assert_planning_is_linear(line: &str) {
    let small = indented_opener_run(line, 2_000);
    let large = indented_opener_run(line, 8_000);
    // Warm up, then keep the best of several alternating rounds so a noisy
    // machine cannot fail the guard on scheduling alone.
    let _ = parse_batch(&small, 1);
    let _ = parse_batch(&large, 1);
    let mut best = f64::INFINITY;
    for round in 0..5 {
        let (small_batch, large_once) = if round % 2 == 0 {
            (parse_batch(&small, 4), parse_batch(&large, 1))
        } else {
            let large_once = parse_batch(&large, 1);
            (parse_batch(&small, 4), large_once)
        };
        let ratio =
            large_once.as_secs_f64() / small_batch.max(Duration::from_micros(1)).as_secs_f64();
        best = best.min(ratio);
    }
    assert!(
        best < 3.0,
        "{line:?}: 8,000 lines cost x{best:.1} of four 2,000-line runs; planning rescans"
    );
}

#[test]
fn indented_fence_openers_plan_in_linear_time() {
    assert_planning_is_linear("   ```a");
}

#[test]
fn indented_html_openers_plan_in_linear_time() {
    assert_planning_is_linear("   <!-- x");
}
