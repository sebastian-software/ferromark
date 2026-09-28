use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::{HtmlRenderer, OutlineOptions};

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    HtmlRenderer::new().render(&document)
}

struct Count(usize);

impl<'a> Visit<'a> for Count {
    fn visit_image(&mut self, _: &ferromark::ast::Image<'a>) {
        self.0 += 1;
    }
    fn visit_link(&mut self, link: &ferromark::ast::Link<'a>) {
        self.0 += 1;
        ferromark::ast::walk_link(self, link);
    }
}

#[test]
fn image_attributes_work_without_captions() {
    let options = ParserOptions {
        image_attributes: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render(
            "Logo ![Company](logo.svg){#company .logo .compact} here",
            options
        ),
        "<p>Logo <img src=\"logo.svg\" alt=\"Company\" id=\"company\" class=\"logo compact\"> here</p>\n"
    );
    let options = ParserOptions {
        image_attributes: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render(
            "![Company][logo]{.small}\n\n[logo]: logo.svg \"Title\"",
            options
        ),
        "<p><img src=\"logo.svg\" alt=\"Company\" title=\"Title\" class=\"small\"></p>\n"
    );
}

#[test]
fn ordinary_image_nodes_keep_attributes_unallocated() {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "![Alt](a.svg)", ParserOptions::gfm())
        .parse()
        .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph");
    };
    let Node::Image(image) = &paragraph.children[0] else {
        panic!("expected image");
    };
    assert!(image.attributes.is_none());
    assert!(std::mem::size_of::<ferromark::ast::Image<'static>>() <= 64);
}

#[test]
fn separate_caption_makes_figure_and_keeps_alt_title_independent() {
    for gap in ["", "\n"] {
        let source = format!(
            "![Three components](pipeline.svg \"Browser title\"){{.diagram}}\n{gap}: The **pipeline** {{#pipeline .wide}}"
        );
        let options = ParserOptions {
            image_attributes: true,
            image_captions: true,
            ..ParserOptions::gfm()
        };
        assert_eq!(
            render(&source, options),
            "<figure id=\"pipeline\" class=\"wide\">\n<img src=\"pipeline.svg\" alt=\"Three components\" title=\"Browser title\" class=\"diagram\">\n<figcaption>The <strong>pipeline</strong></figcaption>\n</figure>\n",
            "{source:?}"
        );
    }
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render("![](empty.svg)\n\n: Visible caption", options),
        "<figure>\n<img src=\"empty.svg\" alt=\"\">\n<figcaption>Visible caption</figcaption>\n</figure>\n"
    );
}

#[test]
fn caption_requires_one_standalone_image_and_opt_in() {
    let source = "![Alt](a.png) and prose\n: source";
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render(source, options),
        "<p><img src=\"a.png\" alt=\"Alt\"> and prose\n: source</p>\n"
    );
    let options = ParserOptions::gfm();
    assert_eq!(
        render("![Alt](a.png)\n\n: source", options),
        "<p><img src=\"a.png\" alt=\"Alt\"></p>\n<p>: source</p>\n"
    );
}

#[test]
fn captions_respect_containers_and_definition_list_precedence() {
    let options = ParserOptions {
        image_captions: true,
        definition_lists: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render("![Alt](a.png)\n: A caption", options.clone()),
        "<figure>\n<img src=\"a.png\" alt=\"Alt\">\n<figcaption>A caption</figcaption>\n</figure>\n"
    );
    let html = render(
        "![a\nb](c)\n: cap",
        ParserOptions {
            image_captions: true,
            definition_lists: true,
            ..ParserOptions::gfm()
        },
    );
    assert!(html.starts_with("<figure>\n<img "), "{html}");
    assert!(html.contains("<figcaption>cap</figcaption>"), "{html}");
    let html = render("![a](c)\n\n: cap", options);
    assert!(html.starts_with("<figure>\n<img "), "{html}");
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render("> ![Alt](a.png)\n>\n> : In the quote", options),
        "<blockquote>\n<figure>\n<img src=\"a.png\" alt=\"Alt\">\n<figcaption>In the quote</figcaption>\n</figure>\n</blockquote>\n"
    );
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render("![Alt](a.png)\n\n> : Outside", options),
        "<p><img src=\"a.png\" alt=\"Alt\"></p>\n<blockquote>\n<p>: Outside</p>\n</blockquote>\n"
    );
}

#[test]
fn unfinished_multiline_images_do_not_trigger_later_captions() {
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    for source in [
        "![a\n: b](c)\n: cap",
        "![a](b \"x\n: y\")\n: cap",
        "![[a](b) x\n: c](d)\n: cap",
    ] {
        let html = render(source, options.clone());
        assert!(!html.contains("<figure>"), "{source:?}: {html}");
    }
}

#[test]
fn a_captioned_image_keeps_a_blank_separated_list_item_loose() {
    let html = render(
        "- intro\n\n  ![Alt](a.png)\n  : Caption",
        ParserOptions {
            image_captions: true,
            ..ParserOptions::gfm()
        },
    );
    assert!(html.contains("<li><p>intro</p>\n<figure>"), "{html}");
}

#[test]
fn caption_lines_can_follow_a_lazy_container_continuation() {
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    let quote = render("> ![Alt](a.png)\n: Caption", options.clone());
    assert!(quote.contains("<blockquote>\n<figure>"), "{quote}");
    let list = render("- ![Alt](a.png)\n: Caption", options);
    assert!(list.contains("<li>\n<figure>"), "{list}");
}

#[test]
fn image_attribute_suffixes_do_not_cross_lines_or_nested_openers() {
    let options = ParserOptions {
        image_attributes: true,
        ..ParserOptions::gfm()
    };
    let html = render("![Alt](a.svg){#x\n.y}", options.clone());
    assert!(html.contains("<img src=\"a.svg\" alt=\"Alt\">"), "{html}");
    assert!(!html.contains("id=\"x\""), "{html}");
    let html = render("![Alt](a.svg){{#x}", options);
    assert!(html.contains("<img src=\"a.svg\" alt=\"Alt\">"), "{html}");
    assert!(!html.contains("id=\"x\""), "{html}");
}

#[test]
fn figure_ast_visits_caption_and_preserves_spans() {
    let source = "![Alt](a.png)\n\n: A [source](https://example.org)";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            image_captions: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let Node::Figure(figure) = &document.children[0] else {
        panic!("expected figure");
    };
    assert_eq!(figure.span.end as usize, source.len());
    assert!(matches!(figure.content, Node::Image(_)));
    let mut count = Count(0);
    count.visit_document(&document);
    assert_eq!(count.0, 2);
}

#[test]
fn figure_and_image_ids_share_heading_collision_planning() {
    let source = "![Alt](a.svg){#shared}\n: Caption {#shared}\n\n# Heading {#shared}";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            image_attributes: true,
            image_captions: true,
            heading_attributes: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::new().render(&document);
    assert!(html.contains("<figure id=\"shared\">"));
    assert!(html.contains("<img src=\"a.svg\" alt=\"Alt\" id=\"shared-1\">"));
    assert!(html.contains("<h1 id=\"shared-2\">Heading</h1>"));
    assert_eq!(
        document.outline(&OutlineOptions::default())[0]
            .id
            .as_deref(),
        Some("shared-2")
    );
}

#[test]
fn invalid_attribute_blocks_remain_literal_and_values_are_escaped() {
    let options = ParserOptions {
        image_attributes: true,
        image_captions: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render("![Alt](a.svg){#one #two}", options.clone()),
        "<p><img src=\"a.svg\" alt=\"Alt\">{#one #two}</p>\n"
    );
    assert_eq!(
        render("![Alt](a.svg){.a&b}", options.clone()),
        "<p><img src=\"a.svg\" alt=\"Alt\" class=\"a&amp;b\"></p>\n"
    );
    assert_eq!(
        render("![Alt](a.svg)\n: Caption {#one #two}", options),
        "<p><img src=\"a.svg\" alt=\"Alt\">\n: Caption {#one #two}</p>\n"
    );
}

#[test]
fn caption_does_not_attach_to_linked_image_or_protected_text() {
    let options = ParserOptions {
        image_captions: true,
        ..ParserOptions::gfm()
    };
    assert_eq!(
        render(
            "[![Alt](a.svg)](/target)\n\n: Not a caption",
            options.clone()
        ),
        "<p><a href=\"/target\"><img src=\"a.svg\" alt=\"Alt\"></a></p>\n<p>: Not a caption</p>\n"
    );
    assert_eq!(
        render(
            "```md\n![Alt](a.svg)\n: Not a caption\n```",
            options.clone()
        ),
        "<pre><code class=\"language-md\">![Alt](a.svg)\n: Not a caption\n</code></pre>\n"
    );
    assert_eq!(
        render("![Alt](a.svg)\n\n\\: Not a caption", options),
        "<p><img src=\"a.svg\" alt=\"Alt\"></p>\n<p>: Not a caption</p>\n"
    );
}
