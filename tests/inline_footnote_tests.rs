use ferromark::{InlineEvent, InlineParser, Options, to_html_with_options};

fn options() -> Options {
    ferromark::options!(Options::default();
        inline_footnotes: true,)
}

fn render(input: &str) -> String {
    to_html_with_options(input, &options())
}

#[test]
fn inline_footnotes_are_disabled_by_default() {
    assert_eq!(
        to_html_with_options("Text.^[A note.]", &Options::default()),
        "<p>Text.^[A note.]</p>\n"
    );
}

#[test]
fn basic_inline_footnote_renders_at_document_end() {
    assert_eq!(
        render("Text.^[A note.]"),
        "<p>Text.<sup><a href=\"#user-content-inline-fn-1\" id=\"user-content-inline-fnref-1\" data-footnote-ref>1</a></sup></p>\n<section data-footnotes class=\"footnotes\">\n<ol>\n<li id=\"user-content-inline-fn-1\">\n<p>A note. <a href=\"#user-content-inline-fnref-1\" class=\"data-footnote-backref\" aria-label=\"Back to reference 1\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    );
}

#[test]
fn inline_note_content_supports_inline_markdown_and_balanced_brackets() {
    let html = render("Text.^[A *formatted* [link](https://example.com).]");

    assert!(html.contains("<em>formatted</em>"), "{html}");
    assert!(
        html.contains("<a href=\"https://example.com\">link</a>"),
        "{html}"
    );
    assert_eq!(html.matches("data-footnote-ref>").count(), 1);
}

#[test]
fn inline_and_reference_footnotes_share_first_appearance_numbering() {
    let input = "\
Inline.^[First.]

Reference.[^ref]

[^ref]: Second.";
    let html = to_html_with_options(
        input,
        &ferromark::options!(Options::default();
            footnotes: true,
            inline_footnotes: true,),
    );

    assert!(html.contains("data-footnote-ref>1</a></sup>"), "{html}");
    assert!(html.contains("data-footnote-ref>2</a></sup>"), "{html}");
    let inline_pos = html.find("id=\"user-content-inline-fn-1\"").unwrap();
    let reference_pos = html.find("id=\"user-content-fn-ref\"").unwrap();
    assert!(inline_pos < reference_pos, "{html}");
}

#[test]
fn inline_notes_take_precedence_over_superscript() {
    let html = to_html_with_options(
        "2^10^ and a note.^[Content.]",
        &ferromark::options!(Options::default();
            superscript: true,
            inline_footnotes: true,),
    );

    assert!(html.contains("2<sup>10</sup>"), "{html}");
    assert_eq!(html.matches("data-footnote-ref>").count(), 1);
}

#[test]
fn whitespace_only_notes_do_not_capture_later_superscript_delimiters() {
    let html = to_html_with_options(
        "some text^[  ]^word^",
        &ferromark::options!(Options::default();
            superscript: true,
            inline_footnotes: true,),
    );

    assert_eq!(html, "<p>some text^[  ]<sup>word</sup></p>\n");
    assert!(!html.contains("data-footnote-ref>"), "{html}");
}

#[test]
fn escaped_empty_unclosed_and_code_syntax_stays_literal() {
    let html = render(r"\^[escaped] ^[] ^[   ] ^[unclosed `^[code]`");

    assert!(!html.contains("data-footnote-ref>"), "{html}");
    assert!(html.contains("^[escaped]"), "{html}");
    assert!(html.contains("^[]"), "{html}");
    assert!(html.contains("^[unclosed"), "{html}");
    assert!(html.contains("<code>^[code]</code>"), "{html}");
}

#[test]
fn multiline_inline_notes_remain_single_paragraph_notes() {
    let html = render("Text.^[First line\nsecond line.]");

    assert!(html.contains("<p>First line\nsecond line."), "{html}");
    assert_eq!(html.matches("<li id=\"user-content-inline-fn-").count(), 1);
}

#[test]
fn parser_exposes_the_note_content_range() {
    let input = b"Text.^[A *note*.]";
    let mut parser = InlineParser::new();
    let mut events = Vec::new();
    parser.parse_with_options(
        input,
        None,
        true,
        true,
        false,
        false,
        false,
        false,
        false,
        true,
        None,
        &mut events,
    );

    let range = events
        .iter()
        .find_map(|event| match event {
            InlineEvent::InlineFootnote(range) => Some(*range),
            _ => None,
        })
        .expect("inline footnote event");
    assert_eq!(range.slice(input), b"A *note*.");
}

#[test]
fn malformed_candidate_runs_remain_bounded_and_literal() {
    let input = "^[".repeat(10_000);
    let html = render(&input);

    assert!(!html.contains("data-footnote-ref>"), "{html}");
    assert!(html.starts_with("<p>^[^[^["));
}
