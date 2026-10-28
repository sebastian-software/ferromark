# Extract text and sections from a prepared document

Use `prepareDocument()` and `prepared.extractText()` to derive static document
text and ordered heading sections from the same native tree you render. This
API is available on repository `main` after this feature lands; check your
installed package's types before using it with an older published version.
Preparation parses source and runs configured native passes once. Extraction
walks that resulting tree without parsing again, rendering JSX, calling a
JavaScript node callback, highlighting, or loading themes and grammars.

```js
import { prepareDocument } from "ferromark";

const source = "# Guide\n\nVisible **prose** and `inline code`.\n\n## Setup\n\nInstall the package.\n";
const prepared = prepareDocument(source);
const extracted = prepared.extractText({ includeBlockCode: false });
console.log(extracted.sections[1].id); // "setup"
console.log(extracted.sections[1].parentIndex); // 0
console.log(extracted.sections[1].text); // "Setup\n\nInstall the package."
```

`CodeCompiler.prepare()` returns the same handle with compiler highlighting
available for rendering. `extractText()` does not use that highlighter. Choose
`format: "mdx"` at preparation time for static children of MDX wrappers. Pass
settings and syntax options belong to preparation; extraction choices apply
only to each projection. The public TypeScript types own the option list.

## Text policy

The projection includes decoded prose and escaped syntax, link labels, image
alt text, inline code, literal TeX without math delimiters, captions, definition
list content, and footnote bodies. Link destinations and titles, footnote
reference labels and backlinks, front matter, ESM imports/exports, expressions,
and MDX attributes do not contribute text. Images contribute their parsed alt
text, even if a runtime component renders them differently. Fenced and indented
code is included by default; `includeBlockCode: false` excludes it while
retaining inline code.

Raw HTML nodes are omitted. Inline raw HTML tags can surround parsed Markdown
text, which remains visible; text in `script`, `style`, `textarea`, and `title`
elements is conservatively omitted. Raw-text state is scoped to each parsed
paragraph, heading, table cell, and caption, so malformed HTML in one run does
not suppress unrelated runs or depend on caption display order. In HTML, a
self-closing slash does not make these four elements void. The same four
intrinsic MDX elements omit static children. Other supported MDX wrappers
contribute only parsed static children. Self-closing components and expressions create a space boundary so
omitting unknown content does not merge surrounding words. Imports, JavaScript,
components, and attribute expressions are never evaluated. Component rendering
and runtime-only content cannot be inferred from this data.

Prose whitespace collapses to ASCII spaces. Paragraphs and blocks use two LF
characters, list items and table rows use one LF, and table cells use tabs,
including empty cells. Inline breaks use a space. Block code preserves the
parser's normalized whitespace except final LF characters. Callouts use their
static title label and body when enabled; with `callouts: false`, the authored
marker stays in prose. Leading and trailing pending separators are omitted.

## Final headings and section boundaries

Each section includes a JSX heading label, effective level, optional final ID,
nearest earlier parent section index, normalized text range, and owned text.
Sections follow tree traversal order. A section includes its heading and text
up to the next heading; descendants have their own sections and do not repeat
inside a parent's text. Preamble text stays in the document's `text` field.
Skipped heading levels are valid. Parent indexes use effective levels after
offset and clamping, so clamped peers are siblings.

Use matching `headingIds`, `headingOffset`, `headingIdPrefix`,
`omitTitleHeading`, and `callouts` choices when extracting and rendering. The
same prepared tree produces matching native JSX body/module IDs, including
duplicates and collisions with authored element/footnote IDs. Title omission
checks only the first top-level authored H1; an omitted title does not reserve
an ID or produce a section. Preparation-time `metadata.outline` does not stand
in for this final plan. Heading labels follow JSX's existing static label rules
and omit image alt text and math, even though those contribute document text.

All ranges are half-open. `headingSpan` and `sourceSpan` count original UTF-8
source bytes. `headingSpan` includes Markdown syntax; `sourceSpan` runs from
that heading to the next heading or document end. Missing or reordered heading
provenance makes `sourceSpan` unavailable. These boundaries are not a per-token
source map. Native passes can change text while retaining original spans;
generated empty spans remain unavailable.

`textRange` counts UTF-8 bytes in normalized `extracted.text`. JavaScript string
indexes count UTF-16 code units, so use `section.text` directly or encode before
slicing:

```js
const bytes = Buffer.from(extracted.text, "utf8");
for (const section of extracted.sections) {
  console.log(bytes.subarray(section.textRange.start, section.textRange.end).toString("utf8"));
}
```

Results and nested records are immutable in Node. Repeated extraction does not
mutate the prepared tree, rerun passes, or affect later rendering. Each call
returns an owned result; no analysis cache is stored on the document. Routes,
locales, versions, ranking, excerpts, and search engines remain caller choices.

## Rust and executable examples

Rust uses `Document::extract_text()` directly after parsing and any native
passes. `TextExtractionOptions::rendering` takes matching `JsxRendererOptions`
for final heading choices. No feature flag or highlighting dependency is needed.

```rust
use ferromark::{Allocator, Parser, TextExtractionOptions};

fn main() -> Result<(), ferromark::ParseError> {
    let source = "# Guide\n\n## Setup\n\nInstall the package.\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse()?;
    let extracted = document.extract_text(&TextExtractionOptions::default());
    assert_eq!(extracted.sections[1].id.as_deref(), Some("setup"));
    assert_eq!(extracted.sections[1].parent_index, Some(0));
    Ok(())
}
```

Run `cargo run --example extract_text --locked` and, after the local Node build,
`node node/examples/extract-text.mjs`. Both examples consume bulk sections and
compare normalized byte ranges without a second Markdown parser.

## Measure extraction separately

The Criterion `text_extraction` suite measures parsing in one region and text
extraction from a prepared tree in separate regions with block code included
and excluded. Extraction includes owned output allocation and section text
copies. It excludes parsing, native pass preparation, JSX generation,
highlighting, and Node boundary conversion. The fixed fixture repeats 64 times
and validates heading IDs against JSX output before timing.

```sh
cargo bench --bench text_extraction --features jsx --locked -- --sample-size 10 --warm-up-time 0.5 --measurement-time 1
```

See [the initial cost record](reports/2026-10-11-text-extraction/README.md) for
machine context and independently measured parsing/extraction costs. No
comparative speed claim is made. Measure the Node boundary and any chosen
highlighting workload separately for your application.
