# Refactoring checks

`fixtures.py` emits ten authored inputs covering nested source spans, indented
JSX and attributes, escaped table pipes, BOM/NUL normalization, root and container
references, footnote-only labels, fenced TypeScript decoys, and default/configured
link rendering. Optional extensions are explicitly configured; none of these synthetic inputs represents the broad corpus.

```sh
python3 benchmarks/refactoring/fixtures.py > /tmp/refactoring.json
python3 benchmarks/runtime-profiles/prepare.py /tmp/before --revision BASE
python3 benchmarks/runtime-profiles/prepare.py /tmp/after --working-tree
python3 benchmarks/suite-regression/run.py /tmp/before /tmp/after /tmp/refactoring.json /tmp/targeted
python3 benchmarks/suite-regression/run.py /tmp/before /tmp/after docs/reports/2026-09-15-suite-regression/corpus.json.gz /tmp/broad
```

The runner requires exact HTML, debug AST (including source spans), and root-child
counts across revisions and fresh/reused execution. It also validates timed
checksums and rechecks outputs after timing. Keep raw outputs and timing windows
with each retained refactor. Run builds and tests before, not during, timing.

The existing compact table-cell map has a deliberately restricted node domain;
consolidating full-tree span traversal does not require replacing this map or
changing its node-domain policy. Renderer hook measurements must use the same
hook-enabled worker on both sides, separately from the default-path comparison.

## Hook path

Generate a separate worker that calls the no-op hook entry points. Its replacement
counts are asserted, and the runner requires the same worker hash in both builds.
It does not add a branch to the ordinary worker or compare hooks against no hooks.

```sh
python3 benchmarks/refactoring/hook_worker.py > /tmp/hook-worker.rs
python3 benchmarks/runtime-profiles/prepare.py /tmp/hook-before --revision BASE --worker-source /tmp/hook-worker.rs
python3 benchmarks/runtime-profiles/prepare.py /tmp/hook-after --working-tree --worker-source /tmp/hook-worker.rs
python3 benchmarks/suite-regression/run.py /tmp/hook-before /tmp/hook-after /tmp/refactoring.json /tmp/hooks-targeted
```

Use `--batches-only` for aggregate-only broad timings; exact-output checks still
cover each input, but that mode cannot establish individual document regressions.
The configured-link profile exercises sanitization, titles, Markdown route
conversion, base URLs, and external-link attributes. Compare hook/default output
records as an additional policy-equivalence check. Existing hook tests also cover
replacing/skipping/wrapping nodes and highlighter fallback; no-op benchmarks do not
measure user callback costs.
