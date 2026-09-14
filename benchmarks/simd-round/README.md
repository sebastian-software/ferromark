# SIMD round: baseline-v2 versus candidate-v2

This directory contains a bounded, reproducible measurement harness for the
v2 parser and renderer. It builds the same worker against two source snapshots
and measures the parser hot path before and after an optimization. It does not
modify the library or historical benchmark reports.

The current workers omit smart punctuation from the `extensions` profile after
its [removal from the core](../../docs/typography.md). Both compared cores use
that reduced profile. Historical reports retain the original configuration;
use their original harness revision to reproduce them.

The four stages are deliberately separate:

| Stage | Timed work |
| --- | --- |
| `fresh` | source-sized arena, parser, new renderer, owned HTML result |
| `reuse` | retained arena, parser, retained renderer, borrowed HTML result |
| `parse` | retained arena and parser only; checksum is top-level AST child count |
| `render` | documents parsed once before timing; retained renderer and borrowed HTML |

Every selected case is verified in every selected stage on the live worker
process. Baseline and candidate HTML bytes and AST `Debug` output must match
exactly. The same worker is verified before and after each batch of paired windows, and
the timing checksum is derived from the verified HTML byte count or child count.
Verification records the parser arena byte metric exposed by the allocator as
`arena_capacity_bytes`; it is diagnostic only and is not an allocation count.
Baseline and candidate capacities must match at the same lifecycle point.
Resetting an arena can discard smaller chunks after its first use, so a warm
capacity can legitimately differ from the initial capacity.

The worker uses Rust 1.95, generic target code generation, optimization level
3, fat LTO, one codegen unit, panic abort, and stripped release binaries. Each
source snapshot keeps its own lockfile; preparation builds offline and checks
that registry package versions and checksums were retained.

Build from two source directories (the baseline is expected to be
`ae963d425a4c15841ced5f7359a4b9adc47ca931`):

```sh
python3 benchmarks/simd-round/prepare.py BASELINE_SOURCE CANDIDATE_SOURCE BUILD_DIR
python3 benchmarks/simd-round/make_corpus.py /tmp/simd-corpus.json
python3 benchmarks/simd-round/run.py BUILD_DIR /tmp/simd-corpus.json RESULTS_DIR
```

The runner accepts `--rounds`, `--pairs`, `--window-ms`, `--modes`, and
`--filter`. A bounded smoke run is:

```sh
python3 benchmarks/simd-round/run.py BUILD_DIR CORPUS.json SMOKE_DIR \
  --rounds 1 --pairs 3 --window-ms 15
```

Results include `build.json`, frozen inputs, `verification.json`, raw
`samples.json`, `run.json`, and paired median summaries in both JSON and CSV.
The fixed seed shuffles jobs and alternates engine order within each pair.

`make_corpus.py` retains all 57 frozen broad Markdown cases and adds 15 labeled
synthetic diagnostics. Keep diagnostics separate from broad-corpus aggregates.
Existing source attribution and licenses remain in
[`broad-comparison`](../broad-comparison/README.md).

Run the admission and checksum guards with:

```sh
python3 -B -m unittest discover -s benchmarks/simd-round -p test_runner.py
```

The allocation check uses separate instrumented binaries. Run it separately
from timing runs so compilation and counting do not compete with measurements:

```sh
python3 benchmarks/simd-round/allocations.py BASELINE_SOURCE CANDIDATE_SOURCE \
  ALLOCATION_BUILD_DIR ALLOCATION_RESULTS_DIR --corpus /tmp/simd-corpus.json
```

It compares aggregate allocation/deallocation/reallocation counts, requested
sizes, output checksums, and arena capacity for all cases in fresh/reuse/parse
stages after one warmup. The complete worked example and archived variants are
in the [SIMD round report](../../docs/reports/2026-09-14-simd-round/README.md).
