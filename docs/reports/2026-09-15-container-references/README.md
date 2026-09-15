# Reference definitions in containers

## Correctness

CommonMark 0.31.2 [section 4.7](https://spec.commonmark.org/0.31.2/#link-reference-definitions)
requires reference definitions in lists and block quotes to apply to the entire
document, including links that precede their definitions. The first definition
of a label wins. Definitions cannot interrupt paragraphs or arise inside code
or raw HTML blocks.

Before this correction, the reference prepass rejected list prefixes, and a
one-line list optimization could bypass definition block parsing. Fence state
could also leak out of list code and hide a later root definition.

The parser now recognizes potential container prefixes, then uses its existing
block grammar in a temporary arena to collect actual definitions in document
order. This pass skips inline parsing. Only reference values survive in the
main arena; the temporary block tree is discarded. Inputs without definition
candidates retain the fast path. Reference links disabled means no reference
collection. Single-line definition-shaped list items enter block parsing.

Regression tests cover bullet and ordered lists, forward/backward links, images,
nested lists/quotes, blank-separated definitions, first-definition precedence,
code/HTML/paragraph decoys, tabs, physical line comments, line endings, and the
reference-disable switch. Existing specification fixtures remain unchanged.

The [line-comment differential result](line-comments.json) has 39/40 exact
matches and 40/40 semantic matches, with both comment-off baseline probes also
agreeing. The only remaining exact-output difference is whitespace inside an
empty list item. The shared conservative comparator retains raw output and
rejects substantive changes. Historical results remain untouched.

## Performance

[Generated timing tables](timing/tables.md) and raw paired results retain the
frozen build identities, options, inputs, output checks, and host observations.
The baseline is `0b529ac`; the candidate is the working-tree correction before
any subsequent convenience-API work. Both use the existing runtime-profile
worker, Rust 1.95 release builds, fat LTO, one codegen unit, generic CPU, and
unchanged locked registry dependencies. Apply [candidate.patch](candidate.patch)
to the baseline to reconstruct the measured core. [Harness hashes](harness-sha256.json)
identify the scripts.

The bounded corpus covers ordinary prose/lists, root definitions, nested
definitions, fenced decoys, a single container definition in a large document,
and four project documents. Seven alternating pairs use 10 ms warmup and 50 ms
windows per side across parse-only, retained parse/render, and fresh parse/render.
No builds or other task tests run during timing. Unaffected controls require
identical HTML and debug AST before timing; corrected cases must change HTML.
Timed checksums and post-timing output checks guard every row.

Changed-output rows measure the cost of correct parsing, not an equivalent-output
optimization. The temporary block pass adds work only on candidate-bearing inputs;
this tradeoff avoids maintaining a second, incomplete container grammar.
Ordinary prose, ordinary lists, and the sampled project documents were close to
unchanged. Root-reference-heavy controls showed a modest regression from the
expanded candidate search. Repeated nested definitions and sparse container
references cost substantially more because they now perform an additional block
pass and produce resolved references. Keep the spec-correct behavior, expose
these costs, and treat a faster collection strategy as a separate measured
optimization. Workstation ratios are descriptive and do not establish performance
on other CPUs; this run is not an independent repetition or a confidence interval.

## Validation

The local validation includes all-feature workspace tests, Clippy with warnings
denied, formatting, the existing 90% line-coverage gate, and Node package checks.
Completed checks: 812 workspace tests passed, including documentation tests;
Clippy, formatting, and all benchmark targets passed. Instrumented tests reached
90.68% line coverage without changing exclusions or the gate. Node build, 28 tests
plus panic-unwind verification, pack validation, and clean installation passed.
Repository contracts, workflow pins, README generation, historical benchmark
verification, and the compatibility comparator's Python tests passed.
