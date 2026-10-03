# ADR-0018: Publish one Rust crate

- Status: Accepted
- Last updated: 2026-10-03
- Date: 2026-09-15
- Amended 2026-09-16: the single crate is now the repository root package rather
  than a member under `crates/ferromark`, which is what Release Please's native
  `rust` strategy requires. Its sources, tests, benchmarks and examples live at
  the repository root and a Cargo `include` allow-list keeps the published
  archive to the file set described here. Nothing about the module boundaries,
  the public API or the decision to publish exactly one crate changes. See
  [ADR-0020](ADR-0020-standards-release-blueprint.md).

## Context

The owner questioned whether Ferromark's four separately published core crates
provide enough independent value to justify their release and versioning cost.
AST analysis, custom rendering and parser-only use are useful APIs, but these
capabilities do not require separate package identities. The components evolve
together and the development workspace already requires exact matching versions.
No external v2 users depend on the development-only crate names.

## Decision

Publish only `ferromark`. Move the allocator, AST, parser and renderer into public
modules with separate source directories, retaining top-level convenience
functions and reexports. Keep the native Node binding as a private workspace
crate, since it has its own cdylib build target and panic policy.

Preserve runtime algorithms, defaults, output, source spans and test expectations.
Keep module-internal items scoped to their original component where applicable.
Do not add a new optional-renderer Cargo feature in this structural change;
that can be considered independently if a concrete consumer needs it.

Move the seven benchmark suites, integration tests and fixtures into the single
package. Rename snapshot paths and unit-test identifiers mechanically without
changing their output bodies. Validate the public modules from integration tests.

Remove the unused internal version updaters, new-crate bootstrap flow and
cargo-deny path-dependency exception. Reuse Trusted Publishing for the existing
`ferromark` crate. The release workflow validates the core archive and every
npm facade/sidecar archive; later amendments define the optional transform
archive and the supported native targets. This supersedes the five-crate
packaging decision in ADR-0017.

## Validation

Require the existing workspace, Node, documentation and package checks. Compare
frozen before/after workers with the same compiler, worker, optimization settings
and external dependency graph. Local lockfile entries necessarily differ after
consolidation; verify each recorded lock hash and compare external entries in full.
Check exact HTML, debug AST and source spans before measuring both individual
real documents and complete-corpus batches. Retain the change only when these
checks and measured performance support it.

## Outcome

The consolidation passed the existing tests and package checks. The retained
[measurement report](../reports/2026-09-15-single-crate/README.md) records complete
corpus timings, individual regressions, unchanged outputs and build identities.

## Amendment (2026-09-25): optional transform extension

Keep `ferromark` as the single published **core** crate. Add
`ferromark-transforms` as one optional extension crate for the native AST pass
pipeline. The extension depends on `ferromark`; the parser and renderer do not
depend on the extension. Both packages share the root product version, and the
release strategy updates their explicit workspace manifests together. When the
extension is enabled for publishing, publish `ferromark` first and then
`ferromark-transforms`. This does not split the allocator, AST, parser or
renderer into separately released core crates.

The extension package and its archive rehearsal are part of the transform
foundation. The publish workflow lists the core before the extension.
The new crate needs an initial credentialed publication before crates.io
Trusted Publishing can be configured for subsequent releases. See [ADR-0022](ADR-0022-native-transform-pipeline.md).

## Amendment (2026-09-29): optional Ferriki integration

Expose the Ferriki code-highlighting adapter as the additive `ferriki` feature of
the root `ferromark` crate. Ferriki already publishes a reusable Rust highlighter
without grammar or theme payloads; the default feature set retains no Ferriki
dependency. A separate adapter crate would add release coordination for one
small bridge without changing the parser or renderer ownership boundary.

The application initializes and owns the Ferriki highlighter and its asset
source. Ferromark borrows it for synchronous rendering, forwards only normalized
code and language to Ferriki, and keeps code-block wrappers, metadata and
annotations. Unknown languages and highlighter failures use the existing escaped
plain-code fallback. Optional error observation stays with the application.
Ferriki's optional CDN source is an asset-source choice outside Ferromark.
The adapter does not select that source, but calls into a highlighter which can
lazily load directory or remote assets on first use unless callers preload the
required languages and themes. The feature does not enable Ferriki's remote-loading feature or change Ferromark's
default dependency graph.

Validate the feature with a compiled public-API example, a dedicated Rust-only
CI lane and integration tests. Preserve the existing Node callback path and
default HTML output. This amends the package boundary without adding a new
published crate or another release workflow entry.

## History

- 2026-10-03: Clarified the credentialed first publication required before
  enabling Trusted Publishing for the new transform crate, and clarified the
  archive contract after the later package and platform amendments.
