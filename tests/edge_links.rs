#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ferromark::parser::ParserOptions;
use ferromark::renderer::HtmlRendererOptions;
#[test]
fn external_links_get_security_attributes() {
    let html = render(
        "[site](https://example.com)",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn parsed_links_can_skip_target_blank_security_attributes() {
    let html = render(
        "[site](https://example.com)",
        ParserOptions::default(),
        HtmlRendererOptions {
            link_target_blank: false,
            ..Default::default()
        },
    );

    assert_eq!(html, "<p><a href=\"https://example.com\">site</a></p>\n");
}

#[test]
fn link_target_blank_does_not_disable_renderer_autolink_attributes() {
    let html = render(
        "Visit https://example.com",
        ParserOptions::default(),
        HtmlRendererOptions {
            link_target_blank: false,
            ..Default::default()
        },
    );

    assert!(
        html.contains(
            "<a href=\"https://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">"
        ),
        "{html}"
    );
}

#[test]
fn relative_links_do_not_get_external_attributes() {
    let html = render(
        "[guide](./guide.md)",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn base_prefixes_root_absolute_markdown_links() {
    let html = render(
        "[Guide](/guide) [Dir](/guide/) [Markdown](/api.md#types)",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".into(),
            ..Default::default()
        },
    );

    insta::assert_snapshot!(html);
}

#[test]
fn base_prefixes_markdown_links_with_or_without_trailing_slash() {
    for base_url in ["/docs", "/docs/"] {
        let html = render(
            "[Markdown](/guide.md)",
            ParserOptions::default(),
            HtmlRendererOptions {
                convert_md_links: true,
                base_url: base_url.into(),
                ..Default::default()
            },
        );

        assert_eq!(
            html,
            "<p><a href=\"/docs/guide/index.html\">Markdown</a></p>\n"
        );
    }
}

#[test]
fn base_prefixes_root_absolute_markdown_images() {
    let html = render(
        "![logo](/img/logo.png)",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".into(),
            ..Default::default()
        },
    );

    assert_eq!(
        html,
        "<p><img src=\"/docs/img/logo.png\" alt=\"logo\"></p>\n"
    );
}

#[test]
fn base_prefixes_root_absolute_raw_html_attrs() {
    let html = render(
        "<div>\n<a href=\"/guide\">Guide</a>\n<img src='/img/logo.png'>\n<script src=\"//cdn.example/app.js\"></script>\n</div>",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".into(),
            ..Default::default()
        },
    );

    insta::assert_snapshot!(html);
}

#[test]
fn nested_parentheses_in_links_are_preserved_in_output() {
    let html = render(
        "[docs](https://example.com/a(b)c)",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn commonmark_url_syntax_bytes_are_serialized_in_href() {
    let options = HtmlRendererOptions {
        autolink_urls: false,
        link_target_blank: false,
        ..Default::default()
    };
    let cases = [
        (
            r"<https://example.com?find=\*>
",
            r#"<p><a href="https://example.com?find=%5C*">https://example.com?find=\*</a></p>
"#,
        ),
        (
            r#"[foo]: /url\bar\*baz "foo\"bar\baz"

[foo]
"#,
            r#"<p><a href="/url%5Cbar*baz" title="foo&quot;bar\baz">foo</a></p>
"#,
        ),
        (
            r"<https://foo.bar.`baz>`
",
            r#"<p><a href="https://foo.bar.%60baz">https://foo.bar.`baz</a>`</p>
"#,
        ),
        (
            r"[link](foo\bar)
",
            r#"<p><a href="foo%5Cbar">link</a></p>
"#,
        ),
        (
            r"[foo<https://example.com/?search=](uri)>
",
            r#"<p>[foo<a href="https://example.com/?search=%5D(uri)">https://example.com/?search=](uri)</a></p>
"#,
        ),
        (
            r"[foo<https://example.com/?search=][ref]>

[ref]: /uri
",
            r#"<p>[foo<a href="https://example.com/?search=%5D%5Bref%5D">https://example.com/?search=][ref]</a></p>
"#,
        ),
        (
            r"<https://example.com/\[\>
",
            r#"<p><a href="https://example.com/%5C%5B%5C">https://example.com/\[\</a></p>
"#,
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(
            render(source, ParserOptions::default(), options.clone()),
            expected,
            "source: {source:?}"
        );
    }
}

#[test]
fn url_serialization_preserves_ipv6_authority_brackets() {
    let options = HtmlRendererOptions {
        autolink_urls: false,
        link_target_blank: false,
        ..Default::default()
    };
    for (source, expected) in [
        (
            "[x](https://[::1]/a)\n",
            "<p><a href=\"https://[::1]/a\">x</a></p>\n",
        ),
        (
            "[x](https://[::1]:/a)\n",
            "<p><a href=\"https://[::1]:/a\">x</a></p>\n",
        ),
        (
            "[x](https://u[x]@[::1]/a)\n",
            "<p><a href=\"https://u%5Bx%5D@[::1]/a\">x</a></p>\n",
        ),
        (
            "[x](https://user:pass@[2001:db8::1]:8443/a)\n",
            "<p><a href=\"https://user:pass@[2001:db8::1]:8443/a\">x</a></p>\n",
        ),
        (
            "<https://[2001:db8::1]/>\n",
            "<p><a href=\"https://[2001:db8::1]/\">https://[2001:db8::1]/</a></p>\n",
        ),
        (
            "[x](https://example.com/a[b])\n",
            "<p><a href=\"https://example.com/a%5Bb%5D\">x</a></p>\n",
        ),
    ] {
        assert_eq!(
            render(source, ParserOptions::default(), options.clone()),
            expected,
            "source: {source:?}"
        );
    }
}

#[test]
fn xhtml_images_self_close() {
    let html = render(
        "![logo](/logo.svg)",
        ParserOptions::default(),
        HtmlRendererOptions {
            xhtml: true,
            ..Default::default()
        },
    );

    insta::assert_snapshot!(html);
}

#[test]
fn script_extensions_preserve_surrounding_punctuation() {
    let html = render(
        "\"Smart\" H~2~O x^2^",
        ParserOptions {
            subscript: true,
            superscript: true,
            ..ParserOptions::default()
        },
        HtmlRendererOptions::default(),
    );

    assert_eq!(
        html,
        "<p>&quot;Smart&quot; H<sub>2</sub>O x<sup>2</sup></p>\n"
    );
}

#[test]
fn ascii_punctuation_stays_outside_gfm_autolink_rendering() {
    let parser_options = ParserOptions {
        autolinks: true,
        ..ParserOptions::default()
    };
    let renderer_options = HtmlRendererOptions {
        link_target_blank: false,
        ..Default::default()
    };

    for (source, expected) in [
        (
            r#"The URL "https://example.com" is valid."#,
            "<p>The URL &quot;<a href=\"https://example.com\">https://example.com</a>&quot; is valid.</p>\n",
        ),
        (
            "The URL 'https://example.com' is valid.",
            "<p>The URL &#39;<a href=\"https://example.com\">https://example.com</a>&#39; is valid.</p>\n",
        ),
        (
            "See https://example.com...",
            "<p>See <a href=\"https://example.com\">https://example.com</a>...</p>\n",
        ),
    ] {
        let html = render(source, parser_options.clone(), renderer_options.clone());

        assert_eq!(html, expected, "source: {source}");
    }
}

#[test]
fn markdown_urls_on_another_origin_are_left_alone() {
    // A `.md` on another origin is not a page this build generates, so there
    // is no `index.html` route to rewrite it to. It must stay verbatim — and
    // stay recognizable as external, so it still gets the security attributes.
    let html = render(
        concat!(
            "[abs](https://ex.com/docs/guide.md)\n\n",
            "[proto](//cdn.example/docs/x.md)\n\n",
            "[mail](mailto:a@b.com/x.md)\n\n",
            "[upper](https://ex.com/docs/GUIDE.MD)\n\n",
            "![alt](https://ex.com/d/g.md)\n\n",
            "<a href=\"https://ex.com/docs/guide.md\">raw</a>\n",
        ),
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".into(),
            source_path: "content/blog/post.md".into(),
            ..Default::default()
        },
    );

    insta::assert_snapshot!(html);
}

#[test]
fn local_markdown_urls_convert_around_query_and_fragment() {
    // The extension lives in the path, not in the query string, so a local
    // link still routes to its generated page and carries its suffix along.
    let html = render(
        "[q](./other.md?tab=2) [both](./other.md?tab=2#anchor) [frag](./other.md#anchor)",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".into(),
            source_path: "api/index.md".into(),
            ..Default::default()
        },
    );

    insta::assert_snapshot!(html);
}
