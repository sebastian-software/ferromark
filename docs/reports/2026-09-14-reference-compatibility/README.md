# Reference compatibility: second correction batch

All seven Ferromark discrepancies in the original cmark oracle corpus are
closed. The 106-case reference comparison and the frozen specification gate
pass. Explicit parser/renderer profiles remove product-specific output from
specification comparisons while preserving the existing convenience defaults.

The larger 256-case corpus agrees with cmark-gfm in 254 cases. The remaining
two are defects in the pinned reference's nested-link output; Ferromark keeps
the innermost link as required by the specification. Those differences remain
visible and are not relabeled as equivalent or silently excluded.

This is an intentional correctness change, following the user's instruction
that speed alone is not sufficient. See the
[reviewed decisions](../../decisions/2026-09-14-reference-compatibility.md).
Starting revision: `c4bb2237b1d02577a5957c1f1c833e30e0ea770a`.
The audit metadata records that parent revision plus the working-tree core
SHA-256 `ae1952feb3505317556523e0bf17a0233f0845cfff393cf7c4a0672dba1c36a4`;
it does not claim the changes were already in the parent commit.

## Fixes and reviewed attempts

| Attempt | Description | Result |
| --- | --- | --- |
| Extend the tilde look-ahead | Skip code, links, and HTML while searching for a closing tilde; check flanking and mixed run lengths | Discarded during review. A separate forward scanner duplicates inline syntax and cannot reliably model its precedence. No accepted timing claim. |
| Use the inline delimiter stack | Parse links, images, code, and HTML first; record single/double tilde delimiters alongside emphasis | Accepted. Fixes the five original cases and their broader failure families; respects mixed runs, intraword strikes, adjacent emphasis, escapes, and inline boundaries. Explicit single-tilde subscript still has priority when enabled. |
| Serialize Unicode URL bytes | Apply uppercase UTF-8 percent encoding to URL attributes, keeping existing escapes and IPv6 authority brackets | Accepted. Visible Unicode text and AST URLs remain unchanged; this is a URI spelling policy, not IDNA conversion. |
| Initial Unicode scanner | Search for a non-ASCII suffix again after each escaped ASCII byte | Rejected in review: escape-heavy URLs could incur quadratic rescanning. The final version bounds each ASCII run once and uses the existing escaper, with a 2,048-escape regression. |
| Normalize a leading BOM | Strip one initial encoding marker; compose its offset with NUL normalization | Accepted. BOM-only input borrows the remaining source; embedded/second BOMs remain content, and nested spans map to original bytes. |
| Explicit render profiles | Switch off IDs, callouts, inline TOC, and fence metadata handling | Accepted. CommonMark/GFM profiles agree across normal, borrowed, hooked, committed, and provisional paths. Review also fixed missing heading classes in the hook renderer. |
| Align the oracle's HTML policy | Add `--unsafe` to cmark-gfm while retaining its `tagfilter` extension | Accepted harness correction. Ordinary raw HTML must pass through in both engines. The original 106 reference outputs do not change; both versions of the expanded baseline are preserved. |

The delimiter implementation uses the existing pairing and failed-search bounds.
It follows the pinned cmark-gfm extension's handling of mismatched single/double
runs and its adjacent-emphasis flanking convention. It removes the old recursive
strikethrough substring parser instead of maintaining a second link/HTML parser.
The reference algorithms were inspected in
[cmark-gfm strikethrough](https://github.com/github/cmark-gfm/blob/587a12bb54d95ac37241377e6ddc93ea0e45439b/extensions/strikethrough.c)
and [inline parsing](https://github.com/github/cmark-gfm/blob/587a12bb54d95ac37241377e6ddc93ea0e45439b/src/inlines.c).

Only two existing snapshots change: Unicode autolink destinations and Unicode
TOC fragment destinations now use percent encoding. Their visible text and
heading IDs are unchanged. The failed snapshot run and reviewed output are
retained; specification fixtures and historical raw results were not rewritten.

## Before and after

| Suite | Before | After |
| --- | --- | --- |
| Original oracle, 106 cases | 76 exact, 17 serialization-equivalent, 6 heading-ID-only, 7 other | 90 exact, 16 serialization-equivalent; no differences/errors |
| Expanded oracle, 256 cases, aligned HTML policy | 166 exact, 90 other | 246 exact, 8 serialization-equivalent, 2 reference defects |
| Configured CommonMark 0.31.2, 652 cases | 40 heading-ID-only differences in the earlier conservative audit | 556 exact, 96 serialization-equivalent; no differences/errors |
| Current GFM extension examples, 28 cases | Passed after the first correction batch | 25 exact, 3 serialization-equivalent; no differences/errors |
| CRLF / lone-CR variants | Passed after the first correction batch | All 1,304 still agree with LF |

The first expanded run used the earlier cmark-gfm HTML policy and reported
98 differences. With matching raw-HTML flags, the baseline has 90. Keep this
method correction separate from implementation gains. `raw/bindings-before`
and `raw/bindings-before-aligned` contain both records.

The original 106 and expanded 256 inputs now run in the ordinary Rust suite
against frozen reference outputs. The two reviewed reference defects have
explicit specification-based output assertions. Checkbox empty boolean values
are canonicalized only for the exact generated input tags in that test; the
shared normalizer is unchanged. Live differential runs still use the existing
conservative comparator and preserve every actual HTML string.

### Two reference defects, not remaining Ferromark fixes

For `[~[x](u) [x]~](target)` and its double-tilde variant, pinned cmark-gfm emits:

```html
<p><a href="target"><del><a href="u">x</a> [x]</del></a></p>
```

Ferromark emits:

```html
<p>[<del><a href="u">x</a> [x]</del>](target)</p>
```

[GFM's links rule](https://github.github.io/gfm/#links) prohibits links within
links at any depth and gives the innermost valid link precedence. The retained
outputs are therefore intentional, and the generic strict 256-case oracle
still exits **1**, with exactly `binding-1-31-link` and `binding-2-31-link` as
`other`. The 106-case strict oracle exits **0**. No comparator exception hides
these cases.

### Profiles and limits

Use `ParserOptions::commonmark()` with `HtmlRendererOptions::commonmark()`,
or `ParserOptions::gfm_spec()` with `HtmlRendererOptions::gfm()`. The GFM pair
enables tables, tasks, strikethrough, autolinks, and the tagfilter, without
footnotes. Existing `ParserOptions::gfm()` and renderer defaults retain the
product conveniences. IDs can also be disabled independently; doing so suppresses
heading permalinks and inline TOC substitution, including explicit IDs.

The full 677-example GFM website still has ten diagnostic differences: five
tagfilter/core interactions, three bare-autolink/core interactions, and two
older HTML-comment rules. These are separate from its 28 extension examples.
The CommonMark core remains version 0.31.2; this change does not regress it to
the older GFM website's core examples. MDX remains bounded syntax capture and
static rendering, not a compiler/runtime. Finite differential corpora cannot
establish correctness for every possible Markdown input.

## Validation

All required gates passed with locked dependencies:

- `cargo fmt --all --check`
- `cargo test --workspace --all-features --locked`: **700 tests passed**
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo bench --workspace --no-run --locked`
- Compatibility-audit Python helpers: **18 tests passed**
- Frozen specification audit and original cmark corpus: strict exit **0**
- Expanded corpus: **254 reference agreements plus two spec assertions**, no engine errors

The workspace run includes original span checks for BOM/NUL, scalar/vector
escaping comparisons, all inherited snapshots, and the permanent oracle tests.
`raw/gfm/stack-first.log` records the intermediate fixture-policy and adjacent
emphasis findings; `stack-final.log` records the corrected focused pass.
`raw/workspace-tests-before-snapshots.log` is the expected red for the two
reviewed URL spelling changes, not an unexplained remaining failure.

## Short performance check

This is a small within-V2 check, not a rerun of the six-engine ranking.
The baseline binary is the verified first-correction core from commit
`4a1e55f190e16fad60314a1c91183123710d8bba`, identical in production code to the
starting `c4bb223`. Its frozen source was verified against Git before binary
reuse. Both workers use Rust 1.95, generic CPU flags, fat LTO, one codegen unit,
and the same frozen worker/lock versions. Timings ran after compilation finished.
Each case/mode has nine alternating pairs across three rounds, 50 ms per sample.

| Input | Fresh time change | Reused-arena time change |
| --- | ---: | ---: |
| `comment-review` | −0.3% | −1.1% |
| `guard-angle-link` | +1.9% | +5.4% |
| `legacy-contributing` | +1.0% | +0.6% |

Positive values mean more time. These are paired medians with exact HTML,
AST-debug, child-count, and checksum verification. The angle-link case's reuse
rounds vary, so a single small-run median is not a precise population estimate.
The three-case geometric mean is +0.8% fresh / +1.6% reuse, with no claim that
this subset represents the full corpus.

The initial four-case verification rejected `wiki-rainbow-article-body` before
timing: its AST is identical but Unicode URL attributes now differ intentionally.
It was excluded without weakening the equality gate. The full HTML difference,
failed run, successful subset, samples, and build hashes are archived under
`raw/performance`. Historical six-engine figures in the root README remain
explicitly labeled as pre-correction measurements.

## Reproduction and evidence

Use the [spec audit instructions](../../../benchmarks/compatibility-audit/README.md)
and [pinned cmark instructions](../../../benchmarks/compatibility-audit/CMARK.md).
Build metadata identifies cmark 0.31.1 and cmark-gfm 0.29.0.gfm.13, exact source
revisions, compiler commands, and binary hashes. Both reference invocations
permit raw HTML; GFM applies its five named extensions including tagfilter.

```sh
cargo test -p ferromark_renderer --test cmark_regressions --test gfm_extension_audit --test profiles --locked
python3 benchmarks/compatibility-audit/cmark_binding_fixtures.py /tmp/cmark-bindings.json
```

Repeat the live oracle command with `--fixtures /tmp/cmark-bindings.json` and
fresh output paths. Its strict exit 1 must retain only the two documented
reference defects. The default 106-case corpus and configured spec gate must
exit 0. The production patch and all validation exit codes are recorded in
`raw/production.patch` and `raw/validation.json`.

[SHA256SUMS.json](SHA256SUMS.json) covers the raw evidence. Original/authored
oracle inputs are MIT licensed with this repository. CommonMark/GFM specification
examples in audit outputs retain their upstream CC BY-SA attribution described
in the [fixture provenance](../../../benchmarks/compatibility-audit/README.md#fixture-provenance).
Performance documents retain the attribution in their original corpus reports.
