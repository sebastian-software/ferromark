# Bound HTML escape searches

Date: September 12, 2026.

## Final adoption scope

The PR adopts this optimization only on **ARM64 with NEON**. Other targets keep
the merged baseline's escaping implementation; the quote-heavy quadratic case
remains open there. This follows the project's clarified ARM performance
priority. CI x86 timings are diagnostic observations on shared hosted runners,
not the acceptance reference for ARM throughput.

The original `linear` snapshot below included a portable growing-window path.
That non-NEON path is **not adopted**. Native Linux follow-ups found plain-escape
costs and integration-sensitive document results; experimental x86 SIMD and
outlining variants are also outside this PR's production implementation. They
are retained on the experiment branch and in the
[separate native experiment archive](https://github.com/sebastian-software/ferromark/blob/e10530a2e70b1e5a49e3483b51442b41623fbbb2/docs/reports/2026-09-12-native-linear-escaping/REPORT.md).

The final source is `snapshots/arm-scoped-escape.rs.gz`, with
`arm-scoped.patch` against the frozen baseline and exact hashes in
`arm-scope.json`. Rebuilding with the original ARM driver, toolchain, source
path and build working directory produces **the same 631,900-byte machine-code
section** as the recorded `linear` executable (SHA-256
`11a01c3adb7ca8171d9ad7b0c51759c23537fc37106a1f0775d08759707c9b38`).
The ARM results below therefore apply to the final scoped implementation. No
new x86 performance or universal cross-platform linearity claim is made.

## Original measured candidate

Fix repeated suffix scans in both text and attribute escaping. The previous
implementation searched the entire remaining suffix for `<`, `>` or `&` before
looking for quotes. Quoted code or attributes without those first three bytes
repeated that unsuccessful search after every quote. At fixed quote density,
doubling the input approximately quadrupled search work.

The proposed `linear` source first searches at most 128 bytes with the existing
shared byte-set scanner. If no escape appears, an outlined helper searches the
rest. AArch64 with NEON uses the shared scanner for tails of at least 256 bytes;
shorter tails retain a bounded memchr search. Other architectures search disjoint
windows of 256, 512, 1024, ... bytes. Both the common-character and quote searches
are restricted to the same window.

A window containing the next escape is bounded by a constant multiple of the
distance to that escape plus the fixed initial window. Earlier windows are
geometrically smaller. Successive escape calls advance the input cursor, so total
work is linear, including when quotes repeatedly occur before `<`, `>` or `&`.
Window growth saturates and each slice is capped by the remaining length.
The NEON path stops at the first byte in the complete escape set directly.

Escaped bytes, replacement strings, output ownership and allocation policy are
unchanged. The change introduces no new unsafe code or SIMD primitive. The
single-quote difference between text and attributes remains intact.

## Measured result

<!-- measured-focus:start -->
The proposed source removes the quadratic rescan pattern. The ordinary
code control improves in repeated pairs; the earlier multi-percent inline
and table losses do not persist in this integration. Small repeated costs
remain visible below. This is an algorithmic fix with disclosed sub-percent
controls, not a claim that every input becomes faster.

| Case | Before µs | After µs | Primary pair changes | Longer follow-ups |
|---|---:|---:|---|---|
| code | 15.745 | 15.257 | -3.03%, -3.10%, -2.97% | -3.17%, -3.03% |
| inline | 39.534 | 39.842 | +0.69%, +0.89%, +0.56% | +0.55%, +0.54% |
| tables | 41.447 | 41.347 | -0.23%, -0.18%, -0.36% | -0.34%, -0.55% |
| guard/deep-emphasis | 38.931 | 39.101 | +0.39%, +0.71%, +0.38% | +0.87%, +0.70% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 16.886 | 16.646 | -1.08%, -1.45%, -1.42% | -1.17%, -1.34% |
| corpus/vue-docs/src/guide/extras/render-function.md | 39.907 | 38.998 | -2.45%, -1.73%, -2.28% | -1.63%, -1.97% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 67.538 | 66.503 | -1.53%, -1.14%, -5.67% | -5.18%, -4.09% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 16.277 | 16.207 | -0.93%, -0.54%, -0.17% | -0.93%, -1.38% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 27.462 | 27.519 | +0.21%, +0.33%, +0.07% | -0.07%, +0.68% |
| escape-regime/code-plain-1048576 | 90.293 | 79.666 | -11.77%, -11.67%, -11.62% | — |
| escape-regime/code-quotes-1048576 | 2249826.459 | 1649.079 | -99.93%, -99.93%, -99.93% | — |
| attribute-fence/double-quotes-65536 | 23955.625 | 253.405 | -98.94%, -98.94%, -98.94% | — |
| attribute-fence/sparse-quotes-1048576 | 35944.334 | 1090.337 | -96.97%, -96.96%, -96.97% | — |

In particular, inline and deep-emphasis controls retain small repeated
positive differences in the longer pairs. These are not removed by A/A
normalization or assumed workload weights. The complete tables preserve
both those costs and the identical-binary observations. Isolated larger HTML
outliers do not reproduce in the longer counterchecks.
<!-- measured-focus:end -->

[RESULTS.md](RESULTS.md) is generated from all archived windows, including every
unfavorable control and rejected experiment. The large quoted-input gains are
synthetic stress results, not representative-document speedups or a new engine
ranking. Published README and homepage tables remain the existing dated full
engine comparison; this experiment compares Ferromark against its merged baseline.
It makes no new allocation-count or memory-consumption claim.

## Selection and counterchecks

The acceptance policy prioritizes clear improvements and rejects meaningful,
repeatable losses on other inputs. It does not average regressions away or assume
workload frequencies. A/A measurements use a byte-identical copy of the baseline
binary to observe the variation of the same two-process protocol. Their values
are disclosed, not subtracted from candidate results.

The first full portable implementation (`production`) removed quadratic work,
but its small initial window added overhead to medium-length unmatched runs.
Using the joint NEON tail unconditionally (`final`, a historical experiment label)
improved these runs but repeatedly slowed ordinary inline and table controls.
The proposed `linear` variant retains memchr for the short bounded tail and uses
the shared NEON scanner for longer tails. Its boundary and integration were
screened as `hybrid-tail` before formatting and final source verification.

Other archived probes include growing inline windows (`grow-memchr`,
`grow-prefix`), independent cached search cursors (`cached-long`), outlining with
and without a shared prefix (`outlined-growth`, `outlined-prefix`), an explicit
short-input branch (`early-short`), thresholds in the search or whole-output loop
(`threshold-*`, `bulk-*`), and larger initial portable windows (`wide-*`).
The threshold and whole-output probes retained unfavorable small-code controls.
The wider-window alternatives were screened but not selected for full adoption.
Changes to layout, inlining or register use are plausible integration effects;
these measurements do not establish a single compiler-level cause.

The [preceding five NEON integrations](../2026-09-12-neon-text-escape/REPORT.md)
remain rejected as those exact patches. This follow-up includes attribute escaping
and a portable bound, and proposes a different integration.

## Method

Baseline: merged main `e731dc8162828e0ac2df30b7567107f560159ff8`.
Measurements use Apple M1 Pro and Rust 1.97.1, opt-level 3, fat LTO, one codegen
unit, panic abort and the system allocator. The standalone native driver removes
ambient Rust flags and uses locked dependencies. Input loading, option creation
and JSON are outside the timer; rendering and destruction of owned output are
inside. Reuse and MDX cases remain explicitly labeled.

- Three fresh process pairs each cover 93 ordinary documents, 27 text/code stress
  cases and 14 attribute stress cases. Each pair has seven alternating windows
  of at least 50 ms. Ordinary cases warm for 60 ms and check the timer every 16
  renders. Stress cases warm for 20 ms and check after each render, including
  overshoot when a baseline render already exceeds the window duration.
- The text suite spans plain text, fenced plain text, late quotes, repeated quotes
  and sparse ampersands from 128 bytes through 1 MiB, with additional unmatched
  inputs at 8 MiB. The attribute suite uses actual Markdown fenced-code info
  strings, which invoke attribute escaping through the normal HTML writer:
  plain, double-quoted, single-quoted and mixed quoted info at 128 bytes, 4 KiB
  and 64 KiB, plus plain and sparsely quoted info at 1 MiB. It measures complete
  Markdown rendering, not an isolated escaping microbenchmark.
- Follow-ups select every ordinary case slower in all three pairs or over 2%
  slower in any pair, plus previous code/table controls and the five profile
  documents. Two fresh A/A and candidate pairs use eleven 80 ms windows and
  80 ms warmup. Stress outliers use the same selection rule and eleven 80 ms
  single-render windows with 20 ms warmup. These are counterexample-selection
  rules, not a blanket 2% acceptance threshold.
- The baseline, `production`, `final` and proposed `linear` sources each pass
  111,902 exact-output guards. Other screened prototypes pass the initial 2,695
  guards; their limited screens are not production validation. The stress runner
  checks complete HTML equality before timing every selected case.
- Two regression tests exercise dense and sparse quotes and count logical search
  prefixes under `cfg(test)`; no timing assertion or release instrumentation is
  introduced. The archived instrumented baseline fails the doubling-work budget.
  A separate scalar oracle checks all byte values across search/window boundaries
  and competing later escapes. These augment the existing escaping tests.
- No build, correctness suite or profiler overlaps recorded timing. OS scheduling
  and core affinity are uncontrolled. Pair ranges are observations, not confidence
  intervals or proof of identical performance on every possible input. Local
  timing covers AArch64 only; cross-compilation and native CI exercise other
  backends' correctness, not comparative throughput.

See [REPRODUCE.md](REPRODUCE.md) for source reconstruction, replay commands and
archive checks. The archive records exact source hashes, patches, driver and
input identities, full timing windows, validation summaries and failed experiments.
