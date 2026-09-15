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
