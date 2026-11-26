//! Regression tests for transforms prose scoping, text replacement indexing,
//! and typography escape handling.

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use ferromark::ast::{Node, Span, Text};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark_transforms::{
    EmojiShortcodesPass, GitHubReferencesPass, TransformContext, TransformPass, TypographyLanguage,
    TypographyOptions, TypographyPass,
};

#[test]
fn unclosed_raw_html_tag_does_not_leak_across_paragraphs() {
    let source = "<span>first :rocket:\n\nsecond :rocket: \"quoted\" #42\n";
    let arena = Allocator::for_source_len(source.len());
    let mut doc = Parser::with_options(&arena, source, ParserOptions::gfm())
        .parse()
        .expect("parse");
    let renderer_options = HtmlRendererOptions::default();
    let ctx = TransformContext::new(&arena, source, &renderer_options);

    EmojiShortcodesPass::new().apply(&mut doc, &ctx).unwrap();
    TypographyPass::new(TypographyOptions::new(TypographyLanguage::English))
        .apply(&mut doc, &ctx)
        .unwrap();
    GitHubReferencesPass::new("sebastian-software/ferromark")
        .unwrap()
        .apply(&mut doc, &ctx)
        .unwrap();

    let html = HtmlRenderer::with_options(renderer_options).render(&doc);
    assert!(
        html.contains("<span>first :rocket:</p>"),
        "first paragraph inside unclosed span should remain untransformed: {html}"
    );
    assert!(
        html.contains("second \u{1f680} \u{201c}quoted\u{201d}"),
        "second paragraph should be transformed normally: {html}"
    );
    assert!(
        html.contains("issues/42"),
        "second paragraph GitHub issue ref should be linked: {html}"
    );
}

#[test]
fn emoji_and_github_handle_preceding_empty_span_text_siblings() {
    let arena = Allocator::new();
    let mut children = arena.new_vec();
    children.push(Node::Text(Text {
        value: "prefix ",
        span: Span::empty(),
    }));
    children.push(Node::Text(Text {
        value: ":rocket: and #7",
        span: Span::empty(),
    }));
    let mut doc_children = arena.new_vec_with_capacity(1);
    doc_children.push(Node::Paragraph(arena.boxed(ferromark::ast::Paragraph {
        children,
        span: Span::empty(),
    })));
    let mut doc = ferromark::ast::Document {
        children: doc_children,
        front_matter: None,
        span: Span::empty(),
    };

    let renderer_options = HtmlRendererOptions::default();
    let ctx = TransformContext::new(&arena, "", &renderer_options);

    EmojiShortcodesPass::new().apply(&mut doc, &ctx).unwrap();
    GitHubReferencesPass::new("sebastian-software/ferromark")
        .unwrap()
        .apply(&mut doc, &ctx)
        .unwrap();

    let html = HtmlRenderer::with_options(renderer_options).render(&doc);
    assert!(html.contains("\u{1f680}"), "missing emoji: {html}");
    assert!(html.contains("issues/7"), "missing github link: {html}");
}

#[test]
fn typography_handles_escaped_backslash_before_punctuation() {
    let source = r#"\\ "hello" \\..."#;
    let arena = Allocator::new();
    let mut children = arena.new_vec_with_capacity(1);
    children.push(Node::Text(Text {
        value: r#"\ "hello" \..."#,
        span: Span::new(0, source.len() as u32),
    }));
    let mut doc_children = arena.new_vec_with_capacity(1);
    doc_children.push(Node::Paragraph(arena.boxed(ferromark::ast::Paragraph {
        children,
        span: Span::new(0, source.len() as u32),
    })));
    let mut doc = ferromark::ast::Document {
        children: doc_children,
        front_matter: None,
        span: Span::new(0, source.len() as u32),
    };
    let renderer_options = HtmlRendererOptions::default();
    let ctx = TransformContext::new(&arena, source, &renderer_options);
    TypographyPass::new(TypographyOptions::new(TypographyLanguage::English))
        .apply(&mut doc, &ctx)
        .unwrap();
    let html = HtmlRenderer::with_options(renderer_options.clone()).render(&doc);
    assert!(
        html.contains("\\ \u{201c}hello\u{201d} \\\u{2026}"),
        "punctuation after escaped backslash should be converted: {html}"
    );

    let escaped = r#"\"hello\""#;
    let arena2 = Allocator::for_source_len(escaped.len());
    let mut doc2 = Parser::new(&arena2, escaped).parse().expect("parse");
    let ctx2 = TransformContext::new(&arena2, escaped, &renderer_options);
    TypographyPass::new(TypographyOptions::new(TypographyLanguage::English))
        .apply(&mut doc2, &ctx2)
        .unwrap();
    let html2 = HtmlRenderer::with_options(renderer_options).render(&doc2);
    assert!(
        html2.contains("&quot;hello&quot;"),
        "backslash-escaped quotes should stay straight: {html2}"
    );
}

#[test]
fn single_pass_emoji_and_github_preserve_empty_nodes_and_dense_runs() {
    let arena = Allocator::new();
    let mut children = arena.new_vec();
    // Mix of split text nodes, empty text nodes (at start, between matches,
    // inside a multi-node match, and at end), plus non-text siblings.
    children.push(Node::Text(Text {
        value: "",
        span: Span::new(0, 1),
    }));
    children.push(Node::Text(Text {
        value: "Hi :",
        span: Span::new(1, 5),
    }));
    children.push(Node::Text(Text {
        value: "",
        span: Span::new(5, 6),
    }));
    children.push(Node::Text(Text {
        value: "rocket",
        span: Span::new(6, 12),
    }));
    children.push(Node::Text(Text {
        value: ":",
        span: Span::new(12, 13),
    }));
    children.push(Node::Text(Text {
        value: "",
        span: Span::new(13, 14),
    }));
    children.push(Node::Text(Text {
        value: ":smile: #12 and #34",
        span: Span::new(14, 33),
    }));
    children.push(Node::Text(Text {
        value: "",
        span: Span::new(33, 34),
    }));

    let mut doc_children = arena.new_vec_with_capacity(1);
    doc_children.push(Node::Paragraph(arena.boxed(ferromark::ast::Paragraph {
        children,
        span: Span::new(0, 34),
    })));
    let mut doc = ferromark::ast::Document {
        children: doc_children,
        front_matter: None,
        span: Span::new(0, 34),
    };

    let renderer_options = HtmlRendererOptions::default();
    let ctx = TransformContext::new(&arena, "", &renderer_options);
    EmojiShortcodesPass::new().apply(&mut doc, &ctx).unwrap();

    let Node::Paragraph(paragraph) = &doc.children[0] else {
        panic!("expected paragraph");
    };
    let summary: Vec<(&str, Span)> = paragraph
        .children
        .iter()
        .map(|node| match node {
            Node::Text(text) => (text.value, text.span),
            _ => panic!("expected text"),
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            ("", Span::new(0, 1)),
            ("Hi ", Span::new(1, 5)),
            ("🚀", Span::new(1, 13)),
            ("", Span::new(5, 6)),
            ("", Span::new(13, 14)),
            ("😄", Span::new(14, 33)),
            (" #12 and #34", Span::new(14, 33)),
            ("", Span::new(33, 34)),
        ]
    );

    GitHubReferencesPass::new("sebastian-software/ferromark")
        .unwrap()
        .apply(&mut doc, &ctx)
        .unwrap();
    let html = HtmlRenderer::with_options(renderer_options).render(&doc);
    assert_eq!(
        html,
        "<p>Hi 🚀😄 <a href=\"https://github.com/sebastian-software/ferromark/issues/12\" target=\"_blank\" rel=\"noopener noreferrer\">#12</a> and <a href=\"https://github.com/sebastian-software/ferromark/issues/34\" target=\"_blank\" rel=\"noopener noreferrer\">#34</a></p>\n"
    );
}

#[test]
fn raw_html_scoping_handles_mixed_case_and_void_tags() {
    let source = "<SPAN>:rocket: #1</span ><BR>:smile: #2 \"quoted\"";
    let arena = Allocator::for_source_len(source.len());
    let mut doc = Parser::with_options(&arena, source, ParserOptions::gfm())
        .parse()
        .expect("parse");
    let renderer_options = HtmlRendererOptions::default();
    let ctx = TransformContext::new(&arena, source, &renderer_options);

    EmojiShortcodesPass::new().apply(&mut doc, &ctx).unwrap();
    GitHubReferencesPass::new("sebastian-software/ferromark")
        .unwrap()
        .apply(&mut doc, &ctx)
        .unwrap();
    TypographyPass::new(TypographyOptions::new(TypographyLanguage::English))
        .apply(&mut doc, &ctx)
        .unwrap();

    let html = HtmlRenderer::with_options(renderer_options).render(&doc);
    assert!(
        html.contains("<SPAN>:rocket: #1</span >"),
        "mixed-case open/close tags should protect enclosed text: {html}"
    );
    assert!(
        html.contains(
            "<BR>😄 <a href=\"https://github.com/sebastian-software/ferromark/issues/2\""
        ),
        "uppercase void tag BR should not suppress later transforms: {html}"
    );
    assert!(
        html.contains("\u{201c}quoted\u{201d}"),
        "typography after void tag should still run: {html}"
    );
}

#[test]
fn typography_does_not_treat_zero_width_span_as_overlapping_raw_html() {
    for guillemets in [false, true] {
        let arena = Allocator::new();
        let mut children = arena.new_vec();
        children.push(Node::Html(ferromark::ast::Html {
            value: "<i>",
            span: Span::new(0, 3),
        }));
        children.push(Node::Text(Text {
            value: "abcd",
            span: Span::new(3, 7),
        }));
        children.push(Node::Html(ferromark::ast::Html {
            value: "</i>",
            span: Span::new(7, 11),
        }));
        children.push(Node::Text(Text {
            value: "...",
            span: Span::new(5, 5),
        }));
        let mut doc_children = arena.new_vec_with_capacity(1);
        doc_children.push(Node::Paragraph(arena.boxed(ferromark::ast::Paragraph {
            children,
            span: Span::new(0, 11),
        })));
        let mut doc = ferromark::ast::Document {
            children: doc_children,
            front_matter: None,
            span: Span::new(0, 11),
        };

        let renderer_options = HtmlRendererOptions::default();
        let ctx = TransformContext::new(&arena, "", &renderer_options);
        let options =
            TypographyOptions::new(TypographyLanguage::English).with_guillemet_digraphs(guillemets);
        TypographyPass::new(options).apply(&mut doc, &ctx).unwrap();

        let html = HtmlRenderer::with_options(renderer_options).render(&doc);
        assert_eq!(
            html, "<p><i>abcd</i>…</p>\n",
            "zero-width span 5..5 outside raw HTML should still transform (guillemets={guillemets})"
        );
    }
}

#[test]
fn single_trailing_emoji_replacement_does_not_reallocate_large_sibling_list() {
    let mut source = "*word* ".repeat(700);
    source.push_str(":rocket:");
    let arena = Allocator::for_source_len(source.len());
    let mut doc = Parser::with_options(&arena, &source, ParserOptions::default())
        .parse()
        .expect("parse");
    let before_bytes = arena.allocated_bytes();
    let renderer_options = HtmlRendererOptions::default();
    let ctx = TransformContext::new(&arena, &source, &renderer_options);

    EmojiShortcodesPass::new().apply(&mut doc, &ctx).unwrap();

    let after_bytes = arena.allocated_bytes();
    assert_eq!(
        after_bytes, before_bytes,
        "replacing one trailing emoji node in place should not grow the arena"
    );
    let html = HtmlRenderer::with_options(renderer_options).render(&doc);
    assert!(html.ends_with("🚀</p>\n"), "{html}");
}
