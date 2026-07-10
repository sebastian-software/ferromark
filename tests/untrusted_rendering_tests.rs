use ferromark::{Options, RenderPolicy, to_html, to_html_with_options};

#[test]
fn default_policy_escapes_inline_and_block_html() {
    assert_eq!(
        to_html("<img src=x onerror=alert(1)>"),
        "&lt;img src=x onerror=alert(1)&gt;"
    );
    assert_eq!(
        to_html("Text <strong>bold</strong>"),
        "<p>Text &lt;strong&gt;bold&lt;/strong&gt;</p>\n"
    );
}

#[test]
fn default_policy_blocks_dangerous_link_and_image_schemes() {
    for markdown in [
        "[click](javascript:alert(1))",
        "[click](JaVaScRiPt:alert(1))",
        "[click](java&#9;script:alert(1))",
        "[click](javas&#99;ript:alert(1))",
        "[click](averylongunknownscheme:payload)",
    ] {
        let html = to_html(markdown);
        assert!(html.contains("href=\"\""), "unsafe link survived: {html}");
        assert!(!html.to_ascii_lowercase().contains("javascript:"));
    }

    let html = to_html("![pixel](data:text/html,<script>alert(1)</script>)");
    assert!(html.contains("src=\"\""), "unsafe image survived: {html}");

    let html = to_html("<javascript:alert(1)>");
    assert!(
        html.contains("href=\"\""),
        "unsafe autolink survived: {html}"
    );
}

#[test]
fn default_policy_keeps_safe_and_relative_urls() {
    assert!(to_html("[web](https://example.com)").contains("href=\"https://example.com\""));
    assert!(to_html("[docs](/guide)").contains("href=\"/guide\""));
    assert!(
        to_html("[mail](mailto:team@example.com)").contains("href=\"mailto:team@example.com\"")
    );
}

#[test]
fn literal_autolinks_use_the_untrusted_policy_path() {
    let html = to_html_with_options(
        "https://example.com",
        &Options {
            autolink_literals: true,
            ..Options::default()
        },
    );

    assert!(html.contains("href=\"https://example.com\""));
}

#[test]
fn trusted_policy_is_an_explicit_passthrough_boundary() {
    let html = to_html_with_options(
        "<span onclick=\"run()\">ok</span> [custom](app:open)",
        &Options {
            render_policy: RenderPolicy::Trusted,
            disallowed_raw_html: false,
            ..Options::default()
        },
    );

    assert!(html.contains("<span onclick=\"run()\">"));
    assert!(html.contains("href=\"app:open\""));
}
