# ADR-0026: Integrate Ferrigura like Ferriki

- Status: Accepted
- Date: 2026-10-31
- Extends [ADR-0024](ADR-0024-native-code-plugins.md) and
  [ADR-0025](ADR-0025-static-mermaid-output.md); follows the Ferriki placement in
  [ADR-0018](ADR-0018-single-rust-crate.md).

## Context

Ferriki and Ferrigura both render parsed code blocks, but they reached
applications differently. Ferriki is the optional `ferriki` feature of the
published `ferromark` crate, and Node.js applications inject an initialized
`@ferriki/core` highlighter. Ferrigura lived in a private workspace crate that
pinned Ferrigura 0.2.0, reached Node.js only through custom native bundles, and
could not be published: Ferrigura 0.2 depended on registry Ferromark, so a
Ferromark dependency on it would have formed a cycle.

Ferrigura 0.3 no longer depends on Ferromark. Its npm facade, `@ferrigura/core`,
exposes a synchronous `Renderer#render(text, options)`.

## Decision

Move the adapter into the root crate as the additive `ferrigura` feature,
mirroring `ferriki`. The default feature set has no Ferrigura, `cssparser`, or
`roxmltree` dependency. The feature exports `FerriguraHooks`, `FerriguraError`,
and `FerriguraDiagnostic`, and re-exports the engine as `ferromark::ferrigura`.
Behavior is unchanged from ADR-0025: deterministic IDs, the frozen clock, the
static SVG policy, diagnostics, and code fallback. The native binding's
`ferrigura` feature enables the root feature; the standard addon still excludes
it.

In Node.js, add `createDiagramCodeRenderer(renderer, options)`. It accepts any
object with a synchronous `render(text, options)` method, handles only the
configured fence language (`mermaid` by default), and falls back to ordinary
code when the renderer throws. Ferromark neither imports nor pins
`@ferrigura/core`; the application installs and owns the renderer, as it does
the Ferriki highlighter. Its output is trusted application HTML, like every
code-renderer callback; the SVG policy applies to the native integration only.

Depend on Ferrigura `0.3` with a caret requirement, as `ferriki` does. Moving
the adapter into the published crate makes Ferrigura's dependency licenses part
of the crate's policy. `deny.toml` allows them as version-scoped exceptions that
reach users only through the optional feature: MPL-2.0 for `cssparser` and
`dtoa-short`, EPL-2.0 for `ferrigura-elk`, `Apache-2.0 WITH LLVM-exception` for
`ferrigura-jsmath`, and OFL-1.1 and CC-BY-4.0 for the fonts and data in
`ferrigura-text`. It ignores RUSTSEC-2024-0449, an unmaintained notice for
`unsafe-libyaml`, a build-time dependency of `ferrigura-model`. A Ferrigura
upgrade must review these exceptions again.

## Consequences and validation

Rust applications enable one feature instead of depending on an unpublished
crate. Node.js applications can use Ferrigura from npm without a custom bundle
once `@ferrigura/core` is published
([ferrigura#3](https://github.com/sebastian-software/ferrigura/issues/3)); the
custom bundle remains the native path with the SVG policy.

Moving to Ferrigura 0.3 changed the pinned SVG bytes of the five shared
fixtures. The rendered fixtures were compared in Chromium before and after the
upgrade and were pixel-identical; the byte changes are in the embedded font
subsets. The fixture hashes were updated in the same change. The existing Rust
tests, now in `tests/ferrigura_integration.rs`, cover HTML, JSX, IDs,
diagnostics, errors, and fallback. Node.js tests cover the callback adapter.
Parser snapshots and conformance output are unchanged. No performance claim is
made.
