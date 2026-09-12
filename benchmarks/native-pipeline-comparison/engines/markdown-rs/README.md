# Native markdown-rs comparison

The adapter pins the stable **1.0.0** release, commit
`1506572f9b406431402928f3a8b3df0b4ae2d8f5`. GitHub's release list also contains
an alpha entry, but that does not supersede the stable version. Cargo locks the
exact native source and transitive dependencies.

Each call uses `markdown::to_html_with_options` directly. Its HTML path parses
to events and compiles those events to owned HTML; the adapter does not build or
serialize an additional MDAST. Input processing, allocation and output destruction
are timed. Only input-independent configuration is reused.

CommonMark constructs stay enabled. Tables, strikethrough and tasks are selected
individually; `gfm_strikethrough_single_tilde=false` matches Ferromark. Both
`allow_dangerous_html` and `allow_dangerous_protocol` are true for the trusted
comparison policy. Bare autolinks, tag filtering, footnotes, math, frontmatter,
and MDX constructs stay off. The overlap is not called full GFM.

MDX support is relevant to candidate selection, but parsing JSX/expressions is
not the same task as emitting or executing a compiled JavaScript component.
Following ADR-0009, this adapter makes no MDX throughput claim.

## Reproduce

```sh
git clone --branch 1.0.0 https://github.com/wooorm/markdown-rs.git /tmp/markdown-rs
python3 benchmarks/native-pipeline-comparison/prepare-engine.py markdown-rs \
  /tmp/markdown-rs-build --source /tmp/markdown-rs
python3 benchmarks/native-pipeline-comparison/run.py /tmp/markdown-rs-build \
  /tmp/markdown-rs-verified --verify-only
python3 benchmarks/native-pipeline-comparison/run.py /tmp/markdown-rs-build \
  /tmp/markdown-rs-measured --case commonmark/5k \
  --case tables/tables-commonmark-inline --case gfm_overlap/features
python3 benchmarks/native-pipeline-comparison/report.py /tmp/markdown-rs-measured
```

Use new build/result directories. Preparation verifies the clean pinned source,
executes locked release tests, Clippy and formatting, and records compiler versions,
dependency trees and executable hashes. Both workers use opt-level 3, fat LTO,
one codegen unit, panic abort, generic CPU features and their system allocator.
Ferromark retains its root direct dependency versions/checksums.

All 18 workloads, all eight feature combinations, URL/dialect probes and 652
stored CommonMark examples are captured before timing. Workload admission follows
ARCH-COMP-002 separately from HTML fidelity. The parent README documents the
three fresh process runs, warmup and alternating native clock windows. No Node,
WASM or CLI startup is timed. Public Bun/mimalloc figures remain separate.

The [measured report](../../../../docs/reports/2026-09-12-ox-corpus-optimizations/markdown-rs/REPORT.md)
retains all native verification outputs and raw samples for three selected workloads.
The common report command with `--check` recomputes the archived summaries.
