//! Project static Markdown from an already parsed MDX document.

use ferromark::{Allocator, MarkdownProjectionOptions, Parser, ParserOptions};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "import Box from './box.js';\n\n# Guide\n\n<Box>**Static** [link](/guide) {dynamic}</Box>\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            mdx: true,
            mdx_compatible: cfg!(feature = "jsx"),
            ..ParserOptions::default()
        },
    )
    .parse()?;
    let result = document.project_markdown(
        source,
        &MarkdownProjectionOptions {
            omit_title_heading: Some("Guide".into()),
            ..MarkdownProjectionOptions::default()
        },
    )?;
    assert!(result.markdown.contains("**Static** [link](/guide)"));
    assert!(!result.markdown.contains("{dynamic}"));
    io::stdout().lock().write_all(result.markdown.as_bytes())?;
    let mut stderr = io::stderr().lock();
    for diagnostic in result.diagnostics {
        writeln!(stderr, "{:?}: {:?}", diagnostic.kind, diagnostic.span)?;
    }
    Ok(())
}
