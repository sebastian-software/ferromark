# Paragraph optimization attempts

Baseline: production core `33c216b`, also unchanged at starting commit `ec06a8c`.
Every candidate preserves CommonMark source normalization. Native comparisons
use the frozen OX diagnostic worker and dependency setup. Feature comparisons
use the existing Rust 1.95 runtime-configured harness; compare within each
harness, not absolute timings across them.

## A — Outline filtering only when a comment is in paragraph content

Replace the ordinary paragraph/setext path with a direct borrowed source slice.
Enter a separate `#[inline(never)]` helper only when an observed eligible
comment is strictly before `content_end`. The helper retains filtering, heading
attributes, inline offsets, and source-map remapping. A trailing comment outside
the content range needs no filtering, matching the existing semantics.

The initial 14-case native screen passes exact HTML and full AST equality.
Complete processing takes 4.7% less time fresh and 5.0% less with reuse; parsing
uses 7.5% less time. The generated differential corpus passes all 1,200
comparisons, including enabled comments and definitions. Broader feature timings
are retained separately.

Two focused regression tests were added. One initially expected explicit heading
IDs under `HtmlRendererOptions::gfm()`, which disables their emission. Its AST ID
was already correct. The fixture now enables `heading_ids` explicitly; this was
a test-configuration correction, not a parser failure or production change.

## B — Specialize the paragraph loop for the runtime comment flag

Build on A. Dispatch once when entering a paragraph to a const-generic loop for
comments enabled or disabled. The disabled instantiation can eliminate comment
position tracking and continuation-line comment probes. The enabled path keeps
A's observed-comment test, so enabled-but-absent syntax still avoids remapping.
Measured outcome and selection follow below after verification and timing.


B's first build caught an additional caller in the math fallback. The old
paragraph comment claiming a sole caller was stale. Retain the existing
`pub(super)` entry point as an inline runtime dispatcher and keep specialization
private behind it. Both failed build logs and the initial patch are retained;
that revision was never timed. The corrected variant uses fresh build paths.


Final Clippy review identified that the outlined helper needs only `&self`, not
`&mut self`. The signature was narrowed. Because reference mutability can affect
code generation, the final source form receives fresh native and runtime builds
and confirmation; earlier B screens remain labeled as the pre-Clippy prototype.


## Accepted result

A's observed-comment helper is retained as the foundation. B's specialization
is retained for eliminating comment discovery from the disabled loop. In the
three-round 57-document prototype comparison, the pipeline differences between
A and B are negligible; B's parser-only gain is about 0.5% and consistent across
rounds. The separately rebuilt final immutable-helper source is the accepted
production implementation.

The final source reduces native time by 2.7% fresh / 2.9% reused / 3.7% parse on
all 57 documents, and by 4.8% / 5.8% / 8.3% on the 14 OX-agreeing documents.
The runtime-profile suite records about 9.4% less reused time for plain prose
with comments off, and 9.2% less with comments on but absent. Active-comment
synthetic inputs instead cost 2.3% more fresh / 2.0% reused / 3.3% parse. This
tradeoff is retained explicitly; there is no claim of a universal improvement.

The final same-binary toggle adds about 0.3% fresh / 0.4% reused / 0.9% parse on
the three plain-prose probes. The baseline toggle was also small: it could not
expose the historical cost of installing the general paragraph-map path. The
larger optimization gain is measured between implementations with identical flags.

Assembly inspection initially looked for two standalone generic symbols; LLVM
had inlined the instances into the existing entry wrapper. The retained wrapper
shows the flag branch between the two loops, a 240-byte stack reservation
(previously 912), and 547 static instructions (previously 483). This is a code
size tradeoff, not proof of fewer executed instructions on every input.

Final validation: 773 workspace tests, 25 focused comment tests within that
suite, all required formatting/Clippy/benchmark-build checks, and 1,200 generated
differential comparisons pass. All 21,183 retained timing windows have verified
checksums. The accepted build snapshots match the working production files.
