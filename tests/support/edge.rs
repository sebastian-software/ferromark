use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

pub fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    renderer.render(&doc)
}
