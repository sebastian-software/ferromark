use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks};
use ferromark::{HtmlRenderer, HtmlRendererOptions, OutlineOptions};

fn document<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    mut options: ParserOptions,
) -> ferromark::ast::Document<'a> {
    options.blockquote_attributions = true;
    Parser::with_options(allocator, source, options)
        .parse()
        .unwrap()
}

fn render(source: &str, enabled: bool) -> String {
    render_with_options(
        source,
        ParserOptions {
            blockquote_attributions: enabled,
            ..ParserOptions::gfm()
        },
    )
}

fn render_with_options(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    HtmlRenderer::new().render(&document)
}

struct Count(usize);

impl<'a> Visit<'a> for Count {
    fn visit_link(&mut self, link: &ferromark::ast::Link<'a>) {
        self.0 += 1;
        ferromark::ast::walk_link(self, link);
    }
}

#[derive(Default)]
struct HookVisits {
    figure: bool,
    quote: bool,
}

impl HtmlRenderHooks for HookVisits {
    fn render_node(
        &mut self,
        node: &Node<'_>,
        _cx: &mut HtmlRenderContext<'_>,
    ) -> HtmlRenderControl {
        self.figure |= matches!(node, Node::Figure(_));
        self.quote |= matches!(node, Node::BlockQuote(_));
        HtmlRenderControl::Default
    }
}

#[test]
fn attribution_is_opt_in_and_attaches_after_a_quote() {
    let source = "> Excerpt\n: Jane Example";
    assert_eq!(
        render(source, false),
        "<blockquote>\n<p>Excerpt\n: Jane Example</p>\n</blockquote>\n"
    );
    assert_eq!(
        render(source, true),
        "<figure>\n<blockquote>\n<p>Excerpt</p>\n</blockquote>\n\n<figcaption>Jane Example</figcaption>\n</figure>\n"
    );
}

#[test]
fn accepts_one_blank_line_and_renders_inline_markdown_without_cite() {
    for source in [
        "> Excerpt\n: **Jane** [site](https://example.org)",
        "> Excerpt\n\n: **Jane** [site](https://example.org)",
    ] {
        let html = render(source, true);
        assert!(html.starts_with("<figure>\n<blockquote>\n"), "{html}");
        assert!(
            html.contains("<figcaption><strong>Jane</strong> <a href=\"https://example.org\" target=\"_blank\" rel=\"noopener noreferrer\">site</a></figcaption>"),
            "{html}"
        );
        assert!(!html.contains("<cite"), "{html}");
        assert!(!html.contains("cite="), "{html}");
    }
}

#[test]
fn figure_wraps_all_quote_blocks_and_preserves_reference_links() {
    let source = "> First paragraph\n>\n> Second paragraph\n: [Jane][author]\n\n[author]: https://example.org \"Author home\"";
    let html = render(source, true);
    assert!(
        html.starts_with("<figure>\n<blockquote>\n<p>First paragraph</p>\n<p>Second paragraph</p>"),
        "{html}"
    );
    assert!(html.contains("<figcaption><a href=\"https://example.org\" target=\"_blank\" rel=\"noopener noreferrer\" title=\"Author home\">Jane</a></figcaption>"), "{html}");

    let allocator = Allocator::new();
    let document = document(&allocator, source, ParserOptions::gfm());
    let Node::Figure(figure) = &document.children[0] else {
        panic!("expected attributed quote figure");
    };
    assert!(matches!(figure.content, Node::BlockQuote(_)));
    assert_eq!(
        figure.span.source_text(source),
        &source[..source.find("\n\n[author]").unwrap() + 1]
    );
    let mut count = Count(0);
    count.visit_document(&document);
    assert_eq!(count.0, 1);

    let mut hooks = HookVisits::default();
    let mut renderer = HtmlRenderer::new();
    assert_eq!(renderer.render_with_hooks(&document, &mut hooks), html,);
    assert!(hooks.figure && hooks.quote);
}

#[test]
fn caption_id_and_classes_belong_to_the_figure_and_join_id_planning() {
    let source = "> Excerpt\n: Jane {#source .byline .wide}\n\n# Heading {#source}";
    let options = ParserOptions {
        heading_attributes: true,
        ..ParserOptions::gfm()
    };
    let html = render_with_options(
        source,
        ParserOptions {
            blockquote_attributions: true,
            ..options
        },
    );
    assert!(
        html.contains("<figure id=\"source\" class=\"byline wide\">"),
        "{html}"
    );
    assert!(html.contains("<h1 id=\"source-1\">Heading</h1>"), "{html}");

    let allocator = Allocator::new();
    let parsed = document(
        &allocator,
        source,
        ParserOptions {
            heading_attributes: true,
            ..ParserOptions::gfm()
        },
    );
    let Node::Figure(figure) = &parsed.children[0] else {
        panic!("expected figure");
    };
    let attributes = figure.attributes.as_ref().expect("figure attributes");
    assert_eq!(attributes.id, Some("source"));
    assert_eq!(attributes.classes.as_slice(), ["byline", "wide"]);
    let outline = parsed.outline(&OutlineOptions::default());
    assert_eq!(outline[0].id.as_deref(), Some("source-1"));
}

#[test]
fn extended_key_value_suffixes_are_stored_and_rendered_on_the_figure() {
    let source = "> Excerpt\n: Jane {#source .byline lang=fr tracking-category=quote title=Source}";
    let options = ParserOptions {
        blockquote_attributions: true,
        extended_attributes: true,
        ..ParserOptions::gfm()
    };
    let allocator = Allocator::new();
    let parsed = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    let Node::Figure(figure) = &parsed.children[0] else {
        panic!("expected attributed quote figure");
    };
    let attributes = figure.attributes.as_ref().expect("figure attributes");
    assert_eq!(attributes.values.len(), 3);
    assert_eq!(attributes.values[0].name, "lang");
    assert_eq!(attributes.values[0].value, "fr");
    assert_eq!(attributes.values[1].name, "tracking-category");
    assert_eq!(attributes.values[1].value, "quote");
    let html = HtmlRenderer::new().render(&parsed);
    assert!(
        html.starts_with("<figure id=\"source\" class=\"byline\" lang=\"fr\" data-tracking-category=\"quote\" title=\"Source\">") ,
        "{html}"
    );
}

#[test]
fn source_line_must_be_outside_quote_and_adjacent_in_same_container() {
    assert_eq!(
        render("> Excerpt\n> : inside", true),
        "<blockquote>\n<p>Excerpt\n: inside</p>\n</blockquote>\n"
    );
    assert!(
        render("- item\n  > Excerpt\n  : Jane", true).contains("<figcaption>Jane</figcaption>")
    );
    let list_quote = render("- > Excerpt\n: Jane", true);
    assert!(!list_quote.contains("<figcaption>"), "{list_quote}");
    assert!(
        list_quote.contains("<p>Excerpt\n: Jane</p>"),
        "{list_quote}"
    );
    let lazy_list_quote = render("- > Excerpt\nlazy continuation\n: Jane", true);
    assert!(
        !lazy_list_quote.contains("<figcaption>"),
        "{lazy_list_quote}"
    );
    assert!(
        lazy_list_quote.contains("Excerpt\nlazy continuation"),
        "{lazy_list_quote}"
    );
    assert!(
        lazy_list_quote.contains("lazy continuation\n: Jane"),
        "{lazy_list_quote}"
    );
    let ordinary_list_colon = render("- item\n: Jane", true);
    assert!(
        ordinary_list_colon.contains("<li>item\n: Jane</li>"),
        "{ordinary_list_colon}"
    );
    assert_eq!(
        render("- item\n  > Excerpt\n\n: outside", true),
        "<ul>\n<li>item\n<blockquote>\n<p>Excerpt</p>\n</blockquote>\n</li>\n</ul>\n<p>: outside</p>\n"
    );
    assert_eq!(
        render("> Excerpt\n\n\n: too far", true),
        "<blockquote>\n<p>Excerpt</p>\n</blockquote>\n<p>: too far</p>\n"
    );
}

#[test]
fn nested_quotes_attach_at_the_nearest_quote_boundary() {
    let html = render("> > Nested excerpt\n> : Inner author", true);
    assert!(
        html.contains("<blockquote>\n<figure>\n<blockquote>\n<p>Nested excerpt</p>"),
        "{html}"
    );
    assert!(
        html.contains("<figcaption>Inner author</figcaption>"),
        "{html}"
    );
}

#[test]
fn empty_escaped_or_malformed_attributions_remain_markdown() {
    let options = ParserOptions {
        blockquote_attributions: true,
        ..ParserOptions::gfm()
    };
    let html = render_with_options("> Quote\n: ", options.clone());
    assert!(!html.contains("<figure"), "{html}");
    assert!(html.contains("Quote\n:"), "{html}");
    for source in [
        "> Quote\n\\: escaped",
        "> Quote\n: Jane {#one #two}",
        "> Quote\n: {#only-id}",
    ] {
        let html = render_with_options(source, options.clone());
        assert!(!html.contains("<figure"), "{source:?}: {html}");
    }
}

#[test]
fn callouts_and_definition_lists_keep_their_existing_meaning() {
    let options = ParserOptions {
        blockquote_attributions: true,
        ..ParserOptions::gfm()
    };
    for source in [
        "> [!NOTE]\n> Body\n\n: Jane",
        "> [!note]\n> Body\n\n: Jane",
        "> \\[!NOTE]\n> Body\n\n: Jane",
    ] {
        let allocator = Allocator::new();
        let parsed = Parser::with_options(&allocator, source, options.clone())
            .parse()
            .unwrap();
        let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&parsed);
        assert!(!html.contains("<figure"), "{source:?}: {html}");
        assert!(html.contains("<blockquote"), "{source:?}: {html}");
        assert!(html.contains("ox-callout--note"), "{source:?}: {html}");
    }

    let allocator = Allocator::new();
    let parsed = Parser::with_options(
        &allocator,
        "> [!NOTE]\n> Body\n\n: Jane",
        ParserOptions {
            blockquote_attributions: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&parsed);
    assert!(!html.contains("<figure"), "{html}");
    assert!(!html.contains("<figcaption"), "{html}");
    assert!(html.ends_with("<p>: Jane</p>\n"), "{html}");

    let html = render("> [!UNKNOWN] ordinary quote\n: Jane", true);
    assert!(html.contains("<figure>"), "{html}");
    assert!(!html.contains("ox-callout"), "{html}");

    let options = ParserOptions {
        blockquote_attributions: true,
        definition_lists: true,
        ..ParserOptions::gfm()
    };
    let html = render_with_options("> Quote\n\nTerm\n: definition", options);
    assert!(html.contains("<blockquote>"), "{html}");
    assert!(html.contains("<dl class=\"ox-definition-list\">"), "{html}");
}

#[test]
fn quote_spans_cover_the_source_attribution_and_protected_blocks_stay_intact() {
    let source = "> ```md\n> : code\n> ```\n: Jane";
    let allocator = Allocator::new();
    let parsed = document(&allocator, source, ParserOptions::gfm());
    let Node::Figure(figure) = &parsed.children[0] else {
        panic!("expected figure");
    };
    assert_eq!(figure.span.source_text(source), source);
    assert_eq!(figure.caption[0].span().source_text(source), "Jane");
    assert!(
        HtmlRenderer::new()
            .render(&parsed)
            .contains("<code class=\"language-md\">: code\n</code>")
    );
}

#[test]
fn raw_html_math_and_mdx_inside_quotes_keep_their_boundaries() {
    let indented_source = ">     code\n>\n: Jane";
    let indented = render(indented_source, true);
    assert!(
        indented.contains("<pre><code>code\n</code></pre>"),
        "{indented}"
    );

    let html_source = "> <div>\n> quoted HTML\n> </div>\n: Jane";
    let html = render_with_options(
        html_source,
        ParserOptions {
            blockquote_attributions: true,
            ..ParserOptions::gfm()
        },
    );
    assert!(html.starts_with("<figure>\n<blockquote>\n<div>"), "{html}");
    assert!(html.contains("quoted HTML"), "{html}");
    assert!(html.contains("<figcaption>Jane</figcaption>"), "{html}");

    let math_source = "> $$\n> x + y\n> $$\n: Jane";
    let math = render_with_options(
        math_source,
        ParserOptions {
            blockquote_attributions: true,
            math: true,
            ..ParserOptions::gfm()
        },
    );
    assert!(math.starts_with("<figure>\n<blockquote>"), "{math}");
    assert!(math.contains("ox-math-block"), "{math}");
    assert!(math.contains("x + y"), "{math}");

    let mdx_source = "> <Chart title=\"quoted\" />\n: Jane";
    let mdx = render_with_options(
        mdx_source,
        ParserOptions {
            blockquote_attributions: true,
            mdx: true,
            ..ParserOptions::gfm()
        },
    );
    assert!(mdx.starts_with("<figure>\n<blockquote>"), "{mdx}");
    assert!(mdx.contains("Chart"), "{mdx}");
    assert!(mdx.contains("<figcaption>Jane</figcaption>"), "{mdx}");
}

#[test]
fn nested_callouts_keep_lazy_continuation_and_lists_stay_whole() {
    for source in [
        "- > [!NOTE]\n  > Body\n: Jane",
        "> > [!NOTE]\n> > Body\n: Jane",
    ] {
        let enabled = render(source, true);
        assert_eq!(enabled, render(source, false), "{source:?}");
        assert!(enabled.contains("Body\n: Jane"), "{enabled}");
        assert!(!enabled.contains("<figure"), "{enabled}");
    }

    let source = "* > q\n: Jane\n* next";
    let enabled = render(source, true);
    assert_eq!(enabled, render(source, false));
    assert_eq!(enabled.matches("<ul>").count(), 1, "{enabled}");
    assert!(enabled.contains("q\n: Jane"), "{enabled}");
}

#[test]
fn callout_decision_uses_parsed_content() {
    for source in [
        "> &#91;!NOTE]\n> Body\n\n: Jane",
        "> &#x5b;!NOTE]\n> Body\n\n: Jane",
        "> &lbrack;!NOTE]\n> Body\n\n: Jane",
    ] {
        let html = render(source, true);
        assert!(html.contains("ox-callout--note"), "{source:?}: {html}");
        assert!(!html.contains("<figure"), "{source:?}: {html}");
    }

    for source in [
        ">     [!NOTE]\n\n: Jane",
        ">     [!NOTE]\n: Jane",
        "> [!NOTE]: /x\n\n: Jane",
        "> [!NOTE]: /x\n: Jane",
        "> [!NOTE]\n> ===\n\n: Jane",
        "> [!NOTE]\n> ===\n: Jane",
        "[!note]: /u\n\n> [!NOTE]\n> Body\n\n: Jane",
        "[!note]: /u\n\n> [!NOTE]\n> Body\n: Jane",
    ] {
        let html = render(source, true);
        assert!(html.contains("<figure>"), "{source:?}: {html}");
        assert!(
            html.contains("<figcaption>Jane</figcaption>"),
            "{source:?}: {html}"
        );
    }
}

#[test]
fn empty_quote_cannot_receive_attribution() {
    let html = render(">\n: Jane", true);
    assert!(!html.contains("<figure"), "{html}");
    assert!(html.contains("<p>: Jane</p>"), "{html}");
}
