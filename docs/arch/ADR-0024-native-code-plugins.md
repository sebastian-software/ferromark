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

## Current implementation

`CodeCompiler` defaults to an empty plugin list. The standard binding's default
Cargo features are empty: it excludes Ferriki, Ferrigura, and their asset and
rendering dependencies. The root Rust `ferriki` feature remains opt-in.

Decision update (2026-10-27): there are no existing consumers requiring a
Ferriki compatibility distribution. Remove the Ferriki-specific `JsxCompiler`
bridge and use `CodeCompiler` for both engines and plain rendering. Do not retain
a default-on engine or a separate migration distribution for compatibility.

The unpublished fixture crate proves HTML/JSX dispatch from an optional crate.
The Ferrigura and Ferriki factories use the same optional boundary; both are
optional features of the root crate ([ADR-0026](ADR-0026-ferrigura-crate-feature.md)).
The [plugin bundle workflow](../plugin-bundles.md) produces installable custom
builds with selected factories and notices. Both engine features require isolated
output so they cannot enter standard package artifacts. Separately installed
addons remain usable through JavaScript callbacks, without a native runtime loader.

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
state, and absence of engines in the standard addon. Keep Ferriki behavioral
coverage in an isolated custom build and check Mermaid-only and combined builds.

- [Native plugin guide](../native-code-plugins.md)
- [Node-API ABI scope](https://nodejs.org/api/n-api.html#implications-of-abi-stability)
- [Rust ABI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi)
