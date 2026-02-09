use ferromark::{Options, to_html, to_html_with_options};

// GFM extension: Disallowed raw HTML
// https://github.github.com/gfm/#disallowed-raw-html-extension-

// Inline HTML filtering

#[test]
fn inline_script_tag_filtered() {
    let result = to_html("foo <script>alert('xss')</script> bar");
    assert_eq!(
        result,
        "<p>foo &lt;script>alert('xss')&lt;/script> bar</p>\n"
    );
}

#[test]
fn inline_title_tag_filtered() {
    let result = to_html("foo <title>bad</title> bar");
    assert_eq!(result, "<p>foo &lt;title>bad&lt;/title> bar</p>\n");
}

#[test]
fn inline_textarea_tag_filtered() {
    let result = to_html("foo <textarea>bad</textarea> bar");
    assert_eq!(result, "<p>foo &lt;textarea>bad&lt;/textarea> bar</p>\n");
}

#[test]
fn inline_style_tag_filtered() {
    let result = to_html("foo <style>body{}</style> bar");
    assert_eq!(result, "<p>foo &lt;style>body{}&lt;/style> bar</p>\n");
}

#[test]
fn inline_xmp_tag_filtered() {
    let result = to_html("foo <xmp>bad</xmp> bar");
    assert_eq!(result, "<p>foo &lt;xmp>bad&lt;/xmp> bar</p>\n");
}

#[test]
fn inline_iframe_tag_filtered() {
    let result = to_html("foo <iframe src=\"x\"> bar");
    assert_eq!(result, "<p>foo &lt;iframe src=\"x\"> bar</p>\n");
}

#[test]
fn inline_noembed_tag_filtered() {
    let result = to_html("foo <noembed>bad</noembed> bar");
    assert_eq!(result, "<p>foo &lt;noembed>bad&lt;/noembed> bar</p>\n");
}

#[test]
fn inline_noframes_tag_filtered() {
    let result = to_html("foo <noframes>bad</noframes> bar");
    assert_eq!(result, "<p>foo &lt;noframes>bad&lt;/noframes> bar</p>\n");
}

#[test]
fn inline_plaintext_tag_filtered() {
    let result = to_html("foo <plaintext> bar");
    assert_eq!(result, "<p>foo &lt;plaintext> bar</p>\n");
}

// Case insensitive

#[test]
fn case_insensitive_filtering() {
    let result = to_html("<SCRIPT>bad</SCRIPT>");
    assert_eq!(result, "&lt;SCRIPT>bad&lt;/SCRIPT>");
}

#[test]
fn mixed_case_filtering() {
    let result = to_html("<Script>bad</Script>");
    assert_eq!(result, "&lt;Script>bad&lt;/Script>");
}

// Allowed tags pass through

#[test]
fn allowed_tags_not_filtered() {
    let result = to_html("<strong>ok</strong>");
    assert_eq!(result, "<p><strong>ok</strong></p>\n");
}

#[test]
fn div_not_filtered() {
    // <div> is block-level HTML, not disallowed
    let result = to_html("<div>ok</div>");
    assert_eq!(result, "<div>ok</div>");
}

// Block HTML filtering

#[test]
fn block_script_filtered() {
    let result = to_html("<script>\nalert('xss')\n</script>");
    assert_eq!(result, "&lt;script>\nalert('xss')\n&lt;/script>");
}

#[test]
fn block_style_filtered() {
    let result = to_html("<style>\nbody { color: red; }\n</style>");
    assert_eq!(result, "&lt;style>\nbody { color: red; }\n&lt;/style>");
}

// Self-closing disallowed tags

#[test]
fn self_closing_disallowed_tag() {
    let result = to_html("foo <script/> bar");
    assert_eq!(result, "<p>foo &lt;script/> bar</p>\n");
}

// Disabled via options

#[test]
fn disallowed_html_disabled() {
    let result = to_html_with_options(
        "foo <script>alert('xss')</script> bar",
        &Options {
            disallowed_raw_html: false,
            ..Options::default()
        },
    );
    assert_eq!(result, "<p>foo <script>alert('xss')</script> bar</p>\n");
}

// Non-tag uses of < are not affected

#[test]
fn less_than_not_affected() {
    let result = to_html("3 < 5 is true");
    assert_eq!(result, "<p>3 &lt; 5 is true</p>\n");
}

// Tag name must match exactly (no partial matches)

#[test]
fn partial_tag_name_not_filtered() {
    // "scripting" is not "script"
    let result = to_html("<scripting>ok</scripting>");
    assert_eq!(result, "<p><scripting>ok</scripting></p>\n");
}
