use ferromark_allocator::Allocator;
use ferromark_ast::{Document, FrontMatter, FrontMatterKind, Span, Visit};
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions};

fn options() -> ParserOptions {
    ParserOptions {
        front_matter: true,
        ..ParserOptions::gfm()
    }
}

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Document<'a> {
    Parser::with_options(allocator, source, options())
        .parse()
        .unwrap()
}

fn render(source: &str, parser_options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document)
}

fn render_with_front_matter(source: &str) -> String {
    render(source, options())
}

#[test]
fn yaml_is_extracted_and_omitted_from_html() {
    let source = "---\ntitle: Hello\n---\n# Content\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let front_matter = document.front_matter.as_ref().expect("front matter");

    assert_eq!(front_matter.kind, FrontMatterKind::Yaml);
    assert_eq!(front_matter.value, "title: Hello\n");
    assert_eq!(front_matter.span, Span::new(0, 21));
    assert_eq!(front_matter.content_span, Span::new(4, 17));
    assert_eq!(
        front_matter.span.source_text(source),
        "---\ntitle: Hello\n---\n"
    );
    assert_eq!(
        front_matter.content_span.source_text(source),
        front_matter.value
    );
    assert_eq!(render_with_front_matter(source), "<h1>Content</h1>\n");
}

#[test]
fn renderer_hooks_and_incremental_paths_omit_front_matter() {
    struct Hooks {
        nodes: usize,
    }
    impl HtmlRenderHooks for Hooks {
        fn render_node(
            &mut self,
            _node: &ferromark_ast::Node<'_>,
            _cx: &mut ferromark_renderer::HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            self.nodes += 1;
            HtmlRenderControl::Default
        }
    }

    let source = "---\ntitle: Hello\n---\n# Content\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let expected = "<h1>Content</h1>\n";
    let body_allocator = Allocator::new();
    let body_document = Parser::with_options(&body_allocator, "# Content\n", ParserOptions::gfm())
        .parse()
        .unwrap();
    let mut baseline_renderer = HtmlRenderer::with_options(HtmlRendererOptions::gfm());
    let mut baseline_hooks = Hooks { nodes: 0 };
    assert_eq!(
        baseline_renderer.render_with_hooks(&body_document, &mut baseline_hooks),
        expected
    );

    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions::gfm());
    let mut hooks = Hooks { nodes: 0 };
    assert_eq!(renderer.render_with_hooks(&document, &mut hooks), expected);
    assert_eq!(hooks.nodes, baseline_hooks.nodes);

    let mut hooks = Hooks { nodes: 0 };
    assert_eq!(
        renderer.render_incremental_fragment_with_hooks(&document, &mut hooks),
        expected
    );
    assert_eq!(hooks.nodes, baseline_hooks.nodes);
    assert_eq!(renderer.render_provisional_fragment(&document), expected);
}

#[test]
fn toml_and_all_supported_line_endings_preserve_value_bytes() {
    for (source, value, closing_end) in [
        (
            "+++\ntitle = \"Hello\"\n+++\n# Content",
            "title = \"Hello\"\n",
            24,
        ),
        (
            "+++\r\ntitle = \"Hello\"\r\n+++\r\n# Content",
            "title = \"Hello\"\r\n",
            27,
        ),
        (
            "+++\rtitle = \"Hello\"\r+++\r# Content",
            "title = \"Hello\"\r",
            24,
        ),
    ] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        let front_matter = document.front_matter.as_ref().expect("front matter");

        assert_eq!(front_matter.kind, FrontMatterKind::Toml);
        assert_eq!(front_matter.value, value);
        assert_eq!(front_matter.span, Span::new(0, closing_end));
        assert_eq!(front_matter.content_span.source_text(source), value);
        assert!(render_with_front_matter(source).contains("<h1>Content</h1>"));
    }
}

#[test]
fn empty_front_matter_and_eof_closing_delimiter_are_supported() {
    for source in ["---\n---\nContent", "---\ntitle: x\n---"] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        let front_matter = document.front_matter.as_ref().expect("front matter");

        assert_eq!(front_matter.kind, FrontMatterKind::Yaml);
        assert_eq!(
            front_matter.value,
            if source == "---\n---\nContent" {
                ""
            } else {
                "title: x\n"
            }
        );
        assert_eq!(
            front_matter.content_span.len(),
            front_matter.value.len() as u32
        );
    }
}

#[test]
fn metadata_only_documents_keep_the_full_original_document_span() {
    let source = "---\ntitle: x\n---";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);

    assert!(document.children.is_empty());
    assert_eq!(document.span, Span::new(0, source.len() as u32));
}

#[test]
fn front_matter_is_disabled_in_every_parser_preset_by_default() {
    for parser_options in [
        ParserOptions::default(),
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        assert!(!parser_options.front_matter);
        let source = "---\ntitle: Hello\n---\n";
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, parser_options.clone())
            .parse()
            .unwrap();
        assert!(document.front_matter.is_none());
        assert_eq!(
            render(source, parser_options),
            "<hr>\n<h2>title: Hello</h2>\n"
        );
    }
}

#[test]
fn invalid_openers_closers_and_prefixes_fall_back_to_markdown() {
    for source in [
        "\n---\ntitle: Hello\n---\n# Content",
        " ---\ntitle: Hello\n---\n# Content",
        "----\ntitle: Hello\n----\n# Content",
        "++++\ntitle: Hello\n++++\n# Content",
        "---\ntitle: Hello\n+++\n# Content",
        "---\ntitle: Hello\nno closing",
        "---\ntitle: Hello\n ---\n# Content",
        "---\ntitle: Hello\n---x\n# Content",
    ] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        assert!(
            document.front_matter.is_none(),
            "unexpected front matter in {source:?}"
        );
        assert_eq!(
            render_with_front_matter(source),
            render(source, ParserOptions::gfm())
        );
    }
}

#[test]
fn delimiter_lines_allow_only_ascii_space_and_tab_after_three_markers() {
    for source in [
        "---  \ntitle: Hello\n---\t\n# Content",
        "+++  \ntitle = \"Hello\"\n+++\t\n# Content",
    ] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        assert!(
            document.front_matter.is_some(),
            "expected front matter in {source:?}"
        );
    }

    for source in [
        "---\ntitle: Hello\n---\u{00a0}\n# Content",
        "+++\ntitle = \"Hello\"\n+++x\n# Content",
        "---\ntitle: Hello\n----\n# Content",
    ] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        assert!(
            document.front_matter.is_none(),
            "unexpected front matter in {source:?}"
        );
    }
}

#[test]
fn value_is_an_exact_borrowed_slice_and_can_contain_nul() {
    let source = "---\nkey: before\0after\r\nblank:\r\n---\r\n# Content";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let front_matter = document.front_matter.as_ref().expect("front matter");

    assert_eq!(front_matter.value, "key: before\0after\r\nblank:\r\n");
    assert_eq!(
        front_matter.value.as_ptr(),
        source
            .as_ptr()
            .wrapping_add(front_matter.content_span.start as usize)
    );
    assert_eq!(
        front_matter.content_span.source_text(source),
        front_matter.value
    );
    assert_eq!(
        front_matter.span.source_text(source),
        "---\nkey: before\0after\r\nblank:\r\n---\r\n"
    );
}

#[test]
fn one_leading_bom_is_allowed_but_excluded_from_front_matter_span() {
    let source = "\u{feff}---\ntitle: Hello\n---\n# Content";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let front_matter = document.front_matter.as_ref().expect("front matter");

    assert_eq!(front_matter.span, Span::new(3, 24));
    assert_eq!(front_matter.content_span, Span::new(7, 20));
    assert_eq!(
        front_matter.span.source_text(source),
        "---\ntitle: Hello\n---\n"
    );
    assert_eq!(document.span, Span::new(0, source.len() as u32));
}

#[test]
fn a_second_front_matter_looking_block_is_ordinary_body_markdown() {
    let source = "---\nsite: docs\n---\n\n---\ninner\n---\n";
    let body = "---\ninner\n---\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);

    assert!(document.front_matter.is_some());
    assert_eq!(
        render_with_front_matter(source),
        render(body, ParserOptions::gfm())
    );
}

#[test]
fn a_bom_after_front_matter_is_body_content_and_is_not_stripped_again() {
    let source = "---\ntitle: docs\n---\n\u{feff}# Content\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);

    assert_eq!(
        render_with_front_matter(source),
        "<p>\u{feff}# Content</p>\n"
    );
    assert_eq!(document.span, Span::new(0, source.len() as u32));
}

#[test]
fn metadata_is_opaque_to_markdown_extensions_and_definitions() {
    let source = "---\n[ref]: /metadata\n[^note]: hidden\n// private\n<Widget />\n---\nUse [ref] and [^note] and [body].\n\n[body]: /body\n";
    let parser_options = ParserOptions {
        front_matter: true,
        line_comments: true,
        mdx: true,
        ..ParserOptions::gfm()
    };
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    assert!(document.front_matter.is_some());

    let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document);
    assert_eq!(
        html,
        "<p>Use [ref] and [^note] and <a href=\"/body\">body</a>.</p>\n"
    );
    assert!(!html.contains("metadata"));
    assert!(!html.contains("private"));
    assert!(!html.contains("Widget"));
}

#[test]
fn body_node_spans_keep_original_coordinates_after_prefix() {
    struct StrongSpans<'s>(&'s mut Vec<Span>);
    impl Visit<'_> for StrongSpans<'_> {
        fn visit_strong(&mut self, strong: &ferromark_ast::Strong<'_>) {
            self.0.push(strong.span);
        }
    }

    for source in [
        "\u{feff}---\ntitle: Hello\n---\n\n**visible**\n",
        "\u{feff}---\nkey: \0\n---\n\n\0 before **visible**\n",
        "---\ntitle: Hello\n---\n\n> \0 before **visible**\n",
    ] {
        let allocator = Allocator::new();
        let document = parse(&allocator, source);
        let mut spans = Vec::new();
        StrongSpans(&mut spans).visit_document(&document);
        let start = source.find("**visible**").unwrap() as u32;
        assert_eq!(spans, [Span::new(start, start + 11)]);
        assert_eq!(spans[0].source_text(source), "**visible**");
        assert_eq!(document.span, Span::new(0, source.len() as u32));
        if source.contains("before") {
            assert!(render_with_front_matter(source).contains("\u{fffd} before"));
        }
    }
}

#[test]
fn visitor_sees_front_matter_once_before_body() {
    struct Events<'s>(&'s mut Vec<&'static str>);
    impl Visit<'_> for Events<'_> {
        fn visit_front_matter(&mut self, _front_matter: &FrontMatter<'_>) {
            self.0.push("front_matter");
        }

        fn visit_heading(&mut self, _heading: &ferromark_ast::Heading<'_>) {
            self.0.push("heading");
        }
    }

    let source = "---\ntitle: Hello\n---\n# Content\n";
    let allocator = Allocator::new();
    let document = parse(&allocator, source);
    let mut events = Vec::new();
    Events(&mut events).visit_document(&document);
    assert_eq!(events, ["front_matter", "heading"]);
}

#[test]
fn front_matter_is_not_extracted_after_comments_or_inside_containers() {
    for source in [
        "// comment\n---\ntitle: Hello\n---\n",
        "<!-- comment -->\n---\ntitle: Hello\n---\n",
        "export const x = 1;\n\n---\ntitle: Hello\n---\n",
        "> ---\n> title: Hello\n> ---\n",
        "- ---\n  title: Hello\n  ---\n",
        "```\n---\ntitle: Hello\n---\n```\n",
        "    ---\n    title: Hello\n    ---\n",
        "<Widget>\n\n---\ntitle: Hello\n---\n\n</Widget>\n",
    ] {
        let enabled = ParserOptions {
            line_comments: true,
            mdx: true,
            ..options()
        };
        let disabled = ParserOptions {
            front_matter: false,
            ..enabled.clone()
        };
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, enabled.clone())
            .parse()
            .unwrap();
        assert!(
            document.front_matter.is_none(),
            "unexpected metadata in {source:?}"
        );
        assert_eq!(
            render(source, enabled),
            render(source, disabled),
            "{source:?}"
        );
    }
}
