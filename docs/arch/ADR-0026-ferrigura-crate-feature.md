# ADR-0026: Integrate Ferrigura like Ferriki

- Status: Accepted
- Date: 2026-10-31
- Last updated: 2026-11-11
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

Depend on Ferrigura `0.7` with a caret requirement, as `ferriki` does. Moving
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
crate. Node.js applications use Ferrigura from npm (`@ferrigura/core` 0.6)
without a custom bundle, through the same code hooks as Ferriki
([ADR-0027](ADR-0027-injected-code-engines.md)); the custom bundle remains the
native path with the SVG policy.

Moving to Ferrigura 0.3 changed the pinned SVG bytes of the five shared
fixtures. The rendered fixtures were compared in Chromium before and after the
upgrade and were pixel-identical; the byte changes are in the embedded font
subsets. The fixture hashes were updated in the same change. The existing Rust
tests, now in `tests/ferrigura_integration.rs`, cover HTML, JSX, IDs,
diagnostics, errors, and fallback. Node.js tests cover the callback adapter.
Parser snapshots and conformance output are unchanged. No performance claim is
made.

## History

- 2026-11-05: Upgraded to Ferrigura 0.4. Only the embedded font subset of the
  flowchart fixture changed; geometry, text and the other four fixtures are
  byte-identical. The license exceptions moved to the 0.4 crates. Node.js
  injection now follows [ADR-0027](ADR-0027-injected-code-engines.md).
- 2026-11-10: Upgraded to Ferrigura 0.6. Diagram colors are `light-dark()`
  values that follow the page's `color-scheme` instead of a
  `prefers-color-scheme` block, so the pinned bytes of all five fixtures
  changed. The SVG policy already allowed `light-dark()`. The dependency
  licenses are unchanged, and the exceptions moved to the 0.6 crates.

- 2026-11-11: Adopted published Ferrigura 0.7 for caller-owned custom fonts.
  Ferromark re-exports the engine and accepts its `Renderer` in the public Rust
  hook constructor, so upgrading from the shipped 4.0.0 dependency on 0.4
  (or unreleased main's 0.6) is a breaking type-identity change. The ordinary
  breaking Conventional Commit initially prepared Ferromark 5.0; callers
  align a direct dependency to 0.7 or use the re-export. The three exact
  Ferrigura license exceptions were reviewed against the published 0.7 crates;
  their licenses are unchanged. Existing SVG sanitization accepts actual
  custom-font payloads and CSS in HTML and JSX without a policy change. See
  [the migration guide](../migration-v4.1.md) and
  [issue #614](https://github.com/sebastian-software/ferromark/issues/614).

  The five actual 0.6/0.7 fixtures retain their view boxes, text, and embedded
  font bytes. Flowchart is byte-identical; four hashes reflect compact sequence
  participants, class/state colors, and an opaque ER label mask. The fixture
  README records the per-diagram comparison.

- 2026-11-11: The owner confirmed that all current Ferromark consumers are
  internal and chose a coordinated direct migration in 4.1.0. The optional
  Rust 0.4/0.6-to-0.7 type boundary and original `BREAKING CHANGE` history remain
  explicit. No legacy renderer adapter is added. A one-shot `Release-As: 4.1.0`
  commit footer makes the existing Release Please flow prepare that version;
  generated release files are left to the updater. This exception is limited
  to this adoption. Release Please's default version selection remains unchanged.
