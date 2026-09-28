use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::{HtmlRenderer, HtmlRendererOptions, OutlineOptions};

fn options() -> ParserOptions {
    ParserOptions {
        extended_attributes: true,
        bracketed_spans: true,
        image_captions: true,
        table_attributes: true,
        ..ParserOptions::gfm()
    }
}

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    HtmlRenderer::new().render(&document)
}

fn render_untrusted(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions {
        sanitize: true,
        ..HtmlRendererOptions::default()
    })
    .render(&document)
}

#[test]
fn untrusted_metadata_keeps_only_inert_names_as_html_attributes() {
    let html = render_untrusted(
        "[Item]{style=color:red name=config href=javascript:bad srcset=javascript:bad srcdoc=evil action=javascript:bad poster=javascript:bad target=_blank lang=de title=Info width=20 aria-label=Item data-sku=7}",
    );
    for name in [
        "style", "name", "href", "srcset", "srcdoc", "action", "poster", "target",
    ] {
        assert!(html.contains(&format!("data-{name}=\"")), "{name}: {html}");
        assert!(!html.contains(&format!(" {name}=\"")), "{name}: {html}");
    }
    for name in ["lang", "title", "width", "aria-label", "data-sku"] {
        assert!(html.contains(&format!(" {name}=\"")), "{name}: {html}");
    }
    assert!(render("[Item]{style=color:red}").contains(" style=\"color:red\""));
}

#[test]
fn spans_propagate_link_rules_autolinks_and_permalink_markers() {
    for source in [
        "[[x [a](b)]{.c}](u)",
        "[z [x [a](b)]{.c}](u)",
        "[[x `c` [a](b)]{.c}](u)",
        "[outer [inner [go](/x)]{.wrap}](/y)",
    ] {
        let nested = render(source);
        assert!(nested.contains("<span"), "{nested}");
        assert_eq!(nested.matches("<a ").count(), 1, "{nested}");
    }
    let autolink = render("[www.example.com]{.wrap}");
    assert!(
        autolink.contains("<span class=\"wrap\"><a href="),
        "{autolink}"
    );

    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "# [Hello [#](#hello)]{.wrap}", options())
        .parse()
        .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..HtmlRendererOptions::default()
    })
    .render(&document);
    assert_eq!(html.matches("href=\"#hello\"").count(), 1, "{html}");
}

#[test]
fn bracketed_spans_work_without_extended_element_attributes() {
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "[A]{lang=de} [B](/b){lang=de}",
        ParserOptions {
            bracketed_spans: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::new().render(&document);
    assert!(html.contains("<span lang=\"de\">A</span>"), "{html}");
    assert!(html.contains("<a href=\"/b\">B</a>{lang=de}"), "{html}");
}

#[test]
fn default_links_and_headings_have_no_metadata_allocation() {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, "# Title\n\n[Link](/url)", ParserOptions::gfm())
            .parse()
            .unwrap();
    let Node::Heading(heading) = &document.children[0] else {
        panic!("heading");
    };
    assert!(heading.attributes.is_none());
    let Node::Paragraph(paragraph) = &document.children[1] else {
        panic!("paragraph");
    };
    let Node::Link(link) = &paragraph.children[0] else {
        panic!("link");
    };
    assert!(link.attributes.is_none());
    let link_size = std::mem::size_of::<ferromark::ast::Link<'static>>();
    let heading_size = std::mem::size_of::<ferromark::ast::Heading<'static>>();
    assert!(link_size <= 80, "Link grew to {link_size} bytes");
    assert!(heading_size <= 64, "Heading grew to {heading_size} bytes");
}

#[test]
fn quoted_braces_and_attribute_names_keep_valid_html_syntax() {
    let html = render("# Heading {title=\"a{b\"}");
    assert!(html.contains("<h1 id=\"heading\" title=\"a{b\">"), "{html}");
    let html = render("[Text]{xlink:href=u}");
    assert!(html.contains("[Text]{xlink:href=u}"), "{html}");
    assert!(!html.contains("data-xlink:href"), "{html}");
}

#[test]
fn attached_attributes_take_precedence_over_mdx_expressions() {
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "[A]{x=1} text {x=1}\n\n# T {a=b}",
        ParserOptions {
            extended_attributes: true,
            bracketed_spans: true,
            ..ParserOptions::mdx()
        },
    )
    .parse()
    .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("paragraph");
    };
    assert!(matches!(paragraph.children.first(), Some(Node::Span(_))));
    assert!(
        paragraph
            .children
            .iter()
            .any(|child| matches!(child, Node::MdxTextExpression(_)))
    );
    let Node::Heading(heading) = &document.children[1] else {
        panic!("heading");
    };
    assert_eq!(heading.attributes.as_ref().unwrap().values[0].name, "a");
}

#[test]
fn bracketed_spans_render_inline_markdown_and_mapped_metadata() {
    let html = render("[Product **offer**]{.product lang=en sku=\"A-17\" tracking-category=offer}");
    assert_eq!(
        html,
        "<p><span class=\"product\" lang=\"en\" data-sku=\"A-17\" data-tracking-category=\"offer\">Product <strong>offer</strong></span></p>\n"
    );
    assert!(
        render("Das [C’est la vie]{lang=fr} trifft es gut.")
            .contains("<span lang=\"fr\">C’est la vie</span>")
    );
    assert!(
        render("[Notice]{#notice .notice .compact lang=en}")
            .contains("<span id=\"notice\" class=\"notice compact\" lang=\"en\">Notice</span>")
    );
}

#[test]
fn shared_suffixes_attach_to_correct_elements() {
    let html = render(
        "[Docs](https://example.org){.external hreflang=en}\n\n![Alt](a.svg){.diagram width=800 loading=lazy}\n\n# Intro {#intro .compact lang=en}\n\n| A |\n| - |\n| B |\n\n: Prices {.prices currency=EUR}\n\n![Alt](b.svg)\n\n: Caption {.figure sku=7}",
    );
    assert!(html.contains("class=\"external\" hreflang=\"en\">Docs</a>"));
    assert!(html.contains(
        "<img src=\"a.svg\" alt=\"Alt\" class=\"diagram\" width=\"800\" loading=\"lazy\">"
    ));
    assert!(html.contains("<h1 id=\"intro\" class=\"compact\" lang=\"en\">Intro</h1>"));
    assert!(html.contains("<table class=\"prices\" data-currency=\"EUR\">"));
    assert!(html.contains("<figure class=\"figure\" data-sku=\"7\">"));
}

#[test]
fn references_nested_links_and_precedence_work() {
    let html = render(
        "[Docs][d]{.external}\n\n![Alt][i]{loading=lazy}\n\n[with [link](/u)]{.wrap}\n\n[d]: /docs\n[i]: /img.svg",
    );
    assert!(html.contains("<a href=\"/docs\" class=\"external\">Docs</a>"));
    assert!(html.contains("<img src=\"/img.svg\" alt=\"Alt\" loading=\"lazy\">"));
    assert!(html.contains("<span class=\"wrap\">with <a href=\"/u\">link</a></span>"));
}

#[test]
fn duplicate_and_reserved_names_have_deterministic_output() {
    let html = render(
        "[Item]{sku=first data-sku=explicit sku=last LANG=en lang=fr class=\"one two\" .three id=old #new}",
    );
    assert_eq!(
        html,
        "<p><span id=\"new\" class=\"one two three\" data-sku=\"explicit\" lang=\"fr\">Item</span></p>\n"
    );
    let html = render(
        "[Docs](/safe){href=javascript:bad title=bad rel=custom}\n\n![Alt](safe.svg \"Good\"){src=bad alt=bad title=bad width=20}",
    );
    assert!(!html.contains("javascript:bad"));
    assert!(html.contains("<a href=\"/safe\""));
    assert!(html.contains("title=\"bad\" rel=\"custom\">Docs</a>"));
    assert!(html.contains("<img src=\"safe.svg\" alt=\"Alt\" title=\"Good\" width=\"20\">"));
}

#[test]
fn quoted_values_case_and_data_collisions_are_escaped() {
    let html = render(
        "[Text]{TITLE=\"A \\\"quote\\\" and {brace}\" SKU=implicit Data-Sku=explicit onload=run}",
    );
    assert!(
        html.contains("title=\"A &quot;quote&quot; and {brace}\""),
        "{html}"
    );
    assert!(html.contains("data-sku=\"explicit\""), "{html}");
    assert!(!html.contains("implicit"));
    assert!(html.contains("data-onload=\"run\""));
    assert!(render("[X]{bad<name=x}").contains("[X]{bad&lt;name=x}"));
}

#[test]
fn heading_text_and_image_alt_ignore_span_metadata() {
    let html = render("# [Bonjour]{lang=fr}\n\n![[Bonjour]{lang=fr}](a.svg)");
    assert!(
        html.contains("<h1 id=\"bonjour\"><span lang=\"fr\">Bonjour</span></h1>"),
        "{html}"
    );
    assert!(
        html.contains("<img src=\"a.svg\" alt=\"Bonjour\">"),
        "{html}"
    );
}

#[test]
fn malformed_or_disabled_syntax_remains_text_and_protected_content() {
    assert!(render("[Text]{bare}").contains("[Text]{bare}"));
    assert!(render("[Text]{lang=}").contains("[Text]{lang=}"));
    assert!(render("[Text]{#a #b}").contains("<span id=\"b\">Text</span>"));
    assert!(render("`[Text]{lang=fr}`").contains("<code>[Text]{lang=fr}</code>"));
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "[Text]{lang=fr}", ParserOptions::gfm())
        .parse()
        .unwrap();
    assert_eq!(
        HtmlRenderer::new().render(&document),
        "<p>[Text]{lang=fr}</p>\n"
    );
}

struct SpanCount(usize);
impl<'a> Visit<'a> for SpanCount {
    fn visit_span(&mut self, span: &ferromark::ast::InlineSpan<'a>) {
        self.0 += 1;
        ferromark::ast::walk_span(self, span);
    }
}

#[test]
fn span_ast_preserves_authored_names_and_claims_heading_ids() {
    let source = "[Item]{#shared sku=\"A-17\"}\n\n# Heading {#shared}";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("paragraph");
    };
    let Node::Span(span) = &paragraph.children[0] else {
        panic!("span");
    };
    assert_eq!(span.span.end as usize, "[Item]{#shared sku=\"A-17\"}".len());
    assert_eq!(span.attributes[0].name, "sku");
    let mut count = SpanCount(0);
    count.visit_document(&document);
    assert_eq!(count.0, 1);
    let html = HtmlRenderer::new().render(&document);
    assert!(html.contains("<span id=\"shared\" data-sku=\"A-17\">"));
    assert!(html.contains("<h1 id=\"shared-1\">"));
    assert_eq!(
        document.outline(&OutlineOptions::default())[0]
            .id
            .as_deref(),
        Some("shared-1")
    );
}
