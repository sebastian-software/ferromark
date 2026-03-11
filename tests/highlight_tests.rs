use ferromark::{Options, to_html_with_options};

fn highlight_html(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            highlight: true,
            heading_ids: false,
            ..Options::default()
        },
    )
}

fn no_highlight_html(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            highlight: false,
            heading_ids: false,
            ..Options::default()
        },
    )
}

#[test]
fn basic_highlight() {
    assert_eq!(highlight_html("==marked=="), "<p><mark>marked</mark></p>\n");
}

#[test]
fn highlight_in_paragraph() {
    assert_eq!(
        highlight_html("before ==marked== after"),
        "<p>before <mark>marked</mark> after</p>\n"
    );
}

#[test]
fn highlight_with_emphasis_inside() {
    assert_eq!(
        highlight_html("==**bold**=="),
        "<p><mark><strong>bold</strong></mark></p>\n"
    );
}

#[test]
fn highlight_disabled() {
    assert_eq!(no_highlight_html("==marked=="), "<p>==marked==</p>\n");
}

#[test]
fn unmatched_highlight_is_literal() {
    assert_eq!(highlight_html("==marked"), "<p>==marked</p>\n");
    assert_eq!(highlight_html("marked=="), "<p>marked==</p>\n");
}

#[test]
fn empty_highlight_is_literal() {
    assert_eq!(highlight_html("===="), "<p>====</p>\n");
}

#[test]
fn whitespace_only_highlight_is_literal() {
    assert_eq!(highlight_html("==  =="), "<p>==  ==</p>\n");
}

#[test]
fn code_span_wins_over_highlight() {
    assert_eq!(
        highlight_html("`==code==`"),
        "<p><code>==code==</code></p>\n"
    );
}

#[test]
fn escaped_highlight_is_literal() {
    assert_eq!(highlight_html("\\==text=="), "<p>==text==</p>\n");
}

#[test]
fn links_work_inside_highlight() {
    assert_eq!(
        highlight_html("==[link](https://example.com)=="),
        "<p><mark><a href=\"https://example.com\">link</a></mark></p>\n"
    );
}

#[test]
fn highlight_works_inside_link_text() {
    assert_eq!(
        highlight_html("[==text==](https://example.com)"),
        "<p><a href=\"https://example.com\"><mark>text</mark></a></p>\n"
    );
}

#[test]
fn highlight_does_not_parse_in_link_destinations() {
    assert_eq!(
        highlight_html("[link](https://example.com/?a==b==c)"),
        "<p><a href=\"https://example.com/?a==b==c\">link</a></p>\n"
    );
    assert_eq!(
        highlight_html("[==text==](https://example.com/?a==b==c==d)"),
        "<p><a href=\"https://example.com/?a==b==c==d\"><mark>text</mark></a></p>\n"
    );
}

#[test]
fn long_equal_runs_are_literal() {
    assert_eq!(
        highlight_html("===not highlighted==="),
        "<p>===not highlighted===</p>\n"
    );
}
