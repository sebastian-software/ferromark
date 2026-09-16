//! Diff-fuzzing helper: render stdin markdown to HTML on stdout.
use std::io::Read;

fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();
    let gfm = std::env::args().any(|a| a == "--gfm");
    let allocator = ferromark::allocator::Allocator::for_source_len(source.len());
    let options = if gfm {
        ferromark::parser::ParserOptions::gfm()
    } else {
        ferromark::parser::ParserOptions::default()
    };
    let parser = ferromark::parser::Parser::with_options(&allocator, &source, options);
    let doc = parser.parse().unwrap();
    let mut renderer = ferromark::renderer::HtmlRenderer::with_options(
        ferromark::renderer::HtmlRendererOptions::new(),
    );
    #[allow(clippy::print_stdout)]
    {
        print!("{}", renderer.render(&doc));
    }
}
