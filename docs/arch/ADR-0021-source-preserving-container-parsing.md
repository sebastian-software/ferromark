# ADR-0021: Migrate container parsing against the existing revision comparator

- Status: Accepted
- Date: 2026-09-25
- Scope: Issue [#432](https://github.com/sebastian-software/ferromark/issues/432)

## Context

Block quotes and complex list items currently build stripped sub-sources, parse
them with a nested parser, and map nodes and errors back through a `SourceMap`.
`Parser::parse_block_quote` also expands marker tabs into synthetic spaces,
tracks lazy continuation lines and removes configured line comments while it
builds that source. `Parser::parse_list` recognizes sibling and continuation
lines; simple single-line items already have a direct inline path, while items
that need block parsing use `item_source` and the list-item sub-parser.

The issue reports container-copy and reparse overhead and proposes parsing
against the original source. A replacement could also change where spans point
or how tabs, laziness, list tightness, comments and nested block syntax behave.
The existing benchmark worker deliberately requires successful parses and
compares the full `Document` debug representation, HTML and top-level child
count between two revisions before it times either one. The debug
representation includes node spans. It cannot compare parse-error spans,
because the worker treats parse failures as invalid benchmark inputs.

There are other sub-source parsers for footnote bodies, definition-list bodies
and MDX JSX children. They share source-remapping concerns, but they do not all
share the same container rules. Folding them into one migration would obscure
which change caused an output or performance difference.

## Decision

Treat this as an incremental, behavior-preserving optimization. Use the
existing revision-paired comparator as the legacy-versus-candidate oracle; do
not add a runtime parser switch or a second benchmark engine. Compare exact
AST debug output (including node spans) and HTML before timing `fresh`,
`reuse`, `parse` and `render` modes. Keep the current test snapshots and
conformance baselines unchanged.

The first implementation stage is block quotes. A later stage can migrate
complex list-item continuation parsing after the first stage is measured and
reviewed. Footnote, definition-list and MDX child parsers remain separate scope
until their relationship to the source cursor and nested definitions is
specified. Keep the existing simple one-line list-item fast path until a
separate paired comparison shows that changing it helps.

The replacement must preserve current behavior for lazy continuation and its
`OpenParagraph` boundaries, line comments, tab-column arithmetic (including
the existing narrow interpretation of tabs after nested container markers),
tight and loose lists, GFM tables, MDX flow, nested depth errors, and every
mapped AST span. A difference is not silently accepted as part of an
optimization: keep or restore the legacy path until a separate decision
defines the semantic change and its tests.

Use the generated `container-*` cases to compare container-heavy shapes with
the existing paired harness. Keep them out of the broad-corpus statistic. The
issue's stated retention bar is a broad geomean of at least 1.010 with no
unexplained fresh or reuse case below 0.970; evaluate that bar using the full
broad corpus, not the diagnostic subset. Require a paired Apple Silicon run and
a paired x86-64 run for any performance claim. The x86 workflow's pull-request
path filter does not run for parser-only changes, and its pull-request default
is an A/A control; dispatch it with the baseline and candidate revisions.

Keep parse-error span coverage in the Rust test suite, including errors nested
through quotes, lists, footnotes and definition bodies. Do not treat the
successful-input comparator as coverage for those errors.

## Consequences

Phase 0 adds reproducible targeted inputs and documents the admission gates; it
does not change the parser or claim a performance improvement. Each later
migration remains independently comparable with the pre-migration revision.
The extra generated cases can isolate container shapes without shifting the
frozen corpus or its baselines.

The source cursor, container stack, and definition visibility design remain
open until the block-quote stage is ready to implement. The reported overhead
alone is not enough to justify a broad parser rewrite or a different reading of
the specification.

## Validation

`python3 -m unittest discover -s benchmarks/optimization-rounds -p 'test_*.py'`
checks that generated container cases cover their declared shapes, retain
valid byte counts and hashes, and match the `containers` filter. Run the
existing parser snapshots and targeted behavior tests, especially
`tests/error_spans.rs`, `tests/lazy_continuation.rs`,
`tests/block_conformance_fixes.rs` and `tests/line_comments.rs`, for every
implementation stage. The paired comparison additionally rejects any exact
HTML, AST/span or child-count difference before its timings begin.

To dispatch the x86 comparison, select `containers`, set `baseline` to the
reviewed pre-migration revision, and `candidate` to the proposed revision. Run
`broad` separately for the issue's broad-corpus performance gate.
