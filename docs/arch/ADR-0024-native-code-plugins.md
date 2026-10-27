# ADR-0024: Compose code plugins in Rust

- Status: Accepted
- Date: 2026-10-25
- Partially supersedes the long-term Ferriki adapter placement in
  [ADR-0018](ADR-0018-single-rust-crate.md) and the Ferriki-specific Node compiler
  direction in [ADR-0023](ADR-0023-framework-neutral-jsx.md).

## Context

Ferriki and Ferrigura both consume parsed code blocks during rendering. Their
engines, assets, and initialization should be optional application choices.
Adding each engine to the default native addon increases the download and
dependency graph even when no block uses it. A JavaScript callback can connect
separate addons today, but serializes each block through JavaScript and puts
adapter logic outside the Rust renderer.

Node-API stabilizes the Node/addon boundary. It does not stabilize Rust trait
objects, allocation ownership, or panics between independently compiled addons.
Passing a boxed hook through an external Node value would not establish a
supported plugin ABI.

## Decision

Use the same engine-neutral Rust render hooks for highlighting and diagrams.
Whole-block replacements precede highlighting, with first-handled ordering
inside each stage. Plugins match parsed language and metadata rather than
searching the original Markdown text.

Compose native plugins into one addon at build time. Optional crate dependencies
and explicit factory registration determine its available plugins. At runtime,
`CodeCompiler` selects and initializes that set once from JSON configuration.
HTML and JSX rendering invoke Rust hooks directly. JavaScript owns the small
facade, validation of existing public options, and optional callback overrides.
Plugins retain responsibility for producing the correct output format.

The host owns plugin instances, shares them with prepared documents, rejects
recursive mutable use, and propagates engine failures. No engine instances,
Rust pointers, vtables, or allocations cross addon boundaries. Plugins are
trusted application code. There is no runtime library loader or global registry.

## Migration and current implementation

`CodeCompiler` defaults to an empty plugin list. The binding builds with
`--no-default-features` without Ferriki or its remote asset stack. A separate,
unpublished fixture crate proves native HTML/JSX dispatch from an optional crate;
it is available only in isolated diagnostic builds.

The current 3.x package still enables Ferriki by default for `JsxCompiler`
compatibility. That class delegates to the same host with a Ferriki plugin.
An empty `CodeCompiler` configuration avoids engine initialization, but does not
shrink that compatibility binary. The root Rust `ferriki` feature also remains
supported during the migration.

Complete external adapter packaging and publish a supported engine-free addon
distribution before removing the compatibility feature in a breaking release.
Ferrigura must use the same optional factory/adapter boundary. The private Ferrigura adapter and factory are implemented; independently
installable native adapter packages remain distribution work.

Implementation update (2026-10-25): #567 also added the optional Ferrigura
factory and private Rust adapter. The [plugin bundle workflow](../plugin-bundles.md)
now produces installable custom builds using this same static composition
boundary. It does not change the standard 3.x package or add a runtime loader.

## Alternatives and consequences

- Separate npm addons connected by callbacks remain useful when installation
  flexibility matters more than keeping all rendering in Rust.
- Runtime native loading would require a versioned C ABI, allocation/release
  functions, panic containment, addon lifetime and worker ownership, and tests
  across independently built binaries. Defer that cost until installing
  arbitrary plugins without rebuilding is a concrete need.
- Static composition requires a new addon build for each supported plugin set
  and matching platform binaries. Installing an npm package cannot add a
  factory to an already compiled addon.
- Additional native dispatch has not been benchmarked. Existing code-hook
  measurements do not establish the cost of this host or an engine.

## Validation and references

Check the engine-free normal/build dependency graph, compile both feature sets,
and load an isolated addon containing the external fixture. Verify fallback,
escaping, parsed language matching, first-handled ordering, retained prepared
state, and absence of the compatibility class in engine-free builds. Preserve
the existing Ferriki tests and compare old and new compiler output.

- [Native plugin guide](../native-code-plugins.md)
- [Node-API ABI scope](https://nodejs.org/api/n-api.html#implications-of-abi-stability)
- [Rust ABI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi)
