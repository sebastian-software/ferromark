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
    render_with_options(
        source,
        TypographyOptions::new(language),
        parser_options,
        renderer_options,
    )
}

fn render_with_options(
    source: &str,
    typography_options: TypographyOptions,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    let mut pass = TypographyPass::new(typography_options);
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
        (
            TypographyLanguage::Czech,
            "<p>She said „Hello“ -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Danish,
            "<p>She said »Hello« -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Finnish,
            "<p>She said ”Hello” -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::NorwegianBokmal,
            "<p>She said «Hello» -- it’s 12 km…</p>\n",
        ),
        (
            TypographyLanguage::Swedish,
            "<p>She said ”Hello” -- it’s 12 km…</p>\n",
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
        render("\"He said 'hello'.\"", TypographyLanguage::Czech),
        "<p>„He said ‚hello‘.“</p>\n"
    );
    assert_eq!(
        render("\"He said 'hello'.\"", TypographyLanguage::Danish),
        "<p>»He said „hello“.«</p>\n"
    );
    assert_eq!(
        render("\"He said 'hello'.\"", TypographyLanguage::Finnish),
        "<p>”He said ’hello’.”</p>\n"
    );
    assert_eq!(
        render("\"He said 'hello'.\"", TypographyLanguage::NorwegianBokmal),
        "<p>«He said ‘hello’.»</p>\n"
    );
    assert_eq!(
        render("\"He said 'hello'.\"", TypographyLanguage::Swedish),
        "<p>”He said ’hello’.”</p>\n"
    );
    assert_eq!(
        render("\"hello\nworld\"", TypographyLanguage::English),
        "<p>“hello\nworld”</p>\n"
    );
}

#[test]
fn guillemet_digraphs_use_the_selected_locale_and_pair_across_inline_markup() {
    let source = "<<Bonjour *tout le monde*>>";
    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    for (language, expected) in [
        (
            TypographyLanguage::French,
            "<p>« Bonjour <em>tout le monde</em> »</p>\n",
        ),
        (
            TypographyLanguage::Danish,
            "<p>»Bonjour <em>tout le monde</em>«</p>\n",
        ),
        (
            TypographyLanguage::English,
            "<p>“Bonjour <em>tout le monde</em>”</p>\n",
        ),
    ] {
        assert_eq!(
            render_with_options(
                source,
                TypographyOptions::new(language).with_guillemet_digraphs(true),
                parser_options.clone(),
                HtmlRendererOptions::default(),
            ),
            expected,
            "language: {}",
            language.as_str()
        );
    }
}

#[test]
fn guillemet_digraph_transform_is_separately_opt_in() {
    assert!(!TypographyOptions::new(TypographyLanguage::French).guillemet_digraphs());
    assert!(
        TypographyOptions::new(TypographyLanguage::French)
            .with_guillemet_digraphs(true)
            .guillemet_digraphs()
    );

    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    assert_eq!(
        render_with(
            "<<Bonjour>>",
            TypographyLanguage::French,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        ),
        "<p>&lt;&lt;Bonjour&gt;&gt;</p>\n"
    );
    assert_eq!(
        render_with_options(
            "<<Bonjour>>",
            TypographyOptions::new(TypographyLanguage::French).with_guillemet_digraphs(true),
            parser_options,
            HtmlRendererOptions::default(),
        ),
        "<p>« Bonjour »</p>\n"
    );
}

#[test]
fn spaced_guillemets_trim_all_ascii_padding_and_pair_balanced_markers() {
    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    let typography_options =
        TypographyOptions::new(TypographyLanguage::French).with_guillemet_digraphs(true);
    assert_eq!(
        render_with_options(
            "<< Bonjour >>",
            typography_options,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        ),
        "<p>« Bonjour »</p>\n"
    );
    assert_eq!(
        render_with_options(
            "a << b and c >> d",
            typography_options,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        ),
        "<p>a «\u{202f}b and c\u{202f}» d</p>\n"
    );
    assert_eq!(
        render_with_options(
            "Il a dit << Bonjour >> et il part.",
            typography_options,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        ),
        "<p>Il a dit «\u{202f}Bonjour\u{202f}» et il part.</p>\n"
    );
    assert_eq!(
        render_with_options(
            "Le mot << oui >> est court.",
            typography_options,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        ),
        "<p>Le mot «\u{202f}oui\u{202f}» est court.</p>\n"
    );
    assert_eq!(
        render_with_options(
            "Il a dit << Bonjour >> 3 fois.",
            typography_options,
            parser_options.clone(),
            HtmlRendererOptions::default(),
        ),
        "<p>Il a dit «\u{202f}Bonjour\u{202f}» 3 fois.</p>\n"
    );
    for source in ["<<>>", "<< >>", "<<\t  \t>>"] {
        assert_eq!(
            render_with_options(
                source,
                typography_options,
                parser_options.clone(),
                HtmlRendererOptions::default(),
            ),
            format!(
                "<p>{}</p>\n",
                source.replace('<', "&lt;").replace('>', "&gt;")
            ),
            "source: {source:?}"
        );
    }
    assert_eq!(
        render_with_options(
            "<<  Bonjour \t >>",
            typography_options,
            parser_options,
            HtmlRendererOptions::default(),
        ),
        "<p>«\u{202f}Bonjour\u{202f}»</p>\n"
    );
    assert_eq!(
        render_with_options(
            "a << b and c >> d",
            TypographyOptions::new(TypographyLanguage::French),
            ParserOptions::default(),
            HtmlRendererOptions::default(),
        ),
        "<p>a &lt;&lt; b and c &gt;&gt; d</p>\n"
    );
}

#[test]
fn nested_guillemets_and_soft_breaks_keep_quote_context() {
    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    let typography_options =
        TypographyOptions::new(TypographyLanguage::English).with_guillemet_digraphs(true);
    for (source, expected) in [
        ("<<outer <<inner>> end>>", "<p>“outer ‘inner’ end”</p>\n"),
        ("<<hello\nworld>>", "<p>“hello\nworld”</p>\n"),
    ] {
        assert_eq!(
            render_with_options(
                source,
                typography_options,
                parser_options.clone(),
                HtmlRendererOptions::default(),
            ),
            expected,
            "source: {source:?}"
        );
    }
}

#[test]
fn adjacent_bare_url_is_not_rewritten_into_a_renderer_link_destination() {
    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    assert_eq!(
        render_with_options(
            "<<https://example.com>>",
            TypographyOptions::new(TypographyLanguage::English).with_guillemet_digraphs(true),
            parser_options,
            HtmlRendererOptions::default(),
        ),
        "<p>&lt;&lt;<a href=\"https://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.com</a>&gt;&gt;</p>\n"
    );
}

#[test]
fn bare_urls_near_a_closer_stay_outside_generated_quote_links() {
    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    for (source, language, url) in [
        (
            "<<voir https://exemple.fr>> ici",
            TypographyLanguage::French,
            "https://exemple.fr",
        ),
        (
            "<<voir https://exemple.fr >> ici",
            TypographyLanguage::French,
            "https://exemple.fr",
        ),
        (
            "<< https://exemple.fr >>",
            TypographyLanguage::French,
            "https://exemple.fr",
        ),
        (
            "<<see https://example.com.>> here",
            TypographyLanguage::English,
            "https://example.com",
        ),
        (
            "<<see (https://example.com)>>",
            TypographyLanguage::English,
            "https://example.com",
        ),
    ] {
        let html = render_with_options(
            source,
            TypographyOptions::new(language).with_guillemet_digraphs(true),
            parser_options.clone(),
            HtmlRendererOptions::default(),
        );
        assert!(html.contains(&format!("href=\"{url}")), "{html}");
        assert!(html.contains("&lt;&lt;"), "{html}");
        assert!(html.contains("&gt;&gt;"), "{html}");
        assert!(!html.contains("%E2%80"), "{html}");
    }
}

#[test]
fn balanced_guillemets_can_enclose_inline_raw_html() {
    let parser_options = ParserOptions {
        guillemet_digraphs: true,
        ..ParserOptions::default()
    };
    assert_eq!(
        render_with_options(
            "<<Bonjour <em>monde</em>>>!",
            TypographyOptions::new(TypographyLanguage::French).with_guillemet_digraphs(true),
            parser_options,
            HtmlRendererOptions::default(),
        ),
        "<p>«\u{202f}Bonjour <em>monde</em>\u{202f}»!</p>\n"
    );
}

#[test]
fn unmatched_escaped_and_protected_digraphs_stay_literal() {
    let options = ParserOptions {
        guillemet_digraphs: true,
        math: true,
        ..ParserOptions::default()
    };
    for (source, expected) in [
        ("<<unmatched", "<p>&lt;&lt;unmatched</p>\n"),
        ("Bonjour>>", "<p>Bonjour&gt;&gt;</p>\n"),
        (
            r#"\"escaped\" and <<Bonjour>>"#,
            "<p>&quot;escaped&quot; and « Bonjour »</p>\n",
        ),
        ("<<<Bonjour>>>", "<p>&lt;&lt;&lt;Bonjour&gt;&gt;&gt;</p>\n"),
        (r"\<\<escaped\>\>", "<p>&lt;&lt;escaped&gt;&gt;</p>\n"),
        (
            "&lt;&lt;entity&gt;&gt; &LT;&LT;uppercase&GT;&GT;",
            "<p>&lt;&lt;entity&gt;&gt; &lt;&lt;uppercase&gt;&gt;</p>\n",
        ),
        ("<<`code`>>", "<p>&lt;&lt;<code>code</code>&gt;&gt;</p>\n"),
        (
            "$<<math>>$",
            "<p><span class=\"ox-math ox-math-inline\" data-ox-tex=\"&lt;&lt;math&gt;&gt;\"><math><mtext>&lt;&lt;math&gt;&gt;</mtext></math></span></p>\n",
        ),
    ] {
        assert_eq!(
            render_with_options(
                source,
                TypographyOptions::new(TypographyLanguage::French).with_guillemet_digraphs(true),
                options.clone(),
                HtmlRendererOptions::default(),
            ),
            expected,
            "source: {source:?}"
        );
    }
}

#[test]
fn symmetric_authored_quotes_close_before_the_next_quote_pair() {
    assert_eq!(
        render("”already” and \"new\".", TypographyLanguage::Swedish),
        "<p>”already” and ”new”.</p>\n"
    );
    assert_eq!(
        render("’already’ and \"new\".", TypographyLanguage::Finnish),
        "<p>’already’ and ”new”.</p>\n"
    );
    assert_eq!(
        render(
            "”It’s ’twas fine’” and \"new\".",
            TypographyLanguage::Swedish
        ),
        "<p>”It’s ’twas fine’” and ”new”.</p>\n"
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
