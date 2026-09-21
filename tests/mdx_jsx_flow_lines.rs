//! A flow JSX element owns its line when `ParserOptions.mdx` is true.
//!
//! `micromark-extension-mdx-jsx` accepts a tag in flow only while nothing
//! but whitespace follows it on that line; a tag with trailing content is
//! text JSX inside a paragraph instead. Self-closing tags already followed
//! that rule, paired tags did not, so `<A>x</A>- item` produced a flow
//! island plus a list. See
//! `docs/decisions/2026-09-21-mdx-flow-and-block-conformance.md`.

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn mdx_tree(source: &str) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, ParserOptions::mdx())
        .parse()
        .expect("parser should not fail on MDX flow-line fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn mdx_html(source: &str) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, ParserOptions::mdx())
        .parse()
        .expect("parser should not fail on MDX flow-line fixtures");
    HtmlRenderer::with_options(HtmlRendererOptions::new()).render(&doc)
}

#[test]
fn trailing_content_after_the_closer_makes_the_tag_text_jsx() {
    // Every one of these used to emit a flow island plus a second block
    // parsed from the trailing text: a list, an empty list item, and a
    // paragraph.
    for source in [
        "<A>x</A>- item\n",
        "<A>x</A>*\n",
        "<A>x</A> *em*\n",
        "<A>x</A>.\n",
    ] {
        let tree = mdx_tree(source);
        assert!(
            tree.contains("Paragraph"),
            "{source:?} should be a paragraph:\n{tree}"
        );
        assert!(
            tree.contains("MdxJsxTextElement name=Some(\"A\") self_closing=false"),
            "{source:?} should hold a text element:\n{tree}"
        );
        assert!(
            !tree.contains("MdxJsxFlowElement"),
            "{source:?} should not be flow:\n{tree}"
        );
    }
}

#[test]
fn trailing_content_renders_as_one_paragraph() {
    assert_eq!(
        mdx_html("<A>x</A>- item\n"),
        "<p><span class=\"ox-island\" data-ox-island=\"A\">x</span>- item</p>\n"
    );
    assert_eq!(
        mdx_html("<A>x</A> *em*\n"),
        "<p><span class=\"ox-island\" data-ox-island=\"A\">x</span> <em>em</em></p>\n"
    );
}

#[test]
fn a_tag_at_line_start_matches_the_same_tag_mid_line() {
    // `t <A>x</A> y` was already a paragraph with a text element; the
    // line-start form now agrees instead of splitting into two blocks.
    let at_line_start = mdx_tree("<A>x</A> y\n");
    let mid_line = mdx_tree("t <A>x</A> y\n");
    for tree in [&at_line_start, &mid_line] {
        assert!(
            tree.contains("MdxJsxTextElement name=Some(\"A\") self_closing=false")
                && !tree.contains("MdxJsxFlowElement"),
            "expected one paragraph with a text element:\n{tree}"
        );
    }
    assert!(
        at_line_start.contains("Text \" y\""),
        "the trailing text stays in the paragraph:\n{at_line_start}"
    );
}

#[test]
fn a_tag_alone_on_its_line_is_still_flow() {
    for source in [
        "<A>x</A>\n",
        "<A>x</A>   \n",
        "<A>x</A>\t\n",
        "<A>x</A>",
        "<A>\nx\n</A>\n",
        "<A />\n",
    ] {
        let tree = mdx_tree(source);
        assert!(
            tree.contains("MdxJsxFlowElement name=Some(\"A\")"),
            "{source:?} should stay flow:\n{tree}"
        );
    }
}

#[test]
fn self_closing_tags_keep_their_existing_line_rule() {
    // The self-closing check is the model for the paired one: a trailing
    // `x` turns `<A/>` into text JSX inside a paragraph.
    let tree = mdx_tree("<A/>x\n");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"A\") self_closing=true")
            && !tree.contains("MdxJsxFlowElement"),
        "expected a paragraph with a self-closing text element:\n{tree}"
    );
}

#[test]
fn a_multiline_element_needs_a_closer_alone_on_its_line() {
    // The rule applies to the closing line of a multi-line element too.
    // Such input is a syntax error in MDX; here it falls back to the
    // Markdown reading of the opening line, exactly like an element whose
    // closing tag is missing altogether.
    let unclosed = mdx_tree("<A>\nx\n");
    let trailing = mdx_tree("<A>\nx\n</A> tail\n");
    assert!(
        unclosed.contains("Html") && !unclosed.contains("MdxJsx"),
        "an unclosed element falls back to raw HTML:\n{unclosed}"
    );
    assert!(
        trailing.contains("Html") && !trailing.contains("MdxJsx"),
        "content after the closer falls back the same way:\n{trailing}"
    );
    assert_eq!(mdx_html("<A>\nx\n</A> tail\n"), "<A>\nx\n</A> tail\n");
}

#[test]
fn nested_flow_children_follow_the_same_rule() {
    // The children of a flow element are re-parsed as blocks, so the rule
    // holds one level down as well.
    let tree = mdx_tree("<Outer>\n<A>x</A> tail\n</Outer>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Outer\")"),
        "the outer element still owns its lines:\n{tree}"
    );
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"A\")"),
        "the inner tag is text JSX:\n{tree}"
    );
}
