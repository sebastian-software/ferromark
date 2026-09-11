# Native cmark and cmark-gfm comparison

**Date:** 2026-09-11

**Base:** main `223cff7495435c8fc44713fd4852ff91e0327bf2` (PR #295).

**Measured revision:** `315e8163e7d29cfedc5f69e005fc2d7ca7da1698`.

**Scope:** Targeted native Markdown-to-HTML comparison, using system allocators.

This first measurement adds the C CommonMark reference implementation and
GitHub's fork to the existing native comparison work. All selected inputs passed
the shared output review before timing. The table below is generated from the
archived samples; smaller times are better. Each competitor has its own
Ferromark baseline measured in the same process.

<!-- native-cmark-measurements:start -->

| Native pair | Input | Bytes | Ferromark µs | Competitor µs | Competitor / Ferromark | Run-median range, Ferromark / competitor µs |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| cmark | `commonmark/5k` | 5,120 | 25.32 | 80.61 | 3.18× | 24.94–25.80 / 79.04–81.89 |
| cmark-gfm | `commonmark/5k` | 5,120 | 25.05 | 75.78 | 3.03× | 25.03–25.22 / 75.27–75.95 |
| cmark-gfm | `tables/tables-plain` | 4,560 | 26.72 | 210.86 | 7.89× | 26.70–26.85 / 210.34–214.65 |
| cmark-gfm | `tables/tables-links` | 6,360 | 38.35 | 257.90 | 6.73× | 38.23–39.09 / 255.53–264.17 |
| cmark-gfm | `tables/tables-commonmark-inline` | 4,800 | 33.32 | 241.62 | 7.25× | 33.22–33.48 / 238.01–242.99 |

<!-- native-cmark-measurements:end -->

These observations cover one 5-KiB CommonMark document and the three realistic
table fixtures added in PR #295. They do not establish a general performance
ranking across documents, machines, or feature configurations. The run-median
ranges show the observed variation across three processes, not confidence
intervals. The two independent Ferromark baselines must not be combined.

## Measured work

Both engines parse the input and render a newly owned HTML output on every
iteration. Parser state, any AST, and the output are destroyed inside the timed
call. cmark's native output is freed using its own allocator, without copying it
into a Rust String or scanning its length during timing. There are no Node.js
bindings, WASM, cached ASTs, or per-document process startup costs. Fixture I/O,
output validation, JSON serialization, and registry initialization are outside
timing; attaching selected extensions to a fresh parser is inside timing.

The CommonMark lane disables all extensions. The table lane enables only tables
in both engines. All lanes use trusted rendering, preserving raw HTML and URL
schemes. Other available verification lanes cover double-tilde strikethrough,
tasks, and the shared three-extension overlap. That overlap is not full GFM:
bare autolinks and tag filtering remain off. The fork's double-tilde option is
explicitly set when strikethrough is enabled and checked by a negative test for
single tildes. The targeted timing run did not select these additional lanes.

## Output and specification diagnostics

All five cmark workloads and all 18 cmark-gfm workloads were admitted before
timing. All selected timing workloads have equivalent normalized HTML. The
task-only verification fixture has a reviewed checkbox presentation difference;
its original outputs and the admission decision are retained. No unsupported
table workload is timed through core cmark.

The separate 652-example CommonMark diagnostic has no normalized mismatches for
Ferromark or cmark 0.31.1. The pinned cmark-gfm release has 11 mismatches against
this corpus. These diagnostics are not a claim of full GFM conformance, and
workload admission is distinct from specification conformance. Raw expected and
actual HTML is available in each archived `spec-output.json.gz`.

## Environment and protocol

- Apple M1 Pro, arm64, macOS 26.6.2; system allocators for both engines.
- Rust 1.97.1 (`8bab26f4f`, LLVM 22.1.6), generic CPU target, optimization level
  3, fat LTO, one codegen unit, panic abort.
- CMake 3.31.6 and AppleClang 21.0.0.21000334; upstream static C libraries built
  with Release `-O3 -DNDEBUG`. No PGO or cross-language LTO.
- cmark 0.31.1 at `bb3678d7a73cb02d35c8876ecd097072636200a8`;
  cmark-gfm 0.29.0.gfm.13 at `587a12bb54d95ac37241377e6ddc93ea0e45439b`.
- Three process runs per pair, three seconds of warmup per engine/case, then 80
  alternating-order windows of at least 63 ms per engine/case. The timer is
  checked every 16 renders. Reported times are medians of the three run medians.

Each executable links Ferromark with only one C library because the libraries
export conflicting symbols. C and Rust retain their own release build settings;
the complete commands and compiler configurations are archived. These results
use a different allocator/toolchain environment from the published Bun/mimalloc
comparison and do not update its README or homepage figures.

## Reproduction and evidence

Follow the [harness build instructions](../../benchmarks/cmark-comparison/README.md)
at the measured revision, then run:

```sh
python3 benchmarks/cmark-comparison/run.py /tmp/ferromark-cmark-build \
  /tmp/ferromark-cmark-measured \
  --case commonmark/5k \
  --case tables/tables-plain \
  --case tables/tables-links \
  --case tables/tables-commonmark-inline
```

The [evidence directory](2026-09-11-native-cmark-comparison/) contains metadata,
source and executable hashes, effective options, all verification outputs,
specification diagnostics, selected inputs, raw timing samples, summaries,
the dependency lockfile, CMake configuration, and the source patch (empty for
this clean measured revision). Large JSON files are losslessly gzip-compressed.
`finished_unix` in the metadata confirms the runner completed its final build
input and executable validation.

Verify archived integrity and regenerate/check the numeric table with:

```sh
(cd docs/reports/2026-09-11-native-cmark-comparison && shasum -a 256 -c SHA256SUMS)
python3 docs/reports/2026-09-11-native-cmark-comparison/render.py --check
```

The generator independently recomputes every run median from the raw samples
and checks the saved summaries before producing the table. Subsequent additions
are described in the [native comparison expansion plan](../plans/2026-09-11-native-comparison-expansion.md):
Goldmark, Sätteri Markdown, then MDX after defining comparable output stages.
