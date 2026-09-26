#![allow(clippy::panic, clippy::unwrap_used)]

use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark_transforms::{
    EmojiShortcodesPass, GitHubReferencesPass, TransformContext, TransformPipeline,
    TypographyLanguage, TypographyOptions, TypographyPass,
};

fn render(source: &str, mut pipeline: TransformPipeline) -> String {
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    pipeline.run(&mut document, &context).unwrap();
    HtmlRenderer::with_options(renderer_options).render(&document)
}

#[test]
fn links_commit_ranges_before_typography_can_replace_the_separator() {
    let source = "Compare e2acebc...2aa9311.";
    let mut github_first = TransformPipeline::new();
    github_first.add(GitHubReferencesPass::new("ferromark/fixtures").unwrap());
    github_first.add(TypographyPass::new(TypographyOptions::new(
        TypographyLanguage::English,
    )));
    assert_eq!(
        render(source, github_first),
        "<p>Compare <a href=\"https://github.com/ferromark/fixtures/compare/e2acebc...2aa9311\" target=\"_blank\" rel=\"noopener noreferrer\">e2acebc…2aa9311</a>.</p>\n"
    );

    let mut typography_first = TransformPipeline::new();
    typography_first.add(TypographyPass::new(TypographyOptions::new(
        TypographyLanguage::English,
    )));
    typography_first.add(GitHubReferencesPass::new("ferromark/fixtures").unwrap());
    assert_eq!(
        render(source, typography_first),
        "<p>Compare <a href=\"https://github.com/ferromark/fixtures/commit/e2acebc\" target=\"_blank\" rel=\"noopener noreferrer\">e2acebc</a>…<a href=\"https://github.com/ferromark/fixtures/commit/2aa9311\" target=\"_blank\" rel=\"noopener noreferrer\">2aa9311</a>.</p>\n"
    );
}

#[test]
fn emoji_and_french_spacing_compose_on_the_same_document() {
    let source = "texte :smile: ;";
    let mut pipeline = TransformPipeline::new();
    pipeline.add(EmojiShortcodesPass::new());
    pipeline.add(TypographyPass::new(TypographyOptions::new(
        TypographyLanguage::French,
    )));
    assert_eq!(render(source, pipeline), "<p>texte 😄 ;</p>\n");
}
