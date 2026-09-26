# ferromark-transforms

`ferromark-transforms` adds an optional, ordered pass pipeline over
Ferromark's native arena-backed AST. The core `ferromark` crate does not depend
on this package, so parser-only and renderer-only users do not pull it in.

```rust
use ferromark::ast::{Document, Node};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, parse};
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
            if let Node::Paragraph(paragraph) = node {
                if let Some(Node::Text(text)) = paragraph.children.last_mut() {
                    let value = format!("{}.", text.value);
                    context.replace_text_value(text, &value);
                }
            }
        }
        Ok(())
    }
}

let source = "Hello";
let allocator = Allocator::new();
let mut document = parse(&allocator, source)?;
let renderer_options = HtmlRendererOptions::new();
let context = TransformContext::new(&allocator, source, &renderer_options);
let mut pipeline = TransformPipeline::new();
pipeline.add(AddPeriod);
pipeline.run(&mut document, &context)?;

let mut renderer = HtmlRenderer::with_options(renderer_options);
assert_eq!(renderer.render(&document), "<p>Hello.</p>\n");
# Ok::<(), Box<dyn std::error::Error>>(())
```

See [`examples/custom_pass.rs`](examples/custom_pass.rs) for a pipeline reused
across separate documents and arena resets.

## Contract

- Passes run in insertion order. The first failure stops later passes and
  reports its zero-based index, name, and original error.
- A failed pass can have partially changed the document. Discard or reparse
  that document; the pipeline does not roll changes back.
- Passes may keep configuration and reusable state, but document-specific
  references must stay inside `apply`. The pipeline keeps no arena references.
  Passes are `Send` so a configured pipeline can move to the document's worker;
  the document and context remain tied to their parsing thread.
- `text_runs` groups adjacent `Text` siblings and exposes each segment's source
  span. It does not cross formatting nodes or choose which nested content is
  prose; the pass owns that traversal policy.
- Replacing text preserves the span being replaced. `replace_text_range`
  preserves source spans for unchanged text and gives a replacement the
  bounding span of the source nodes it covers. Insertions use `Span::empty()`;
  that value is not a distinct provenance marker.
- URL ranges are computed only when a pass explicitly asks for them. A
  `TextRun` scans each original text segment independently, using the matcher
  built from the same `HtmlRendererOptions` passed to `TransformContext`.
  Disabled URL autolinking and custom prefixes therefore match the renderer.

## Build a caller-placed table of contents

The optional TOC helper turns a core `Document::outline` into a nested list AST
node. It never scans for a Markdown marker or chooses an insertion location:

```rust
let outline = document.outline(&OutlineOptions::default());
if let Some(toc) = build_table_of_contents(&allocator, &outline)? {
    document.children.insert(0, toc);
}
```

Compute the outline after other passes and use ID settings that match the
renderer. An empty outline produces no node; an outline without heading IDs is
an error. Generated nodes use `Span::empty()` and describe one complete
document. See the [Rust API guide](../docs/rust-api.md#document-outline-and-table-of-contents)
and the [TOC placement decision](../docs/decisions/2026-09-26-caller-placed-table-of-contents.md).
