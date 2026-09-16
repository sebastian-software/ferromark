# Native engine comparison

This harness measures UTF-8 Markdown to complete HTML for Ferromark v2, the
original OX-Content core, current local Ferromark v1 main, md4c, pulldown-cmark,
and Bun's original native Markdown engine. It uses the frozen 57-document
[broad corpus](../broad-comparison/README.md), including the original source
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
C/C++ use clang `-O3`; Highway retains runtime target dispatch. The default
build enables no PGO and no cross-language LTO; `prepare.py --pgo` adds the
separate, clearly labeled build described in
[Profile-guided optimization](#profile-guided-optimization) below. Current
reproduction supports macOS.

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
The [current flag contract](../../docs/reports/2026-09-15-native-arm/FLAGS.md)
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

## Profile-guided optimization

Ferromark's published native binaries are moving to profile-guided
optimization, so the harness can build the same comparison executable with PGO
instead of comparing a PGO Ferromark against non-PGO competitors. `prepare.py
--pgo` keeps every other build setting identical to the default build — the
same pinned nightly toolchain, generic CPU baseline, optimization level 3, fat
LTO, one codegen unit, panic abort, lockfile, and `--offline --locked` replay —
and adds:

1. One extra worker build with `-Cprofile-generate=<build>/profraw` appended to
   the existing `RUSTFLAGS`, into its own `target-profile-generate` directory.
   Cargo's own build scripts write elsewhere, so only training profiles are
   merged. That throwaway binary also links `pgo_stubs.c`: instrumentation
   keeps Bun's unreachable WebKit, URL, and simdutf support code alive, so the
   linker needs symbols the standalone integration does not build. Every stub
   aborts, and none of them reaches the measured worker — the final
   `-Cprofile-use` build carries no instrumentation, so that dead code is
   removed again and the measured executable links exactly like the default
   build.
2. A training pass over the training corpus that drives **every engine through
   both lifecycles** — `commonmark` and `gfm`, `fresh` and `reuse` — with one
   process per engine, profile, and lifecycle and an explicit `LLVM_PROFILE_FILE`
   per run, so the profile each engine contributed is named in `build.json`.
   Training documents whose corpus profile this harness does not implement
   (`autolink`, `mdx`, `extensions`, `opt-*`) are ordinary Markdown and train
   under both harness profiles instead of borrowing one.
3. `llvm-profdata merge` from the pinned toolchain's own
   `$(rustc +<toolchain> --print sysroot)/lib/rustlib/<host>/bin/`, so profile
   formats match the compiler. If it is missing, run
   `rustup component add llvm-tools --toolchain <toolchain>`.
4. The final worker build with `-Cprofile-use=<merged> -Cllvm-args=-pgo-warn-missing-function`,
   through the same Cargo invocation and into the same `target` directory the
   default build uses.

`-pgo-warn-missing-function` reports every function the profile does not cover,
so the build log is long and most of it is noise: Cargo's proc-macro and
build-script crates run at compile time, and their profiles are deliberately
kept out of the merged file. The lines worth reading name engine crates, and
they mean the training corpus never reached that code path.

`build.json` gains a `pgo` object recording that PGO was applied, both
`RUSTFLAGS` strings, the training corpus path/SHA-256 and the exact training
case names, both filter files with their SHA-256 values, the merged profile's
path, SHA-256, and byte size, the `.profraw` files each engine produced, and
`engine_profile_data` naming which engines received profile data. The default
build writes no `pgo` object and its executable is unchanged.

### Fairness

- PGO is applied to **all four Rust engines** — Ferromark v2, the original
  OX-Content core, Ferromark v1, and pulldown-cmark — under **one recipe** and
  **one training set**. `RUSTFLAGS` reaches every Rust crate in the shared
  executable, and every engine is driven during training, so no engine is left
  in a `-Cprofile-use` build with no profile data for its own functions.
- The **C engines are not PGO-built**. md4c is C and Bun's engine is a
  Rust/C++ mix; clang compiles those parts with plain `-O3` in both builds.
  Giving them PGO would need a separate clang `-fprofile-generate` recipe,
  which this step does not add. `engine_profile_data` therefore records md4c as
  `none` and Bun as `rust-pgo-partial`: Bun's Rust crates are inside the Rust
  recipe, its C++ Highway and support objects are not. Neither is a PGO-built
  engine in the sense the four Rust rows claim.
- **Published Ferromark binaries are the PGO-built ones** once the release
  pipeline adopts PGO — [`docs/releasing.md`](../../docs/releasing.md) and the
  `native` CI job are where it lands. A PGO row is then the row that describes
  a shipped binary; a default-build row is not.
- **A PGO row may be compared only with the PGO rows of the other engines.**
  Never compare a PGO Ferromark v2 number with a default-build number from any
  engine, and never read md4c's or Bun's PGO-run row as a PGO result for that
  engine. Run both builds over the same held-out documents and compare like
  with like.

### Training and measured sets are disjoint

The harness measures the frozen 57-document broad corpus, so training on it
would report an in-sample number. `--pgo` instead trains on the authored
diagnostics plus one half of the broad documents and leaves the other half for
measurement, exactly as the round-3 PGO split did. Both halves are explicit
file inputs:

| Input | File | Selects |
| --- | --- | --- |
| `prepare.py --pgo-training-filter` | [`filter-broad-train.txt`](../../docs/reports/2026-09-16-arm-round-3/harness/filter-broad-train.txt) | 29 broad documents + 45 authored diagnostics |
| `prepare.py --pgo-measurement-filter` | [`filter-broad-test.txt`](../../docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt) | the other 28 broad documents |
| `run.py --filter-file` | the same `filter-broad-test.txt` | the measured set |

`prepare.py` fails the build if any training document would also be selected by
the measurement filter, so the split cannot silently drift. `test_pgo.py`
additionally checks that the two filters partition all 57 broad documents and
that the measured half still covers both syntax profiles. `run.py --filter` and
`--filter-file` mirror `optimization-rounds/run.py --filter`; the measured
selection, not the whole corpus, is archived as the run's `corpus.json`, and
`run.json` records the filter and its source file.

The training corpus is the optimization-rounds corpus generator's output, which
carries the 57 broad documents and the authored diagnostics in one file:

```sh
python3 benchmarks/optimization-rounds/make_corpus.py \
  /private/tmp/native-bench-training-corpus.json.gz --include-scanner-diagnostics
```

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

To build the PGO variant of the same executable, add the four PGO inputs. Both
builds must exist before a PGO comparison, and both are measured on the
held-out half only:

```sh
python3 benchmarks/optimization-rounds/make_corpus.py \
  /private/tmp/native-bench-training-corpus.json.gz --include-scanner-diagnostics
python3 benchmarks/native-comparison/prepare.py /private/tmp/native-bench-pgo-build \
  --bun-source /private/tmp/native-bench-cache/bun \
  --bun-native-cache /private/tmp/native-bench-cache/native \
  --md4c-source /private/tmp/native-bench-cache/md4c \
  --ox-archive /private/tmp/native-bench-cache/ox.tar.gz \
  --ferromark-v1-source ../ferromark --ferromark-v2-source . \
  --worker benchmarks/native-comparison/worker.rs --compile \
  --lockfile docs/reports/2026-09-15-native-arm/Cargo.lock \
  --pgo --pgo-training-corpus /private/tmp/native-bench-training-corpus.json.gz \
  --pgo-training-filter docs/reports/2026-09-16-arm-round-3/harness/filter-broad-train.txt \
  --pgo-measurement-filter docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt
for build in native-bench-build native-bench-pgo-build; do
  python3 benchmarks/native-comparison/run.py \
    /private/tmp/$build/build.json \
    docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz \
    /private/tmp/$build-heldout-results \
    --filter-file docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt
  python3 benchmarks/native-comparison/report.py /private/tmp/$build-heldout-results
done
```

The default source pins are v2 `e93394e` and v1 `4e15141`; other engine pins are
unchanged. The
[previous matched-flags comparison](../../docs/reports/2026-09-14-native-matched/README.md)
and original pre-correction comparison are preserved separately in
[`2026-09-14-native-engines`](../../docs/reports/2026-09-14-native-engines/README.md).

## Release-readiness reruns

Pass `--ferromark-v2-revision <commit>` with a committed v2 revision to rerun
against the same five comparison-engine pins. The default remains the historical
`e93394e` measurement for reproducibility. The source audit reads the selected
v2 revision from the build metadata. Never substitute working-tree sources.

The [release-readiness report](../../docs/reports/2026-09-15-release-native/README.md)
measures `c232d97` on the original 57 inputs and compares its relative position
with the preceding ARM run. Both the current ranking and changes in relative
throughput use identical, output-agreeing input sets. Small microbenchmark
regressions do not replace this document-level evidence.
