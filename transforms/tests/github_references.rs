#![allow(clippy::panic, clippy::unwrap_used)]

use ferromark::ast::{Node, Span};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark_transforms::{
    GitHubReferencesPass, TransformContext, TransformPass, TransformPipeline,
};

fn render(source: &str, parser_options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    GitHubReferencesPass::new("ferromark/fixtures")
        .unwrap()
        .apply(&mut document, &context)
        .unwrap();
    HtmlRenderer::with_options(renderer_options).render(&document)
}

#[test]
fn links_qualified_issues_mentions_and_commits_with_authored_labels() {
    assert_eq!(
        render(
            "Fixes #123 and closes gH-7; see org/my_repo#9, @octocat and @org/team. Commit 1f2a4fb and range e2acebc...2aa9311.",
            ParserOptions::default()
        ),
        "<p>Fixes <a href=\"https://github.com/ferromark/fixtures/issues/123\" target=\"_blank\" rel=\"noopener noreferrer\">#123</a> and closes <a href=\"https://github.com/ferromark/fixtures/issues/7\" target=\"_blank\" rel=\"noopener noreferrer\">gH-7</a>; see <a href=\"https://github.com/org/my_repo/issues/9\" target=\"_blank\" rel=\"noopener noreferrer\">org/my_repo#9</a>, <a href=\"https://github.com/octocat\" target=\"_blank\" rel=\"noopener noreferrer\">@octocat</a> and <a href=\"https://github.com/org/team\" target=\"_blank\" rel=\"noopener noreferrer\">@org/team</a>. Commit <a href=\"https://github.com/ferromark/fixtures/commit/1f2a4fb\" target=\"_blank\" rel=\"noopener noreferrer\">1f2a4fb</a> and range <a href=\"https://github.com/ferromark/fixtures/compare/e2acebc...2aa9311\" target=\"_blank\" rel=\"noopener noreferrer\">e2acebc...2aa9311</a>.</p>\n"
    );
}

#[test]
fn leaves_emails_invalid_boundaries_and_hash_like_words_alone() {
    assert_eq!(
        render(
            "foo@bar.com; @mention; @mentions; @octocat.computer; x#123; #0; #123word; acceded; a1b2c3.",
            ParserOptions::default()
        ),
        "<p>foo@bar.com; @mention; @mentions; @octocat.computer; x#123; #0; #123word; acceded; a1b2c3.</p>\n"
    );
}

#[test]
fn protects_existing_links_bare_urls_code_html_and_math_without_nested_links() {
    let source = "[existing #12](https://example.com/#34) https://example.com/#56 `#78` <i>#90</i> $#91$ #92";
    let parser_options = ParserOptions {
        math: true,
        ..ParserOptions::default()
    };
    let html = render(source, parser_options);
    assert!(
        html.contains("<a href=\"https://example.com/#34\" target=\"_blank\" rel=\"noopener noreferrer\">existing #12</a>"),
        "{html}"
    );
    assert!(
        html.contains("<a href=\"https://example.com/#56\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.com/#56</a>"),
        "{html}"
    );
    assert!(html.contains("<code>#78</code>"), "{html}");
    assert!(html.contains("<i>#90</i>"), "{html}");
    assert!(html.contains("#91"), "{html}");
    assert!(
        html.ends_with("<a href=\"https://github.com/ferromark/fixtures/issues/92\" target=\"_blank\" rel=\"noopener noreferrer\">#92</a></p>\n"),
        "{html}"
    );
    assert_eq!(html.matches("<a ").count(), 3, "{html}");
}

#[test]
fn protects_gfm_autolinks_and_large_issue_numbers() {
    let source =
        "https://host/#123 and www.example.org/#456; #999999999999999999999999999999999999.";
    let html = render(source, ParserOptions::gfm());
    assert!(
        html.contains("href=\"https://host/#123\" target=\"_blank\" rel=\"noopener noreferrer\">https://host/#123</a>"),
        "{html}"
    );
    assert!(
        html.contains("href=\"http://www.example.org/#456\" target=\"_blank\" rel=\"noopener noreferrer\">www.example.org/#456</a>"),
        "{html}"
    );
    assert!(
        html.contains("href=\"https://github.com/ferromark/fixtures/issues/999999999999999999999999999999999999\" target=\"_blank\" rel=\"noopener noreferrer\">#999999999999999999999999999999999999</a>"),
        "{html}"
    );
}

#[test]
fn replaces_fragmented_reference_runs_and_preserves_source_span() {
    let source = "org/my_repo#1";
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    GitHubReferencesPass::new("ferromark/fixtures")
        .unwrap()
        .apply(&mut document, &context)
        .unwrap();

    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    let Node::Link(link) = &paragraph.children[0] else {
        panic!("expected a reference link");
    };
    assert_eq!(link.url, "https://github.com/org/my_repo/issues/1");
    assert_eq!(link.span, Span::new(0, source.len() as u32));
    assert!(
        matches!(link.children.first(), Some(Node::Text(text)) if text.value == source && text.span == link.span)
    );
    assert_eq!(paragraph.children.len(), 1, "references do not nest links");
}

#[test]
fn repeated_application_does_not_relink_generated_nodes() {
    let source = "Fix #123 and @octocat";
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    let renderer_options = HtmlRendererOptions::default();
    let context = TransformContext::new(&allocator, source, &renderer_options);
    let mut pipeline = TransformPipeline::new();
    pipeline.add(GitHubReferencesPass::new("ferromark/fixtures").unwrap());
    pipeline.add(GitHubReferencesPass::new("ferromark/fixtures").unwrap());
    pipeline.run(&mut document, &context).unwrap();

    assert_eq!(
        HtmlRenderer::with_options(renderer_options).render(&document),
        "<p>Fix <a href=\"https://github.com/ferromark/fixtures/issues/123\" target=\"_blank\" rel=\"noopener noreferrer\">#123</a> and <a href=\"https://github.com/octocat\" target=\"_blank\" rel=\"noopener noreferrer\">@octocat</a></p>\n"
    );
}
