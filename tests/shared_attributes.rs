use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::{HtmlRenderer, OutlineOptions};

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
