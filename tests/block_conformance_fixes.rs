//! Two CommonMark 0.31.2 block rules the parser did not follow.
//!
//! * Section 2.2 expands a tab after a list marker to a tab stop of four
//!   columns, so `-\tfoo` starts its content in column 4 and section 5.2
//!   asks continuation lines for that much indentation.
//! * Section 4.6, start condition 1: a `<pre>`, `<script>`, `<style>` or
//!   `<textarea>` block ends on the first line containing *any* of the four
//!   end tags, whichever one opened it, and the end tag has to be complete.
//!
//! See `docs/decisions/2026-09-21-mdx-flow-and-block-conformance.md`.

use ferromark::allocator::Allocator;
use ferromark::ast::Node;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

fn commonmark_html(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::commonmark())
        .parse()
        .expect("block conformance fixtures parse");
    HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&document)
}

fn first_html_block(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::commonmark())
        .parse()
        .expect("block conformance fixtures parse");
    match &document.children[0] {
        Node::Html(html) => html.value.to_owned(),
        other => panic!("expected an HTML block, got {other:?}"),
    }
}

#[test]
fn tab_after_a_bullet_marker_starts_the_content_in_column_four() {
    // The tab runs from column 1 to the tab stop at column 4, so the item
    // content is indented four columns and `  bar` (two) closes the list.
    // The item used to take the content indent from the marker alone and
    // swallowed `bar` as a second paragraph.
    assert_eq!(
        commonmark_html("-\tfoo\n\n  bar\n"),
        "<ul>\n<li>foo</li>\n</ul>\n<p>bar</p>\n"
    );
    assert_eq!(
        commonmark_html("*\tfoo\n\n  bar\n"),
        "<ul>\n<li>foo</li>\n</ul>\n<p>bar</p>\n"
    );
}

#[test]
fn tab_after_an_ordered_marker_starts_the_content_in_column_four() {
    assert_eq!(
        commonmark_html("1.\tfoo\n\n   bar\n"),
        "<ol>\n<li>foo</li>\n</ol>\n<p>bar</p>\n"
    );
}

#[test]
fn a_continuation_indented_to_the_content_column_stays_in_the_item() {
    // Four spaces (or a tab) reach the content column, so these keep the
    // paragraph inside the item — and the blank line makes the list loose.
    assert_eq!(
        commonmark_html("-\tfoo\n\n    bar\n"),
        "<ul>\n<li><p>foo</p>\n<p>bar</p>\n</li>\n</ul>\n"
    );
    assert_eq!(
        commonmark_html("1.\tfoo\n\n    bar\n"),
        "<ol>\n<li><p>foo</p>\n<p>bar</p>\n</li>\n</ol>\n"
    );
    assert_eq!(
        commonmark_html("-\tfoo\n\tbar\n"),
        "<ul>\n<li>foo\nbar</li>\n</ul>\n"
    );
}

#[test]
fn mixed_spaces_and_tabs_after_a_marker_use_the_expanded_column() {
    // `-  \tfoo` expands to `-   foo`: content in column 4, so three
    // spaces are not enough to continue the item.
    assert_eq!(
        commonmark_html("-  \tfoo\n\n   bar\n"),
        "<ul>\n<li>foo</li>\n</ul>\n<p>bar</p>\n"
    );
    // `-\t foo` expands to `-    foo`: four columns of separation, content
    // in column 5.
    assert_eq!(
        commonmark_html("-\t foo\n\n     bar\n"),
        "<ul>\n<li><p>foo</p>\n<p>bar</p>\n</li>\n</ul>\n"
    );
    // Four spaces fall one column short of that content column, so the
    // list ends and the line is an indented code block of its own.
    assert_eq!(
        commonmark_html("-\t foo\n\n    bar\n"),
        "<ul>\n<li>foo</li>\n</ul>\n<pre><code>bar\n</code></pre>\n"
    );
}

#[test]
fn two_tabs_after_a_marker_still_start_indented_code() {
    // Spec example 9: five or more columns of separation mean the item
    // starts with indented code, the content offset is the marker width
    // plus one, and the remaining columns stay in the code block.
    assert_eq!(
        commonmark_html("-\t\tfoo\n"),
        "<ul>\n<li>\n<pre><code>  foo\n</code></pre>\n</li>\n</ul>\n"
    );
}

#[test]
fn tabbed_items_still_group_into_one_list() {
    assert_eq!(
        commonmark_html("-\tfoo\n-\tbar\n"),
        "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>\n"
    );
    // Four columns reach the content column, so this marker nests; two do
    // not, so that one is a sibling.
    assert_eq!(
        commonmark_html("-\tfoo\n    - bar\n"),
        "<ul>\n<li>foo\n<ul>\n<li>bar</li>\n</ul>\n</li>\n</ul>\n"
    );
    assert_eq!(
        commonmark_html("-\tfoo\n  - bar\n"),
        "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>\n"
    );
}

#[test]
fn a_stripped_container_keeps_the_narrow_reading_of_a_marker_tab() {
    // A container hands its content to a second parse with the prefix
    // removed, so the column a tab expands to is no longer knowable there:
    // the tab in `> -\tfoo` really is one column wide, the one in `-\tfoo`
    // three. Inside a container the item keeps the narrowest reading, one
    // separating column, which is what these two lines need to stay in the
    // item — and what they did before the column fix.
    assert_eq!(
        commonmark_html("> -\tfoo\n>\n>   bar\n"),
        "<blockquote>\n<ul>\n<li><p>foo</p>\n<p>bar</p>\n</li>\n</ul>\n</blockquote>\n"
    );
    // The five-column rule does not depend on the starting column, so
    // indented code inside an item is the same in both places.
    assert_eq!(
        commonmark_html(">\t\tfoo\n"),
        "<blockquote>\n<pre><code>  foo\n</code></pre>\n</blockquote>\n"
    );
    assert_eq!(
        commonmark_html("> -\t\tfoo\n"),
        "<blockquote>\n<ul>\n<li>\n<pre><code>  foo\n</code></pre>\n</li>\n</ul>\n</blockquote>\n"
    );
}

#[test]
fn a_type1_block_ends_at_any_raw_text_end_tag() {
    // `</script>` closes a `<pre>` block: the end condition lists all four
    // tags and does not require the block's own. This used to run to the
    // end of the document.
    assert_eq!(
        first_html_block("<pre>\nx\n</script>\ny\n\nz\n"),
        "<pre>\nx\n</script>\n"
    );
    assert_eq!(
        commonmark_html("<pre>\nx\n</script>\ny\n\nz\n"),
        "<pre>\nx\n</script>\n<p>y</p>\n<p>z</p>\n"
    );
    assert_eq!(
        commonmark_html("<style>\np {}\n</textarea>\ny\n"),
        "<style>\np {}\n</textarea>\n<p>y</p>\n"
    );
}

#[test]
fn a_type1_end_tag_is_matched_case_insensitively() {
    assert_eq!(
        commonmark_html("<textarea>\nx\n</PRE>\ny\n\nz\n"),
        "<textarea>\nx\n</PRE>\n<p>y</p>\n<p>z</p>\n"
    );
    assert_eq!(
        first_html_block("<script>\nlet a = 1 < 2;\n</SCRIPT> tail\n\nAfter"),
        "<script>\nlet a = 1 < 2;\n</SCRIPT> tail\n"
    );
}

#[test]
fn a_type1_end_tag_on_the_opening_line_ends_the_block() {
    assert_eq!(
        commonmark_html("<script>alert(1)</style>\ntext\n"),
        "<script>alert(1)</style>\n<p>text</p>\n"
    );
}

#[test]
fn a_type1_block_needs_a_complete_end_tag() {
    // `</script` never closes anything, and neither does `</pre >`: the
    // end condition spells the four end tags out in full.
    let unterminated = "<pre>\nx\n</script\ny\n\nz\n";
    assert_eq!(first_html_block(unterminated), unterminated);
    let spaced = "<pre>\nx\n</pre >\ny\n";
    assert_eq!(first_html_block(spaced), spaced);
    // A tag whose name only starts with a raw-text name does not close it.
    let prefixed = "<pre>\nx\n</prefix>\ny\n";
    assert_eq!(first_html_block(prefixed), prefixed);
}
