# Performance and correctness audit — 2026-09-04

Baseline: `14009ced40e6c06d81e8208fc4e2f8ad4a9ac29b` (ferromark 0.7.0).

Two bounded scanner fixes were implemented locally. Eight distinct findings
were filed as GitHub issues, including the two fixes pending review/merge.
The other six findings remain open; this audit does not claim to fix them.

## Method and scope

- macOS 26.6.2, ARM64, rustc 1.97.1 / LLVM 22.1.6. The repository's
  `target-cpu=apple-m1` configuration was used; no PGO.
- Current test suite, trusted CommonMark conformance, public API regression
  tests, existing Criterion parsing/extension/pathological benchmarks, and
  targeted source review of inline parsing, rendering, footnotes and MDX.
- CPU profiling with macOS `sample`: a five-second commonmark-50k profile,
  a five-second angle-start profile, and four-second profiles for exclusion
  ranges and footnotes. Symbol/source attribution can include inlined callees.
- Scaling probes use five 150 ms windows, a warmed caller-owned output buffer,
  and separate fresh-parser and reusable-Renderer lanes. Source construction
  and JSON output are outside the timed regions. A disabled allocation-counter
  branch remains in this probe, so use it for scaling and paired comparisons,
  not as a replacement for production Criterion throughput numbers.
- Allocation probes separately count allocations/reallocations and cumulative
  requested bytes during one warmed render. These are **not peak live memory**.
- Criterion confirmation uses 30 samples, 0.3 s warmup and 1 s measurement;
  the pathological group retains its explicit 20-sample override. The original
  executable was retained and measured again after builds completed, then
  compared with the changed executable. Short local measurements are screening
  evidence, not publication-quality cross-machine or cross-parser rankings.

The normal `parsing/*` benchmarks use `Options::default()` and allocate their
output. This differs from the README's GFM-overlap buffer-reuse publication
lane. In particular, **default options disable autolink literals**; the
autolink stress probes explicitly enable them. No x86-64 measurements or fresh
competitor comparison were performed.

## Baseline and final measurements

Criterion arithmetic means (microseconds per render; negative change is faster):

| Fixture (default or required extension options) | Before | Final | Change |
|---|---:|---:|---:|
| parsing/commonmark_5k | 22.748 | 23.136 | +1.71% |
| parsing/commonmark_20k | 77.341 | 76.151 | -1.54% |
| parsing/commonmark_50k | 202.528 | 200.802 | -0.85% |
| parsing/commonmark_1m | 3787.646 | 3808.095 | +0.54% |
| parsing/tables_5k | 38.016 | 38.689 | +1.77% |
| extensions/deflists_5k | 21.873 | 22.141 | +1.22% |
| extensions/tables_merged_5k | 33.124 | 33.230 | +0.32% |
| extensions/mixed_extended_5k | 34.866 | 35.551 | +1.96% |

The ordinary document and extension lanes stay within about ±2% in this short
confirmation run. This does not establish a general throughput improvement.
For orientation, final commonmark-50k/default is 248.6 MiB/s.

Targeted scaling medians, fresh parser with reused output buffer:

| Case (8,192 repeats) | Before | Final | Speedup |
|---|---:|---:|---:|
| angle_starts | 21.824 ms | 184.054 µs | 118.6× |
| trailing_parens | 11.291 ms | 19.633 µs | 575.1× |

Both size series now grow approximately linearly. Parenthesis output remains
unchanged, while invalid URI acceptance is intentionally corrected.

**Measured tradeoff:** `invalid_html_starts_64k` rises from 297.818 to
310.835 µs (+4.37%); `emphasis_many_link_boundaries` rises 2.64%. The former
is a confirmed cost of the stricter candidate scan on repeated short invalid
tags. A combined byte classification reduced the initial approximately 5.3%
regression but did not eliminate it. This limitation is recorded in #244;
the change is retained for its correctness fix and removal of quadratic work.

## Profiling findings

On commonmark-50k/default, 4,146 flat samples include:

| Symbol / operation | Samples | Share |
|---|---:|---:|
| `InlineParser::parse_with_options_in_document` | 799 | 19.3% |
| `RenderContext::render_block_event` | 510 | 12.3% |
| `render_to_writer_with_state` (including inlined work) | 424 | 10.2% |
| `memmove` | 269 | 6.5% |
| `render_inline_content` | 236 | 5.7% |
| `escape_text_into` | 236 | 5.7% |
| `HeadingIdTracker::make_id` | 165 | 4.0% |

These are flat symbol samples, not independent end-to-end stage percentages.
They justify examining inline parsing first, but alone do not establish waste
in ordinary escaping or copies. No speculative issue was filed for those costs.

The targeted profiles and scaling tests identified concrete problems:

- **Angle candidates:** 4,098 / 4,126 samples at the inlined autolink-discovery
  call. The original time grows approximately fourfold when input doubles.
- **GFM exclusion ranges:** 2,694 samples at the literal-discovery call;
  8,192 existing angle links take 15.965 ms with literal discovery enabled,
  versus 1.022 ms disabled, with the same output. Existing range lists are
  linearly searched for each candidate.
- **Footnotes:** 1,071 / 3,001 samples in `RenderState::reset -> __bzero`.
  8,192 notes in 299,774 input bytes cause 572,209,004 cumulative requested
  allocation bytes. Reusing the outer Renderer barely changes this because
  each note recreates an array sized to the entire definition store.
- **MDX segments:** 128 formatted paragraphs separated by JSX (4,736 input
  bytes) cause 6,592 allocation/reallocation calls and 2,083,022 cumulative
  requested bytes. Every Markdown segment is block-parsed twice and receives
  fresh render state. This is an allocation/setup problem, not an assertion
  of quadratic MDX parsing.

## Implemented changes

`try_parse_autolink` stops at another `<`, spaces and all ASCII controls. This
makes candidate scanning linear for repeated openers and corrects acceptance
of nested `<` and literal control characters inside URI autolinks. Valid
autolinks following failed candidates remain recognized, including beyond the
inline mark cap.

`trim_autolink_trailing` lazily computes the excess closing-parenthesis count
once and decrements it as parentheses are removed. Other removable suffixes
do not contain parentheses, so the balance remains valid across punctuation
and entity trimming. URLs without trailing parentheses incur no added count.

Both cases now have persistent Criterion benchmarks. Public-API tests cover
32,768 unmatched parentheses, balanced and partially balanced suffixes,
punctuation/entities, nested URI starts, controls, and a valid link after
8,192 failed angle starts.

## Issues

| Issue | Finding | Status |
|---|---|---|
| [#244](https://github.com/sebastian-software/ferromark/issues/244) | Quadratic angle scanning and invalid URI acceptance | Local fix and tests prepared |
| [#245](https://github.com/sebastian-software/ferromark/issues/245) | Quadratic GFM closing-parenthesis trimming | Local fix and tests prepared |
| [#246](https://github.com/sebastian-software/ferromark/issues/246) | Quadratic GFM exclusion-range searches | Solution proposed |
| [#247](https://github.com/sebastian-software/ferromark/issues/247) | Quadratic footnote ordinal-array initialization | Solution proposed |
| [#248](https://github.com/sebastian-software/ferromark/issues/248) | Generated heading suffixes collide with natural IDs | Reproduced, solution proposed |
| [#249](https://github.com/sebastian-software/ferromark/issues/249) | MDX resets heading/footnote identity across segments | Reproduced, solution proposed |
| [#250](https://github.com/sebastian-software/ferromark/issues/250) | MDX double parsing and per-segment allocations | Measured, solution proposed |
| [#251](https://github.com/sebastian-software/ferromark/issues/251) | Profiling scripts reference the moved benchmark | Reproduced, solution proposed |

Existing open and closed issues were checked before filing. #246 is distinct
from #167's emphasis-boundary fix; #247 concerns repeated initialization rather
than #44's ordinal lookup; #249/#250 extend the remaining gaps after #179's
shared link-reference support.

## Validation and artifacts

- `cargo test --locked --all-features`: **941 passed, 0 failed, 3 ignored**,
  including `commonmark_spec_trusted_full_conformance` (652/652 spec examples).
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- The nested-angle and control-character regressions were observed failing
  before the fix and passing afterward. Parenthesis tests protect existing
  output semantics; the scaling probe demonstrates the performance correction.
- A stale preexisting default-feature test artifact required a package-local
  build-cache cleanup; the final all-feature validation recompiled the crate.
- New Criterion cases have no pre-change named baseline. They were measured
  separately after the named-baseline comparison reached those new cases.

Evidence: [measurements](2026-09-04-performance-audit/measurements.json),
[commonmark profile](2026-09-04-performance-audit/commonmark-profile.txt),
[angle profile](2026-09-04-performance-audit/angle-profile.txt),
[range profile](2026-09-04-performance-audit/ranges-profile.txt),
[footnote profile](2026-09-04-performance-audit/footnotes-profile.txt), and
[correctness reproductions](2026-09-04-performance-audit/correctness-reproductions.txt).

Reproduce:

```sh
cargo build --locked --profile release-debug --features mdx --example performance_audit
target/release-debug/examples/performance_audit check
target/release-debug/examples/performance_audit measure angle_starts
target/release-debug/examples/performance_audit measure trailing_parens
target/release-debug/examples/performance_audit measure autolink_ranges
target/release-debug/examples/performance_audit alloc footnotes 8192
target/release-debug/examples/performance_audit alloc mdx_segments
cargo bench --locked --bench parsing -- --sample-size 30 --warm-up-time 0.3 --measurement-time 1
```

For sampling, run `performance_audit spin <case> <size> <seconds>` and attach
macOS `sample` to that process. The existing `profile_harness` also remains
usable for fixture/preset profiling. The broken wrapper scripts were bypassed.
