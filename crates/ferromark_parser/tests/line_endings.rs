use ferromark_allocator::Allocator;
use ferromark_ast::{Node, Span};
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::HtmlRenderer;

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .expect("fixture should parse");
    HtmlRenderer::new().render(&document)
}

#[test]
fn lone_cr_keeps_blank_list_siblings_separate() {
    assert_eq!(
        render("- a\r\r- b\r\r- c\r"),
        "<ul>\n<li><p>a</p>\n</li>\n<li><p>b</p>\n</li>\n<li><p>c</p>\n</li>\n</ul>\n"
    );
    assert_eq!(
        render("1. a\r\r  2. b\r\r   3. c\r"),
        "<ol>\n<li><p>a</p>\n</li>\n<li><p>b</p>\n</li>\n<li><p>c</p>\n</li>\n</ol>\n"
    );
    let comment_and_code = render("-   foo\r\r    notcode\r\r-   foo\r\r<!-- -->\r\r    code\r");
    assert!(comment_and_code.starts_with(
        "<ul>\n<li><p>foo</p>\n<p>notcode</p>\n</li>\n<li><p>foo</p>\n</li>\n</ul>\n"
    ));
    assert!(comment_and_code.contains("<!-- -->"));
    assert!(comment_and_code.contains("<pre><code>code\n</code></pre>\n"));
    assert_eq!(
        render("1. a\r\r  2. b\r\r    3. c\r"),
        "<ol>\n<li><p>a</p>\n</li>\n<li><p>b</p>\n</li>\n</ol>\n<pre><code>3. c\n</code></pre>\n"
    );
}

#[test]
fn lone_cr_does_not_duplicate_nested_list() {
    assert_eq!(
        render("- a\r  - b\r  - c\r\r- d\r  - e\r  - f\r"),
        "<ul>\n<li><p>a</p>\n<ul>\n<li>b</li>\n<li>c</li>\n</ul>\n</li>\n<li><p>d</p>\n<ul>\n<li>e</li>\n<li>f</li>\n</ul>\n</li>\n</ul>\n"
    );
}

#[test]
fn html_tag_line_endings_accept_cr_and_crlf() {
    assert_eq!(render("<a  /><b2\r\ndata=\"foo\" >\r\n"), "<p><a  /><b2\ndata=\"foo\" ></p>\n");
    assert_eq!(
        render("<a foo=\"bar\" bam = 'baz <em>\"</em>'\r_boolean zoop:33=zoop:33 />\r"),
        "<p><a foo=\"bar\" bam = 'baz <em>\"</em>'\n_boolean zoop:33=zoop:33 /></p>\n"
    );
    assert_eq!(
        render("< a><\rfoo><bar/ >\r<foo bar=baz\rbim!bop />\r"),
        "<p>&lt; a&gt;&lt;\nfoo&gt;&lt;bar/ &gt;\n&lt;foo bar=baz\nbim!bop /&gt;</p>\n"
    );
}

#[test]
fn normalized_inline_html_keeps_original_span() {
    let source = "x <b2\r\ndata=\"foo\">\r\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .expect("fixture should parse");
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph");
    };
    let Some(Node::Html(html)) =
        paragraph.children.iter().find(|node| matches!(node, Node::Html(_)))
    else {
        panic!("expected inline HTML");
    };
    assert_eq!(html.value, "<b2\ndata=\"foo\">");
    assert_eq!(html.span, Span::new(2, source.len() as u32 - 2));
}

#[test]
fn angle_link_destination_rejects_cr_line_endings() {
    assert_eq!(render("[link](<foo\rbar>)\r"), "<p>[link](<foo\nbar>)</p>\n");
    assert_eq!(render("[link](<foo\r\nbar>)\r\n"), "<p>[link](<foo\nbar>)</p>\n");
}
