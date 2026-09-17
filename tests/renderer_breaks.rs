//! `soft_break` and `hard_break` are caller-supplied strings written verbatim.
//!
//! `soft_break` was stored on the public options and never read before the
//! 2.0.0 API freeze, so these tests pin both that it now takes effect and that
//! the documented default leaves rendering exactly as it was.

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

/// Every renderer entry point must agree: the two inline text paths (the
/// specialized one in `visit_inline_node` and `render_text`) are reached by
/// different combinations of these.
fn render_all_paths(source: &str, options: &HtmlRendererOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();

    let html = HtmlRenderer::with_options(options.clone()).render(&document);

    let mut borrowed = HtmlRenderer::with_options(options.clone());
    assert_eq!(borrowed.render_borrowed(&document), html, "render_borrowed");

    let mut incremental = HtmlRenderer::with_options(options.clone());
    assert_eq!(
        incremental.render_incremental_fragment(&document),
        html,
        "render_incremental_fragment"
    );

    let mut hooked = HtmlRenderer::with_options(options.clone());
    assert_eq!(
        hooked.render_with_hooks(&document, &mut NoHtmlRenderHooks),
        html,
        "render_with_hooks"
    );

    html
}

fn with_soft_break(source: &str, soft_break: &'static str, xhtml: bool) -> String {
    render_all_paths(
        source,
        &HtmlRendererOptions {
            soft_break: soft_break.into(),
            xhtml,
            ..HtmlRendererOptions::new()
        },
    )
}

/// Documents whose rendering must not move when the default soft break is
/// spelled out instead of left implicit.
const CORPUS: &[&str] = &[
    "first\nsecond\n",
    "*emphasis\nover lines* and `code`\n",
    "setext\nheading\n===\n",
    "> quoted\n> lines\n",
    "- item one\n  continued\n- item two\n",
    "| a | b |\n| --- | --- |\n| c | d |\n",
    "[link\ntext](/target) and https://example.com/auto\n",
    "```\nfenced\ncode\n```\n",
    "line 1\\\nline 2\n",
    "text with &#10; a numeric line ending\n",
];

#[test]
fn the_default_soft_break_renders_a_line_ending() {
    assert_eq!(
        render_all_paths("first\nsecond\n", &HtmlRendererOptions::new()),
        "<p>first\nsecond</p>\n"
    );
}

#[test]
fn spelling_out_the_default_soft_break_changes_nothing() {
    for source in CORPUS {
        let implicit = render_all_paths(source, &HtmlRendererOptions::new());
        let explicit = with_soft_break(source, "\n", false);
        assert_eq!(explicit, implicit, "soft_break: \"\\n\" changed {source:?}");
    }
}

#[test]
fn a_space_soft_break_joins_the_lines() {
    assert_eq!(
        with_soft_break("first\nsecond\n", " ", false),
        "<p>first second</p>\n"
    );
}

#[test]
fn a_br_soft_break_is_written_verbatim_with_and_without_xhtml() {
    // `xhtml` self-closes the tags the renderer generates itself — `<hr>`,
    // `<img>`, `<col>` — and never rewrites a caller-supplied break string.
    // This mirrors `hard_break`, whose value is emitted verbatim too.
    for xhtml in [false, true] {
        assert_eq!(
            with_soft_break("first\nsecond\n", "<br>", xhtml),
            "<p>first<br>second</p>\n",
            "xhtml={xhtml}"
        );
        assert_eq!(
            with_soft_break("first\nsecond\n", "<br />", xhtml),
            "<p>first<br />second</p>\n",
            "xhtml={xhtml}"
        );
    }
}

#[test]
fn the_soft_break_reaches_every_inline_container() {
    assert_eq!(
        with_soft_break("*emphasis\nover lines*\n", " ", false),
        "<p><em>emphasis over lines</em></p>\n"
    );
    assert_eq!(
        with_soft_break("setext\nheading\n===\n", " ", false),
        "<h1 id=\"setext-heading\">setext heading</h1>\n"
    );
    assert_eq!(
        with_soft_break("> quoted\n> lines\n", " ", false),
        "<blockquote>\n<p>quoted lines</p>\n</blockquote>\n"
    );
    assert_eq!(
        with_soft_break("[link\ntext](/target)\n", " ", false),
        "<p><a href=\"/target\">link text</a></p>\n"
    );
}

#[test]
fn the_soft_break_leaves_hard_breaks_and_code_alone() {
    assert_eq!(
        with_soft_break("line 1\\\nline 2\n", " ", false),
        "<p>line 1<br>\nline 2</p>\n"
    );
    assert_eq!(
        with_soft_break("```\nfenced\ncode\n```\n", " ", false),
        "<pre><code>fenced\ncode\n</code></pre>\n"
    );
    assert_eq!(
        with_soft_break("`inline\ncode`\n", " ", false),
        "<p><code>inline code</code></p>\n"
    );
}

#[test]
fn both_break_strings_can_be_configured_together() {
    let html = render_all_paths(
        "first\nsecond\\\nthird\n",
        &HtmlRendererOptions {
            soft_break: " ".into(),
            hard_break: "<br />\n".into(),
            xhtml: true,
            ..HtmlRendererOptions::new()
        },
    );
    assert_eq!(html, "<p>first second<br />\nthird</p>\n");
}
