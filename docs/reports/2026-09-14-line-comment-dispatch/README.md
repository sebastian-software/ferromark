# Reuse block dispatch for line comments

The earlier **+3.2%** measured enabling line comments on a **4,125-byte document
without comments**. It was the cost of looking for absent syntax, approximately
36 ns per document in that run, not the cost of removing a comment. This
follow-up integrates that recognition into the existing line-prefix scans.

Baseline: `78d7e21`, including the previously retained feature-scan optimizations.
The candidate changes neither syntax nor runtime options. All attempts are
recorded in [ATTEMPTS.md](ATTEMPTS.md); both patches apply against this baseline.

## Implementation

Block dispatch and paragraph continuation already find the first non-space/tab
byte. They now consult comment eligibility only when that byte is `/`.
Ordinary text avoids the extra flag, physical-line, and indentation checks.
Other physical-line callers also reject prefixes other than space or slash
before inspecting the preceding newline and indentation.

The mapped eligibility set remains authoritative: dedenting `> // literal`
must not turn it into a comment. Eligible lines copied through quotes, lists,
definitions, and footnotes retain their slash prefix. The regression fixture
checks all four containers with zero to three spaces and LF/CRLF/CR endings,
including the original span of surviving strong text. Comments are still
skipped before enforcing the nesting limit.

A context-blind source prefilter would change behavior inside code and raw
HTML. Even an eligible comment must be recognized and its line ending found.
Comments within a paragraph can require joining surviving slices and mapping
inline positions back to the original source. Producing no comment node or
HTML saves downstream work; it does not make all input processing disappear.

## Remaining off/on cost

Each binary was measured separately with three process rounds of three 50 ms
off/on pairs, using identical HTML and AST in both configurations. Percentages
are medians of paired ratios, not ratios of independently rounded time medians.

| Plain input | Parser overhead before | Parser overhead after | Complete reused overhead before | Complete reused overhead after |
| --- | ---: | ---: | ---: | ---: |
| 330 B | +5.0% | +0.5% | +3.0% | +0.5% |
| 4,125 B | +3.6% | +1.0% | +2.6% | +0.5% |
| 65,670 B | +3.8% | +0.9% | +2.5% | +0.4% |

The newly measured +3.6% reproduces the earlier +3.2% residual approximately.
For the new 4,125-byte parser result, round medians range from +0.3% to +1.3%
and individual pairs from −0.2% to +1.5%. At 65,670 B, one round is +3.0% despite
the overall +0.9% median. These small residuals are workload- and
measurement-sensitive; this is not a claim that comments are universally free.
All [before](off-on-baseline/summary.csv) and
[after](off-on-candidate/summary.csv) samples, including outliers, are retained.

## Same-option before/after results

Three process rounds, three interleaved 40 ms pairs per case/stage. Lower times
are better. Savings use the paired median ratio.

| Line comments enabled | Bytes | Parser before → after | Parser time saved |
| --- | ---: | ---: | ---: |
| Plain prose | 330 | 0.120 → 0.112 µs | 6.3% |
| Plain prose | 4,125 | 1.220 → 1.161 µs | 4.8% |
| Plain prose | 65,670 | 19.088 → 18.335 µs | 4.2% |
| Repeated active comments | 343 | 0.789 → 0.785 µs | 0.3% |
| Repeated active comments | 4,116 | 8.930 → 8.903 µs | 0.2% |
| Repeated active comments | 65,562 | 143.263 → 141.437 µs | 1.0% |

Active probes are effectively unchanged at the smaller sizes. Their different
paragraph structure prevents treating the active/plain time difference as a
per-comment price. Complete reused processing of the 4,125-byte plain probe
changes from 1.765 to 1.707 µs, a 3.3% paired saving.

Crucially, the lower off/on tax does not come from slowing the disabled path:
same-option parser controls with comments disabled improve by 1.8–2.2% across
the three plain sizes. Slash-heavy near-misses remain around 1.00×.

| Mixed configuration, 13 documents each | Parse | Fresh complete | Reused complete | Render prebuilt AST |
| --- | ---: | ---: | ---: | ---: |
| Docs recipe plus definition lists | 1.021× | 1.011× | 1.014× | 1.003× |
| Strict CommonMark controls | 1.019× | 1.010× | 1.015× | 1.004× |

Ratios are geometric means of per-document median baseline/candidate ratios;
above 1 favors the candidate. Documents have equal weight, range from 37 to
113,609 bytes, and include correlated Wikipedia views. This is not a population
estimate. Small losses remain: the 64 KiB active-comment render control is
0.994×; CommonMark incident fresh processing is 0.996×; slash-heavy 64 KiB reused
processing is 0.997×. Renderer code and verified ASTs are unchanged. Code layout
and measurement effects can move controls; the exact cause is unproven.
[All 148 rows](final-C2/summary.csv) include per-round ratios and full pair ranges.

## Validation and replay

The frozen final candidate passes exact HTML, full AST Debug including spans,
and child-count comparison for all 75 corpus cases across four lifecycles.
The measured subset contains 37 cases and 1,332 paired observations. Every
timed worker is checked before and after timing and each sample has an output
checksum. Another 400 generated documents under three configurations pass
1,200 differential comparisons. These are regression checks against the
baseline, not a new independent specification claim.

The final workspace passes **771 tests in 54 suites**. Formatting, strict
workspace Clippy, benchmark builds, and the three harness tests pass. See
[commands and logs](checks/commands.json), [full corpus gate](all-verified/result.json),
and [generated checks](generated/result.json). The initial attempted verification
command used an unsupported `--verify-only` argument; the retained
[verification-only driver](checks/verify_corpus.py) invokes the existing gate
directly. The failed invocation is retained alongside the successful checks.

[All 373 frozen core files](measured-source.json) match the retained source,
including its regression fixture. Worker, source, binary, and lock hashes,
compiler settings, and build logs are archived under [builds](builds/C2/build.json).
Both binaries use Rust 1.95, generic AArch64, opt-level 3, fat LTO, one codegen
unit, and unchanged registry dependency versions/checksums. Timings ran serially
on the shared macOS arm64 workstation, on AC power, without CPU pinning or
concurrent builds. CPU-model and thermal probes were unavailable in the sandbox.

The [existing harness](../../../benchmarks/feature-scan-optimization/README.md)
recreates temporary config paths from `runtime_options`. Input bytes and origins
are preserved in the compressed corpora; the
[broad corpus documentation and licenses](../../../benchmarks/broad-comparison/README.md)
remain authoritative for third-party inputs. Large raw JSON files are gzip
compressed without discarding samples or verification outputs.

To replay C2, create an isolated checkout of `78d7e21`, apply
`patches/C2-fused-dispatch.patch`, and run from the repository root:

```sh
python3 benchmarks/feature-scan-optimization/prepare.py \
  --baseline-revision 78d7e21 --candidate-path /path/to/patched-checkout \
  --out /private/tmp/line-comment-replay-build
python3 benchmarks/feature-scan-optimization/run.py \
  /private/tmp/line-comment-replay-build \
  docs/reports/2026-09-14-line-comment-dispatch/final-C2/corpus.json.gz \
  /private/tmp/line-comment-replay-results \
  --rounds 3 --pairs 3 --window-ms 40 \
  --modes parse reuse fresh render --filter 'line_comments|slash-paths|mixed--'
```

The off/on corpora under `off-on-*` can be decompressed and replayed with
`benchmarks/runtime-profiles/run.py` against the corresponding
`runtime-views/baseline` or `runtime-views/candidate` directories, using
`--rounds 3 --pairs 3 --window-ms 50`.
