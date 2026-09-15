# Native engine comparison

Archived harness documentation. Shell examples below assume the repository root;
Markdown links have been adjusted for this archive location.

This harness measures UTF-8 Markdown to complete HTML for Ferromark v2, the
original OX-Content core, current local Ferromark v1 main, md4c, pulldown-cmark,
and Bun's original native Markdown engine. It uses the frozen 57-document
[broad corpus](../../../../benchmarks/broad-comparison/README.md), including the original source
attribution and licenses. It does not run Markdown through a JavaScript, WASM,
CLI, or network boundary in a timed operation.

## Native calls and lifecycles

| Engine | Fresh call | Reuse call |
| --- | --- | --- |
| Ferromark v2 | New arena, `Parser::with_options().parse()`, new `HtmlRenderer::render()` | Reset arena and retain renderer; `render_borrowed()` |
| Original OX-Content | Same original public arena/parser/renderer APIs | Same original arena reset and borrowed-output APIs |
| Ferromark v1 | `to_html_with_options()` | Retain `Renderer`, use `render_into()` and clear/reuse output |
| md4c | Original C `md_html()`, grow a fresh byte vector through its callback | Same C call, retain only the callback's output vector |
| pulldown-cmark | Stream `Parser::new_ext()` directly into `html::push_html()` | New parser/event stream each time, retain output string |
| Bun native `bun_md` | `render_to_html_with_options()`, owned byte box | Same call: the public HTML API does not expose retained parser/output state |

Both parsing and HTML rendering, output consumption, and destruction of fresh
state/output occur inside timing. Reuse measurements include resetting state.
Bun's reuse column is its fresh API under the reuse experiment schedule; it is
not a claim that Bun reuses output. There is no fake event collection or HTML
reimplementation for an engine. Engine dispatch happens before the generic
measurement loop. Each worker loads its input before accepting commands.

The standalone Bun integration is adapted from Ferromark v1's
`benchmarks/bun-comparison` (MIT OR Apache-2.0). Its original Rust parser,
renderer, internal crates, Highway search implementation, and mimalloc sources
are compiled directly. The macOS stack adapter supplies the actual cached
pthread stack boundary; recursion checks remain active. A forced header supplies
platform/release assertion macros in place of Bun's WebKit umbrella header.
The resulting executable is **not the full Bun runtime** and these timings do
not estimate the JavaScript-facing `Bun.markdown.html()` call.

All six engines share one native executable and Bun's mimalloc environment.
Rust uses Bun's global allocator; md4c's C allocation calls are redirected by
`md4c_alloc.h` to that same allocator. The parser and renderer algorithms remain
unchanged. This controls allocator differences, but is not a stock system-malloc
comparison. All Rust crates use the same pinned nightly compiler, generic CPU
baseline, release optimization, fat LTO, one codegen unit, and panic abort.
C/C++ use clang `-O3`; Highway retains runtime target dispatch. No PGO or
cross-language LTO is enabled. Current reproduction supports macOS.

The single Cargo workspace also uses a common pinned dependency resolution,
seeded from the existing Bun comparison lock. This differs from individual
engines' original lockfiles (for example, shared `memchr` is 2.8.0). The report
records these differences; this is a controlled native engine comparison,
not a comparison of untouched release build environments. Pass the archived
`Cargo.lock` with `prepare.py --lockfile` to replay that exact resolution.

## Syntax and output agreement

The corpus's 17 `commonmark` cases disable optional syntax. Its 40 `gfm` cases
use a **shared subset**: tables, strikethrough, and task lists. This is not full
GFM. Bare URL autolinks, tag filtering, footnotes, MDX, smart punctuation, and
other optional extensions are disabled. Raw HTML passes through; V1 uses its
trusted rendering policy. OX/v2's extra renderer URL autolinking and new-tab
link targets are disabled. This measures configured engine throughput, not
Ferromark's secure product defaults. Documents mentioning MDX remain ordinary
Markdown input; this is not an MDX feature benchmark.

V2 uses `HtmlRendererOptions::commonmark()` in both lanes, disabling heading
IDs, callouts, inline TOC substitution, and fence metadata cleanup. V1 also
disables its renderer extras. Original OX has no native flags to turn off those
four behaviors; it remains unchanged. The harness neither replaces its renderer
through hooks nor adds a slugifier to an engine that lacks one.
The [current flag contract](../../../../docs/reports/2026-09-15-native-arm/FLAGS.md)
lists each adapter's choices and executable guards. Every actual HTML output
is archived. `verify.py` groups
agreement without treating an engine as the correctness oracle and distinguishes:

- Exact bytes.
- Serialization equivalence: HTML flow whitespace, entity spelling, void-tag
  slashes, boolean attributes, equivalent table alignment spelling, UTF-8 URL
  spelling. Code whitespace, URLs/fragments, and content remain significant.
- Heading-ID-only differences, retained as semantic differences.
- Other differences, including task markup, classes, extension interpretations,
  and changes to code or content.

Only the first two categories enter the all-six strict-agreement subset.
The report also publishes a broader subset requiring agreement among the five
engines with matching renderer options (v2, v1, md4c, pulldown-cmark, and Bun).
OX gets no score in that subset. Subset membership is decided before timing;
neither IDs nor task CSS classes are removed to increase agreement. The current
run admits 14 of 57 inputs for all six engines and 50 for the configurable five.
All 57 cases are still timed and published as native-workload diagnostics,
including the mismatches. A throughput result does not establish conformance.
V1 resource-limit fallbacks cause verification to fail rather than becoming
fast, incomplete output. All engines must pass executable option guards.

## Measurement protocol

The default run uses three independent process rounds, six windows per round,
40 ms minimum per window, and 60 ms per-engine warmup for each document/lifecycle.
Each six-window rotation puts every engine in every order position; alternate
rounds reverse direction. Document/lifecycle order is deterministically shuffled.
Only one worker executes at a time. Input I/O, process startup, IPC, verification,
and JSON serialization stay outside timing. The inner loop checks time after
32 complete document cycles and black-boxes input and output. Every timed and
warmup window checks its output-length checksum.

Each engine must produce identical HTML across fresh/reuse modes, before/after
timing, and over repeated mixed-document cycles. Two additional rotating batches
cover each syntax profile. An operation in these batches means a complete batch,
not one document; they are reported separately from per-document averages.

For each engine/document/lifecycle, take each process round's median and then the
median of those three medians. Group comparisons use the geometric mean of
per-document time ratios with equal document weight. Size groups and categories
are kept separate; overlapping Wikipedia views are not independent samples.
This is one-machine steady-state evidence, not a universal ranking or a
statistical significance claim. Raw windows and process-round ranges are retained.

## Reproduction

`prepare.py --help` lists source/cache inputs. Preparation exports pinned Git
commits rather than using dirty working trees, re-extracts the checksummed
original OX archive, builds native support, and records source, adapter, lock,
and executable hashes. It uses a disposable directory and does not change the
supplied source checkouts. Dependency choices and any lockfile differences are
recorded with the build.

If the temporary caches are missing, copy the current report's `restore.py` to
an empty `/private/tmp/native-bench-cache` directory and run it there. It restores
the original source pins and checks the original archive hashes. Adjust source
paths below for your checkout layout.

```sh
python3 benchmarks/native-comparison/prepare.py /private/tmp/native-bench-build \
  --bun-source /private/tmp/native-bench-cache/bun \
  --bun-native-cache /private/tmp/native-bench-cache/native \
  --md4c-source /private/tmp/native-bench-cache/md4c \
  --ox-archive /private/tmp/native-bench-cache/ox.tar.gz \
  --ferromark-v1-source ../ferromark --ferromark-v2-source . \
  --worker benchmarks/native-comparison/worker.rs --compile \
  --lockfile docs/reports/2026-09-15-native-arm/Cargo.lock
python3 -m unittest discover -s benchmarks/native-comparison -p 'test_*.py'
python3 benchmarks/native-comparison/run.py \
  /private/tmp/native-bench-build/build.json \
  docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz \
  /private/tmp/native-bench-results
python3 benchmarks/native-comparison/report.py /private/tmp/native-bench-results
```

Use `run.py --verify-only` for behavior/output checks without timing. Build and
output directories must be new so previous evidence is never overwritten.
The current source pins are v2 `e93394e` and v1 `4e15141`; other engine pins are
unchanged. The
[previous matched-flags comparison](../../../../docs/reports/2026-09-14-native-matched/README.md)
and original pre-correction comparison are preserved separately in
[`2026-09-14-native-engines`](../../../../docs/reports/2026-09-14-native-engines/README.md).
