# Native code highlighting with Ferriki

Enable Ferromark's `ferriki` Cargo feature to connect a reusable, N-API-free
Ferriki highlighter to the Rust HTML renderer. The default Ferromark build has
no Ferriki dependency, and ordinary rendering never loads assets or invokes
the highlighter. This integration implements [issue #393].

[issue #393]: https://github.com/sebastian-software/ferromark/issues/393
[`ferriki` crate]: https://crates.io/crates/ferriki
[Ferriki asset documentation]: https://github.com/sebastian-software/ferriki/blob/v0.10.0/docs/rust-api.md#assets-and-lifecycle

## JSX output

`FerrikiJsxHooks` provides the same native integration for the framework-neutral
JSX renderer. It supports a single theme or a light/dark pair and preserves
fence titles, labels, highlighted lines, and line numbers as JSX props and
children. See the [JSX renderer contract](jsx-renderer.md#native-ferriki-highlighting).

The Node `JsxCompiler` owns that highlighter and loads verified standard assets
itself; reuse it across documents. Rust applications can enable
`ferriki-remote` to make Ferriki's CDN/cache source available through Ferromark's
re-export, or keep `ferriki` for custom/local asset sources. The core's default
dependency graph remains free of Ferriki, remote-loading code, and Oxc.

## Use the adapter

Build a Ferriki `Highlighter` once, then borrow it for each render. Ferromark
does not choose an asset source. The following compiled
[`ferriki` example](../examples/ferriki.rs) loads Ferriki's current filesystem
catalog from a matching Ferriki 0.10.0 release checkout. Published Ferriki crates
contain catalog manifests, not grammar or theme payloads. See the [Ferriki asset
documentation] for directory, embedded, and verified CDN sources.

This feature is not released yet. To try this branch, use a local checkout:

```toml
ferromark = { path = "../ferromark", features = ["ferriki"] }
```

After the feature ships, use a published Ferromark 2.x version that includes it.

````rust
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
````

`FerrikiHighlightHooks` lowercases the normalized language for Ferriki, keeps
Ferromark's wrappers and annotations, and returns escaped plain code for missing
or unknown languages and failed highlighting. Add `with_error_handler` to record
errors other than unknown languages. The handler is called synchronously.
Ferriki can lazily load local assets on first use; preload the languages and
themes you need if rendering must avoid asset I/O. Ferriki owns the asset source,
including its optional CDN source; Ferromark does not select or fetch assets.

## Use release-pinned CDN assets

Ferriki 0.10.0 provides the `remote` feature. Enable it in the application alongside
Ferromark's adapter; the `ferriki` feature alone keeps network code disabled:

```toml
ferromark = { path = "../ferromark", features = ["ferriki"] }
ferriki = { version = "0.10.0", features = ["remote"] }
```

Construct the highlighter before rendering and load the required assets eagerly:

```rust
use ferromark::ferriki::{Highlighter, RemoteAssets, StandardAssetCatalogs};

let mut highlighter = Highlighter::builder()
    .with_assets(StandardAssetCatalogs::remote(RemoteAssets::default())?)
    .load_languages(["rust"])
    .load_themes(["nord"])
    .build()?;
```

Use this highlighter with `FerrikiHighlightHooks` as above. Ferriki verifies payload
sizes, SHA-256 digests, and format versions against its compiled release manifest,
then caches payloads by digest. An unavailable asset is an error during eager
construction; handle that error before rendering. If an application leaves an
asset lazy, loading can still occur synchronously on first use during rendering.
Preload every language and theme needed to keep network/file I/O out of render
timing. Ferriki's environment settings support a mirror or pre-populated cache;
see its [remote asset documentation](https://github.com/sebastian-software/ferriki/blob/v0.10.0/docs/rust-api.md#remote-assets).

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
records Ferriki 0.4.1 and does not cover standard catalog or CDN I/O. Those
historical timings are not measurements of the current Ferriki 0.10.0 adapter.
The [Ferriki 0.7.0 report](reports/2026-10-01-ferriki-compatibility/README.md)
records the historical local lifecycle diagnostic and 0.7.0 shared-consumer evidence.

## Shared Rust/Node fixture lane

The [public-consumer contract](../scripts/ferriki-compatibility/README.md) runs
the same pinned Rust, TypeScript/TSX, Markdown embedding, fallback and metadata
fixtures through a clean packaged Rust consumer and the published Node
highlighter at Ferriki 0.10.0. The Rust consumer retains frozen token, UTF-8
offset, scope and token-type checks. The public Node peer records HTML only;
it does not call removed Node tokenization APIs. Both peers check source text,
public errors, reuse and separately frozen HTML output. Separate raw Markdown
snapshots make the existing wrapper and callback differences visible.
The Rust-only CI lane needs no Node installation; a second lane exercises both
peers. Both cover reuse and asset-error fallback with deterministic local assets.
