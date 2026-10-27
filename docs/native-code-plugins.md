# Native code plugins in Node.js

`CodeCompiler` initializes plugins once in Rust. HTML and JSX rendering do not
need a JavaScript callback for each block.

These APIs are available on repository `main` and in custom plugin bundles.
The published `ferromark@3.6.0` facade does not export `CodeCompiler`,
`availableCodePlugins`, or the new callback-composition helpers. Build from
source or install a [plugin bundle](plugin-bundles.md) for the examples below.
For the existing published highlighter API and a comparison of the integration
paths, start with the [code-rendering guide](https://ferromark.dev/guide/code-rendering).

```js
import { availableCodePlugins, CodeCompiler } from "ferromark";

console.log(availableCodePlugins()); // Factories linked into this addon.
const compiler = new CodeCompiler({
  plugins: [{
    name: "ferriki",
    options: { theme: "github-dark-default", assets: { remote: false } },
  }],
});

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

Engine errors propagate without trying another plugin unless an adapter offers
an explicit fallback policy. Later independent
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

## Engine-free builds and the 3.x migration

The default npm addon retains Ferriki for `JsxCompiler` compatibility. Selecting
no plugins avoids initialization, but does **not** remove that binary dependency.
Ferrigura is excluded from the default build; its optional Rust adapter and
factory are described in the [Mermaid guide](ferrigura.md). Inspect the engine-free
graph from the repository root:

```sh
cargo tree -p ferromark-node --no-default-features --edges normal,build --locked
cargo check -p ferromark-node --no-default-features --locked
```

For an isolated local build, run this from `node/ferromark` with an absolute
output directory outside the package:

```sh
FERROMARK_NAPI_NO_DEFAULT_FEATURES=1 \
FERROMARK_NATIVE_OUTPUT_DIR=/tmp/ferromark-engine-free \
node scripts/build-native.mjs
```

Load the raw addon with Node's `createRequire`. Its native constructor takes a
JSON string: `new addon.CodeCompiler("{}")`. It has no engine factories and no
legacy `JsxCompiler`. The output does not replace the facade's platform binary.
`FERROMARK_NAPI_FEATURES` selects optional factories for custom builds. PGO must
be disabled. Diagnostic features cannot be copied into package artifacts.

`pnpm test` in `node` includes `verify-native-plugins.mjs`, which checks the
engine-free graph and loads a fixture-only addon. Default package binaries and
declarations stay untouched.

The [plugin bundle workflow](plugin-bundles.md) distributes installable custom
builds with the selected factories and required notices. Moving the compatibility
Ferriki adapter into a separate crate and switching the standard package to an
engine-free build remain breaking-version migration work.
Existing settings migrate to
`new CodeCompiler({ plugins: [{ name: "ferriki", options: settings }] })`, with
`lineNumbers` moved to the outer compiler options.

## Independently installed native plugins

Callbacks already allow separate npm addons to compose. Direct native calls
between independently compiled addons would need a versioned C ABI. Node-API's
guarantee covers Node/addon interaction, not Rust trait objects or engine
allocation ownership. This implementation passes no pointers or vtables between
addons and adds no runtime native loader. See
[ADR-0024](arch/ADR-0024-native-code-plugins.md).

Retained local checks and dependency graphs are in the
[native plugin validation report](reports/2026-10-25-native-code-plugins/README.md).
