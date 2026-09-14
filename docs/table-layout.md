# Table spans and CSS layout

Ferromark v2 has opt-in table structure and CSS layout extensions. Every preset leaves
them disabled, including the GFM convenience and specification profiles.

| Option | Owner | Effect |
| --- | --- | --- |
| `merged_table_cells` | `ParserOptions` | Adjacent closing pipes produce horizontal cell spans. |
| `table_attributes` | `ParserOptions` | A following attribute/caption line supplies the table ID and CSS classes. |
| `table_colgroup` | `HtmlRendererOptions` | Emit one `<col>` per logical column, with classes `col-1`, `col-2`, etc. |
| `table_column_names` | `HtmlRendererOptions` | Add `col-name-<slug>` classes derived from the first row; requires `table_colgroup`. |

The two parser extensions require `tables: true`. `ParserOptions::gfm()` enables
tables but leaves these extensions off. The renderer's `table_colgroup` option
can also be used with ordinary GFM tables and without table attributes.

## Example

```markdown
| Item | Net | Tax |
| :--- | ---: | ---: |
| Book | 20.00 | 1.40 |
| Gift | Included ||

: Prices *today* {#prices .price-list}
```

The last cell spans the Net and Tax columns. The caption becomes a `<caption>`;
the attributes belong to `<table id="prices" class="price-list">`.
Enabling `table_colgroup` emits this before the table head, after any caption:

```html
<colgroup>
<col class="col-1">
<col class="col-2">
<col class="col-3">
</colgroup>
```

Spans do not reduce the number of generated columns. External CSS can set widths
by class or index; the renderer does not generate inline width styles:

```css
#prices { width: 100%; table-layout: fixed; }
#prices > colgroup > .col-1 { width: 60%; }
#prices > colgroup > .col-2 { width: 25%; }
#prices > colgroup > .col-3 { width: 15%; }
```

Set `table_column_names: true` as well to address columns by their header text:

```html
<colgroup>
<col class="col-1 col-name-item">
<col class="col-2 col-name-net">
<col class="col-3 col-name-tax">
</colgroup>
```

```css
#prices > colgroup > .col-name-item { width: 60%; }
#prices > colgroup > .col-name-net { width: 25%; }
#prices > colgroup > .col-name-tax { width: 15%; }
```

Names use the same inline-text collector and Unicode-aware slug rules as heading
IDs. `**Netto Preis**` becomes `col-name-netto-preis`, `Größe` becomes
`col-name-größe`, and link labels or inline code contribute their text. Raw HTML
tags and nodes omitted by heading text extraction, such as images and math,
do not supply names. Empty or punctuation-only headers retain just `col-N`.

Duplicate header names get `-1`, `-2`, etc., with collision checks against
already assigned names, including naturally numbered headers such as `Price 1`.
Names are scoped to each table and do not change heading IDs. A spanning header
gives every covered column its shared name; positional classes still identify
individual columns. The `col-name-` prefix prevents numeric header text from
colliding with positional classes. Names follow the parsed first row before
render hooks run; changing or translating a header can change its name.

Enable the options with the public Rust API:

```rust
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

let source = "| Item | Price |\n| --- | ---: |\n| Included ||\n\n: {#prices .price-list}";
let allocator = Allocator::for_source_len(source.len());
let options = ParserOptions {
    merged_table_cells: true,
    table_attributes: true,
    ..ParserOptions::gfm()
};
let document = Parser::with_options(&allocator, source, options).parse().unwrap();
let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
    table_colgroup: true,
    table_column_names: true,
    ..HtmlRendererOptions::gfm()
});
let html = renderer.render(&document);
```

A complete CSS example is runnable with
`cargo run --locked -p ferromark_renderer --example table_layout > table-layout.html`.

## Syntax and boundaries

- The span is the number of directly adjacent, unescaped pipes closing a cell:
  `||` spans two columns and `|||` spans three. This works in both the header
  and body, including at the end of a row.
- Whitespace between pipes preserves an explicit empty cell: `| value | |`.
  An escaped pipe remains cell content, including inside code spans, following
  the existing GFM rules. Unescaped pipes inside code spans remain separators.
- Header spans must cover exactly the delimiter row's column count. A body
  span is clamped to the remaining columns; excess cells are discarded and
  short rows are padded. A merged cell uses its first covered column's alignment.
- Metadata follows the table directly or after one blank line. It must be a
  complete line with at most three leading spaces. `: Caption {#id .class}`
  and `: {#id .class}` are supported. The caption is inline Markdown and the
  attribute list must contain at least one ID or class.
- There may be one ID and multiple classes. Arbitrary key/value attributes,
  duplicate IDs, empty names, quotes, angle brackets, backslashes, and control
  characters in names are rejected. Invalid metadata remains ordinary Markdown.
  Attribute values are also escaped during rendering, including values supplied
  by an application that transforms the AST.
- Attributes and captions are stored in optional `Table::attributes` metadata;
  `TableCell::colspan` records the logical width. Generic AST visitors include
  caption nodes and their source spans map to the original document, including
  inside lists, block quotes, and MDX containers.
- Default and hook-based rendering share the same table attributes and column
  output. Caption children also traverse inline render hooks. XHTML output uses
  self-closing `<col />` elements.

This adds horizontal spans; it does not add row spans, multi-row headers, grid
tables, or numeric column-width hints. Positional classes remain available even
when header-derived classes are enabled. CSS text
alignment and font properties should target cells, since table cells are not
descendants of `<col>` elements.

## Decision and compatibility

Implemented at the project owner's request on 2026-09-14. Horizontal spans follow
the [MultiMarkdown convention](https://fletcher.github.io/MultiMarkdown-6/syntax/tables.html)
and Ferromark v1's `merged_table_cells` option. Table metadata uses a bounded
subset of [Pandoc's caption attribute syntax](https://pandoc.org/MANUAL.html#extension-table_attributes).
[HTML columns](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/col)
provide CSS width control independently of cell spans.

These are extensions beyond GFM. Existing conformance fixtures and default HTML
snapshots remain unchanged. Public AST/options structs gain fields in this
unpublished development API; their `Debug` representation therefore also changes.
Historical benchmark reports retain their original source and schema. No speed
claim is made for the added features.

## Validation

- `cargo fmt --all --check`
- `cargo test --workspace --all-features --locked`: 727 passing tests, including
  existing conformance/snapshot suites, table layout, header-derived names,
  duplicate-name collisions, renderer reuse, and hook/XHTML parity.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo bench --workspace --no-run --locked`
- Separate offline compilation of all three active SIMD/optimization timing
  and allocation worker sources against the updated facade.
- The runnable HTML example has three logical columns in every row, including
  the merged final cell, with metadata and caption/column elements in order.
  The rendered README feature matrix and local documentation links were checked.
