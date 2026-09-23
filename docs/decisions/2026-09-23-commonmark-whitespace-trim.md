# ASCII whitespace at block boundaries

The parser trimmed block content with `str::trim`, `trim_start` and
`trim_end`. Those methods strip every Unicode `White_Space` character, while
CommonMark and GFM only ever remove spaces, tabs and line endings at block
boundaries. The result was a conformance bug and a span bug:
`ferromark::to_html("a\u{a0}")` rendered `<p>a</p>` where cmark keeps the
no-break space, and in `"\u{a0}*a*"` the emphasis span came out as `[0, 3]`
instead of `[2, 5]`. A profile of the 57-document broad corpus also attributed
about 6% of parse time to `core::str::trim_matches`, most of it from table
cells.

## Decisions

- **Non-ASCII whitespace is content at block boundaries.** A new
  parser-internal module, `src/parser/whitespace.rs`, provides byte-level
  `trim`, `trim_start`, `trim_end`, `trim_with_leading` and `is_blank`. They
  strip the ASCII members of `White_Space`: space, tab, line feed, vertical tab,
  form feed and carriage return. Every block-boundary site now uses them, so
  U+0085, U+00A0, U+1680, U+2000–U+200A, U+2028, U+2029, U+202F, U+205F and
  U+3000 survive as content.

- **Vertical tab and form feed keep today's behavior.** The specification's
  wording is narrower than the helper set: paragraph and heading raw content
  loses "initial and final spaces or tabs", and a blank line holds only spaces
  or tabs. Vertical tab and form feed are stripped anyway because `str::trim`
  stripped them before. Keeping them makes non-ASCII whitespace the only
  behavioral difference of this change; whether they should become content is
  a separate decision.

- **Inline offsets start at the trimmed content.** `parse_paragraph` trimmed
  the content but passed the untrimmed `start` as the inline offset, and the
  setext heading path shared that code. `start` is the line start, so every
  inline span in an indented paragraph was shifted left by the indentation
  (`"   *a*"` produced an emphasis span of `[0, 3]` instead of `[3, 6]`), and a
  leading no-break space shifted it by two bytes. Paragraphs, setext headings,
  the comment-filtering paragraph path and the single-line list item fast path
  now pass the offset plus the length of the trimmed prefix. Block spans are
  unchanged: a paragraph still starts at its line.

- **Made-up tab spaces are not source bytes.** Inside a container, and for
  an item that starts with indented code, a tab after a list marker is
  expanded into spaces in front of the item content, and those spaces have
  no bytes in the source. The parsed item records how many there are. The
  one-line fast path leaves them out of the trimmed prefix, and the item's
  sub-source maps them to the point where the real content starts. Without
  this, the new offsets would shift `> -\t*foo*` one to three bytes right.
  The inline spans of such items now match their source text, which they
  did not always do before either: in `> -\tfoo` + `>   bar` the text span
  stopped two bytes short of its end.

- **A lint keeps the rule in place.** `clippy.toml` disallows `str::trim`,
  `str::trim_start` and `str::trim_end`, pointing at the helper module. The
  workspace allows `clippy::disallowed_methods`, and only `src/parser`,
  `src/ast` and `src/allocator` deny it; the latter two call none of the trim
  methods, and the renderer and bindings are unaffected. The remaining
  legitimate parser calls carry a local
  `#[allow(clippy::disallowed_methods, reason = "...")]`.

## Specification references

Line numbers refer to the vendored fixtures in `tests/spec_fixtures/`.

- Blank line: "a line containing only spaces (`U+0020`) or tabs (`U+0009`)"
  (`commonmark-0.31.2-spec.txt`, line 314).
- Unicode whitespace is defined separately (line 319) and used by the
  specification only where it says so, such as delimiter-run flanking.
- Setext heading and paragraph raw content: "removing initial and final spaces
  or tabs" (lines 1371 and 3519).
- Thematic break characters are "each followed optionally by any number of
  spaces or tabs" (line 874).
- The info string is "trimmed of leading and trailing spaces or tabs" and a
  closing fence "may be followed only by spaces or tabs" (lines 1943 and 1960).
- HTML block type 7 ends in "zero or more spaces and tabs" (line 2411).
- A list item that interrupts a paragraph must not begin with a blank line
  (line 4128).
- GFM tables: "Spaces between pipes and cell content are trimmed", and a
  delimiter cell holds only hyphens and optional colons
  (`gfm-extensions-spec.txt`, lines 22 and 25).

cmark's trimming functions classify whitespace with `cmark_isspace`, which
holds the same six ASCII bytes. The offline cmark oracle in
`benchmarks/compatibility-audit/CMARK.md` needs local, pinned cmark and
cmark-gfm source trees, which were not present on the machine that made this
change, so it was not run and nothing was downloaded. The expected outputs
below follow the specification text.

## Intended output change

| Source | Before | After |
| --- | --- | --- |
| `a␣` (`␣` = U+00A0) | `<p>a</p>` | `<p>a␣</p>` |
| `␣a`, and likewise U+2003, U+3000 | `<p>a</p>` | `<p>␣a</p>` |
| `␣` alone | nothing | `<p>␣</p>` |
| `h␣` + `---` | `<h2>h</h2>` | `<h2>h␣</h2>`, matching `# h␣` |
| `\| ␣y␣ \|` as a table cell | `<td>y</td>` | `<td>␣y␣</td>` |
| `\| a \|` + `\|␣- \|` | a table | a paragraph: `␣-` is not a delimiter cell |
| `␣\| a \|` + `\| - \|` | a table | a paragraph: the header has two cells |
| `- ␣item` | `<li>item</li>` | `<li>␣item</li>` |
| `a` + `- ␣` | `<p>a\n-</p>` | `<p>a</p>` and a list holding `␣` |
| `- a` + `␣` + `- b` | loose list, `␣` dropped | tight list, `␣` continues `a` lazily |
| `***␣` | `<hr>` | `<p>***␣</p>` |

The same rule applies to the other block-boundary sites listed below; they
change output only for lines whose leading, trailing or only whitespace is
non-ASCII. The fenced-code info string `␣rust` now reaches the AST as
`␣rust`; the HTML renderer normalizes the language with its own `str::trim`,
so the rendered `language-rust` class is unchanged. The renderer is outside
this change.

Span changes, with no change to the HTML: inline spans in paragraphs and
setext headings indented by spaces, in paragraphs and list items that start
with a trimmed vertical tab or form feed, and after a leading non-ASCII space
now point at the content they cover.

## Sites

The audit covered the 46 lines in `src/parser` that call `str::trim`,
`trim_start` or `trim_end`, plus the other Unicode-aware whitespace checks.

Switched to the helper (38 calls), because each decides block structure or a
block content boundary:

- Paragraph and setext heading content, including the comment-filtering path
  and its inline offset (`block.rs`, 4 calls).
- Table header and delimiter lines, row edges, cells and delimiter cells
  (`table.rs`, 9 calls).
- Thematic breaks (`leaf.rs`), the fenced-code info string (`fenced_code.rs`),
  HTML block type 7 trailing whitespace (`html/start.rs`), and the lazy-paragraph
  tracker's blank lines, setext underlines and closing fences
  (`lazy_paragraph.rs`, `reference/scan.rs`).
- List markers, empty items, paragraph interruption and blank continuation
  lines (`list_item.rs` 5, `list.rs` 3, `list/item_source.rs` 2).
- The bare-marker check in block quote laziness (`block_quote.rs`, 2), and
  blank lines in footnote bodies (`footnote.rs`), definition lists
  (`definition_list.rs` 2, `definition_list/lines.rs` 2), inline-note bodies
  (`inline_footnote.rs`), and MDX JSX flow children (`mdx_jsx.rs`).

Kept on the `str` methods, with a local `allow` and reason:

- Link-label emptiness in reference definitions and full reference links and
  images (`reference.rs`, `inline_link.rs`, `inline/image.rs`). The check has
  to agree with `normalize_reference_label`, which collapses Unicode
  whitespace. Switching the emptiness check alone would accept `[␣]: /u` as a
  definition whose key normalizes to the empty string. The specification
  folds only spaces, tabs and line endings in labels, so label normalization
  is a candidate for its own decision.
- Wiki-link target and label trimming (`inline_link.rs`, 2 calls): extension
  syntax inside an inline, not a block boundary.
- The MDX ESM value (`mdx_esm.rs`): JavaScript source, not Markdown content.
- The spec-fixture reader in the test-only `prepass/equivalence.rs`, which the
  module's existing `allow` covers. `cursor.rs` only mentions `trim_start` in a
  comment.

Not method calls, and unchanged: emphasis flanking (`inline/emphasis.rs`) uses
Unicode whitespace by definition; the GFM autolink boundary test
(`inline/gfm_autolink/scan.rs`) is a per-character inline check rather than a
trim and is outside this change; heading-attribute parsing (`leaf.rs`), which
finds and trims around `{…}` with `char::is_whitespace`, and table-attribute
tokens (`table_attributes.rs`) are opt-in extension syntax and keep Unicode
whitespace; `normalize_reference_label` is covered above.

## Verification

`tests/commonmark_whitespace_trim.rs` covers no-break, em and ideographic spaces
at both ends of a paragraph and on a line of their own, setext headings next to
ATX headings, table cells and delimiter rows, list item content, paragraph
interruption, a no-break-space line inside a list next to a real blank line,
thematic breaks, and the inline spans of `"\u{a0}*a*"`, an indented paragraph,
an indented setext heading and a leading vertical tab. All eight tests fail on
the previous parser. A ninth pins the inline spans of list items whose tab was
expanded inside a container, on the fast path and on the sub-parser path.
`src/parser/whitespace.rs` checks the helper's byte set against
`char::is_whitespace`, keeps every non-ASCII `White_Space` character up to
U+3000, and matches the `str` methods on ASCII input.

The CommonMark and GFM conformance suites and every snapshot are unchanged.
A temporary `str::trim` call in `src/parser` fails `cargo clippy` with the
configured reason, which confirms that the lint resolves the inherent `str`
methods.

## Performance

The helpers avoid the UTF-8 decoding that `str::trim` performs. The effect on
parse time will be measured by the coordinator with the paired
optimization-rounds harness against `main`; this record makes no speed claim.
