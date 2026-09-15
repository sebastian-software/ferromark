# ADR-0015: Develop v2 in the Ferromark repository

Status: Accepted for the v2 development branch.

## Decision

Import the clean `ferromark-v2` source at `e9dd98b` into `codex/v2` using a
merge with both histories as parents. The original Ferromark parent is
`523e77e`. Preserve the full imported history, attribution, fixtures, and
measured reports; the sibling source repository remains unchanged.

The root now contains the v2 virtual Rust workspace. This supersedes the
v1 streaming/no-AST decision (ADR-0001) on this branch: v2 uses OX-Content's
arena-allocated AST. Earlier v1 ADRs remain available in the first parent's
history; v2 feature decisions remain under `docs/decisions/`.

Reuse the repository's standards, pinned actions, dependency checks, README
themes, issue templates, Node package scaffolding, and Ardo/Ferramenta site.
Adapt CI to workspace tests, the v2 MSRV, benchmark compilation and harness
checks. Keep Node's eight native targets, consumer floor, package verification,
highlighter interface, and panic-unwind boundary.

## Compatibility and publication

The Rust API is intentionally breaking. Node keeps its public rendering entry
points but maps options to v2 semantics; unsupported v1 options fail explicitly.
See [the migration guide](../migration-v2.md). The website describes this actual
surface and links archived v2 evidence without relabeling v1 measurements.

All packages remain unpublished. The v1 publisher and fuzz/benchmark workflows
cannot operate on this tree and are not enabled here. Release preparation is
tracked in `docs/releasing.md`. Homepage deployment stays restricted to `main`.

The inherited managed rustfmt configuration replaces upstream's dense layout;
formatting is mechanical and does not change parser behavior or fixture bytes.
The upstream MIT notice and separately licensed specification fixtures remain
intact. The Node distribution includes both upstream and local MIT notices.

The sibling checkout contained 50 older report CSVs with CRLF bytes that its
Git text attributes normalized to LF. Import those files from the clean source
checkout and mark all report CSVs binary for text conversion, preserving their
original measured bytes and checksums on every platform.

Coverage keeps the inherited 90% line floor. Corpus-wide no-op hook equivalence
tests validate the path now used by Node highlighters, including configured
extensions and reused renderer state. The coverage lane runs tests serially
so instrumented timing guards do not compete with one another.

See [integration validation](../v2-integration-validation.md) for the checks,
platform limits, and preserved-source audit.
