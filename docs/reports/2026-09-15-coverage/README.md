# Coverage improvement: first pass

Rust line coverage increased from **90.80% to 93.55%**, with no new exclusions.
The existing measurement scope and 90% CI floor remain unchanged.

| Measurement | Before | After |
| --- | ---: | ---: |
| Covered / measured lines | 12,223 / 13,461 | 12,686 / 13,561 |
| Uncovered lines | 1,238 | 875 |
| Line coverage | 90.80% | 93.55% |
| Function coverage | 92.20% | 96.94% |

[Before](before.json) and [after](after.json) retain the complete per-file
summaries. Measurements use Rust 1.95 and cargo-llvm-cov on macOS ARM64 with
all features; the N-API crate remains excluded as in CI and has separate Node
tests. Other platforms compile different SIMD paths, so this workstation
percentage is not an aggregate across all supported architectures. Additional
instantiations exercised by new tests also change the measured denominator.

## Tests and actual defects

- Exercise the public HTML visitor's typed dispatch across every parsed node
  kind, separately checking directly stored list items and whole-document output.
  This found a missing `visit_highlight` override: the typed visitor emitted
  `mark` while the normal renderer emitted `<mark>mark</mark>`. The override is
  fixed. The existing fast document-rendering path is unchanged.
- Check MDX ESM statement boundaries with nested brackets/parentheses, single and
  double quotes, template literals, escaped quotes, line/block comments, division,
  CR/LF/CRLF, and unterminated constructs. Assert retained source and that the next
  Markdown heading remains a separate node.
- Compare Markdown inside indented JSX against standalone Markdown, including
  UTF-8 source spans, nested JSX, attribute expressions, tables/captions, lists,
  math, definitions, footnotes with document-level labels, and writing extensions.
  Two-space and tab indentation must preserve structure and the original source
  slices after removing only wrapper indentation.
- Verify span containment/merge boundaries and byte offsets for Unicode text.
- Exercise byte-class construction with varied ASCII sets and the non-ASCII
  rejection invariant. Existing fixed classes are evaluated at compile time;
  the new tests validate construction beyond those fixed sets.

## Remaining work and exclusion policy

99% is an aspiration, not the current result. The largest remaining gaps are
source mapping, transformed table cells, definition lists, renderer hooks, math,
and TOC traversal. These are useful behavior tests, so excluding these modules
would hide meaningful gaps. Work through them in separate, reviewable changes.

A further uncovered behavior is recorded explicitly: a footnote defined only
inside a tab-indented MDX component is parsed as a definition, but its local
`[^label]` use stays literal unless a document-level label already exists.
This is separate from CommonMark link definitions. The span test supplies a
root label to isolate span mapping; it does not claim that nested MDX footnote
collection is fixed. A follow-up should define the intended MDX footnote scope,
then add a blocking behavior regression and implement collection accordingly.

No new ignore attributes, file filters, or lowered gates were added. An exclusion
must have a local reason and a record here: for example, an invariant proven
unreachable from valid inputs. Platform-specific code should instead be covered
on the corresponding CI architecture. Const-evaluated construction and defensive
error handling are not automatically reasons to exclude code. Keep the full
measurement scope visible beside any public percentage; line coverage does not
prove specification completeness, as the [oracle analysis](../2026-09-15-suite-regression/README.md#why-the-conformance-tests-were-green)
illustrates.

## Reproduction

```sh
cargo llvm-cov --workspace --exclude ferromark-node --all-features --locked --no-report -- --test-threads=1
cargo llvm-cov report --json --output-path coverage.json
cargo llvm-cov report --fail-under-lines 90
```

The full workspace passes 826 tests, including eight new tests and the regression
that failed before the typed visitor fix. Formatting, Clippy with warnings denied,
and all workspace benchmark builds pass. The CONTRIBUTING/CI contract also passes.
