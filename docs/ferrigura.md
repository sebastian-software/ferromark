# Ferrigura Mermaid plugin

The private `plugins/ferrigura` adapter renders parsed `mermaid` code blocks as
inline SVG. It owns a reusable Ferrigura renderer and implements the same
HTML/JSX hook contracts used by Ferriki. Recognition, rendering, ID allocation,
and SVG-to-JSX conversion run in Rust. No browser Mermaid runtime or per-block
JavaScript callback is required.

The core crate and standard npm addon do not depend on this adapter. The Node
factory is available only in custom builds with the `ferrigura` Cargo feature.
The adapter and custom binaries are not published packages yet.

## Node.js

For a custom addon containing the factory, the facade accepts:

```js
import { CodeCompiler } from "ferromark";

const compiler = new CodeCompiler({
  plugins: [{
    name: "ferrigura",
    options: { colorScheme: "auto", errorPolicy: "throw" },
  }],
});
const html = compiler.toHtml(markdown);
const jsx = compiler.compile(markdown);
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

Omit `FERROMARK_NAPI_NO_DEFAULT_FEATURES` to include the compatibility Ferriki
factory as well. Use both entries in `plugins` to render diagrams and highlight
ordinary code. Whole-block replacements precede all highlighting plugins.
Custom Ferrigura builds require an isolated output directory so they cannot
overwrite standard package artifacts.

Options are validated once:

| Option | Behavior |
| --- | --- |
| `config` | Object passed to Ferrigura's renderer configuration; defaults to an empty object. Engine-specific validation belongs to Ferrigura. |
| `colorScheme` | Optional `light`, `dark`, or `auto`; otherwise the engine default applies. |
| `idPrefix` | Optional output namespace. Starts with an ASCII letter and contains only ASCII letters, digits, `-`, or `_`. |
| `errorPolicy` | `throw` by default. `code` declines a failed block so later plugins or the normal escaped-code renderer can handle it. |

Default prefixes are distinct within the loaded addon. Increasing suffixes
distinguish blocks and repeated renders, including prepared handles. For output
combined across processes, builds, or compiler instances with explicit prefixes,
supply distinct namespaces yourself. IDs are stateful, so repeated renders need
not produce byte-identical SVG.

The exact parsed language token must be `mermaid`. Nested fences and decoded
info strings work; unrelated languages and metadata suffix variants decline.
Authored document HTML keeps its existing trust policy. Plugin output is trusted
engine output. JSX conversion preserves SVG element names, maps attributes and
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

Check `take_error` before using rendered output. The infallible core hook trait
cannot return an engine error directly; the adapter retains the first error and
declines further diagrams until it is taken. The Node host checks after each
hook invocation and applies the configured error policy. A tuple composes these
hooks with Ferriki hooks; the adapter itself has no Node dependency.

## Engine version and validation

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
check HTML, JSX modules, nested blocks, IDs, errors, and fallback. These tests
do not measure rendering latency or memory. See the
[native plugin guide](native-code-plugins.md) for the composition boundary and
remaining distribution work.

Retained checks, dependency graphs, and local addon size measurements are in the
[integration report](reports/2026-10-25-ferrigura/README.md).
