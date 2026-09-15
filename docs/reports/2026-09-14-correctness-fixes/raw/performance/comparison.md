# Paired short-run comparison

Both runs used the same frozen build metadata, native57 corpus, four cases,
`fresh reuse`, three rounds, three pairs per round, 50-ms windows, and seed
20260914. The runner performed exact HTML and AST verification before timing.

Raw results:

- `results-01/` — 2026-09-14 11:14:08Z–11:14:18Z
- `results-02/` — 2026-09-14 11:15:25Z–11:15:35Z

Geometric means of the eight per-case row medians, where a ratio is
`baseline_ns_per_document / candidate_ns_per_document`:

| mode | run 01 | run 02 | combined | candidate time implied by combined ratio |
| --- | ---: | ---: | ---: | ---: |
| fresh | 0.98125 | 0.98062 | 0.98094 | +1.94% |
| reuse | 0.96495 | 0.96255 | 0.96375 | +3.76% |

The central effect is consistent between runs. Per-case row-median ratios
were 0.979–0.985 for `fresh` in run 01 and 0.970–0.993 in run 02; `reuse`
was 0.946–0.990 in run 01 and 0.946–0.980 in run 02. Run 01 had one raw
`wiki-rainbow-article-body/reuse` pair at 0.615; the corresponding run-02
row had no comparable low pair. Run 02 had one raw `guard-angle-link/reuse`
pair above 1.33, but its row median remained 0.958.

These are short, host-loaded sanity checks for four documents and do not
support a claim about the full 57-document corpus.

