# Ferrigura Mermaid plugin

The private `plugins/ferrigura` adapter renders parsed `mermaid` code blocks as
inline SVG. It owns a reusable Ferrigura renderer and implements the same
HTML/JSX hook contracts used by Ferriki. Recognition, rendering, ID allocation,
and SVG-to-JSX conversion run in Rust. No browser Mermaid runtime or per-block
JavaScript callback is required.

The core crate and standard npm addon do not depend on this adapter. The Node
factory is available only in custom builds with the `ferrigura` Cargo feature.
The adapter crate remains private. The [plugin bundle workflow](plugin-bundles.md)
produces installable custom npm tarballs; these are workflow artifacts, not
registry releases.

## Node.js

For a custom addon containing the factory, the facade accepts:

```js
import { CodeCompiler } from "ferromark";

const compiler = new CodeCompiler({
  plugins: [{
    name: "ferrigura",
    options: { idPrefix: "page" },
  }],
});
const markdown = "```mermaid\nflowchart LR\nA[Hello] --> B[World]\n```";
const html = compiler.toHtml(markdown);
const jsx = compiler.compile(markdown);
console.log(compiler.takeDiagnostics()); // [] for a valid diagram.
```

The standard package reports this factory as unavailable. To test an isolated
Mermaid-only addon, run from `node/ferromark`:

```sh
FERROMARK_PGO=0 \
FERROMARK_NAPI_NO_DEFAULT_FEATURES=1 \
FERROMARK_NAPI_FEATURES=ferrigura \
FERROMARK_NATIVE_OUTPUT_DIR=/tmp/ferromark-mermaid \
node scripts/build-native.mjs
```

Load the resulting platform `.node` file with Node's `createRequire`; it does
not replace the facade's platform binary. Its constructor takes a JSON string:

```js
const compiler = new addon.CodeCompiler(JSON.stringify({
  plugins: [{ name: "ferrigura" }],
}));
const html = compiler.toHtml("```mermaid\npie\n\"A\" : 1\n```");
```

Set `FERROMARK_NAPI_FEATURES=ferriki,ferrigura` to include both factories. Use both entries in `plugins` to render diagrams and highlight
ordinary code. Whole-block replacements precede all highlighting plugins.
Custom Ferrigura builds require an isolated output directory so they cannot
overwrite standard package artifacts.

Options are validated once:

| Option | Behavior |
| --- | --- |
| `config` | Object passed to Ferrigura's renderer configuration; defaults to an empty object. Engine-specific validation belongs to Ferrigura. |
| `colorScheme` | `auto` by default: embedded CSS follows `prefers-color-scheme` without JavaScript. Explicit `light` and `dark` select one theme. |
| `idPrefix` | Optional output namespace. Starts with an ASCII letter and contains only ASCII letters, digits, `-`, or `_`. |
| `errorPolicy` | `code` by default: failed or unsafe diagrams retain escaped code and a diagnostic. `throw` also records a diagnostic, then rejects the render. |

The default namespace is `fm-mermaid`. IDs append the original fence's UTF-8
start offset. Repeated HTML/JSX renders, fresh compiler instances, and prepared
handles therefore produce the same IDs and SVG bytes. Different fences in one
document have distinct IDs, including identical diagrams. Supply a different
`idPrefix` for each document when combining their output in one page. The
engine's implicit clock is fixed at the Unix epoch, including Gantt's implicit
dates and today marker; use explicit dates in time-dependent diagrams.

Drain diagnostics after rendering:

```js
const source = "é\n\n```mermaid\nfutureDiagram\nA --> B\n```";
const html = compiler.toHtml(source); // The fence remains escaped code.
for (const diagnostic of compiler.takeDiagnostics()) {
  console.log(diagnostic.plugin, diagnostic.start, diagnostic.end, diagnostic.message);
  // Offsets count original UTF-8 bytes, not JavaScript string indexes.
  console.log(Buffer.from(source).subarray(diagnostic.start, diagnostic.end).toString());
}
```

Diagnostics accumulate until drained. `prepared.takeDiagnostics()` drains the
same queue as its compiler, including after that compiler has been collected.
A failed block does not suppress later diagrams under the default fallback
policy. Analysis methods do not render diagrams or create diagnostics.

The exact parsed language token must be `mermaid`. Nested fences and decoded
info strings work; unrelated languages and metadata suffix variants decline.
Authored document HTML keeps its existing trust policy. Mermaid output passes
the SVG policy below before HTML emission or JSX conversion. JSX conversion
preserves SVG element names, maps attributes and
inline styles, and escapes text/CSS as string expressions without
`dangerouslySetInnerHTML`. Standalone `renderCodeBlock` remains a highlighting
API; use document HTML/JSX methods for diagrams.

## Rust

In this workspace, depend on the private adapter with a path dependency. Enable
its `jsx` feature only when needed:

```rust
use ferromark::{Allocator, HtmlRenderer, Parser};
use ferromark_ferrigura::{FerriguraHooks, ferrigura::{Renderer, Value}};

let allocator = Allocator::new();
let source = "```mermaid\npie\n\"A\" : 1\n```";
let document = Parser::new(&allocator, source).parse()?;
let mut hooks = FerriguraHooks::new(Renderer::new(&Value::empty_object()), "page")?;
let html = HtmlRenderer::new().render_with_code_hooks(&document, &mut hooks);
if let Some(error) = hooks.take_error() {
    return Err(error.into());
}
```

`hooks.take_diagnostics()` returns failures with `span` and `message`; render
output already contains escaped code at those fences. `take_error()` optionally
returns the first original error for strict application policies. A failed
fence does not prevent later diagrams from rendering. A tuple composes these
hooks with Ferriki hooks; the adapter has no Node dependency.

The runnable native example reads Markdown from stdin and reports ranges on
stderr while writing HTML to stdout:

```sh
cargo run -p ferromark-ferrigura --example mermaid --locked < input.md > output.html
```

## SVG policy and deterministic fixtures

Every diagram is parsed as XML and checked against a static SVG element and
attribute allow-list. CSS is tokenized, including escaped names and nested
functions. The policy rejects scripts, event attributes, `foreignObject`, image
and animation elements, foreign namespaces, DTDs, processing instructions,
external/relative links, CSS imports, and resource-loading CSS functions.
Only local fragment references and embedded base64 font data are allowed.
Text and attributes are serialized again, so CDATA and markup-looking labels
cannot turn into HTML. Rejected output follows the same diagnostic/fallback
path as a parse failure. Application callbacks remain trusted application code;
this policy belongs to the Ferrigura adapter. See the
[SVG decision](arch/ADR-0025-static-mermaid-output.md) for rationale and dependency
redistribution details.

Shared fixtures cover flowchart, sequence, state, class, and ER diagrams. Their
complete SVG SHA-256 values are pinned for Rust and installed Node consumers.
CI checks the same expected bytes on its native platforms and installed musl
bundle, as well as repeated renders, multiple diagrams, invalid and unknown
types, and UTF-8 diagnostic ranges. Explicit light/dark options and automatic
CSS themes need no client runtime. These checks implement the remaining
[issue #531](https://github.com/sebastian-software/ferromark/issues/531) contract;
they do not promise pixel parity with mermaid.js.

Image-based diagrams, external links, and other engine constructs outside this
SVG policy fall back to code with a diagnostic. The adapter is private, the
standard addon excludes Ferrigura, and custom bundles provide installation.
`@ferrigura/core` is not yet publicly available on npm.

### Pinned engine

The adapter pins published Ferrigura **0.2.0**. That release still uses a registry
Ferromark internally for Markdown labels. The optional adapter sits outside the
core, so the dependency graph remains acyclic. Ferrigura's separate
`label-markdown` branch removes that internal dependency; adopt its replacement
through a reviewed engine release. No unpublished engine branch is required.

Ferrigura embeds font assets. Custom binary distributions must retain its font
licenses and attribution, including `ferrigura-text`'s MIT, OFL-1.1, and CC-BY-4.0
notices. The standard addon does not include those assets.

Run `pnpm test:ferrigura` from `node` to build and exercise Mermaid-only and
Ferriki-plus-Ferrigura addons. Rust integration tests use real diagrams and
check HTML, JSX modules, nested blocks, IDs, diagnostics, errors, and fallback. These tests
do not measure rendering latency or memory. See the
[native plugin guide](native-code-plugins.md) for the composition boundary and
remaining distribution work.

Retained checks, dependency graphs, and local addon size measurements are in the
[integration report](reports/2026-10-25-ferrigura/README.md).
