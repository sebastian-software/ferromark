# Retained refactor: inline helper file boundaries

Baseline: `61eafed`. This bounded change replaces `inline_helpers.rs` (493 lines)
with named owners without changing algorithms or parser state:

| Owner | Contents | Lines |
| --- | --- | ---: |
| `inline/image.rs` | Image syntax, alternative-text flattening | 153 |
| `delimiters.rs` | Marker runs, closed-code skipping, cached closer lookup, balanced brackets | 153 |
| `delimiters/tests.rs` | Existing scalar oracle and scanner comparisons | 170 |
| `inline.rs` | Existing small node-capacity and text-construction helpers | 342 total |

The image parser is now visible only inside the inline module; the capacity
helper is private. Other internal visibility is unchanged. Delimiter scans stay
at parser level because math, MDX and several inline constructs share them.
No traits, crates, runtime dispatch, extra allocation or public API are introduced.
The change adds a small amount of module documentation/import overhead rather than
reducing total source lines; its value is discoverability and narrower ownership.

## Correctness and validation

All **828 workspace tests**, Clippy with warnings denied, `cargo fmt --all --check`
and `cargo bench --workspace --no-run --locked` pass. The moved scalar-oracle test
body is unchanged after removing its former inline-module indentation. No expected
outputs, official specification fixtures, or coverage exclusions are changed.
The core test suite covers images, links, math, code spans, MDX, inline notes,
malformed input, nesting and source positions. Node bindings are unchanged.

The 57-document broad corpus and thirteen authored diagnostics are checked for
exact HTML, debug AST (including source spans), and root-child counts before and
after timing, across revisions and fresh/reused lifecycles. Three new authored
probes exercise image alt text/fallbacks, balanced brackets with tighter-binding
syntax, and many unclosed delimiters. They supplement existing conformance tests
and are not independent standards oracles.

## Retention decision

Retain the named boundaries. The broad aggregate is **−0.19% fresh / −0.31%
reused**; the thirteen-diagnostic aggregate is **+0.26% / −0.29%**. All exact-output
gates pass. Treat these small changes as effectively neutral on this workstation,
not as a speedup claim. The directly affected image, balanced-bracket and unclosed-
delimiter probes in the initial run stay within 0.21% of baseline.

The initial fresh footnote probe has mixed rounds (+2.72%, +2.72%, −1.94%), so it
was checked again with image/bracket controls, three rounds, nine alternating pairs
and 80 ms windows. Its confirmation median is **−0.08% fresh / +0.10% reused**;
the initial slowdown does not repeat. Confirmation image-alt results are +0.54% /
+0.07%, brackets −0.35% / +0.31%, and the combined confirmation workload +0.07% /
+0.12%. Both original and follow-up samples are retained; no rounds are dropped.
There is no demonstrated material slowdown warranting rejection of this bounded
organizational change. The follow-up was prompted by that specific unresolved
outlier rather than repeated benchmarking to select a favorable result.

## Method and reproduction

Broad timing uses three rounds, seven alternating pairs and 50 ms windows, with
20 ms warmups. It measures rotating full-profile batches; output checks cover each
document, but aggregate-only timing cannot establish individual broad-document
regressions. Targeted timing covers each of thirteen inputs plus rotating profile
batches, with three rounds, five pairs, 40 ms windows and 20 ms warmups.
All tests and builds finished before timing.

Both builds use Rust 1.95, generic CPU, fat LTO, the default allocator, identical
worker source and unchanged locked registry versions. Each archived run includes
compiler/binary/source hashes, exact outputs, raw samples, options, corpus provenance
and host observations. Workstation results do not establish other architectures'
performance. See [tables](tables.md) and the
[decision](../../decisions/2026-09-15-refactoring.md#4-inline-helper-file-boundaries--retained).

Apply [candidate.patch](candidate.patch) to the baseline and use the
[harness instructions](../../../benchmarks/refactoring/README.md) to reproduce.
The broad corpus retains its [licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).
