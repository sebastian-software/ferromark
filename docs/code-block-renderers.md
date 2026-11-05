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

The optional [Ferrigura integration](ferrigura.md) (`ferrigura` feature) owns
its renderer, reuses engine state, exposes diagram errors, and assigns distinct
SVG IDs. Without the feature, Ferromark does not depend on the diagram engine.

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

Node.js applications inject their engines, the same way for highlighting and
diagrams: the application installs and initializes `@ferriki/core`,
`@ferrigura/core` or another engine, and passes code hooks to Ferromark. The
standard addon contains no engine. The hooks mirror the two Rust stages:

| Hook | Stage | Returns | Ferromark keeps |
| --- | --- | --- | --- |
| `renderCodeBlock(code, language, meta, block)` | whole-block replacement | HTML (HTML entry points), JSX source (JSX entry points), or `{ html }` for either | nothing; the block is replaced |
| `highlightCodeBlock(code, language, meta, block)` | highlighting | `{ lines, foreground?, background?, darkForeground?, darkBackground? }` | wrapper, line annotations, title/label, `codeBlockComponent` |

`block` is the complete fence's original UTF-8 byte range, `{ start, end }`, for
diagnostics that name a file and line. `renderCodeBlock` receives the language
token as authored; `highlightCodeBlock` receives the normalized bare language.
Highlighting runs only when no whole-block hook handled the block. A highlighted
result needs one tag-balanced fragment per code line, or Ferromark uses plain
code. JSX entry points convert `{ html }`, including inline SVG, with the rules for
authored raw HTML: SVG attribute names keep their case (`markerWidth`), and
hyphenated or namespaced names are camel-cased (`strokeWidth`, `xlinkHref`).
Rust exposes the same conversion as `ferromark::html_to_jsx`.

Every entry point that accepts code rendering takes either a whole-block
function or a hook object: `toHtmlWithCodeRenderer`, `transformWithCodeRenderer`,
`compileJsx`, `CodeCompiler.compile`, and a prepared document's `render` and
`renderModule`. `composeCodeHooks` combines functions and hook objects in
priority order; within each stage the first result wins. `composeCodeRenderers`
combines whole-block functions only. Results are checked synchronously, so
Promises and other shapes throw. Exceptions propagate; the adapters below catch
their engine's errors instead and fall back to ordinary code.

```js
import { Renderer } from "@ferrigura/core";
import { createHighlighter } from "@ferriki/core";
import {
  compileJsx,
  composeCodeHooks,
  createDiagramCodeRenderer,
  createLineHighlighter,
} from "ferromark";

const themes = { light: "github-light-default", dark: "github-dark-default" };
const highlighter = await createHighlighter({ themes: Object.values(themes), langs: ["ts"] });
const hooks = composeCodeHooks(
  createDiagramCodeRenderer(new Renderer(), {
    colorScheme: "auto",
    onRenderError: (error, { start }) => report(error, start),
  }),
  {
    highlightCodeBlock: createLineHighlighter(highlighter, {
      themes,
      onHighlightError: (error, { start }) => report(error, start),
    }),
  },
);
const result = compileJsx(markdown, { codeBlockComponent: "CodeBlock" }, hooks);
```

`createLineHighlighter` adapts any Shiki-compatible synchronous highlighter. It
reads tokens and block colors through the `tokens` and `pre` transformer hooks
and writes the same per-line fragments as the native Ferriki integration,
including `--shiki-light`/`--shiki-dark` variables for a theme pair. Load the
document's languages before rendering; a missing language falls back to plain
code and reports through `onHighlightError`.

`createDiagramCodeRenderer` accepts any object with a synchronous
`render(text, options)` method. It handles only its fence language (`mermaid` by
default), returns `{ html }`, and gives each fence the SVG namespace
`<idPrefix>-<start offset>` (`fm-mermaid` by default), as the native integration
does. The exact parsed language must match; a fence written as `merm&#97;id`
reaches the hook as `mermaid`. Hook output is trusted application HTML; the
static SVG policy belongs to the native Ferrigura integration.

`createHighlighterCodeRenderer` adapts the older `CodeHighlighter` interface to a
whole-block HTML renderer with one theme. It retains `fallbackLanguage` and
`onHighlightError`; `toHtmlWithHighlighter` and `transformWithHighlighter` still
use it. Prefer `createLineHighlighter`, which keeps Ferromark's code block markup
and also works for JSX.

Engines compiled into a custom addon remain available through
[`CodeCompiler`](native-code-plugins.md) plugins. Hooks passed to
`CodeCompiler.compile` run before its native plugins within each stage.
