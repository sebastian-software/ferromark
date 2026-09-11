# Native Markdig comparison

Markdig **1.3.2** is pinned by exact NuGet version and package content hash in
`packages.lock.json`. The upstream release source is checked at
`fc705234fa211d179ee1d5e7656b51ab99f70ca9`. SDK **10.0.401** is pinned in
`global.json`; the build records `dotnet --info` and publishes a self-contained
worker whose complete runtime, assemblies and configuration are hashed.

This measures the direct .NET `Markdown.ToHtml` API in a persistent process.
It is a warmed JIT comparison, not AOT and not a Node binding. Each call performs
parsing and HTML rendering and returns a fresh string kept alive through the
call. Immutable pipelines are built outside timing. Native internal pooling,
if used by Markdig, is retained; the adapter does not cache rendered documents.

## Configuration and runtime

Only `UsePipeTables`, `UseEmphasisExtras(Strikethrough)` and `UseTaskLists` are
selected by the three independent switches. `UseAdvancedExtensions` is never
used. CommonMark remains the default; raw HTML and arbitrary URL schemes are
preserved. Typography, bare autolinks, heading IDs, footnotes and other extensions
stay off. The overlap lane is not full GFM.

Normal workstation GC, concurrent GC, tiered compilation and tiered PGO are
explicitly enabled. Inherited `DOTNET_*` and `COMPlus_*` variables are cleared
before worker launch; only the recorded runtime settings are applied. No forced
collection, no-GC region or heap limit is used. Each sampled window retains
collection-count deltas for generations 0–2, allocated bytes and ending heap size;
those counter reads and JSON serialization occur outside the timer.

GC work during a window is timed; concurrent GC may also progress between
windows. The result is steady-state render-window latency, not total process CPU
or a forced collection cost per document. Each engine/case warms for three
seconds in each of three fresh process runs before 80 alternating windows.
Startup and fixture decoding are excluded. Remaining tiering or GC variation is
visible in raw windows and run-median ranges.

## Reproduce

Install the official .NET SDK 10.0.401 and put `dotnet` on PATH. No global tool
installation is needed. Preparation uses the checked-in package lock and rejects
a different SDK.

```sh
git clone --branch 1.3.2 https://github.com/xoofx/markdig.git /tmp/markdig
python3 benchmarks/native-pipeline-comparison/prepare-engine.py markdig \
  /tmp/markdig-build --source /tmp/markdig
python3 benchmarks/native-pipeline-comparison/run.py /tmp/markdig-build \
  /tmp/markdig-verified --verify-only
python3 benchmarks/native-pipeline-comparison/run.py /tmp/markdig-build \
  /tmp/markdig-measured --case commonmark/5k \
  --case tables/tables-commonmark-inline --case gfm_overlap/features
python3 benchmarks/native-pipeline-comparison/report.py /tmp/markdig-measured
```

Use new build/result directories. The committed package lock records the measured
`osx-arm64` runtime identifier. Preparing a different platform requires a reviewed
lockfile update; preparation never regenerates the lock implicitly. First restore
needs access to NuGet. The independent Ferromark worker
uses the root's direct dependency versions/checksums, release optimization and
system allocator. All 18 workloads, the independent feature/policy probes and
652 stored CommonMark examples are captured before timing. Admission follows
ARCH-COMP-002, separately from HTML fidelity. The parent README documents the
shared protocol; these results do not replace the published Bun/mimalloc tables.

The [measured report](../../../../docs/reports/2026-09-11-native-markdig/REPORT.md)
retains the complete verification, runtime configuration, per-window GC counters
and raw samples for three selected workloads. Verify it with the shared report
command and `--check`.
