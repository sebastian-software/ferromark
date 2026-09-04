use ferromark::{Options, to_html_with_options};

fn autolink_html(input: &str) -> String {
    let options = ferromark::options!(Options::default();
        autolink_literals: true,);
    to_html_with_options(input, &options)
}

#[test]
fn url_autolinks() {
    let input = ": http://google.com https://google.com";
    let expected = "<p>: <a href=\"http://google.com\">http://google.com</a> <a href=\"https://google.com\">https://google.com</a></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn email_autolink() {
    let input = "scyther@pokemon.com";
    let expected = "<p><a href=\"mailto:scyther@pokemon.com\">scyther@pokemon.com</a></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn email_with_special_chars() {
    let input = "scy.the_rbe-edr+ill@pokemon.com";
    let expected = "<p><a href=\"mailto:scy.the_rbe-edr+ill@pokemon.com\">scy.the_rbe-edr+ill@pokemon.com</a></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn email_trailing_dot() {
    let input = "scyther@pokemon.com.";
    let expected = "<p><a href=\"mailto:scyther@pokemon.com\">scyther@pokemon.com</a>.</p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn email_trailing_slash() {
    let input = "scyther@pokemon.com/";
    let expected = "<p><a href=\"mailto:scyther@pokemon.com\">scyther@pokemon.com</a>/</p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn email_slash_separated() {
    let input = "scyther@pokemon.com/beedrill@pokemon.com";
    let expected = "<p><a href=\"mailto:scyther@pokemon.com\">scyther@pokemon.com</a>/<a href=\"mailto:beedrill@pokemon.com\">beedrill@pokemon.com</a></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn www_autolink() {
    let input = "www.github.com";
    let expected = "<p><a href=\"http://www.github.com\">www.github.com</a></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn www_underscore_in_last_segment() {
    let input = "Underscores not allowed in host name www.xxx.yyy._zzz";
    let expected = "<p>Underscores not allowed in host name www.xxx.yyy._zzz</p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn www_underscore_in_second_to_last() {
    let input = "Underscores not allowed in host name www.xxx._yyy.zzz";
    let expected = "<p>Underscores not allowed in host name www.xxx._yyy.zzz</p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn autolink_with_emphasis() {
    let input = "**Autolink and http://inlines**";
    let expected =
        "<p><strong>Autolink and <a href=\"http://inlines\">http://inlines</a></strong></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn url_trailing_dot() {
    let input = "Full stop outside parens shouldn't be included http://google.com/ok.";
    let expected = "<p>Full stop outside parens shouldn't be included <a href=\"http://google.com/ok\">http://google.com/ok</a>.</p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn short_email() {
    let input = "a.w@b.c";
    let expected = "<p><a href=\"mailto:a.w@b.c\">a.w@b.c</a></p>\n";
    assert_eq!(autolink_html(input), expected);
}

#[test]
fn no_autolink_for_at_only() {
    let input = "These should not link:\n\n* @a.b.c@. x\n* n@.  b";
    let result = autolink_html(input);
    assert!(
        !result.contains("<a"),
        "Expected no autolinks in output, got: {}",
        result
    );
}

#[test]
fn autolink_disabled() {
    let input = "http://google.com";
    let options = ferromark::options!(Options::default();
        autolink_literals: false,);
    let result = to_html_with_options(input, &options);
    assert!(
        !result.contains("<a"),
        "Expected no autolink when autolink_literals is disabled, got: {}",
        result
    );
}

#[test]
fn many_unmatched_trailing_parentheses_stay_outside_the_link() {
    let suffix = ")".repeat(32_768);
    let input = format!("https://example.com/path{suffix}");
    assert_eq!(
        autolink_html(&input),
        format!(
            "<p><a href=\"https://example.com/path\">https://example.com/path</a>{suffix}</p>\n"
        )
    );
}

#[test]
fn trailing_parentheses_preserve_balance_across_punctuation_and_entities() {
    for (tail, linked, rest) in [
        ("(x))", "(x)", ")"),
        ("(x)).", "(x)", ")."),
        ("(x))&amp;)", "(x)", ")&amp;)"),
        ("((x))", "((x))", ""),
        ("((x)", "((x)", ""),
        (")(x))", ")(x", "))"),
    ] {
        let url = format!("https://example.com/{linked}");
        assert_eq!(
            autolink_html(&format!("https://example.com/{tail}")),
            format!("<p><a href=\"{url}\">{url}</a>{rest}</p>\n"),
            "tail={tail:?}"
        );
    }
}

#[test]
fn repeated_angle_autolinks_keep_gfm_output_flat() {
    let input = "<https://example.com> ".repeat(8192);
    let html = to_html_with_options(&input, &Options::gfm());

    assert_eq!(html.matches("<a href=").count(), 8192);
    assert!(!html.contains("<a href=\"https://example.com\"><a href="));
}
