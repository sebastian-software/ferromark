# Build a link and asset inventory

This additive API is available on repository `main` after the reference
extraction change lands; check your installed package's types before using it.

Use the parsed document to collect reference occurrences without rendering or
parsing Markdown again. Destinations remain parser-resolved strings: your
application supplies URL classification, route lookup, base URLs, filesystem
checks, and network requests.

## Node

`prepareDocument` owns the source, runs the selected syntax and native passes
once, and returns the same immutable handle shape as `JsxCompiler.prepare`.
It does not construct a Ferriki highlighter or load themes or grammars. Its
optional preparation settings are exactly `JsxPreparationOptions`; rendering
choices belong to `render` and `renderModule`.

```js
import { prepareDocument } from "ferromark";

const source = '[Guide][guide] ![Logo](./logo.svg)\n\n[guide]: /guide\n';
const document = prepareDocument(source);
const inventory = document.collectReferences();
for (const reference of inventory) {
  console.log(reference.kind, reference.destination, reference.span);
}
// link /guide { start: 0, end: 14 }
// image ./logo.svg { start: 15, end: 34 }
// Apply your application's route or asset rules to each destination here.
```

Call `collectReferences()` on a compiler-prepared handle to reuse its exact
AST and passes. Repeated collection does not reparse, rerun passes, highlight,
change metadata, or change subsequent JSX body/module rendering. Each call
returns a fresh frozen array of frozen owned occurrences and ranges. Extraction
results are allocated on demand. When native passes are configured, preparation
retains borrowed reference identity data and collected labels for exact source
provenance comparisons; ordinary preparation without passes retains none.

Asset-free handles can also render ordinary JSX, including plain code blocks,
and accept the existing synchronous code callback. Compiler-prepared handles
retain their compiler's native highlighting settings and reentrancy behavior.

## Rust

```rust
use ferromark::{Allocator, Parser};

fn main() -> Result<(), ferromark::ParseError> {
    let source = "[Guide][guide] ![Logo](./logo.svg)\n\n[guide]: /guide\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse()?;
    for reference in document.collect_references(source) {
        println!("{:?}: {} {:?}", reference.kind, reference.destination, reference.span);
        // Resolve local paths or external URLs according to your application.
    }
    Ok(())
}
```

`Document::collect_references(source)` expects an unmodified parser tree and
its original input. For custom mutable passes, capture
`ReferenceSnapshot::capture(&document, source)` before mutation and call
`snapshot.collect(&document, source)` afterward. Keep the same document and
arena alive: snapshots borrow the original destination/title strings and use
arena node identities. Replaced nodes conservatively count as transformed.
The snapshot is explicit so ordinary Rust rendering incurs no bookkeeping.
The executable inventory is [examples/references.rs](../examples/references.rs).

## Occurrences and source ranges

Occurrences stay in original document order, including duplicates. Inline,
full/collapsed/shortcut reference links and images use the parser's definition
resolution, including definitions in different MDX segments. Definitions
alone are not occurrences; unresolved syntax stays ordinary text. CommonMark
angle autolinks and enabled GFM bare autolinks have `kind: "autolink"`.
Fenced code, indented code, and inline code produce no references.

Markdown entries expose their destination, available plain label/alt text,
and optional title. Literal MDX `href`/`src` entries use
`kind: "mdxAttribute"` with the authored `element` and `attribute` names.
Quoted values use the same entity decoding as JSX rendering. Custom component
attributes remain distinguishable from Markdown links; extraction does not
claim that a custom component renders a link or image.

`span` is a half-open range `[start, end)` of **original UTF-8 bytes**, including
Markdown occurrence syntax, or the complete MDX named attribute and quoted
value. A reference occurrence's range never points to its definition. Slice
`Buffer.from(source).subarray(span.start, span.end)` in Node; JavaScript string
indices count UTF-16 code units and cannot be substituted for these offsets.
For malformed UTF-8 byte input, the existing Node decoding policy replaces
invalid sequences. Ranges address that decoded text's UTF-8 bytes; re-encode
`Buffer.from(input.toString("utf8"))` before slicing malformed byte input.

Unchanged references have `provenance: "source"` and a `span`. When a native
pass changes a retained reference's destination, label, title, or attribute,
its provenance becomes `"transformed"` and `span` is absent; `originalSpan`
identifies the related original occurrence. Pass-generated references have
neither `span` nor `originalSpan`. A generated value is never assigned an
invented authored range. Rust represents absent ranges with `None`.

## Limits

Collection inventories authored static syntax, not all links produced at
runtime. Raw HTML is excluded; this API is not an HTML parser. In MDX mode,
HTML-shaped syntax parsed as JSX follows the literal attribute rules. Boolean
attributes, expression-valued attributes (even string literal expressions),
spreads, ESM, and embedded JavaScript are not evaluated or searched. Duplicate
literal attributes remain individual authored occurrences; a later attribute
or spread can override an earlier literal at runtime. An empty inventory does
not prove that a document or component has no runtime links.
