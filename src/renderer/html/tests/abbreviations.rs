use std::collections::BTreeMap;

use crate::allocator::Allocator;
use crate::parser::{Parser, ParserOptions};
use crate::renderer::html::{
    AbbreviationOptions, HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks,
};

fn options(overrides: BTreeMap<String, Option<String>>) -> AbbreviationOptions {
    AbbreviationOptions { overrides }
}

fn renderer(overrides: BTreeMap<String, Option<String>>) -> HtmlRenderer {
    HtmlRenderer::with_options_and_abbreviations(HtmlRendererOptions::default(), options(overrides))
}

#[test]
fn automatic_abbreviations_are_opt_in_and_include_unknown_terms() {
    let allocator = Allocator::new();
    let source = "API HTTP HTML5 UTF8 XYZ A";
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(
        HtmlRenderer::new().render(&document),
        "<p>API HTTP HTML5 UTF8 XYZ A</p>\n"
    );
    assert_eq!(
        renderer(BTreeMap::new()).render(&document),
        concat!(
            "<p><abbr title=\"Application Programming Interface\">API</abbr> ",
            "<abbr title=\"Hypertext Transfer Protocol\">HTTP</abbr> ",
            "<abbr title=\"Hypertext Markup Language, version 5\">HTML5</abbr> ",
            "<abbr title=\"Unicode Transformation Format, 8-bit\">UTF8</abbr> ",
            "<abbr>XYZ</abbr> A</p>\n"
        )
    );
}

#[test]
fn overrides_keep_missing_empty_and_suppressed_entries_distinct() {
    let allocator = Allocator::new();
    let source = "API XYZ GraphQL HTTP/2 UTF-8";
    let document = Parser::new(&allocator, source).parse().unwrap();
    let overrides = BTreeMap::from([
        (
            "API".to_string(),
            Some("A & <Custom> \"interface\"".to_string()),
        ),
        ("XYZ".to_string(), None),
        ("GraphQL".to_string(), Some(String::new())),
    ]);

    assert_eq!(
        renderer(overrides).render(&document),
        concat!(
            "<p><abbr title=\"A &amp; &lt;Custom&gt; &quot;interface&quot;\">API</abbr> ",
            "XYZ <abbr>GraphQL</abbr> ",
            "<abbr title=\"Hypertext Transfer Protocol, version 2\">HTTP/2</abbr> ",
            "<abbr title=\"Unicode Transformation Format, 8-bit\">UTF-8</abbr></p>\n"
        )
    );
}

#[test]
fn automatic_tokens_obey_case_unicode_identifier_digit_and_plural_boundaries() {
    let allocator = Allocator::new();
    let source = "API API2 APIs API-like xAPI APIx \\_API API\\_ ÄAPI APIÄ A";
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(
        renderer(BTreeMap::new()).render(&document),
        concat!(
            "<p><abbr title=\"Application Programming Interface\">API</abbr> ",
            "<abbr>API2</abbr> ",
            "<abbr title=\"Application Programming Interface\">API</abbr>s ",
            "<abbr title=\"Application Programming Interface\">API</abbr>-like ",
            "xAPI APIx _API API_ ÄAPI APIÄ A</p>\n"
        )
    );
}

#[test]
fn renderer_preserves_heading_text_links_images_code_urls_and_authored_abbr() {
    let allocator = Allocator::new();
    let tick = char::from(96);
    let source = format!(
        "# API\n\n**HTTP** {tick}HTML{tick} [API](/API) ![API](/image) \
         https://example.com/API\n\n<abbr title=\"API\">API <strong>HTTP</strong></abbr>\n"
    );
    let document = Parser::new(&allocator, &source).parse().unwrap();
    let mut html_renderer = HtmlRenderer::with_options_and_abbreviations(
        HtmlRendererOptions {
            sanitize: false,
            source_spans: true,
            ..HtmlRendererOptions::default()
        },
        options(BTreeMap::new()),
    );

    assert_eq!(
        html_renderer.render_borrowed(&document),
        concat!(
            "<h1 id=\"api\" data-source-span=\"0-6\">",
            "<abbr title=\"Application Programming Interface\">API</abbr></h1>\n",
            "<p data-source-span=\"7-74\"><strong><abbr title=\"Hypertext Transfer Protocol\">HTTP</abbr></strong> ",
            "<code>HTML</code> <a href=\"/API\"><abbr title=\"Application Programming Interface\">API</abbr></a> ",
            "<img src=\"/image\" alt=\"API\"> ",
            "<a href=\"https://example.com/API\" target=\"_blank\" rel=\"noopener noreferrer\">",
            "https://example.com/API</a></p>\n",
            "<p data-source-span=\"75-126\"><abbr title=\"API\">API <strong>HTTP</strong></abbr></p>\n"
        )
    );
}

#[test]
fn link_labels_are_prose_but_url_text_and_destinations_are_preserved() {
    let allocator = Allocator::new();
    let source = "[API and https://API.example/API](/API) https://API.example/API";
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(
        HtmlRenderer::with_options_and_abbreviations(
            HtmlRendererOptions {
                autolink_urls: false,
                ..HtmlRendererOptions::default()
            },
            options(BTreeMap::new())
        )
        .render(&document),
        concat!(
            "<p><a href=\"/API\"><abbr title=\"Application Programming Interface\">API</abbr> and ",
            "https://API.example/API</a> https://API.example/API</p>\n"
        )
    );
}

#[test]
fn ordinary_and_hook_rendering_share_adjacent_text_boundaries() {
    let allocator = Allocator::new();
    let source = "x\\_API &Auml;API [x\\_API](/x) [API](/x)";
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut ordinary = renderer(BTreeMap::new());
    let mut hooked = renderer(BTreeMap::new());
    let mut hooks = NoHtmlRenderHooks;
    let html = ordinary.render(&document);

    assert_eq!(hooked.render_with_hooks(&document, &mut hooks), html);
    assert_eq!(html.matches("<abbr").count(), 1, "{html}");
    assert!(html.contains("x_API ÄAPI"), "{html}");

    let escaped_url = Parser::new(&allocator, "https://example.com/a\\_b")
        .parse()
        .unwrap();
    assert_eq!(
        renderer(BTreeMap::new()).render(&escaped_url),
        HtmlRenderer::new().render(&escaped_url)
    );
}

#[test]
fn inline_html_state_is_scoped_and_skipped_when_sanitizing_or_rendering_blocks() {
    let cases = [
        (
            "<span>API\n\n# API",
            false,
            "<h1 id=\"api\"><abbr title=\"Application Programming Interface\">API</abbr></h1>",
        ),
        (
            "<div>\nAPI\n</div>\n\nAPI",
            false,
            "<p><abbr title=\"Application Programming Interface\">API</abbr></p>",
        ),
        (
            "<span>API\n\nAPI",
            true,
            "<p><abbr title=\"Application Programming Interface\">API</abbr></p>",
        ),
    ];

    for (source, sanitize, expected) in cases {
        let allocator = Allocator::new();
        let document = Parser::new(&allocator, source).parse().unwrap();
        let mut renderer = HtmlRenderer::with_options_and_abbreviations(
            HtmlRendererOptions {
                sanitize,
                ..HtmlRendererOptions::default()
            },
            AbbreviationOptions::default(),
        );
        let html = renderer.render(&document);
        assert!(html.contains(expected), "source: {source:?}; html: {html}");
    }
}

#[test]
fn callout_urls_remain_unlinked_when_abbreviations_are_enabled() {
    let allocator = Allocator::new();
    let source = "> [!NOTE]\n> body https://example.com/API\n\nAPI";
    let document = Parser::new(&allocator, source).parse().unwrap();
    let plain = HtmlRenderer::new().render(&document);
    let annotated = renderer(BTreeMap::new()).render(&document);

    assert!(
        !plain.contains("href=\"https://example.com/API\""),
        "{plain}"
    );
    assert!(
        !annotated.contains("href=\"https://example.com/API\""),
        "{annotated}"
    );
    assert!(
        annotated
            .ends_with("<p><abbr title=\"Application Programming Interface\">API</abbr></p>\n")
    );
}

#[test]
fn sanitized_raw_html_text_stays_unannotated() {
    let allocator = Allocator::new();
    let source = "<abbr title=\"authored\">API</abbr> API";
    let document = Parser::new(&allocator, source).parse().unwrap();

    assert_eq!(
        HtmlRenderer::with_options_and_abbreviations(
            HtmlRendererOptions {
                sanitize: true,
                ..HtmlRendererOptions::default()
            },
            options(BTreeMap::new())
        )
        .render(&document),
        "<p>&lt;abbr title=&quot;authored&quot;&gt;API&lt;/abbr&gt; <abbr title=\"Application Programming Interface\">API</abbr></p>\n"
    );
}

#[test]
fn renderer_reuse_keeps_configuration_but_not_document_state() {
    let allocator = Allocator::new();
    let first = Parser::new(&allocator, "API").parse().unwrap();
    let second = Parser::new(&allocator, "XYZ API").parse().unwrap();
    let mut renderer = renderer(BTreeMap::new());

    assert_eq!(
        renderer.render_borrowed(&first),
        "<p><abbr title=\"Application Programming Interface\">API</abbr></p>\n"
    );
    assert_eq!(
        renderer.render_borrowed(&second),
        "<p><abbr>XYZ</abbr> <abbr title=\"Application Programming Interface\">API</abbr></p>\n"
    );
}

#[test]
fn front_matter_and_math_stay_outside_abbreviation_processing() {
    let allocator = Allocator::new();
    let source = "---\napi: API\n---\n\n$API$ and API";
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            front_matter: true,
            math: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();

    assert_eq!(
        document
            .front_matter
            .as_ref()
            .map(|metadata| metadata.value),
        Some("api: API\n")
    );
    assert_eq!(
        renderer(BTreeMap::new()).render(&document),
        concat!(
            "<p><span class=\"ox-math ox-math-inline\" data-ox-tex=\"API\">",
            "<math><mtext>API</mtext></math></span> and ",
            "<abbr title=\"Application Programming Interface\">API</abbr></p>\n"
        )
    );
}

#[test]
fn representative_technical_markdown_documents_false_positives_and_exclusions() {
    let source = include_str!("../../../../benches/fixtures/abbreviations.md");
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::new(&allocator, source).parse().unwrap();
    let unfiltered_html = renderer(BTreeMap::new()).render(&document);
    assert!(unfiltered_html.contains("<abbr>README</abbr>"));

    let overrides = BTreeMap::from([
        ("README".to_string(), None),
        ("ID".to_string(), Some("Identifier".to_string())),
        ("GraphQL".to_string(), Some(String::new())),
    ]);
    let html = renderer(overrides).render(&document);

    assert!(html.contains("<abbr title=\"Hypertext Transfer Protocol\">HTTP</abbr>"));
    assert!(html.contains("<abbr>GraphQL</abbr>"));
    assert!(html.contains("<abbr title=\"Identifier\">ID</abbr>"));
    assert!(html.contains("<abbr>MUST</abbr>"));
    assert!(html.contains("<abbr>SDK</abbr>"));
    assert!(!html.contains("<abbr>README</abbr>"));
    assert!(html.contains("clientID"));
    assert!(html.contains("API_TOKEN belongs"));
    assert!(!html.contains("API</abbr>_TOKEN"));
    assert!(html.contains("const API_TOKEN = process.env.API_TOKEN;"));
    assert!(html.contains("alt=\"API flow\""));
}
