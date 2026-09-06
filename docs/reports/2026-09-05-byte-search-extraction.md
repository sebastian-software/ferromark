# Byte-set search extraction — 2026-09-05

## Result and boundary

Added the unpublished workspace crate `ferro-byte-search`: a reusable, safe
`ByteSet<N>` API with `find` and `contains_any`, no allocation, no runtime
dependencies, and `no_std` support. The implementation uses native Rust SSE2
on x86-64, NEON on enabled AArch64 targets, and scalar lookup elsewhere.
Highway and C/C++ bindings are not dependencies.

Inline-special detection, mark collection, and short HTML escape scans now
share this implementation. Markdown character sets, extension switches, HTML
escape policy, and the 128-byte short/long escape threshold remain in
Ferromark. Long escape scans retain their existing `memchr` strategy.

There are two internal `unsafe` blocks, one per architecture backend, behind
safe helpers that check a full readable vector before an unaligned load.
NUL, invalid UTF-8, empty sets, duplicates, and overlapping tails have defined
behavior and scalar-reference coverage. Each reusable set stores its supplied
bytes plus a 256-byte membership table. Large sets are supported for
correctness; the initial performance target is small text-scanning sets.

## Unpublished packaging

The public workspace wrapper compiles `src/byte_search.rs` from the repository
root. Ferromark embeds that same source as a private module: one implementation,
without a private runtime Cargo dependency or source-copy generation.

This is an intentional staging arrangement. An unpublished path-only dependency
would prevent a normal Ferromark publication, while Cargo omits nested packages
from the parent's archive. The actual `cargo package` archive was inspected and
successfully compiled; it contains the shared implementation. The private crate
currently requires this repository layout. A future independent release can
move source ownership into it and give Ferromark a versioned dependency.

## End-to-end measurement

Host: Apple M1 Pro, rustc 1.97.1, LLVM 22.1.6, repository Apple M1/NEON flags,
release fat LTO and one codegen unit. The baseline is the retained executable
built immediately before extraction: HEAD
`617a29dbe8833e5bdbb207186cfe573051cfd18c` plus the preceding uncommitted
core-performance work. These numbers do **not** compare against clean HEAD or
Bun. Earlier comparison reports remain historical measurements.

Five alternating baseline/candidate pairs, three 75 ms windows per case per
executable; 13 inputs × 3 presets × fresh/reused parser = 78 cases. The harness
checks byte-identical HTML on every pair. All comparisons passed. Changes below
are median paired elapsed-time ratios minus one; negative is faster. Ranges
cover all six preset/lifecycle combinations, not statistical confidence bounds.

| Input | Elapsed-time change |
| --- | --- |
| `tiny` | -3.90% to +0.92% |
| `prose` | -0.54% to +0.11% |
| `emphasis` | -6.14% to -4.91% |
| `links` | -1.46% to +0.18% |
| `entities` | -1.14% to +0.20% |
| `late_special` | +0.68% to +1.12% |
| `references` | -4.54% to -2.87% |
| `lists` | -2.81% to -1.13% |
| `commonmark_5k` | -3.12% to -1.39% |
| `commonmark_20k` | -3.14% to -2.33% |
| `commonmark_50k` | -3.09% to -2.26% |
| `commonmark_1m` | -4.32% to -2.54% |
| `tables` | -9.65% to -1.64% |

Mixed CommonMark inputs improve roughly 1–4%, emphasis about 5–6%, and reference
links about 3–5% on this host. The largest positive median is +1.12% for a late
special character. Small differences should be treated cautiously; there is no
claim of a universal speedup or x86-64 performance parity.

The extraction initially caused real regressions. Three hypotheses were ranked:
scalar first-hit localization, short-input padding, then crate inlining. A
repeated `bytes.contains` check in scalar localization caused the emphasis
regression; adding the precomputed membership table removed it. Reference-heavy
inputs still regressed until inputs shorter than 16 bytes bypassed SIMD padding
for sets larger than five bytes. Each change was measured independently before
the final full run. Rejected implementations are not in production code.

Evidence: `screen` (initial implementation, 3 × 30 ms), `table` (focused emphasis,
3 × 60 ms), `table-full` (3 × 40 ms), `short-refs` (3 × 60 ms), and `final`
(5 × 75 ms) JSON/log pairs in the adjacent artifact directory. Window counts
refer to the harness's three timing windows per case in each executable run.

## Actual crate-boundary microbenchmark

The Criterion benchmark compares the external crate against the exact same
source compiled as a local module. Search-set construction and input setup are
outside timing. All 216 registered cases passed the benchmark smoke check.
The bounded timed subset ran three times, with 30 samples, 100 ms warmup and
300 ms measurement per case. It uses the repository's release/LTO configuration.

The table shows medians of three mean estimates (nanoseconds); the last column
is the median of the three external/embedded ratios minus one.

| Set | Bytes | Pattern | External ns | Embedded ns | Difference |
| --- | ---: | --- | ---: | ---: | ---: |
| four | 8 | absent | 4.82 | 4.75 | +1.69% |
| four | 8 | last | 7.28 | 7.25 | -0.20% |
| four | 64 | absent | 3.86 | 3.87 | +0.00% |
| four | 64 | last | 9.54 | 9.52 | -0.49% |
| four | 1024 | absent | 46.53 | 46.69 | -0.02% |
| four | 1024 | last | 50.80 | 50.50 | +0.62% |
| twelve | 8 | absent | 3.49 | 3.48 | +0.18% |
| twelve | 8 | last | 3.80 | 3.80 | -0.10% |
| twelve | 64 | absent | 9.94 | 9.95 | -0.11% |
| twelve | 64 | last | 13.90 | 13.58 | +2.39% |
| twelve | 1024 | absent | 143.01 | 138.36 | +3.37% |
| twelve | 1024 | last | 142.79 | 141.59 | +0.85% |

Most cases are close; two 12-byte-set cases consistently cost about 2–3% more
through the external arrangement in this executable. An initial +8.47% result
for an 8-byte absent input did not repeat. This does not establish a general
zero-cost guarantee for a future dependency under every compiler/LTO setting.
Ferromark currently embeds the shared source, and its complete parser measurements
above are the acceptance check. Raw Criterion estimates from all three runs are
retained, including their confidence intervals.

Reproduce the bounded comparison:

```sh
cargo bench -p ferro-byte-search --bench search --locked -- \
  '^(four|twelve)/(8|64|1024)/(find|embedded_find)/(absent|last)$'
```

To compare retained full-parser executables:

```sh
python3 scripts/compare-core-performance.py \
  target/byte-search-extraction/baseline \
  target/byte-search-extraction/final /tmp/byte-search-final.json \
  --pairs 5 --window-ms 75
```

## Validation and limits

- Root `cargo test --all-features --locked`: 969 passed, 3 intentionally ignored,
  no failures; includes the retained 652-example CommonMark regression corpus.
- New crate: four scalar-reference integration tests and one doctest passed on
  stable and Rust 1.88. Tests exercise every byte value, vector boundaries,
  unaligned slices, overlapping tails, padding and runtime-generated sets.
- Root and new-crate Clippy checks with warnings denied passed.
- Scalar `thumbv7em-none-eabi` and baseline SSE2 `x86_64-unknown-linux-gnu`
  checks passed with warnings denied. The former verifies the `no_std` build.
- `cargo package --allow-dirty --locked --offline` successfully packaged and
  compiled Ferromark without an unpublished dependency.
- Contributor/CI, CI-hardening, benchmark-CI and native-matrix contract self-tests
  passed. The contributor contract now includes the new crate checks.
- CI now explicitly runs the new crate tests in the existing OS/MSRV matrix and
  lints its targets. These edits have not been pushed or run in remote CI.

Local runtime and performance verification is AArch64 only. x86-64 was
cross-checked at compile time; this host has no working Rosetta runtime.
The configured CI jobs provide the next x86-64 runtime check.

The adjacent `metadata.json` records toolchain, build flags, executable hashes,
and relevant source hashes. `tracked-source.patch` captures the tracked working
tree diff against HEAD, including preceding retained work; it is not a standalone
patch for untracked files. Baseline escape/SIMD source snapshots are also kept.
New source, tests and harnesses live in this change. Test logs and benchmark
JSON retain the observed results, rather than just the selected summary rows.
