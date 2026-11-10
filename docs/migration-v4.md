# Upgrading from Ferromark v3 to v4

Version 4 makes native code engines opt-in and adds source locations to code
hooks. Plain Markdown rendering and parser option defaults stay the same.
Applications using code highlighting, diagrams, custom hook implementations, or
`JsxCompiler` should follow the steps below. For earlier upgrades, apply the
[v1-to-v2](migration-v2.md) and [v2-to-v3](migration-v3.md) guides first.

## Node.js: initialize engines in your application

The standard v4 addon contains neither Ferriki nor Ferrigura. It does not fetch
grammars or themes. Installing an engine package does not register a native
plugin. The old `JsxCompiler` and its `theme`, `languages`, and `assets` constructor
options are removed; use `CodeCompiler` with explicit hooks, or `compileJsx`.

Before (v3):

````js
import { JsxCompiler } from "ferromark";

const compiler = new JsxCompiler({ theme: "github-light-default" });
const result = compiler.compile("```rust\nfn main() {}\n```");
````

After (v4), install `ferromark`, `@ferriki/core`, and `@ferrigura/core`, then
initialize the engines before rendering:

````js
import { Renderer } from "@ferrigura/core";
import { createHighlighter } from "@ferriki/core";
import {
  CodeCompiler,
  composeCodeHooks,
  createDiagramCodeRenderer,
  createLineHighlighter,
} from "ferromark";

const themes = { light: "github-light-default", dark: "github-dark-default" };
const highlighter = await createHighlighter({ themes: Object.values(themes), langs: ["rust"] });
const hooks = composeCodeHooks(createDiagramCodeRenderer(new Renderer(), { colorScheme: "auto" }), {
  highlightCodeBlock: createLineHighlighter(highlighter, { themes }),
});
const compiler = new CodeCompiler();
const source = "```rust\nfn main() {}\n```\n\n```mermaid\nflowchart LR\nA --> B\n```";
const result = compiler.compile(source, {}, hooks);
// result.body contains highlighted JSX and the diagram SVG converted to JSX.
````

Load every required language and theme before rendering if you need to avoid
lazy engine asset I/O. Whole-block rendering runs before line highlighting;
missing languages and failed engine calls fall back to escaped code. Hook output
is trusted application markup. See the [code rendering guide](https://ferromark.dev/guide/code-rendering)
for error observers and engine setup.

For rendering without an engine, `compileJsx(source)` and `new CodeCompiler()`
remain sufficient. For engines compiled into Rust, build an isolated
[custom native bundle](plugin-bundles.md), enable its `ferriki` and/or `ferrigura`
features, and configure `CodeCompiler({ plugins: [...] })`. The standard addon
reports those factories as unavailable; passing a plugin name does not install it.

## Node.js: accept the fourth callback argument

Whole-block renderers and line highlighters now receive
`(code, language, meta, block)`. `block` contains the complete block's `start` and
`end` offsets in UTF-8 bytes, including the fence. Existing callbacks that ignore
extra arguments continue to work. Update wrappers that forward callbacks to
pass the location through too.

Before:

```js
const wrap = (render) => (code, language, meta) => render(code, language, meta);
```

After:

```js
const wrap = (render) => (code, language, meta, block) => render(code, language, meta, block);
```

The locations are byte offsets, not JavaScript string indices. Use
`Buffer.from(source).subarray(block.start, block.end)` when slicing source by a
callback's location. The public callback type allows `block` to be omitted for
manual calls to an adapter.

## Node.js: unwrap diagram results when calling adapters directly

`createDiagramCodeRenderer` now returns `{ html }` for a handled diagram, rather
than a string. Ferromark uses that explicit HTML result to convert SVG into JSX.
Applications passing the adapter directly as a hook need no extra unwrapping;
applications calling it themselves must handle the result object.

Before:

```js
const svg = diagrams("flowchart LR\nA --> B\n", "mermaid");
```

After (complete example with an application-owned renderer):

```js
import { createDiagramCodeRenderer } from "ferromark";

const renderer = { render: () => '<svg xmlns="http://www.w3.org/2000/svg"></svg>' };
const diagrams = createDiagramCodeRenderer(renderer);
const result = diagrams("flowchart LR\nA --> B\n", "mermaid");
const svg = result && typeof result === "object" ? result.html : undefined;
// svg is the renderer's SVG string. Unhandled languages return null.
```

Custom whole-block callbacks may still return a string, `{ html }`, `null`, or
`undefined`. A string means HTML for HTML entry points and JSX source for JSX
entry points. `{ html }` explicitly requests HTML-to-JSX conversion. Empty strings
remove a block. Callbacks are synchronous; Promise results are rejected.

## Rust: opt into engines and include the highlight span

The root crate keeps engines behind optional features:

```toml
[dependencies]
ferromark = { version = "4", features = ["ferriki", "ferrigura"] }
```

Select only the features you use, initialize engines, and connect
`FerrikiHighlightHooks` or the [Ferrigura hooks](ferrigura.md) explicitly. Enable
`ferriki-remote` only if you want Ferriki's remote asset loading. Default features
are empty; plain parsing and rendering need no engine feature.

`CodeHighlightInput` adds a required `span: Span` field. Destructuring code can
ignore the new field with `..`; struct literals must supply the original complete
block range:

Before:

```rust
let CodeHighlightInput { code, language, raw_language, raw_meta } = input;
```

After:

```rust
use ferromark::CodeHighlightInput;

fn inspect(input: CodeHighlightInput<'_>) {
    let CodeHighlightInput { code, span, .. } = input;
    println!("Code at bytes {}..{}: {}", span.start, span.end, code);
}
```

The span uses the same UTF-8 source coordinates as the AST. Rust whole-block
hooks continue to receive `HtmlCodeBlockInput`; Node's fourth argument mirrors
that location. See the [hook contract](code-block-renderers.md) and
[4.0 changelog](../CHANGELOG.md#400-2026-11-05) for the complete release context.
