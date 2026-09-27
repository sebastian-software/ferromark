use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Span, Visit, walk_insertion};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{
    HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions,
};

fn render(source: &str, parser_options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .expect("fixture should parse");
    HtmlRenderer::new().render(&document)
}

fn insertion_options() -> ParserOptions {
    ParserOptions {
        insertions: true,
        ..ParserOptions::gfm()
    }
}

struct Hooks(bool);

impl HtmlRenderHooks for Hooks {
    fn render_node(
        &mut self,
        node: &Node<'_>,
        cx: &mut HtmlRenderContext<'_>,
    ) -> HtmlRenderControl {
        if let Node::Insertion(insertion) = node {
            self.0 = true;
            cx.write("<ins data-hook=\"yes\">");
            cx.render_nodes(&insertion.children, self);
            cx.write("</ins>");
            HtmlRenderControl::Handled
        } else {
            HtmlRenderControl::Default
        }
    }
}

struct Counter(usize);

impl<'a> Visit<'a> for Counter {
    fn visit_insertion(&mut self, insertion: &ferromark::ast::Insertion<'a>) {
        self.0 += 1;
        walk_insertion(self, insertion);
    }
}

struct Spans(Vec<Span>);

impl<'a> Visit<'a> for Spans {
    fn visit_insertion(&mut self, insertion: &ferromark::ast::Insertion<'a>) {
        self.0.push(insertion.span);
        walk_insertion(self, insertion);
    }
}

#[test]
fn insertion_syntax_is_opt_in_and_off_in_every_preset() {
    for options in [
        ParserOptions::default(),
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        assert!(!options.insertions);
        assert_eq!(
            render("++text++ and C++", options),
            "<p>++text++ and C++</p>\n"
        );
    }
    assert_eq!(
        render(
            "The timeout is ~~30 seconds~~ ++60 seconds++.",
            insertion_options()
        ),
        "<p>The timeout is <del>30 seconds</del> <ins>60 seconds</ins>.</p>\n"
    );
}

#[test]
fn insertions_use_commonmark_flanking_and_markdown_it_ins_run_consumption() {
    let cases = [
        ("++important++", "<p><ins>important</ins></p>\n"),
        ("++ foo ++", "<p>++ foo ++</p>\n"),
        ("word++new++word", "<p>word<ins>new</ins>word</p>\n"),
        ("(++added++).", "<p>(<ins>added</ins>).</p>\n"),
        ("++open", "<p>++open</p>\n"),
        ("close++", "<p>close++</p>\n"),
        ("C++", "<p>C++</p>\n"),
        ("+++added++", "<p>+<ins>added</ins></p>\n"),
        ("++added+++", "<p><ins>added</ins>+</p>\n"),
        ("+++added+++", "<p>+<ins>added</ins>+</p>\n"),
        ("++++", "<p>++++</p>\n"),
        (r"\+\+literal\+\+", "<p>++literal++</p>\n"),
    ];

    for (source, expected) in cases {
        assert_eq!(
            render(source, insertion_options()),
            expected,
            "source: {source:?}"
        );
    }
}

#[test]
fn insertion_bodies_parse_inline_markdown_and_soft_breaks() {
    assert_eq!(
        render(
            "++*important* [guide](/guide) `code`++",
            insertion_options(),
        ),
        "<p><ins><em>important</em> <a href=\"/guide\">guide</a> <code>code</code></ins></p>\n"
    );
    assert_eq!(
        render("++soft\nline++", insertion_options()),
        "<p><ins>soft\nline</ins></p>\n"
    );
    assert_eq!(
        render("++one\n\ntwo++", insertion_options()),
        "<p>++one</p>\n<p>two++</p>\n"
    );
    let list_html = render("- ++one++\n- ++two++", insertion_options());
    assert_eq!(list_html.matches("<ins>").count(), 2, "{list_html}");
}

#[test]
fn protected_contexts_and_untrusted_rendering_keep_their_policies() {
    assert_eq!(
        render("`++code++`", insertion_options()),
        "<p><code>++code++</code></p>\n"
    );
    assert_eq!(
        render(
            "<span title=\"++attribute++\">++text++</span>",
            insertion_options(),
        ),
        "<p><span title=\"++attribute++\"><ins>text</ins></span></p>\n"
    );
    assert_eq!(
        render("```\n++code block++\n```", insertion_options()),
        "<pre><code>++code block++\n</code></pre>\n"
    );
    let raw_block = render("<div>\n++raw html++\n</div>", insertion_options());
    assert!(raw_block.contains("++raw html++"), "{raw_block}");
    assert!(!raw_block.contains("<ins>"), "{raw_block}");
    assert_eq!(
        render(
            "[label](/++destination++ \"++title++\")",
            insertion_options()
        ),
        "<p><a href=\"/++destination++\" title=\"++title++\">label</a></p>\n"
    );

    let autolinks = ParserOptions {
        insertions: true,
        autolinks: true,
        ..ParserOptions::gfm()
    };
    let linked_url = render("https://example.com/++path++", autolinks);
    assert!(
        linked_url.contains("href=\"https://example.com/++path++\""),
        "{linked_url}"
    );
    assert!(!linked_url.contains("<ins>"), "{linked_url}");

    let math_options = ParserOptions {
        insertions: true,
        math: true,
        ..ParserOptions::default()
    };
    let math_allocator = Allocator::new();
    let math_document = Parser::with_options(&math_allocator, "$++math++$", math_options)
        .parse()
        .unwrap();
    let Node::Paragraph(math_paragraph) = &math_document.children[0] else {
        panic!("expected inline math paragraph");
    };
    assert!(matches!(
        math_paragraph.children.as_slice(),
        [Node::InlineMath(math)] if math.value == "++math++"
    ));

    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "++<script>alert(1)</script>++",
        insertion_options(),
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        sanitize: true,
        ..HtmlRendererOptions::new()
    })
    .render(&document);
    assert_eq!(
        html,
        "<p><ins>&lt;script&gt;alert(1)&lt;/script&gt;</ins></p>\n"
    );
}

#[test]
fn mdx_expressions_and_module_payloads_remain_opaque() {
    let options = ParserOptions {
        insertions: true,
        ..ParserOptions::mdx()
    };
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "{++expression++}", options.clone())
        .parse()
        .unwrap();
    assert!(matches!(
        document.children.as_slice(),
        [Node::MdxFlowExpression(expression)] if expression.value == "++expression++"
    ));

    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "Text {++expression++}", options.clone())
        .parse()
        .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected an MDX text-expression paragraph");
    };
    assert!(matches!(
        paragraph.children.as_slice(),
        [Node::Text(_), Node::MdxTextExpression(expression)]
            if expression.value == "++expression++"
    ));

    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "import value from \"++module++\";", options)
        .parse()
        .unwrap();
    assert!(matches!(
        document.children.as_slice(),
        [Node::MdxjsEsm(esm)] if esm.value == "import value from \"++module++\";"
    ));
}

#[test]
fn insertion_children_reach_headings_images_autolinks_and_hooks() {
    assert_eq!(
        render("# ++Important update++", insertion_options()),
        "<h1 id=\"important-update\"><ins>Important update</ins></h1>\n"
    );
    assert_eq!(
        render("![++alt text++](/image.png)", insertion_options()),
        "<p><img src=\"/image.png\" alt=\"alt text\"></p>\n"
    );

    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        "++https://example.com++",
        ParserOptions {
            insertions: true,
            autolinks: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::new().render(&document);
    assert!(
        html.contains(
            "<ins><a href=\"https://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.com</a></ins>"
        ),
        "{html}"
    );

    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, "++hooked++", insertion_options())
        .parse()
        .unwrap();
    let mut hooks = Hooks(false);
    assert_eq!(
        HtmlRenderer::new().render_with_hooks(&document, &mut hooks),
        "<p><ins data-hook=\"yes\">hooked</ins></p>\n"
    );
    assert!(hooks.0);
}

#[test]
fn insertion_nodes_have_delimiter_spans_and_visit_their_children() {
    let source = "before ++inserted **bold**++ after";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, insertion_options())
        .parse()
        .unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected paragraph");
    };
    let insertion = paragraph
        .children
        .iter()
        .find_map(|node| match node {
            Node::Insertion(insertion) => Some(insertion),
            _ => None,
        })
        .expect("the syntax creates an insertion node");
    assert_eq!(insertion.span.source_text(source), "++inserted **bold**++");
    let mut visitor = Counter(0);
    visitor.visit_document(&document);
    assert_eq!(visitor.0, 1);
}

#[test]
fn insertion_spans_remap_through_containers_and_table_cells() {
    let cases = [
        (
            "> ++quoted++",
            ParserOptions {
                insertions: true,
                ..ParserOptions::gfm()
            },
        ),
        (
            "| ++cell++ |\n| --- |\n",
            ParserOptions {
                insertions: true,
                tables: true,
                ..ParserOptions::default()
            },
        ),
    ];
    for (source, options) in cases {
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, options)
            .parse()
            .unwrap();
        let mut spans = Spans(Vec::new());
        spans.visit_document(&document);
        let source_spans = spans
            .0
            .iter()
            .map(|span| span.source_text(source))
            .collect::<Vec<_>>();
        let expected = if source.starts_with('>') {
            vec!["++quoted++"]
        } else {
            vec!["++cell++"]
        };
        assert_eq!(source_spans, expected);
    }
}
