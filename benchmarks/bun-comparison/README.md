# Direct Bun Markdown / Rust comparison

Compares **ferromark**, **Bun `bun_md`**, **pulldown-cmark 0.13.4**, and
**comrak 0.54.0**, and pinned **md4c (C)** through native Markdown-to-HTML functions. There is no
JavaScript engine, Node-API call, or installed Bun executable in the measured path.
See [PROVENANCE.md](PROVENANCE.md) for the documented md4c → Zig → Rust lineage.

The Bun commit is pinned to `76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`.
Its Rust parser, HTML renderer, internal crates, string-search kernels, and
data structures are compiled from unchanged upstream sources. This is **not
entirely Rust**: Bun calls its C++ Highway SIMD search routines and C mimalloc.

## Standalone integration

The build uses a dedicated Bun checkout. `prepare.py` changes only its workspace
membership, creates a separate comparison crate and configure-time metadata,
and lets Cargo resolve a separate lockfile. It does not patch parser algorithms.
The following native integrations replace the surrounding Bun executable:

- Original Bun mimalloc, pinned to `6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a`,
  is installed as the process-wide Rust allocator for all Rust parsers. md4c is compiled with a benchmark-only header mapping
  `malloc`/`calloc`/`realloc`/`free` to that same mimalloc; parser sources are unchanged.
- Original `highway_strings.cpp`, dispatch header, and substring fallback are
  built with Highway `2607d3b5b0113992fe84d3848859eae13b3b52c1`. A small forced
  header supplies release assertions and platform macros instead of pulling in
  the WebKit umbrella header. The original libc `memmem` alias is retained and
  is process-wide, so these are results in a shared Bun-native support environment.
- `stack.c` obtains and caches the actual lower pthread stack bound on macOS.
  This replaces two WebKit stack-bound accessors. Bun's recursion checks and
  reserve threshold remain unchanged; an uninitialized bound aborts.

The adapter currently supports **macOS only**. Rust is pinned to Bun's
`nightly-2026-07-20`; all Rust candidates use `target-cpu=generic`, release
optimization, fat LTO, one codegen unit, and aborting panics. Native code uses
`clang`/`clang++ -O3`, Highway runtime SIMD dispatch, and C++23. No PGO or
cross-language LTO is enabled. This is a direct native comparison, not an
estimate of the performance of the complete Bun binary.

## Run

Requires Python 3.11+, Git, Xcode Command Line Tools, and Rustup. From the
ferromark repository root, choose a **new, disposable** Bun checkout:

```sh
export BUN_BENCH_DIR=/private/tmp/ferromark-bun-source
export BUN_BENCH_WORK=/private/tmp/ferromark-bun-build
git clone --filter=blob:none --no-checkout https://github.com/oven-sh/bun.git "$BUN_BENCH_DIR"
git -C "$BUN_BENCH_DIR" sparse-checkout set src scripts/build patches
git -C "$BUN_BENCH_DIR" checkout --detach 76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1
rustup toolchain install nightly-2026-07-20 --profile minimal
git clone https://github.com/mity/md4c.git /private/tmp/ferromark-md4c
git -C /private/tmp/ferromark-md4c checkout --detach 65c6c9d72cebd9a731aaa5597414ce04d9ea5de3
python3 benchmarks/bun-comparison/prepare.py "$BUN_BENCH_DIR" "$BUN_BENCH_WORK" \
  --md4c /private/tmp/ferromark-md4c
python3 -m unittest discover -s benchmarks/bun-comparison -p 'test_*.py'
python3 benchmarks/bun-comparison/run.py "$BUN_BENCH_DIR" /private/tmp/ferromark-bun-results
```

`prepare.py` downloads the pinned native dependency archives and rebuilds them.
To reproduce a saved dependency set, pass `--lockfile /path/to/result/Cargo.lock`;
this uses Cargo's `--locked` mode. The initial resolution retains compatible
Bun lockfile versions for shared dependencies; Ferromark's `html-escape` is
explicitly pinned to its checked-in 0.2.14 version. Version 0.2.15 changes numeric
NUL entity decoding, so silently updating it would change the compared behavior.

## Measurement contract

Five configurations are compared: CommonMark; the shared GFM subset (tables,
strikethrough, task lists); and each of those three extensions separately. The
single-extension lanes use targeted inputs plus a neutral light-document control.
Raw HTML is preserved; heading IDs, tag filtering, bare autolinks, callouts, and
other extensions are disabled. This is trusted syntax overlap, not Ferromark's
secure-default product configuration. Effective parser options are retained.

Each timed call creates and drops fresh parser state and owned HTML output.
Options, input construction, output verification, and JSON serialization stay
outside timing. All five output buffers start empty. The allocator adapter does
not replace the individual parsers' own allocation strategies.

Publication uses 80 alternating-order windows of at least 63 ms per parser/case
(at least five seconds of sampling), after a three-second warmup. The clock is
checked after batches of 16 renders. The full eligible matrix runs once; cases
selected for the public tables run in two additional processes with rotated
parser order. The reported estimate is the median of those three run medians;
raw windows and run-to-run spread remain available. `--screening` uses short
windows for smoke checks and must never feed public tables. `--verify-only`
performs the output gate without timing. Run without other benchmarks or builds
competing for the CPU, and keep the machine on AC power.

The headline size cases are 2, 5, and 10 KiB of synthetic CommonMark with headings,
emphasis, links, lists, a quote, and fenced code. They are controlled examples,
not a claim about a population's typical document size. Tiny inputs diagnose
per-call overhead; 50 KiB and 1 MiB inputs remain long-document/stress cases.
Feature-set tables cover tables alone, strikethrough alone, their GFM combination,
links/images, and entities. None of these measurements is an additive feature price.

Before timing, all 652 stored CommonMark examples and every benchmark input
are rendered. Exact outputs and hashes are retained. A limited HTML tokenizer
also normalizes entity spelling, void-tag slashes, attribute order, boolean
attributes, equivalent table alignment attributes/styles, and ordinary flow
formatting at known HTML block boundaries. ASCII flow whitespace is collapsed,
while separators between inline words and non-breaking spaces are retained.
Literal text in pre/code/textarea/script/style is preserved. It preserves different alignments, URLs, code text,
and substantive text differences. This is a serialization check, not a full
browser-DOM equivalence test or the official CommonMark conformance harness.
HTML agreement is recorded separately from admission to timing. All five parsers
must perform comparable Markdown work with the requested options. The workload
review accepts table-alignment differences, task classes, checkbox spacing, and
checkbox placement inside or immediately before the first list-item paragraph.
Missing feature output, changed checkbox states, missing content, and unreviewed
differences remain ineligible. Parsers render their original HTML during timing;
all review transformations happen outside the timer. Spec mismatches are reported
without suppressing them or rewriting parser output. This assumes default HTML flow, not arbitrary
CSS whitespace rules. The [exclusion audit](../../docs/reports/2026-09-11-output-parity-audit.md)
groups agreeing parsers and distinguishes harmless serialization, renderer
conventions, resource limits, and parser bugs. `audit.py` reproduces its groups
from archived output without treating one parser as the correctness oracle.

A native `selftest` checks actual md4c task-list and wiki-link behavior before
verification. `render <configuration bits>` accepts Markdown on standard input
and emits all five HTML results plus Ferromark's resource-limit report. This
provides a small reproduction loop for output differences.

Use repeated `--case configuration/input` arguments for a targeted correction:
all cases are verified, but only the selected cases are measured, three times
each in publication mode. Selection never bypasses workload review. See the
[workload contract](../../docs/arch/ARCH-COMP-002-workload-comparability.md) for
accepted differences and the separate HTML-fidelity diagnostics.

Results include original output, excluded cases, spec diagnostics, compiler and
machine details, executable/source/fixture hashes, the effective Cargo lockfile,
and Ferromark's local source patch. Existing result directories are never
overwritten. Rebuild after source changes. The root package excludes this
harness, so ordinary builds and CI acquire no Bun dependency.

## Generate and check public tables

After a complete publication run, `publish.py` generates README tables and the
homepage data from the same raw samples. It rejects short screening protocols,
missing samples, edited summaries, and cases that fail workload review. The
original outputs and recorded md4c options are checked again rather than trusting
the saved pass flag. A sibling `publication-sources.json` may replace a complete
public case from a corrected run; it never replaces individual parser cells.
The September 11 correction replaces the GFM-subset case after fixing the md4c
task flag. Historical GFM/task-lane samples remain archived but must not be reused
as matched-option comparisons.

```bash
python3 benchmarks/bun-comparison/publish.py docs/reports/2026-09-11-benchmark-refresh/native
python3 benchmarks/bun-comparison/publish.py --check
python3 -m unittest discover -s benchmarks/bun-comparison -p 'test_*.py'
```

Raw JSONL files may be gzip-compressed after the run; the generator reads either
form. Normal CI checks the archived evidence without compiling or running Bun.

## Historical results

The [September 5 report](../../docs/reports/2026-09-05-bun-comparison.md) used four
Rust APIs and short exploratory windows before the later allocation optimizations.
It is retained as historical evidence and must not be mixed into this publication
run. The conventional stable/System-allocator Criterion suites remain separate
from this pinned nightly/shared-mimalloc comparison.
