use std::error::Error;
use std::io::Write;

use ferromark::ast::{Document, Node};
use ferromark::{Allocator, HtmlRenderer, parse};
use ferromark_transforms::{BoxError, TransformContext, TransformPass, TransformPipeline};

struct AddPeriod;

impl TransformPass for AddPeriod {
    fn name(&self) -> &'static str {
        "add-period"
    }

    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        for node in &mut document.children {
            let Node::Paragraph(paragraph) = node else {
                continue;
            };
            let Some(Node::Text(text)) = paragraph.children.last_mut() else {
                continue;
            };
            let value = format!("{}.", text.value);
            context.replace_text_value(text, &value);
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arena = Allocator::new();
    let mut renderer = HtmlRenderer::new();
    let mut pipeline = TransformPipeline::new();

    // The pass instance is reused, but it stores no references into either
    // document. Dropping the document and resetting the arena is therefore safe.
    pipeline.add(AddPeriod);
    for source in ["Hello", "Again"] {
        {
            let mut document = parse(&arena, source)?;
            let context = TransformContext::new(&arena, source);
            pipeline.run(&mut document, &context)?;
            std::io::stdout().write_all(renderer.render(&document).as_bytes())?;
        }
        arena.reset();
    }

    Ok(())
}
