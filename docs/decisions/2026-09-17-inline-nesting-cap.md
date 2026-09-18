# Inline nesting counts toward `max_nesting_depth`

Two decisions, one bound: bracket nesting (issue #349) below, and the
delimiter-run pairing that builds emphasis (issue #371) further down.

## Bracket nesting (issue #349)

### Decision

Every construct that re-enters inline parsing on a bracketed slice — link
text, image alt text, wiki-link labels, inline notes, superscript and
subscript spans, and inline JSX phrasing — counts one nesting level, the same
way block quotes, list items, footnote definitions and JSX children already
count one. Input nested deeper than `ParserOptions::max_nesting_depth` fails
with `ParseErrorKind::NestingTooDeep` instead of recursing.

The default stays `100`, for inline content as for blocks, and `0` still
means unlimited. Inline depth and block depth are counted separately against
that one limit: a document may be 100 containers deep *and* 100 brackets deep.
They do not have to share a budget, because a block container cannot occur
inside inline content, so along any one path the two counts are summed at most
once — the parse holds at most `max_nesting_depth` block frames plus
`max_nesting_depth` inline frames at a time. Nothing that used to parse within
the block cap loses its inline budget to it.

This changes accepted input. Bracket nesting past the cap used to render (when
it did not abort the process); it is now refused. No hand-written document
reaches 100 levels of nested brackets, and the shapes that do are the
pathological ones below.

### Why change

`max_nesting_depth` was enforced in one place, `parse_block`, which only sees
constructs that re-enter the parser on a sub-source. Inline parsing re-enters
`parse_inline` on the same parser and was never counted, although the option
documentation claimed the cap "bounds the recursion depth of a parse no matter
how the constructs are combined". Measured on the release build of
`examples/render_stdin` (x86_64 Linux) before the fix:

| Input | Size | Result |
| --- | ---: | --- |
| `[`×8000 `a` `]`×8000 | 16 KB | stack overflow, abort (8 MB stack) |
| `[`×1000 `a` `]`×1000 | 2 KB | stack overflow with a 1 MB stack |
| `![`×16000 `a` `](u)`×16000 | 96 KB | stack overflow (8 MB stack) |
| `[ `×50000 `]`×50000 | 150 KB | stack overflow (8 MB stack) |
| `[![`×5000…, `^[`×20000…, `[[`×5000… | 20–60 KB | stack overflow (8 MB stack) |

The same input through the Node addon segfaulted the process with no
JavaScript error. A stack overflow is not a panic: `catch_unwind` and the
`panic = "unwind"` release profile cannot recover from one, so the host
process dies. That contradicts the "untrusted by default" contract the cap
exists to keep, which is why this is a release blocker for 2.0.0 rather than a
hardening task.

The count lives at the entry of `parse_inline` rather than at each nested call
site, so a construct cannot be added later that recurses without counting. One
consequence needed a deliberate choice: `parse_link` has to parse its bracket
text speculatively before it can tell whether the outer bracket is a link at
all, and that probe used to swallow every error from the sub-parse. A depth
failure is a verdict on the document, not on this bracket text, so the probe
now propagates that one error kind and keeps swallowing the rest. Without it
the literal-bracket fallback would go on to probe the next opener at the same
depth: bounded, but quadratic per level over the whole run.

### Validation

`tests/nesting_depth.rs` runs each shape above at N = 20000 in the default,
GFM and all-options profiles, at the exact 100/101 boundary, under a custom
cap, and on a `stack_size(1 << 20)` thread — the Windows main-thread default,
the stack that 2 KB of brackets used to overflow. It also pins that block
depth does not spend the inline budget. `tests/nested_links.rs` keeps its
depth-64 parses inside the cap and adds the refusal past it;
`tests/input_panics.rs` covers the hostile inputs end to end. The Node test
suite covers the addon call that segfaulted.

Two shapes parse to a document rather than failing when every extension is on:
`wiki_links` consumes `[[…]]` up to the first `]]`, so the label handed to
inline parsing holds openers with nothing to close them and nothing nests.
Both outcomes are acceptable; the property under test is that the parse
settles instead of aborting.

Snapshot output and the CommonMark and GFM conformance baselines are
unchanged. The cubic runtime of nested links within the cap is a separate
concern and is not addressed here.

## Delimiter-run pairing counts too (issue #371)

### Decision

The nesting depth that delimiter-run pairing produces — emphasis, strong
emphasis, and the optional delimiter constructs: GFM strikethrough, `==`
highlight, `~` subscript, `^` superscript and CJK emphasis — counts against
`ParserOptions::max_nesting_depth` as well, and input nested deeper fails
with `ParseErrorKind::NestingTooDeep` rather than building the tree. The
counting matches the bracket bound: the outermost level is depth zero, so
`*`×200 `a` `*`×200 nests 100 levels and parses, and `*`×202 `a` `*`×202
does not.

Emphasis depth is *not* a third separate count. Brackets and delimiter runs
add up along one path — an emphasis node inside link text sits below the
link node — so the two share the inline budget: at 50 bracket levels, 50
levels of emphasis remain. Blocks keep their own count, as before, because a
block container still cannot occur inside inline content. Given the
"untrusted by default" contract, the useful property is that the whole
inline tree is bounded by the cap however the constructs are mixed; a
per-sequence bound would have left 100 bracket levels × 100 emphasis levels
= a 10,000-level tree, which is what the measurement below found.

The alternative was to stop pairing at the cap and leave the remaining
delimiters as literal text. It was rejected: it silently changes the output
of a document that is only nearly too deep, it has no error to report, and
it is harder to reason about than "this document is refused", which is what
#349 and the block cap already do.

### Depth means the depth of the tree

A delimiter *run* is not a nesting level. `*`×200000 with nothing to close
it builds no node at all, and `*a* *b* *c*` is one level however often it
repeats, because a closer pairs with the *nearest* opener and the pairs sit
next to one another. Depth grows only where pairs enclose one another, which
is what `*`×2n `a` `*`×2n does: the same two runs pair n times, each pairing
wrapping the node the previous one built.

So the measurement follows the tree, not the input. Each opener remembers
the depth of the node it currently holds; a pairing takes the deepest of the
records it encloses — a range it already walks to retire them — adds one,
and stores that on the opener. Two more counts complete the path: the inline
contexts open above the sequence, and the deepest subtree finished inside it
(a link or JSX element that pairing is about to wrap). Neither adds a walk,
so pairing stays linear: 200 KB of unpaired `*` parses in 3.7 ms and 100,000
adjacent `*a*` pairs in 46 ms, as before.

The count inside is conservative in one place. It is per sequence, not per
node, and a speculative sub-parse that is thrown away — a `[…]` that turns
out not to be a link — still counts. A paragraph that mixes a 90-level
construct with emphasis elsewhere can therefore be refused a few levels
early. That only reaches documents already at the cap, and erring toward
refusal is the safe direction for a bound whose failure mode is an abort.

### Why change

The parser builds emphasis iteratively, so it survived what it produced; the
HTML renderer (`render_node`) and the public `ast::visit` walkers recurse
over the result, and they did not. Measured on the release build of
`examples/render_stdin` (x86_64 Linux) before the fix:

| Input | Size | 8 MB stack | 1 MB stack |
| --- | ---: | --- | --- |
| `*`×1000 `a` `*`×1000 | 2 KB | renders | renders |
| `*`×2000 `a` `*`×2000 | 4 KB | renders | stack overflow, abort |
| `*`×20000 `a` `*`×20000 | 40 KB | stack overflow, abort | stack overflow |
| `_`×20000 `a` `_`×20000 | 40 KB | stack overflow | stack overflow |
| `**`×20000 `a` `**`×20000 | 80 KB | stack overflow | stack overflow |
| `<A>`+`*`×200 ×100 levels | 40 KB | — | stack overflow (10,102 levels) |

The last row is the mixture that made the shared budget the design: it is
inside the bracket cap and inside any per-sequence emphasis cap, and it
still builds a tree two orders of magnitude deeper than either. Through the
Node addon all of these are a segfault with no JavaScript error, as in #349.
`~~`×n `a` `~~`×n stays flat and always was fine.

### Validation

`tests/nesting_depth.rs` runs `*`, `_`, `**`, `==`, `^`, `~`, `~~` and a CJK
variant at n = 20000 in the default, GFM and all-options profiles, on the
normal test thread and on a `stack_size(1 << 20)` thread, and renders every
document it parses — the renderer walk is the one that used to abort. It
pins the 100/101 boundary for emphasis and for the `Highlight` and `Delete`
nodes that only nest through separate runs, the shared budget against
brackets and JSX phrasing, a custom cap, `0` still meaning unlimited, and
that a long flat run and 20,000 adjacent pairs still parse.
`tests/input_panics.rs` covers the hostile inputs end to end, and the Node
test suite covers the addon call that segfaulted.

Snapshot output and the CommonMark 0.31.2 and GFM conformance baselines are
unchanged: no fixture nests emphasis anywhere near 100 levels.
