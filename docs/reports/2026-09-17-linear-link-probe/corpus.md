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

each with `max_nesting_depth = 0`, so deep shapes are parsed rather than
refused.

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

| Seed | Sources | Tokens per source | Renders | Checksum |
| --- | ---: | ---: | ---: | --- |
| `0x9e3779b97f4a7c15` | 60,129 | 1–14 | 180,387 | `3405cd6a654954cb` |
| 11111111 | 200,129 | 1–8 | 600,387 | `4c640df9f294ad25` |
| 2468013579 | 200,129 | 1–24 | 600,387 | `57f9c2ff9c06f9a8` |
| 777777777 | 60,129 | 1–48 | 180,387 | `cf7151318335b39f` |

The checksum is the sum of the per-case hashes; it is equal on both sides of
every run, and so is every individual line — the runs were compared with
`diff`, not by checksum alone.

## Harness

[`harness/differential.rs`](harness/differential.rs) is the corpus and the
comparison; [`harness/timing.rs`](harness/timing.rs) is the shape generator
and the timing loop used for the tables in the report, including the
fixed-stack mode. Both are the `main.rs` and a module of a small binary crate
that depends on this repository by path; building it once per revision and
diffing the output of `differential` reproduces the equality result.
