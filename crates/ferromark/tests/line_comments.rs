use ferromark::allocator::Allocator;
use ferromark::ast::{Span, Visit};
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks};

fn options() -> ParserOptions {
    ParserOptions {
        line_comments: true,
        ..ParserOptions::gfm()
    }
}

fn render_with_options(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    HtmlRenderer::with_options(renderer_options).render(&document)
}

fn render(source: &str) -> String {
    render_with_options(source, options(), HtmlRendererOptions::gfm())
}

struct StrongSpans {
    spans: Vec<Span>,
}

impl<'a> Visit<'a> for StrongSpans {
    fn visit_strong(&mut self, strong: &ferromark::ast::Strong<'a>) {
        self.spans.push(strong.span);
    }
}

fn first_strong_span(document: &ferromark::ast::Document<'_>) -> Span {
    let mut visitor = StrongSpans { spans: Vec::new() };
    visitor.visit_document(document);
    visitor
        .spans
        .into_iter()
        .next()
        .expect("expected a strong node")
}

fn line_ending_variants(source: &str) -> [String; 3] {
    [
        source.to_string(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
    ]
}

#[test]
fn line_comments_are_disabled_by_default() {
    for preset in [
        ParserOptions::default(),
        ParserOptions::commonmark(),
        ParserOptions::gfm(),
        ParserOptions::gfm_spec(),
        ParserOptions::mdx(),
    ] {
        assert!(!preset.line_comments);
    }
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "// private note").parse().unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&document);
    assert_eq!(html, "<p>// private note</p>\n");
}

#[test]
fn comments_at_line_start_are_transparent_and_need_no_space() {
    assert_eq!(
        render("first\n//one\n // two\n  // three\n   //\nsecond\n"),
        "<p>first\nsecond</p>\n"
    );
    assert_eq!(render("// first\n   // second\n//\n"), "");
}

#[test]
fn explicit_blank_lines_still_separate_paragraphs() {
    assert_eq!(
        render("first\n\n// private\n\nsecond\n"),
        "<p>first</p>\n<p>second</p>\n"
    );
}

#[test]
fn trailing_comments_outside_paragraph_content_do_not_change_html() {
    assert_eq!(
        render("first\n// trailing\n\nsecond\n"),
        "<p>first</p>\n<p>second</p>\n"
    );
    assert_eq!(
        render("first\n// trailing\n# heading\n"),
        "<p>first</p>\n<h1>heading</h1>\n"
    );
    assert_eq!(render("first\n// trailing"), "<p>first</p>\n");
}

#[test]
fn ordinary_slashes_escaped_markers_and_inline_code_remain_text() {
    assert_eq!(
        render("https://example.com\nText // ordinary\n\\// escaped\n`// inline`\n"),
        "<p><a href=\"https://example.com\">https://example.com</a>\nText // ordinary\n// escaped\n<code>// inline</code></p>\n"
    );
}

#[test]
fn slash_heavy_prose_without_eligible_comments_preserves_text() {
    assert_eq!(
        render("https://example.com/a//b\npath /// suffix\nText // ordinary\n"),
        "<p><a href=\"https://example.com/a//b\">https://example.com/a//b</a>\npath /// suffix\nText // ordinary</p>\n"
    );
}

#[test]
fn tabs_and_four_space_indentation_are_not_comments() {
    let html = render("   // hidden\n    // four spaces\n\t// tab\n");
    assert_eq!(html, "<pre><code>// four spaces\n// tab\n</code></pre>\n");
}

#[test]
fn fenced_code_and_raw_html_blocks_remain_opaque() {
    assert_eq!(
        render("```\n// code\n```\n"),
        "<pre><code>// code\n</code></pre>\n"
    );
    assert_eq!(
        render("<div>\n// html content\n</div>\n"),
        "<div>\n// html content\n</div>\n"
    );
}

#[test]
fn fenced_code_and_raw_html_inside_lists_keep_physical_comments_literal() {
    assert_eq!(
        render("- item\n  ```\n  // code\n  ```\n"),
        "<ul>\n<li>item\n<pre><code>// code\n</code></pre>\n</li>\n</ul>\n"
    );
    assert_eq!(
        render("- <div>\n  // html content\n  </div>\n"),
        "<ul>\n<li>\n<div>\n// html content\n</div>\n</li>\n</ul>\n"
    );
    for source in [
        "- ```\n  // code\n  ```\n",
        "- item\n  ```\n  // code\n  ```\n",
        "- <div>\n  // html content\n  </div>\n",
        "> ```\n> // code\n> ```\n",
    ] {
        assert_eq!(
            render(source),
            render_with_options(source, ParserOptions::gfm(), HtmlRendererOptions::gfm(),)
        );
    }
}

#[test]
fn explicit_container_prefixes_keep_slashes_literal() {
    assert_eq!(
        render("> // visible\n"),
        "<blockquote>\n<p>// visible</p>\n</blockquote>\n"
    );
    assert_eq!(
        render("- // visible\n"),
        "<ul>\n<li>// visible</li>\n</ul>\n"
    );
    assert_eq!(
        render("> - // nested visible\n"),
        "<blockquote>\n<ul>\n<li>// nested visible</li>\n</ul>\n</blockquote>\n"
    );
}

#[test]
fn comment_markers_survive_container_dedenting_with_original_inline_spans() {
    for indent in ["", " ", "  ", "   "] {
        for template in [
            "- first\nCOMMENT// hidden\n  **second**\n",
            "> first\nCOMMENT// hidden\n> **second**\n",
            "Term\n: first\nCOMMENT// hidden\n    **second**\n",
            "Note[^n]\n\n[^n]: first\nCOMMENT// hidden\n    **second**\n",
        ] {
            for source in line_ending_variants(&template.replace("COMMENT", indent)) {
                let allocator = Allocator::new();
                let document = Parser::with_options(
                    &allocator,
                    &source,
                    ParserOptions {
                        definition_lists: true,
                        ..options()
                    },
                )
                .parse()
                .unwrap();
                let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document);
                assert!(!html.contains("hidden"), "{source:?}: {html}");
                assert!(
                    html.contains("<strong>second</strong>"),
                    "{source:?}: {html}"
                );
                let span = first_strong_span(&document);
                assert_eq!(
                    &source[span.start as usize..span.end as usize],
                    "**second**"
                );
            }
        }
    }
}

#[test]
fn root_four_space_comment_markers_remain_visible_code() {
    assert_eq!(
        render("    // visible\n"),
        "<pre><code>// visible\n</code></pre>\n"
    );
}

#[test]
fn trailing_comments_keep_lists_tight_but_real_blanks_make_them_loose() {
    assert_eq!(
        render("- one\n// trailing\n- two\n"),
        "<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n"
    );
    assert_eq!(
        render("- one\n\n// trailing\n\n- two\n"),
        "<ul>\n<li><p>one</p>\n</li>\n<li><p>two</p>\n</li>\n</ul>\n"
    );
}

#[test]
fn unprefixed_comments_do_not_break_quote_or_list_structure() {
    assert_eq!(
        render("> first\n// private\n> second\n"),
        "<blockquote>\n<p>first\nsecond</p>\n</blockquote>\n"
    );
    assert_eq!(
        render("- first\n// private\n- second\n"),
        "<ul>\n<li>first</li>\n<li>second</li>\n</ul>\n"
    );
}

#[test]
fn comments_do_not_prevent_setext_headings_or_table_rows() {
    assert_eq!(render("Heading\n// private\n---\n"), "<h2>Heading</h2>\n");

    let html = render("A | B\n// private header\n- | -\n1 | 2\n// private body\n3 | 4\n");
    assert_eq!(
        html,
        concat!(
            "<table>\n",
            "<thead>\n<tr>\n<th>A</th>\n<th>B</th>\n</tr>\n</thead>\n",
            "<tbody>\n",
            "<tr>\n<td>1</td>\n<td>2</td>\n</tr>\n",
            "<tr>\n<td>3</td>\n<td>4</td>\n</tr>\n",
            "</tbody>\n</table>\n"
        )
    );
}

#[test]
fn absent_comments_preserve_full_ast_with_either_flag_value() {
    for template in [
        "  **text**  \n",
        "first\n\tcontinued\n",
        "Title {#topic .wide}\n---\n",
        "> // visible\n> **continued**\n",
        "- // visible\n  **continued**\n",
        "\u{feff}before\0\n**after**\n",
        "---\ntitle: Metadata\n---\n\n**body**\n",
    ] {
        for source in line_ending_variants(template) {
            let results: Vec<_> = [false, true]
                .into_iter()
                .map(|line_comments| {
                    let allocator = Allocator::new();
                    let document = Parser::with_options(
                        &allocator,
                        &source,
                        ParserOptions {
                            line_comments,
                            front_matter: true,
                            heading_attributes: true,
                            ..ParserOptions::gfm_spec()
                        },
                    )
                    .parse()
                    .unwrap();
                    let html = HtmlRenderer::with_options(HtmlRendererOptions {
                        source_spans: true,
                        ..HtmlRendererOptions::gfm()
                    })
                    .render(&document);
                    (format!("{document:?}"), html)
                })
                .collect();
            assert_eq!(results[0], results[1], "{source:?}");
        }
    }
}

#[test]
fn setext_comment_boundaries_preserve_attributes_and_original_spans() {
    for (template, strong_text) in [
        (
            "**Title** {#topic .wide}\n// trailing\0\n---\n",
            "**Title**",
        ),
        (
            "Title\n// hidden\0\n**continued** {#topic .wide}\n// trailing\n---\n",
            "**continued**",
        ),
    ] {
        for prefix in ["", "\u{feff}"] {
            for source in line_ending_variants(&format!("{prefix}{template}")) {
                let allocator = Allocator::new();
                let document = Parser::with_options(
                    &allocator,
                    &source,
                    ParserOptions {
                        heading_attributes: true,
                        ..options()
                    },
                )
                .parse()
                .unwrap();
                let ferromark::ast::Node::Heading(heading) = &document.children[0] else {
                    panic!("expected setext heading");
                };
                assert_eq!(document.children.len(), 1);
                assert_eq!(heading.depth, 2);
                assert_eq!(heading.id, Some("topic"));
                assert_eq!(heading.classes.as_slice(), &["wide"]);
                assert_eq!(heading.span.end as usize, source.len());
                assert_eq!(
                    first_strong_span(&document).source_text(&source),
                    strong_text
                );
                let html = HtmlRenderer::with_options(HtmlRendererOptions {
                    heading_ids: true,
                    ..HtmlRendererOptions::gfm()
                })
                .render(&document);
                assert!(
                    !html.contains("hidden") && !html.contains("trailing"),
                    "{source:?}: {html}"
                );
                assert!(html.contains("<h2 id=\"topic\" class=\"wide\">"), "{html}");
            }
        }
    }
}

#[test]
fn comments_before_forward_references_do_not_leak() {
    assert_eq!(
        render("// private\n[target]: /url\n\nvisible [target]\n"),
        "<p>visible <a href=\"/url\">target</a></p>\n"
    );
}

#[test]
fn comments_inside_multiline_references_and_between_definitions_do_not_leak() {
    assert_eq!(
        render(
            "[first]:\n// between destination\n/first \"First title\"\n[second]: /second\n// between definitions\n[third]: /third\n\nUse [first], [second], and [third].\n"
        ),
        "<p>Use <a href=\"/first\" title=\"First title\">first</a>, <a href=\"/second\">second</a>, and <a href=\"/third\">third</a>.</p>\n"
    );
}

#[test]
fn comments_inside_footnote_and_definition_list_bodies_disappear() {
    let footnote = render("Note[^one]\n\n[^one]: first\n  // private\n    second\n");
    assert!(
        footnote.contains("<p>Note<sup><a href=\"#fn-one\" id=\"fnref-one\">one</a></sup></p>"),
        "{footnote}"
    );
    assert!(
        footnote.contains("<div id=\"fn-one\" class=\"footnote\">\n<p>first\nsecond</p>"),
        "{footnote}"
    );
    assert!(!footnote.contains("private"), "{footnote}");

    let definition_options = ParserOptions {
        definition_lists: true,
        ..options()
    };
    let definition = render_with_options(
        "Term\n: first\n  // private\n    second\n",
        definition_options,
        HtmlRendererOptions::gfm(),
    );
    assert!(definition.contains("<dt>Term</dt>"), "{definition}");
    assert!(
        definition.contains("<dd>first\nsecond</dd>"),
        "{definition}"
    );
    assert!(!definition.contains("private"), "{definition}");
}

#[test]
fn comments_before_table_attributes_do_not_drop_metadata_or_column_names() {
    let parser_options = ParserOptions {
        table_attributes: true,
        ..options()
    };
    let renderer_options = HtmlRendererOptions {
        table_colgroup: true,
        table_column_names: true,
        ..HtmlRendererOptions::gfm()
    };
    let html = render_with_options(
        "| Name | Price |\n| --- | --- |\n| Widget | 10 |\n// private\n: Caption {#products .wide}\n",
        parser_options,
        renderer_options,
    );
    assert!(
        html.contains("<table id=\"products\" class=\"wide\">"),
        "{html}"
    );
    assert!(html.contains("<caption>Caption</caption>"), "{html}");
    assert!(
        html.contains("<col class=\"col-1 col-name-name\">"),
        "{html}"
    );
    assert!(
        html.contains("<col class=\"col-2 col-name-price\">"),
        "{html}"
    );
    assert!(!html.contains("private"), "{html}");
}

#[test]
fn strong_spans_survive_removed_comments_in_root_setext_quote_and_list_contexts() {
    let cases = [
        ("\u{feff}before\n// private\0\n**visible**\n", "**visible**"),
        ("\u{feff}**Heading**\n// private\0\n---\n", "**Heading**"),
        (
            "\u{feff}> before\n// private\0\n> **visible**\n",
            "**visible**",
        ),
        (
            "\u{feff}- before\n// private\0\n- **visible**\n",
            "**visible**",
        ),
        (
            "\u{feff}- before\n// private\0\n  **visible**\n",
            "**visible**",
        ),
    ];

    for (source, expected) in cases {
        for variant in line_ending_variants(source) {
            let allocator = Allocator::new();
            let document = Parser::with_options(&allocator, &variant, options())
                .parse()
                .unwrap();
            let span = first_strong_span(&document);
            assert_eq!(span.source_text(&variant), expected);
        }
    }
}

#[test]
fn crlf_and_lone_cr_comments_match_lf() {
    let lf = "first\n// private\nsecond\n";
    let expected = render(lf);
    for source in [lf.replace('\n', "\r\n"), lf.replace('\n', "\r")] {
        assert_eq!(render(&source), expected);
    }
}

#[test]
fn bom_and_nul_preserve_source_spans_while_comments_disappear() {
    let source = "\u{feff}// private\rvisible\0\r";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    assert_eq!(document.span.start, 0);
    assert_eq!(document.span.end as usize, source.len());
    assert_eq!(document.children.len(), 1);
    let paragraph = match &document.children[0] {
        ferromark::ast::Node::Paragraph(paragraph) => paragraph,
        node => panic!("expected paragraph, got {node:?}"),
    };
    assert_eq!(paragraph.span.source_text(source), "visible\0\r");

    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        source_spans: true,
        ..HtmlRendererOptions::gfm()
    })
    .render(&document);
    assert_eq!(html, "<p data-source-span=\"14-23\">visible�</p>\n");
}

#[test]
fn renderer_paths_agree_when_comments_are_removed() {
    let source = "before\n// private\nafter\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::gfm();
    let expected = HtmlRenderer::with_options(renderer_options.clone()).render(&document);

    let mut hooks_renderer = HtmlRenderer::with_options(renderer_options.clone());
    assert_eq!(
        hooks_renderer.render_with_hooks(&document, &mut NoHtmlRenderHooks),
        expected
    );

    let mut borrowed_renderer = HtmlRenderer::with_options(renderer_options);
    assert_eq!(borrowed_renderer.render_borrowed(&document), expected);
}
