# Footnote definitions use document-wide block scope

## Decision

With reference footnotes enabled, every footnote definition recognized by the
block parser contributes its normalized label to the whole document. This
includes list items, block quotes, footnote bodies, and Markdown block content
inside MDX components. References can precede or follow the definition and can
appear outside its container. Disabling link references does not disable
footnotes. Code, raw HTML, front matter, and comments cannot introduce labels
unless their content is actually parsed as a footnote definition.

This is an extension policy, not a CommonMark conformance correction. It makes
the existing document-wide footnote contract agree with the AST we emit.

## Why change

The old physical-line scan missed tab-indented MDX definitions and definitions
behind quote/list prefixes, while accepting lookalikes in raw HTML. It could
therefore emit a definition without resolving its references, or emit a reference
without an actual definition. The MDX span comparison previously supplied an
extra root definition to isolate source mapping and did not test this behavior.

The existing structural collection phase now collects both link definitions and
footnote labels in one traversal. It uses the real parser options and skips
inline parsing. Paragraph boundaries still use the block grammar, but collection
does not allocate paragraph nodes. Footnote bodies without `]:` cannot contain
nested definitions and are skipped only during collection. The document parse
still parses and validates those bodies fully. No collection runs when both reference features are disabled or
when the cheap definition candidate filter finds nothing. Nested definitions are
visited recursively. Source mapping continues to use the ordinary parsing path.

## Validation

End-to-end regressions cover document-wide references across MDX (including children on the opening tag line), lists, and
quotes with LF, CRLF, and CR, with link references enabled and disabled. Negative
cases cover fenced code, indented code, and raw HTML. The MDX source-span test
now runs without a synthetic root definition. Existing snapshots and official
specification baselines remain unchanged.

See the [measured performance and coverage](../reports/2026-09-15-footnote-scope/README.md).
The owner accepts the measured cost when footnotes are used. Retain this fix;
further performance work prioritizes disabled and enabled-but-unused paths.
