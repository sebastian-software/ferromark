# Native Ox Content comparison

Ox Content **3.2.0** is pinned to commit
`5c97078779cf099245a78aacca8fa7cc5e993316`, with a committed Cargo dependency
lock. The adapter uses the native allocator, parser and HTML renderer crates;
it does not use NAPI, a JavaScript plugin pipeline or the documentation generator.

Every call creates a fresh allocator arena, runs `Parser::parse`, then calls
`HtmlRenderer::render` and destroys the document/arena. `render` moves the owned
HTML string out of the renderer; output destruction is also timed. The normal
renderer configuration and scratch storage are reused, with per-document state
reset by the native API. A repeated-document test checks that heading counters
and prior content do not leak. `render_borrowed`, cached ASTs, pooled arenas and
incremental rendering are not used.

## Options and output boundaries

Tables, strikethrough and task lists are enabled independently on default parser
options. MDX, footnotes, math, attributes, typography and parser autolinks stay
off; the normal nesting limit remains 100. The HTML renderer's separate automatic
URL linking and external-link target rewriting are explicitly disabled. Raw
HTML and arbitrary URL schemes are preserved without tag filtering.

The renderer always generates heading IDs, even with heading attributes and
permalinks disabled. Those IDs are retained. The existing ARCH-COMP-002 projection
does not discard them, so affected workloads remain ineligible rather than being
silently normalized or rendered through a custom replacement hook. The measured
subset uses a short CommonMark input and realistic table/GFM-overlap documents
without headings. The three-extension overlap is not full GFM.

## Reproduce

```sh
git clone --branch v3.2.0 https://github.com/ubugeeei-prod/ox-content.git /tmp/ox-content
python3 benchmarks/native-pipeline-comparison/prepare-engine.py ox-content \
  /tmp/ox-content-build --source /tmp/ox-content
python3 benchmarks/native-pipeline-comparison/run.py /tmp/ox-content-build \
  /tmp/ox-content-verified --verify-only
python3 benchmarks/native-pipeline-comparison/run.py /tmp/ox-content-build \
  /tmp/ox-content-measured --case commonmark/short \
  --case tables/tables-commonmark-inline --case gfm_overlap/features
python3 benchmarks/native-pipeline-comparison/report.py /tmp/ox-content-measured
```

Use new build/result directories and a Rust toolchain meeting the pinned native
crates' requirements. Preparation runs locked release adapter tests, Clippy and
formatting, and records source/executable hashes, versions, flags and dependency
trees. Both workers use opt-level 3, fat LTO, one codegen unit, panic abort,
generic CPU features and their system allocator. The Ferromark baseline retains
its root direct dependency versions/checksums.

Verification captures all 18 workloads, independent feature/policy probes and
652 CommonMark examples before timing. Differences and exclusions remain visible.
The parent README documents three fresh process runs with warmup and alternating
native monotonic-clock windows. No MDX compilation or site-generation speed ratio
is inferred, and published Bun/mimalloc figures remain unchanged.

The [measured report](../../../../docs/reports/2026-09-11-native-ox-content/REPORT.md)
retains the complete verification, all eight exclusions and raw timing samples
for the three selected admitted workloads. The shared report command with
`--check` recomputes its summaries from that archive.
