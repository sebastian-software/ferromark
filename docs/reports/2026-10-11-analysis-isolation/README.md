# Prepared analysis performance isolation

This report checks the ordinary paths affected by the reference inventory,
text extraction, and Markdown projection stack. Extraction cost on a prepared
tree is a separate question; the earlier [text extraction report](../2026-10-11-text-extraction/README.md)
does not establish that ordinary parsing or preparation is unaffected.

## Execution contract

- Rust parsing and HTML rendering perform no reference inventory, source alignment,
  text extraction, or Markdown projection. Public AST layouts are unchanged.
- Bare autolink source alignment runs only during `collect_references` or explicit
  `ReferenceSnapshot::capture`. The GFM autolink implementation matches the baseline.
- Node `trackReferences` defaults to `false`. A snapshot is captured only when the
  setting is `true` and a nonempty native pass pipeline runs. Untracked transformed
  handles reject reference collection instead of guessing source provenance.
- Text extraction and Markdown projection do not require reference tracking, even
  after native passes. Their output arrays and strings are allocated when called.
- Prepared Node handles share analysis methods on their prototype. Existing `render`
  and `renderModule` calls remain bound and usable after destructuring.

## Rejected implementations

The first implementation did extra work during ordinary use. The retained initial
screen measured bare links after long prose prefixes at 15.70 to 82.86 microseconds
(about +428%), dense GFM autolink parsing at 135.41 to 158.33 microseconds (+16.93%)
and entity/escape-heavy autolink parsing at 110.65 to 150.23 microseconds (+35.77%).
Node preparation with native passes increased from 331.21 to 365.32 microseconds
(+10.30%), and preparation of a 62-byte document from 6.09 to 6.47 microseconds
(+6.32%). These measurements rejected automatic provenance capture and per-handle
analysis closures.

A pointer fast path and a bounded entity scanner fixed most ordinary autolink work,
but the second screen still measured entity/escape-heavy parsing at 110.36 to 127.75
microseconds (+15.76%). That result rejected source alignment inside the parser.
The final implementation moves it entirely into explicit reference analysis.
Raw estimates, samples, and logs for both screens are under [rejected/](rejected/).

## Final measurements

Positive percentages mean slower. The complete stack source revision measured here is
`e283bae2b6c1b465ea6c97dea17af77aa2665179`; later changes retain this report only.

| Rust fixture / operation | Before (µs) | After (µs) | Change |
| --- | ---: | ---: | ---: |
| gfm_bare / compile_html | 21.134 | 21.020 | -0.54% |
| gfm_bare / parse | 15.327 | 15.098 | -1.49% |
| gfm_bare / render_html | 5.554 | 5.558 | +0.06% |
| gfm_decoded / compile_html | 138.750 | 137.954 | -0.57% |
| gfm_decoded / parse | 108.334 | 108.196 | -0.13% |
| gfm_decoded / render_html | 30.478 | 30.103 | -1.23% |
| gfm_dense / compile_html | 160.784 | 161.423 | +0.40% |
| gfm_dense / parse | 130.593 | 130.688 | +0.07% |
| gfm_dense / render_html | 29.491 | 30.231 | +2.51% |
| mdx / compile_jsx | 993.263 | 981.447 | -1.19% |
| mdx / render_jsx | 589.379 | 588.369 | -0.17% |
| prose / compile_html | 122.666 | 123.889 | +1.00% |
| prose / parse | 63.215 | 63.339 | +0.20% |
| prose / render_html | 59.468 | 62.002 | +4.26% |

The initial final run showed +4.26% for prose render-only and +2.51% for dense
autolink render-only. A longer confirmation used 40 samples, 1 second warmup,
and 5 seconds measurement per region, without changing code:

| Rust render-only confirmation | Before (µs) | After (µs) | Change |
| --- | ---: | ---: | ---: |
| prose | 59.495 | 59.384 | -0.19% |
| gfm_dense | 29.533 | 29.819 | +0.97% |

The larger render deviations did not persist. Remaining small differences,
including positive sub-percent Node ratios, are reported rather than rounded
to a zero-cost claim. No additional analysis work runs in the ordinary paths.

| Node operation | Before median (µs) | After median (µs) | Paired change | Paired bootstrap interval |
| --- | ---: | ---: | ---: | ---: |
| compile_html | 107.356 | 107.827 | +0.59% | +0.39% to +1.21% |
| compile_jsx | 964.699 | 968.140 | +0.33% | +0.21% to +0.52% |
| prepare_default | 91.053 | 90.259 | -1.31% | -1.85% to -0.48% |
| prepare_with_passes | 299.855 | 298.267 | -0.38% | -0.61% to -0.22% |
| prepare_small | 5.509 | 4.132 | -23.49% | -25.35% to -21.95% |
| render_default | 900.178 | 903.055 | +0.41% | +0.02% to +0.58% |
| render_with_passes | 1528.452 | 1544.329 | +0.92% | -1.13% to +2.10% |
| render_small | 7.021 | 7.032 | -0.39% | -0.55% to +0.11% |

Raw final data is under [final/](final/), the longer repetitions under
[confirmation/](confirmation/), and machine-readable estimates in
[summary.json](summary.json). The summaries can be regenerated with:

```sh
python3 docs/reports/2026-10-11-analysis-isolation/harness/summarize.py \
  docs/reports/2026-10-11-analysis-isolation/final
```

## Method and scope

The baseline is repository commit `2df7d368`, before all three APIs. Both versions
are built and run on one Apple M1 Ultra host with 64 GiB RAM, macOS 27.0.1,
Rust 1.95.0 / LLVM 22.1.2, and Node 24.21.0. PGO is disabled. Heavy compilation and
test jobs finish before timing; no other task benchmark runs concurrently.

Rust uses two path dependencies in one Criterion 0.8.2 executable with the same
retained dependency lock, fat LTO, and one codegen unit. Only the baseline package
version is renamed to allow both source versions in one dependency graph; its Rust
source is unchanged. This harness lock is separate from the repository lock.
Each region uses 20 samples, 0.5 seconds of warmup, and 2 seconds of measurement.
HTML and JSX output equality is asserted before measurement. Parsing, combined
parse/render, and rendering an existing document are measured separately.

Node uses independently built `release-node` addons and the corresponding facades.
Compiler creation, an inline offline theme, preparation for render-only cases, and
output equality checks occur before timing. Each operation warms up for 200 ms per
version and records 16 pairs of 100 ms samples, alternating the version order.
Explicit GC occurs outside each timed sample. The summary reports each version's
median and the median paired ratio, with a deterministic 10,000-resample paired
bootstrap interval. Fixture byte lengths and runtime versions are retained with
the samples. Rust summaries use Criterion's slope estimates and their individual
confidence intervals; percentages compare the paired executable's two versions,
not Criterion's comparison against an earlier run of the same benchmark name.

These are measurements of the retained fixtures on one host, not a guarantee for
every input or machine. They exclude module startup, highlighter construction,
code highlighting, and the cost of actually requesting analysis. Small timing
differences require confirmation; unchanged source paths provide additional evidence
that analysis work is absent from default parsing and rendering.

## Reproduction

The standalone harness does not modify the repository or baseline checkout.
Use Python 3.12 or newer and choose a destination that does not exist:

```sh
python3 docs/reports/2026-10-11-analysis-isolation/harness/prepare.py \
  --repository . --destination /tmp/ferromark-analysis-comparison
cd /tmp/ferromark-analysis-comparison
cargo bench --bench ordinary_paths --locked -- \
  --sample-size 20 --warm-up-time 0.5 --measurement-time 2
```

Build both Node checkouts with the repository's native build script and PGO disabled.
Pass their facade paths to the retained harness; no network assets are used:

```sh
node --expose-gc docs/reports/2026-10-11-analysis-isolation/harness/node-paths.mjs \
  /path/to/baseline/node/ferromark/index.mjs \
  /path/to/candidate/node/ferromark/index.mjs > node.json
```
