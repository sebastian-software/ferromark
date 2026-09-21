# Opt-in extension scans keep what one walk decided

The final pre-release review of Ferromark v2 found quadratic parse cost in
three opt-in extensions. Math (`math`), MDX (`mdx`) and definition lists
(`definition_lists`) each answered "nothing closes this" by reading to the end
of the document, once for every opener, so text that is ordinary under those
options cost O(n²): `$5 for a $10 book` repeated, a shell snippet full of
braces outside a fence, a list of terms under a definition body.

Measured on the release build of `main` (8d957e3), best of three, 32 KiB
against 128 KiB — x16 for every x4 of input, which is the signature — and
again on this branch:

| shape | 32 KiB | 128 KiB | after |
| --- | --- | --- | --- |
| `$a ` | 0.37 s | 5.94 s | 0.7 / 1.1 ms |
| `$1 ` | 0.38 s | 6.22 s | 0.7 / 1.1 ms |
| `$$a ` | 0.54 s | 6.72 s | 0.7 / 1.4 ms |
| `$1 ` with one `$x$` behind it | 0.54 s | 8.19 s | 0.7 / 1.1 ms |
| `$a ` with `` `$` `` behind it | 0.47 s | 5.97 s | 0.7 / 1.6 ms |
| `$$ a` lines | 0.29 s | 3.71 s | 0.7 / 1.4 ms |
| `<A>` with one `</A>` behind it | 0.72 s | 9.37 s | 0.7 / 2.3 ms |
| `<A>` over a `{` run | 0.41 s | 5.27 s | 1.1 / 2.2 ms |
| `<A>` over a `{` run with one `}` | 0.83 s | 10.73 s | 1.6 / 3.2 ms |
| the same, with `</A>` behind it | 0.98 s | 17.47 s | 1.7 / 3.1 ms |
| `{` run with one `}` behind it | 0.33 s | 5.38 s | 1.3 / 2.2 ms |
| `<A {>`, an unclosed attribute expression | 0.09 s | 1.29 s | 0.7 / 3.0 ms |
| `<A x={>` | 0.05 s | 0.81 s | 0.2 / 0.8 ms |
| `</A {>` inside an element | 0.12 s | 2.00 s | 0.3 / 1.3 ms |
| lazy definition-body lines | 0.04 s | 0.82 s | 0.1 / 0.5 ms |

Every one of them now grows about x4 for every x4 of input. Ordinary
documents are unchanged or slightly faster — 128 KiB of `$x$ text ` 0.38 s
against 0.36 ms, of `<A x={1}>t</A>` 0.88 ms against 0.81 ms — except a
document that is nothing but short `{x}` expressions, which pays about 4% for
the brace record it no longer needs.

## The invariant every memo rests on

Each of these scans decides its answer from the position it stands on and the
bytes around it, never from where the scan began. Two scans that both reach a
position therefore agree from there on, which is what lets one walk answer for
the openers behind it. It is the same argument the
[linear link probe](2026-09-17-linear-link-probe.md) records for
`scan_balanced_matched`, applied to four more scans. Where a scan *steps over*
a region — a code span, a string, a comment, a tag body — it has not read the
positions inside it, and a start in there is not one it may answer for.

## Decisions

- **The inline-math closer is memoized in two layers**
  (`src/parser/math.rs`). A `$` opener scans every later `$` to the end of the
  content before it can report that nothing closes it, and unlike the `^`/`~`
  script spans it keeps going past a candidate that cannot close. The first
  layer records the first `$` at or after a scan start that *could* close an
  opener of that width, tested on that byte and its neighbors alone. Skipping
  a code span only ever removes candidates from the real scan, so "no
  candidate at all in the suffix" is an answer no scan start can disagree
  with, and where no backtick stands between the start and the candidate the
  two scans read the same bytes and reach the same `$`. The second layer
  catches the run whose only candidate sits inside a code span: a scan that
  ends in `None` records the ranges it read byte by byte — the ones between
  the code spans it stepped over — and a later start inside one of them is
  answered `None` without a walk. A start inside a stepped-over region scans
  for itself, because the bytes that hid it from that walk need not hide it
  from the parse: in ``$a <i t="`">$b$ x` z`` the backtick belongs to a raw
  HTML tag, so `$b$` is dispatched from inside a region the first scan skipped
  and still closes.

- **The display-math terminator is memoized per source.** Whether a byte ends
  a `$$` block depends on that byte and its neighbors, so the first terminator
  at or after one start is the first for every start up to it. The block
  dispatch and the block-start probe now share one walk per run instead of
  taking one each per line.

- **The `*`/`_` search behind the digit-prefixed probe is a forward window.**
  `$1` opens math only when something closes it *and* an emphasis marker
  stands between the two, and that search spanned the whole distance to the
  closer. One forward window over the slice answers it for every opener.

- **The JSX closer walk records every opener it passes**
  (`src/parser/mdx_jsx/scan.rs`). `<A>` repeated nests one level per tag, so
  every opener but the innermost is unclosed and each one used to walk the
  rest of the slice for itself. One walk keeps the opener stack it otherwise
  only counts and reports each opener at the tag that returns the walk to that
  opener's own depth. That walk is also the search: an opening tag asked
  whether anything closed it and then walked again to find out where, and both
  answers now come from the one walk (`Parser::mdx_jsx_close`), which is why
  an ordinary document of MDX elements parses slightly faster than before.

- **MDX expressions take the matching `}` from one brace walk**
  (`src/parser/mdx_jsx/braces.rs`). The balanced scan reports that nothing
  closed only after reading to the end of the content, and the cheap
  last-closer guard in front of it is defeated by a single `}` behind the run.
  One walk records the match of every brace it passes. Every scan that steps
  over a brace asks that record rather than the plain one: the closing-tag
  walk, the tag skip inside it, and the opening-tag scan the flow and inline
  dispatch run, all of which take the brace skip as a parameter. An attribute
  expression that never closes is the case the opening-tag scan pays for —
  it runs for every `<` the dispatch reaches, before any closer walk starts —
  and a run of `<A {>` cost one read to the end of the slice per tag.
  `braces::skip_braces` survives as the reference the tests hold the record
  against, as `scan::find_matching_close` does for tags.

- **The definition-list term scan records the window it walked**
  (`src/parser/definition_list.rs`). Every non-indented continuation line of a
  body asks whether a new item starts there, and the term scan walks forward
  to the next blank line to answer. When that walk ends without an item, it
  ends the same way for every start inside the run it walked, so the window is
  recorded and each line is examined once. The window is not recorded when the
  scan stops before counting a term line, because a start inside the skipped
  comment lines there can answer differently.

- **Unclosed runs are recorded as ranges, not as entries.** A memo that keeps
  one record per opener costs more than the walk it replaces once the run is
  long: 512 KiB of `<A>` spent 44 ms almost entirely in the map. The closing-tag
  and brace walks therefore record only what *closes*, plus one range covering
  the openers they left open. The range starts after the last region the walk
  stepped over that could hide what is being looked for — a string or comment
  for braces; an expression, a backtick run or a tag whose own body holds a
  `<` for tags — and openers before that point are still recorded one by one,
  so a run interleaved with such regions stays one walk. The same 512 KiB now
  costs 13 ms. Neither walk records the opener it started from: that answer is
  its return value, and a document of independent elements would otherwise pay
  a record for every one of them.

## The one output change

`has_mdx_jsx_closer` is now keyed by the opening tag it answers for — the
slice, the position the tag ends at, and the tag name — and not by the slice
and name alone.

The memo added in [#377](https://github.com/sebastian-software/ferromark/pull/377)
keyed it by the slice and name, so the first opening tag's verdict was handed
to every later tag in the same slice. That is wrong in the direction that
loses nodes: a tag whose closer the walk cannot see, because a backtick run or
an expression stands in the way, caches "no closer" for tags the same closer
does close. With `mdx: true`:

```
<A>`
<A>x</A>
`
```

`main` renders the second line as inline JSX inside a paragraph; every build
before #377, this branch, and a build of this branch with the memo removed
render it as the flow element the line is. `main` is the outlier, and the fix
restores what the parser did before the memo existed. The shape is pinned in
`tests/extension_scaling.rs`.

Five of 60,000 generated sources under seven option sets differ from `main` in
this way; nothing else does.

## Validation

Output equality first, timing after. A corpus of 10,650 generated adversarial
sources built from 71 units — each unit alone, repeated, every ordered pair,
and every unit run with one closer of six kinds behind it — plus the 415
Markdown documents in the repository, and 60,000 seeded token-salad sources
from the same units, each rendered under seven option sets (CommonMark, GFM,
math, MDX, definition lists, every extension, and every extension at a nesting
cap of four) is byte-identical in HTML *and* in the AST `Debug` against a build
of the same branch with every memo here replaced by the plain scan it stands
in for. Against `main` the generated corpus and every repository document are
byte-identical, and five fuzz sources differ, all of the class above.

`src/parser/math/tests.rs`, `src/parser/mdx_jsx/scan/tests.rs` and
`src/parser/mdx_jsx/braces/tests.rs` pin each memo against the plain scan from
every position of a hand-picked corpus, cold and after filling, including the
claim each recorded range makes about the positions it covers.
`tests/extension_scaling.rs` pins the cost of fifteen shapes and the rendered
shape each extension specifies. `cargo test --workspace --all-features
--locked` passes with no `.snap.new` file, so the snapshot suites and the
CommonMark 0.31.2 and GFM conformance baselines are unchanged.
