# Test quality and performance evidence

The owner prioritizes a coherent v2 and accepts incremental quality work. Aim
for high meaningful coverage, with 99% as an aspiration rather than a number to
manufacture by excluding working features.

- A supported specification bug found by an oracle needs an ordinary blocking
  regression. Finite specification examples are not exhaustive coverage of
  every combination described by the standard.
- A diagnostic mismatch is not resolved by adding it to an expected-failure list.
  Record the behavior, its support decision, and the remaining work explicitly.
  Historical diagnostic artifacts remain unchanged as evidence.
- Prefer public behavior tests for missing coverage. Exclusions need a narrow
  local justification and a record of the excluded scope. No module-wide ignores
  for MDX, visitors, mapping, or other supported functionality merely to raise a
  percentage. Preserve the current CI floor until cross-platform data justifies
  raising it.
- Report measurement scope with coverage claims. Architecture-specific code,
  separate Node tests, and line versus branch coverage are distinct concerns.
- Before attributing a microbenchmark regression to the product as a whole,
  measure the full frozen workload and keep per-document results visible.
  Preserve exact output/AST gates for equivalent-output comparisons. Retain
  targeted changed-output probes separately for correctness fixes.

The [full-corpus measurement](../reports/2026-09-15-suite-regression/README.md)
and [first coverage pass](../reports/2026-09-15-coverage/README.md) apply these
rules. Next performance work should remove unnecessary structural reference
scanning on fenced TypeScript and reduce reference-heavy prepass costs without
undoing correct document-wide definitions. Further coverage work should start
with source mapping, table source transformations, definition lists, hooks, and
TOC traversal. The recorded MDX footnote collection gap needs its own behavior
regression and scope decision.
