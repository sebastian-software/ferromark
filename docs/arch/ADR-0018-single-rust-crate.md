# ADR-0018: Publish one Rust crate

- Status: Accepted
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
`ferromark` crate. GitHub releases contain one Rust and nine npm archives.
This supersedes the five-crate packaging decision in ADR-0017.

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
