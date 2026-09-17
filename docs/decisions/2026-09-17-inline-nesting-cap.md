# Inline bracket nesting counts toward `max_nesting_depth`

## Decision

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

## Why change

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

## Validation

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
