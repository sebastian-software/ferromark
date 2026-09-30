# Completed ecosystem comparisons — 2026-09-30

## Completed native and Linux comparisons

These independently measured pairs fill the remaining homepage cells. The
Ferromark core source hashes match the earlier clean-core ecosystem runs.
**Factors mean Ferromark throughput relative to the library.** Each row uses
its own equivalent-HTML set; rows do not establish a shared-set ranking.

| Platform | Project | Equivalent documents | Fresh | Reuse |
| --- | --- | ---: | ---: | ---: |
| macOS arm64 | Comrak | 57/57 | 7.83× | 8.60× |
| macOS arm64 | cmark | 57/57 | 7.01× | 7.84× |
| macOS arm64 | cmark-gfm | 57/57 | 7.08× | 7.85× |
| Linux x86-64 | Comrak | 57/57 | 5.05× | 5.55× |
| Linux x86-64 | cmark | 57/57 | 5.60× | 6.18× |
| Linux x86-64 | cmark-gfm | 57/57 | 5.82× | 6.38× |
| Linux x86-64 | markdown-rs | 57/57 | 99.64× | 109.52× |
| Linux x86-64 | micromark | 56/57 | 105.10× | 152.72× |
| Linux x86-64 | marked | 56/57 | 10.45× | 15.25× |
| Linux x86-64 | markdown-it | 56/57 | 11.11× | 15.87× |
| Linux x86-64 | remark / unified | 56/57 | 123.65× | 176.62× |
| Linux x86-64 | Showdown | 31/57 | 34.11× | 55.95× |
| Linux x86-64 | commonmark.js | 57/57 | 4.95× | 7.28× |

cmark 0.31.2 uses CommonMark only in **both** engines for every document, as
commonmark.js does. Comrak 0.55.0 and cmark-gfm 0.29.0.gfm.13 enable only tables,
strikethrough, and task lists on the extension inputs. All use their public
Markdown-to-HTML APIs and preserve raw HTML. The C adapters call native library
functions directly; no CLI startup or IPC is timed. cmark's convenience call
and cmark-gfm's parser/feed/finish/render sequence own fresh parser/AST/output
state in both schedules. C HTML consumption includes `strlen` before freeing
the returned allocation. Comrak uses `markdown_to_html`, also with fresh owned
AST/output. Reuse retains configuration in these competitors; Ferromark retains
its arena and renderer. AST and output destruction remain within native timing.

The native pairs use system malloc, Rust 1.95.0, generic CPU, optimization level
3, fat LTO, one codegen unit, and panic abort. C libraries/adapters use clang
`-O3`, system malloc, and no PGO; no allocator redirection or cross-language LTO.
The Node pairs use Node 24.21.0 and the release-node addon without PGO, including
public binding/string conversion costs and JavaScript GC.

Every pair has three process rounds, six rotating 40 ms windows and 60 ms warmup,
with verification before timing and around each round. All 57 inputs, exact
outputs, option guards, 2,052 paired windows, locks, source hashes, and committed
adapter snapshots are retained. Only exact or conservative serialization-
equivalent HTML contributes to factors; code whitespace, attributes, task-list
markup, and text differences remain significant.

macOS runs use a shared Apple M1 Ultra workstation. Linux pairs each run on their
own GitHub-hosted Ubuntu 24.04 x86-64 VM; the pair alternates on one VM, but the
CPU/host can differ between pairs. Metadata records CPU, load, clocks, thermal
observations and hypervisor steal counters. No other workflow step runs during
timing. These are separate steady-state measurements, with no significance
claim or comparison of absolute durations between hosts.

[Raw evidence and per-pair host metadata](https://github.com/sebastian-software/ferromark/tree/main/docs/reports/2026-09-30-ecosystem-platforms),
[reproduction](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md).

## Evidence

- macOS arm64 / Comrak: [metadata](macos-arm64/native-comrak/run.json), [aggregate](macos-arm64/native-comrak/summary.json), [outputs](macos-arm64/native-comrak/verification.json.gz), [windows](macos-arm64/native-comrak/samples.json.gz), [guards](macos-arm64/native-comrak/behavior.json.gz), [corpus](macos-arm64/native-comrak/corpus.json.gz), [adapters](macos-arm64/native-comrak/adapters/).
- macOS arm64 / cmark: [metadata](macos-arm64/native-cmark/run.json), [aggregate](macos-arm64/native-cmark/summary.json), [outputs](macos-arm64/native-cmark/verification.json.gz), [windows](macos-arm64/native-cmark/samples.json.gz), [guards](macos-arm64/native-cmark/behavior.json.gz), [corpus](macos-arm64/native-cmark/corpus.json.gz), [adapters](macos-arm64/native-cmark/adapters/).
- macOS arm64 / cmark-gfm: [metadata](macos-arm64/native-cmark-gfm/run.json), [aggregate](macos-arm64/native-cmark-gfm/summary.json), [outputs](macos-arm64/native-cmark-gfm/verification.json.gz), [windows](macos-arm64/native-cmark-gfm/samples.json.gz), [guards](macos-arm64/native-cmark-gfm/behavior.json.gz), [corpus](macos-arm64/native-cmark-gfm/corpus.json.gz), [adapters](macos-arm64/native-cmark-gfm/adapters/).
- Linux x86-64 / Comrak: [metadata](linux-x86-64/native-comrak/run.json), [aggregate](linux-x86-64/native-comrak/summary.json), [outputs](linux-x86-64/native-comrak/verification.json.gz), [windows](linux-x86-64/native-comrak/samples.json.gz), [guards](linux-x86-64/native-comrak/behavior.json.gz), [corpus](linux-x86-64/native-comrak/corpus.json.gz), [adapters](linux-x86-64/native-comrak/adapters/).
- Linux x86-64 / cmark: [metadata](linux-x86-64/native-cmark/run.json), [aggregate](linux-x86-64/native-cmark/summary.json), [outputs](linux-x86-64/native-cmark/verification.json.gz), [windows](linux-x86-64/native-cmark/samples.json.gz), [guards](linux-x86-64/native-cmark/behavior.json.gz), [corpus](linux-x86-64/native-cmark/corpus.json.gz), [adapters](linux-x86-64/native-cmark/adapters/).
- Linux x86-64 / cmark-gfm: [metadata](linux-x86-64/native-cmark-gfm/run.json), [aggregate](linux-x86-64/native-cmark-gfm/summary.json), [outputs](linux-x86-64/native-cmark-gfm/verification.json.gz), [windows](linux-x86-64/native-cmark-gfm/samples.json.gz), [guards](linux-x86-64/native-cmark-gfm/behavior.json.gz), [corpus](linux-x86-64/native-cmark-gfm/corpus.json.gz), [adapters](linux-x86-64/native-cmark-gfm/adapters/).
- Linux x86-64 / markdown-rs: [metadata](linux-x86-64/native-markdown-rs/run.json), [aggregate](linux-x86-64/native-markdown-rs/summary.json), [outputs](linux-x86-64/native-markdown-rs/verification.json.gz), [windows](linux-x86-64/native-markdown-rs/samples.json.gz), [guards](linux-x86-64/native-markdown-rs/behavior.json.gz), [corpus](linux-x86-64/native-markdown-rs/corpus.json.gz), [adapters](linux-x86-64/native-markdown-rs/adapters/).
- Linux x86-64 / micromark: [metadata](linux-x86-64/node-micromark/run.json), [aggregate](linux-x86-64/node-micromark/summary.json), [outputs](linux-x86-64/node-micromark/verification.json.gz), [windows](linux-x86-64/node-micromark/samples.json.gz), [guards](linux-x86-64/node-micromark/behavior.json.gz), [corpus](linux-x86-64/node-micromark/corpus.json.gz), [adapters](linux-x86-64/node-micromark/adapters/).
- Linux x86-64 / marked: [metadata](linux-x86-64/node-marked/run.json), [aggregate](linux-x86-64/node-marked/summary.json), [outputs](linux-x86-64/node-marked/verification.json.gz), [windows](linux-x86-64/node-marked/samples.json.gz), [guards](linux-x86-64/node-marked/behavior.json.gz), [corpus](linux-x86-64/node-marked/corpus.json.gz), [adapters](linux-x86-64/node-marked/adapters/).
- Linux x86-64 / markdown-it: [metadata](linux-x86-64/node-markdown-it/run.json), [aggregate](linux-x86-64/node-markdown-it/summary.json), [outputs](linux-x86-64/node-markdown-it/verification.json.gz), [windows](linux-x86-64/node-markdown-it/samples.json.gz), [guards](linux-x86-64/node-markdown-it/behavior.json.gz), [corpus](linux-x86-64/node-markdown-it/corpus.json.gz), [adapters](linux-x86-64/node-markdown-it/adapters/).
- Linux x86-64 / remark / unified: [metadata](linux-x86-64/node-remark/run.json), [aggregate](linux-x86-64/node-remark/summary.json), [outputs](linux-x86-64/node-remark/verification.json.gz), [windows](linux-x86-64/node-remark/samples.json.gz), [guards](linux-x86-64/node-remark/behavior.json.gz), [corpus](linux-x86-64/node-remark/corpus.json.gz), [adapters](linux-x86-64/node-remark/adapters/).
- Linux x86-64 / Showdown: [metadata](linux-x86-64/node-showdown/run.json), [aggregate](linux-x86-64/node-showdown/summary.json), [outputs](linux-x86-64/node-showdown/verification.json.gz), [windows](linux-x86-64/node-showdown/samples.json.gz), [guards](linux-x86-64/node-showdown/behavior.json.gz), [corpus](linux-x86-64/node-showdown/corpus.json.gz), [adapters](linux-x86-64/node-showdown/adapters/).
- Linux x86-64 / commonmark.js: [metadata](linux-x86-64/node-commonmark/run.json), [aggregate](linux-x86-64/node-commonmark/summary.json), [outputs](linux-x86-64/node-commonmark/verification.json.gz), [windows](linux-x86-64/node-commonmark/samples.json.gz), [guards](linux-x86-64/node-commonmark/behavior.json.gz), [corpus](linux-x86-64/node-commonmark/corpus.json.gz), [adapters](linux-x86-64/node-commonmark/adapters/).

Linux source: [successful workflow](https://github.com/sebastian-software/ferromark/actions/runs/36702822137); [origin metadata](origin.json). The macOS hardware identifier is retained in [macos-host.txt](macos-host.txt). [SHA256SUMS](SHA256SUMS) covers the complete report archive.

Input attribution remains in the [frozen broad corpus](../../../benchmarks/broad-comparison/README.md). Original measurements are preserved separately. Regenerate with `python3 benchmarks/markdown-ecosystem/publish_values.py`.
