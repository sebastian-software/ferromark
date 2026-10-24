//! Inspect image provenance and extract callout prose without structural markers.

use ferromark::ast::{InlineCode, Node, Text, Visit};
use ferromark::{Allocator, CalloutKind, Parser, ParserOptions};

#[derive(Default)]
struct Prose(String);
impl<'a> Visit<'a> for Prose {
    fn visit_text(&mut self, text: &Text<'a>) {
        self.0.push_str(text.value);
    }
    fn visit_inline_code(&mut self, code: &InlineCode<'a>) {
        self.0.push_str(code.value);
    }
}

fn main() -> Result<(), ferromark::ParseError> {
    let source =
        "![Code `sample`](sample.png)\n\n> [!NOTE]\n> Keep this **note**.\n>\n> Second paragraph.";
    let arena = Allocator::new();
    let document = Parser::with_options(
        &arena,
        source,
        ParserOptions {
            image_alt_content: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()?;
    let Node::Paragraph(paragraph) = &document.children[0] else {
        unreachable!()
    };
    let Node::Image(image) = &paragraph.children[0] else {
        unreachable!()
    };
    assert_eq!(image.alt, "Code sample");
    let alt = image.alt_content.as_ref().unwrap();
    let Node::InlineCode(code) = &alt.children[1] else {
        unreachable!()
    };
    assert_eq!(code.span.source_text(source), "`sample`");

    let Node::BlockQuote(quote) = &document.children[1] else {
        unreachable!()
    };
    let callout = quote.callout(source).unwrap();
    assert_eq!(callout.kind, CalloutKind::Note);
    assert_eq!(callout.marker_span.source_text(source), "[!NOTE]");
    let mut prose = Prose::default();
    for (index, node) in callout.body.inlines.iter().enumerate() {
        if index == 0
            && let Node::Text(text) = node
        {
            prose.0.push_str(&text.value[callout.body.text_offset..]);
        } else {
            prose.visit_node(node);
        }
    }
    for block in callout.body.blocks {
        prose.0.push('\n');
        prose.visit_node(block);
    }
    assert_eq!(prose.0, "Keep this note.\nSecond paragraph.");
    Ok(())
}
