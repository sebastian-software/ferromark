# Retained refactor: inline scanner backend files

Baseline: `1063140`. The former 622-line `inline/scan.rs` now separates CPU
implementations while preserving the existing internal entry points:

| File | Responsibility | Lines |
| --- | --- | ---: |
| `scan.rs` | Common classification, option tables, CPU detection and dispatch | 215 |
| `scan/scalar.rs` | Portable loops and short-vector fallback | 89 |
| `scan/neon.rs` | AArch64 NEON implementation | 80 |
| `scan/x86.rs` | SSSE3 and AVX2 implementations | 166 |
| `scan/tests.rs` | Existing differential scanner tests | 105 |

The backend modules are compiled only for their architecture. No new dispatch
layer, traits, allocations or public API are added. Algorithms, table values,
inlining and target-feature attributes, CPU detection order, and fallback behavior
remain the same. Backend functions are visible only inside `scan`. This adds module
imports/documentation rather than reducing total code; each CPU implementation
can now be read without unrelated architecture conditions and intrinsics.

## Validation

All **828 native workspace tests**, Clippy with warnings denied, formatting, and
workspace benchmark builds pass. The moved scanner tests are unchanged except for
the scalar import. Existing tests compare all 256 byte values, offsets, prefix/vector
boundaries and overlapping tails; the existing marker-scan tests cover all sixteen
extension masks. Specification fixtures, expected outputs and coverage exclusions
are unchanged.

Additional compile/lint checks pass:

```sh
cargo clippy -p ferromark_parser --all-targets --all-features --locked --target x86_64-apple-darwin -- -D warnings
cargo clippy -p ferromark_parser --lib --all-features --locked --target wasm32-unknown-unknown -- -D warnings
```

Only AArch64 executes tests and timings here. x86 and WebAssembly checks establish
compilation/lint compatibility, not execution or performance on those targets.

### Existing x86 lint defect

The first x86 check rejected four unsafe wrapper declarations under the repository's
unsafe-code lint and four implicit unsafe calls under Rust 2024. The unchanged
baseline reproduces all eight failures. Add narrowly scoped unsafe-code allowances
to those four intrinsic wrappers, explicit unsafe blocks, and caller feature
requirements. The unsafe-operation lint remains enabled; runtime feature detection
and bounds checks remain intact. The [baseline failure](baseline-x86.txt) and
[successful candidate check](candidate-x86.txt) are retained. This is a lint fix,
not an instruction-selection or Markdown-output change. Native checks ran before
the x86-only correction; the final measured build includes that correction.

## Retention decision

Retain the backend boundaries and x86 lint correction. The [generated tables](tables.md)
show **+0.21% fresh / −0.07% reused** over the broad workload and **+0.08% / +0.52%**
over the thirteen diagnostics. Exact HTML/AST gates pass throughout.
Broad fresh rounds are +0.21%, +0.17%, +0.51%; reused rounds −0.43%, +0.28%, −0.07%.

The broad result is effectively stable in this workstation study, but the change
is not universally free: synthetic root references are +1.66% fresh / +1.29% reused,
reused tables +1.41%, and reused footnotes +2.01%. These small costs are accepted for
clearer backend ownership and the verified x86 gate fix. They are not relabeled as
speedups or removed from the record. Individual rounds fluctuate more (for example,
one fresh JSX round +2.96%); the tables report the median of all three rounds.
There is no claim of performance improvement, or of x86/portable runtime parity
based on ARM-only measurements.

## Measurement method

Exact HTML, debug AST including spans, and root-child counts are compared before
and after timing for all 57 broad documents and thirteen authored diagnostics,
across revisions and fresh/reused lifecycles. Timed checksums must match too.
The broad corpus uses rotating profile batches: three rounds, seven alternating
pairs and 50 ms windows. Each document is checked for output equality, but this
aggregate-only timing cannot establish individual broad-document regressions.
Diagnostics are timed individually and in profile batches, with three rounds,
five alternating pairs and 40 ms windows. Warmups are 20 ms throughout.
All builds and tests completed before timing.

Both workers use Rust 1.95, generic CPU, fat LTO, the default allocator, identical
worker source and unchanged locked registry versions. Run folders retain exact
outputs, raw samples, build hashes/settings, corpus provenance and host observations.
Apply [candidate.patch](candidate.patch) to the baseline and use the
[harness](../../../benchmarks/refactoring/README.md) to reproduce. See the
[decision](../../decisions/2026-09-15-refactoring.md) for scope.
The corpus retains its [licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md)
and [Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).
