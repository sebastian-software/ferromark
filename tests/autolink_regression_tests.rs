use ferromark::{Options, to_html_with_options};

#[test]
fn angle_bracket_terminates_an_invalid_uri_before_the_next_autolink() {
    assert_eq!(
        to_html_with_options("<https://outer<https://inner>", &Options::commonmark()),
        "<p>&lt;https://outer<a href=\"https://inner\">https://inner</a></p>\n"
    );
}

#[test]
fn ascii_controls_cannot_be_part_of_a_uri_autolink() {
    for control in ['\t', '\r', '\u{000b}', '\u{001f}', '\u{007f}'] {
        let source = format!("a <https://example.com/{control}path>");
        let html = to_html_with_options(&source, &Options::commonmark());
        assert!(
            !html.contains("<a href="),
            "control={control:?}, html={html:?}"
        );
    }
}

#[test]
fn many_angle_starts_preserve_literal_text_and_a_later_valid_autolink() {
    let prefix = "<".repeat(8192);
    assert_eq!(
        to_html_with_options(
            &format!("a {prefix}<https://example.com>"),
            &Options::commonmark()
        ),
        format!(
            "<p>a {}<a href=\"https://example.com\">https://example.com</a></p>\n",
            "&lt;".repeat(8192)
        )
    );
}
