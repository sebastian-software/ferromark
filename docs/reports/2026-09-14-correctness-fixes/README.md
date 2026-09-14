# First CommonMark and GFM correctness fixes

The first correction batch fixes line-ending content corruption, literal NUL
handling, URL escaping, and the five failures in the current GFM extension
examples. Ferromark v1 already handles several of these cases correctly, so
they are regressions relative to v1 even though the previous audit traced the
reproduced v2 behavior back to original OX-Content.

These are intentional semantic corrections authorized by the user, documented
in the [decision record](../../decisions/2026-09-14-correctness-fixes.md). The
historical fixtures, expected-failure lists, and existing snapshots are unchanged.

## Before and after

The baseline is `4342b310d8a6612b5df67d697b9d2be733c4ca70`, the committed
[original audit](../2026-09-14-compatibility-audit/README.md). The corrected
working tree is identified by its core source SHA-256 in
[metadata.json](raw/audit/metadata.json):
`39d5e0652dca70e57f90ebcf40d93b3ad0edd9665c8fe8f7f23a3a03155b0d33`.
The metadata's `source_revision` is the baseline HEAD, not a claim that the
uncommitted fixes were present in that commit. The hash covers sorted
`crates/*/src/**/*.rs` paths and bytes; normalizer, runner, worker, fixture, and
dependency hashes are recorded separately.

| Check | Before | After |
| --- | --- | --- |
| Configured CommonMark, 652 examples | 7 URL-output differences beyond heading IDs/serialization | 0 such differences |
| Current GFM extension sections, 28 examples | 23 match, 5 differ | All 28 match |
| CRLF/CR variants of all CommonMark examples | 12 differences in 1,304 | 0 differences in 1,304 |
| Six original literal-NUL contexts | 6 failures | All 6 match U+FFFD expectations |
| Full configured GFM website, 677 examples | 22 classified differences | 10 classified profile/version differences |
| Two deliberately incorrect normalizer counterexamples | Both incorrectly accepted | Both rejected |

The conservative comparison still separates output spelling and heading policy:

| Profile | N | Exact HTML | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: | ---: |
| Configured CommonMark | 652 | 514 | 98 | 40 | 0 |
| Public defaults / CommonMark | 652 | 500 | 98 | 40 | 14 |
| Configured GFM extensions | 28 | 25 | 3 | 0 | 0 |
| Full configured GFM | 677 | 526 | 101 | 40 | 10 |
| Full GFM without tagfilter | 677 | 530 | 101 | 40 | 6 |
| Public GFM preset | 677 | 510 | 101 | 40 | 26 |

All 652 configured CommonMark examples and all 28 configured GFM extension
examples also pass the strengthened Rust spec normalizer. That normalizer
retains the documented heading-ID allowance. These are finite fixture results,
not an unrestricted compatibility or byte-for-byte equality claim. No parser
error or panic occurred in the audit sweeps or 30 targeted probes.

## Implementation and regression coverage

- **Line endings:** list-item boundary detection used `str::lines()`, which
  does not split lone CR. Reusing the parser's existing line scanner fixes
  sibling and nested-list duplication. Inline HTML recognizes CR as whitespace
  and stores normalized LF values; angle destinations reject CR. The tests
  preserve original source spans and compare all 1,304 variants with LF output.
- **Literal NUL:** normalization occurs before syntax parsing and reference
  collection. Clean input remains borrowed. Only NUL-bearing input allocates
  replacement text and one map entry per replacement. Existing recursive span
  traversal maps the result back to original bytes, including nested lists,
  tables, and MDX attributes. Eleven source contexts are checked in three
  parser profiles, with Unicode, reference-label, and span controls.
- **URL attributes:** backslash, brackets, and backtick are percent-encoded.
  Reserved URL delimiters and existing percent escapes retain their meaning.
  Valid IPv6 host brackets remain syntax, including userinfo and port forms;
  host validation prevents arbitrary bracketed text from bypassing attribute
  escaping. Scalar, word, and SIMD classifiers use the same byte set, checked
  against the independent byte-at-a-time reference and adversarial inputs.
- **GFM:** single and double tildes support strikethrough; runs of three or
  more remain literal. Delimiter scanning respects escapes and code spans.
  Explicit subscript configuration keeps its single-tilde precedence. Extended
  `mailto:`/`xmpp:` autolinks require contiguous addresses and bounded resource
  syntax; malformed prefixes cannot capture a later email address.
- **Test harness:** code/raw-text whitespace and reserved ASCII URL escapes
  remain significant. Parser errors and panics fail the spec suites directly.
  The three intentional CommonMark-to-GFM autolink exceptions now have explicit
  expected HTML, instead of relying only on their known-failure IDs. Current
  GFM examples and line-ending transformations are continuous workspace tests.

Initial failing reproductions and passing focused runs are preserved in
[`raw/`](raw/). The audit gate now checks both comparison results for the
configured suites, rejects errors in every lane, and requires the negative
normalizer controls to stay rejected. Ten Python helper tests cover extraction,
comparison boundaries, and failure-gate behavior.

## Ferromark v1 comparison

The comparison uses v1 main
`143ec2ce151d87d2a3d804a048014afc97733ae0` (package 0.9.0), through the frozen
native worker from the [six-engine report](../2026-09-14-native-engines/README.md).
The v1 checkout was not modified. The selected 25 reproductions deliberately
include both original examples and overlapping minimized forms; **8/25 is not
a representative conformance rate**.

V1 matches eight of these expectations after output line-ending normalization:
the backslash-path URL, triple tildes, and six original/minimized HTML or
line-ending cases. V2 previously failed these, supporting the user's regression
concern. V1 also fails the selected CR-list, literal-NUL, and single-tilde cases.
The corrected v2 is checked against the specifications, not forced to reproduce
v1's remaining differences. Extended `mailto:`/`xmpp:` probes are excluded from
this v1 comparison because the frozen worker's shared GFM lane disables that
extension.

[The recorded comparison](raw/v1-comparison.json) retains all input, expected
HTML, previous v2 HTML, v1 HTML, classifications, and the worker hash.
[The reproduction script](reproduce_v1.py) verifies the frozen binary hash and
reruns the selected cases.

## Differential checks against cmark

The user's additional suggestion is implemented as a separate
[offline oracle harness](../../../benchmarks/compatibility-audit/CMARK.md).
[cmark](https://github.com/commonmark/cmark) is the CommonMark C reference
implementation; [cmark-gfm](https://github.com/github/cmark-gfm) is GitHub's
fork with GFM extensions. This run pins cmark 0.31.1 at
`bb3678d7a73cb02d35c8876ecd097072636200a8` and cmark-gfm 0.29.0.gfm.13 at
`587a12bb54d95ac37241377e6ddc93ea0e45439b`.

The versioned corpus contains 106 deterministic inputs: nested lists, quotes,
links, code and emphasis, tilde combinations, Unicode/NUL, raw HTML, GFM syntax,
and line-ending variants. The frozen spec fixtures remain the normative tests;
an oracle discrepancy creates a review case instead of automatically replacing
our expected output. The reference versions also differ from the current
CommonMark 0.31.2/GFM website snapshots.

| Profile | Inputs | Exact | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: | ---: |
| CommonMark / cmark | 82 | 61 | 14 | 5 | 2 |
| GFM / cmark-gfm | 24 | 15 | 3 | 1 | 5 |

Five additional GFM binding cases remain open despite passing the 28 official
extension examples:

| Corpus ID | Input | Difference to investigate |
| --- | --- | --- |
| `tilde-01` | `~foo ~ bar~` | An intermediate whitespace-adjacent tilde prevents the outer strike |
| `tilde-02` | `~~foo ~~ bar~~` | The same issue with double tildes |
| `tilde-03` | `~a [b](u~r)~` | The strike closes inside a link destination, breaking the link |
| `tilde-04` | `~~a [b](u~~r)~~` | The same link-boundary issue with double tildes |
| `gfm-11` | `~~one~ two~~ and ~three~~` | Mixed delimiter runs bind differently; check the intended GFM rule |

For `tilde-03`, cmark-gfm produces a complete anchor inside `<del>`; v2 emits
`<p><del>a [b](u</del>r)~</p>`. This is exactly the kind of combination missing
from the simple official examples. These cases are persisted as the next
delimiter-fix targets; they are not hidden by the passing specification count.

The two CommonMark differences require separate interpretation: `unicode-02`
percent-encodes a Unicode URL host/path in cmark while v2 retains UTF-8;
`unicode-05` drops a leading BOM in cmark while v2 preserves it. Neither result
alone establishes a CommonMark parsing violation. All inputs and raw outputs,
reference build provenance, and final run metadata are retained under
[`raw/cmark-oracle/`](raw/cmark-oracle/).

The final oracle run has no worker/reference errors. Its strict mode returns
**exit 1** for the six heading-ID-only and seven other differences; this is
separate from the passing frozen-spec audit. All **17 Python helper tests**
pass, including partial-response, crash, timeout, and exact fixture-generation
checks. The reference binaries were rebuilt from the pinned clean source
trees. CMake commands, actual compiler identities, cache files, and binary
hashes are included with the results.

## Remaining work

The ten full-GFM differences are classified, rather than accepted as ten
undifferentiated parser bugs:

| GFM examples | Explanation |
| --- | --- |
| 140, 141, 142, 145, 147 | A global GFM tagfilter changes raw HTML shown unfiltered in core examples |
| 617, 620, 621 | The GFM extension links bare URLs/emails shown as text by core autolink examples |
| 649, 650 | Older GFM HTML-comment rules differ from CommonMark 0.31.2 |

The next product decision remains explicit CommonMark/formal-GFM/product
presets, with switches for heading IDs, callouts, TOC, and fence metadata.
Footnote defaults and renderer autolinking also need to be named clearly in
those profiles. Current defaults are unchanged. MDX remains syntax capture and
static output; this batch does not add JavaScript validation, compilation, or
a component runtime. The five oracle delimiter cases above are concrete next
fix targets. Further fuzz and differential testing can expand that coverage.

## Short performance check

Two bounded runs compare the corrected `4a1e55f` core with `4342b31` using the
existing native optimization worker: four documents, fresh and reused arenas,
three process rounds, three paired 50-ms windows per round. The frozen builds
use Rust 1.95, fat LTO, one codegen unit, and generic target CPU. Exact HTML and
AST Debug output agree before timing for every measured case and mode.

| Mode | Baseline/candidate time, run 1 | Run 2 | Combined | Candidate time change |
| --- | ---: | ---: | ---: | ---: |
| Fresh | 0.98125 | 0.98062 | 0.98094 | +1.94% |
| Reuse | 0.96495 | 0.96255 | 0.96375 | +3.76% |

Each run uses the geometric mean of four case-level median ratios; the combined
column uses both runs. The cases are `comment-review`, `guard-angle-link`,
`legacy-contributing`, and `wiki-rainbow-article-body`. The corrections have a
small measured cost on this subset. These measurements do not isolate which
individual correction causes it.

The host remained loaded, and both runs contain noisy pairs: run 1 has a 0.615
ratio for the prose/reuse case; run 2 has a ratio above 1.33 for the angle-link/
reuse case. The median effects are similar between runs. Two short runs from
one build are not evidence for the full 57-document corpus or a new six-engine
ranking. The README's earlier native comparison is explicitly historical.

Both [raw runs](raw/performance/), build metadata, exact local build scripts,
commands, samples, and verification output are retained. Large verification
JSON files are gzip-compressed losslessly. The corpus is identical to the
[already committed native corpus](../2026-09-14-native-engines/corpus.json.gz).
For a portable rebuild, use new directories and the wrapper that selects this
batch's baseline without editing the historical harness:

```sh
mkdir -p /tmp/ferromark-fix-perf/baseline /tmp/ferromark-fix-perf/candidate
git archive 4342b310d8a6612b5df67d697b9d2be733c4ca70 | tar -x -C /tmp/ferromark-fix-perf/baseline
git archive 4a1e55f190e16fad60314a1c91183123710d8bba | tar -x -C /tmp/ferromark-fix-perf/candidate
python3 docs/reports/2026-09-14-correctness-fixes/prepare_performance.py \
  --baseline-path /tmp/ferromark-fix-perf/baseline \
  --candidate-path /tmp/ferromark-fix-perf/candidate \
  --out /tmp/ferromark-fix-perf/build --lto fat
python3 benchmarks/optimization-rounds/run.py /tmp/ferromark-fix-perf/build \
  docs/reports/2026-09-14-native-engines/corpus.json.gz \
  /tmp/ferromark-fix-perf/results \
  --modes fresh reuse --rounds 3 --pairs 3 --window-ms 50 \
  --filter '^(comment-review|legacy-contributing|wiki-rainbow-article-body|guard-angle-link)$'
```

## Validation and reproduction

Rust 1.95.0 on aarch64 macOS; root dependencies remain locked. The full workspace
run passed **674 Rust test executions**, including integration tests and
doctests. Formatting, Clippy with warnings denied, and benchmark compilation
are checked separately. The initial ten Python helper tests and the strict
spec audit pass; the later oracle harness expands the helper suite to 17.
Raw gate logs retain the exact command output. Final test-only style corrections
were rerun in the affected suites after the full workspace run.

From the repository root:

```sh
cargo fmt --all --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo bench --workspace --no-run --locked
python3 -m unittest discover -s benchmarks/compatibility-audit -p 'test_*.py'
python3 benchmarks/compatibility-audit/build.py /tmp/ferromark-fixed-audit-build
python3 benchmarks/compatibility-audit/run.py \
  /tmp/ferromark-fixed-audit-build/target/release/compatibility-audit-worker \
  benchmarks/compatibility-audit/fixtures/gfm-0.29-2026-09-14.html \
  /tmp/ferromark-fixed-audit-results --fail-on-differences
```

Use new output directories for the audit. The retained run reused the audit
worker's existing separate build directory; its final build log is included.
See the [harness documentation](../../../benchmarks/compatibility-audit/README.md)
for profile configuration and comparator limits. Specification-derived inputs
and outputs retain the CommonMark/GFM
[CC BY-SA attribution](../../../crates/ferromark_renderer/tests/spec_fixtures/README.md).
