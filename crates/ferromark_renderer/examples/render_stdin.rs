//! Diff-fuzzing helper: render stdin markdown to HTML on stdout.
use std::io::Read;

fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();
    let gfm = std::env::args().any(|a| a == "--gfm");
    let allocator = ferromark_allocator::Allocator::for_source_len(source.len());
    let options = if gfm {
        ferromark_parser::ParserOptions::gfm()
    } else {
        ferromark_parser::ParserOptions::default()
    };
    let parser = ferromark_parser::Parser::with_options(&allocator, &source, options);
    let doc = parser.parse().unwrap();
    let mut renderer = ferromark_renderer::HtmlRenderer::with_options(
        ferromark_renderer::HtmlRendererOptions::new(),
    );
    #[allow(clippy::print_stdout)]
    {
        print!("{}", renderer.render(&doc));
    }
}
