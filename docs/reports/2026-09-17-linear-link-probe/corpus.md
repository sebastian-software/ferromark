# Differential corpus

Output equality was established before anything was timed, on both sides of
the change, by rendering the same corpus with two binaries built from the two
revisions and diffing their output line by line.

## What is compared

For every source, both builds print the FNV-1a hash of the rendered HTML
*and* of the document's AST `Debug` form, so a change that renders the same
but shifts a node boundary or a span still shows up. A parse error is
compared as its message. Each source is rendered under three option sets:

- `ParserOptions::default()`
- `ParserOptions::gfm()`
- GFM plus `wiki_links`, `inline_footnotes`, `superscript`, `subscript`,
  `highlight`, `math`, `mdx` and `definition_lists`

in seven combinations: those three with `max_nesting_depth = 0`, so deep
shapes are parsed rather than refused; GFM at the default cap of 100; and
three more at an artificially tight cap of 4 (all extensions, GFM, and all
extensions without `wiki_links`), where any difference in how a bracket level
is counted turns into a different verdict on a short document.

## What is in it

Sources are built by concatenating 1..N tokens drawn from a fixed
bracket-heavy alphabet (`[`, `]`, `![`, `](u)`, `[[`, `]]`, `[r]`, `][r]`,
`^[`, the delimiter runs, a backtick, a backslash, `<`, `<b>`, `&`, a
newline, parentheses, quotes and a few letters), so the generated documents
are dense in exactly the constructs the in-place bracket walk has to give up
on. Added to each run: the hand-picked shapes around the new path (nested
links, an inner destination that closes after the outer bracket, emphasis
pairing across a bracket, a code span covering the inner link, images in
link text, reference and shortcut forms, each also with a prefix and a
suffix), and the depth series 1, 2, 3, 4, 8, 17 of five nesting shapes.

## Runs

| Seed | Sources | Tokens per source | Renders | Differing |
| --- | ---: | ---: | ---: | ---: |
| 777777777 | 60,129 | 1–48 | 420,903 | 1 |
| 2468013579 | 60,129 | 1–24 | 420,903 | 0 |
| 31415926 | 60,129 | 1–32 | 420,903 | 0 |
| 8675309 | 60,129 | 1–16 | 420,903 | 0 |
| 112358 | 60,129 | 1–40 | 420,903 | 1 |

Every line of every run was compared with `diff`, not by checksum. Both
differing sources are the `wiki_links` case described in the decision record
and both are under the cap of 4: every other option set, the default cap and
the lifted cap included, is identical in all five runs. An earlier round of
the same corpus, before the walk read and wrote the probe cache and before an
abandoned attempt restored the depth counter, differed in five sources of one
run instead of one.

## Harness

[`harness/differential.rs`](harness/differential.rs) is the corpus and the
comparison; [`harness/timing.rs`](harness/timing.rs) is the shape generator
and the timing loop used for the tables in the report, including the
fixed-stack mode. Both are the `main.rs` and a module of a small binary crate
that depends on this repository by path; building it once per revision and
diffing the output of `differential` reproduces the equality result.
