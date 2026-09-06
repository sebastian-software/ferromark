# ferro-byte-search (unpublished)

A small `no_std` Rust library for searching a slice for any byte in a reusable
set. No allocation, runtime dependencies, C/C++, or Markdown knowledge.

```rust
use ferro_byte_search::ByteSet;

const ESCAPES: ByteSet<4> = ByteSet::new(b"&<>\"");
assert_eq!(ESCAPES.find(b"plain < text"), Some(6));
assert!(ESCAPES.contains_any(b"a & b"));
```

`ByteSet::new` is `const` and copies the supplied array. `N` includes duplicate
bytes; duplicates do not change results. Empty sets never match. All 256 byte
values are supported, including NUL, high bytes, and invalid UTF-8. Results are
byte offsets within the provided slice.

Build the set once and reuse it. It stores the supplied bytes and a 256-byte
membership table: SIMD compares a vector against the set, and scalar tails or
first-hit localization use the table. Tiny, fixed sets benefit from constant
propagation. Larger sets are correct but this initial implementation is aimed
at the small sets encountered in text scanning, not a promise of optimal
performance for every cardinality.

## Backends and safety

- x86-64: baseline SSE2.
- AArch64 with NEON enabled: NEON.
- Other targets: scalar table lookup.

The public API is safe. The two architecture-specific load/comparison helpers
contain the only `unsafe` blocks. Full readable vectors are checked before
loading, loads accept unaligned slices, and partial short inputs are copied
into a stack buffer. Padded NUL bytes cannot become out-of-range matches.
The final overlapping chunk only returns matches from the unscanned tail.
Neither backend requires optional CPU features or runtime dispatch.

AVX2, Highway, and alternative Rust SIMD frameworks are future experiments;
none is required by this extraction. Existing `memchr` use for individual bytes,
small byte alternatives, and long escape scans remains in Ferromark.

## Source and packaging boundary

This workspace crate is intentionally `publish = false`. Its public wrapper
compiles the generic search implementation from `src/byte_search.rs` at the
repository root. Ferromark embeds that exact same source as a private module.
There is one implementation, with no copied/generated synchronization step.

This arrangement keeps Ferromark publishable: Cargo cannot publish a dependency
on an unpublished path-only crate, and nested package sources are omitted from
the parent package. Ferromark's package therefore owns the shared source until
this crate is ready for an independent release. The private crate currently
requires the repository layout; it is not a standalone distributable tarball.
Its integration tests and Criterion benchmarks call it through a real crate
boundary. A future publication can move source ownership and replace the
embedded module with a versioned dependency.

Markdown syntax sets remain in `src/inline/simd.rs`; HTML escape policy and the
short/long scan choice remain in `src/escape.rs`. Neither is part of this API.

## Checks and benchmarks

From the repository root:

```sh
cargo test -p ferro-byte-search --locked
cargo clippy -p ferro-byte-search --all-targets --locked -- -D warnings
cargo bench -p ferro-byte-search --bench search -- --test
```

Tests compare to scalar membership on vector boundaries, overlapping tails,
unaligned subslices, all possible byte values, empty/duplicate sets, NUL padding,
and deterministic binary inputs with runtime-generated search sets. The CI
matrix runs them on Linux, macOS, Windows, and the repository MSRV.

The benchmark includes absent, early, late, and dense matches, 0–65536 bytes,
and sets of 4 and 12 bytes. `find` and `embedded_find` compare the same engine
through the external crate and a local module, respectively. `contains` avoids
localizing the first matching lane. Inputs and search-set construction are
outside the timer. A bounded subset:

```sh
cargo bench -p ferro-byte-search --bench search -- \
  '^(four|twelve)/(8|64|1024)/(find|embedded_find)/(absent|last)$'
```

The existing `examples/core_performance.rs` and
`scripts/compare-core-performance.py` compare complete retained parser
executables and require byte-identical HTML. Use that end-to-end check before
accepting an optimization; scanner microbenchmarks alone are insufficient.

See the [extraction report](../../docs/reports/2026-09-05-byte-search-extraction.md)
for retained parser comparisons, crate-boundary measurements, and build checks.
