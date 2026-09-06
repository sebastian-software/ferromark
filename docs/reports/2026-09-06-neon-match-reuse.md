# NEON position and match-cache experiments — 2026-09-06

## Decision

**Do not adopt any of these six variants.** Direct NEON hit localization and
reusing a block's matches both produced workload-specific gains, but the gains
were offset by repeatable regressions or insufficient improvement on mixed
Markdown. Production source and the public byte-search API remain identical to
baseline `dc7716be5728f233353c8cec789597cd0e33c85a` (merged PR #283).

This is a completed experiment, not an implementation waiting to be enabled.
The source patches and expanded tests are archived for inspection. Revisit these
approaches only if new profiles, a different target, or a substantially different
algorithm justify it.

## Hypotheses and isolated changes

1. Scalar localization after a positive NEON comparison costs enough to replace
   with a vector-derived first-hit index. Prediction: formatted and escaped text
   improve when the scalar search is removed, without sacrificing mixed Markdown.
2. Repeated mark searches redo comparisons for nearby hits. Prediction: caching a
   block's hits improves dense delimiter workloads in the complete parser.
3. Mask conversion and cursor bookkeeping can offset those gains. Prediction:
   short paragraphs and long gaps between delimiters regress; preserving the
   no-hit path or building the cache lazily should mitigate this.

Each variant started from the same committed baseline and was compared to the
same retained baseline executable. The cache experiments do not stack the
first-hit experiments.

- `first-min`: derive the first matching lane by replacing nonmatching lanes
  with 255 and taking the horizontal minimum of lane indices.
- `first-packed`: narrow the byte comparison vector into a 64-bit value carrying
  four bits per byte; trailing zeros give the first position on little-endian
  NEON. Big-endian NEON retains scalar localization.
- `small`: use packed localization only for sets of at most five bytes; large
  inline sets retain the existing scalar localization after the vector test.
- `finder`: add a safe, input-bound `ByteFinder` with `find_from`, cache each
  matching block, and use it in mark collection. The parser still controls run
  lengths and escape skipping. Cache queries support repeated offsets, forward
  jumps, backwards queries and out-of-range offsets.
- `finder-guard`: preserve a NEON no-hit check before converting a block into
  packed positions, testing the cost of conversion on empty blocks.
- `finder-lazy`: perform the first search with the baseline implementation and
  materialize the other positions only when another query can reuse the block.

The cache variants include an SSE2 mask backend and a scalar fallback, but their
runtime performance was measured only on this AArch64 host. No x86-64 speedup or
cross-platform runtime validation is claimed for these rejected prototypes.

## Measurement

Apple M1 Pro, rustc 1.97.1 / LLVM 22.1.6, repository Apple M1/NEON flags, release
fat LTO, one codegen unit, no PGO. Complete Markdown-to-HTML calls are timed using
`examples/core_performance.rs` and `scripts/compare-core-performance.py`.

The screening covers 13 inputs × 3 presets × fresh/reused rendering = 78 cases,
with three alternating baseline/candidate pairs and three timing windows per
case. Windows are 40 ms for `first-min`, 35 ms for the other variants. Input
creation and HTML verification are outside timing. Every compared output was
byte-identical in every pair.

Changes below are median paired elapsed-time ratios minus one; negative is
faster. Ranges span presets and fresh/reused rendering; the mixed column spans
all 5 KB, 20 KB, 50 KB and 1 MB fixtures. These are local screening measurements,
not confidence intervals or cross-parser rankings. Differences near 1% alone
are not considered sufficient evidence for added implementation complexity.

| Variant | Emphasis | Links | Mixed CommonMark |
| --- | --- | --- | --- |
| `first-min` | -4.61% to -2.90% | +2.69% to +3.19% | -1.83% to +0.47% |
| `first-packed` | -4.93% to -2.89% | +0.59% to +2.17% | -1.12% to +0.78% |
| `small` | -1.38% to -0.06% | -0.76% to +0.41% | -1.28% to +1.94% |
| `finder` | -3.65% to -1.48% | -1.89% to +0.40% | +0.10% to +1.35% |
| `finder-guard` | -4.07% to -1.89% | -1.37% to +1.12% | -1.17% to +0.87% |
| `finder-lazy` | -2.14% to -0.16% | +1.80% to +5.01% | +0.31% to +3.02% |

## Longer counterchecks

The most relevant regressions were checked again with **five alternating pairs
and three 100 ms windows per case**, using all presets and both rendering APIs:

- `first-packed` (links, all presets/APIs): **+1.68% to +2.60%** more elapsed time.
- `small` (tables parsed as CommonMark, fresh/reused): **+2.11% to +2.61%** more elapsed time.
- `finder-guard` (late first delimiter, all presets/APIs): **+1.64% to +3.48%** more elapsed time.

All confirmation runs preserved identical HTML. The targeted gains from the
screening do not outweigh these confirmed costs and the lack of a substantial
mixed-Markdown improvement. The lazy variant also worsened mixed inputs across
its screening matrix, so it did not warrant longer confirmation.

The evidence supports hypothesis 3 as the limiting factor for these particular
implementations. It does not prove that all vector localization or match-cache
algorithms are slower. No instruction-level profile was collected, so the exact
split between conversion, bookkeeping, inlining and code layout remains an
inference rather than a measured attribution.

## Correctness and cleanup

- Each first-hit variant passed four existing scalar-reference tests and its
  crate doctest.
- Each cache variant passed those tests plus two new integration tests and its
  doctest. The new tests exercise all 65,536 possible 16-byte hit masks, repeated
  and backwards queries, forward jumps, unaligned input, tails, empty sets, NUL,
  high bytes and offsets beyond the input.
- The initial cache implementation passed the full root all-features suite:
  969 tests passed, 3 ignored. Later cache changes passed the expanded crate
  tests and all 78 output comparisons.
- All prototype changes and prototype-only APIs/tests were removed from the
  working tree after archiving. `git diff --exit-code -- src crates` confirms
  the production implementation is unchanged from the baseline.
- After restoring the baseline, all 969 root tests and the four crate tests plus
  doctest passed again. Workspace Clippy with warnings denied, workspace formatting,
  and the README/benchmark CI contracts passed.
- Every archived patch was checked with `git apply --check` against that baseline.
  No temporary instrumentation was introduced into production code.

## Reproduction and evidence

[The artifact directory](2026-09-06-neon-match-reuse/) contains compressed source
patches, complete timing JSON, command output and test logs. `metadata.json`
records compiler/build details, the baseline commit, executable hashes and
artifact hashes. Gzip preserves the evidence bytes while keeping review size
manageable. Each patch applies independently to the baseline.

In a disposable checkout of the baseline, build and save the baseline executable,
then apply one decompressed patch and save that executable separately:

```sh
cargo build --locked --release --example core_performance
cp target/release/examples/core_performance /tmp/neon-baseline
gzip -dc /path/to/artifacts/finder-guard.patch.gz | git apply
cargo test -p ferro-byte-search --locked
cargo build --locked --release --example core_performance
cp target/release/examples/core_performance /tmp/neon-candidate
python3 scripts/compare-core-performance.py \
  /tmp/neon-baseline /tmp/neon-candidate /tmp/neon-confirm.json \
  --pairs 5 --window-ms 100 --filter late_special
```

Use a fresh baseline checkout for each variant. Remove `--filter` to repeat the
complete matrix. Measurements on another machine or toolchain constitute a new
run, not a reproduction of the exact recorded numbers.
