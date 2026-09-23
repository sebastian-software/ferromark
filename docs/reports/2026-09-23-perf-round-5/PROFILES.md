# Profiles before and after the round

Sample-based profiles of the 57 broad documents, taken with macOS `sample` (1 ms
interval, 10 s) on the Apple M1 Pro against the profiling driver in
[`harness/profile`](harness/profile): a fat-LTO release build with line tables
(`debug = 1`), linking the ferromark revision under test. The driver sweeps
every broad document at an equal byte volume (about 400 KB of input per
document and sweep): the 40 `gfm` documents with the GFM parser options minus
footnotes and the tag filter on, the 17 `commonmark` documents with the
defaults, XHTML output in both. `parse` parses into one reset arena; `render`
renders pre-parsed documents with `render_borrowed`.

The first pair was taken at `cb352020` (release 2.0.1) before the round, the
second at `060b02d2` (release 2.1.0), which holds the round's four merges and
the heading-id work of #408–#410. The raw call graphs are in
[`profiles/`](profiles/); `harness/profile/tree.py` aggregates them
(`tree.py <file> incl 40`, `callers <name>`, `callees <name>`). Shares are
inclusive: a symbol counts once per stack. **These are profile shares, not
measured speedups**; the measurements are in [NUMBERS.md](NUMBERS.md).

## Inclusive shares

### parse: 8,001 samples at `cb352020`, 8,237 at `060b02d2`

| Symbol | `cb352020` | `060b02d2` |
| --- | ---: | ---: |
| `Parser::parse_document` | 89.0% | 90.1% |
| `inline::Parser::parse_inline_block` | 47.1% | 49.5% |
| `block::Parser::parse_paragraph` | 47.1% | 49.2% |
| `inline::Parser::parse_inline` | 35.4% | 37.9% |
| `inline_link::Parser::parse_link` | 14.0% | 14.1% |
| `list::Parser::parse_list` | 12.4% | 11.8% |
| `table::Parser::parse_table` | 11.6% | 10.1% |
| `inline::marker_scan::InlineMarkerScan::next` | 10.9% | 11.3% |
| `table::Parser::parse_table_row` | 9.6% | 8.4% |
| `Parser::with_phase` | 9.1% | 8.1% |
| `memchr::memmem::searcher::searcher_kind_neon` | 8.3% | 5.8% |
| `core::str::<impl str>::trim_matches` | 6.1% | 0.2% |
| `table::Parser::parse_table_cell` | 5.2% | 5.7% |
| `table::Parser::table_row_cells_with_offsets` (closure) | 5.1% | 3.0% |
| `inline_link::Parser::resolve_link` | 4.9% | 4.9% |
| `inline::link_target::Parser::parse_link_target` | 4.5% | 4.4% |
| `inline::link_target::parse_destination` | 3.3% | 3.0% |
| `_platform_memmove` | 2.6% | 2.8% |
| `bumpalo` `RawVec::reserve_internal_or_panic` | 1.7% | 2.4% |
| `cursor::Parser::probe_line_inner` | 2.3% | 2.4% |
| `fenced_code::fenced_close_bounds` | 1.6% | 1.7% |
| `prepass::next_fence_run_line` | 1.6% | 1.7% |
| `delimiters::Parser::scan_balanced_matched` | 1.7% | 1.7% |
| `inline::emphasis::Parser::push_delimiter_run` | 1.7% | 1.6% |
| `delimiters::Parser::has_closer_from` | 1.6% | 1.5% |
| `table::Parser::try_parse_table` | 1.7% | 1.2% |
| `prepass::Parser::discover_definitions` | 1.1% | 1.3% |
| `line_scan::next_line_start` | 1.1% | 1.2% |
| `core::str::<impl str>::trim_start_matches` | 1.1% | — |
| `spans::remap::Parser::remap_node_spans` | 0.8% | 1.1% |
| `inline::gfm_autolink::Parser::apply_gfm_autolinks` | 0.5% | 0.9% |

### render: 7,771 samples at `cb352020`, 7,807 at `060b02d2`

| Symbol | `cb352020` | `060b02d2` |
| --- | ---: | ---: |
| `HtmlRenderer::render_node` | 94.2% | 94.4% |
| `escape::escape_into` | 36.6% | 36.0% |
| `write::HtmlRenderer::write_heading_id` | 11.5% | 11.5% |
| `escape::push_run_long` | 8.6% | 8.5% |
| `write::HtmlRenderer::write_html_value` | 6.4% | 6.5% |
| `heading::slugify_heading_into` | 6.3% | 5.7% |
| `blocks::HtmlRenderer::visit_table_row_with_header` | 5.6% | 6.1% |
| `blocks::HtmlRenderer::render_list_item_with_tightness` | 5.3% | 5.2% |
| `escape::url::write_url_segment` | 4.5% | 4.9% |
| `heading::HeadingIdPlanner::plan_into` | — | 4.4% |
| `tagfilter::matching_tag` | 3.7% | 4.3% |
| `_platform_memmove` | 4.0% | 3.7% |
| `write::HtmlRenderer::visit_inline_node` | 3.5% | 2.9% |
| `escape::write_url_escaped_into` | 2.2% | 2.4% |
| `HtmlRenderer::prepare_render` | 2.3% | 2.0% |
| `hashbrown` `HashMap::insert` | 1.4% | 1.1% |
| `compact_str` `HeapBuffer::alloc_copy` | 0.9% | 1.1% |

## What the first profile pointed at

- **Unicode trimming, 7.2% of parse.** `str::trim_matches` (6.1%) and
  `trim_start_matches` (1.1%) decode UTF-8 to test `White_Space`. Callers of
  `trim_matches`: the table cell closure in `table_row_cells_with_offsets`
  (220 of 492 samples), `parse_paragraph` (113), `parse_list` and the list
  item fast path (95), then table metadata and reference definitions. After
  #414 only a Unicode trim that #414 keeps on purpose remains in the profile,
  in `parse_reference_definition`: 0.2%.
- **The GFM autolink pre-flight, about 7–9% of parse on the `gfm`
  documents.** `gfm_autolink::may_contain_autolink` is inlined into
  `parse_inline_block`, whose self time is 7.0% of all parse samples, and its
  `www.` `memmem` adds 3.4% (274 of the 662 `memmem` samples come from
  `parse_inline_block`). It walks each block's content once more after the
  inline marker scan has read it. A throwaway prototype that answered the
  pre-flight in one standalone NEON pass left the output unchanged and gained
  only about 1% in parsing (quick A/B, not archived). #415 then tried to take
  the second pass away entirely; see [REJECTED.md](REJECTED.md).
- **The root pre-scans, about 5–6% of parse.** `Parser::with_phase` (9.1%
  inclusive) held 4.3% self time, most of it the inlined `memchr` for NUL, and
  called `memmem` for `]:` (227 samples, 2.8%) that runs to the end of every
  document without a reference definition. After #413 the `memmem` call is
  gone; the fused NEON scan shows as `with_phase` self time (6.0%) and the
  function falls to 8.1% inclusive.
- **Render.** `escape_into` is 36% and was measured as a floor in
  [round 2](../2026-09-15-arm-round-2/README.md) (a raw output cursor lost).
  Heading ids cost 11.5% (`write_heading_id`, of it `slugify_heading_into`
  6.3%, the id map's hashing and small-string copies about 2%).
  `tagfilter::matching_tag` held 3.7%, all of it from `write_html_value` in
  the `gfm` profile: every `<` in raw HTML tried all nine disallowed names.

## After the round, at `060b02d2`

The same driver, rebuilt against 2.1.0, completed **753 parse sweeps in 14 s
against 675** at `cb352020`, about 12% more; render sweeps moved by about 1%.
The two runs were hours apart under different machine load, so this is
indicative only: the paired measurement of the three Rust changes together is
parse 1.080 ([cumulative effect](README.md#cumulative-effect)).

What remains at the top:

- **Autolink pre-flight, about 10% of parse**: `parse_inline_block` self 6.4%
  plus its `www.` `memmem` 3.5% (288 samples). Still a second pass per block.
- **Tables, 10.1%** inclusive (`parse_table`), of it `parse_table_cell` 5.7%;
  the cell closure fell from 5.1% to 3.0% with ASCII trimming.
- **Lists, 11.8%** inclusive (`parse_list`).
- **Heading ids, 11.5% of render**: `slugify_heading_into` 5.7% and the new
  `HeadingIdPlanner::plan_into` 4.4% (unique ids and prefixes, #408–#410),
  which now carries the id map's hashing and small-string copies.
- **Tag filter, still 4.3% of render.** The first-byte gate settles tags such
  as `<div`, `<a`, `<br` and `<!--`, but six first letters (`t`, `s`, `x`,
  `i`, `n`, `p`) cover the list, so common tags such as `<td`, `<tr`, `<th`,
  `<span`, `<sup`, `<img` and `<p` pass the gate and walk all nine names.
- **The escaper, 36% of render**, a measured floor.
