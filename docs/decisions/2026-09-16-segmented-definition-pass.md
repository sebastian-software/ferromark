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

## What the walk reads

Only two kinds of line can change a plan: one that holds a candidate, and one
that opens a region. So while no segment is open the walk jumps between them,
taking the nearest of the next fence-run line for either fence byte, the next
`<` or — with `math` on — `$` at an indent of at most three columns, and the
line of the next candidate opener. Each searcher remembers one answer and a
stale one is re-searched from a position past it, so the searches partition the
document and read it once in total. Past the last candidate nothing can matter,
and the walk stops there. Once a segment is open every line is read again,
because the boundary that closes it can be any of them.

Two pieces of state follow. A candidate's segment start is found by walking
back from its line to the nearest boundary, which costs the bytes that segment
is about to parse anyway; the walk back stops at the end of the most recent
tracked region, which is a root position in its own right and therefore a valid
segment start — that also makes segments after a region tighter than a
forward-tracked boundary would be. And whether the parser starts a root block
on a line is decided from the line itself: it is the first line, the line after
a blank one, or a tracked region's end.

Not reading a line that cannot matter is not a change of rules, but it does
have one visible effect: an ambiguous opener past the last candidate no longer
forces a fallback, because the walk never reaches it.

## What is not planned at all

Planning is not free, and two shapes cannot profit from it. Both are decided
before the planner runs, and both choose the whole-body pass this work
replaced, so neither can change what is collected.

A segment builds a parser over the temporary arena, block-parses its own bytes
and hands its tree to the collector. Against the block-parse throughput of the
measured corpora that fixed part is worth on the order of **256 bytes** of
ordinary parsing. A plan with `k` segments therefore costs at least `k * 256`
bytes more than the bytes it actually parses, while it can never skip more than
the span `S` its candidates cover. So when `S / k` is under 256 the plan cannot
pay for itself, whatever it comes out to be. The shape filter stops as soon as
it holds a link candidate and **8** openers — fewer proves nothing, since a
document can open with two adjacent definitions and then hold a megabyte of
prose — whose span averages under that, and reports the source as dense; the
pre-pass maps that straight to the full pass, which also restores the early
return the old filter had for reference-dense input.

The same arithmetic merges two planned segments whose gap is under 256 bytes:
skipping the gap buys fewer bytes than the second segment costs. Merging is
exact — the merged range keeps the earlier start and the later end, and those
are the only offsets the boundary proof rests on.

A body shorter than **64 bytes** is a handful of lines whose plan can only be
the whole body or nothing, so it skips the planner too; its own setup already
costs more than the difference.

These three numbers are performance heuristics, and only these three. Every
plan the planner does produce is exact, and so is every fallback.

## What falls back

The whole body is parsed, exactly as before, when `mdx` or `line_comments` is
on (both give lines meanings these rules do not model), when a `$` line at
indent three or less appears with `math` on, when the body starts with a
byte-order mark, when an ambiguous opener the walk reaches fails the proofs
above, and when either threshold of the previous section says planning cannot
pay. On the 57 broad documents of the frozen corpora no document falls back for
a structural reason; the reference-dense scanner diagnostic falls back on
density, and about one in seven hundred specification examples falls back on an
adversarial container or HTML shape.

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
and no successful output reads the map. So is a source the shape filter called
dense — its opener list is deliberately incomplete, and the planner is never
handed one.

Public regressions render the shapes the boundary rule has to get right:
example 128 for both fence characters and all three line endings, indented
fences inside list items, a root HTML block holding a fence run, an HTML
comment and a `<script>` block spanning blank lines, a type-7 tag after a
paragraph line, definition-list bodies with the option on and off, a closer
longer than its opener, a backtick in an info string, definitions in lists and
block quotes, first-definition precedence across two segments, a footnote and a
link definition in different segments, a lazy continuation that is not a
definition, definitions on the first and last line, an unterminated fence, a
reference-dense document and a twenty-byte one — the two shapes that skip the
planner — and two timing guards that keep planning linear on indented openers
that never close.

The planner's own tests pin the plans the rules produce, including the two
thresholds' effect on segment merging and the walk's early stop: an opener that
would force a fallback is not read when it lies past the last candidate, and is
read when it lies between two.

Specification fixtures, snapshots and conformance baselines are unchanged.

## Measured

Paired against `main` (`e35e9f64`) with the round harness, 3 rounds × 5 pairs ×
40 ms, exact HTML and AST equality verified first
([report](../reports/2026-09-16-definition-segments/README.md)):
`comment-incident` 1.295× fresh / 1.342× reuse / 1.443× parse,
`rust-book-ch00-00-introduction` 1.197× / 1.196× / 1.235×,
`typescript-handbook-advanced-types` 1.061× / 1.041× / 1.095×,
`scan-extension-links` 1.170× / 1.172× / 1.218×; the marker-free copies of the
same documents are unchanged. The 57 broad documents as a whole move
1.007× / 1.009× / 1.009× / 1.002× (fresh / reuse / parse / render) with no
document under 0.968×; the reference-dense diagnostic that the density rule
sends to the whole-body pass is at 1.00×, and the 20-byte one that skips
planning by length at 0.97–0.98×.
