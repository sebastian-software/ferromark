# Ideas declined before measurement

## Delimiter run length and flanking from the NEON marker mask

Declined without a patch, on the code and on the corpus:

- The inline scanner (`src/parser/inline/scan.rs`) returns an index, not a
  mask. The mask exists only inside the `classify` closures of the NEON and
  x86 paths and the loop returns the moment it is nonzero; the scalar path
  ORs eight flag-table entries into one byte and has no per-byte bits at all,
  so the reference implementation could not produce what the vector path
  would have to match.
- The mask flags "any inline marker", not "this marker": `**[`, `` *` ``,
  `_<`, `~~*` all yield contiguous bits from different bytes, so a run length
  needs its own broadcast compare, a second pass. The nibble tables carry no
  whitespace or punctuation class either; the comment in `scan.rs` explains
  that the four opt-in markers already used the last free nibble
  intersections.
- Over all 207 corpus cases every `*`, `_` and `~` run has length 1 (7,466)
  or 2 (2,236), with one run of 6; `marker_run_len` exits after two or three
  byte compares. `vite-docs-api-plugin`, the densest real document, holds 100
  runs in 31,890 bytes, so a fused run-and-flanking step could save perhaps
  400–600 cycles of a ~10⁵-cycle parse: under 0.5% on the best case, below
  the A/A control's spread, and only the authored `table-formatted-*`
  diagnostics would register.
- The cost is a third implementation of CommonMark 6.2 flanking, in
  architecture-specific `unsafe`, next to the shared `Neighbors::flanking`
  that the exhaustive differential test in `emphasis.rs` keeps identical for
  the ASCII and Unicode classifications.
