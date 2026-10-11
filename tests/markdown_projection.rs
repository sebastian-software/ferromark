use ferromark::{
    Allocator, HtmlRenderer, MarkdownProjection, MarkdownProjectionDiagnosticKind as Kind,
    MarkdownProjectionOptions, Parser, ParserOptions,
};

fn project(source: &str, options: &MarkdownProjectionOptions) -> MarkdownProjection {
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: cfg!(feature = "jsx"),
            front_matter: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    document.project_markdown(source, options).unwrap()
}

#[test]
fn ordinary_markdown_and_all_code_forms_are_preserved_verbatim() {
    let source = "# Title\r\n\r\n[linked][ref] and `import x; <A /> {x}`.\r\n\r\n[ref]: /target \"Title\"\r\n\r\n- **One**\r\n  - Two\r\n\r\n| A | B |\r\n| - | - |\r\n| 1 | 2 |\r\n\r\n````mdx\r\n```nested\r\nimport X from 'x'\r\n<A>{x}</A>\r\n````\r\n\r\n~~~~~mdx\r\n~~~\r\nexport const x = 1\r\n<B />\r\n~~~~~\r\n\r\n    import x from 'x';\r\n    <C>{x}</C> [ref]\r\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert_eq!(result.markdown, source);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn unwraps_multiline_nested_components_fragments_and_static_children() {
    let source = "import {\n  Box,\n  Note\n} from './components.js';\n\nexport const meta = {\n  title: 'Guide',\n};\n\n<Box\n title=\"has > and < characters\"\n options={{ nested: { value: '>' } }}\n {...props}\n>\n\n# Guide\n\n<>\n<Note>**Static** [link][ref]</Note>\n</>\n\n[ref]: /guide\n\n</Box>\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert!(!result.markdown.contains("import"));
    assert!(!result.markdown.contains("export"));
    assert!(!result.markdown.contains("Box"));
    assert!(result.markdown.contains("# Guide\n"));
    assert!(result.markdown.contains("**Static** [link][ref]\n"));
    assert!(result.markdown.contains("[ref]: /guide\n"));
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == Kind::EsmOmitted)
            .count(),
        2
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == Kind::JsxWrapperUnwrapped)
            .count(),
        3
    );
}

#[test]
fn preserves_boundaries_and_reports_original_unicode_byte_spans() {
    let source = "日本<A>語</A><B>text</B>fin{value}ish <Badge /> done.\n\n{dynamic}\n\nAfter.\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert_eq!(
        result.markdown,
        "日本 語 text fin ish  done.\n\n\n\nAfter.\n"
    );
    let expressions: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == Kind::ExpressionOmitted)
        .collect();
    assert_eq!(expressions.len(), 2);
    assert_eq!(expressions[0].span.source_text(source), "{value}");
    assert_eq!(
        expressions[0].span.start as usize,
        source.find("{value}").unwrap()
    );
    assert_eq!(expressions[1].span.source_text(source).trim(), "{dynamic}");
    assert!(
        result
            .diagnostics
            .windows(2)
            .all(|pair| pair[0].span.start <= pair[1].span.start)
    );
}

#[test]
fn multiline_tags_keep_blockquote_and_list_prefixes() {
    let source = "> <Box\n>   title=\"a > b\"\n>   options={{ x: '>' }}\n> >\n> # Quoted\n>\n> <Inner>Text</Inner>\n> </Box>\n\n- <Box\n    title=\"listed\"\n  >\n  ## Listed\n\n  - Child\n  </Box>\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert!(
        result.markdown.contains("> # Quoted\n"),
        "{}",
        result.markdown
    );
    assert!(result.markdown.contains("> Text\n"));
    assert!(result.markdown.contains("  ## Listed\n"));
    assert!(result.markdown.contains("  - Child\n"));
    assert!(!result.markdown.contains("Box"));
    assert!(!result.markdown.contains("title"));
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == Kind::UnsupportedJsxOmitted)
    );
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, &result.markdown, ParserOptions::gfm())
        .parse()
        .unwrap();
    let html = HtmlRenderer::new().render(&document);
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<h1 id=\"quoted\">Quoted</h1>"));
    assert!(html.contains("<h2 id=\"listed\">Listed</h2>"));
    assert!(html.contains("<li>Child</li>"));
}

#[test]
fn whole_construct_omissions_preserve_container_markers() {
    let source =
        "> Before.\n>\n> <Badge />\n>\n> {dynamic}\n>\n> After.\n\n- <Badge />\n  Continued.\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert!(!result.markdown.contains("Badge"));
    assert!(!result.markdown.contains("dynamic"));
    assert!(
        result
            .markdown
            .contains("> Before.\n>\n> \n>\n> \n>\n> After.\n"),
        "{}",
        result.markdown
    );
    assert!(result.markdown.contains("- \n  Continued.\n"));
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, &result.markdown).parse().unwrap();
    let html = HtmlRenderer::new().render(&document);
    assert_eq!(html.matches("<blockquote>").count(), 1);
    assert!(html.contains("<li>Continued.</li>"));
}

#[test]
fn frontmatter_and_title_omission_are_explicit_and_read_only() {
    let source = "---\ntitle: Guide\n---\n\n# **Guide**\n\n# Guide\n\nBody.\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            front_matter: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let before = HtmlRenderer::new().render(&document);
    assert_eq!(
        document
            .project_markdown(source, &MarkdownProjectionOptions::default())
            .unwrap()
            .markdown,
        source
    );
    let options = MarkdownProjectionOptions {
        omit_front_matter: true,
        omit_title_heading: Some(" Guide ".into()),
    };
    let first = document.project_markdown(source, &options).unwrap();
    assert!(!first.markdown.contains("title: Guide"));
    assert!(!first.markdown.contains("**Guide**"));
    assert!(first.markdown.contains("# Guide\n"));
    assert_eq!(
        first
            .omitted_title_heading
            .unwrap()
            .source_text(source)
            .trim(),
        "# **Guide**"
    );
    assert_eq!(document.project_markdown(source, &options).unwrap(), first);
    assert_eq!(HtmlRenderer::new().render(&document), before);
    assert!(
        document
            .project_markdown(
                source,
                &MarkdownProjectionOptions {
                    omit_title_heading: Some("Different".into()),
                    ..MarkdownProjectionOptions::default()
                }
            )
            .unwrap()
            .omitted_title_heading
            .is_none()
    );
}

#[test]
fn raw_html_is_omitted_conservatively_without_interpreting_its_body() {
    let source = "before<i>inline</i>after\n\n<div>\nblock body\n</div>\n\nAfter.\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let result = document
        .project_markdown(source, &MarkdownProjectionOptions::default())
        .unwrap();
    assert!(result.markdown.starts_with("before inline after\n"));
    assert!(!result.markdown.contains("block body"));
    assert_eq!(result.diagnostics.len(), 3);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind == Kind::RawHtmlOmitted)
    );
}

#[test]
fn inline_raw_text_bodies_are_omitted_in_markdown_grammar() {
    let source = "before<script>const hidden = 1;</script>after\n\nbefore<STYLE>secret CSS</STYLE>after\n\nbefore<title>Internal</title>after\n\nbefore<textarea>Editable</textarea>after\n\nVisible.\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let result = document
        .project_markdown(source, &MarkdownProjectionOptions::default())
        .unwrap();
    assert_eq!(
        result.markdown,
        "before after\n\nbefore after\n\nbefore after\n\nbefore after\n\nVisible.\n"
    );
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == Kind::RawTextElementOmitted)
        .collect();
    assert_eq!(diagnostics.len(), 4);
    assert_eq!(
        diagnostics[0].span.source_text(source),
        "<script>const hidden = 1;</script>"
    );
}

#[test]
fn raw_html_slashes_do_not_self_close_nonvoid_raw_text_elements() {
    for name in ["script", "STYLE", "title", "textarea"] {
        let source = format!("before<{name}/>hidden</{name}>after\n");
        let allocator = Allocator::new();
        let document = Parser::new(&allocator, &source).parse().unwrap();
        let result = document
            .project_markdown(&source, &MarkdownProjectionOptions::default())
            .unwrap();
        assert_eq!(result.markdown, "before after\n");
        let omission = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.kind == Kind::RawTextElementOmitted)
            .unwrap();
        assert_eq!(
            omission.span.source_text(&source),
            format!("<{name}/>hidden</{name}>")
        );
    }
}

#[test]
fn raw_html_context_follows_table_rows_before_the_authored_caption() {
    let sources = [
        "| before</script>visible |\n| --- |\n| cell |\n: before<script>hidden\n\nVisible\n",
        "| before<script>hidden |\n| --- |\n| cell |\n: hidden</script>after\n\nVisible\n",
    ];
    for (index, source) in sources.iter().enumerate() {
        let allocator = Allocator::new();
        let document = Parser::with_options(
            &allocator,
            source,
            ParserOptions {
                table_attributes: true,
                ..ParserOptions::gfm()
            },
        )
        .parse()
        .unwrap();
        let result = document
            .project_markdown(source, &MarkdownProjectionOptions::default())
            .unwrap();
        assert!(!result.markdown.contains("hidden"));
        let omission = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.kind == Kind::RawTextElementOmitted)
            .unwrap();
        assert_eq!(
            omission.span.start as usize,
            source.find("<script>").unwrap()
        );
        if index == 0 {
            assert!(result.markdown.contains("before visible |\n"));
            assert!(result.markdown.contains("| cell |\n"));
            assert!(!result.markdown.contains("Visible"));
            assert_eq!(omission.span.end as usize, source.len());
        } else {
            assert!(result.markdown.ends_with("after\n\nVisible\n"));
            assert_eq!(
                omission.span.end as usize,
                source.find("</script>").unwrap() + "</script>".len()
            );
        }
    }
}

#[test]
fn unterminated_inline_raw_text_body_is_omitted_through_the_end_of_source() {
    let source = "before<script>const hidden = 1;\n\n## Hidden heading\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let result = document
        .project_markdown(source, &MarkdownProjectionOptions::default())
        .unwrap();
    assert_eq!(result.markdown, "before\n\n\n");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == Kind::RawTextElementOmitted
                && diagnostic.span.end as usize == source.len())
    );
}

#[test]
fn invalid_source_ranges_return_an_error_instead_of_slicing() {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, "# 日本語\n").parse().unwrap();
    assert!(
        document
            .project_markdown("short", &MarkdownProjectionOptions::default())
            .is_err()
    );
}

#[test]
fn removals_do_not_form_new_emphasis_code_or_link_tokens() {
    let source = "*left*{value}*right*\n\n`left`{value}`right`\n\n<A>*left*</A><B>*right*</B>\n\n<A>[left](/left)</A><B>[right](/right)</B>\n\n**<A>strong</A>** and [<A>link</A>](/url).\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert_eq!(
        result.markdown,
        "*left* *right*\n\n`left` `right`\n\n*left* *right*\n\n[left](/left) [right](/right)\n\n**strong** and [link](/url).\n"
    );
}

#[test]
#[cfg(feature = "jsx")]
fn intrinsic_raw_text_bodies_are_omitted_and_generic_wrappers_are_unwrapped() {
    let source = "<script>const secret = 'code';</script>\n\n<style>{'.hidden { color: red; }'}</style>\n\n<title>Internal title</title>\n\n<textarea>Editable input</textarea>\n\n<div>Visible **Markdown**</div>\n\n<details>\n\n# Detail\n\n</details>\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert!(!result.markdown.contains("secret"));
    assert!(!result.markdown.contains("hidden"));
    assert!(!result.markdown.contains("Internal"));
    assert!(!result.markdown.contains("Editable"));
    assert!(result.markdown.contains("Visible **Markdown**\n"));
    assert!(result.markdown.contains("# Detail\n"));
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == Kind::RawTextElementOmitted)
            .count(),
        4
    );
}

#[test]
fn code_inside_components_preserves_original_fences_and_child_indentation() {
    let source = "<Box>\n\n  ~~~~~mdx\n  <A>{example}</A>\n  import x from 'x'\n  ~~~~~\n\n</Box>\n\n<Other>`<B /> {inline}`</Other>\n";
    let result = project(source, &MarkdownProjectionOptions::default());
    assert!(
        result
            .markdown
            .contains("  ~~~~~mdx\n  <A>{example}</A>\n  import x from 'x'\n  ~~~~~\n")
    );
    assert!(result.markdown.contains("`<B /> {inline}`\n"));
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == Kind::ExpressionOmitted)
    );
}

#[test]
fn title_selection_uses_static_jsx_children_and_only_the_first_top_level_h1() {
    let source = "# <Badge>Guide</Badge>{value}\n\n# Other\n";
    let result = project(
        source,
        &MarkdownProjectionOptions {
            omit_title_heading: Some("Guide".into()),
            ..MarkdownProjectionOptions::default()
        },
    );
    assert!(result.omitted_title_heading.is_some());
    assert!(!result.markdown.contains("Badge"));
    assert!(result.markdown.contains("# Other"));
    let result = project(
        "# Other\n\n# Guide\n",
        &MarkdownProjectionOptions {
            omit_title_heading: Some("Guide".into()),
            ..MarkdownProjectionOptions::default()
        },
    );
    assert!(result.omitted_title_heading.is_none());
}
