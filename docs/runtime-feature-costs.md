# Runtime feature cost map

This is a source-level map of where optional parser and HTML renderer settings
add work. It describes control flow and dependencies; it does not provide
timings or rank features by runtime cost. The [runtime-profile
harness](../benchmarks/runtime-profiles/README.md) measures those questions
with fixed inputs and lifecycle-specific workers. Results and proposed profile
changes belong in [runtime-profile results](runtime-profiles.md).

## Scope labels

| Label | Meaning |
| --- | --- |
| Baseline structural work | Work already present for ordinary Markdown support, even when optional extensions are off. |
| Structural scan | A document-wide or AST-wide walk used to discover facts before the main pass. |
| Block/inline hot path | A cheap option check or probe paid repeatedly while parsing/rendering ordinary nodes. |
| Trigger-local | Extra work after the relevant syntax or AST node is encountered. |
| Output-only | Mostly changes emitted bytes or a branch at an already-rendered node. |
| No-op | Public configuration exists but is currently not consumed by the implementation. |

## Parser options

The public fields and presets are defined in the [parser option
type](../src/parser/options.rs). Parser construction
always performs source normalization and invokes the fused reference/footnote
prepass: [construction](../src/parser/mod.rs),
[normalization](../src/parser/source_normalization.rs),
[prepass](../src/parser/prepass.rs).
Reference definitions are baseline CommonMark support, so their candidate scan
is not an optional-feature comparison. The prepass returns early when no
block-shaped definition candidate exists; a real candidate causes a line
scan.

| Field | Scope and work | Dependencies or interactions |
| --- | --- | --- |
| `footnotes` | Extends the baseline prepass with footnote-label collection when `[^`/definition-shaped input is present; definitions then dedent and recursively parse a subdocument. | Shares the fused prepass with reference definitions; renderer `semantic_footnotes` is independent. |
| `task_lists` | Trigger-local list-item prefix check for `[x]`, `[X]`, and `[ ]`. | Only applies after list-item recognition. |
| `tables` | Block hot path: enables pipe probing during block dispatch and paragraph continuation checks; pipe candidates receive a two-line table probe. Actual rows/cells are trigger-local. | `merged_table_cells` and `table_attributes` only matter after a table is recognized. See [table dispatch](../src/parser/block.rs) and [table parser](../src/parser/table.rs). |
| `merged_table_cells` | Trigger-local table header/row splitting that preserves adjacent pipe runs as spans. | Requires `tables`; no work on non-table input. |
| `table_attributes` | Trigger-local lookahead after a table for a caption/`{#id .class}` line, followed by attribute and caption parsing if matched. | Requires `tables`; a non-matching lookahead only pays the table-local check. |
| `line_comments` | Block and paragraph dispatch reuse the first non-space/tab byte and check physical eligibility only for slash prefixes. Paragraph joining reuses the first observed comment; absent comments need no second discovery scan. Reference definitions retain a marker preflight. Actual removed comments can require joined text and a source map. | Changes block, table, definition, and nested-source handling; code and raw HTML remain opaque. See [line-comment handling](../src/parser/line_comments.rs) and the [dispatch measurement](reports/2026-09-14-line-comment-dispatch/README.md). |
| `front_matter` | Constructor-local leading scan for `---`/`+++`; when an opener is valid it scans to the closing delimiter and removes that range from Markdown parsing. | Root-document only; BOM and source-span normalization interact with the extracted body. See [front matter extraction](../src/parser/front_matter.rs). |
| `strikethrough` | Inline hot path for `~` runs; also changes flanking classification for `*`/`_` runs when enabled. | Competes with `subscript` for single tildes. |
| `autolinks` | Each block-level inline parse runs a cheap `may_contain_autolink` preflight. The recursive text coalescing/rewrite pass runs only when that preflight finds a possible candidate. | Does not make the renderer’s bare-URL option redundant; parser autolinks create link nodes, while renderer autolinks remaining text. See [inline block path](../src/parser/inline.rs) and [GFM rewrite](../src/parser/inline/gfm_autolink.rs). |
| `superscript` | Adds `^` to the fused inline marker classifier and parses matching spans locally. | Changes the selected classifier; no separate scan is added. Full parsing occurs only for candidates. |
| `subscript` | Trigger-local single-tilde branch and matching scan. | Shares `~` with strikethrough; does not add a new base marker because tilde is already scanned. |
| `math` | Adds `$` to every inline marker scan; `$` block lines receive a block probe. Candidate inline/display math scans forward for a valid closer. | Math AST nodes are only syntax preservation; mathematical typesetting is external. |
| `definition_lists` | A cached necessary-marker scan rejects marker-free suffixes before term collection. Candidate terms are represented by source coordinates; ordinary ASCII-letter prefixes skip irrelevant block recognizers. Real bodies still allocate a dedented source/map and recursively parse. | Each dedented subparser has a fresh marker cache. A later marker can retain speculative work on earlier text; definition bodies can combine with lists, tables, comments, and other block syntax. See [definition-list collector](../src/parser/definition_list.rs). |
| `heading_attributes` | Heading-local trailing trim and attribute-token parsing. | Explicit IDs require renderer `heading_ids`; CSS classes are emitted independently. |
| `wiki_links` | Only `[` inline candidates are affected; `[[...]]` candidates scan for a closing pair and parse/probe the label. | Nested-link rules still apply; renderer sees a normal `Link` node. |
| `cjk_emphasis` | Delimiter-local Unicode punctuation classification. | No prepass or allocation; it changes emphasis pairing semantics for East Asian punctuation. |
| `mdx` | Adds MDX marker/block dispatch for JSX, expressions, and ESM. Matching JSX tags or balanced expressions can scan forward; flow JSX children normalize and recursively parse subdocuments. | The renderer handles MDX AST nodes without a separate renderer flag. `max_nesting_depth` applies to JSX child subparsers. |
| `max_nesting_depth` | Cheap per-block guard and recursion bound. | This is a safety limit, not a speed knob; lowering it changes accepted input and can produce `NestingTooDeep`. |

## Renderer options

The public fields and strict profiles are defined in the [HTML renderer option
type](../src/renderer/html/options.rs). Every normal render
first performs the renderer’s allocation-free structural AST scan for heading
counts and a possible `[[toc]]` marker: [render setup](../src/renderer/html/renderer.rs),
[structural scan](../src/renderer/html/toc.rs). This scan
is baseline renderer work and still occurs when TOC substitution is disabled.

| Field | Scope and work | Dependencies or interactions |
| --- | --- | --- |
| `xhtml` | Output-only branches for breaks, images, and table `<col>` tags. | Only affects nodes that emit one of those tags. |
| `soft_break` | Output-only value substituted for every line ending in inline text. A cached flag compares it against the default once per renderer, so the default configuration adds one predictable branch on the text path and no scan; a non-default value splits each text value at its line endings. | Does not affect parsing. Emitted verbatim, so `xhtml` does not rewrite it. Hard breaks keep using `hard_break`. |
| `hard_break` | Output-only value emitted when a parsed hard-break node is visited. | Does not affect parsing or ordinary text. |
| `sanitize` | Trigger-local URL safety checks for links/images and escaping of raw HTML values. | Independent from `disallow_raw_html`; sanitization escapes all raw HTML values. |
| `disallow_raw_html` | Trigger-local tag-filter check on raw HTML values; filtering work occurs only when the value needs filtering. | This is the GFM fixed-tag filter, not general sanitization. MDX island children are filtered by their own renderer state. |
| `convert_md_links` | Trigger-local Markdown link/image conversion; raw HTML values with `href`/`src` are scanned and may allocate a rewritten string. | `base_url` and `source_path` matter only on conversion paths. |
| `base_url` | Read while converting root-absolute or Markdown URLs; otherwise inert. | Requires `convert_md_links`. |
| `source_path` | Read while converting Markdown URLs to distinguish `index.md` from other source files. | Requires `convert_md_links`; it affects relative route shape, not parsing. |
| `code_annotations` | Code-block-local line-state construction and metadata parsing when enabled. VitePress modes may parse inline directives and wrap every line. | `code_fence_metadata` must also be enabled for the annotation path. |
| `code_annotation_meta_key` | Read only for enabled attribute-style annotations with nonempty fence metadata. | Requires `code_annotations` and a syntax including `Attribute`. |
| `code_annotation_syntax` | Chooses attribute, VitePress, or both metadata parsers. | Most work is gated by `code_annotations`; VitePress/Both can add line splitting and directive parsing. |
| `code_annotation_default_line_numbers` | Trigger-local default line-number state for VitePress-inclusive syntax. | Requires `code_annotations` and `code_annotation_syntax` of `VitePress` or `Both`. |
| `toc_max_depth` | Only used during full TOC collection after a marker is found. That collection walks headings, gathers text, slugifies IDs, and builds entries. | Requires `inline_toc` and `heading_ids` plus a standalone `[[toc]]` paragraph. |
| `autolink_urls` | Builds a small first-byte index at render setup when enabled and patterns are nonempty; text nodes then run the bare-URL gate/scanner. | Independent from parser `autolinks`; `autolink_patterns` controls recognized prefixes. |
| `autolink_patterns` | Determines renderer autolink index construction and candidate matching. An empty list disables the path even if `autolink_urls` is true. | Custom patterns change matching work and output semantics. |
| `autolink_target_blank` | Output-only attributes on renderer-created bare URL links. | Only matters when a bare URL match is emitted. |
| `link_target_blank` | Output-only attributes on parsed Markdown links whose final href is HTTP(S). | Independent from `autolink_target_blank`. |
| `semantic_footnotes` | Footnote-local maps and records; definitions render into temporary body HTML and a final ordered section is emitted. | Requires parser `footnotes` to produce footnote nodes; legacy footnote rendering remains the off path. |
| `heading_permalinks` | Heading-local marker detection and permalink output. | Work is skipped unless `heading_ids` is also enabled; reuses the generated heading ID. |
| `source_spans` | Output-only attribute writes on rendered block nodes with nonempty spans. | Does not add a renderer-wide scan. |
| `heading_ids` | Heading-local text collection, slugification, duplicate-ID map updates, and ID output. | The baseline structural scan still counts headings even when IDs are disabled. `heading_attributes` supplies explicit IDs; CSS classes are emitted independently. |
| `callouts` | Every block quote checks its first paragraph for a `[!KIND]` marker; matching quotes use the callout renderer. | Non-callout quotes pay only detection; callout body handling has its own inline output path. |
| `inline_toc` | No full TOC collection unless the baseline scan finds a marker and `heading_ids` is enabled. Matching marker paragraphs are replaced by collected entries. | Requires `heading_ids`; `toc_max_depth` applies only after activation. |
| `code_fence_metadata` | Code-block-local language normalization and metadata path selection. Disabling it selects the plain fence path; it also short-circuits annotations through the combined guard. | `code_annotations` cannot take effect while this is false. |
| `table_colgroup` | Table-local `<colgroup>` and one `<col>` per alignment entry. | Only applies to rendered table nodes. |
| `table_column_names` | Header-local text collection/slugification and duplicate-name tracking while writing columns. | Requires `table_colgroup`; otherwise it is a no-op branch. |

## Interpreting profiles

Keep syntax options and rendering policy separate when constructing a profile:

- Parser `footnotes` and renderer `semantic_footnotes` measure different
  stages. The former changes prepass/AST work; the latter changes HTML state and
  final emission.
- Parser `tables` is required before table renderer options can be exercised;
  `merged_table_cells` and `table_attributes` are parser-side table features.
- Parser `heading_attributes` and renderer `heading_ids` interact for emitted
  explicit IDs, while ordinary generated IDs are renderer work.
- Parser and renderer autolink options are independent and should be toggled
  separately in a study.
- `inline_toc` requires both `heading_ids` and an actual marker, and code
  annotation settings require `code_fence_metadata` to remain enabled.
- A strict profile can disable product conveniences without removing baseline
  reference-definition support or the renderer’s structural AST scan.

The [runtime-profile harness](../benchmarks/runtime-profiles/README.md) records
these dimensions across parse, render, fresh, and reuse lifecycles. The results preserve those lifecycle distinctions; this source map is not a
performance ranking.
