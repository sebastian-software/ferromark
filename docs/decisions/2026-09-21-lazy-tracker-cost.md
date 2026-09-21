# The laziness tracker runs on demand, and the parser's memo tables allocate on first use

Rerunning the six-engine native comparison before the 2.0.0 release
([2026-09-21-native-release](../reports/2026-09-21-native-release/README.md))
found v2 at `bffc89f6` 7–12% slower relative to every engine than the
preceding run at `7c887a2b` had measured, on the same 57 frozen documents,
with identical HTML. The paired harness
([2026-09-21-release-fixes-paired](../reports/2026-09-21-release-fixes-paired/README.md))
confirmed it against the `7c887a2b` core on `main` (`c4af9525`): parse 0.894×,
fresh 0.929×, reuse 0.922×, render 1.000× over the 57 documents, 56 of 57
slower in parsing, comments with task lists and block quotes down to 0.62×.
Screening every intermediate revision of the range attributed it: #369, #376,
#377, #380 and #382 cost nothing; #372 and #374 together cost about 2.5% of
parse time on the 16 most affected documents; **#383 cost 17%** and #384 a
further 2% on the same documents.

## What #383 did

[Container and inline fixes](2026-09-21-container-and-inline-fixes.md) gave
block quotes and list items a tracker, `lazy_paragraph::OpenParagraph`, that
follows the container's stripped content line by line so laziness continues
only an open paragraph. The tracker is exact, and it stays. What it cost is
how it classified a line: every line went through the thematic-break, ATX,
setext, fence, HTML-block-start and type-7 HTML checks and the marker
stripper's list-marker probes in turn, each returning "no" after its own
prefix work. A container's paragraph text — nearly every line a container
holds — therefore paid a dozen small scans on top of the sub-parse that reads
the same bytes again. On `comment-checklist` (four task items, 287 bytes)
that was 230 ns on a 390 ns parse.

The type-7 check was also wrong on its own terms. `is_html_block_type7_line`
hands the line to the inline tag scanner, which assumes its caller has seen
the `<` and starts reading a tag name at the second byte. Asked about `ab>`
it read a tag, the tracker ended the paragraph, and `- ab>\nlazy` rendered
`lazy` outside the item where CommonMark keeps it inside. The block parser
never reached this path because it checks the first byte before it asks.

## Decisions

- **The tracker runs on demand.** A block quote or list item collects its
  stripped content into one text anyway, and it only needs the tracker's
  answer when a line without its marker turns up that could continue it —
  after a blank line, an indented continuation or a sibling marker the
  question never arises. `OpenParagraph::catch_up` now observes the lines
  collected since it last answered, from that text, in the order the
  collector produced them, and the collectors call it at that one point
  (`list.rs`, `block_quote.rs`). A list item that has not materialized its
  text yet observes its first line alone and resumes after it once the text
  exists; a line comment the collector copies verbatim was never observed
  and is skipped past. Every line is still observed at most once, in the
  same sequence as before, so every answer is the one eager observation
  gave; a document whose containers end at blank lines never pays for the
  tracker, and one full of lazy lines pays what it paid before.

- **A line is classified from its first byte.** Every block start the
  tracker recognizes is decided by the first byte of the trimmed line: `-`,
  `*` or `_` for a thematic break, `#` for an ATX heading, `=` for a setext
  underline, `` ` `` or `~` for a fence, `<` for an HTML block of any type,
  a bullet or a digit for a list marker. `observe` matches on that byte and
  enters a classifier only for its own alphabet, and the marker stripper
  returns any line whose first byte is not a bullet or a digit before it
  probes. The thematic-break check trims Unicode whitespace itself, so a
  first byte that could begin such whitespace still reaches it. The same
  classifiers run with the same inputs on the lines that can start a block;
  the table, fence and HTML-block state machines are untouched.

- **The memo tables allocate on first use.** The review's memos — the last
  `]`, `>` and `}` per slice, the link-probe and bracket-match maps, the
  MDX closer, brace, math and wiki tables and their gap cells — were all
  held inline in `Parser`, which grew it from 272 to 736 bytes between
  `7c887a2b` and `bffc89f6`. Every parse zeroed and moved that struct, and
  every block quote and list item builds a sub-parser, so a comment with
  four items moved more memo state than it had input; on `comment-ack`
  (37 bytes, a 50 ns parse) that was the whole of the loss the tracker fix
  left behind. The three core-syntax maps now sit behind one `OnceCell`
  each and are allocated in the arena the first time they are consulted;
  the nine opt-in tables share one lazily allocated struct; the three
  cells the definition-list and bracket scans read on every probe stay
  inline, so a rejected probe still allocates nothing. The parser is 336
  bytes. Sub-parsers get fresh, empty cells as before, so no memo answers
  across a container boundary that did not before.

- **The closer memo answers repeats from one cell, and bracket matches
  are read without allocating.** With the tracker and the struct repaired,
  what was left sat on link-dense encyclopedia prose: every `[` asks
  `has_closer_from` whether a `]` follows in the slice and then
  `scan_balanced_matched` whether a recording walk already answered it —
  two hashed table lookups per bracket, from the linear link probe (#374).
  The `[` of one paragraph all ask about the same slice, so the last answer
  now sits in a cell in front of the table, and the recorded-match table is
  read through its `OnceCell` without being allocated: only a recording
  walk — a nested run — creates it, and a parse that never met one answers
  every `[` with a null check. The table stays the record; the cell mirrors
  its last entry, so every answer is the one the table gives.

The one behavioral difference is the one above. `is_html_block_type7_line`
is now only asked about lines that start with `<`, so `- ab>\nlazy` and
`> ab>\nlazy` keep `lazy` in the container, as cmark does.
`tests/lazy_continuation.rs` pins both shapes and the real type-7 case.

## Verification

The CommonMark 0.31.2 and GFM conformance suites, every snapshot suite and
the laziness, container-scaling and error-span tests pass unchanged apart
from the added test. The native comparison's 342 archived HTML outputs are
byte-identical before and after.

Paired against `c4af9525` with the same harness, the fixed core measures
0.982× fresh, 0.978× reuse, 0.967× parse and 1.000× render over the 57
broad documents, from 0.929×, 0.922×, 0.894× and 1.000× before it; on the 16
documents the native comparison had moved most, parse went from 0.796× to
0.975×, which is where the last revision before #383 stood (0.977×).
`comment-checklist` is back from 0.62× to 0.96×, `comment-quote` from 0.65×
to 1.03×, `vite-docs-api-plugin` from 0.76× to 0.95×. Each stage of the fix
is measured separately in the paired report.

What remains — about 3% of parse time and 2% of fresh processing over the
57 documents, up to 7% of parse time on link-dense encyclopedia prose and on
sub-microsecond comments — is the cost of #372 and #374: the emphasis
nesting bound threads a depth cell through every inline context, and the
linear link probe still checks a cell and a null per bracket. Those are the
cap and memo mechanisms the review introduced on purpose, and they are
recorded here as measured, not removed. The records that introduced them
measured on a sandbox that could not resolve changes under 20%, which is
why a few percent went unseen; the paired harness resolves 1%.
