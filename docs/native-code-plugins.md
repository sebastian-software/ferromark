# Native code plugins in Node.js

`CodeCompiler` initializes plugins once in Rust. HTML and JSX rendering do not
need a JavaScript callback for each block.

These APIs are available on repository `main` and in custom plugin bundles.
The published `ferromark@3.6.0` facade does not export `CodeCompiler`,
`availableCodePlugins`, or the new callback-composition helpers. Build from
source or install a [plugin bundle](plugin-bundles.md) for the examples below.
For the existing published highlighter API and a comparison of the integration
paths, start with the [code-rendering guide](https://ferromark.dev/guide/code-rendering).

The following example requires a Ferriki-enabled bundle and uses an in-memory
theme, so it does not download standard assets.

```js
import { availableCodePlugins, CodeCompiler } from "ferromark";

console.log(availableCodePlugins()); // Factories linked into this addon.
const compiler = new CodeCompiler({
  plugins: [{
    name: "ferriki",
    options: {
      theme: {
        name: "guide-theme",
        settings: [{ settings: { foreground: "#222222", background: "#ffffff" } }],
      },
      assets: { remote: false },
    },
  }],
});

const markdown = "# Demo\n\n```text\nHello world\n```";
const html = compiler.toHtml(markdown);
const result = compiler.transform(markdown); // HTML, headings, front matter.
const jsx = compiler.compile(markdown);
const prepared = compiler.prepare(markdown);
const again = prepared.render();
```

With remote loading disabled, provide the required verified assets locally or
in Ferriki's cache. Selecting a plugin can initialize or load assets immediately;
failures propagate. Unknown languages retain plain-code fallback. The example
selects a single theme for both formats. Ferriki also accepts the existing
light/dark theme pair for JSX; HTML currently uses its light theme.

`new CodeCompiler()` selects no plugins and loads no engine assets. It supports
the existing JSX body/module options, callback overrides, preparation, extraction,
and standalone code metadata. Prepared handles retain plugin instances after the
compiler is collected. Plugins run synchronously and in list order; all
whole-block replacements precede highlighting. An empty replacement handles a
block. Enabling `lineNumbers` also enables VitePress-compatible annotations for
HTML. Standalone `renderCodeBlock` produces intrinsic code and consults only
highlighting plugins, as the existing standalone API does.

Engine errors propagate unless an adapter offers a fallback policy. Ferrigura
defaults to escaped-code fallback and records a fence diagnostic; its explicit
`errorPolicy: "throw"` rejects the render. Drain `compiler.takeDiagnostics()` or
`prepared.takeDiagnostics()` to obtain `{ plugin, message, start, end }` failures.
Both handles share a queue; offsets count original UTF-8 bytes. Later independent
documents can still use the compiler. Recursive rendering on the same compiler
or a prepared handle sharing its plugins is rejected. Compilers belong to their
Node environment and cannot be transferred to workers.

## Add a Rust plugin

The available set is determined at **build time**. Installing an npm package
does not change an existing `.node` binary. For a custom addon build:

1. Implement `HtmlRenderHooks` and `JsxRenderHooks` in a separate adapter crate
   using the [code-block contract](code-block-renderers.md). Own and reuse the
   engine there. Use highlighting fragments when Ferromark should retain code
   wrappers and annotations. No Node dependency is needed in that crate. Compile
   adapters against the host's Ferromark version so the hook types match.
2. Add it as an optional dependency and Cargo feature of `node/native`.
3. Register its constructor in `node/native/src/plugins.rs`. The addon-local
   `NativeCodePlugin` bridge exposes pending engine errors through `take_error`;
   external adapters can use their own error types. Validate options once in
   the factory and list its name in `available_code_plugins`.
4. Build and distribute the complete addon for the intended platforms. Consumers
   pass the factory's name and JSON-serializable options to `CodeCompiler`.

The unpublished `node/plugin-fixture` crate is an executable example. It only
depends on Ferromark's Rust API and implements whole-block HTML/JSX hooks. The
`plugin-test` feature links it into an isolated addon. This is a source
integration point, not a published addon-host SDK.

## Engine-free standard builds

The standard npm addon includes neither Ferriki nor Ferrigura. Its
`availableCodePlugins()` result is `[]`; `new CodeCompiler()` renders ordinary
HTML and JSX without loading engine assets. Explicitly selecting an unavailable
plugin fails with a configuration error. Inspect the default dependency graph
from the repository root:

```sh
cargo tree -p ferromark-node --edges normal,build --locked
cargo check -p ferromark-node --locked
```

Use `pnpm build` from `node` for the standard package. PGO remains available
through `FERROMARK_PGO=1` for release builds. No feature override is needed to
exclude engines.

The [plugin bundle workflow](plugin-bundles.md) produces installable custom
builds with `ferriki`, `ferrigura`, both, or neither, including required notices.
For raw custom builds, `FERROMARK_NAPI_FEATURES` selects factories. Both engine
features require an isolated `FERROMARK_NATIVE_OUTPUT_DIR` outside the package
and PGO disabled, so custom binaries cannot enter standard platform packages.

`pnpm test` in `node` tests the standard facade with no factories, checks the
default dependency graph, loads a fixture-only addon, and runs the Ferriki
highlighting and prepared-document suite against an isolated custom addon.
`pnpm test:ferrigura` checks Mermaid-only and combined builds.

`CodeCompiler` is the shared compiler API. The Ferriki-specific `JsxCompiler`
compatibility class has been removed. To select highlighting in a Ferriki-enabled
bundle, pass `new CodeCompiler({ plugins: [{ name: "ferriki", options: settings }] })`.
`lineNumbers` belongs to the outer compiler options.

## Independently installed native plugins

Callbacks already allow separate npm addons to compose. Direct native calls
between independently compiled addons would need a versioned C ABI. Node-API's
guarantee covers Node/addon interaction, not Rust trait objects or engine
allocation ownership. This implementation passes no pointers or vtables between
addons and adds no runtime native loader. See
[ADR-0024](arch/ADR-0024-native-code-plugins.md).

Retained local checks and dependency graphs are in the
[native plugin validation report](reports/2026-10-25-native-code-plugins/README.md).
