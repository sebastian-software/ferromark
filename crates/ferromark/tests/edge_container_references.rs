//! CommonMark 0.31.2 section 4.7: container definitions have document scope.
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(source: &str) -> String {
    let arena = Allocator::new();
    let doc = Parser::with_options(&arena, source, ParserOptions::commonmark())
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&doc)
}

#[test]
fn list_definitions_resolve_forward_backward_and_image_references() {
    for definition in [
        "- [target]: /url \"title\"",
        "+ [target]: /url\n  \"title\"",
        "7. [target]: /url \"title\"",
        "- Item\n\n  [target]: /url \"title\"",
        "- Outer\n  - [target]: /url \"title\"",
        "> - [target]: /url \"title\"",
        "- > [target]: /url \"title\"",
        "- Item\n\n  > [target]: /url \"title\"",
    ] {
        let source = format!("[target]\n\n{definition}\n\n[target][] ![alt][target]\n");
        let html = render(&source);
        assert_eq!(
            html.matches("href=\"/url\" title=\"title\"").count(),
            2,
            "{source:?}: {html}"
        );
        assert!(
            html.contains("src=\"/url\" alt=\"alt\" title=\"title\""),
            "{source:?}: {html}"
        );
        assert!(!html.contains("[target]:"), "{source:?}: {html}");
    }
}

#[test]
fn first_definition_wins_across_root_and_nested_containers() {
    for definitions in [
        "- [target]: /first\n\n[target]: /second",
        "[target]: /first\n\n- [target]: /second",
        "> - [target]: /first\n\n- > [target]: /second",
    ] {
        let html = render(&format!("[target]\n\n{definitions}\n"));
        assert!(html.contains("href=\"/first\""), "{definitions:?}: {html}");
        assert!(!html.contains("href=\"/second\""), "{html}");
    }
}

#[test]
fn definition_shapes_in_container_code_and_paragraphs_remain_literal() {
    for decoy in [
        "- ```\n  [target]: /wrong\n  ```",
        "- Item\n\n      [target]: /wrong",
        "- Item\n  [target]: /wrong",
        "- <div>\n  [target]: /wrong\n  </div>",
    ] {
        let html = render(&format!("{decoy}\n\n[target]: /right\n\n[target]\n"));
        assert!(html.contains("href=\"/right\""), "{decoy:?}: {html}");
        assert!(!html.contains("href=\"/wrong\""), "{html}");
    }
}

#[test]
fn container_definitions_survive_comments_tabs_and_line_ending_normalization() {
    let cases = [
        "[target]\n\n- [target]: /url\n  // private\n  \"title\"\n",
        "[target]\n\n-\t[target]: /url \"title\"\n",
        "[target]\n\n- Outer\n\n  2) [target]: /url \"title\"\n",
    ];
    for source in cases {
        for ending in ["\n", "\r\n", "\r"] {
            let source = source.replace('\n', ending);
            let arena = Allocator::new();
            let doc = Parser::with_options(
                &arena,
                &source,
                ParserOptions {
                    line_comments: true,
                    ..ParserOptions::commonmark()
                },
            )
            .parse()
            .unwrap();
            let html = HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&doc);
            assert!(
                html.contains("href=\"/url\" title=\"title\""),
                "{source:?}: {html}"
            );
        }
    }
}

#[test]
fn disabling_references_keeps_container_definition_syntax_visible() {
    let source = "[target]\n\n- [target]: /url\n";
    let arena = Allocator::new();
    let doc = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            allow_link_refs: false,
            ..ParserOptions::commonmark()
        },
    )
    .parse()
    .unwrap();
    let html = HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&doc);
    assert!(html.contains("[target]: /url"), "{html}");
    assert!(!html.contains("<a "), "{html}");
}

#[test]
fn unclosed_quoted_fence_ends_before_root_reference_definitions() {
    // CommonMark 0.31.2 section 4.5, example 128: the enclosing quote
    // terminates an unclosed fence. The old flat prepass leaked fence state.
    for fence in ["```", "~~~"] {
        for ending in ["\n", "\r\n", "\r"] {
            let source = format!("> {fence}{ending}{ending}[a]: /url{ending}{ending}[a]");
            assert_eq!(
                render(&source),
                "<blockquote>\n<pre><code></code></pre>\n</blockquote>\n<p><a href=\"/url\">a</a></p>\n",
                "{source:?}"
            );
        }
    }
}
