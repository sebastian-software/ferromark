# SIMD round: a single probe for link components

**Follow-up:** The [subsequent optimization rounds](../2026-09-14-optimization-rounds/README.md)
repeat the old binaries, vary LTO, and evaluate a combined implementation. The
single-probe source change is now promoted locally; the original decision and
measurements below remain the record of this first study.

**Original decision: keep the candidate on local branch `codex/simd-link-probe`; keep
`main`'s parser unchanged.** The candidate improves parsing and typical complete
Markdown processing, but two large render-only controls regress reproducibly.
That prevents an unconditional promotion under the port discipline.

The existing core already uses extensive SIMD/SWAR for markers, lines, and
escaping. This experiment adds one `memchr2` probe for backslash or ampersand at
the start of link URL/title unescaping. A component without either byte returns
its original borrowed slice immediately. A component with a candidate uses the
original scalar loop from that position. Reference definitions share the path.
No new library unsafe code or dependency is needed.

## Full-corpus result

Measured on an Apple M1 Pro, with 57 frozen broad cases plus 15 separate
diagnostics. Figures below are geometric means of per-document median paired
ratios from three rounds of three pairs. A ratio above 1 favors the candidate.

| Broad group | Cases | Fresh processing | Reused processing | Parse only |
| --- | ---: | ---: | ---: | ---: |
| All broad cases | 57 | 1.034× | 1.034× | 1.049× |
| Comments | 12 | 1.017× | 1.007× | 1.012× |
| Encyclopedia views with Markdown links | 12 | 1.096× | 1.106× | 1.146× |
| Plain prose | 4 | 1.001× | 1.002× | 1.005× |
| Technical documents | 22 | 1.023× | 1.024× | 1.036× |

The overall ratio corresponds to approximately **3.4% more processing throughput**
or **3.3% less time**, with a larger parsing gain. Most of the benefit comes from
links, not prose scanning. The linked Volcano opening paragraph improved by
13.4% in reused processing; Vue's slots document by 11.3%. The 310-byte table
comment stayed essentially unchanged. The prior table gap against Ferromark v1
is therefore not resolved by this patch. These results compare v2 with v2;
Ferromark v1 was not remeasured in this round.

Synthetic long clean links improved by 25.8% in reused processing, Unicode links
by 22.6%, long references by 16.4%, and the MDX link diagnostic by 6.6%. Those
diagnostics are excluded from the broad averages. The full run's largest broad
reused-processing loss was about 1.3%; a targeted follow-up put Advanced Types
at about 1.7% slower. Small differences should be read with their round ranges.

## Why the candidate remains experimental

The aggregate render-only control was 1.000×, but that hides individual losses.
Although renderer source and verified AST contents are unchanged, the changed
combined executable rendered the Chess article body more slowly, as well as
TypeScript's 5.0 release notes. A longer follow-up used three rounds of five
100 ms pairs on nine selected cases:

| Case | Complete reused processing | Render only |
| --- | ---: | ---: |
| Chess article body | 1.087× | 0.933× |
| TypeScript 5.0 | 0.995× | 0.945× |
| Table comment | 1.000× | 0.979× |
| Long clean links diagnostic | 1.259× | 1.008× |

The Chess render ratio corresponds to roughly **7.2% more time**; TypeScript's
to **5.8% more time**. The TypeScript end-to-end result also varied between
rounds, from 0.941× to 1.029×. These are retained observations, not excluded
outliers.

A separate A/A check ran the **same baseline executable** in both worker roles
on six render cases, again with three rounds of five 100 ms pairs. Its median
ratios were 0.993–1.012×, including 1.012× for Chess. This argues against dismissing
the larger cross-binary render differences as ordinary paired-run noise. Fat-LTO
code generation/layout or process/AST memory placement remain plausible causes;
this round does not establish which one is responsible. Equal allocation
counters do not establish equal addresses or access costs.

The candidate is promising for parsing and full processing, but it is not a
demonstrated win across every exposed stage. The next acceptance step is to
explain or remove those render-only differences, then repeat the affected cases
and broad matrix. The current branch is a reviewable prototype for that work.

## Correctness, memory, and reproduction

- All 72 cases have byte-identical AST Debug and HTML output across all four
  stages and both versions. No compatibility exclusions or canonicalization.
- All 216 fresh/reuse/parse allocation comparisons match, including call counts,
  requested sizes, deallocations/reallocations, and arena capacity.
- The candidate passes 628 tests, unchanged snapshots/conformance, strict Clippy,
  formatting, and all seven Criterion benchmark builds.
- Three discarded scanner variants and the selected single-probe screen retain
  their patches and raw measurements under `screens/`.

See [the method and experiment history](METHOD.md), [all derived tables](RESULTS.md),
[the remaining candidate inventory](INVENTORY.md), and
[the reproducible harness](../../../benchmarks/simd-round/README.md).
`candidate.patch` applies to the pinned baseline
`ae963d425a4c15841ced5f7359a4b9adc47ca931`; `source-verification.json` verifies
every archived core file against that commit. The benchmark files on `main`
preserve the study independently of the prototype branch.

The measurements describe this AArch64 host and build configuration. An x86-64
portability/performance check remains separate work.
