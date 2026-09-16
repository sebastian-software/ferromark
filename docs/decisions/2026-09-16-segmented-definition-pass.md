# The structural definition pass runs on candidate segments

## Decision

The document-wide discovery of link reference definitions and footnote
definitions keeps the real block grammar as its only authority, but no longer
block-parses the whole document to use it. A planner turns the cheap `]:` shape
filter's candidate positions into byte ranges, and the collection phase parses
those ranges instead — in document order, into one collector, so
first-definition precedence is unchanged.

Every range starts and ends at a *safe root boundary*: a line start where the
real parser is provably at the document root with every container and leaf
block closed. Parsing `&source[start..end]` with a fresh `Definitions`-phase
parser therefore yields exactly the definitions the full parse yields for that
region. No parser change was needed for the sub-slices; the collector only
copies `identifier`, `url` and `title`.

When the planner cannot prove a bound it returns the whole body and nothing
about the pass changes. When no candidate survives, no allocator and no parser
are built at all.

## Why change

`build_prepass` decides from a `]:`-driven shape scan whether a document *may*
hold a definition, and the structural pass then re-parsed the entire document
with the block grammar. One `]:` line was enough, wherever it sat. Paired
measurements on the frozen corpora attribute +37 % to that second pass on a
1.1 KiB support comment whose only `[help]: /getting-started` line sits inside a
fenced example and is correctly rejected, and +23 % on 10.7 KiB of prose with
three real definitions. A 1204-line TypeScript handbook page was block-parsed
twice because all thirteen of its `]:` lines are index signatures inside fenced
code.

The flat scanner this design replaced was removed for good reason
([2026-09-15-refactoring](2026-09-15-refactoring.md)): it approximated block
structure and got container boundaries wrong. Nothing here restores it. The
planner never decides what a definition *is*; it only decides which bytes the
grammar has to see.

## The boundary predicate

A line start `q` is a boundary when `q == 0`, or when all of the following
hold:

* the previous line is blank (only spaces and tabs before its terminator, for
  LF, CRLF and CR alike);
* `q` begins a column-0 content line — the byte at `q` is neither a space, a
  tab nor a line ending;
* `q` is not inside an opaque region the planner is tracking;
* not (`definition_lists` is on and the byte at `q` is `:`);
* `q` does not begin a byte-order mark.

### Why column 0 after a blank line is trustworthy

Only two things can put a column-0 line inside a container: an explicit marker,
and lazy paragraph continuation. A boundary excludes both.

* **Lists.** A list item's continuation indent is at least two columns, so a
  column-0 line can never be item content. After a blank line the item
  continues only through a line indented that far, and the list continues only
  through a sibling marker. A column-0 marker after a blank line therefore
  starts a fresh list when the segment is parsed on its own — and each item's
  children are a function of its own marker line, so no definition moves.
* **Block quotes.** A blank line ends a block quote outright. Lazy continuation
  is what would otherwise let a column-0 line join one, and it is blocked right
  after a blank line and while a fence is open inside the quote. Every opener
  that could start a region also starts a block, which ends the lazy
  continuation and closes the container — which is exactly what makes
  CommonMark 0.31.2 example 128 (`> ```\n> aaa\n\nbbb`) behave: `bbb` is a root
  paragraph, and a later root definition is not hidden.
* **Leaf blocks.** Indented code, footnote-definition bodies, definition-list
  bodies and table bodies all stop at a non-blank line indented fewer than four
  columns. Paragraphs, headings, thematic breaks and setext underlines end at
  the blank line.

### Why definition-list `:` lines are excluded

With `definition_lists` on, a column-0 `:` line after a blank line is a body
line that continues the definition list above it, so the parser is inside a
`DefinitionList` there rather than at the root. Such a line is never a
boundary; the segment reaches back past the term instead. A term line after a
blank line *is* a boundary: it continues the same list in the full parse, but
each item is collected from its own start, so splitting changes nothing.

### Why a byte-order mark is excluded

Parser construction strips a mark from the start of its source. A segment that
began on one would lose three bytes the full parse keeps as ordinary content,
so a mark at the start of the body falls back and a mark elsewhere is not a
boundary.

## Opaque regions

Constructs that span blank lines would otherwise hide a later boundary, so the
planner tracks them — and only from column 0, where the argument above says the
line is at the root:

* **Fenced code.** A column-0 line accepted by the parser's own `fence_open`
  opens one, including the rule that a backtick fence's info string may not
  contain a backtick. It ends where `fenced_close_bounds` says, which is now the
  one implementation both fenced-code parsing and the planner call.
* **HTML blocks.** `parse_html_block_start` classifies the line and
  `html_block_end` closes it, again shared with the real parser: types 2-5 end
  on the line holding their terminator, type 1 on the line holding its closing
  tag, types 6 and 7 before the next blank line.
* A type-7 line is a block start only when the previous line is blank or it is
  the first line. Type 7 is the one HTML kind that never interrupts a
  paragraph.

Inside a region nothing is interpreted: a fence run or a `<` line in a
`<div>` block is HTML content, not an opener. Candidates inside a region are
dropped, because code and raw HTML cannot contain a definition.

## Lines the planner cannot place

Two shapes are genuinely ambiguous, and both are resolved by proof rather than
by tracking:

* **An opener indented one to three columns** is either a root block or a list
  item's body. Four columns would be indented code, and every opener here
  interrupts a paragraph, so it cannot be a lazy continuation. Misjudging it
  would flip the open/closed state of every later region.
* **A column-0 type-7 HTML line after a non-blank line** is either a block or
  paragraph text, and it can even be a container's lazy continuation.

For both, the planner locates the end the "block" reading would give, with the
closer search truncated at the next column-0 content line. The truncation both
bounds the search and settles it: a closer found inside the truncated view is
the one the untruncated search would find, while a region that runs out of
input could reach past that column-0 line. The opener may then be skipped when
nothing inside it can begin a root block under either reading, which two proofs
establish:

* the region covers only the opener's own line, so it has no interior; or
* the region closes before the next column-0 content line, every non-blank
  interior line is indented at least as far as the opener, and the opener
  interrupts paragraphs. The block reading makes the interior opaque, and the
  container reading keeps the enclosing item open through all of it, because an
  item whose content includes the opener has a continuation indent no greater
  than the opener's.

Candidates inside such a skipped region stay alive: the two readings may
disagree about where the region ends, so an ordinary segment has to cover them
and let the grammar decide. Skipping rather than walking the interior is also
what keeps planning linear — every closer search either ends the walk or covers
a range the walk then jumps over.

## What falls back

The whole body is parsed, exactly as before, when `mdx` or `line_comments` is
on (both give lines meanings these rules do not model), when a `$` line at
indent three or less appears with `math` on, when the body starts with a
byte-order mark, and when an ambiguous opener fails the proofs above. On the
57 broad documents of the frozen corpora no document falls back; about one
percent of the specification examples do, all of them adversarial container and
HTML shapes.

## How it is tested

A differential harness collects definitions and footnote labels twice for the
same source — once from the planner's segments, once from the whole body — and
requires the two collections to be identical, with every kept candidate inside
a segment and the segments disjoint and ascending. It runs over every example
of the bundled CommonMark 0.31.2 and GFM fixtures including CR and CRLF
variants, over every document of three frozen measurement corpora, and over
generated token soup built from the shapes that decide a boundary, a candidate
or an opaque region, each under seven option sets (default, GFM, footnotes,
definition lists, math, tables, and GFM without tables). A document the block
grammar rejects for nesting depth is skipped: the real parse reports that error
and no successful output reads the map.

Public regressions render the shapes the boundary rule has to get right:
example 128 for both fence characters and all three line endings, indented
fences inside list items, a root HTML block holding a fence run, an HTML
comment and a `<script>` block spanning blank lines, a type-7 tag after a
paragraph line, definition-list bodies with the option on and off, a closer
longer than its opener, a backtick in an info string, definitions in lists and
block quotes, first-definition precedence across two segments, a footnote and a
link definition in different segments, a lazy continuation that is not a
definition, definitions on the first and last line, an unterminated fence, and
two timing guards that keep planning linear on indented openers that never
close.

Specification fixtures, snapshots and conformance baselines are unchanged.

No speed claim is made here; the measured numbers belong to the separate
report.
