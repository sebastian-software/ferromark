use ferromark::{to_html_with_options, Options};

fn autolink_html(input: &str) -> String {
    let options = Options {
        autolink_literals: true,
        ..Options::default()
    };
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
    let expected = "<p><strong>Autolink and <a href=\"http://inlines\">http://inlines</a></strong></p>\n";
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
    let options = Options {
        autolink_literals: false,
        ..Options::default()
    };
    let result = to_html_with_options(input, &options);
    assert!(
        !result.contains("<a"),
        "Expected no autolink when autolink_literals is disabled, got: {}",
        result
    );
}
