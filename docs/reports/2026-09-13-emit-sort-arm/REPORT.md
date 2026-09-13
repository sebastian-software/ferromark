# Emit-point sorting on ARM

**Decision:** Retain the existing unstable sort with the `(position, is_end)`
tuple key. The three alternatives do not meet the adopted tradeoff policy:
limited regressions are acceptable when most documents benefit, but the ARM
screen does not show that distribution. Focused repeats preserve material
counterexamples. This report does not propose a runtime change.

The starting point is PR #314 after its test-only review update,
`39412990e50a6afb6a600d5e29632297c369b508`. The review's x86-64 Callgrind results
motivated these experiments; they are instruction counts with rustc 1.94.1,
whereas this run measures elapsed native render time on Apple M1 Pro with
rustc 1.97.1. It is not an exact reproduction of the review environment.

Three formatted variants isolate the sort algorithm and comparison key:

- `unstable-packed`: keep `sort_unstable_by_key`; use a `u64` key containing
  `(u64::from(position) << 1) | u64::from(is_end)`.
- `stable-packed`: use `sort_by_key` with that packed key.
- `stable-tuple`: use `sort_by_key` with the original tuple key.

The key is computed by the key function; there is no cached-key vector or
additional field in `EmitPoint`. The element remains 16 bytes on this target.
Resolver order and the set of emitted points are unchanged.

## Measured results

<!-- generated-results:start -->
Negative time changes mean less time; positive changes mean more time.

| Variant | >1% faster / within ±1% / >1% slower (103 inputs) | Geometric mean time change |
| --- | ---: | ---: |
| unstable-packed | 5 / 60 / 38 | +0.92% |
| stable-packed | 10 / 64 / 29 | +0.50% |
| stable-tuple | 11 / 67 / 25 | +0.40% |

The following are the median changes across the three longer pairs,
with the full range of pair results in parentheses. A/A uses identical
baseline binaries and is a single pair, not a confidence interval.

| Document | Unstable, packed | Stable, packed | Stable, tuple | A/A |
| --- | ---: | ---: | ---: | ---: |
| Vue Suspense | +1.37% (+1.00…+1.52%) | +0.88% (+0.54…+1.57%) | +1.08% (+0.85…+1.46%) | -0.31% |
| Rust book ch09-02 | +4.60% (+4.42…+5.63%) | +4.51% (+3.69…+5.48%) | -1.12% (-1.63…-0.69%) | +1.31% |
| TypeScript 6.0 | +2.02% (-0.23…+3.15%) | +1.78% (-0.61…+1.80%) | +0.55% (+0.47…+0.76%) | +0.46% |
| Short control | +3.79% (+0.22…+4.26%) | +1.79% (+0.15…+3.68%) | +3.24% (+1.00…+4.96%) | +0.69% |
| Heading control | -1.06% (-6.36…-1.05%) | -0.56% (-1.28…-0.54%) | -0.43% (-7.64…-0.34%) | +0.75% |

All per-input results, including improvements, scaling controls and
counterexamples, are retained in each variant directory. The focused set
was selected after screening; its aggregate is not a workload average.

| Variant | Inputs with changed allocation statistics / 123 | Maximum extra peak requested bytes |
| --- | ---: | ---: |
| unstable-packed | 0 / 123 | 0 |
| stable-packed | 30 / 123 | 188,416 |
| stable-tuple | 30 / 123 | 188,416 |

All four variants produce equal output on all 779 fixed inputs.
The packed unstable variant has identical recorded allocation statistics
on all 123 memory cases. The two stable variants have identical allocation
statistics to each other, including their extra temporary allocations.
<!-- generated-results:end -->

## Method and limits

The screen includes the prior experiment's 103 selected inputs plus 24 scaling
controls: ordered code spans, mixed inline syntax and emphasis at eight sizes.
The 103 inputs include 24 real corpus documents, synthetic controls, explicit
renderer-reuse controls and MDX cases. This is not another 638-document census.
The tables separate the existing inputs from the added scaling controls.
Large synthetic cases retain the normal resource limits; the measurements
describe the complete public operation, not isolated sort complexity.

Screening uses five alternating 20 ms windows after 20 ms warmup per worker.
The focused set includes the three review documents, small/heading/HTML
controls, and the best two and worst two existing cases from each screen.
It receives three fresh process pairs per variant, seven alternating 60 ms
windows after 40 ms warmup. All windows render and drop owned output in batches
of 16; input loading, options, transport and JSON are outside timing. Reuse/MDX
controls retain their explicit input flags. No builds or memory probes run
alongside timing. An identical-baseline-binary A/A run records local noise.

Allocation probes use separate binaries with a counting system allocator.
They perform five deterministic fresh renders after first-use initialization,
check the owned output and verify that requested live bytes return to the
starting level after dropping it. They cover the 123 fresh non-MDX selected
inputs. The figures describe requested live bytes and allocation/reallocation
calls, not RSS, allocator capacity rounding, transient realloc copies or stack
space. Neither stable variant allocates extra heap memory on the 24 selected
real documents; the observed extra allocations occur in controls and larger
paragraphs. Growing input size is therefore part of the check.

The independent output check compares all 779 archived inputs, including MDX
cases and added scaling controls. None of the rejected candidates is a claim
of broader conformance. The original runtime change and its much broader
guards remain documented in the
[membership report](../2026-09-13-suspense-inline-code/REPORT.md).

No new Ox run or public parser ranking is inferred from this experiment.
The README and homepage figures continue to describe their original measured
runs. No unchecked UTF-8 conversion or renderer reuse was added to the public
one-shot API.

## Reproduction

Run `python3 check-evidence.py` from this report directory to validate the
archive and regenerate the tables in check mode. See [REPRODUCE.md](REPRODUCE.md)
for rebuilding and repeating the measurements. An isolated replay rebuilt all
four variants and reproduced all output identities and allocation observations;
[the replay record](replay-verification.json) distinguishes that check from
repeating the timed runs. The corpus provenance and
licenses are recorded in the
[original corpus archive](../2026-09-12-ox-corpus-profiling/licenses/ATTRIBUTION.md).
