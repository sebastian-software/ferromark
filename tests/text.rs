use ferromark::{
    Allocator, JsxRenderer, JsxRendererOptions, Parser, ParserOptions, TextExtractionOptions,
};

fn options(rendering: JsxRendererOptions) -> TextExtractionOptions {
    TextExtractionOptions {
        rendering,
        ..TextExtractionOptions::default()
    }
}

#[test]
fn projects_decoded_prose_with_stable_lists_tables_and_block_separators() {
    let source = concat!(
        "Intro **bold** &amp; \\*literal\\* [label](https://example.com \"title\") ![Café](image.png).\n",
        "Soft\nline and `x & y`  \nnext.\n\n",
        "- first\n- second\n  - nested\n\n",
        "| A | B | C |\n| - | - | - |\n| one | two | |\n\n",
        "---\n\nLast.\n",
    );
    let arena = Allocator::new();
    let document = Parser::with_options(&arena, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let result = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(
        result.text,
        concat!(
            "Intro bold & *literal* label Café. Soft line and x & y next.\n\n",
            "first\nsecond\nnested\n\n",
            "A\tB\tC\none\ttwo\t\n\nLast."
        )
    );
    assert!(result.sections.is_empty());
}

#[test]
fn section_ranges_use_utf8_output_bytes_and_separate_original_source_spans() {
    let source =
        "Intro 🎉\n\n# Café &amp; 東京\n\nA **B**.\n\n### 子\n\n`🦀`\n\n## Next\n\nDone.\n";
    let arena = Allocator::new();
    let document = Parser::new(&arena, source).parse().unwrap();
    let result = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(
        result.text,
        "Intro 🎉\n\nCafé & 東京\n\nA B.\n\n子\n\n🦀\n\nNext\n\nDone."
    );
    assert_eq!(result.sections.len(), 3);
    assert_eq!(result.sections[0].text, "Café & 東京\n\nA B.");
    assert_eq!(result.sections[1].parent_index, Some(0));
    assert_eq!(result.sections[2].parent_index, Some(0));
    for section in &result.sections {
        assert_eq!(
            &result.text[section.text_range.start..section.text_range.end],
            section.text
        );
        assert!(source[section.heading_span.unwrap().start as usize..].starts_with('#'));
        assert!(
            section
                .source_span
                .unwrap()
                .contains_span(&section.heading_span.unwrap())
        );
    }
    assert_ne!(
        result.sections[1].heading_span.unwrap().start as usize,
        result.sections[1].text_range.start
    );
    assert_eq!(
        result.sections[0].source_span.unwrap().source_text(source),
        "# Café &amp; 東京\n\nA **B**.\n\n"
    );
    assert_eq!(
        result.sections[2].source_span.unwrap().end as usize,
        source.len()
    );
}

#[test]
fn final_heading_planning_matches_jsx_including_omission_footnotes_and_element_ids() {
    let source = concat!(
        "> # Title\n\n# Title\n\n",
        "[link](url){#docs-duplicate} ![alt](img){#docs-duplicate-1} [span]{#docs-duplicate-2}\n\n",
        "[^x] [^x]\n\n# Duplicate\n\n## Explicit {#duplicate}\n\n# fn-x\n\n# fnref-x\n\n",
        "[^x]: Body\n\n    ### Footnote heading\n",
    );
    let arena = Allocator::new();
    let document = Parser::with_options(&arena, source, ParserOptions::ffm())
        .parse()
        .unwrap();
    for prefix in ["", "docs-"] {
        for heading_ids in [true, false] {
            for offset in [-10, 0, 10] {
                let rendering = JsxRendererOptions {
                    omit_title_heading: Some("Title".to_owned()),
                    heading_ids,
                    heading_level_offset: offset,
                    heading_id_prefix: prefix.to_owned(),
                    ..JsxRendererOptions::default()
                };
                let rendered =
                    JsxRenderer::with_options(rendering.clone()).render(&document, source);
                let extracted = document.extract_text(&options(rendering));
                assert_eq!(extracted.sections.len(), rendered.headings.len());
                for (section, heading) in extracted.sections.iter().zip(rendered.headings) {
                    assert_eq!(
                        (
                            &section.heading,
                            section.level,
                            &section.id,
                            section.heading_span
                        ),
                        (
                            &heading.text,
                            heading.level,
                            &heading.id,
                            Some(heading.span)
                        )
                    );
                }
                assert_eq!(
                    extracted.omitted_title_heading,
                    rendered.omitted_title_heading
                );
                assert!(extracted.text.contains("Body"));
                assert!(!extracted.text.contains("↩"));
            }
        }
    }
}

#[test]
fn title_omission_considers_only_first_top_level_h1_and_does_not_claim_its_id() {
    let arena = Allocator::new();
    let source = "# Guide\n\n# Guide\n\n## Guide\n";
    let document = Parser::new(&arena, source).parse().unwrap();
    let extracted = document.extract_text(&options(JsxRendererOptions {
        omit_title_heading: Some(" Guide ".into()),
        ..JsxRendererOptions::default()
    }));
    assert_eq!(
        extracted
            .sections
            .iter()
            .map(|section| section.id.as_deref())
            .collect::<Vec<_>>(),
        [Some("guide"), Some("guide-1")]
    );
    let arena = Allocator::new();
    let source = "# Other\n\n# Guide\n";
    let document = Parser::new(&arena, source).parse().unwrap();
    assert!(
        document
            .extract_text(&options(JsxRendererOptions {
                omit_title_heading: Some("Guide".into()),
                ..JsxRendererOptions::default()
            }))
            .omitted_title_heading
            .is_none()
    );
    assert!(
        document
            .extract_text(&options(JsxRendererOptions {
                omit_title_heading: Some(" ".into()),
                ..JsxRendererOptions::default()
            }))
            .omitted_title_heading
            .is_none()
    );
}

#[test]
fn code_can_be_excluded_without_losing_inline_code_math_or_footnote_bodies() {
    let source = "# Source\n\nUse `inline &amp;` $a + b$.[^n]\n\n```rust\n  let x = 1;\n\n```\n\n    indented\n\n$$\nx^2\n$$\n\n[^n]: footnote **body**\n\n<div>raw &amp; html</div>\n";
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            math: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let included = document.extract_text(&TextExtractionOptions::default());
    assert!(included.text.contains("  let x = 1;"));
    assert!(included.text.contains("indented"));
    let excluded = document.extract_text(&TextExtractionOptions {
        include_block_code: false,
        ..TextExtractionOptions::default()
    });
    assert_eq!(
        excluded.text,
        "Source\n\nUse inline &amp; a + b.\n\nx^2\n\nfootnote body"
    );
    assert_eq!(included.sections[0].id, excluded.sections[0].id);
}

#[test]
#[cfg(feature = "jsx")]
fn mdx_contributes_static_children_without_imports_expressions_attributes_or_evaluation() {
    let source = concat!(
        "export const runtime = 'secret';\n\n",
        "# Hello <Badge>{runtime}**Café**</Badge>\n\n",
        "<Note label=\"not visible\">\n\n## Inner\n\nStatic <em>prose</em> {runtime} and <Badge>child</Badge>.\n\n</Note>\n\n",
        "<OnlyRuntime value={runtime} />\n\n{runtime}\n\n",
        "<script>hidden script</script>\n\n<style>hidden style</style>\n",
    );
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let rendered = JsxRenderer::new().render(&document, source);
    let extracted = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(
        extracted.text,
        "Hello Café\n\nInner\n\nStatic prose and child."
    );
    assert_eq!(
        extracted
            .sections
            .iter()
            .map(|section| &section.heading)
            .collect::<Vec<_>>(),
        [&"Hello Café".to_owned(), &"Inner".to_owned()]
    );
    for (section, heading) in extracted.sections.iter().zip(rendered.headings) {
        assert_eq!(section.id, heading.id);
    }
    assert_eq!(extracted.sections[1].parent_index, Some(0));
}

#[test]
fn callout_and_extension_text_policies_use_the_parsed_tree() {
    let source = "> [!NOTE]\n> Static **body**\n\nTerm\n: Definition\n\n![alt](img.png)\n: Figure *caption*\n\n| A | B |\n| - | - |\n| C | D |\n: Table *caption*\n";
    let arena = Allocator::new();
    let document = Parser::with_options(&arena, source, ParserOptions::ffm())
        .parse()
        .unwrap();
    let extracted = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(
        extracted.text,
        "Note\n\nStatic body\n\nTerm\n\nDefinition\n\nalt\n\nFigure caption\n\nTable caption\n\nA\tB\nC\tD"
    );
    let disabled = document.extract_text(&options(JsxRendererOptions {
        callouts: false,
        ..JsxRendererOptions::default()
    }));
    assert!(disabled.text.starts_with("[!NOTE] Static body"));
}

#[test]
fn extraction_is_immutable_repeatable_and_generated_spans_remain_unavailable() {
    let source = "# Original\n\nBody.\n\n## Child\n";
    let arena = Allocator::new();
    let mut document = Parser::new(&arena, source).parse().unwrap();
    let ferromark::ast::Node::Heading(heading) = &mut document.children[0] else {
        panic!()
    };
    heading.span = ferromark::ast::Span::empty();
    let ferromark::ast::Node::Text(text) = &mut heading.children[0] else {
        panic!()
    };
    text.value = arena.alloc_str("Transformed 🦀");
    let before = JsxRenderer::new().render(&document, source);
    let first = document.extract_text(&TextExtractionOptions::default());
    let second = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(first, second);
    assert_eq!(before, JsxRenderer::new().render(&document, source));
    assert_eq!(first.sections[0].heading, "Transformed 🦀");
    assert!(first.sections[0].heading_span.is_none());
    assert!(first.sections[0].source_span.is_none());
    assert_eq!(first.sections[0].id, before.headings[0].id);
    document.children.swap(0, 2);
    let reordered = document.extract_text(&TextExtractionOptions::default());
    assert!(reordered.sections[0].source_span.is_none());
}

#[test]
fn raw_html_and_mdx_hidden_elements_do_not_leak_prose_and_runtime_gaps_separate_words() {
    let arena = Allocator::new();
    let source = "Visible <SCRIPT>secret</SCRIPT> text <style>hidden</style> after <textarea>input</textarea> end.\n";
    let document = Parser::new(&arena, source).parse().unwrap();
    assert_eq!(
        document
            .extract_text(&TextExtractionOptions::default())
            .text,
        "Visible text after end."
    );
    let arena = Allocator::new();
    let source = "left{runtime}right <Badge />tail <Badge>static</Badge>word <script>secret</script>end <Script>visible child</Script>\n";
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: cfg!(feature = "jsx"),
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    assert_eq!(
        document
            .extract_text(&TextExtractionOptions::default())
            .text,
        "left right tail staticword end visible child"
    );
}

#[test]
fn reordered_real_headings_do_not_fabricate_contiguous_source_sections() {
    let arena = Allocator::new();
    let source = "# First\n\n## Second\n";
    let mut document = Parser::new(&arena, source).parse().unwrap();
    document.children.swap(0, 1);
    let result = document.extract_text(&TextExtractionOptions::default());
    assert!(
        result
            .sections
            .iter()
            .all(|section| section.heading_span.is_some() && section.source_span.is_none())
    );
}

#[cfg(feature = "jsx")]
#[test]
fn custom_title_components_keep_nested_headings_and_match_final_jsx_anchors() {
    let source = "# Root\n\n<Title>\n\n## Child\n\nretained\n\n</Title>\n\n<Script>visible</Script>\n\n<script>hidden</script>\n\n<title>hidden title</title>\n";
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let extracted = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(extracted.text, "Root\n\nChild\n\nretained\n\nvisible");
    let rendered = JsxRenderer::new().render(&document, source);
    assert_eq!(
        extracted
            .sections
            .iter()
            .map(|section| (&section.heading, &section.id))
            .collect::<Vec<_>>(),
        rendered
            .headings
            .iter()
            .map(|heading| (&heading.text, &heading.id))
            .collect::<Vec<_>>()
    );
    assert_eq!(extracted.sections[1].parent_index, Some(0));
}

#[test]
fn html_self_closing_slashes_do_not_make_raw_text_elements_void() {
    for tag in ["script", "style", "title", "textarea"] {
        let source = format!("before<{tag}/>hidden</{tag}>after\n");
        let arena = Allocator::new();
        let document = Parser::new(&arena, &source).parse().unwrap();
        assert_eq!(
            document
                .extract_text(&TextExtractionOptions::default())
                .text,
            "before after",
            "{tag}"
        );
    }
}

#[cfg(feature = "jsx")]
#[test]
fn mdx_self_closing_raw_text_elements_do_not_hide_following_prose() {
    for tag in ["script", "style", "title", "textarea"] {
        let source = format!("before<{tag}/>after\n");
        let arena = Allocator::new();
        let document = Parser::with_options(
            &arena,
            &source,
            ParserOptions {
                mdx: true,
                mdx_compatible: true,
                ..ParserOptions::default()
            },
        )
        .parse()
        .unwrap();
        assert_eq!(
            document
                .extract_text(&TextExtractionOptions::default())
                .text,
            "before after",
            "{tag}"
        );
    }
}

#[test]
fn raw_text_context_does_not_cross_table_cells_captions_or_paragraphs() {
    for (source, expected) in [
        (
            "| before</script>visible |\n| --- |\n| cell |\n: before<script>hidden\n\nVisible\n",
            "before\n\nbeforevisible\ncell\n\nVisible",
        ),
        (
            "| A |\n| --- |\n| before<script>hidden |\n: hidden</script>after\n\nVisible\n",
            "hiddenafter\n\nA\nbefore\n\nVisible",
        ),
        ("before<script>hidden\n\nVisible\n", "before\n\nVisible"),
        (
            "> [!NOTE]\n> before<script>hidden</script>after\n",
            "Note\n\nbefore after",
        ),
        (
            "before<script>hidden *inner</script>after* visible\n",
            "before after visible",
        ),
    ] {
        let arena = Allocator::new();
        let document = Parser::with_options(
            &arena,
            source,
            ParserOptions {
                table_attributes: true,
                ..ParserOptions::gfm()
            },
        )
        .parse()
        .unwrap();
        assert_eq!(
            document
                .extract_text(&TextExtractionOptions::default())
                .text,
            expected
        );
    }
}

#[test]
fn omitted_generated_title_has_no_original_source_provenance() {
    let arena = Allocator::new();
    let source = "# Generated\n\n## Following\n";
    let mut document = Parser::new(&arena, source).parse().unwrap();
    let ferromark::ast::Node::Heading(heading) = &mut document.children[0] else {
        panic!()
    };
    heading.span = ferromark::ast::Span::empty();
    let extraction = document.extract_text(&options(JsxRendererOptions {
        omit_title_heading: Some("Generated".to_owned()),
        ..JsxRendererOptions::default()
    }));
    assert_eq!(extraction.text, "Following");
    assert_eq!(extraction.sections.len(), 1);
    assert_eq!(extraction.omitted_title_heading, None);
}
