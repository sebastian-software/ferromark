# Why current v2 trails original OX

The largest measured regression on the published 14-document OX comparison
comes from **line-comment plumbing in the paragraph parser, even with line
comments disabled**. CommonMark source normalization is a second, smaller
contributor. Rendering is close overall and essentially equal on large prose.

The starting benchmark work was already committed as `d173c24`, with a clean
tree. This investigation preserves production core `33c216b`: it adds diagnostics
and evidence, not a correctness-breaking rollback.

## Where the regression entered

The unchanged native executable reproduces the published gap: v2 takes about
**9.3% longer fresh and 12.4% longer reused**. The added stage harness independently
reproduces approximately 9.2%/12.0%. Its parse stage is 15.4% slower than OX;
rendering an existing AST is only 2.4% slower.

Historical cores were rebuilt with the same compiler, native libraries,
allocator, and dependency lock. All 14 inputs produce exactly the same HTML.
The fine history run gives these time differences relative to OX:

| Checkpoint | Fresh | Reuse |
| --- | ---: | ---: |
| Compatibility fixes complete, `db987c8` | +1.4% | +2.4% |
| Typography removed, `7010384` | +1.7% | +2.5% |
| Table spans/metadata, `e01c611` | +1.9% | +2.7% |
| Derived table columns, `46c761d` | +1.9% | +3.1% |
| Line comments introduced, `40047f7` | **+8.7%** | **+11.3%** |
| Frontmatter introduced, `e0eba1d` | +9.5% | +12.2% |
| Current core, `33c216b` | +9.3% | +11.4% |

The major step is the line-comment commit. Table work affects actual tables
more than this aggregate suggests; for example, the 310-byte table comment's
render stage rises from about 187 to 207 ns at the table-metadata commit.
Frontmatter adds a smaller net change. Differences around 1% should be treated
cautiously. The earlier coarse run also shows that pre-compatibility v2
(`adf891a`) was slightly faster than OX on this subset.

## Why an off flag still has a cost

Line comments changed paragraph processing from borrowing the source slice
directly to calling `without_line_comments_with_first`, receiving an optional
source map, selecting an inline offset, and potentially remapping child spans.
With the option off, the helper returns the original slice and no map. It still
appears as an actual call in the compiled paragraph function; the general
result-handling path remains part of that function.

An isolated diagnostic build restores direct source access only at the two
paragraph/setext call sites. Normalization, other comment checks, parser state,
and renderer code remain. Exact HTML and full AST Debug—including source spans—
agree with current v2 on all 14 inputs before timing.

| Direct-paragraph diagnostic vs current v2 | Time reduction |
| --- | ---: |
| Complete fresh processing | **5.0%** |
| Complete reused processing | **6.3%** |
| Full parsing | **8.6%** |
| Rendering an existing AST | 0.1%, effectively unchanged |

The effect is consistent in both rounds. On the 37-byte acknowledgment, parsing
drops from 50.61 to 43.77 ns. On 80,966-byte chess prose, it drops from 21.09 to
19.36 µs. The complete pipeline still trails OX by 3.8% fresh / 4.4% reused in
this diagnostic build.

The [disassembly](assembly/prologues.txt) supplies additional evidence: the
current paragraph function reserves **912 bytes** of stack, versus **208 bytes**
in the direct-source diagnostic. The complete function contains 483 versus 386
arm64 instructions. These are static function properties, not executed-instruction
counts or a claim that every stack byte is touched. They demonstrate that adding
an optional general path changed the compiled function substantially. The
experiment measures that whole change, including code generation, rather than
assigning the gain solely to one branch.

The earlier feature-on/off benchmarks measured the incremental cost of enabling
comments **inside an implementation that already contained this machinery**.
They could not expose its full cost relative to the implementation before line
comments existed. That distinction explains why an apparently small flag cost
coexists with a larger historical regression.

## The normalization scan is real, but partial

`4a1e55f` introduced root source normalization for CommonMark NUL handling;
`db987c8` extended prefix/BOM handling. Even clean input receives a complete
`memchr(0, ...)` scan before borrowing the unchanged source. Original OX lacks
that pass.

A second diagnostic toggles normalization within the **same executable**. All
14 inputs contain no NUL or leading BOM, and exact HTML/AST equality passes.
Bypassing it cuts full-pipeline time by **1.6% fresh / 2.8% reused**, and parsing
by 3.3%. For chess prose, initialization falls from 4.05 to 3.04 µs. This confirms
a redundant scan on ordinary input, but most of the original gap remains.

The instrumented control differs slightly from the uninstrumented build, so
the reported effect compares only bypass versus its own same-binary control.
Separate NUL/BOM fixtures reject the bypass as expected. Required normalization
and original source positions must survive any production optimization.

## Other costs and limits

The parser grew from 192 to 256 bytes, options from 24 to 32, and `Document`
from 40 to 48. The `Node` enum remains 32 bytes. These changes can affect
construction and code layout, but the measurements do not isolate their exact
contribution. In particular, removing paragraph source-map handling recovers a
substantial amount while leaving those structure sizes unchanged.

Renderer preparation exists in both OX and v2. Its stage measurement does not
support it as the broad primary cause. Actual table rendering has its own
measurable regression and deserves a separate targeted round.

These findings apply to the **same 14 agreeing documents**, not all Markdown
syntax or all hardware. Stage times cannot be added. Neither can the two bypass
gains: no combined variant was measured. Exact methods, adaptations, negative
controls, and timing limits are in [METHOD.md](METHOD.md).

## Next optimization target

First separate the ordinary paragraph path from comment filtering and span
remapping. A cold helper or specialized path should preserve enabled-comment
behavior while keeping the common borrowed-source path compact. Validate both
disabled and enabled profiles, nested containers, setext headings, and full AST
source positions before measuring the complete pipeline.

Then investigate combining NUL discovery with an existing source scan, preserving
BOM/NUL behavior and remapping. Table metadata overhead is the third specific
target. The findings point to avoiding extra work and keeping common functions
compact before adding more SIMD.

**Neither diagnostic bypass is promoted:** one breaks comment filtering when
enabled; the other breaks CommonMark input normalization. Production performance
and behavior are unchanged in this round.

## Evidence and validation

- [All experiments and rejected bypasses](ATTEMPTS.md)
- [Generated tables, round medians, and structure sizes](TABLES.md)
- [Measurement method and replay instructions](METHOD.md)
- `runs/`: every timed window, verification output/AST, input attribution, and host observations
- `builds/`: 11 build records, exact compiled workers, build logs, and shared dependency lock
- `patches/`, `assembly/`, and `normalization-guards/`: isolated interventions and counterexamples

All workspace tests, workspace formatting, Clippy with warnings denied, and
benchmark compilation pass. Six harness tests verify that HTML drift, source-span
drift, missing AST observations, and inconsistent stages cannot pass the semantic
gate. Logs are retained in `validation/`. `SHA256SUMS` covers the report artifacts.
