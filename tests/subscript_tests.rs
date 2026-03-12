use ferromark::{Options, to_html_with_options};

fn subscript_html(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            subscript: true,
            heading_ids: false,
            ..Options::default()
        },
    )
}

fn no_subscript_html(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            subscript: false,
            heading_ids: false,
            ..Options::default()
        },
    )
}

#[test]
fn basic_subscript() {
    assert_eq!(subscript_html("H~2~O"), "<p>H<sub>2</sub>O</p>\n");
}

#[test]
fn subscript_in_paragraph() {
    assert_eq!(
        subscript_html("before ~sub~ after"),
        "<p>before <sub>sub</sub> after</p>\n"
    );
}

#[test]
fn subscript_with_emphasis_inside() {
    assert_eq!(
        subscript_html("~**bold**~"),
        "<p><sub><strong>bold</strong></sub></p>\n"
    );
}

#[test]
fn subscript_disabled() {
    assert_eq!(no_subscript_html("H~2~O"), "<p>H~2~O</p>\n");
}

#[test]
fn subscript_does_not_parse_as_strikethrough() {
    assert_eq!(no_subscript_html("~sub~"), "<p>~sub~</p>\n");
}

#[test]
fn unmatched_subscript_is_literal() {
    assert_eq!(subscript_html("~sub"), "<p>~sub</p>\n");
    assert_eq!(subscript_html("sub~"), "<p>sub~</p>\n");
}

#[test]
fn empty_subscript_is_literal() {
    assert_eq!(subscript_html("~~"), "<p>~~</p>\n");
}

#[test]
fn whitespace_only_subscript_is_literal() {
    assert_eq!(subscript_html("~  ~"), "<p>~  ~</p>\n");
}

#[test]
fn code_span_wins_over_subscript() {
    assert_eq!(subscript_html("`~x~`"), "<p><code>~x~</code></p>\n");
}

#[test]
fn links_work_inside_subscript() {
    assert_eq!(
        subscript_html("~[link](https://example.com)~"),
        "<p><sub><a href=\"https://example.com\">link</a></sub></p>\n"
    );
}

#[test]
fn subscript_works_inside_link_text() {
    assert_eq!(
        subscript_html("[~x~](https://example.com)"),
        "<p><a href=\"https://example.com\"><sub>x</sub></a></p>\n"
    );
}

#[test]
fn subscript_does_not_parse_in_link_destinations() {
    assert_eq!(
        subscript_html("[link](https://example.com/~x~)"),
        "<p><a href=\"https://example.com/~x~\">link</a></p>\n"
    );
}
