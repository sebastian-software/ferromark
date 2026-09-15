# Container-reference fix: full-corpus impact

## Result

The complete 57-document workload does **not** show a general 5% regression.
Positive values below mean slower; each document occurs once per corpus pass.

| Workload | Fresh parser/renderer | Reused arena/renderer |
| --- | ---: | ---: |
| All 57 documents | −0.08% | +0.90% |
| CommonMark: 17 documents | −0.14% | −0.93% |
| GFM shared subset: 40 documents | +0.53% | +2.18% |

Complete-suite round ranges are −0.18% to +0.06% (fresh) and +0.55% to +1.09%
(reused). Treat the fresh result as unchanged. The retained path has a small,
repeatable cost on this machine. The TypeScript Advanced Types document is a
real outlier: +10.80% fresh, +10.16% reused. Indented TypeScript index/mapped type
signatures such as `[key: string]: T` inside code fences are false-positive
candidates for the structural reference pass. An optimization should target
that unnecessary work and retain correct definitions inside actual containers.

The earlier [focused reference probes](../2026-09-15-container-references/README.md)
remain valid: dense root-reference documents cost about 5–6% more; inputs with
many corrected container references cost substantially more. Those targeted
results are not the full-suite aggregate. This corpus does not replace the
correctness probes and cannot establish the cost on every Markdown workload.

## Method and evidence

- Before: `0b529ac39236219906c30ee905d91183532e65a1`.
- After: `196355392a2dab7e2f887f2940b21423a23eabc7` (container fix plus the new
  convenience facade; the worker uses the existing lower-level API).
- Same worker, Rust 1.95, generic CPU, fat LTO, one codegen unit, locked registry
  dependencies, and default allocator; build hashes are in [builds.json](builds.json).
- Three rounds, each with fresh worker processes, shuffled jobs, five alternating
  before/after pairs, 20 ms warmup, and 40 ms timing windows. No task builds or
  tests ran during timing. Host telemetry limitations are retained in the JSON.
- All 57 documents match in exact HTML, debug AST, and root-child count across
  revisions, lifecycles, and rounds. Timed checksums and post-timing checks pass.
- The full-suite ratio sums separately measured CommonMark and GFM batch times;
  each batch rotates its documents once per iteration. This weights by elapsed
  time, rather than averaging document percentages.
- This is the complete native comparison **input corpus**, measured directly
  before/after. It is not a rerun of the six-engine leaderboard, which uses a
  different compiler/allocator setup. Published competitor figures are unchanged.

[Tables](tables.md), [summary](summary.json), [raw timing](samples.json.gz),
[exact outputs and ASTs](verification.json.gz), [frozen corpus](corpus.json.gz),
and [harness hashes](harness-sha256.json) retain the evidence. Reproduce with
[the runner](../../../benchmarks/suite-regression/README.md). Corpus licensing:
[licenses](../../../benchmarks/broad-comparison/licenses/ATTRIBUTION.md),
[Wikipedia attribution](../../../benchmarks/broad-comparison/WIKIPEDIA-ATTRIBUTION.md).

## Why the conformance tests were green

The CommonMark fixture says definitions in lists and quotes have document scope,
but its concrete example at that point uses a quote only. The 652 official
examples are not exhaustive tests of all combinations in the prose specification.

The existing cmark regression test reads 106 frozen reference outputs. That
corpus includes lists and reference links, but not definitions inside list items.
The cmark executable can only detect differences in inputs it is given.

The later line-comment oracle **did detect** the list-definition mismatch. Commit
`40047f7` explicitly accepted `list-definition-title` plus its two baseline probes
as known mismatches, and the old README documented the limitation. The oracle was
a manual diagnostic; CI did not run its live cmark comparison. This was a known
limitation allowed by the acceptance criteria, not an unseen failure in a test
that should have been red. The fix in `bd75924` removed the semantic allowance
and added ordinary Rust regression tests that run in the CI matrix.

Future oracle discoveries need a blocking regression for intended supported
behavior. Specification examples, independent-oracle comparisons, and coverage
answer different questions; a high line percentage cannot substitute for an
assertion about the right output.
