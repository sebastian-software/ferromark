mod autolink;
mod blocks;
mod code;
mod inline_media;
mod links;
mod lists_tables;
mod mdx;
mod mdx_islands;

use crate::allocator::Allocator;
use crate::parser::Parser;
use crate::renderer::html::{HtmlRenderer, HtmlRendererOptions};

/// Short documents carry markup overhead that is not proportional to their
/// source, so the output estimate has a floor and one reservation covers the
/// whole render. An empty document is the deliberate exception: it renders to
/// nothing and must still hand back a string that never allocated.
#[test]
fn short_documents_are_sized_in_one_reservation() {
    let allocator = Allocator::new();
    for source in ["# Hi", "- a\n- b", "Hello", "> quote", "`x`"] {
        let doc = Parser::new(&allocator, source).parse().unwrap();
        let html = HtmlRenderer::new().render(&doc);
        assert!(
            html.capacity() >= html.len().max(super::renderer::MIN_OUTPUT_CAPACITY),
            "{source:?} grew to {} bytes out of {} reserved",
            html.len(),
            html.capacity()
        );
        let mut reused = HtmlRenderer::new();
        assert_eq!(html, reused.render_borrowed(&doc), "{source:?}");
    }

    let empty = Parser::new(&allocator, "").parse().unwrap();
    let html = HtmlRenderer::new().render(&empty);
    assert_eq!(html, "");
    assert_eq!(html.capacity(), 0, "empty source must not allocate output");
}

#[test]
fn static_default_options_match_owned_defaults() {
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "# Title\n\n> [!NOTE]\n> Visit https://example.com and use **care**.\n\nLine  \\nnext",
    )
    .parse()
    .unwrap();

    let fast = HtmlRenderer::new().render(&doc);
    let owned = HtmlRenderer::with_options(HtmlRendererOptions::default()).render(&doc);

    assert_eq!(fast, owned);
}
