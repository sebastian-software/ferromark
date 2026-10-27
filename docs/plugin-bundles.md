# Installable native plugin bundles

The **Native plugin bundles** workflow builds a platform-specific npm tarball
with one selected plugin set: `none`, `ferriki`, `ferrigura`, or `all`. It uses
the same Rust factories as `CodeCompiler`. Consumers install the complete
custom Ferromark build; no Rust compiler, install script, browser runtime, or
JavaScript adapter per code block is needed.

This is static composition. A bundle replaces the application's `ferromark`
installation; it does not add factories to another already loaded addon.
Independent native addon loading and a cross-addon Rust ABI are not supported.
The Rust adapter crate and native build workspace remain private.

## Download and install

Run `.github/workflows/plugin-bundles.yml` from a reviewed ref and choose the
plugin set. Download the artifact for the consuming application's target,
extract it, and install its `.tgz`:

```sh
npm install /path/to/ferromark-3.6.0-custom.ferrigura.PLATFORM.COMMIT.tgz
```

Use the actual filename from the artifact. The workflow builds the existing
seven native targets. Matching runner hosts install the package and test HTML,
JSX, plugin availability, and prepared documents; an Alpine job tests x64 musl.
ARM64 musl is cross-built and packaged but has no native runtime test in this
workflow. CI checks the combined bundle on Linux for pull requests.

```js
import { availableCodePlugins, CodeCompiler } from "ferromark";

console.log(availableCodePlugins());
const compiler = new CodeCompiler({
  plugins: [{ name: "ferrigura", options: { colorScheme: "auto" } }],
});
const html = compiler.toHtml(markdown);
const jsx = compiler.compile(markdown);
```

All variants use `CodeCompiler`. The standard package includes no engine
factories; a custom bundle includes only the selected engines. Selecting no
plugins at runtime does not initialize the engines compiled into a custom bundle.

Each artifact includes:

- An installable tarball with the facade, declarations, one native binary, and
  third-party license/attribution texts, including Ferrigura's font notices.
- `bundle.json` with the source commit, dirty-state marker, selected plugins,
  platform, lockfile hash, binary/archive hashes, and compressed/unpacked sizes.
- `installation-test.json` where a matching host installed and exercised it.

The tarballs carry a distinct `custom` version and `private: true`. They can be
installed from files but cannot accidentally be published to npm. This workflow
uploads build artifacts only; reviewed registry releases still use the verified
release workflow. Standard package artifacts and dependencies are untouched.
PGO is disabled for these custom builds; platform memory and speed can differ
from the standard published PGO build.

## Build and verify locally

Install the repository's pinned Node dependencies first. Run from `node`, with
a new absolute output directory:

```sh
pnpm bundle:plugins --plugins all --output /tmp/ferromark-plugins
pnpm verify:plugin-bundle /tmp/ferromark-plugins/bundle.json
```

Use `none`, `ferriki`, or `ferrigura` for the other variants. Package creation
compiles Rust into an isolated temporary directory. Verification installs the
tarball into a clean project offline and runs without install scripts. Explicit
`FERROMARK_RUST_TARGET` selects a target supported by the existing native build
script; cross-compiled binaries must be verified on a matching host.

## Measure rendering and memory

After verifying a bundle containing Ferrigura, run from the repository root:

```sh
node node/ferromark/bench/plugins.mjs /tmp/ferromark-plugins/bundle.json /tmp/plugin-measurements
```

The characterization measures HTML and JSX for ordinary Markdown, a small
flowchart, a larger flowchart, and a mixed document. It checks ordinary output
equality between the regular API, an empty compiler, and an initialized but
unused Ferrigura plugin. Diagram cases perform different work and are not a
speed comparison with plain code output.

Results retain raw batches from three fresh processes per case, cold addon
load, initialization, first render, warm medians and ranges, and process RSS.
RSS includes Node/V8, output allocation, native code, and engine caches; it is
not Rust heap attribution. Node's high-water RSS is converted from KiB to bytes
per its [resource usage contract](https://nodejs.org/docs/latest-v24.x/api/process.html#processresourceusage).
Warm timings exclude setup and explicit GC. Run without competing builds and
retain environment and source/binary hashes with any reported result.

The engine-free bundle runs only ordinary-document lanes and provides a
baseline. Retained local results are in the
[delivery and performance report](reports/2026-10-25-plugin-delivery/README.md).
