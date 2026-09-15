# Migrating to Ferromark v2

V2 is developed on `codex/v2` in the existing Ferromark repository. The original
v1 history and the separately developed v2 history are both retained. It is an
unpublished development baseline, not a drop-in stable upgrade.

## Rust

The parser uses an arena AST pipeline. For owned HTML output, v2 provides
`to_html` and `to_html_with_options`; `to_html_into` and
`to_html_into_with_options` append to a caller-owned `String`.

```rust
let html = ferromark::to_html("# Hello")?;
```

These helpers return `Result` and expose separate `ParserOptions` and
`HtmlRendererOptions`. They are v2 APIs rather than drop-in v1 signatures.
The append helpers leave the output unchanged on parse errors. See
[the Rust API guide](rust-api.md) for examples and AST access.

When using the AST directly, the source and allocator outlive the document.
Rust defaults pass raw HTML through; `sanitize: true` escapes raw HTML and
filters link/image schemes. There is no v1-compatible CLI or MDX `segment`,
`render`, or `to_component` API.

## Node.js

The package keeps `toHtml`, `toHtmlBuffer`, `Renderer`, `transform`,
`toHtmlWithHighlighter`, and `transformWithHighlighter`. All call the v2 engine.
Node defaults continue to escape raw HTML and filter link/image URL schemes.
Highlighter output is trusted HTML and is inserted verbatim. Errors propagate;
highlighter fallback and observers retain their existing JavaScript behavior.

| Area | v2 behavior |
| --- | --- |
| `tableColumnWidths`, `indentedCodeBlocks` | Removed; passing any of these throws an unknown-option error |
| `highlight`, `inlineFootnotes`, `allowLinkRefs` | Supported again; see [syntax and policy](optional-writing.md). Writing extensions default off; reference links default on |
| Heading IDs | v2 slug rules and duplicate suffixes; metadata follows the same rules |
| Footnotes | v2 markup and definition rendering, including unreferenced definitions |
| `tableColgroup`, `tableColumnNames` | CSS-addressable colgroup output replaces v1 numeric width hints |
| `linkBasePath` | Enables v2 site routing, including images/raw HTML root URLs and .md-to-index.html conversion |
| Highlighter callback | Receives both fenced and indented code blocks |
| New options | `tableAttributes`, `headingAttributes`, `wikiLinks`, `cjkEmphasis`, `mdx` |

The restored writing switches are opt-in in v2. Enable `highlight` and
`inlineFootnotes` explicitly where their syntax is required; inline notes use
v2's existing footnote output rather than promising v1 HTML compatibility.
`allowLinkRefs: false` also leaves definitions visible as ordinary Markdown.
Enabled-but-unused extensions have a measurable cost; see the
[performance assessment](reports/2026-09-15-optional-writing/README.md).

`allowHtml: false` escapes raw HTML even with `renderPolicy: 'trusted'`; it does
not disable HTML recognition in the parser. Node enables GFM tables, task lists,
and strikethrough, plus heading IDs and callouts; autolink literals and footnotes
remain opt-in. Consult `node/ferromark/index.d.mts` for the supported options.

MDX captures syntax and static island payloads; it does not compile or execute
JavaScript. The facade, native packages, and platform binaries remain unpublished.
Build and pack locally using [the package checks](releasing.md#local-package-checks).
