//! Block boundaries trim ASCII whitespace only.
//!
//! CommonMark removes "initial and final spaces or tabs" from paragraph and
//! heading content, calls a line of spaces or tabs blank, and GFM trims the
//! spaces around table cells. Non-ASCII whitespace such as the no-break space
//! is content in all of these places, as it is in cmark. See
//! `docs/decisions/2026-09-23-commonmark-whitespace-trim.md`.

use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit, walk_node};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

const NBSP: &str = "\u{a0}";
const EM_SPACE: &str = "\u{2003}";
const IDEOGRAPHIC_SPACE: &str = "\u{3000}";

fn render_with(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions {
        heading_ids: false,
        ..HtmlRendererOptions::new()
    })
    .render(&document)
}

fn render(source: &str) -> String {
    render_with(source, ParserOptions::default())
}

fn render_gfm(source: &str) -> String {
    render_with(source, ParserOptions::gfm())
}

/// Every node as `kind [start, end]`, indented by depth.
fn spans(source: &str) -> Vec<String> {
    struct Spans(Vec<String>, usize);
    impl<'a> Visit<'a> for Spans {
        fn visit_node(&mut self, node: &Node<'a>) {
            let span = node.span();
            let kind = format!("{node:?}");
            let kind: String = kind
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();
            self.0.push(format!(
                "{}{kind} [{}, {}]",
                "  ".repeat(self.1),
                span.start,
                span.end
            ));
            self.1 += 1;
            walk_node(self, node);
            self.1 -= 1;
        }
    }

    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let mut visitor = Spans(Vec::new(), 0);
    for child in &document.children {
        visitor.visit_node(child);
    }
    visitor.0
}

#[test]
fn paragraphs_keep_non_ascii_whitespace_at_both_ends() {
    for space in [NBSP, EM_SPACE, IDEOGRAPHIC_SPACE] {
        assert_eq!(render(&format!("a{space}\n")), format!("<p>a{space}</p>\n"));
        assert_eq!(render(&format!("{space}a\n")), format!("<p>{space}a</p>\n"));
        assert_eq!(
            render(&format!("{space}a{space}\n")),
            format!("<p>{space}a{space}</p>\n")
        );
        // A line holding nothing else is a paragraph, not a blank line.
        assert_eq!(render(&format!("{space}\n")), format!("<p>{space}</p>\n"));
    }
    // ASCII spaces and tabs are still trimmed.
    assert_eq!(render("  a \t\n"), "<p>a</p>\n");
}

#[test]
fn setext_headings_trim_like_atx_headings() {
    assert_eq!(
        render(&format!("h{NBSP}\n---\n")),
        format!("<h2>h{NBSP}</h2>\n")
    );
    assert_eq!(
        render(&format!("h{NBSP}\n===\n")),
        format!("<h1>h{NBSP}</h1>\n")
    );
    assert_eq!(
        render(&format!("# h{NBSP}\n")),
        format!("<h1>h{NBSP}</h1>\n")
    );
}

#[test]
fn table_cells_keep_non_ascii_padding() {
    let html = render_gfm(&format!("| a |\n| - |\n| {NBSP}y{NBSP} |\n"));
    assert!(html.contains(&format!("<td>{NBSP}y{NBSP}</td>")), "{html}");

    let html = render_gfm(&format!("| a |\n| - |\n|{NBSP}|\n"));
    assert!(html.contains(&format!("<td>{NBSP}</td>")), "{html}");
}

#[test]
fn a_delimiter_row_holds_only_ascii_padding() {
    // GFM delimiter cells are hyphens with optional colons and spaces; a
    // no-break space makes the row ordinary paragraph text.
    let source = format!("| a |\n|{NBSP}- |\n");
    assert_eq!(render_gfm(&source), format!("<p>| a |\n|{NBSP}- |</p>\n"));
}

#[test]
fn list_item_content_keeps_a_leading_no_break_space() {
    assert_eq!(
        render(&format!("- {NBSP}item\n")),
        format!("<ul>\n<li>{NBSP}item</li>\n</ul>\n")
    );
    // Such an item does not start with a blank line, so it may interrupt a
    // paragraph.
    assert_eq!(
        render(&format!("a\n- {NBSP}\n")),
        format!("<p>a</p>\n<ul>\n<li>{NBSP}</li>\n</ul>\n")
    );
}

#[test]
fn a_no_break_space_line_inside_a_list_is_not_blank() {
    // A lazy continuation line: the list stays tight.
    assert_eq!(
        render(&format!("- a\n{NBSP}\n- b\n")),
        format!("<ul>\n<li>a\n{NBSP}</li>\n<li>b</li>\n</ul>\n")
    );
    // A line of ASCII spaces is blank and loosens the list.
    let loose = render("- a\n \n- b\n");
    assert!(loose.contains("<p>a</p>"), "{loose}");
}

#[test]
fn a_thematic_break_ends_in_spaces_or_tabs_only() {
    assert_eq!(render("*** \t\n"), "<hr>\n");
    assert_eq!(
        render(&format!("***{NBSP}\n")),
        format!("<p>***{NBSP}</p>\n")
    );
}

#[test]
fn inline_spans_start_at_the_trimmed_paragraph_content() {
    assert_eq!(
        spans(&format!("{NBSP}*a*")),
        [
            "Paragraph [0, 5]",
            "  Text [0, 2]",
            "  Emphasis [2, 5]",
            "    Text [3, 4]"
        ]
    );
    // Indentation is trimmed, and the inline offset now skips it.
    assert_eq!(
        spans("   *a*"),
        ["Paragraph [0, 6]", "  Emphasis [3, 6]", "    Text [4, 5]"]
    );
    assert_eq!(
        spans("  *a*\n---\n"),
        ["Heading [0, 10]", "  Emphasis [2, 5]", "    Text [3, 4]"]
    );
    // A vertical tab is still trimmed, now without shifting the text.
    assert_eq!(spans("\u{b}a\n"), ["Paragraph [0, 3]", "  Text [1, 2]"]);
}

#[test]
fn tab_columns_after_a_nested_list_marker_do_not_shift_inline_spans() {
    // Inside a container, a tab after a list marker becomes spaces that have
    // no source bytes. Only the real bytes move the inline offset, on the
    // one-line fast path and on the sub-parser path alike.
    assert_eq!(
        spans("> -\t*foo*"),
        [
            "BlockQuote [0, 9]",
            "  List [2, 9]",
            "    Paragraph [4, 9]",
            "      Emphasis [4, 9]",
            "        Text [5, 8]"
        ]
    );
    assert_eq!(
        spans("-\ta\n\t-\t*b*"),
        [
            "List [0, 10]",
            "  Paragraph [2, 4]",
            "    Text [2, 3]",
            "  List [4, 10]",
            "    Paragraph [7, 10]",
            "      Emphasis [7, 10]",
            "        Text [8, 9]"
        ]
    );
    assert_eq!(
        spans("> -\tfoo\n>   bar"),
        [
            "BlockQuote [0, 15]",
            "  List [2, 15]",
            "    Paragraph [4, 15]",
            "      Text [4, 15]"
        ]
    );
    // A space before the tab is a real byte inside the expanded run.
    assert_eq!(spans("> - \tfoo")[3], "      Text [5, 8]");
}
