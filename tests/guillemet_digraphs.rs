#![allow(clippy::panic, clippy::unwrap_used)]

use ferromark::{
    Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions,
    ast::{Node, Visit},
};

fn enabled() -> ParserOptions {
    ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    }
}

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions::default()).render(&document)
}

#[test]
fn digraph_parsing_is_opt_in_and_preserves_the_existing_default() {
    let source = "Il a dit <<Bonjour>>.";
    assert_eq!(
        render(source, ParserOptions::default()),
        "<p>Il a dit &lt;<Bonjour>&gt;.</p>\n"
    );
    assert_eq!(
        render(source, enabled()),
        "<p>Il a dit &lt;&lt;Bonjour&gt;&gt;.</p>\n"
    );

    for options in [
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        assert!(!options.guillemet_digraphs);
    }
}

#[test]
fn digraphs_keep_inline_markdown_and_inline_html_in_their_content() {
    assert_eq!(
        render("<<Bonjour *tout le monde* [guide](/guide)>>", enabled()),
        "<p>&lt;&lt;Bonjour <em>tout le monde</em> <a href=\"/guide\">guide</a>&gt;&gt;</p>\n"
    );
    assert_eq!(
        render("<<Bonjour <em>monde</em>>>!", enabled()),
        "<p>&lt;&lt;Bonjour <em>monde</em>&gt;&gt;!</p>\n"
    );
}

#[test]
fn bracket_pre_scan_does_not_parse_the_doubled_opener_as_html() {
    assert_eq!(
        render("[<<https://example.com/[label](inner)>>](outer)", enabled()),
        "<p>[&lt;&lt;<a href=\"https://example.com/\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.com/</a><a href=\"inner\">label</a>&gt;&gt;](outer)</p>\n"
    );
}

#[test]
fn mdx_looking_content_is_preserved_when_guillemet_syntax_is_enabled() {
    let options = ParserOptions {
        mdx: true,
        ..enabled()
    };
    assert_eq!(
        render("<<Foo />>", options),
        "<p>&lt;&lt;Foo /&gt;&gt;</p>\n"
    );
}

#[test]
fn unmatched_and_longer_angle_runs_remain_literal_text() {
    for source in [
        "<<Bonjour",
        "Bonjour>>",
        "<<<Bonjour>>>",
        "<<<<Bonjour>>>>",
        "a << b and x >> 2",
    ] {
        let html = render(source, enabled());
        assert!(
            html.contains("&lt;") || !source.contains('<'),
            "{source:?}: {html}"
        );
        assert!(!html.contains("<Bonjour"), "{source:?}: {html}");
        assert!(!html.contains("Bonjour></p>"), "{source:?}: {html}");
    }
    assert_eq!(
        render("a << b and x >> 2", enabled()),
        "<p>a &lt;&lt; b and x &gt;&gt; 2</p>\n"
    );
}

#[test]
fn escaped_digraphs_code_math_and_html_blocks_keep_their_source() {
    assert_eq!(
        render(r"\<\<Bonjour\>\>", enabled()),
        "<p>&lt;&lt;Bonjour&gt;&gt;</p>\n"
    );
    assert_eq!(
        render(
            "`<<Bonjour>>`\n\n```\n<<Bonjour>>\n```\n\n    <<Bonjour>>\n",
            enabled()
        ),
        "<p><code>&lt;&lt;Bonjour&gt;&gt;</code></p>\n<pre><code>&lt;&lt;Bonjour&gt;&gt;\n</code></pre>\n<pre><code>&lt;&lt;Bonjour&gt;&gt;\n</code></pre>\n"
    );
    assert_eq!(
        render("<div>\n<<Bonjour>>\n</div>\n", enabled()),
        "<div>\n<<Bonjour>>\n</div>\n"
    );

    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "$<<x>>$",
        ParserOptions {
            math: true,
            ..enabled()
        },
    )
    .parse()
    .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    assert!(matches!(&paragraph.children[0], Node::InlineMath(math) if math.value == "<<x>>"));
}

#[test]
fn a_line_start_blockquote_marker_is_still_parsed_at_block_level() {
    assert_eq!(
        render(">> nested quote\n", enabled()),
        "<blockquote>\n<blockquote>\n<p>nested quote</p>\n</blockquote>\n</blockquote>\n"
    );
}

#[test]
fn angle_guillemet_markers_keep_the_enclosed_url_as_text() {
    let allocator = Allocator::new();
    let source = "<<https://example.com>>";
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            autolinks: true,
            ..enabled()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm_spec()).render(&document);
    assert_eq!(html, "<p>&lt;&lt;https://example.com&gt;&gt;</p>\n");
}

struct TextValues(Vec<String>);

impl<'a> Visit<'a> for TextValues {
    fn visit_text(&mut self, text: &ferromark::ast::Text<'a>) {
        self.0.push(text.value.to_owned());
    }
}

#[test]
fn heading_text_is_preserved_for_metadata_consumers() {
    let allocator = Allocator::new();
    let source = "# <<Bonjour>>";
    let document = Parser::with_options(&allocator, source, enabled())
        .parse()
        .unwrap();
    let mut text = TextValues(Vec::new());
    text.visit_document(&document);
    assert_eq!(text.0.concat(), "<<Bonjour>>");
}
