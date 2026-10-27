# Code-block renderer plugins

Code-block plugins replace a parsed code block during rendering. They do not
change the AST or run in the optional typography/emoji transform pipeline.
A diagram engine can supply a plugin in its own package, so applications that
only need Markdown do not install or compile that engine.

## Callback contract

A whole-block renderer receives the original parsed code, language/info token,
and remaining metadata. Parser normalization and info-string entity decoding
have already happened; line annotation processing has not. Fenced and indented
code blocks use the same callback. Indented blocks have no language or metadata.

Return a string to handle the block, including an empty string to omit it.
Return `None` in Rust, or `null`/`undefined` in Node, to continue. Whole-block
replacements run before code highlighting. If no plugin handles a block, the
existing escaped code output, annotations, and wrappers apply.

Replacements own their complete markup and whitespace. HTML callbacks return
HTML; JSX callbacks return JSX source. An SVG string containing HTML attribute
names is not automatically valid JSX. Plugins must escape values from the
source and handle any required SVG-to-JSX conversion. Explicitly supplied
callbacks are trusted rendering code even when document HTML is untrusted;
installing a callback does not enable raw authored HTML elsewhere.

## Rust

Implement `HtmlRenderHooks::render_code_block(HtmlCodeBlockInput)` for complete
HTML output. Keep using `highlight_code_block(CodeHighlightInput)` for line
fragments, such as Ferriki's output. A pair `(first, second)` composes hook sets;
pairs can be nested to add more plugins. Within each stage the first `Some`
wins. All whole-block overrides precede the highlighting stage.

Use `HtmlRenderer::render_with_code_hooks` or
`render_borrowed_with_code_hooks` for code plugins. These entry points ignore
`render_node` and use the normal inline renderer. Use the existing
`render_with_hooks` entry point when per-node overrides are needed as well.
The default `render` path continues to use no hooks.

The optional [Ferrigura adapter](ferrigura.md) owns its renderer in a separate
crate, reuses engine state, exposes diagram errors, and assigns distinct SVG
IDs. The core Ferromark crate does not depend on the diagram engine.

The existing Ferriki adapter works with the code-only entry point:

```rust
use ferromark::ast::Document;
use ferromark::ferriki::Highlighter;
use ferromark::{FerrikiHighlightHooks, HtmlRenderHooks, HtmlRenderer};

fn render_with_plugins<H: HtmlRenderHooks>(
    document: &Document<'_>,
    diagram_plugin: H,
    highlighter: &mut Highlighter,
    theme: &str,
) -> String {
    let mut hooks = (diagram_plugin, FerrikiHighlightHooks::new(highlighter, theme));
    HtmlRenderer::new().render_with_code_hooks(document, &mut hooks)
}
```

For JSX, implement `JsxRenderHooks::render_code_block`. The same pair composition
works with `FerrikiJsxHooks`. Whole-block callbacks retain the code-block metadata
in `JsxOutput`; standalone `render_code_block_with_hooks` continues to invoke
only the highlighting stage.

## Node.js

The callback-composition APIs below are available on repository `main`; they
are not exported by the published `ferromark@3.6.0` facade. Use a source build
or custom bundle. See [release availability](https://ferromark.dev/guide/code-rendering).

For Rust plugins without per-block JavaScript calls, use
[`CodeCompiler`](native-code-plugins.md). The callbacks below remain available
for application code and separately installed addons. The standard addon includes
no engines. Select an engine-enabled custom bundle when code rendering should stay
entirely in Rust.

`toHtmlWithCodeRenderer` returns HTML. `transformWithCodeRenderer` returns HTML
plus the same headings and front matter as `transform`. `composeCodeRenderers`
tries callbacks in order and rejects non-string, non-nullish results, including
Promises. Exceptions propagate. Diagram plugins that want plain-code fallback
can catch their engine's errors and return `null`.

`createHighlighterCodeRenderer` adapts the existing `CodeHighlighter` interface
to this contract. It retains `fallbackLanguage` and `onHighlightError`; the
existing `toHtmlWithHighlighter` and `transformWithHighlighter` APIs still work.

```js
import {
  composeCodeRenderers,
  createHighlighterCodeRenderer,
  transformWithCodeRenderer,
} from "ferromark";

// Application-owned synchronous diagram and highlighting engines.
const renderCode = composeCodeRenderers(
  (code, language) => language === "mermaid" ? diagramEngine.render(code) : null,
  createHighlighterCodeRenderer(highlighter, { theme: "github-dark" }),
);
const result = transformWithCodeRenderer(markdown, renderCode);
```

The diagram callback above expects a bare `mermaid` language token. A plugin
that also accepts language metadata suffixes must recognize those explicitly.
It should match parsed callback fields rather than search Markdown source text:
for example, a fence written as `merm&#97;id` reaches the callback as `mermaid`.

For JSX, use an ordered callback with `compileJsx`, `CodeCompiler.compile`, or a
prepared document's render methods. Return JSX from each plugin. Returning
`null` lets the compiler continue to component mappings and explicitly configured
native plugins. Do not pass `createHighlighterCodeRenderer` to JSX compilation:
its output is HTML, while a Ferriki-enabled `CodeCompiler` supplies JSX-safe
highlighting.

A separately installed diagram package can export these callbacks without
adding its native binary to the default Ferromark package. Lazy initialization
of an engine compiled into a shared addon does not remove its download cost.
