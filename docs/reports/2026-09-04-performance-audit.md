# Performance and correctness audit — 2026-09-04

Baseline: `14009ced40e6c06d81e8208fc4e2f8ad4a9ac29b` (ferromark 0.7.0).

The initial audit identified eight findings. Follow-up implementation and review
now cover all eight, plus two additional review findings, in seven pull requests.
The original measurements below describe the initial pass; the final follow-up
results and dependency order are recorded at the end of this report.

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


## Follow-up implementation and review

The remaining issues were delegated to GPT-5.6 Luna agents at high reasoning
and reviewed independently. Review corrections covered nested footnote numbering,
reset ownership, document-wide heading identity, UTF-8 source ranges, event-buffer
retention, document-only frontmatter, Cargo PGO configuration, and signal cleanup.

| Issues | Change | Pull request | Base dependency |
| --- | --- | --- | --- |
| #244, #245 | Bound angle scanning and trailing-parenthesis trimming | [#252](https://github.com/sebastian-software/ferromark/pull/252) | main |
| #246 | Index autolink exclusion ranges | [#254](https://github.com/sebastian-software/ferromark/pull/254) | #252 |
| #247 | Reuse footnote state and global numbering | [#253](https://github.com/sebastian-software/ferromark/pull/253) | main |
| #248 | Reserve all emitted heading IDs | [#256](https://github.com/sebastian-software/ferromark/pull/256) | #253 |
| #249, #250, #257 | Parse MDX segments once; preserve document state/frontmatter | [#259](https://github.com/sebastian-software/ferromark/pull/259) | #256 |
| #251 | Repair standalone profiling and process cleanup | [#260](https://github.com/sebastian-software/ferromark/pull/260) | main |
| #255 | Patch the transitive homepage TOML dependency | [#258](https://github.com/sebastian-software/ferromark/pull/258) | main |

Merge #258 first to remove the shared homepage audit blocker. Then merge
#252 → #254 and #253 → #256 → #259, retargeting each dependent PR to main once
its base has merged. #260 is otherwise independent. The complete CI for #258
is green; older branches still report the homepage audit until that fix is included.

### Final measurements

Same release-debug audit harness and input/options on the baseline and optimized
revisions; five 150 ms windows per size, with source construction outside timing.
Times below use the output-only reuse lane (MDX constructs its returned output).

| Probe | Baseline | Optimized | Observation |
| --- | ---: | ---: | --- |
| 8,192 angle autolinks, GFM literals enabled | 16.065 ms | 1.310 ms | 12.3× faster; range-query scans removed |
| 8,192 footnote definitions | 9.219 ms | 4.394 ms | 2.1× faster in this paired run |
| 8,192 MDX segments | 13.650 ms | 3.929 ms | 3.5× faster on final code |
| CommonMark 52,342-byte corpus, all core changes combined | 206.049 µs | 203.790 µs | −1.1% |
| Tables 4,552-byte corpus, all core changes combined | 37.647 µs | 37.962 µs | +0.8% |

The reusable-Renderer control lanes for CommonMark/tables were within +1.1%.
Local timings are machine-specific; the raw windows retain observed variance.

Footnotes at 8,192 definitions requested 572,209,004 → 3,688,052 cumulative
allocation bytes (99.36% reduction); allocation/reallocation calls fell from
106,578 to 49,239. With a reusable Renderer, requested bytes fell from
570,968,540 to 2,447,588.

MDX at 128 segments requested 2,083,022 → 138,512 cumulative bytes and made
6,592 → 458 allocation/reallocation calls. Retaining parsed events trades memory
for avoiding the second parse: a separate live-allocation probe at 8,192 segments
measured 1,408,112 → 2,883,664 additional live requested bytes above the pre-render
baseline. This is **not RSS**, and there is no claim of lower peak memory. Review
reduced the minimum per-segment event capacity to eight and frees each event
buffer immediately after rendering its segment.

Raw follow-up data: [measurements](2026-09-04-performance-audit/follow-up-measurements.json).
The [live-allocation probe](2026-09-04-performance-audit/mdx-memory-probe.rs)
includes reproduction instructions. The final combined review commit was
`0a46520` (all implementation PRs, before this documentation update).

### Final validation

- Combined implementation: **968 passed, 0 failed, 3 ignored**, including trusted
  CommonMark coverage; all-target/all-feature Clippy with warnings denied, format,
  diff checks and the new profiling contract tests pass.
- The combined MDX integration regression includes natural heading suffixes,
  Unicode frontmatter, JSX, inline notes, forward and nested reference notes.
  It failed before the changes and passes after their integration.
- Both profiling entry points ran with real macOS `sample` from `/tmp`, with
  MD4C_DIR unset and a custom CARGO_TARGET_DIR. Both produced profiles and their
  workload PIDs were confirmed reaped. Mocked tests cover failure, SIGTERM,
  PGO flags/paths, exact Cargo JSON artifacts, and cross-parser dispatch.
- Actual PGO profile-use and cross-parser execution were not exercised; their
  invocation contracts were tested. Ferromark-only profiling explicitly uses
  the commonmark preset and a reused output buffer.
- The homepage fix passes frozen install, typecheck, build/prerender verification,
  production audit, and actual remark TOML-frontmatter pipeline checks for valid
  and malformed input. No high advisories remain in that audit; one low and six
  moderate advisories remain.
