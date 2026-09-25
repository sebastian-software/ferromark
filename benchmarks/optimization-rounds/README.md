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
  --include-scanner-diagnostics --include-autolink-broad --include-container-diagnostics
python3 benchmarks/optimization-rounds/prepare.py \
  --baseline-path /tmp/fmv2-baseline --candidate-path /tmp/fmv2-candidate \
  --out /tmp/fmv2-build
python3 benchmarks/optimization-rounds/run.py \
  /tmp/fmv2-build /tmp/fmv2-corpus.json /tmp/fmv2-results \
  --rounds 3 --pairs 3 --window-ms 40
python3 -m unittest discover -s benchmarks/optimization-rounds -p 'test_*.py'
```

The full command above measures all four stages for all 233 cases and takes
several minutes. The optional container set contains 26 generated inputs for
blockquote, list, tab, lazy-continuation, GFM-table and line-comment shapes.
`--filter REGEX` and `--modes fresh reuse parse render` select subsets. The
actual study used four suites with different stage selections; its exact
commands are archived in the report. Do not run measurements in parallel with
each other or with compilation.

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
bits 1/2/4. `gfm-comments` adds line comments to strict GFM for container
diagnostics. `autolink` uses CommonMark parsing and enables renderer bare-URL
recognition; other HTML comparison options stay fixed. The 57 `autolink-broad`
replays use that profile even when the original case used GFM, so they form a
separate within-profile comparison, not another sample in the original broad
geometric mean. Authored diagnostics, including the optional container set,
are also excluded from broad means.

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

`filters/screen.txt` (20 documents and 16 table and scan diagnostics) and
`filters/broad.txt` (the 57 broad documents) are the case sets the recent
rounds screened and confirmed with; pass one with
`--filter "$(cat benchmarks/optimization-rounds/filters/broad.txt)"`.
`step_summary.py <results>` renders a run as a Markdown table: stage geomeans,
per-round geomeans, how many cases improved, and every case below 0.970.

## x86-64 on CI

The rounds so far measured on Apple Silicon only. The `x86-64 paired
benchmark` workflow (`.github/workflows/bench-x86.yml`) runs this harness
unchanged on three GitHub-hosted `ubuntu-latest` runners in parallel. Start it
from the Actions tab with a baseline and a candidate revision; using the same
revision for both gives an A/A control. A pull request that changes the
harness or the workflow runs that A/A control on its own head.

The workflow's `containers` case choice generates the 26 targeted container
diagnostics and selects `filters/containers.txt`. Parser-only pull requests do
not trigger the workflow; manually dispatch it with the reviewed baseline and
candidate revisions. `screen` and `broad` keep their existing case sets.

Baseline and candidate alternate on the same VM, so differences between
runner hosts largely cancel out. What remains is the noise of a shared VM,
which varies from host to host. Each host is an independent measurement of
the same pair. Read the three together, compare them with an A/A control
from the same day, and treat effects inside the A/A spread as ties. Local
Apple Silicon numbers stay the reference for small effects. The workflow suits
changes whose expected effect clearly exceeds that spread, such as new x86
SIMD paths.

## Profiling

`profile/` is a standalone driver crate (not a workspace member) that loops one
stage over a corpus suite for a fixed time, so a sampling profiler can attribute
where it goes. Each document contributes about the same byte volume per sweep.
It builds with fat LTO, one codegen unit and line tables:

```sh
cargo build --release --manifest-path benchmarks/optimization-rounds/profile/Cargo.toml
benchmarks/optimization-rounds/profile/target/release/ferromark-profile corpus.json parse 20
```

The stages are `parse`, `render` (documents parsed once up front) and `reuse`;
an optional fourth argument picks the corpus suite (default `broad`). On macOS,
`sample <pid> 10` gives a call tree; on Linux use `perf record -g`.

The `x86-64 profile` workflow (`.github/workflows/profile-x86.yml`) samples the
driver with perf on two GitHub-hosted runners for any revision. Its RUSTFLAGS
are `-C target-cpu=generic -C force-frame-pointers=yes`, so the build matches
the published addons, which pick SSSE3/AVX2 paths at run time, and perf gets
frame-pointer call chains. It keeps self-time, inclusive and caller reports as
artifacts. A pull request that changes the driver or the workflow profiles its
own head.
