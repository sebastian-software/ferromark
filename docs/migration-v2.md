# Migrating to Ferromark v2

V2 is developed on `codex/v2` in the existing Ferromark repository. The original
v1 history and the separately developed v2 history are both retained. It is an
unpublished development baseline, not a drop-in stable upgrade.

## Rust

The streaming renderer becomes an arena AST pipeline:

```rust
use ferromark::{Allocator, HtmlRenderer, Parser};
let source = "# Hello";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::new(&allocator, source).parse()?;
let html = HtmlRenderer::new().render(&document);
```

The source and allocator outlive the document. Use `ParserOptions` for syntax
and `HtmlRendererOptions` for output. Rust defaults pass raw HTML through;
`sanitize: true` escapes raw HTML and filters link/image schemes. There is no
v1-compatible CLI or MDX `segment`, `render`, or `to_component` API.

## Node.js

The package keeps `toHtml`, `toHtmlBuffer`, `Renderer`, `transform`,
`toHtmlWithHighlighter`, and `transformWithHighlighter`. All call the v2 engine.
Node defaults continue to escape raw HTML and filter link/image URL schemes.
Highlighter output is trusted HTML and is inserted verbatim. Errors propagate;
highlighter fallback and observers retain their existing JavaScript behavior.

| Area | v2 behavior |
| --- | --- |
| `tableColumnWidths`, `highlight`, `inlineFootnotes`, `allowLinkRefs`, `indentedCodeBlocks` | Removed; passing any of these throws an unknown-option error |
| Heading IDs | v2 slug rules and duplicate suffixes; metadata follows the same rules |
| Footnotes | v2 markup and definition rendering, including unreferenced definitions |
| `tableColgroup`, `tableColumnNames` | CSS-addressable colgroup output replaces v1 numeric width hints |
| `linkBasePath` | Enables v2 site routing, including images/raw HTML root URLs and .md-to-index.html conversion |
| Highlighter callback | Receives both fenced and indented code blocks |
| New options | `tableAttributes`, `headingAttributes`, `wikiLinks`, `cjkEmphasis`, `mdx` |

`allowHtml: false` escapes raw HTML even with `renderPolicy: 'trusted'`; it does
not disable HTML recognition in the parser. Node enables GFM tables, task lists,
and strikethrough, plus heading IDs and callouts; autolink literals and footnotes
remain opt-in. Consult `node/ferromark/index.d.mts` for the supported options.

MDX captures syntax and static island payloads; it does not compile or execute
JavaScript. The facade, native packages, and platform binaries remain unpublished.
Build and pack locally using [the package checks](releasing.md#local-package-checks).
