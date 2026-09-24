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
are compiled directly. The stack adapter supplies the actual cached pthread
stack boundary on macOS and Linux; recursion checks remain active. A forced
header supplies platform/release assertion macros in place of Bun's WebKit
umbrella header.
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
[Profile-guided optimization](#profile-guided-optimization) below. The build
runs on macOS (Apple Silicon) and Linux (x86-64); the few platform differences
are listed under [Platforms](#platforms).

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
IDs, callouts, and fence metadata cleanup. V1 also
disables its renderer extras. Original OX has no native flags to turn off those
three behaviors; it remains unchanged. The harness neither replaces its renderer
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

## Platforms

`prepare.py` builds on macOS and Linux and refuses other hosts. Both platforms
use clang and clang++ with the same flags, the same pinned sources and
adapters, the same mimalloc configuration and md4c allocator redirection, the
same pinned nightly Rust toolchain, and the same `RUSTFLAGS`. What differs is
listed in `prepare.PLATFORMS` and recorded in every `build.json` under
`platform`, together with the host triple and both clang versions:

| Aspect | macOS (Apple Silicon) | Linux (x86-64) |
| --- | --- | --- |
| C++ runtime linked for Highway | libc++, Apple clang's only runtime | libstdc++, clang's default on Linux |
| Rust link driver and linker | rustc's default `cc` (Apple clang) with the system ld64; environment unchanged | `clang`, through `CARGO_TARGET_<host>_LINKER`, with the system GNU ld rather than rustc's bundled rust-lld |
| Stack bound for Bun's recursion check (`stack.c`) | `pthread_get_stackaddr_np` and `pthread_get_stacksize_np` | `pthread_getattr_np` and `pthread_attr_getstack`, as `WTF::StackBounds` does on Linux |
| Bun's `OS()` branch (`native.h`) | `OS(DARWIN)`: Highway `memmem` replaces libc's through an assembler alias | `OS(LINUX)`: the same replacement through a weak alias |
| `-C target-cpu=generic` | generic AArch64 | the x86-64 baseline (SSE2); v2, v1's `memchr`, and Highway still select SSSE3/AVX2 paths at run time |
| Compiler and linker versions | Apple clang and ld64 from the installed Xcode tools | the runner's default clang and GNU ld |
| PGO training-binary stubs (`pgo_stubs.c`) | the shared list | the shared list plus the Bun support symbols GNU ld also resolves; the measured executable links none of them |

On macOS, the adapters compile to byte-identical objects and `prepare.py`
passes Cargo the same environment as before Linux support, so the macOS
executable is unchanged. Bun's own Linux release build also turns on
mimalloc's global `malloc` override and disables transparent huge pages for
mimalloc arenas. The harness applies neither, on either platform: every engine
already allocates through the same mimalloc, and one mimalloc configuration
keeps the platforms comparable.

The Linux path is exercised on GitHub's x86-64 runners by the
[workflow](#linux-x86-64-workflow) below. `run.py` records `pmset` power and
thermal state on macOS; on Linux it records `/proc/stat` CPU counters (including
hypervisor steal), per-CPU clocks, and any exposed thermal zones before and
after every process round.

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
   linker needs symbols the standalone integration does not build. GNU ld on
   Linux resolves more of them than ld64 on macOS, so Linux adds 42 function
   stubs and one zero-initialized data symbol (Bun's closed-stdio flags). Every
   function stub aborts, and none of them reaches the measured worker — the final
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
an empty `/private/tmp/native-bench-cache` directory, create its `native/`
subdirectory, and run it there. It restores the original source pins and checks
the original archive hashes. Adjust source paths below for your checkout layout;
on Linux, use a directory under `/tmp` instead of `/private/tmp`. The commands
are the same on both platforms. The builds run with `--offline`, so on a
machine whose Cargo registry cache lacks the locked crates, fill it first with
`cargo +nightly-2026-07-20 fetch` in the `bun/` directory of a throwaway
`prepare.py` run without `--compile`, as the workflow does.

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

`archive.py` turns a finished run into a `docs/reports`-style directory: it
copies and compresses the evidence, copies this harness, `restore.py`, the
source audit and the host description, rechecks every timed window, compares
every HTML output with a reference report, and generates `README.md`,
`PROVENANCE.md`, `checks/commands.json`, and `SHA256SUMS` from the archived
files. `python3 benchmarks/native-comparison/archive.py --help` lists its inputs.

## Linux x86-64 workflow

[`.github/workflows/native-comparison.yml`](../../.github/workflows/native-comparison.yml)
produces a complete comparison report on a GitHub-hosted `ubuntu-latest`
x86-64 runner. It runs on every pull request that changes this directory or the
workflow, measuring the pull request's head, and on demand:

```sh
gh workflow run native-comparison.yml -f revision=main            # default build, 57 documents
gh workflow run native-comparison.yml -f revision=<commit> -f pgo=true
```

`revision` accepts a branch, tag, or commit and is resolved to a full commit
before `prepare.py` exports it. With `pgo` (always on for pull requests, so both
paths stay working) the job also builds the PGO executable and measures both
builds on the held-out half, as described above. One job does everything on one
runner:

1. Describe the host (`lscpu`, CPU flags, transparent huge page mode, memory,
   clang) into `host.txt`, and pass the CPU model to `run.py` as `BENCH_CPU`.
2. Install the pinned nightly named by `prepare.BUN_TOOLCHAIN`, with
   `llvm-tools` for PGO, and run the harness unit tests.
3. Restore the sources with a copy of the latest report's `restore.py`, then
   fill Cargo's registry cache from a throwaway workspace so the real builds
   stay `--offline`.
4. Build with the latest report's `Cargo.lock` seeded through `--bun-lock`,
   run `run.py --verify-only`, and build the PGO executable if requested — all
   builds finish before any timing starts.
5. Measure all 57 documents (and the held-out half with both builds), generate
   the tables, audit the built sources, and assemble the report with
   `archive.py`.

The artifact `native-report-linux-x86-64` holds one directory,
`<date>-native-linux-x86-64/`, in the layout of the archived reports. The job
summary shows its headline table. Budget about 30 minutes for a default run and
60 for a PGO run; the job stops at 90.

### What a CI report can and cannot claim

- **One host per run.** All six engines alternate in the same process rounds
  and rotating windows on that runner, one worker at a time, so the ratios
  between engines are meaningful for that host's CPU.
- **A shared runner.** The runner is a virtual machine on shared hardware.
  Neighbors, clocks, and even the CPU model change between runs, so absolute
  nanoseconds are not comparable across runs; `run.json` records hypervisor
  steal for every process round and `host.txt` names the CPU. A second run on a
  different host is an independent measurement, not a replication.
- **Its own platform.** An x86-64 figure does not describe Apple Silicon or
  the reverse: SIMD paths, compiler back ends, and cache hierarchies differ.
  Publish the two platforms side by side, each with its own machine named.
- **No significance claim**, as for every report of this harness.

### Archiving a CI report

1. Download the artifact:
   `gh run download <run-id> -n native-report-linux-x86-64 -D /tmp/native-ci`.
2. Check it: `(cd /tmp/native-ci/<date>-native-linux-x86-64 && sha256sum -c SHA256SUMS)`
   and read `checks/commands.json` — the registry resolution, the source audit,
   and the HTML comparison with the reference report should be clean, and any
   output that differs from the Apple Silicon reference needs an explanation
   before publication.
3. Move the directory unchanged to `docs/reports/`. Its README links the
   reference report as a sibling directory. Add narrative, such as comparisons
   with earlier runs, as separate files so the generated evidence and its
   checksums stay intact.

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
