use ferromark::{to_html, to_html_with_options, Options};

// cmark-gfm extension spec tests

#[test]
fn single_tilde_strikethrough() {
    let result = to_html("A proper ~strikethrough~.");
    assert_eq!(result, "<p>A proper <del>strikethrough</del>.</p>\n");
}

#[test]
fn unmatched_single_tilde_opening() {
    let result = to_html("These are ~not strikethroughs.");
    assert_eq!(result, "<p>These are ~not strikethroughs.</p>\n");
}

#[test]
fn unmatched_single_tilde_closing() {
    let result = to_html("No, they are not~");
    assert_eq!(result, "<p>No, they are not~</p>\n");
}

#[test]
fn nested_tilde_in_strikethrough() {
    let result = to_html("This ~is ~ legit~ isn't ~ legit.");
    assert_eq!(
        result,
        "<p>This <del>is ~ legit</del> isn't ~ legit.</p>\n"
    );
}

#[test]
fn five_tildes_not_strikethrough() {
    let result = to_html("This is not ~~~~~one~~~~~ huge strikethrough.");
    assert_eq!(
        result,
        "<p>This is not ~~~~~one~~~~~ huge strikethrough.</p>\n"
    );
}

#[test]
fn one_and_two_tildes() {
    let result = to_html("~one~ ~~two~~ ~~~three~~~");
    assert_eq!(result, "<p><del>one</del> <del>two</del> ~~~three~~~</p>\n");
}

#[test]
fn mismatched_tilde_runs() {
    let result = to_html("No ~mismatch~~");
    assert_eq!(result, "<p>No ~mismatch~~</p>\n");
}

// Edge-case tests

#[test]
fn strikethrough_with_emphasis() {
    let result = to_html("~~**bold**~~");
    assert_eq!(result, "<p><del><strong>bold</strong></del></p>\n");
}

#[test]
fn strikethrough_in_paragraph() {
    let result = to_html("before ~~deleted~~ after");
    assert_eq!(result, "<p>before <del>deleted</del> after</p>\n");
}

#[test]
fn empty_strikethrough() {
    // ~~~~ is a fenced code block delimiter in CommonMark, not inline
    // Test with tildes that aren't code fences: inside a paragraph
    let result = to_html("a~~~~b");
    // 4 tildes can't form strikethrough (only 1 or 2 match)
    assert!(result.contains("~~~~"), "Expected literal tildes: {result}");
}

#[test]
fn strikethrough_across_words() {
    let result = to_html("~~foo bar~~");
    assert_eq!(result, "<p><del>foo bar</del></p>\n");
}

#[test]
fn double_tilde_strikethrough() {
    let result = to_html("a ~~hi~~ b");
    assert_eq!(result, "<p>a <del>hi</del> b</p>\n");
}

#[test]
fn strikethrough_disabled() {
    let result = to_html_with_options(
        "~~test~~",
        &Options {
            strikethrough: false,
            ..Options::default()
        },
    );
    assert_eq!(result, "<p>~~test~~</p>\n");
}
