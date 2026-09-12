# NEON text-escape integration screens

Date: September 12, 2026.

**Decision: retain the merged library implementation.** Five ways of integrating
a shared four-byte escape scan all have repeated unfavorable controls. The
simplest version improves several real documents, but the longer follow-ups
confirm smaller losses on tables, MDX and separate HTML blocks. The acceptance
policy does not permit averaging those losses away without evidence about actual
workload frequencies. No performance implementation is proposed by this report.

This tightens the follow-up to the [native profiles](../2026-09-12-ox-profile-after-block-render/REPORT.md).
That earlier screen's “not over 2% slower in both confirmation pairs” condition
is insufficient for the stricter policy. The autolink prefilter, HTML
specialization, short copies and soft-break prototypes remain deferred or
rejected as described there. Neither library source nor published benchmark
tables changes in this investigation.

[RESULTS.md](RESULTS.md) is generated from archived measurement windows. It lists
all measured cases, favorable and unfavorable. [REPRODUCE.md](REPRODUCE.md)
explains how to reconstruct the source and driver and check the archive.

## A real rescan problem

For suffixes longer than 128 bytes, the baseline first searches for `<`, `>` or
`&`, then searches for a quote before that position. When quoted code contains
none of the first three characters, each successive quote makes the first search
traverse the remaining suffix again. At fixed quote density this produces
quadratic scanning work. Finding all four characters together bounds each search
to the next escape; successive searches advance through the input.

The 27-case stress suite covers ordinary text, fenced code without escapes,
a late quote, repeated quoted code and sparse ampersands from 128 bytes to 1 MiB,
plus plain text and fenced code at 8 MiB. It tests both the rescan problem and the
possibility that a narrower SIMD scan loses on long unmatched runs. The complete
screen shows a particularly large improvement for the 1 MiB quoted-code case;
long unmatched runs also improve. These synthetic results do not represent
typical-document speedups or establish an acceptable whole-library integration.

This is a useful reproducer for a future fix. It does not justify merging any of
the exact patches below despite their ordinary-workload regressions. The same
source pattern exists in other architectures' long text path and in attribute
escaping; this investigation does not measure or modify those paths.

## What was tried

All five integrations restrict the new behavior to AArch64 with NEON. They use
the existing `ByteSet<4>` implementation and preserve the escape byte set,
replacement strings and allocation policy. There is no new SIMD primitive.

1. **Direct scan (`direct-neon`).** Replace `first_text_escape` with the shared
   scanner at every length. Vue and Rust improve in repeated measurements, but
   `guard/one-table`, `mdx/tables-code` and separate HTML blocks repeatedly lose.
   This source includes the formatted boundary test prepared during the attempted
   implementation. It was initially called `production` in the harness; that
   label remains in its raw windows. The final decision is rejection, and the
   actual repository library is restored to the baseline.
2. **Outlined long search (`outlined-neon`).** Keep the existing short-input
   branch and call an outlined shared scanner for long suffixes. This helps some
   of the preceding controls but produces repeatable losses on code and dense
   escaping. Outlining is not a free integration boundary.
3. **Only subsequent searches (`tail-neon`).** Retain the original first search;
   use the shared scanner only after an escape has already been found. This
   avoids changing the initial inline entity probe and removes the original MDX
   loss in the screen. Follow-ups still reproduce a loss on the long-declaration
   HTML control, with less stable smaller losses elsewhere. It is deferred.
4. **Complete long-output loop (`bulk-neon`).** For text-output inputs over the
   existing 128-byte crossover, call one outlined loop that uses the shared
   scanner throughout. Short writes and `first_text_escape` retain their prior
   implementations. The extra dispatch/integration substantially penalizes
   small code and fence controls despite other gains.
5. **Explicitly inline the bulk dispatcher (`bulk-inline-neon`).** Force inlining
   of the preceding variant's public dispatcher on NEON while retaining the
   outlined long loop. The targeted screen and follow-ups show that this reduces
   some losses but does not eliminate them. It is not expanded into another
   complete confirmation because the counterexamples already disqualify it.

Several losing controls do not exercise a long escape scan at all. Changes in
code layout, dispatch and inlining are plausible explanations, not established
attribution from this round. The explicit-inlining probe tests one such change;
it does not prove the cause of all observed differences. No workload-frequency
assumptions are introduced to turn a mixed result into an overall win.

## Method and validation

The frozen baseline is merged main `e731dc8162828e0ac2df30b7567107f560159ff8`.
Native release builds use Rust 1.97.1, opt-level 3, fat LTO, one codegen unit,
panic abort and the system allocator on Apple M1 Pro. Input loading, options and
JSON are outside timing; rendering and destruction of owned output are inside.
Reuse and MDX cases remain explicitly identified in the corpus. The standalone
workspace removes ambient Rust flags and retains its locked dependencies.

- The direct scan has three fresh process pairs over all 93 ordinary cases,
  seven alternating windows of at least 50 ms, and 60 ms warmup. Eight cases
  receive two additional pairs of eleven 80 ms windows and 80 ms warmup: every
  positive difference in all three initial pairs, plus any case over 2% slower
  in one of those pairs. This includes an isolated large outlier that does not
  reproduce.
- Outlined search, tail-only search and bulk-loop variants each receive the same
  93-case initial screen. The explicit-inline bulk variant screens 24 cases,
  including known unfavorable controls and the five target corpus documents.
  Tail-only follow-ups include every screen loss over 0.5%, the target documents
  and preceding controls. The remaining follow-ups target the code, escape,
  table, MDX and fence counterexamples listed in the generated tables. Each
  variant has two longer fresh pairs; no missing case is treated as a pass.
- Ordinary windows check the timer after every 16 renders. The complete
  `single-neon` stress screen checks after each render, with five alternating
  30 ms windows and 20 ms warmup. The slowest baseline render already exceeds the
  requested window minimum. Times divide elapsed time by actual render count,
  including overshoot. Shared timer overhead limits the tiniest stress cases.
- Frozen baseline and the initial direct-scan prototype each pass 111,902 exact
  output checks, including public events, MDX and adversarial fence cases. All
  five integration variants pass the initial 2,695 guards and all 93 timed-case
  output guards. The complete stress runner additionally checks exact HTML for
  its 27 cases. These are differential checks, not a claim that every rejected
  prototype completed all production gates. The proposed boundary test passed
  on the baseline; it remains only in the archived direct-scan patch.
- No builds, correctness guards or profiles overlap recorded timing windows.
  OS scheduling remains uncontrolled. Repeated ranges are observations, not
  confidence intervals, and do not prove absence of regressions elsewhere.
  There is no x86 performance conclusion and no additional memory measurement.

Two preparation problems are excluded from the numerical evidence: the initial
sixteen-render stress pilot lost its raw files when a rerun reused its directory,
and that rerun was then interrupted by moving its output directory. A separate
attempted production run was stopped because the source-copy step had failed.
The complete stress screen and integration confirmations use validated source
copies and fresh destinations. `validation/` records these exclusions; their
figures do not enter the generated tables.

The archive checker reconstructs every source patch, verifies file and driver
identities, recomputes all timings and checks guard records. The repository's
required local checks cover the restored, unchanged library and this report
submission. README/homepage figures remain their separately dated full-suite
snapshots. The preceding report supplies the native Ox comparison; this
investigation adds neither an engine ranking nor a memory-consumption claim.
