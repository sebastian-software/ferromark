# ARCH-EXP-017: Markdown feature costs and scratch allocation

**Date:** 2026-09-10

**Hypothesis:** fixed parser setup dominates small fresh render calls, while
several optional features allocate temporary storage repeatedly during a render.

**Method:** separate option activation on plain input from actual syntax use;
measure both fresh owned rendering and a retained Renderer. Use a controlled
catalog covering every Markdown option, CommonMark constructs, and lightweight
inputs. Collect allocation counts in a separate instrumented executable. Check
exact HTML before alternating old/new timing windows. Use complex GFM/CommonMark
workloads as regression controls. CPU profiles of tiny and lightweight documents
support prioritizing allocation work before changing scanning algorithms.

**Decision:** allocate optional inline scratch and the heading-ID registry when
first needed. Retain scratch across renders, share opener storage across
sequential extension resolvers, and clear it at every resolution boundary.
Retain resolved math spans rather than replacing their vector per paragraph.
Borrow stored inline-note content during emission. Write footnote numbers directly to the HTML buffer instead of creating temporary
Strings. Keep the document-wide heading-ID registry shared with nested footnotes.
These changes preserve the existing parser phases and safe-Rust implementation.

The preallocated event buffers from ARCH-EXP-006 remain in place: this experiment
changes optional scratch storage, not every allocation indiscriminately.

**Results and limits:** the [feature study](../reports/2026-09-10-markdown-feature-costs.md)
preserves the baseline, and the [optimization log](../reports/2026-09-10-markdown-feature-optimizations.md)
preserves each experiment and its raw measurements. Performance measurements are
local to Apple M1 Pro. Enabling a feature can remove work, and normalized syntax
costs include differing block structure and HTML output; they are not additive
or universal prices for individual features.

Reference resolution, entity decoding, and many small list blocks remain useful
profiling targets. Their larger total cost alone does not justify changing their
algorithms: investigate phase/allocation profiles and representative documents
before another optimization. This study introduces no SIMD backend or dependency.

**Known trade-off:** a warmed Renderer processing the dense inline-code control
is 4.8% slower in a longer paired recheck (about 0.26 µs per 32-paragraph document).
Fresh owned rendering of the same input improves 9.0%, and larger mixed controls
do not show the same consistent regression. Retain the changes for the broader
gains, record this limitation, and avoid claiming universal speedups. The cause
of this microbenchmark regression has not been isolated.
