#![allow(clippy::panic, clippy::unwrap_used)]

use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark_transforms::{
    TransformContext, TransformPass, TransformPipeline, TypographyLanguage, TypographyOptions,
    TypographyPass,
};

fn render(source: &str, language: TypographyLanguage) -> String {
    render_with(
        source,
        language,
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    )
}

fn render_with(
    source: &str,
    language: TypographyLanguage,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    let mut pass = TypographyPass::new(TypographyOptions::new(language));
    pass.apply(&mut document, &context).unwrap();
    HtmlRenderer::with_options(renderer_options).render(&document)
}

#[test]
fn language_defaults_match_the_reviewed_reference_cases() {
    let source = "She said \"Hello\" -- it's 12 km...";
    let cases = [
        (
            TypographyLanguage::English,
            "<p>She said “Hello” — it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Spanish,
            "<p>She said «Hello» -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::French,
            "<p>She said « Hello » -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Portuguese,
            "<p>She said «Hello» -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::German,
            "<p>She said „Hello“ -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Italian,
            "<p>She said «Hello» -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Dutch,
            "<p>She said ‘Hello’ -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Polish,
            "<p>She said „Hello” -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Russian,
            "<p>She said «Hello» — it&#39;s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Ukrainian,
            "<p>She said «Hello» -- it&#39;s 12 km…</p>\n",
        ),
    ];

    for (language, expected) in cases {
        assert_eq!(
            render(source, language),
            expected,
            "language: {}",
            language.as_str()
        );
    }
}

#[test]
fn quotes_pair_across_inline_markup_and_nest_by_locale() {
    assert_eq!(
        render(
            "\"Hello **world**\" and \"hello [friend](/docs)\".",
            TypographyLanguage::English
        ),
        "<p>“Hello <strong>world</strong>” and “hello <a href=\"/docs\">friend</a>”.</p>\n"
    );
    assert_eq!(
        render("\"He said 'hello'.\"", TypographyLanguage::English),
        "<p>“He said ‘hello’.”</p>\n"
    );
    assert_eq!(
        render("\"He said 'hello'.\"", TypographyLanguage::German),
        "<p>„He said ‚hello‘.“</p>\n"
    );
    assert_eq!(
        render("\"hello\nworld\"", TypographyLanguage::English),
        "<p>“hello\nworld”</p>\n"
    );
}

#[test]
fn dashes_and_ellipses_can_be_disabled_independently() {
    let source = "She said \"Hello\" -- it's fine...";
    for (options, expected) in [
        (
            TypographyOptions::new(TypographyLanguage::English).with_dashes(false),
            "<p>She said “Hello” -- it’s fine…</p>\n",
        ),
        (
            TypographyOptions::new(TypographyLanguage::English).with_ellipses(false),
            "<p>She said “Hello” — it’s fine...</p>\n",
        ),
    ] {
        let allocator = Allocator::new();
        let mut document = ferromark::parse(&allocator, source).unwrap();
        let renderer_options = HtmlRendererOptions::default();
        let context = TransformContext::new(&allocator, source, &renderer_options);
        let mut pass = TypographyPass::new(options);
        pass.apply(&mut document, &context).unwrap();
        assert_eq!(
            HtmlRenderer::with_options(renderer_options).render(&document),
            expected
        );
    }
}

#[test]
fn escaped_and_entity_authored_quotes_stay_straight() {
    let source = r#"\"escaped\" and &quot;entity&quot; and "converted""#;
    let html = render(source, TypographyLanguage::English);
    assert!(html.contains("&quot;escaped&quot;"), "{html}");
    assert!(html.contains("&quot;entity&quot;"), "{html}");
    assert!(html.contains("“converted”"), "{html}");
}

#[test]
fn existing_typography_and_protected_content_are_preserved() {
    let source = "Already “curly” — fine…; word--pair; code `--...`; URL https://example.com/a--b...c; [link](/a--b...c \"--...\").";
    let html = render(source, TypographyLanguage::English);
    assert!(html.contains("Already “curly” — fine…"), "{html}");
    assert!(html.contains("word--pair"), "{html}");
    assert!(html.contains("<code>--...</code>"), "{html}");
    assert!(
        html.contains("href=\"https://example.com/a--b...c\""),
        "{html}"
    );
    assert!(
        html.contains("href=\"/a--b...c\" title=\"--...\""),
        "{html}"
    );
}

#[test]
fn math_html_mdx_expressions_and_image_metadata_are_protected() {
    let source = r#"<span title="--...">--...</span> $--...$ ![alt --...](image--... "title --...") {value("--...")}"#;
    let allocator = Allocator::new();
    let parser_options = ParserOptions {
        math: true,
        mdx: true,
        ..ParserOptions::default()
    };
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    TypographyPass::new(TypographyOptions::new(TypographyLanguage::English))
        .apply(&mut document, &context)
        .unwrap();

    let ferromark::ast::Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, ferromark::ast::Node::Html(_)))
    );
    assert!(paragraph.children.iter().any(
        |node| matches!(node, ferromark::ast::Node::InlineMath(math) if math.value == "--...")
    ));
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, ferromark::ast::Node::Text(text) if text.value == "--..."))
    );
    assert!(paragraph.children.iter().any(
        |node| matches!(node, ferromark::ast::Node::Image(image) if image.alt == "alt --..." && image.url == "image--..." && image.title == Some("title --..."))
    ));
    assert!(paragraph.children.iter().any(
        |node| matches!(node, ferromark::ast::Node::MdxTextExpression(expression) if expression.value == "value(\"--...\")")
    ));
}

#[test]
fn pass_is_idempotent_and_preserves_source_spans() {
    let source = "\"Hello\" -- it's fine...";
    let allocator = Allocator::new();
    let mut document = ferromark::parse(&allocator, source).unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    let mut pipeline = TransformPipeline::new();
    pipeline.add(TypographyPass::new(TypographyOptions::new(
        TypographyLanguage::English,
    )));
    pipeline.add(TypographyPass::new(TypographyOptions::new(
        TypographyLanguage::English,
    )));
    pipeline.run(&mut document, &context).unwrap();
    assert_eq!(
        HtmlRenderer::with_options(renderer_options).render(&document),
        "<p>“Hello” — it’s fine…</p>\n"
    );

    let ferromark::ast::Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    let ferromark::ast::Node::Text(text) = &paragraph.children[0] else {
        panic!("expected a text node");
    };
    assert_eq!(text.span, ferromark::ast::Span::new(0, source.len() as u32));
}

#[test]
fn typography_reaches_headings_lists_tables_and_footnotes() {
    let source = "# \"heading\"...\n\n- \"item\"...\n\n> \"quote\"...\n\n| first | second |\n| --- | --- |\n| \"cell\"... | 12 km |\n: \"caption\"... {#inventory}\n\n[^note]\n\n[^note]: \"note\"...\n";
    let renderer_options = HtmlRendererOptions {
        heading_ids: true,
        ..HtmlRendererOptions::default()
    };
    let html = render_with(
        source,
        TypographyLanguage::English,
        ParserOptions {
            table_attributes: true,
            ..ParserOptions::gfm()
        },
        renderer_options,
    );

    assert!(html.contains("id=\"heading\""), "{html}");
    assert!(html.contains("“heading”…"), "{html}");
    assert!(html.contains("“item”…"), "{html}");
    assert!(html.contains("“quote”…"), "{html}");
    assert!(html.contains("“cell”…"), "{html}");
    assert!(html.contains("<caption>“caption”…</caption>"), "{html}");
    assert!(html.contains("12 km"), "{html}");
    assert!(html.contains("“note”…"), "{html}");
}

#[test]
fn typography_reaches_definition_list_descriptions() {
    let source = "Term\n: \"definition\"...";
    let parser_options = ParserOptions {
        definition_lists: true,
        ..ParserOptions::default()
    };
    let html = render_with(
        source,
        TypographyLanguage::English,
        parser_options,
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("<dd>“definition”…</dd>"), "{html}");
}
