# Native cmark comparison

Compare Ferromark with the C CommonMark reference implementation and GitHub's
GFM fork through their native parsing and HTML-rendering APIs. There are no
Node.js bindings, JavaScript runtimes, WASM engines, subprocess-per-document
costs, or cached ASTs in the measured path.

Each executable links Ferromark and **one** cmark library. cmark and cmark-gfm
export overlapping C symbols, so they must not be linked into the same process.
Every comparison includes a fresh Ferromark measurement in the same process;
do not combine the two independent Ferromark baselines into one ranking.

This is a separate **system-allocator** experiment, not an extension of the
published Bun/mimalloc environment. Its results do not feed the README or
homepage publication generator. Existing published measurements stay unchanged.

## Build

Requirements: Python 3.11+, Git, CMake 3.x, a C/C++ compiler, and the repository's
Rust toolchain with Clippy. The build records the actual toolchain and compiler
configuration; reproduce those versions when replaying a measurement.

Clone the two release revisions pinned in `support.py` into clean, dedicated
checkouts. Upstream sources are never patched. Build and result directories must
be new; a rebuild uses a fresh directory.

```sh
git clone --branch 0.31.1 https://github.com/commonmark/cmark.git /tmp/cmark-0311
git clone --branch 0.29.0.gfm.13 https://github.com/github/cmark-gfm.git /tmp/cmark-gfm-13
python3 benchmarks/cmark-comparison/prepare.py /tmp/ferromark-cmark-build \
  --cmark /tmp/cmark-0311 --cmark-gfm /tmp/cmark-gfm-13
```

Use `--cmake /path/to/cmake` if CMake is not on PATH. The script builds static
libraries with upstream CMake, runs the native adapter tests in both builds,
and runs Clippy. It requires the committed Cargo.lock (`--locked`). cmark 0.31.1
is pinned at `bb3678d7a73cb02d35c8876ecd097072636200a8`; cmark-gfm 0.29.0.gfm.13
is pinned at `587a12bb54d95ac37241377e6ddc93ea0e45439b`.

## Verify and measure

```sh
python3 -m unittest discover -s benchmarks/cmark-comparison -p 'test_*.py'
python3 benchmarks/cmark-comparison/run.py /tmp/ferromark-cmark-build \
  /tmp/ferromark-cmark-verify --verify-only
python3 benchmarks/cmark-comparison/run.py /tmp/ferromark-cmark-build \
  /tmp/ferromark-cmark-screen --screening
python3 benchmarks/cmark-comparison/run.py /tmp/ferromark-cmark-build \
  /tmp/ferromark-cmark-measured
```

`--case commonmark/5k` selects a case and may be repeated. A requested case that
fails workload review stops measurement. Unsupported extension cases are
explicitly unavailable for core cmark, never silently measured as plain text.
All cases and all 652 stored CommonMark examples are verified before timing,
including cases not selected for a targeted run. Spec diagnostics are retained
separately from workload admission.

The default protocol uses three independent processes per pair, with 80
alternating-order sample windows of at least 63 ms after three seconds of warmup
per parser/case. The timer is checked after batches of 16 calls. The summary is
the median of the three run medians, retaining raw samples, run medians, input
size, and output size. `--screening` uses one short run for validation only; its
numbers must not be presented as publication-quality speed claims. Run on AC
power without concurrent builds, tests, or benchmarks.

## Workload and feature contract

| Lane | cmark | cmark-gfm | Ferromark configuration |
| --- | --- | --- | --- |
| CommonMark | Yes | Extensions off | `Options::commonmark()` + trusted rendering |
| Tables | Unavailable | `table` only | `tables` only |
| Strikethrough | Unavailable | `strikethrough` only | `strikethrough` only; double-tilde fixtures |
| Task lists | Unavailable | `tasklist` only | `task_lists` only |
| GFM overlap | Unavailable | The three above | The three above |

Raw HTML and arbitrary URL schemes are preserved. Bare autolinks, GFM tag
filtering, heading IDs, footnotes, smart punctuation, and other extensions remain
off. The overlap lane is **not full GFM**. The cmark-gfm registry is initialized
before timing; attaching selected descriptors to a fresh parser is timed.
Strikethrough enables `CMARK_OPT_STRIKETHROUGH_DOUBLE_TILDE` to match Ferromark;
an executable negative test ensures `~single~` remains literal. The actual
native option bits are retained in each verification row.

Inputs include short prose, 2/5/10/50-KiB core Markdown, isolated extension
fixtures, neutral extension controls, and the exact `tables-plain`,
`tables-links`, and `tables-commonmark-inline` fixtures added in PR #295. The
same table inputs run with tables alone and with all three overlap extensions.

The shared [workload review](../bun-comparison/workload.py) implements
[ARCH-COMP-002](../../docs/arch/ARCH-COMP-002-workload-comparability.md): original
HTML is retained, serialization differences and reviewed table/task renderer
conventions are disclosed, missing content or unsupported syntax is excluded.
Neither normalization nor JSON serialization is timed.

Every timed iteration includes parsing and rendering into a newly owned output,
then destruction of parser state, AST, and output. cmark's allocated HTML is
freed with its own allocator, **without copying it into a Rust String** or
scanning it for length in the timed loop. Ferromark uses its owned String API.
Output inspection happens outside timing. Both use their default system
allocators; upstream allocation strategies are preserved.

## Evidence

The first [measured report](../../docs/reports/2026-09-11-native-cmark-comparison.md)
contains targeted CommonMark and realistic-table results with archived raw data.

`build-info.json` records release commits, hashes of all tracked upstream files,
local code/adapter/lockfile hashes, commands, compiler settings, and executable
hashes. The runner rejects changed local build inputs or executables. Each
result captures the generated corpus, effective options, raw HTML, spec
diagnostics, excluded/unavailable cases, build information, CMake compile
commands, lockfile, Ferromark source patch, protocol, and raw samples. A missing
`finished_unix` means the run did not finish successfully.

The next adapters and MDX boundaries are described in the
[native comparison expansion plan](../../docs/plans/2026-09-11-native-comparison-expansion.md).
