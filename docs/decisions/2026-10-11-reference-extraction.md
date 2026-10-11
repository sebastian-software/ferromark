# Read-only reference extraction

Issue: [#521](https://github.com/sebastian-software/ferromark/issues/521).

## Decision

Add `Document::collect_references(source)` and opt-in `ReferenceSnapshot` in
the core, with `collectReferences()` on Node prepared handles. The public
`prepareDocument` entry reuses the prepared AST owner, preparation validation,
parser options, native passes, metadata, and rendering methods without
constructing or registering a Ferriki highlighter. Compiler preparation keeps
its existing highlighting ownership.

Reference inventories use AST occurrences and resolved destinations. They
preserve duplicates and source ordering, exclude code and raw HTML, and report
quoted MDX href/src attributes with their authored element origin. Expressions
and spreads remain unevaluated; static inventory does not promise runtime
coverage or custom component rendering behavior.

The source range contract uses half-open original UTF-8 byte offsets. A
pre-pass identity map borrows source/arena values and retains labels for exact
comparison when native passes run. Changed and generated nodes never claim a
span for the resulting value; retained changes expose a related original
range. Core snapshots are explicit. Reference extraction arrays are only
materialized when requested.

## Source-span correction

The GFM autolink post-pass previously added offsets in decoded text directly
to source offsets. Entities or punctuation escapes before or within a bare
link made that range inaccurate. Map decoded candidate boundaries through the
original inline source before existing container/BOM/line-ending maps run.
This corrects location metadata without changing autolink recognition, rendered
HTML/JSX, or conformance expectations. Regression fixtures cover escaped and
entity-encoded source, Unicode, blockquotes, BOM, and CRLF input.

## Validation

Rust and Node fixtures exercise Markdown link/image forms, definition
resolution across MDX boundaries, literal attributes, unsupported forms,
code exclusion, original offsets, duplicates, transform provenance, immutable
results, and repeat extraction with stable body/module rendering. The Node
example is exercised with Ferriki remote access disabled. No speed claim is
made.
