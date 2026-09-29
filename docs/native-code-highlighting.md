# Native code highlighting with Ferriki

Enable Ferromark's `ferriki` Cargo feature to connect a reusable, N-API-free
Ferriki highlighter to the Rust HTML renderer. The default Ferromark build has
no Ferriki dependency, and ordinary rendering never loads assets or invokes
the highlighter. This integration implements [issue #393].

[issue #393]: https://github.com/sebastian-software/ferromark/issues/393
[`ferriki` crate]: https://crates.io/crates/ferriki
[Ferriki asset documentation]: https://github.com/sebastian-software/ferriki/blob/main/docs/rust-api.md#assets-and-lifecycle

## Use the adapter

Build a Ferriki `Highlighter` once, then borrow it for each render. Ferromark
does not choose an asset source. The following compiled
[`ferriki` example](../examples/ferriki.rs) loads Ferriki's current filesystem
catalog; see the [Ferriki asset documentation] for other sources and the CDN
work in progress.

```toml
ferromark = { version = "3", features = ["ferriki"] }
```

```rust
use std::path::Path;

use ferromark::ferriki::{Highlighter, StandardAssetCatalogs};
use ferromark::{Allocator, FerrikiHighlightHooks, HtmlRenderer, Parser};

fn render(asset_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let assets = StandardAssetCatalogs::load_from_root(asset_root)?;
    let mut highlighter = Highlighter::builder()
        .with_assets(assets)
        .load_languages(["rust"])
        .load_themes(["nord"])
        .build()?;
    let source = "```rust\nfn main() {}\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse()?;
    let mut renderer = HtmlRenderer::new();
    let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "nord");
    Ok(renderer.render_with_hooks(&document, &mut hooks))
}
```

`FerrikiHighlightHooks` lowercases the normalized language for Ferriki, keeps
Ferromark's wrappers and annotations, and returns escaped plain code for missing
or unknown languages and failed highlighting. Add `with_error_handler` to record
errors other than unknown languages. The handler is called synchronously.
Ferriki can lazily load local assets on first use; preload the languages and
themes you need if rendering must avoid asset I/O. Ferriki owns the asset source,
including the planned optional CDN source; Ferromark does not fetch assets.

## Contract

For a different highlighter, implement `HtmlRenderHooks::highlight_code_block`
and render with `HtmlRenderer::render_with_hooks`. The hook receives:

- `code`: the displayed code after Ferromark removes enabled VitePress inline
  annotation directives;
- `language`: the normalized fence language used for the HTML class;
- `raw_language` and `raw_meta`: the original fence fields, so metadata is not
  silently conflated with the language.

The hook returns `HighlightedCodeBlock::new(lines)`, with one independently
tag-balanced HTML fragment for each `code.split('\n')` line, including a final
empty line when the code ends in a newline. `with_colors(foreground,
background)` carries theme colors onto the outer `<pre>` style attribute.
Ferromark keeps titles, line numbers, links, annotations, source spans and
classes. The hook may return `None` for an unknown language or recoverable
highlighter error. Ferromark then escapes and renders plain code. A result with
the wrong line count or a carriage return or newline inside one fragment also
falls back to plain code.

Fragments are **trusted HTML**. Theme colors are also trusted adapter output:
Ferromark escapes them for the HTML `style` attribute, but does not validate
their CSS syntax or prevent additional declarations in a supplied value. Never
pass colors derived from untrusted Markdown or metadata without validation.
Any custom adapter must escape every source token and attribute it emits, and
must not return `<pre>`, `<code>`, or tags spanning line boundaries. Ferromark
escapes its own metadata and attributes. The built-in adapter maps Ferriki
failures to plain-code fallback; applications can record non-language errors
before returning `None`.

The older `render_node` hook still runs first. Its `Handled` result retains the
whole-block replacement contract used by the Node callback renderer. The new
method is called only when `render_node` returns `Default`; ordinary
`HtmlRenderer::render` never dispatches hooks or loads highlighting assets.

The built-in adapter calls `highlight_html_lines` and passes the escaped line
fragments and theme colors to `HighlightedCodeBlock`. It borrows the highlighter
for one render and can be reconstructed for subsequent renders.

The self-contained [lifecycle benchmark](../benches/ferriki.rs) measures
highlighter construction, construction plus first render, and repeated render
with custom in-memory assets. Its [initial report](reports/2026-09-29-ferriki-integration/README.md)
does not cover standard catalog or CDN I/O.
