# Direct Bun Markdown / Rust comparison

Compares **ferromark**, **Bun `bun_md`**, **pulldown-cmark 0.13.4**, and
**comrak 0.54.0** through their native Markdown-to-HTML functions. There is no
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
  is installed as the process-wide Rust allocator for **all four** parsers.
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
python3 benchmarks/bun-comparison/prepare.py "$BUN_BENCH_DIR" "$BUN_BENCH_WORK"
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

Two configurations are compared: CommonMark, and CommonMark plus GFM tables,
strikethrough, and task lists. Raw HTML is preserved, heading IDs, tag filtering,
bare autolinks, callouts, and other extensions are disabled. This is trusted
syntax overlap, not ferromark's secure-default product configuration.

Each timed call creates and drops a fresh owned HTML output. No parser state or
output capacity is reused across documents. Options, input construction, output
verification, and JSON serialization are outside the timer. Warmup is 50 ms per
parser/case; five 150 ms windows alternate parser order, with 16 renders per
clock check. Raw window means and their median are retained. These short local
measurements are exploratory, not confidence intervals or publication-grade
claims; they must not replace the existing README headline benchmarks.

Before timing, all 652 stored CommonMark examples and every benchmark input
are rendered. Exact outputs and hashes are retained. A limited HTML tokenizer
also normalizes entity spelling, void-tag slashes, attribute order, boolean
attributes, equivalent table alignment attributes/styles, and newline-only
formatting outside code. It preserves different alignments, URLs, code text,
and substantive text differences. This is a serialization check, not a full
browser-DOM equivalence test or the official CommonMark conformance harness.
Only benchmark cases matching across **all four** parsers after this check
enter the timing allowlist. Spec mismatches are reported without suppressing
them or rewriting parser output.

Results include original output, excluded cases, spec diagnostics, compiler and
machine details, executable/source/fixture hashes, the effective Cargo lockfile,
and Ferromark's local source patch. Existing result directories are never
overwritten. Rebuild after source changes. The root package excludes this
harness, so ordinary builds and CI acquire no Bun dependency.
