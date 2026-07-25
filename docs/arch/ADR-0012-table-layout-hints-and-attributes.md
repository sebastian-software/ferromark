# ADR-0012: Table Layout Hints and Attribute Hooks

**Status:** Proposed
**Date:** 2026-07-25

## Context

GFM pipe tables carry alignment but no portable column-width or table-attribute
contract. Authors nevertheless use delimiter rows with different dash counts
to make source tables readable, and Pandoc can interpret those counts as
relative widths. Other Markdown implementations expose broad attribute-list
syntaxes for IDs, classes, and arbitrary key/value attributes.

Ferromark has no AST and renders a compact semantic event stream directly.
Adding arbitrary attributes would therefore affect parsing, public events,
every renderer, escaping, and the trust boundary at once. It would also create
syntax ambiguity around braces in cell content and around whether an attribute
belongs to a cell, row, section, or the table.

This evaluation separates numeric layout metadata from arbitrary author-supplied
attributes.

## Compatibility Matrix

| Proposal | CommonMark | GFM | Disabled behavior | Main ambiguity |
| --- | --- | --- | --- | --- |
| Delimiter dash ratios | Outside the spec | Valid table syntax; adds meaning to formatting | Exact current GFM output | Authors may vary dashes only for source alignment |
| Explicit units in delimiter cells | Outside the spec | Invalid delimiter syntax | Would remain literal Markdown | Unit grammar, CSS support, and injection policy |
| Attribute lists after a table | Outside the spec | Parsed as ordinary text | Would remain ordinary text | Attachment to table vs prior/next block |
| Attribute lists inside cells | Outside the spec | Cell text | Would remain cell text | Cell vs inline-element attachment |

## Decision

Prototype only delimiter dash ratios behind `Options::table_column_widths`.
The option is disabled in `minimal`, `commonmark`, `gfm`, and `default`.
`tables` remains the independent gate: width hints never enable table parsing.

For a valid pipe-table delimiter row, Ferromark already scans each run of
dashes. With the option enabled, it additionally:

1. performs a second, bounded pass to collect each dash count;
2. normalizes all counts to basis points whose sum is exactly 10,000; and
3. emits typed `TableColumnWidth` events rendered as numeric `<col>` percentage
   hints.

Alignment colons do not contribute to the ratio. The generated HTML uses a
`<colgroup>` before `<thead>`.

Do not prototype explicit CSS units, classes, IDs, or generic attribute lists
in this change. Those features are deferred until a concrete consumer can
justify their attachment rules and a renderer-independent typed policy.

## Security Policy

The accepted prototype has no user-controlled attribute name or raw attribute
value. Its only HTML attribute is a fixed `style="width: …%"` template populated
from a parser-derived integer. It therefore cannot inject CSS tokens or HTML.

A future attribute API must not pass arbitrary source strings through by
default. It needs:

- a typed allowlist for supported targets and attribute names;
- HTML escaping after policy validation;
- duplicate-ID and duplicate-key behavior;
- explicit handling for `style`, event-handler names, URLs, and `data-*`; and
- tests under both trusted and untrusted rendering policies.

`style` and event handlers should be denied by default. Classes and IDs may be
considered separately because their risk and use cases differ from arbitrary
key/value attributes.

## Performance Expectations

The disabled path keeps the existing alignment-only delimiter scan and its
compact alignment buffer. It adds one option branch only after a table has
already been recognized, so ordinary text and non-table Markdown do no new work.

The enabled path performs a second delimiter scan and one linear normalization
pass over at most the existing table-column limit. It emits one additional event
per column plus the two `<colgroup>` boundaries. Benchmark
`options/shared_corpus/default` against
`options/shared_corpus/table_column_widths`; a material default-path regression
rejects the prototype.

On the 2026-07-25 Apple Silicon development run, the unchanged `main` default
measured 180.98 µs and the option-disabled prototype measured 182.37 µs
(+0.8%, reported by Criterion as within its noise threshold). The enabled
configuration measured 189.86 µs, about 4.1% above the disabled prototype on
this deliberately table-heavy corpus.

## Consequences

- Authors can request portable relative hints without embedding CSS or HTML.
- Existing CommonMark and GFM output remains byte-for-byte unchanged unless the
  new option is explicitly enabled.
- Equal delimiter lengths intentionally produce equal width hints; callers that
  use dashes only for source formatting should leave the option disabled.
- The public event stream can expose widths to non-HTML consumers without
  reparsing source text.
- General attribute hooks remain unresolved rather than silently expanding the
  trust surface.

## Alternatives Considered

### Explicit width units

Syntaxes such as `---{20%}` or `---[12rem]` are more expressive, but make the
delimiter grammar incompatible with GFM and require a CSS value parser or raw
value passthrough. Rejected for now.

### Generic attribute lists

Python-Markdown and kramdown demonstrate useful attribute-list syntax, but also
show that attachment rules vary by element and context. A generic implementation
would be much larger than table layout and is deferred.

### Renderer callback only

A callback could add attributes without parser syntax, but it cannot recover
the delimiter dash ratios from the current semantic stream. It also moves
document semantics into renderer-specific code. Rejected for width hints.

## References

- Pandoc User's Guide, `pipe_tables` extension:
  <https://pandoc.org/MANUAL.html#extension-pipe_tables>
- GitHub Flavored Markdown, tables extension:
  <https://github.github.com/gfm/#tables-extension->
- Python-Markdown attribute lists:
  <https://python-markdown.github.io/extensions/attr_list/>
- kramdown attribute lists:
  <https://kramdown.gettalong.org/syntax.html#block-inline-attribute-lists>
