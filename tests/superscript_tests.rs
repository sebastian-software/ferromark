use ferromark::{Options, to_html_with_options};

fn superscript_html(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            superscript: true,
            heading_ids: false,
            ..Options::default()
        },
    )
}

fn no_superscript_html(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            superscript: false,
            heading_ids: false,
            ..Options::default()
        },
    )
}

#[test]
fn basic_superscript() {
    assert_eq!(superscript_html("2^10^"), "<p>2<sup>10</sup></p>\n");
}

#[test]
fn superscript_in_paragraph() {
    assert_eq!(
        superscript_html("before ^sup^ after"),
        "<p>before <sup>sup</sup> after</p>\n"
    );
}

#[test]
fn superscript_with_emphasis_inside() {
    assert_eq!(
        superscript_html("^**bold**^"),
        "<p><sup><strong>bold</strong></sup></p>\n"
    );
}

#[test]
fn superscript_disabled() {
    assert_eq!(no_superscript_html("2^10^"), "<p>2^10^</p>\n");
}

#[test]
fn unmatched_superscript_is_literal() {
    assert_eq!(superscript_html("^sup"), "<p>^sup</p>\n");
    assert_eq!(superscript_html("sup^"), "<p>sup^</p>\n");
}

#[test]
fn empty_superscript_is_literal() {
    assert_eq!(superscript_html("^^"), "<p>^^</p>\n");
}

#[test]
fn whitespace_only_superscript_is_literal() {
    assert_eq!(superscript_html("^  ^"), "<p>^  ^</p>\n");
}

#[test]
fn code_span_wins_over_superscript() {
    assert_eq!(superscript_html("`^x^`"), "<p><code>^x^</code></p>\n");
}

#[test]
fn links_work_inside_superscript() {
    assert_eq!(
        superscript_html("^[link](https://example.com)^"),
        "<p><sup><a href=\"https://example.com\">link</a></sup></p>\n"
    );
}

#[test]
fn superscript_works_inside_link_text() {
    assert_eq!(
        superscript_html("[^x^](https://example.com)"),
        "<p><a href=\"https://example.com\"><sup>x</sup></a></p>\n"
    );
}

#[test]
fn superscript_does_not_parse_in_link_destinations() {
    assert_eq!(
        superscript_html("[link](https://example.com/^x^)"),
        "<p><a href=\"https://example.com/^x^\">link</a></p>\n"
    );
}
