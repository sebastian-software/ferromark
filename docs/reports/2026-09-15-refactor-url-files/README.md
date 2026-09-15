# Retained refactor: URL escaping file boundary

Baseline: `c7d55f1`. Keep all three existing escaping entry points together, and move URL encoding
and IPv6 recognition into a child module. Algorithms, replacements, allocation
behavior, scan/copy primitives and optimization attributes stay unchanged. The URL
entry point retains its original owner and direct calls; no wrapper is added and
production visibility is not widened.

| File | Responsibility | Lines |
| --- | --- | ---: |
| `escape.rs` | Existing entry points, HTML text/attributes, shared primitives | 321 (formerly 496) |
| `escape/url.rs` | URL replacement, percent encoding and IPv6 recognition | 189 |
| `escape/tests.rs` | Mixed text/URL/attribute differential checks | 216 |
| `escape/url/tests.rs` | URL-only regressions and existing scalar reference | 70 |

The four URL-specific tests and scalar reference move with their implementation.
The reference is accessible to mixed tests only under `cfg(test)`. All thirteen
escaping test function bodies and expected outputs are preserved verbatim; imports
and the test reference's visibility are adjusted. Specification fixtures and
coverage exclusions are unchanged. This adds small module/documentation overhead;
it improves ownership rather than reducing total lines.

## Checks

All **828 native workspace tests**, Clippy with warnings denied, formatting and
workspace benchmark builds pass for the final entry-point-preserving variant. Additional checks cover renderer
all-targets Clippy for `x86_64-apple-darwin` and library Clippy for
`wasm32-unknown-unknown`, with all features, locked dependencies and `-D warnings`.
These are compile/lint checks; only AArch64 executes tests and timings here.

Exact HTML, debug AST including spans, and root-child counts are checked before
and after timing, across both revisions and fresh/reused lifecycles. Timed checksums
must also match. The corpus contains the existing 57 broad documents and fifteen
authored diagnostics. Two new diagnostics exercise Unicode percent encoding,
existing percent escapes, ampersands, valid IPv6 with ports/userinfo, invalid IPv6,
and brackets in paths/queries. They supplement the existing differential and
specification tests, not an independent standards oracle.

## Rejected initial candidate

The first candidate moved the URL entry point into the child too, re-exporting it
from the old path. Tests and exact-output checks passed; its broad aggregate was
−0.13% fresh / +0.49% reused. However, unclosed delimiters consistently took about
5% longer (fresh +4.98%, +5.12%, +5.04%; reused +4.97%, +5.44%, +5.05%). Reject that
variant despite its reassuring broad average. Its [patch](rejected-full-move.patch)
and `url-files-*` results remain archived. No unfavorable rounds are discarded.

The revised boundary keeps the existing entry point in the parent and moves only
its detailed encoding/recognition helpers. This is a coherent organizational
alternative with no extra call, not an inlining change. Compiler layout or code
generation may explain timing changes from file moves, but a specific mechanism
has not been established.

A focused check of the revised variant uses unclosed delimiters, Unicode URLs and
IPv6 URLs: three rounds, nine alternating pairs, 80 ms windows and 20 ms warmups.
Unclosed delimiters measure **−0.08% fresh / −0.16% reused**; Unicode −0.12% /
+0.05%; IPv6 +0.07% / +0.04%. The original slowdown does not recur in this variant.
See `url-entry-probe` for all samples. The revised variant then goes through the
complete broad and fifteen-case diagnostic study described below.

## Retention decision

Retain the entry-point-preserving variant, not the full move. The final broad
aggregate is **+0.50% fresh / +0.13% reused**; the fifteen-diagnostic aggregate is
**−0.09% / −0.22%**. Exact-output gates pass throughout. In the full final diagnostic
run, unclosed delimiters are −0.41% / −0.22%, Unicode URLs −0.06% / +0.04%, and IPv6
URLs +0.21% / +0.08%. The earlier 5% slowdown is absent in both the focused probe
and full final diagnostic run. Small workload-dependent costs remain; these
workstation results are treated as broadly neutral, not a universal speedup.
See [generated timing tables](tables.md) for every individual diagnostic median.

A separate direct comparison of **all three file-boundary refactors**, from
`61eafed` to the final candidate, measures **−0.34% fresh / −0.26% reused** on the
57-document broad corpus, with exact output/AST preserved. It uses the same
three-round aggregate method; results are not multiplied across separate studies.
See `file-boundaries-combined` for raw evidence. This closes the planned inline
helpers, scanner backends and URL escaping file-boundary sequence.

## Measurement method

Broad runs time rotating full-profile batches: three rounds, seven alternating
pairs and 50 ms windows. Each document is still checked for output equality, but
aggregate-only timing cannot establish individual broad-document regressions.
Diagnostics are timed individually and in profile batches, using three rounds,
five alternating pairs and 40 ms windows. Warmups are 20 ms throughout. Builds and
checks finish before timing. Both workers use Rust 1.95, generic CPU, fat LTO,
the default allocator, identical worker source and unchanged locked registry versions.

Run folders retain exact outputs, raw samples, build hashes/settings, input
provenance and host observations. Apply [candidate.patch](candidate.patch) to the
baseline and use the [harness](../../../benchmarks/refactoring/README.md) to reproduce.
See the [decision](../../decisions/2026-09-15-refactoring.md) for scope. The broad
corpus retains its [licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).
