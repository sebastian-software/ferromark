# Optimization experiments

The [round report](../../docs/reports/2026-09-14-optimization-rounds/README.md)
records accepted and rejected variants, exact patches, build controls, raw
samples, output verification, and arena occupancy measurements.

The latest [complete Apple Silicon rerun](../../docs/reports/2026-09-15-arm-full-suite/README.md)
compares main `a7f0a00` with the finished performance branch `e93394e` on all
207 cases and all four stages. Its archived preparation script pins that newer
baseline; use the report's reproduction instructions to replay it. The original
[individual ARM attempts](../../docs/reports/2026-09-15-arm-iterations/README.md)
and the separate [six-engine comparison](../../docs/reports/2026-09-15-native-arm/README.md)
remain available with their own profiles and build settings.

The live preparation script's original reference core remains
`4de75d4843747a218771b5ec46df9171d4f54a15`. Rounds that compare against the
last promoted commit pass that revision with `--baseline-revision <rev>`;
the baseline checkout is then verified against `git archive <rev>` instead of
the frozen reference core, and `build.json` records the resolved hash.
Prepare isolated source checkouts of the baseline revision and the desired
candidate; then run, using fresh output directories:

```sh
python3 benchmarks/optimization-rounds/make_corpus.py /tmp/fmv2-corpus.json \
  --include-scanner-diagnostics --include-autolink-broad
python3 benchmarks/optimization-rounds/prepare.py \
  --baseline-path /tmp/fmv2-baseline --candidate-path /tmp/fmv2-candidate \
  --out /tmp/fmv2-build
python3 benchmarks/optimization-rounds/run.py \
  /tmp/fmv2-build /tmp/fmv2-corpus.json /tmp/fmv2-results \
  --rounds 3 --pairs 3 --window-ms 40
python3 -m unittest discover -s benchmarks/optimization-rounds -p 'test_*.py'
```

The full command above measures all four stages for all 207 cases and takes
several minutes. `--filter REGEX` and `--modes fresh reuse parse render` select
subsets. The actual study used four suites with different stage selections;
its exact commands are archived in the report. Do not run measurements in
parallel with each other or with compilation.

`prepare.py` snapshots both core sources beside the temporary worker crates,
checks the baseline against Git, checks exact source lock agreement and resolved
registry versions/checksums, and freezes worker bytes and binary hashes. Use
`--lto fat` (default), `thin`, or `off` for an explicitly labeled build control.
`--reuse-baseline-build PATH` can reuse a matching baseline binary. Changing
sources, locks, worker bytes, or LTO invalidates reuse. The runner validates the
frozen copy in the build directory, so later source edits cannot silently change
what an archived run claims to measure.

The worker preserves the original SIMD round's timed loops and output barriers.
Inputs and verification are outside timing. Fresh includes a new source-sized
arena, parsing, rendering to owned HTML, and destruction; reuse includes parsing,
borrowed HTML, and arena reset; parse omits rendering and consumes the AST via
`black_box`; render repeatedly renders a prebuilt AST. Every measured batch is
surrounded by exact HTML/AST/child-count checks, and every sample's iteration
checksum is checked. Arena capacity changes are recorded and allowed.

Profiles `commonmark`, `gfm` (footnotes off), `mdx`, and `extensions` follow the
first SIMD study's options, except that current timing and allocation workers
omit smart punctuation after its [removal from the core](../../docs/typography.md).
Both compared cores use that reduced profile. Historical reports retain their
original configuration; reproduce them with their original harness revision.
`opt-0` through `opt-7` enable the MDX/math/superscript
bits 1/2/4. `autolink` uses CommonMark parsing and enables renderer bare-URL
recognition; other HTML comparison options stay fixed. The 57 `autolink-broad`
replays use that profile even when the original case used GFM, so they form a
separate within-profile comparison, not another sample in the original broad
geometric mean. The 93 authored diagnostics are also excluded from broad means.

`allocations.py` uses the prior global counting allocator in a separate worker;
it records changing counts rather than rejecting the structural optimization.
Its profiles are the original four profiles, not `opt-*` or `autolink`.
`arena_usage.py` builds an additional untimed parse-only GFM diagnostic inside an
allocation-worker build. It records occupied chunk bytes including alignment
padding independently of reserved arena capacity. Do not interpret lower occupied
bytes as lower system allocation requests when the reserved capacity is equal.

The generator reads the repository's frozen corpora without network access.
Original document attribution remains in the broad comparison and first SIMD
reports. New authored diagnostic inputs are MIT licensed with this repository.
