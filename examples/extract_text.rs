//! Extract sections from the same document that can be passed to a renderer.
use ferromark::{Allocator, Parser, TextExtractionOptions};

fn main() -> Result<(), ferromark::ParseError> {
    let source =
        "# Guide\n\nVisible **prose** and `inline code`.\n\n## Setup\n\nInstall the package.\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse()?;
    let extracted = document.extract_text(&TextExtractionOptions {
        include_block_code: false,
        ..TextExtractionOptions::default()
    });
    assert_eq!(extracted.sections[1].parent_index, Some(0));
    assert_eq!(extracted.sections[1].id.as_deref(), Some("setup"));
    assert_eq!(extracted.sections[1].text, "Setup\n\nInstall the package.");
    // Text offsets index normalized UTF-8; heading spans index original source.
    for section in &extracted.sections {
        assert_eq!(
            &extracted.text[section.text_range.start..section.text_range.end],
            section.text
        );
    }
    Ok(())
}
