# Markdown extensions

Syntax details for the optional extensions introduced in the
[README](../README.md#markdown-configuration). Start from an `Options` preset
and enable each extension explicitly; the
[API reference](https://docs.rs/ferromark/latest/ferromark/struct.Options.html)
documents every field and its default.

## Merged table cells

`merged_table_cells` adds MultiMarkdown/iA-style horizontal spans to GFM pipe
tables. The number of directly adjacent pipes after a cell is its column span:

```markdown
| Name | Price | Tax |
| --- | ---: | ---: |
| Widget | 10$ | 1$ |
| Gift | 0$ ||
```

The last cell renders as `<td colspan="2">0$</td>`. `|||` spans three
columns, and multiple cells in one row may be merged. Whitespace between pipes
preserves an explicit empty cell (`| value | | next |`). A merged cell uses
the alignment of its first covered column; body spans are clamped to the table
width and ragged rows are padded after the final span.

The flag requires `tables` and is disabled by default. With the flag off,
consecutive pipes retain standard GFM behavior and create empty cells.

## Table column widths

`table_column_widths` is a separate, opt-in extension to GFM pipe tables. When
enabled, the relative number of dashes in each delimiter cell becomes a numeric
HTML column-width hint:

```markdown
| Short | Long |
| -- | ------ |
```

The example renders 25% and 75% `<col>` hints. Alignment colons are not counted.
No preset enables this interpretation because GFM otherwise treats delimiter
dash counts as formatting only. The extension accepts neither CSS nor arbitrary
HTML attributes. It composes with `merged_table_cells`: widths describe the
underlying table columns, while a merged cell spans those columns.

## Inline footnotes

`inline_footnotes` enables Pandoc-style `^[note text]` independently of
reference footnotes:

```markdown
The result needs context.^[This note can contain *inline Markdown*.]
```

The opening caret may be escaped as `\^[literal]`. Balanced brackets, links,
code spans, and soft line breaks are supported inside a note, but an inline
note is always one paragraph. The iA Presenter form `[^Footnote text.]` is not
accepted as an inline note because it is indistinguishable from Ferromark's
existing `[^label]` reference syntax.

The HTML renderer numbers inline and reference notes together by first
appearance and emits their definitions in the document-end footnote section.
Presentation adapters should consume `InlineEvent::InlineFootnote` and flush
collected notes at their own slide boundary; the core HTML renderer does not
infer slides.

## Definition lists

Enable `definition_lists` for PHP Markdown Extra-style terms and descriptions:

```markdown
Term
: A definition with *inline Markdown*.
```

Markers may have up to three leading spaces and require whitespace after the
colon. Continuation paragraphs and nested blocks must be indented to the
description content; lazy continuation is supported only for paragraph text.
The option is disabled by default and in every dialect constructor.

## Line comments

Enable `line_comments` to omit source-only note lines from HTML:

```markdown
Published text.

// Review this wording before publishing.
```

Only `//` at the physical line start (after at most three spaces) is a
comment. URLs, trailing `//`, code blocks, raw HTML blocks, and explicit
container-prefixed lines remain ordinary Markdown. Comment text remains in the
source and is not suitable for secrets.

## Indented code blocks

Set `indented_code_blocks: false` for dialects that require fenced code blocks
and interpret four-space indentation as ordinary paragraph content. Fenced code
blocks remain available.

