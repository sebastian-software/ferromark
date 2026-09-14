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
a component runtime. Broader delimiter combinations and fuzz/differential
testing remain useful follow-up coverage beyond the official finite examples.

## Validation and reproduction

Rust 1.95.0 on aarch64 macOS; root dependencies remain locked. The full workspace
run passed **674 Rust test executions**, including integration tests and
doctests. Formatting, Clippy with warnings denied, and benchmark compilation
are checked separately. The ten Python helper tests and the strict audit pass.
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
