# Output agreement before timing

All 57 original inputs are retained byte for byte, with original provenance.
Each engine's fresh/reuse output must agree exactly. Cross-engine agreement
uses the unchanged conservative serialization comparator: entity spelling,
HTML flow whitespace, void-tag/boolean-attribute spelling, table alignment,
and literal Unicode versus UTF-8 URL spelling. It retains content, code
whitespace, destinations/fragments, IDs, and CSS classes.

| Engine versus v2 | Exact | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: |
| v2 | 57 | 0 | 0 | 0 |
| OX original | 16 | 0 | 39 | 2 |
| v1 | 18 | 34 | 0 | 5 |
| md4c | 19 | 33 | 0 | 5 |
| pulldown-cmark | 14 | 43 | 0 | 0 |
| Bun native | 16 | 40 | 0 | 1 |

These pairwise counts overlap. All-six agreement admits 14 cases; agreement
among v2/v1/md4c/pulldown/Bun admits 50. Sets are determined before timing and
are common to every engine scored in that set. No competitor gets a different
input mix within one aggregate. Pairwise agreement with v2 is symmetric HTML
comparison, not a declaration that v2 is the correctness oracle.

## Round 4 changed no output on this corpus

Every one of the 342 archived HTML outputs (57 documents × 6 engines) is
byte-identical to [2026-09-21-native-release-fixed](../2026-09-21-native-release-fixed/README.md),
which measured the repaired head `60602a5a`, and therefore to
[2026-09-21-native-release](../2026-09-21-native-release/README.md),
[2026-09-16-native-segments](../2026-09-16-native-segments/README.md) and the
two reports before it. The agreement classifications are identical too, so the
14-case and 50-case scored subsets are the same sets of documents. `audit.py`
asserts this equality over the whole `verification.json`, and
`compare_previous.py` refuses to emit a comparison if either the outputs or the
agreement sets differ.

The three round-4 changes ([round report](../2026-09-21-perf-round-4/README.md))
alter no output by design: the closer probe answers the same question from
a forward window, the arena reservation changes only how much memory a
fresh parse reserves, and the inline depth is counted in a field instead of
a shared cell. The paired harness verified exact HTML and AST equality
between baseline and candidate on every measured case before timing, for
each of them alone and again on top of each other, and this report adds the
six-engine HTML over the frozen corpus.

## Original OX

OX always emits heading IDs. Its 39 heading-only cases remain excluded from
all-six agreement, even when text and other HTML agree. The Vue `slots` and
`reactivity-in-depth` documents also differ in fenced-code language attributes:
OX cleans suffixes such as `{2}` from `vue-html{2}`, whereas the strict native
renderers preserve the first info token. These are the two `other` cases.
Neither behavior can be disabled through its original public option type.

The guards separately confirm OX's automatic callouts and `[[toc]]` handling.
Those outputs remain archived; replacing them through custom rendering hooks
would no longer benchmark the original native renderer path.

## Seven cases outside configurable-five agreement

| Input | Difference retained by the comparator |
| --- | --- |
| `comment-checklist` | md4c and Bun add task-list CSS classes and omit the space after each checkbox that the other renderers emit. |
| `guard-angle-link` | V1 emits an additional URL link after the authored angle-destination link. |
| `vite-docs-features` | V1 similarly emits an additional link for an angle-delimited destination. |
| `wiki-rainbow-article-body` | V1 differs in emphasis/link parsing and surviving label text; md4c percent-encodes apostrophes in URL destinations. |
| `wiki-chess-article-body` | V1 differs in emphasis/link parsing and surviving label text; md4c percent-encodes apostrophes in URL destinations. |
| `wiki-volcano-article-body` | V1 differs in emphasis/link parsing and surviving label text; md4c percent-encodes apostrophes in URL destinations. |
| `wiki-tea-article-body` | md4c percent-encodes apostrophes in URL destinations. |

These are observations of pinned engines, not new fixes or a specification
audit. The comparator deliberately does not decode ASCII reserved characters
such as apostrophes, discard task classes, or repair missing content to broaden
the matching subset. The unchanged corpus is still timed in full; nonmatching
results are labeled workload diagnostics rather than equal-output speed claims.

Full HTML is in [verification.json.gz](verification.json.gz); executable option
and repeat-state guard outputs are in [behavior.json.gz](behavior.json.gz).
The input corpus retains all source attributions and licenses.
