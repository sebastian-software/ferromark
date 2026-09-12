# Native Goldmark and Sätteri comparisons

Compare native Markdown-to-HTML work in Goldmark and Sätteri
with independent Ferromark baselines. Persistent native workers expose small
JSON control messages; each worker's monotonic timer surrounds only fresh
parsing/rendering calls. There are no Node.js bindings, JavaScript plugins,
WASM runtimes, cgo calls, or per-document CLI launches.

This system-allocator experiment is separate from the published Bun/mimalloc
comparison. The README and homepage display it in a separate native overview.

## Pinned engines

| Engine | Pinned source | Native measured stages |
| --- | --- | --- |
| Goldmark | v2.0.2, `4dd635b1d163b39e2983acca140cc29d4bb994be` | `Parser.Parse` followed by `Renderer.Render`, fresh AST and `bytes.Buffer` |
| Sätteri | release `satteri-v0.10.5`, `b3d38e1e341c809b20b76a655e9b1601d11bd1f0` | `satteri_pulldown_cmark::parse` to MDAST arena, then `satteri_ast::mdast_to_html` |

The Sätteri release tag names the JavaScript package's release train; native
crate versions are `satteri` 0.2.13, `satteri-pulldown-cmark` 0.6.3,
`satteri-ast` 0.5.3, and `satteri-mdxjs` 0.3.13, with OXC 0.121.0. Cargo pins all
four crates to the same repository revision. Sätteri's fork is not replaced by
stock pulldown-cmark. The native build includes its default MDX capability;
Markdown timing disables MDX via runtime parsing options.

Sätteri's convenience `markdown_to_html` enables broad default syntax flags.
The adapter calls its same two public stages with the selected syntax flags,
retaining position tracking, fresh arena allocation, and the regular fused HTML
renderer and HAST fallback. An executable test compares those stages with the
convenience API under its default options. No cached AST, pooled arena, or
position-free shortcut is used.

## Build and check

Requirements: Python 3.11+, the repository's Rust toolchain with Clippy, Git, a C
toolchain for Sätteri's native stack dependency, and **Go 1.27.1**. The script
rejects a different Go version and records the actual Rust compiler. The Goldmark
module version and checksums are committed in `goldmark/go.mod` and `go.sum`;
Go automatic toolchain switching and cgo are disabled.

```sh
git clone --branch satteri-v0.10.5 https://github.com/bruits/satteri.git /tmp/satteri-0105
python3 benchmarks/native-pipeline-comparison/prepare.py /tmp/ferromark-pipeline-build \
  --satteri /tmp/satteri-0105
python3 -m unittest discover -s benchmarks/native-pipeline-comparison -p 'test_*.py'
cargo fmt --manifest-path benchmarks/native-pipeline-comparison/Cargo.toml --check
gofmt -l benchmarks/native-pipeline-comparison/goldmark
```

Use a new build directory. The source checkout must be clean and at the pinned
revision. `prepare.py` checks Ferromark's direct dependency versions/checksums
against the root lockfile, builds both workers, runs release Rust adapter tests
and Clippy, runs Go tests/vet/module verification, and records provenance. Rust
uses optimization level 3, fat LTO, one codegen unit, panic abort, and a generic
CPU target. Go uses its normal optimized build, baseline CPU features, and
explicitly disabled PGO. Complete flags and dependency trees are retained.

## Verify and measure

```sh
python3 benchmarks/native-pipeline-comparison/run.py /tmp/ferromark-pipeline-build \
  /tmp/ferromark-pipeline-verify --verify-only
python3 benchmarks/native-pipeline-comparison/run.py /tmp/ferromark-pipeline-build \
  /tmp/ferromark-pipeline-screen --screening
python3 benchmarks/native-pipeline-comparison/run.py /tmp/ferromark-pipeline-build \
  /tmp/ferromark-pipeline-measured \
  --case commonmark/5k \
  --case tables/tables-commonmark-inline \
  --case gfm_overlap/features
python3 benchmarks/native-pipeline-comparison/report.py /tmp/ferromark-pipeline-measured
```

Omit `--case` to measure every admitted workload. Unknown requested cases or
requested cases that fail workload admission stop timing. Screening uses one
short process run and is not publication evidence. Build and result directories
must be new. Run full measurements on AC power without concurrent builds,
tests, or other benchmarks.

Before timing, the runner captures all 18 workloads from the cmark harness,
all eight combinations of feature switches, single-tilde and URL-policy probes,
and all 652 stored CommonMark examples. It also saves native MDX outputs for
five stage-compatibility probes. Source hashes include fixture and shared review
code; changed local build inputs or worker executables invalidate the build.

Each competitor/Ferromark pair gets three fresh process runs. For each selected
case, each engine warms up for three seconds, then the coordinator alternates
80 windows of at least 63 ms per engine. Starting order rotates between samples
and process runs. Timer checks happen after 16 calls. Each worker renders into
a fresh owned output; Rust frees state and output before returning from the
call. The report takes the median of the three run medians and exposes their
range, input/output sizes, and every raw window's elapsed time and call count.

The Go process reuses parser/renderer configuration while allocating a new AST
and output per render. Normal automatic GC is enabled with `GOGC=100`,
`GOMAXPROCS=1`, and no explicit `GOMEMLIMIT`, `GODEBUG`, or forced collection.
Per-window GC cycles, allocated bytes, and ending heap size are recorded outside
the timer. GC work that happens during a window is timed, but Go may also do GC
between windows. This measures steady-state render-window latency, not total
process CPU or a forced collection after every document. IPC and stats reads
are outside timing for both languages.

## Feature and output contracts

| Lane | Selected features |
| --- | --- |
| CommonMark | All extensions off |
| Tables | Tables only |
| Strikethrough | Strikethrough only; measured fixtures use double tildes |
| Task lists | Task lists only |
| GFM overlap | Tables, strikethrough, and task lists |

All engines preserve raw HTML and arbitrary URL schemes. Bare autolinks, tag
filtering, footnotes, math, heading IDs/attributes, frontmatter, and typography
remain off. The overlap is not full GFM. Goldmark and Sätteri also recognize
single-tilde strikethrough; Ferromark does not. The native behavior is retained
and disclosed in probe outputs, not patched or silently presented as identical
syntax support.

Workload admission reuses [ARCH-COMP-002](../../docs/arch/ARCH-COMP-002-workload-comparability.md).
Sätteri adds `contains-task-list` on list containers. That specific class and
the already reviewed task-item classes are accepted renderer conventions, while
HTML fidelity remains separately reported. Unknown classes, lost content,
different list structure, missing checkboxes, and wrong checked states remain
significant. Neither normalization nor output inspection is timed.

## MDX stages

The diagnostic invokes Ferromark's `mdx::render_with_options(...).to_component`
and Sätteri's `compile_mdx` through native Rust APIs. It covers plain Markdown,
root JSX, inline expressions, JSX in a block quote, and invalid JavaScript.
It preserves each generated module or compiler error and names the output stage.

Ferromark emits a JSX module after segmentation; Sätteri emits JavaScript after
MDX/JavaScript parsing and compilation. The invalid-JavaScript test confirms
different validation contracts. Following [ADR-0009](../../docs/arch/ADR-0009-mdx-compatibility-and-performance-boundaries.md),
these outputs are diagnostics, with no timing ratio implying equal work. The
probe is not a complete MDX compatibility suite and does not execute components.

## Evidence

The current [measured report](../../docs/reports/2026-09-12-ox-corpus-optimizations/native-pipeline/REPORT.md)
contains the three targeted workloads above, with archived raw samples and all
verification/MDX diagnostics. It records an independent Ferromark baseline for
each competitor.

Each completed result contains build metadata, source and binary hashes, the
dependency locks, original HTML and effective options, capability/spec/MDX
diagnostics, per-pair admission decisions, raw samples, warmups, and summaries.
Only results with `finished_unix` completed final build validation. The report
generator recomputes all run medians, checks every window and saved summary,
and refuses screening or incomplete runs. It also reads gzip-compressed JSON
archives. Use `report.py RESULT --check` to verify a saved report.

## Additional engine adapters

`engines/*/engine.json` registers independent native adapters. Each engine keeps
its own source pin, build entry point, dependency lock and README in its directory;
adding a candidate does not alter the Goldmark/Sätteri build or archived reports.
List the available adapters with `python3 benchmarks/native-pipeline-comparison/prepare-engine.py --help`.

```sh
python3 benchmarks/native-pipeline-comparison/prepare-engine.py ENGINE /tmp/engine-build \
  --source /tmp/clean-pinned-upstream
python3 benchmarks/native-pipeline-comparison/run.py /tmp/engine-build /tmp/engine-verify --verify-only
python3 benchmarks/native-pipeline-comparison/run.py /tmp/engine-build /tmp/engine-screen --screening
python3 benchmarks/native-pipeline-comparison/run.py /tmp/engine-build /tmp/engine-measured \
  --case commonmark/5k --case tables/tables-commonmark-inline --case gfm_overlap/features
python3 benchmarks/native-pipeline-comparison/report.py /tmp/engine-measured
```

Each prepared build contains one competitor and a separately compiled Ferromark
worker with the root's direct dependency versions/checksums and system allocator.
The same output review, capability probes, monotonic windows, interleaving and
report validation apply. No upstream implementation is patched. MDX is outside
these Markdown adapters' measurement contract.

Adapters must expose the independently selectable syntax required by the
comparison lanes. Document limitations explicitly; do not patch upstream syntax
or substitute an older release to obtain convenient feature switches.

## Public overview

`publish.py` verifies the complete archived reports, raw samples and checksums
before generating `homepage/app/data/native-benchmarks.json`. The archive
retains each measured Ferromark baseline, workload coverage, specification
diagnostics and runtime disclosures. The public overview shows one Ferromark row
per document, using the median of the independently measured reference medians.
Candidate rows retain their own measured times; relative speeds use the single
overview reference. The homepage renders those generated values directly.

The five-parser publisher incorporates this overview into `README.md.src` while
keeping the two allocation environments separate. Run the native publisher,
`python3 benchmarks/bun-comparison/publish.py`, then `mise run readme:write`.
The homepage build checks both datasets against the archives and the generated
README. See [README themes](../../docs/readme-theme.md#benchmark-content).
