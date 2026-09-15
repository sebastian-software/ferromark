# Refactoring checks

`fixtures.py` emits nine authored inputs covering nested source spans, indented
JSX and attributes, escaped table pipes, BOM/NUL normalization, root and container
references, footnote-only labels, fenced TypeScript decoys, and link rendering. Optional extensions are
explicitly configured; none of these synthetic inputs represents the broad corpus.

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
