# Native Rushdown comparison

The adapter pins Rushdown **0.18.0**, commit
`e5eb4e4446541ea0ed53111c1b37e779283ff57c`, with a committed Cargo lockfile.
This is the independent Rust engine, not another Goldmark version or a binding.

Each call runs `Parser::parse` into a fresh arena and `html::Renderer::render`
into a new `String`. Parser/renderer configuration is built outside timing;
reader state, AST allocation, rendering, arena destruction and output destruction
remain inside timing. An adapter test checks these stages against Rushdown's
public `new_markdown_to_html_string` pipeline. No AST/result cache is used.
The complete default `html-entities` feature stays enabled.

Tables, strikethrough and tasks are registered independently. Default CommonMark
parsers remain active, `allows_unsafe=true` matches trusted raw HTML and URL
policy, and bare autolinks, typography, attributes and heading IDs stay off.
Single-tilde behavior is retained in the dialect probe. The three-extension
overlap is not labeled full GFM.

## Reproduce

```sh
git clone --branch v0.18.0 https://github.com/yuin/rushdown.git /tmp/rushdown
python3 benchmarks/native-pipeline-comparison/prepare-engine.py rushdown \
  /tmp/rushdown-build --source /tmp/rushdown
python3 benchmarks/native-pipeline-comparison/run.py /tmp/rushdown-build \
  /tmp/rushdown-verified --verify-only
python3 benchmarks/native-pipeline-comparison/run.py /tmp/rushdown-build \
  /tmp/rushdown-measured --case commonmark/5k \
  --case tables/tables-commonmark-inline --case gfm_overlap/features
python3 benchmarks/native-pipeline-comparison/report.py /tmp/rushdown-measured
```

Use new build/result directories. The common preparer verifies source cleanliness
and hashes, runs locked release tests, Clippy and formatting, and records compiler
versions, dependency trees and executable hashes. Both Rust workers use opt-level
3, fat LTO, one codegen unit, panic abort, generic CPU features and their system
allocator. The baseline retains Ferromark's root dependency versions/checksums.

Before timing, all 18 workloads, all eight switch combinations, URL/dialect probes
and 652 stored CommonMark examples are captured. Admission uses ARCH-COMP-002,
separately from HTML fidelity. The parent README documents three fresh process
runs, warmup and alternating monotonic-clock windows. No Node/WASM or CLI startup
is included; these results do not replace the published Bun/mimalloc tables.

The [measured report](../../../../docs/reports/2026-09-12-ox-corpus-optimizations/rushdown/REPORT.md)
archives all verification outputs, raw samples and run medians for three selected
workloads. It is reproducible with the common report generator's `--check` mode.
