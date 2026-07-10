use ferromark::{Options, to_html, to_html_with_options};

fn without_indented_code(input: &str) -> String {
    to_html_with_options(
        input,
        &Options {
            indented_code_blocks: false,
            ..Options::default()
        },
    )
}

#[test]
fn indented_code_blocks_remain_enabled_by_default() {
    assert_eq!(
        to_html("    fn main() {}\n"),
        "<pre><code>fn main() {}\n</code></pre>\n"
    );
}

#[test]
fn top_level_indented_code_falls_back_to_paragraph_when_disabled() {
    assert_eq!(
        without_indented_code("    fn main() {}\n"),
        "<p>fn main() {}</p>\n"
    );
}

#[test]
fn fenced_code_blocks_still_work_when_indented_code_is_disabled() {
    assert_eq!(
        without_indented_code("```rust\nfn main() {}\n```\n"),
        "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>\n"
    );
}

#[test]
fn nested_list_indented_code_falls_back_to_paragraph_when_disabled() {
    assert_eq!(
        without_indented_code("- item\n\n        nested\n"),
        "<ul>\n<li>\n<p>item</p>\n<p>nested</p>\n</li>\n</ul>\n"
    );
}

#[test]
fn blockquote_indented_code_falls_back_to_paragraph_when_disabled() {
    assert_eq!(
        without_indented_code("> quote\n>\n>     nested\n"),
        "<blockquote>\n<p>quote</p>\n<p>nested</p>\n</blockquote>\n"
    );
}

#[test]
fn footnote_indented_code_falls_back_to_paragraph_when_disabled() {
    let result = to_html_with_options(
        "[^1]: note\n\n        nested\n\nText[^1].\n",
        &Options {
            footnotes: true,
            indented_code_blocks: false,
            ..Options::default()
        },
    );

    assert!(result.contains("<p>note</p>"));
    assert!(result.contains("<p>nested <a href=\"#user-content-fnref-1\""));
    assert!(!result.contains("<pre><code>nested"));
}
