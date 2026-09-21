# MDX flow lines, list-marker tabs, and type-1 HTML block ends

Three conformance corrections from the final v2 review. All three change
parse output on purpose, each against a written rule: the MDX flow-tag rule
of `micromark-extension-mdx-jsx` and two CommonMark 0.31.2 sections. The
CommonMark and GFM conformance baselines and every snapshot stay as they
are; one existing test pinned the behavior being corrected and is updated
here.

## A flow JSX element owns its line (`src/parser/mdx_jsx.rs`)

### Decision

With `ParserOptions::mdx`, a paired JSX tag at the start of a line is a
`MdxJsxFlowElement` only when nothing but whitespace follows its matching
closing tag on that line. Anything else makes the tag text JSX: the line is
a paragraph holding an `MdxJsxTextElement` followed by the trailing content.
Self-closing tags were already checked this way; the paired case now uses
the same check, on the closing tag.

The rule applies to the closing line of a multi-line element too. Such input
(`<A>`, content, `</A> tail`) is a syntax error in MDX proper; here the line
falls back to its Markdown reading — a type-7 HTML block — exactly like an
element whose closing tag never arrives.

### Why change

`micromark-extension-mdx-jsx` parses flow JSX per line: after a tag it
accepts whitespace, another tag, or the end of the line, and rejects
anything else, which makes the whole line text JSX inside a paragraph. The
parser applied that to self-closing tags only, so with MDX enabled:

| Input | Before | After |
| --- | --- | --- |
| `<A>x</A>- item` | island + `<ul><li>item</li></ul>` | one paragraph |
| `<A>x</A>*` | island + an empty list item | one paragraph |
| `<A>x</A> *em*` | island + `<p><em>em</em></p>` | one paragraph |

The trailing text was not merely misplaced, it was re-parsed as a block: a
list marker, a thematic-break candidate or a paragraph appeared out of a
line the author wrote as prose. The same tag one column further right —
`t <A>x</A> y` — was already one paragraph with a text element, so the
line-start case was also inconsistent with the mid-line case.

`tests/mdx_jsx_remainder.rs::nested_fragments_match_by_depth` pinned the old
reading of `<><>inner</>outer</>`: a nested *flow* fragment plus a separate
paragraph for `outer`. The inner fragment is followed by `outer` on its
line, so it is now text JSX inside the outer element's paragraph, which is
what MDX produces. The test keeps its purpose — the outer fragment still
closes on the second `</>`, which is what depth matching is for — and now
asserts the whole tree instead of counting flow elements.

## Tabs after a list marker expand to a tab stop of four (`src/parser/list_item.rs`)

### Decision

The whitespace run after a list marker is measured in columns, with tabs
advancing to the next multiple of four from the marker's own column
(section 2.2). One to four columns of separation put the item's content —
and the indentation its continuation lines need — at the column the run ends
on. Five or more keep the existing rule from section 5.2: the item starts
with indented code, its content offset is the marker width plus one, and the
remaining columns stay in front of the content.

Column-exact expansion applies where the columns are known: to the document
parser, whose lines are the source's own. A container re-parses its content
with the prefix stripped, so column 0 of such a line is not column 0 of the
source line, and the width of a tab after a marker can no longer be
recovered — the tab in `> -\tfoo` is one column wide, the one in `-\tfoo`
three. Stripped sub-sources therefore keep the narrowest reading a tab can
have, one separating column, which is what they already did. It never
splits an item that the real column would hold together, and the branch
between the two rules does not depend on the starting column at all: one
tab always lands one to four columns on, two always five or more, so
`-\t\tfoo` is an item holding indented code wherever it appears. Carrying
the real column into container sub-parsers would fix the remainder and is
left for the work that owns that plumbing.

So `-\tfoo` has its content in column 4, not column 2:

```text
-<TAB>foo

  bar
```

parses as a one-item list followed by `<p>bar</p>`, because two spaces no
longer reach the content column.

### Why change

The tab branch applied the "indented code" rule to every tab, whatever
column it stopped at, and gave the item a content indent of marker width
plus one. A tab after `-` in column 0 reaches column 4, which is three
columns after the marker — inside the one-to-four range, so the content
starts in column 4. With a content indent of 2, the two-space line above
became a second paragraph *inside* the item, and the list silently swallowed
text that belongs after it.

The five-or-more branch is unchanged and still produces spec example 9
(`-\t\tfoo` is an item holding a code block of `  foo`), which is what made
the single-tab case easy to miss: both tabs and one tab were handled by the
same rule, and only the two-tab example is in the spec.

`cmark` measures the same thing in absolute columns: after the marker it
advances one *column* at a time over the following whitespace, partially
consuming a tab, and uses that count as the item's padding. That count is
three for `-\tfoo` at the start of a line and one for the same item inside
`> `, which is why the correction is scoped to the parser that still knows
which of the two it is looking at.

## A type-1 HTML block ends at any raw-text end tag (`src/parser/html.rs`)

### Decision

A block opened by `<pre`, `<script`, `<style` or `<textarea` ends on the
first line containing `</pre>`, `</script>`, `</style>` or `</textarea>` —
any of the four, compared without regard to case, and the end tag has to be
complete, `>` included. The block kind no longer carries the tag that opened
it (`HtmlBlockStart::Type1` lost its payload, and `Type1HtmlBlockTag` is
gone with it).

### Why change

Section 4.6, start condition 1, states the end condition as "line contains
an end tag `</pre>`, `</script>`, `</style>`, or `</textarea>`
(case-insensitive; it need not match the start tag)". The parser searched
for its own tag only, and searched for `</name` without the closing `>`.
`commonmark.js` uses one pattern for all four,
`/<\/(?:script|pre|style|textarea)>/i`, which is both halves of the rule.

Before, `<pre>` / `x` / `</script>` / `y` / blank / `z` was a single raw
block covering the whole document, including `z`; it now ends on the
`</script>` line and leaves `<p>y</p>` and `<p>z</p>`. In the other
direction, an incomplete `</script` used to close the block; it no longer
does, and neither does `</pre >`.

## Validation

New regression tests, each verified to fail with the corresponding fix
reverted and to pass with it:

- `tests/mdx_jsx_flow_lines.rs` (7 tests): the three reported inputs and
  their rendered HTML, agreement between the line-start and mid-line forms,
  tags that are alone on their line staying flow (including trailing spaces,
  a tab, and end of input), the self-closing rule that is the model for this
  one, the multi-line fallback, and the rule inside a flow element's
  children.
- `tests/block_conformance_fixes.rs` (11 tests): bullet and ordered markers
  followed by a tab, continuations that do and do not reach the content
  column, mixed spaces and tabs, spec example 9, tabbed siblings and a
  nested list, and the narrow reading a block quote's content keeps; and
  for HTML, an end tag from another raw-text tag, case insensitivity, an
  end tag on the opening line, and the incomplete `</script`, `</pre >` and
  `</prefix>` forms.
- `src/parser/tests.rs`: `find_closing_tag_matches_case_insensitively`
  becomes `find_type1_end_tag_matches_any_raw_text_tag_case_insensitively`
  for the renamed scanner, covering all four tags and the `>` requirement.

Full-suite results on this branch: `cargo test --workspace --all-features
--locked` passes; `cargo test --locked --test spec_commonmark --test
spec_gfm` passes with the baselines unchanged — all 652 CommonMark 0.31.2
examples in both the core and GFM modes, all 28 frozen GFM extension
examples, and the same three known GFM autolink exceptions as before. No
snapshot file changed (`git status` on `tests/snapshots` and
`tests/snapshot_*` is empty), so nothing outside the three corrected rules
moved.

Two behaviors that were left alone on purpose, because they are outside
these rules: a task-list marker after a tab (`-\t[x] a`) is still not
recognized as a checkbox, as before; and `-\t\tfoo` keeps its section 5.2
reading. Both are unchanged by this work.
