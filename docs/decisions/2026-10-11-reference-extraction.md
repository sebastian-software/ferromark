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
preserve duplicates and current document traversal ordering, exclude code and raw HTML, and report
quoted MDX href/src attributes with their authored element origin. Expressions
and spreads remain unevaluated; static inventory does not promise runtime
coverage or custom component rendering behavior.

The source range contract uses half-open original UTF-8 byte offsets. A
pre-pass identity map borrows source/arena values and retains labels for exact
comparison only when native passes run and `trackReferences: true` was selected
during preparation. The preparation-only option defaults to `false` on both
Node entry points, preserving ordinary preparation without a reference walk
or snapshot allocations. Collection with untracked passes fails explicitly;
untouched trees need no tracking. Changed and generated nodes never claim a
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
results, opt-in provenance capture, default preparation without snapshots,
getter validation, and repeat extraction with stable body/module rendering. The Node
example is exercised with Ferriki remote access disabled. No speed claim is
made.
