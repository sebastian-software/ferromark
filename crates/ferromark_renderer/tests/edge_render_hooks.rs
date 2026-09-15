use ferromark_allocator::Allocator;
use ferromark_ast::Node;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{
    HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions,
    NoHtmlRenderHooks,
};

#[test]
fn render_hooks_can_skip_block_nodes() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            _cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Heading(_) => HtmlRenderControl::Handled,
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "# Hidden\n\nVisible")
        .parse()
        .unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert_eq!(html, "<p>Visible</p>\n");
}

#[test]
fn render_hooks_can_wrap_block_nodes() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Paragraph(paragraph) => {
                    cx.write("<section class=\"wrapped\"><p>");
                    cx.render_nodes(&paragraph.children, self);
                    cx.write("</p></section>\n");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "Wrapped <em>x</em>")
        .parse()
        .unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert_eq!(
        html,
        "<section class=\"wrapped\"><p>Wrapped <em>x</em></p></section>\n"
    );
}

#[test]
fn render_hooks_can_replace_inline_nodes_with_escape_helpers() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Strong(strong) => {
                    cx.write("<mark title=\"");
                    cx.write_attribute_escaped("\"strong\"");
                    cx.write("\">");
                    cx.render_nodes(&strong.children, self);
                    cx.write("</mark>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "A **bold** word").parse().unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert_eq!(
        html,
        "<p>A <mark title=\"&quot;strong&quot;\">bold</mark> word</p>\n"
    );
}

#[test]
fn render_hooks_preserve_inline_context_for_child_nodes() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Link(link) => {
                    cx.write("<span class=\"linked\">");
                    cx.render_nodes(&link.children, self);
                    cx.write("</span>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "Go [<em>x</em>](./x).")
        .parse()
        .unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert_eq!(
        html,
        "<p>Go <span class=\"linked\"><em>x</em></span>.</p>\n"
    );
}

#[test]
fn render_hooks_reach_callout_child_nodes() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Strong(strong) => {
                    cx.write("<mark>");
                    cx.render_nodes(&strong.children, self);
                    cx.write("</mark>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "> [!NOTE]\n> **watch**")
        .parse()
        .unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert!(html.contains("<p><mark>watch</mark></p>"), "{html}");
    assert!(!html.contains("<strong>"), "{html}");
}

#[test]
fn render_hooks_reach_named_mdx_child_nodes() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Paragraph(paragraph) => {
                    cx.write("<section>");
                    cx.render_nodes(&paragraph.children, self);
                    cx.write("</section>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "<Widget>\n\nchild\n\n</Widget>",
        ParserOptions::mdx(),
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert_eq!(
        html,
        "<div class=\"ox-island\" data-ox-island=\"Widget\"><section>child</section></div>\n"
    );
}

#[test]
fn render_hooks_reach_inline_math_inside_footnote_definitions() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::InlineMath(math) => {
                    cx.write("<span class=\"my-math\">");
                    cx.write_escaped(math.value);
                    cx.write("</span>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "text[^1]\n\n[^1]: footnote $x^2$ here\n",
        ParserOptions {
            math: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::new().render_with_hooks(&document, &mut Hooks);

    assert!(
        html.contains("<p>footnote <span class=\"my-math\">x^2</span> here</p>"),
        "{html}"
    );
    assert!(!html.contains("ox-math"), "{html}");
}

#[test]
fn render_hooks_reach_nested_blocks_inside_semantic_footnote_definitions() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Strong(strong) => {
                    cx.write("<mark>");
                    cx.render_nodes(&strong.children, self);
                    cx.write("</mark>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "note[^n]\n\n[^n]: > nested **value**\n",
        ParserOptions::gfm(),
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        semantic_footnotes: true,
        ..HtmlRendererOptions::default()
    })
    .render_with_hooks(&document, &mut Hooks);

    assert!(
        html.contains("<blockquote>\n<p>nested <mark>value</mark></p>\n</blockquote>"),
        "{html}"
    );
    assert!(!html.contains("<strong>value</strong>"), "{html}");
}

#[test]
fn render_hooks_work_for_incremental_fragments() {
    struct Hooks;

    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            node: &Node<'_>,
            cx: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            match node {
                Node::Strong(strong) => {
                    cx.write("<mark>");
                    cx.render_nodes(&strong.children, self);
                    cx.write("</mark>");
                    HtmlRenderControl::Handled
                }
                _ => HtmlRenderControl::Default,
            }
        }
    }

    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "**stream**").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render_incremental_fragment_with_hooks(&document, &mut Hooks);

    assert_eq!(html, "<p><mark>stream</mark></p>\n");
}

#[test]
fn empty_hooks_are_byte_identical_to_default_render() {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "# Title\n\nA [link](https://example.com).")
        .parse()
        .unwrap();
    let mut default_renderer = HtmlRenderer::new();
    let expected = default_renderer.render(&document);

    let mut hooked_renderer = HtmlRenderer::new();
    let actual = hooked_renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks);

    assert_eq!(actual, expected);
}

#[path = "spec_support/spec_txt.rs"]
mod spec_txt;

#[test]
fn highlighter_fallback_preserves_the_complete_spec_corpus() {
    for source in [
        include_str!("spec_fixtures/commonmark-0.31.2-spec.txt"),
        include_str!("spec_fixtures/gfm-extensions-spec.txt"),
    ] {
        for example in spec_txt::parse_spec(source) {
            assert!(!example.section.is_empty());
            // The independent conformance suite checks this reference output;
            // here we check that installing a hook cannot change any other node.
            for parser_options in [ParserOptions::commonmark(), ParserOptions::gfm_spec()] {
                let arena = Allocator::new();
                let document = Parser::with_options(&arena, &example.markdown, parser_options)
                    .parse()
                    .unwrap();
                for options in [
                    HtmlRendererOptions::commonmark(),
                    HtmlRendererOptions {
                        sanitize: true,
                        ..HtmlRendererOptions::gfm()
                    },
                    HtmlRendererOptions::default(),
                ] {
                    let expected = HtmlRenderer::with_options(options.clone()).render(&document);
                    let actual = HtmlRenderer::with_options(options)
                        .render_with_hooks(&document, &mut NoHtmlRenderHooks);
                    assert_eq!(
                        actual, expected,
                        "{} example {}; reference HTML: {:?}",
                        example.section, example.number, example.html
                    );
                }
            }
        }
    }
}

#[test]
fn highlighter_fallback_preserves_extended_documents_and_reused_state() {
    let sources = [
        "# Title {#custom}\n\n[TOC]\n\n# Title\n\n> [!NOTE]\n> **bold** and x^2^ H~2~O $math$\n\nTerm\n: Definition with *emphasis*.\n\n---\n\n$$\nx = 2\n$$\n",
        "| A | B |\n| :--- | ---: |\n| merged ||\n: A *caption* {#table .wide}\n\n- [x] Task\n- [ ] Next\n\n[^note]: Footnote with **bold**\n\nText[^note] and [[wiki|label]].",
        "import Notice from './notice.js'\n\n<Notice title={value}>\n\n**Body** and {expression}\n\n</Notice>\n\n{standalone}\n\nText <Badge>inline</Badge>.",
        "[internal](/guide.md) ![image](/image.png)\n\n<a href=\"/guide\">HTML</a>\n\nhttps://example.com\n\n```rust title=\"example\"\nlet value = 1;\n```\n",
    ];
    for sanitize in [false, true] {
        for semantic_footnotes in [false, true] {
            let options = HtmlRendererOptions {
                sanitize,
                semantic_footnotes,
                table_colgroup: true,
                table_column_names: true,
                heading_permalinks: true,
                convert_md_links: true,
                base_url: "/docs/".into(),
                ..HtmlRendererOptions::default()
            };
            let mut normal = HtmlRenderer::with_options(options.clone());
            let mut hooked = HtmlRenderer::with_options(options);
            for source in sources {
                let arena = Allocator::new();
                let document = Parser::with_options(
                    &arena,
                    source,
                    ParserOptions {
                        mdx: true,
                        math: true,
                        superscript: true,
                        subscript: true,
                        definition_lists: true,
                        heading_attributes: true,
                        wiki_links: true,
                        merged_table_cells: true,
                        table_attributes: true,
                        ..ParserOptions::gfm()
                    },
                )
                .parse()
                .unwrap();
                assert_eq!(
                    hooked.render_with_hooks(&document, &mut NoHtmlRenderHooks),
                    normal.render(&document),
                    "{source}"
                );
            }
        }
    }
}
