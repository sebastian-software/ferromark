# Direct native Bun Markdown comparison — 2026-09-05

Bun's Rust Markdown parser can be built and measured without its JavaScript
runtime. The [reproducible harness](../../benchmarks/bun-comparison/README.md)
now compares its unchanged Rust parser and HTML renderer against ferromark,
pulldown-cmark, and comrak. Bun's native SIMD search kernels and mimalloc are
retained; the surrounding executable is replaced by the benchmark driver and
a macOS pthread stack-bound adapter.

## Provenance

The documented chain is **md4c (C) → Bun's Zig port → Bun's Rust port**.
The [January 29 introducing commit](https://github.com/oven-sh/bun/commit/1bfe5c6b37e65995ac58e761ccbff7bb7b8dc954)
explicitly names the C-to-Zig port. The Rust sources arrived in the
[May 14 Rust rewrite](https://github.com/oven-sh/bun/commit/23427dbc12fdcff30c23a96a3d6a66d62fdc091d).
The [June 25 cleanup](https://github.com/oven-sh/bun/commit/d4514457e837fc3c884496c70a90df555bb95221)
confirms that remaining Zig sources were behavioral references, not compiled implementations.

Later fixes and optimizations mean this is a maintained md4c-derived
implementation, not a frozen translation. See [the provenance investigation](../../benchmarks/bun-comparison/PROVENANCE.md)
for further primary-source links. Comparing Bun and C md4c would compare
implementations with shared ancestry; it would not isolate a Rust-versus-C
language effect. C md4c remains in the existing separate harness and was not
re-measured in this four-Rust-API run.

## Exploratory measurements

Apple M1 Pro, macOS; Rust 1.99.0-nightly (2026-07-19), LLVM 22.1.8, pinned by
Bun's `nightly-2026-07-20`. Bun source:
`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`.
Pulldown-cmark 0.13.4 and comrak 0.54.0 are pinned.

All Rust candidates use generic CPU code generation, fat LTO, one codegen
unit, and Bun's original global mimalloc. Each call produces a fresh owned
HTML output and drops it inside the timed loop. No output or parser state is
reused. Options are prepared outside timing; raw HTML is trusted and preserved.
Bun's Highway search code uses its original runtime SIMD dispatch. Its libc
`memmem` alias is process-wide, including any calls from competitors.

Ferromark is commit `617a29dbe8833e5bdbb207186cfe573051cfd18c` **plus the local
performance edits from the preceding task**. The exact patch and source hashes
are included. This is not a released-version or clean-commit publication baseline.

Throughput in MiB/s; higher is better. Five 150 ms windows per parser/case,
50 ms warmup, rotating/reversing order; table values are medians of window means.

| Input/configuration | ferromark | Bun `bun_md` | pulldown-cmark | comrak | ferro / Bun |
| --- | ---: | ---: | ---: | ---: | ---: |
| commonmark/tiny | 33.36 | 52.32 | 59.77 | 27.14 | 0.64× |
| commonmark/prose | 748.37 | 188.34 | 497.96 | 191.72 | 3.97× |
| commonmark/links | 169.44 | 116.07 | 159.16 | 79.34 | 1.46× |
| commonmark/entities | 95.80 | 85.17 | 89.55 | 57.56 | 1.12× |
| commonmark/commonmark-5k | 311.14 | 160.98 | 278.66 | 120.14 | 1.93× |
| commonmark/commonmark-50k | 300.68 | 156.33 | 275.45 | 113.74 | 1.92× |
| gfm_overlap/gfm-tables | 96.82 | 92.20 | 114.52 | 32.43 | 1.05× |

The mixed CommonMark fixtures favor ferromark by about 1.93× over Bun in this
setup. The shared GFM table/strikethrough document is much closer (about 1.05×);
pulldown-cmark leads that case. The tiny 18-byte input favors Bun over
ferromark: approximately 328 ns versus 515 ns per fresh-output call.
The synthetic prose case should not be extrapolated to mixed Markdown.

These short local runs establish feasibility and indicative differences, not
confidence intervals or universal rankings. They use a different compiler,
allocator, output lifecycle, and trust configuration from the README's existing
benchmarks. The README headline has therefore not been changed.

## Output verification and exclusions

**14 of 20** input/configuration pairs passed the shared-output gate and were
measured. The gate retains exact output and applies a narrow serialization
normalizer (documented in the harness); it does not silently remove semantic
attributes or alter the Markdown inputs.

Excluded pairs:

- GFM mixed 5 KB, mixed 50 KB, and the aligned-tables fixture: Bun emits different
  table alignments. For example, a column emitted with right alignment by the
  other three implementations is emitted with center alignment by Bun.
- GFM tasklist/table document: Bun adds task-related CSS classes and uses different
  spacing; pulldown-cmark also serializes the checkbox-to-text gap differently.
  These are retained as output differences rather than normalized away.
- Mixed 1 MB in both configurations: ferromark leaves later reference links as
  literal text where the other three render links. This is consistent with its
  bounded reference-resolution work policy; no result from this input is used
  to claim superior throughput. The raw output is retained for inspection.

The separate `gfm-tables` input uses ordinary unaligned tables and strikethrough,
so it exercises real shared GFM features and passes the gate.

For the stored 652 CommonMark examples:

| Parser | Exact expected HTML | After limited normalization |
| --- | ---: | ---: |
| ferromark | 652/652 | 652/652 |
| Bun `bun_md` | 649/652 | 649/652 |
| pulldown-cmark | 630/652 | 652/652 |
| comrak | 652/652 | 652/652 |

Bun's three differences (556, 587, 649) are extra spaces before soft line breaks.
These counts describe our stored corpus and comparator, not a separate official
CommonMark certification. During initial dependency resolution, html-escape
0.2.15 changed Ferromark's numeric-NUL entity output; the final harness pins
0.2.14, matching Ferromark's checked-in lockfile. Initial exploratory numbers
with the newly resolved decoder are not included in these results.

## Evidence and validation

[Saved results](2026-09-05-bun-comparison/summary.json) contain every measured
case, all five raw window samples, medians, and throughput. The adjacent folder
also contains the full effective Cargo lockfile, source/executable/fixture
hashes, hardware/compiler details, patches, excluded-case diagnostics, and
original verification/spec/timing JSONL compressed with gzip. `SHA256SUMS`
covers the retained files. Decompress with `gzip -dc <file>.jsonl.gz`.

The final build was repeated with the saved lockfile and `--locked`. Five
verification-gate tests pass, including rejection of changed table alignment,
link destinations, text, and code whitespace. The three existing core
performance regression tests also pass. Rust formatting, Python syntax, and
git whitespace checks pass. Cargo's package listing confirms that the new
harness is excluded; normal builds and CI gain no Bun dependency.
