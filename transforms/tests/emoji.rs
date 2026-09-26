#![allow(clippy::panic, clippy::unwrap_used)]

use ferromark::ast::{Node, Span};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark_transforms::{
    EmojiShortcodesPass, TransformContext, TransformPass, TransformPipeline,
};

fn render(source: &str, parser_options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    EmojiShortcodesPass::new()
        .apply(&mut document, &context)
        .unwrap();
    HtmlRenderer::with_options(renderer_options).render(&document)
}

#[test]
fn replaces_versioned_aliases_and_leaves_unknown_or_mis_cased_names_alone() {
    assert_eq!(
        render(
            ":white_check_mark: :+1: :woman_technologist: :heart: :not_real: :Smile: :other:smile: :smile::rocket:",
            ParserOptions::default()
        ),
        "<p>✅ 👍 👩‍💻 ❤️ :not_real: :Smile: :other😄 😄🚀</p>\n"
    );
}

#[test]
fn visits_fragmented_text_runs_and_emits_source_spans() {
    let source = ":white_check_mark:";
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    EmojiShortcodesPass::new()
        .apply(&mut document, &context)
        .unwrap();

    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected transformed text");
    };
    assert_eq!(text.value, "✅");
    assert_eq!(text.span, Span::new(0, source.len() as u32));
    assert_eq!(
        HtmlRenderer::with_options(renderer_options).render(&document),
        "<p>✅</p>\n"
    );
}

#[test]
fn leaves_code_html_math_mdx_image_metadata_and_destinations_untouched() {
    let source = r#"`code :smile:` <span title=":rocket:">:smile:</span> $:rocket:$ ![alt :smile:](image-:rocket: ":smile:") {value(":rocket:")}"#;
    let parser_options = ParserOptions {
        math: true,
        mdx: true,
        ..ParserOptions::default()
    };
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    EmojiShortcodesPass::new()
        .apply(&mut document, &context)
        .unwrap();

    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::InlineCode(code) if code.value == "code :smile:"))
    );
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::InlineMath(math) if math.value == ":rocket:"))
    );
    assert!(paragraph.children.iter().any(
        |node| matches!(node, Node::Image(image) if image.alt == "alt :smile:" && image.url == "image-:rocket:" && image.title == Some(":smile:"))
    ));
    assert!(paragraph.children.iter().any(
        |node| matches!(node, Node::MdxTextExpression(expression) if expression.value == "value(\":rocket:\")")
    ));
    assert!(
        paragraph
            .children
            .iter()
            .any(|node| matches!(node, Node::Text(text) if text.value == ":smile:"))
    );
}

#[test]
fn protects_bare_urls_in_default_and_gfm_profiles_and_transforms_visible_labels() {
    let source = "See https://host/:smile: and [label :rocket:](/guide \"title :smile:\").";
    assert_eq!(
        render(source, ParserOptions::default()),
        "<p>See <a href=\"https://host/:smile\" target=\"_blank\" rel=\"noopener noreferrer\">https://host/:smile</a>: and <a href=\"/guide\" title=\"title :smile:\">label 🚀</a>.</p>\n"
    );

    let gfm = ParserOptions::gfm();
    assert_eq!(
        render("See www.example.org/:smile: and https://host/:smile:.", gfm),
        "<p>See <a href=\"http://www.example.org/:smile\" target=\"_blank\" rel=\"noopener noreferrer\">www.example.org/:smile</a>: and <a href=\"https://host/:smile\" target=\"_blank\" rel=\"noopener noreferrer\">https://host/:smile</a>:.</p>\n"
    );
}

#[test]
fn is_idempotent_and_changes_heading_text_before_html_slugging() {
    let source = "# Ship it :rocket:";
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions {
        heading_ids: true,
        ..HtmlRendererOptions::default()
    };
    let context = TransformContext::new(&allocator, source, &renderer_options);
    let mut pipeline = TransformPipeline::new();
    pipeline.add(EmojiShortcodesPass::new());
    pipeline.add(EmojiShortcodesPass::new());
    pipeline.run(&mut document, &context).unwrap();

    assert_eq!(
        HtmlRenderer::with_options(renderer_options).render(&document),
        "<h1 id=\"ship-it\">Ship it 🚀</h1>\n"
    );
}
