//! Laziness only continues an open paragraph.
//!
//! CommonMark 5.1 and 5.2 let a line that lacks a container's marker
//! continue the container as paragraph continuation text, and nothing else:
//! after a closed fence, an HTML block, a heading, a thematic break, indented
//! code or a GFM table the container ends and the line starts a new block
//! outside it. The list item parser used to accept such a line after any
//! block, and the block quote parser after a closed fence, an HTML block or a
//! table, so `lazy` ended up inside the container — or inside the fence.
//!
//! These tests pin the spec behavior on the shapes that reached users, and
//! the shapes laziness still has to cover.

use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("source should parse");
    HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&document)
}

fn commonmark(source: &str) -> String {
    render(source, ParserOptions::commonmark())
}

fn gfm(source: &str) -> String {
    render(source, ParserOptions::gfm_spec())
}

#[test]
fn a_list_item_ends_after_a_closed_fence() {
    assert_eq!(
        commonmark("- a\n  ```\n  code\n  ```\nlazy\n"),
        "<ul>\n<li>a\n<pre><code>code\n</code></pre>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
    // An unindented line after an unclosed item fence is not inside it.
    assert_eq!(
        commonmark("- ```\nfoo\n```\n"),
        "<ul>\n<li>\n<pre><code></code></pre>\n</li>\n</ul>\n<p>foo</p>\n<pre><code></code></pre>\n"
    );
}

#[test]
fn a_list_item_ends_after_other_leaf_blocks() {
    assert_eq!(
        commonmark("- # h\nlazy\n"),
        "<ul>\n<li>\n<h1>h</h1>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("- ***\nlazy\n"),
        "<ul>\n<li>\n<hr>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("- <div>\nlazy\n"),
        "<ul>\n<li>\n<div>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("- a\n\n      code\nlazy\n"),
        "<ul>\n<li><p>a</p>\n<pre><code>code\n</code></pre>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("-     code\nlazy\n"),
        "<ul>\n<li>\n<pre><code>code\n</code></pre>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
}

#[test]
fn a_list_item_still_takes_lazy_paragraph_text() {
    assert_eq!(commonmark("- a\nlazy\n"), "<ul>\n<li>a\nlazy</li>\n</ul>\n");
    assert_eq!(
        commonmark("- a\n\n  b\nlazy\n"),
        "<ul>\n<li><p>a</p>\n<p>b\nlazy</p>\n</li>\n</ul>\n"
    );
    assert_eq!(
        commonmark("- - a\nlazy\n"),
        "<ul>\n<li>\n<ul>\n<li>a\nlazy</li>\n</ul>\n</li>\n</ul>\n"
    );
    assert_eq!(
        commonmark("1. a\n   b\nlazy\n"),
        "<ol>\n<li>a\nb\nlazy</li>\n</ol>\n"
    );
    // A marker that cannot interrupt a paragraph is continuation text.
    assert_eq!(
        commonmark("- a\n  b\n  2. c\nlazy\n"),
        "<ul>\n<li>a\nb\n2. c\nlazy</li>\n</ul>\n"
    );
}

#[test]
fn a_block_quote_ends_after_a_closed_fence_or_html_block() {
    assert_eq!(
        commonmark("> ```\n> x\n> ```\nlazy\n"),
        "<blockquote>\n<pre><code>x\n</code></pre>\n</blockquote>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("> ~~~\n> x\n> ~~~\nlazy\n"),
        "<blockquote>\n<pre><code>x\n</code></pre>\n</blockquote>\n<p>lazy</p>\n"
    );
    // An open fence is not a paragraph either: the quote ends there too.
    assert_eq!(
        commonmark("> ```\n> x\nlazy\n"),
        "<blockquote>\n<pre><code>x\n</code></pre>\n</blockquote>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("> <div>\nlazy\n"),
        "<blockquote>\n<div>\n</blockquote>\n<p>lazy</p>\n"
    );
    // A type-1 block closes on its own line; the paragraph after it is open.
    assert_eq!(
        commonmark("> <script>x</script>\n> y\nlazy\n"),
        "<blockquote>\n<script>x</script>\n<p>y\nlazy</p>\n</blockquote>\n"
    );
}

#[test]
fn a_block_quote_looks_through_nested_containers() {
    assert_eq!(
        commonmark("> - ```\n>   x\n>   ```\nlazy\n"),
        "<blockquote>\n<ul>\n<li>\n<pre><code>x\n</code></pre>\n</li>\n</ul>\n</blockquote>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("> > a\nlazy\n"),
        "<blockquote>\n<blockquote>\n<p>a\nlazy</p>\n</blockquote>\n</blockquote>\n"
    );
    assert_eq!(
        commonmark("> - a\nlazy\n"),
        "<blockquote>\n<ul>\n<li>a\nlazy</li>\n</ul>\n</blockquote>\n"
    );
}

#[test]
fn a_block_quote_paragraph_is_only_closed_by_real_headings() {
    // `#tag` is paragraph text, not a heading, so the paragraph stays open.
    assert_eq!(
        commonmark("> #tag\nlazy\n"),
        "<blockquote>\n<p>#tag\nlazy</p>\n</blockquote>\n"
    );
    assert_eq!(
        commonmark("> # h\nlazy\n"),
        "<blockquote>\n<h1>h</h1>\n</blockquote>\n<p>lazy</p>\n"
    );
    assert_eq!(
        commonmark("> a\n> ===\nlazy\n"),
        "<blockquote>\n<h1>a</h1>\n</blockquote>\n<p>lazy</p>\n"
    );
}

#[test]
fn a_gfm_table_takes_no_lazy_rows() {
    assert_eq!(
        gfm("> | a |\n> |---|\nlazy\n"),
        "<blockquote>\n<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n</blockquote>\n<p>lazy</p>\n"
    );
    assert_eq!(
        gfm("- | a |\n  |---|\nlazy\n"),
        "<ul>\n<li>\n<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n</li>\n</ul>\n<p>lazy</p>\n"
    );
    // Marked rows keep extending the table; the table is still not a paragraph.
    assert_eq!(
        gfm("> | a |\n> |---|\n> | 1 |\nlazy\n"),
        "<blockquote>\n<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>1</td>\n</tr>\n</tbody>\n</table>\n</blockquote>\n<p>lazy</p>\n"
    );
}

#[test]
fn an_indented_opening_fence_closes_only_on_a_bare_fence() {
    // The unindented opener already required a bare closing fence; the
    // indented opener accepted `` ``` bar `` as its closer.
    assert_eq!(
        commonmark(" ```\nfoo\n``` bar\nbaz\n```\n"),
        "<pre><code>foo\n``` bar\nbaz\n</code></pre>\n"
    );
    assert_eq!(
        commonmark(" ~~~\nfoo\n~~~ bar\n"),
        "<pre><code>foo\n~~~ bar\n</code></pre>\n"
    );
    assert_eq!(
        commonmark(" ```\nfoo\n ```  \nbar\n"),
        "<pre><code>foo\n</code></pre>\n<p>bar</p>\n"
    );
}
