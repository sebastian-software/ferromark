# Keep comment handling out of ordinary paragraphs

The paragraph parser now uses a direct borrowed-source path unless an eligible
comment actually falls inside the paragraph's content. Filtering and original
source-position remapping live in a separate helper. A runtime dispatch selects
one of two const-generic paragraph loops, so the comments-disabled loop contains
no paragraph comment discovery or comment-map handling.

This replaces the intentionally invalid bypass from the
[previous diagnosis](../2026-09-14-ox-regression/README.md) with an implementation
that preserves enabled comments, source spans, and CommonMark normalization.
The public parser options and defaults are unchanged.

## Measured changes

On the 14 documents used for the native OX comparison, the selected B candidate
reduces elapsed time relative to the previous v2 core by approximately:

| Work | Time reduction |
| --- | ---: |
| Complete fresh processing | 4.8% |
| Complete reused processing | 5.8% |
| Full parsing | 8.3% |

In that same run, the complete pipeline trails OX by about **4.0% fresh / 4.9%
reused**, down from roughly 9% / 11–12% before this change. This applies to the
14 equal-output inputs, not to every Markdown feature or workload.

The 75-case runtime-profile suite supplies a separate check with another
compiler/worker setup. On its three plain-prose sizes (about 300 B, 4 KiB, and
64 KiB), reused processing takes about **9.4% less time with comments off**, and
**9.2% less with comments enabled but absent**. These percentages are within
that harness; they are not pooled with the native-engine comparison.

There is a small tradeoff: its synthetic inputs with many active comments take
about **2.3% more time fresh, 2.0% more reused, and 3.3% more in parsing**. The
outlined filtering path adds a call boundary on those inputs. The implementation
is therefore not advertised as faster on every case. All raw windows and
individual documents remain available for follow-up work.

Across all 57 original native-comparison inputs, the accepted source reduces
time by **2.7% fresh, 2.9% reused, and 3.7% in parsing**. [TABLES.md](TABLES.md)
also retains the earlier three-round A/B confirmation and prototype screens.

The final source includes the Clippy-requested `&self` helper signature and was
rebuilt and remeasured after that adjustment. The headline numbers above refer
to that accepted source, not to the earlier mutable-reference prototype.

## Why the implementation is safe

The existing parser already observes each eligible physical comment while
collecting paragraph lines. The new path reuses that fact. A comment at or after
`content_end` lies outside the text slice; one strictly before it requires
filtering and remapping. This is the same boundary test the previous helper
performed after every paragraph called it.

The helper preserves both ordinary paragraphs and setext headings, including
explicit IDs/classes and inline offsets. The original entry point remains for
the math fallback. Nested parsers select their specialization using their own
mapped physical-comment eligibility, so `> // text` and `- // text` remain literal.
References, footnotes, definition-list bodies, and table source handling retain
their existing specialized paths.

The disabled path still has one runtime choice at entry. Parser structure sizes
and other optional-feature checks are not removed by this change; “disabled” is
not claimed to make the whole library identical to a build without extensions.
The measured paragraph filtering cost is removed without skipping correctness
work. NUL replacement, BOM handling, and source remapping remain intact.

The recorded arm64 disassembly reduces the paragraph function's stack reservation
from **912 to 240 bytes**. LLVM inlines the two specializations into the entry
wrapper and branches between their loops using the flag. The complete wrapper
contains 547 instructions versus 483 before the change: specialization increases
static code size while reducing work on the disabled path. Stack reservation
and static instruction counts are not dynamic work or cache-miss measurements.

## Selection and evidence

A isolates actual comment filtering; B adds flag specialization. B is selected
for its disabled loop and incremental parsing gain. In the broad three-round
prototype comparison, B improves parsing about 0.5% over A; their complete-pipeline
times are effectively tied. The larger benefit comes from A's separation of
filtering and mapping. [ATTEMPTS.md](ATTEMPTS.md) records
both candidates and the initial B build failure, including its correction.

- Exact HTML and full AST equality precede timing on all measured inputs.
- Each candidate passes 1,200 generated differential comparisons.
- Two new regression tests cover flag parity and setext comment boundaries,
  including attributes, containers, frontmatter, BOM/NUL, and line endings.
- All **773 workspace tests**, formatting, Clippy with warnings denied, and
  benchmark compilation pass for the accepted source.
- The [method and replay instructions](METHOD.md) explain the two build setups,
  timing boundaries, source provenance, and limitations.
- `builds/`, `runs/`, `patches/`, and `validation/` retain the evidence;
  `SHA256SUMS` covers the report artifacts.

The main README's six-engine table remains explicitly pinned to its previous
core revision. This report records the newer optimization with controlled
before/after measurements rather than rescaling another run's engine scores.
