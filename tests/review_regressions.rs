//! Regression tests for DoS scaling bounds, parser/renderer edge cases,
//! projection, and JSX fixes identified during the project audit.

use std::time::{Duration, Instant};

use ferromark::ast::{Document, Heading, Node, Span, Text};
use ferromark::{
    Allocator, HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer,
    HtmlRendererOptions, JsxRenderer, JsxRendererOptions, MarkdownProjectionOptions, Parser,
    ParserOptions, html_to_jsx,
};

fn parse_and_render(source: &str, parser_options: ParserOptions) -> String {
    let arena = Allocator::for_source_len(source.len());
    let doc = Parser::with_options(&arena, source, parser_options)
        .parse()
        .expect("parse should succeed");
    HtmlRenderer::new().render(&doc)
}

#[test]
fn unclosed_nested_images_stay_linear() {
    let source = format!("{}{}", "![".repeat(40), "]".repeat(40));
    let start = Instant::now();
    let html = parse_and_render(&source, ParserOptions::default());
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "nested unclosed images took {:?}",
        start.elapsed()
    );
    assert!(html.contains("!["));
}

#[test]
fn inline_scanner_edge_cases_stay_linear() {
    // Distinct backtick widths (1..=128) ensure every run is genuinely unmatched,
    // exercising both the 1..=64 bitmask and the >64 overflow SmallVec.
    let unmatched_distinct_backticks = (1..=128)
        .map(|n| "`".repeat(n))
        .collect::<Vec<_>>()
        .join(" ");
    let cases = [
        ("unclosed_lt", format!("{}>", "<".repeat(20_000))),
        ("unclosed_comment", format!("{}>", "<!-- ".repeat(5_000))),
        ("unclosed_pi", format!("{}>", "<? ".repeat(5_000))),
        ("unclosed_cdata", format!("{}>", "<![CDATA[ ".repeat(5_000))),
        ("unmatched_backticks_1_to_128", unmatched_distinct_backticks),
        (
            "unmatched_backticks_1_64_65",
            format!("{} {} {} tail", "`", "`".repeat(64), "`".repeat(65)),
        ),
        ("digit_prefixed_math", format!("{}$", "$1 ".repeat(5_000))),
        ("many_gfm_autolinks", "a@b.c ".repeat(5_000)),
        (
            "mdx_unclosed_backticks",
            format!("<A>`{}</A>", "` ".repeat(5_000)),
        ),
        (
            "mdx_single_line_inline_code_spans",
            format!("<Box>{}</Box>", "`x` ".repeat(10_000)),
        ),
        (
            "mdx_single_line_triple_backtick_spans",
            format!("<Box>{}</Box>", "```x``` ".repeat(5_000)),
        ),
        (
            "mdx_repeated_unclosed_fences",
            format!("<Box>\n{}</Box>\n", "```js\n".repeat(5_000)),
        ),
        (
            "mdx_repeated_unclosed_fences_with_overindented_closer",
            format!("<Box>\n{}        ```\n</Box>\n", "```js\n".repeat(5_000)),
        ),
    ];

    let mut options = ParserOptions::gfm();
    options.math = true;
    options.mdx = true;
    for (name, source) in cases {
        let start = Instant::now();
        let arena = Allocator::for_source_len(source.len());
        let doc = Parser::with_options(&arena, &source, options.clone())
            .parse()
            .unwrap_or_else(|err| panic!("case `{name}` failed to parse: {err:?}"));
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "case `{name}` took {:?}",
            start.elapsed()
        );
        assert!(!doc.children.is_empty(), "case `{name}` produced empty AST");
        if name.starts_with("unmatched_backticks") {
            let Node::Paragraph(p) = &doc.children[0] else {
                panic!("case `{name}` expected paragraph");
            };
            assert!(
                p.children.iter().all(|node| matches!(node, Node::Text(_))),
                "case `{name}` unexpectedly formed inline code"
            );
        }
    }
}

#[test]
fn inline_math_closer_cache_handles_interleaved_code_spans() {
    let source = "$a `b$` c$ $d$ and $1 `code` $2 `x$` $3";
    let arena = Allocator::for_source_len(source.len());
    let opts = ParserOptions {
        math: true,
        ..ParserOptions::gfm()
    };
    let doc = Parser::with_options(&arena, source, opts)
        .parse()
        .expect("parse should succeed");
    let html = HtmlRenderer::new().render(&doc);
    assert!(
        html.contains("data-ox-tex=\"a `b$` c\""),
        "code span inside math should not close math early: {html}"
    );
    assert!(
        html.contains("data-ox-tex=\"d\""),
        "subsequent math after cached window should still parse: {html}"
    );
}

#[test]
fn mdx_flow_code_fence_closed_by_longer_fence_preserves_outer_element() {
    for source in [
        "<Box>\n```\n</Box>\n````\n</Box>\n",
        "<Box>\n    ```\n    </Box>\n    ````\n</Box>\n",
    ] {
        let arena = Allocator::for_source_len(source.len());
        let opts = ParserOptions {
            mdx: true,
            ..ParserOptions::default()
        };
        let doc = Parser::with_options(&arena, source, opts)
            .parse()
            .expect("parse should succeed");
        assert_eq!(doc.children.len(), 1, "outer <Box> should wrap the fence");
        let Node::MdxJsxFlowElement(el) = &doc.children[0] else {
            panic!("expected MdxJsxFlowElement, got {:?}", doc.children[0]);
        };
        assert_eq!(el.name, Some("Box"));
        assert_eq!(el.span, Span::new(0, source.len() as u32));
    }
}

#[test]
fn mdx_flow_indented_code_block_with_backticks_preserves_outer_element() {
    let source = "<Box>\nparagraph\n\n    ```\n</Box>\n";
    let arena = Allocator::for_source_len(source.len());
    let opts = ParserOptions {
        mdx: true,
        ..ParserOptions::default()
    };
    let doc = Parser::with_options(&arena, source, opts)
        .parse()
        .expect("parse should succeed");
    assert_eq!(
        doc.children.len(),
        1,
        "indented code block must not swallow </Box>"
    );
    let Node::MdxJsxFlowElement(el) = &doc.children[0] else {
        panic!("expected MdxJsxFlowElement, got {:?}", doc.children[0]);
    };
    assert_eq!(el.name, Some("Box"));
    assert_eq!(el.span, Span::new(0, source.len() as u32));
    assert!(
        matches!(el.children.first(), Some(Node::Paragraph(_))),
        "expected leading paragraph inside <Box>: {:?}",
        el.children
    );
    assert!(
        matches!(el.children.get(1), Some(Node::CodeBlock(_))),
        "expected indented code block inside <Box>: {:?}",
        el.children
    );
}

#[test]
fn mdx_nested_component_fence_with_outer_closing_tag_preserves_outer_element() {
    let source =
        "<Outer>\npara\n\n    <Inner>\n    ```\n    </Outer>\n    ````\n    </Inner>\n</Outer>\n";
    let arena = Allocator::for_source_len(source.len());
    let opts = ParserOptions {
        mdx: true,
        ..ParserOptions::default()
    };
    let doc = Parser::with_options(&arena, source, opts)
        .parse()
        .expect("parse should succeed");
    assert_eq!(
        doc.children.len(),
        1,
        "outer <Outer> must not close inside nested fence"
    );
    let Node::MdxJsxFlowElement(outer) = &doc.children[0] else {
        panic!("expected MdxJsxFlowElement, got {:?}", doc.children[0]);
    };
    assert_eq!(outer.name, Some("Outer"));
    assert_eq!(outer.span, Span::new(0, source.len() as u32));
}

#[test]
fn strict_mdx_single_line_triple_backticks_stay_linear() {
    let source = format!("<Box>{}</Box>\n", "```x``` ".repeat(5_000));
    let start = Instant::now();
    let arena = Allocator::for_source_len(source.len());
    let opts = ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::default()
    };
    let doc = Parser::with_options(&arena, &source, opts)
        .parse()
        .expect("parse should succeed");
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "strict MDX single-line backticks took {:?}",
        start.elapsed()
    );
    assert_eq!(doc.children.len(), 1);
}

#[test]
fn gfm_autolink_reference_ranges_stay_exact_after_entity_expansion() {
    for source in ["&nGt;&nGt;&nGt; a@b.c tail", "&amp;éé a@b.c"] {
        let arena = Allocator::for_source_len(source.len());
        let doc = Parser::with_options(&arena, source, ParserOptions::gfm())
            .parse()
            .expect("parse should succeed");
        let refs = doc.collect_references(source);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].destination, "mailto:a@b.c");
        assert_eq!(refs[0].span.unwrap().source_text(source), "a@b.c");
        let html = HtmlRenderer::new().render(&doc);
        assert!(html.contains("mailto:a@b.c"));
    }
}

#[test]
fn renderer_autolink_handles_prefix_only_and_punctuation_prefixes() {
    let opts = HtmlRendererOptions {
        autolink_urls: true,
        link_target_blank: false,
        autolink_patterns: vec!["http://".into(), "mailto:".into(), ":".into()].into(),
        ..HtmlRendererOptions::default()
    };

    for input in ["http://.", "mailto:.", ":.", "http://example.com."] {
        let arena = Allocator::for_source_len(input.len());
        let doc = Parser::new(&arena, input).parse().expect("parse");
        let html = HtmlRenderer::with_options(opts.clone()).render(&doc);
        if input == "http://example.com." {
            assert!(
                html.contains("href=\"http://example.com\"")
                    && html.contains(">http://example.com</a>."),
                "unexpected autolink output: {html}"
            );
        } else {
            assert!(
                !html.contains("<a "),
                "input {input:?} should not autolink: {html}"
            );
        }
    }
}

#[test]
fn provisional_render_preserves_explicit_and_footnote_ids_when_heading_ids_disabled() {
    let parser_opts = ParserOptions {
        footnotes: true,
        ..ParserOptions::gfm()
    };
    let render_opts = HtmlRendererOptions {
        heading_ids: false,
        ..HtmlRendererOptions::default()
    };

    let mut renderer = HtmlRenderer::with_options(render_opts);

    let committed = "Intro[^1]\n\n[^1]: Note";
    let provisional = "Draft[^1]\n\n[^1]: Draft note";
    let final_frag = "Next[^2]\n\n[^2]: Second";

    let a1 = Allocator::for_source_len(committed.len());
    let d1 = Parser::with_options(&a1, committed, parser_opts.clone())
        .parse()
        .expect("parse");
    let _ = renderer.render_incremental_fragment(&d1);

    let a2 = Allocator::for_source_len(provisional.len());
    let d2 = Parser::with_options(&a2, provisional, parser_opts.clone())
        .parse()
        .expect("parse");
    let prov_html = renderer.render_provisional_fragment(&d2);
    assert!(prov_html.contains("fn-1"));

    // A second identical provisional render must yield identical IDs (state was restored).
    let prov_html_2 = renderer.render_provisional_fragment(&d2);
    assert_eq!(prov_html, prov_html_2);

    let a3 = Allocator::for_source_len(final_frag.len());
    let d3 = Parser::with_options(&a3, final_frag, parser_opts)
        .parse()
        .expect("parse");
    let final_html = renderer.render_incremental_fragment(&d3);
    assert!(final_html.contains("fn-2"));
}

#[test]
fn code_hooks_do_not_invoke_render_node_on_tight_list_paragraphs() {
    struct CountingHooks {
        node_calls: usize,
    }
    impl HtmlRenderHooks for CountingHooks {
        fn render_node(
            &mut self,
            _node: &Node<'_>,
            _context: &mut HtmlRenderContext<'_>,
        ) -> HtmlRenderControl {
            self.node_calls += 1;
            HtmlRenderControl::Default
        }
    }

    let source = "- one\n- two\n";
    let arena = Allocator::for_source_len(source.len());
    let doc = Parser::new(&arena, source).parse().expect("parse");
    let mut hooks = CountingHooks { node_calls: 0 };
    let html = HtmlRenderer::new().render_with_code_hooks(&doc, &mut hooks);
    assert_eq!(hooks.node_calls, 0);
    assert_eq!(html, "<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n");
}

#[test]
fn rewrite_html_root_urls_skips_comments_and_quoted_non_url_attributes() {
    let source = concat!(
        "<!-- <a href=\"/guide\">ignored</a> -->\n\n",
        "<div><!--><a href=\"/guide\">empty1</a></div>\n\n",
        "<div><!---><a href=\"/guide\">empty2</a></div>\n\n",
        "<div><!-- comment --!><a href=\"/guide\">abrupt</a></div>\n\n",
        "<div><textarea>1 < 2. That's fine.</textarea><a href=\"/guide\">after-ta</a></div>\n\n",
        "<div><svg><title/></svg><a href=\"/guide.md\">after-svg</a></div>\n\n",
        "<div><script.demo-box><a href=\"/guide.md\">after-dotted</a></script.demo-box></div>\n\n",
        "<p>1 < 2. That's fine. <a href=\"/guide\">after-lt</a></p>\n\n",
        "<a title=\"see href=/guide\" href=\"/guide\">link</a>\n\n",
        "<a title=\"a > b\" href=\"/guide\">gt</a>\n"
    );
    let arena = Allocator::for_source_len(source.len());
    let doc = Parser::new(&arena, source).parse().expect("parse");
    let opts = HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/docs".into(),
        ..HtmlRendererOptions::default()
    };
    let html = HtmlRenderer::with_options(opts).render(&doc);

    assert!(
        html.contains("<!-- <a href=\"/guide\">ignored</a> -->"),
        "HTML comment should remain untouched: {html}"
    );
    assert!(
        html.contains("<div><!--><a href=\"/docs/guide\">empty1</a></div>"),
        "empty `<!-->` comment should close immediately before anchor: {html}"
    );
    assert!(
        html.contains("<div><!---><a href=\"/docs/guide\">empty2</a></div>"),
        "empty `<!--->` comment should close immediately before anchor: {html}"
    );
    assert!(
        html.contains("<div><!-- comment --!><a href=\"/docs/guide\">abrupt</a></div>"),
        "HTML5 `--!>` comment closer should not suppress following anchor rewrite: {html}"
    );
    assert!(
        html.contains(
            "<div><textarea>1 < 2. That's fine.</textarea><a href=\"/docs/guide\">after-ta</a></div>"
        ),
        "raw-text `<textarea>` containing `<` and apostrophe should not suppress following anchor rewrite: {html}"
    );
    assert!(
        html.contains(
            "<div><svg><title/></svg><a href=\"/docs/guide/index.html\">after-svg</a></div>"
        ),
        "self-closing `<title/>` inside SVG should not wait for `</title>`: {html}"
    );
    assert!(
        html.contains(
            "<div><script.demo-box><a href=\"/docs/guide/index.html\">after-dotted</a></script.demo-box></div>"
        ),
        "dotted custom tag `<script.demo-box>` should not be truncated to `<script>`: {html}"
    );
    assert!(
        html.contains("<p>1 < 2. That's fine. <a href=\"/docs/guide\">after-lt</a></p>"),
        "bare `<` comparison in text followed by apostrophe should not suppress following anchor rewrite: {html}"
    );
    assert!(
        html.contains("<a title=\"see href=/guide\" href=\"/docs/guide\">link</a>"),
        "quoted title attribute should not confuse URL rewriting: {html}"
    );
    assert!(
        html.contains("<a title=\"a > b\" href=\"/docs/guide\">gt</a>"),
        "quoted `>` inside attribute should not terminate tag early: {html}"
    );
}

#[test]
fn table_cell_image_alt_content_spans_are_remapped() {
    let source = "| col |\n| --- |\n| ![*em* \\| x](img.png) |\n";
    let arena = Allocator::for_source_len(source.len());
    let mut opts = ParserOptions::gfm();
    opts.image_alt_content = true;
    let doc = Parser::with_options(&arena, source, opts)
        .parse()
        .expect("parse");
    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table");
    };
    let cell = &table.children[1].children[0];
    let Node::Image(img) = &cell.children[0] else {
        panic!("expected image");
    };
    let cell_start = cell.span.start;
    let cell_end = cell.span.end;
    let alt_content = img.alt_content.as_ref().expect("expected alt_content");
    assert!(!alt_content.children.is_empty());
    assert_eq!(alt_content.children[0].span().source_text(source), "*em*");
    for alt_node in &alt_content.children {
        let span = alt_node.span();
        assert!(
            span.start >= cell_start && span.end <= cell_end,
            "alt_content span {span:?} outside cell span {cell_start}..{cell_end}"
        );
    }
}

#[test]
fn projection_preserves_tight_nested_inline_jsx_and_ignores_empty_heading_span() {
    let source = "**<A><B>text</B></A>**\n";
    let arena = Allocator::for_source_len(source.len());
    let opts = ParserOptions {
        mdx: true,
        ..ParserOptions::default()
    };
    let doc = Parser::with_options(&arena, source, opts)
        .parse()
        .expect("parse");
    let projected = doc
        .project_markdown(source, &MarkdownProjectionOptions::default())
        .expect("projection");
    assert_eq!(projected.markdown, "**text**\n");

    // Synthetic heading with empty span must not claim an omitted_title_heading span.
    let arena2 = Allocator::new();
    let mut text_nodes = arena2.new_vec_with_capacity(1);
    text_nodes.push(Node::Text(Text {
        value: "Title",
        span: Span::empty(),
    }));
    let mut children = arena2.new_vec_with_capacity(1);
    children.push(Node::Heading(arena2.boxed(Heading {
        depth: 1,
        children: text_nodes,
        attributes: None,
        span: Span::empty(),
    })));
    let synthetic = Document {
        children,
        front_matter: None,
        span: Span::empty(),
    };
    let projected_empty = synthetic
        .project_markdown(
            "",
            &MarkdownProjectionOptions {
                omit_front_matter: false,
                omit_title_heading: Some("Title".into()),
            },
        )
        .expect("projection");
    assert_eq!(projected_empty.omitted_title_heading, None);
}

#[test]
fn jsx_preserves_standard_attrs_with_data_siblings_and_decodes_entities_strictly() {
    // Verify HTML5 numeric entity decoding through html_to_jsx: preserves leading
    // zeros (`&#00000065;` -> A, `&#x00000042;` -> B) in text and unquoted attributes,
    // rejects signs (`&#+65;`), and maps overflow/NUL to U+FFFD.
    let entities_jsx = html_to_jsx(
        "<p title=&#00000065;&#x00000042;>&#00000065; &#x00000042; &#+65; &#9999999; &#0; &amp;&lt;&gt;&quot;</p>",
    );
    assert!(
        entities_jsx.contains("title={\"AB\"}"),
        "leading-zero numeric entities in unquoted attribute should decode: {entities_jsx}"
    );
    assert!(
        entities_jsx.contains("A B &#+65; \u{fffd} \u{fffd} &<>\\\""),
        "unexpected entity decoding output: {entities_jsx}"
    );

    let source = "[x]{title=\"a\" data-title=\"b\"}\n";
    let arena = Allocator::for_source_len(source.len());
    let opts = ParserOptions {
        bracketed_spans: true,
        ..ParserOptions::default()
    };
    let doc = Parser::with_options(&arena, source, opts)
        .parse()
        .expect("parse");
    let out = JsxRenderer::with_options(JsxRendererOptions::default()).render(&doc, source);
    assert!(
        out.body.contains("title={\"a\"}"),
        "standard `title` attr dropped: {}",
        out.body
    );
    assert!(
        out.body.contains("data-title={\"b\"}"),
        "data-title attr missing: {}",
        out.body
    );

    let svg_jsx = html_to_jsx("<svg><feGaussianBlur stdDeviation=\"2\" /><br></svg>");
    assert!(svg_jsx.contains("<feGaussianBlur stdDeviation=\"2\" />"));
    assert!(svg_jsx.contains("<br />"));
}
