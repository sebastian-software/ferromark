use ferromark::{Options, to_html_with_options};

fn math_html(input: &str) -> String {
    let mut options = Options::default();
    options.math = true;
    options.heading_ids = false;
    to_html_with_options(input, &options)
}

fn no_math_html(input: &str) -> String {
    let mut options = Options::default();
    options.math = false;
    options.heading_ids = false;
    to_html_with_options(input, &options)
}

#[test]
fn test_inline_math() {
    let html = math_html("$x^2$");
    assert_eq!(
        html,
        "<p><code class=\"language-math math-inline\">x^2</code></p>\n"
    );
}

#[test]
fn test_display_math() {
    let html = math_html("$$E=mc^2$$");
    assert_eq!(
        html,
        "<p><code class=\"language-math math-display\">E=mc^2</code></p>\n"
    );
}

#[test]
fn test_multiple_inline_math() {
    let html = math_html("$a$ and $b$");
    assert!(
        html.contains("<code class=\"language-math math-inline\">a</code>"),
        "Got: {html}"
    );
    assert!(
        html.contains("<code class=\"language-math math-inline\">b</code>"),
        "Got: {html}"
    );
}

#[test]
fn test_lone_dollar_is_literal() {
    let html = math_html("$ alone");
    assert!(!html.contains("<code"), "Got: {html}");
    assert!(html.contains("$ alone"), "Got: {html}");
}

#[test]
fn test_escaped_dollar() {
    let html = math_html("\\$escaped\\$");
    assert!(!html.contains("language-math"), "Got: {html}");
    assert!(html.contains("$escaped$"), "Got: {html}");
}

#[test]
fn test_math_strips_space() {
    let html = math_html("$ x $");
    assert_eq!(
        html,
        "<p><code class=\"language-math math-inline\">x</code></p>\n"
    );
}

#[test]
fn test_math_no_emphasis_inside() {
    let html = math_html("$*not emphasis*$");
    assert!(html.contains("*not emphasis*"), "Got: {html}");
    assert!(!html.contains("<em>"), "Got: {html}");
}

#[test]
fn test_math_disabled() {
    let html = no_math_html("$x^2$");
    assert!(!html.contains("language-math"), "Got: {html}");
    // $ should be literal text
    assert!(html.contains("$x^2$"), "Got: {html}");
}

#[test]
fn test_code_span_wins_over_math() {
    // Code spans have higher precedence
    let html = math_html("`$x^2$`");
    assert!(html.contains("<code>$x^2$</code>"), "Got: {html}");
    assert!(!html.contains("language-math"), "Got: {html}");
}

#[test]
fn test_display_math_with_newlines() {
    let html = math_html("$$\n\\sum\n$$");
    // Newlines in math are converted to spaces in output
    assert!(html.contains("language-math math-display"), "Got: {html}");
}

#[test]
fn test_math_in_paragraph() {
    let html = math_html("The equation $x = y$ is simple.");
    assert!(
        html.contains("<code class=\"language-math math-inline\">x = y</code>"),
        "Got: {html}"
    );
    assert!(html.contains("The equation "), "Got: {html}");
    assert!(html.contains(" is simple."), "Got: {html}");
}

#[test]
fn test_display_math_inline_context() {
    let html = math_html("See $$ax^2 + bx + c = 0$$ here.");
    assert!(html.contains("language-math math-display"), "Got: {html}");
    assert!(html.contains("ax^2 + bx + c = 0"), "Got: {html}");
}

#[test]
fn test_math_html_entities_escaped() {
    let html = math_html("$a < b$");
    assert!(html.contains("a &lt; b"), "Got: {html}");
}
