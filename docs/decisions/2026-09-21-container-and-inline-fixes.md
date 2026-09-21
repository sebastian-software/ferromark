# Container and inline fixes from the final v2 review

The final pre-release review of Ferromark v2 found six problems in the parser
and one in the renderer that reached users with the default options: a memory
amplification in nested containers, five inline shapes with quadratic cost,
two departures from CommonMark's laziness rule, a closing-fence rule the
indented opener did not apply, callout bodies that sent inline HTML through the
block renderer, and nesting errors that reported offsets into a container's
stripped copy. This record covers the fixes made on the coordinator's branch;
the renderer, packaging and opt-in parser packages have their own records.

Three of the fixes change rendered output, all of them toward the
specification. The rest change cost or error reporting only.

## Decisions

- **Blank runs share one source-map entry.** A block quote, list item, footnote
  body or definition body hands its sub-parser a stripped copy of its content,
  and every interior blank line of that copy was recorded as its own forty-byte
  `SourceMapLine`. Every nesting level does this again for its own copy, so
  under the default cap of a hundred levels one blank line cost roughly four
  kilobytes: a hundred-level list followed by a hundred thousand blank lines
  (100 KB of input) held 412 MB of resident memory, scaling to about four
  gigabytes per megabyte of input. `SourceMap::push_blank_line` now extends the
  previous entry when it is a blank run this line continues. No node starts or
  ends inside a blank run, so a run mapped as a whole loses nothing a span can
  observe; the interior mapping is approximate only inside the run. The same
  100 KB now peaks at 18 MB, of which the arena's one copy of the content per
  level is the bulk. Memory therefore remains bounded by depth × input, which
  the `max_nesting_depth` documentation states; lowering the cap bounds it
  further.

- **Bracket scans answer from the recorded walk.** Image alt text, the
  `[text][label]` reference label (for links and images) and `^[inline notes]`
  used the plain `scan_balanced`, which walks to the end of the content when
  nothing closes the bracket. A run of openers with a single closer at the end
  therefore walked the whole remainder once per opener: `![`×n `a]` cost 7.9 s
  at 128 KB with the default options, `[a][`×n 4.4 s. They now use
  `scan_balanced_matched`, whose recorded walk already answers every opener
  below the first one — the mechanism the linear link probe introduced for
  `[`. The decisions of that walk depend only on position, so the recorded
  answers are the same the plain scan would give.

- **`]]` is looked up once per slice.** With wiki links on, every `[[` walked
  to the end of the content to find that no `]]` closes it. Like
  `has_closer_from` for `]`, the position of the last `]]` in the slice settles
  it for every opener at once. A run whose `]]` sits inside a code span still
  walks, since the memo only rules a closer out.

- **The autolink trim is linear in the candidate.** GFM autolink literals strip
  trailing punctuation, and the `)` rule recounted every parenthesis in the
  candidate for each `)` it removed, while the `;` rule searched backwards for
  `&` over the whole candidate for each `;`. `http://x.com/` + `)`×n cost 10 s
  at 128 KB with `ParserOptions::gfm()`. The parentheses are now counted once
  and the count of closers lowered as they are stripped, and an entity name is
  walked back over its alphanumerics instead of searched for. The stripping
  rules are unchanged.

- **Retired delimiter runs are unlinked.** Emphasis pairing retires the runs
  strictly inside a pair, and an unequal strikethrough pair retires its whole
  range without building a node. Retired runs stayed in the vector and every
  later opener search stepped over them, so `~a`×n `b_`×m `a~~`×n visited the
  same retired entries once per closer. The runs are now threaded as a doubly
  linked list and a retired run is unlinked, so it is visited once in total;
  a run that can neither open nor close is no longer recorded at all, as the
  specification's delimiter stack never holds one. The depth a retired run
  holds is folded into the live run before it, which is the opener of the pair
  that retired it or a run inside that pair, so the nesting bound still
  measures every node a later pair encloses. Pairing order and results are
  unchanged; the conformance suites and every snapshot pin them.

- **Laziness continues only an open paragraph.** CommonMark 5.1 and 5.2 let a
  line without the container's marker continue a block quote or list item as
  paragraph continuation text, and nothing else. The list item parser accepted
  such a line after any block, and the block quote parser recomputed its
  "paragraph open" state from the closing-fence line and treated HTML block
  and table lines as paragraph text, so `- a\n  ```\n  code\n  ```\nlazy`
  kept `lazy` inside the item and `> ```\n> x\n> ```\nlazy` inside the quote.
  Both containers now consult `lazy_paragraph::OpenParagraph`, a tracker fed
  with the stripped content lines. It follows fenced code and HTML blocks
  with their closing rules, ATX and setext headings, thematic breaks, indented
  code, GFM tables and blank lines, and looks through nested container markers
  so the innermost content decides, the way the sub-parser will. It is a
  tracker rather than a parse: a line is classified from its own bytes and
  the state before it. That is exact for every shape the suites pin, and a
  contrived mix of nested containers that could still fool it ends at the old
  behavior — a line absorbed into, or split off from, the container — never a
  wrong block inside the sub-parse. Definition-list bodies keep their own
  laziness rule; it is opt-in and untouched here.

- **An indented opening fence closes only on a bare fence.** The unindented
  opener already required a closing fence followed by nothing but whitespace
  (`fenced_close_bounds`); the indented opener accepted `` ``` bar `` as its
  closer. Both paths now apply the rule, so `` ``` bar `` is content in both.

- **Callout bodies stay on the inline path.** The marker paragraph of a
  callout is rendered by a routine that strips `[!KIND]`, and it handed every
  non-text child to the block renderer. An inline raw HTML node therefore took
  the HTML block path, which appends a line break after every fragment:
  `> [!NOTE]\n> <b>x</b> y` rendered `<p><b>\nx</b>\n y</p>` while the hooks
  path rendered `<p><b>x</b> y</p>`. Both paths now use the inline writer for
  every child, including the first text run, so a configured `soft_break` also
  reaches every line of the body. Bare-URL autolinking inside callout bodies
  stays off, as before.

- **Nesting errors report document offsets.** A `NestingTooDeep` error raised
  inside a container carried the offsets of the container's stripped copy, so
  `intro paragraph\n\n> > > > a` failed at `Span { 0, 0 }`. The error now takes
  the same span map its sibling nodes take at every container boundary —
  block quotes, list items, footnote and definition bodies, JSX children and
  comment-stripped paragraphs — so the span points at the construct that went
  too deep in the source the caller handed to the root parser.

## Intended output changes

| Input | Before | After |
| --- | --- | --- |
| `- a\n  ```\n  code\n  ```\nlazy` | `lazy` inside the `<li>` | `<p>lazy</p>` after the list |
| `> ```\n> x\n> ```\nlazy` | `<p>lazy</p>` inside the quote | `<p>lazy</p>` after the quote |
| `> <div>\nlazy`, `- <div>\nlazy` | `lazy` inside the HTML block | `<p>lazy</p>` after the container |
| `> \| a \|\n> \|---\|\nlazy` (GFM) | `lazy` as a table row | `<p>lazy</p>` after the quote |
| `> #tag\nlazy` | quote ends before `lazy` | `#tag\nlazy` is one paragraph |
| `` ` ```\nfoo\n``` bar\nbaz\n``` `` | closes at `` ``` bar `` | `` ``` bar `` is content |
| `> [!NOTE]\n> <b>x</b> y` | `<p><b>\nx</b>\n y</p>` | `<p><b>x</b> y</p>` |

Every entry matches what cmark or cmark-gfm produce for the same input. The
CommonMark 0.31.2 and GFM conformance suites and every snapshot suite pass
unchanged: none of them contained the shapes above.

## Verification

Each problem was reproduced in a release build before it was fixed. After the
fix, the 100 KB nested list peaks at 18 MB instead of 412 MB and parses in
110 ms instead of 540 ms; the inline shapes take 2–12 ms at 128 KB instead of
4–10 s, growing about fourfold for fourfold input.

`tests/lazy_continuation.rs` pins the laziness rule and the fence closer,
`tests/callout_inline_html.rs` the callout body on both render paths,
`tests/error_spans.rs` the error offsets through quotes, lists, footnotes and
definition bodies, and `tests/container_scaling.rs` the memory bound through a
counting allocator and the linear growth of every inline shape, best of three
parses under a budget.
