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
