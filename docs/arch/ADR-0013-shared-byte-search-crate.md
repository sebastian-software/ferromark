# ADR-0013: Shared byte-search implementation and unpublished crate

**Status:** Accepted
**Date:** 2026-09-06

## Context

Inline-special detection, mark collection, and short HTML escape scans need
similar SIMD byte-set searches. Duplicated architecture-specific implementations
spread unsafe load contracts across the parser and make independent experiments
harder. Markdown syntax and escaping policy still belong to Ferromark.

## Decision

Expose a small safe `ByteSet<N>` API through the unpublished workspace crate
`ferro-byte-search`. It provides `find` and `contains_any`, accepts arbitrary
bytes, and supports `no_std` without allocation or runtime dependencies. Native
SSE2 and NEON loads are confined to checked helpers, with scalar lookup on other
targets. Byte sets retain a precomputed membership table for scalar tails and
first-hit localization.

During this unpublished phase, `src/byte_search.rs` at the repository root owns
the implementation. The workspace crate compiles that source through a path
module, and Ferromark compiles it as a private module. This keeps a single source
while allowing Ferromark's Cargo package to build without an unpublished path
dependency. Cargo excludes nested package sources from the parent archive.

Markdown-specific sets and option selection stay in the inline parser. HTML
escape policy and the existing short/long scanning threshold stay in the escape
module. Existing long-input `memchr` scans remain in place. Highway and other
SIMD frameworks are not required by this extraction.

## Consequences

- The independent API and its architecture backends can be tested and benchmarked
  without building the Markdown parser.
- The unpublished crate requires the repository layout and is not independently
  distributable yet. Publishing it later requires moving source ownership and
  adding a versioned Ferromark dependency.
- Microbenchmarks cover both the external crate and the embedded module. Complete
  parser benchmarks remain the acceptance check; the crate boundary is not
  assumed to be free under every compiler and LTO configuration.
- Runtime state and document semantics remain in the parser. Reused block scratch
  retains only cleared allocation buffers between renders.

## Evidence

The [extraction report](../reports/2026-09-05-byte-search-extraction.md) preserves
paired parser measurements, scalar-reference checks, crate-boundary timings,
and verification of the actual Cargo package. Those measurements predate this
PR's update to the current main branch and are historical evidence, not a fresh
benchmark of the rebased source.
