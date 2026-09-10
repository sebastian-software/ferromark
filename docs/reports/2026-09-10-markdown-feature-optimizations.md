# Measured Markdown feature optimizations — 2026-09-10

See [the baseline feature study](2026-09-10-markdown-feature-costs.md) for the
catalog, machine, interpretation, and measurement method. This log preserves
individual hypotheses, paired measurements, and decisions. Production starts at
`99fbf01`; measurement infrastructure is committed in `bfb3687`. Negative timing
changes mean less elapsed time. These are local Apple M1 Pro results.

## 1. Allocate inline scratch buffers on demand — retained

Hypothesis: preallocating scratch space for every possible inline construct
accounts for much of the fresh-parser cost, even on documents with no markup.
CPU samples of the tiny CommonMark workload are dominated by allocator paths.
Replace initial capacities with empty buffers; existing reserve/resize/growth
paths allocate them when needed, and a retained Renderer keeps that capacity.

Seven paired >=100 ms windows, alternating binary order, with exact HTML checks
for all 292 feature configurations:

| Workload | Fresh owned time change | Retained Renderer change |
| --- | ---: | ---: |
| Tiny CommonMark | -63.7% | +1.0% |
| Tiny default | -60.8% | +0.3% |
| Plain 1 KiB CommonMark | -32.2% | -3.5% |
| Light 1 KiB CommonMark | -5.9% | +0.2% |
| Light 1 KiB default | -5.6% | +0.8% |
| README CommonMark | +1.8% | +1.3% |
| README default | -0.4% | +1.4% |

Tiny CommonMark drops from 40 allocation calls / 11,239 requested bytes to
9 / 3,103. Default drops from 43 / 14,703 to 12 / 6,567. The separate GFM guard
checks 90 HTML configurations: complex CommonMark/GFM cases stay within about
2.2%, and plain input with a fresh parser improves by 7–8%. A +4.6% retained
heading-ID result in the feature probe needs rechecking in the final combined
comparison; warmed buffers should not repeatedly pay initialization costs.

Validation: `cargo test --locked --all-features` passed. Evidence is in
`2026-09-10-markdown-feature-costs/optimizations/lazy-{comparison,counts,gfm-guard}.json`.
