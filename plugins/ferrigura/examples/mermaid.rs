//! Render Markdown from stdin with optional Mermaid SVG and fence diagnostics.

use std::io::{Read, Write};

use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser};
use ferromark_ferrigura::{
    FerriguraHooks,
    ferrigura::{Renderer, Value},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source)?;
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, &source).parse()?;
    let mut hooks = FerriguraHooks::new(Renderer::new(&Value::empty_object()), "page")?;
    let options = HtmlRendererOptions {
        sanitize: true,
        ..Default::default()
    };
    let html = HtmlRenderer::with_options(options).render_with_code_hooks(&document, &mut hooks);
    std::io::stdout().write_all(html.as_bytes())?;
    for diagnostic in hooks.take_diagnostics() {
        writeln!(
            std::io::stderr(),
            "{}..{}: {}",
            diagnostic.span.start,
            diagnostic.span.end,
            diagnostic.message
        )?;
    }
    Ok(())
}
