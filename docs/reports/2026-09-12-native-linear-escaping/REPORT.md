# Native Linux follow-up for linear HTML escaping

Date: September 12, 2026.

## Why the implementation was reopened

The initial [ARM report](../2026-09-12-linear-html-escaping/REPORT.md) demonstrated
that bounded searching removes quadratic quote-heavy escaping. Its portable
integration was not ready for adoption: the PR's Linux performance job reported
an 18.7% direct plain-text escaping regression and several 3–5% document costs.
The job was green because its failure threshold is 20%. Passing that gate did
not satisfy the stricter acceptance policy for this change. The original
[CI run](https://github.com/sebastian-software/ferromark/actions/runs/34710137604)
and its complete performance log are retained in this archive.

Native Linux experiments therefore compare every candidate against merged main
`e731dc8162828e0ac2df30b7567107f560159ff8`, including pure escaping controls that
ordinary Markdown can bypass. No positive and negative changes are averaged
away using assumed workload frequencies. A/A controls disclose identical-binary
variation; they are not subtracted from the candidate results.

## Experiments

1. Round 1 evaluates the initial portable implementation, different unmatched
   prefix/window sizes, and thresholds that retain the original search below
   a fixed size. It ran on AMD EPYC 9V74. The original portable implementation's
   plain escaping regression reproduced, while the broad document slowdown from
   the first CI run did not. The 8 KiB threshold looked better in this screen,
   but was not accepted on that evidence alone.
2. Round 2 retains the 8 KiB threshold control and tries two cached-cursor
   integrations for whole large writes. It ran on AMD EPYC 7763. Both cache
   integrations repeatedly slowed ordinary controls; the threshold integration
   also developed repeated costs in this CPU/driver context. All three were
   rejected. The differing CPUs and driver controls preclude attributing the
   difference to one cause.
3. Round 3 tests a joint x86 search using SSE2 or runtime-selected AVX2. It has
   two independent runner jobs, three fresh pairs per candidate, the full 93
   ordinary controls, 27 text/code stress cases, 14 attribute stress cases, and
   25 direct API controls. `masked-direct` uses the joint result directly.
   `masked-twin` deliberately retains a redundant bounded quote probe to test
   integration/code-shape effects; that experiment is not a proposed algorithm.

Both round-3 jobs used AMD EPYC 7763. Both variants are rejected: the direct
scanner improves pure escaping, but the complete renderer has repeated
multi-percent costs, including roughly 28–34% on long plain attribute info.
The direct variant also slows sparse-ampersand rendering and several HTML
controls. These losses persist across both jobs and are not explained away by
the favorable direct API measurements.

All Linux rounds use Rust 1.98.1. The exact scripts, CPUs, source snapshots,
build logs, guards and observations are archived. [RESULTS.md](RESULTS.md)
contains every case and pair, including all unfavorable results. Direct capacity
controls added after round 1 are separate rows. Round 3 also adds plain attribute
controls. Earlier rounds are screens, not complete acceptance evidence.

## Joint x86 search

The search recognizes the complete four- or five-byte escape set at once.
Clearing bit 2 pairs `"` with `&`; clearing bit 1 pairs `<` with `>`. Comparisons
retain every other bit, so unrelated bytes and non-ASCII bytes cannot match.
Attribute mode adds a comparison for `'`. The earliest set bit selects the first
escape in input order.

The first vector is checked immediately; subsequent scans use four-vector groups,
then single vectors and an overlapping final vector. Every load fits entirely
inside the supplied slice. The overlapping bytes have already been checked.
SSE2 is guaranteed on x86-64; AVX2 executes only after Rust's runtime CPU/OS
feature check. Inputs at or below the existing 128-byte short threshold retain
the shared byte-set search. Repeated escape calls advance past the found byte,
so scanning a complete write is linear, including quote-heavy input.

The ARM search and output loops remain the previous measured implementation.
`arm-code.json` records that both x86 candidates produce exactly the original
ARM benchmark's 631,900-byte machine-code section, using its unchanged driver,
source path, toolchain and build working directory. This establishes code
identity for that ARM build; it is not a native x86 performance result.
Other architectures retain bounded growing windows. Their native throughput
has not been measured here.

The replacement bytes, public APIs, output ownership and capacity policy remain
unchanged. The x86 specialization introduces unsafe intrinsics with explicit
feature/load bounds. Native tests call both SSE2 and AVX2 against scalar oracles
for all byte values, vector-boundary lengths, and every alignment modulo 32.
Large mixed-byte writes test complete output as well. Work-count tests exercise
the real fenced-code path and attribute writer; release code has no counters.

## Final scope decision

The user clarified that ARM throughput is the primary target. The production
PR therefore adopts only the measured ARM64/NEON escaping change. No x86
variant in this report is adopted; x86 retains the merged implementation.
Hosted-runner timings remain diagnostic, with uncontrolled host contention.
No further native tuning rounds were started after that clarification.

Round 4 was already running and completed. It screened separate one-shot text
probes and outlined text/attribute writers. Outlining the attribute writer
removed the long plain-attribute cost in that screen; other integrations still
had counterexamples. These are limited screens, not adoption measurements.
The same binaries were sampled with `perf` only after all timings completed.
All nine software CPU-clock profiles succeeded. On the plain-attribute case,
the direct SIMD variant moved CPU time into `HtmlWriter::code_block_start`;
the combined outlined variant restored a distribution close to the baseline.
Instruction annotations, assembly, profile data, setup logs and metadata are
preserved under `measurements/round-4-1/native-profile`. This supports an
integration effect; it does not establish a universal compiler-level cause.

## Interpretation

The large quote-heavy cases diagnose an algorithmic defect. Their gains are
synthetic stress results and do not imply equivalent gains on normal documents.
No new memory claim or ranking against Ox is made. The README and homepage keep
the existing dated full engine comparison; this experiment is a before/after
Ferromark comparison.

Runner scheduling, CPU frequency and core affinity are uncontrolled. Repeated
pair values are observations, not confidence intervals or proof that all inputs
on all machines improve. AVX2 throughput is measured; SSE2 throughput on a
non-AVX2 machine is not. See [REPRODUCE.md](REPRODUCE.md) for the native timing
boundary, allocation flags, reconstruction and archive checks.
