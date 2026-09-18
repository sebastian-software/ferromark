# Migrating to Ferromark v2

V2 lives in the existing Ferromark repository. The original v1 history and the
separately developed v2 history are both retained. The first release candidate is
`2.0.0-rc.1`; it is a breaking upgrade intended for testing before stable v2.

## Rust

V2 publishes one crate: `ferromark`. Its `allocator`, `ast`, `parser`, and
`renderer` modules expose advanced APIs, while top-level imports such as
`ferromark::Parser` and `ferromark::HtmlRenderer` remain available. The four
separate crates from the development branch were consolidated before publication;
replace imports like `ferromark_ast::Node` with `ferromark::ast::Node`.

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

### Renderer option strings take `.into()`

After `2.0.0-rc.1`, the string-valued fields of `HtmlRendererOptions` are
`Cow<'static, str>` and `autolink_patterns` is `Cow<'static, [Cow<'static, str>]>`,
so the documented defaults are borrowed from static data instead of being
rebuilt on the heap for every options value. Assignments need `.into()`:

```rust
// before
HtmlRendererOptions {
    base_url: "/docs/".to_string(),
    autolink_patterns: vec!["mailto:".to_string()],
    ..HtmlRendererOptions::default()
}

// after
HtmlRendererOptions {
    base_url: "/docs/".into(),
    autolink_patterns: vec!["mailto:".into()].into(),
    ..HtmlRendererOptions::default()
}
```

A `&'static str` borrows, an owned `String` moves in, and a value that is
neither must be owned first (`value.to_string().into()`). Reads are unchanged:
the fields still deref to `str`. Rendered HTML is unchanged, including for empty
strings and an empty pattern list, which keep their existing meaning. See the
[decision record](decisions/2026-09-15-borrowed-renderer-options.md).

When using the AST directly, the source and allocator outlive the document.
Rust defaults pass raw HTML through; `sanitize: true` escapes raw HTML and
filters link/image schemes. There is no v1-compatible CLI or MDX `segment`,
`render`, or `to_component` API.

### Changes after `2.0.0-rc.2`

The API freeze before the stable 2.0.0 release removed dead and no-op surface
and corrected two names. All of it is breaking for release-candidate users only.
See the [API surface decision](decisions/2026-09-17-api-surface.md).

**Removed types.** The `Renderer` trait, `RenderError` and `RenderResult` are
gone. `HtmlRenderer::render` is an inherent method returning `String`; rendering
cannot fail, so drop the `?` and the `Result`:

```rust
// before
use ferromark::Renderer;
let html = renderer.render(&document)?;

// after
let html = renderer.render(&document);
```

Use `HtmlRenderHooks` for custom output, and declare your own trait if you need
to abstract over renderers. `ast::Position` is removed as well; the AST carries
byte offsets in `Span`.

**Removed error variants.** `ParseErrorKind::UnexpectedToken`, `InvalidSyntax`
and `UnexpectedEof` were never produced by the parser and are gone;
`NestingTooDeep` remains. `ParseErrorKind` is now `#[non_exhaustive]`, so a
`match` over it needs a wildcard arm:

```rust
match error.kind() {
    ParseErrorKind::NestingTooDeep { max_depth, .. } => { /* … */ }
    _ => { /* … */ }
}
```

**Removed option fields.** `ParserOptions.gfm` and
`HtmlRendererOptions.highlight` did nothing and are removed. Delete them from
struct literals; `ParserOptions::gfm()` still enables the GFM extensions, and
`ParserOptions.highlight` — the `==mark==` syntax switch, unrelated to the
removed renderer field — is unchanged. Syntax highlighting goes through
`HtmlRenderHooks` in Rust and `toHtmlWithHighlighter` in Node.

**`soft_break` now takes effect.** The field was stored and never read, so any
value other than the default `"\n"` was silently ignored. It is now emitted for
the line ending that joins two lines of the same block. If you set it to
something else during `rc` and relied on the old no-op behavior, remove the
assignment. Like `hard_break`, the value is written verbatim, so `xhtml` does
not rewrite it: pass `"<br />"` yourself when the output has to be XHTML.

**The renderer's `gfm()` changed meaning.** `HtmlRendererOptions::gfm()` was the
strict profile; it is now the convenience profile, matching
`ParserOptions::gfm()`. The strict profile is `HtmlRendererOptions::gfm_spec()`,
matching `ParserOptions::gfm_spec()`. Parser names are unchanged.

```rust
// before: strict GFM output
HtmlRendererOptions::gfm()

// after: strict GFM output
HtmlRendererOptions::gfm_spec()
```

`HtmlRendererOptions::gfm()` now keeps heading IDs, callouts, TOC substitution,
URL autolinking, link targets and fence metadata cleanup, and adds the GFM tag
filter to them. Node's rendered output is unchanged: the binding moved to
`gfm_spec()`.

**Licensing.** The crate and the npm packages are MIT. `LICENSE-APACHE` is no
longer published; `LICENSE`, `LICENSE-MIT` and `UPSTREAM.md` continue to ship.

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
JavaScript. Release candidates use npm's `next` channel and explicit Cargo prerelease versions.
Build and pack locally using [the package checks](releasing.md#local-package-checks).
