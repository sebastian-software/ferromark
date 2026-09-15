# Container reference correction measurements

Freeze before/after cores with `benchmarks/runtime-profiles/prepare.py`, then:

```sh
python3 benchmarks/runtime-profiles/prepare.py /tmp/container-before --revision 0b529ac
python3 benchmarks/runtime-profiles/prepare.py /tmp/container-after --working-tree
python3 benchmarks/container-references/run.py /tmp/container-before /tmp/container-after /tmp/container-results
```

Output directories must not exist. The run compares identical explicit
CommonMark profiles over ordinary prose/lists, root and nested definitions,
code decoys, a sparse container definition, and project documents. Three
lifecycles use seven alternating pairs with 50 ms timing windows and 10 ms warmup.
See [the correction report](../../docs/reports/2026-09-15-container-references/README.md)
for interpretation and limitations. Corrected rows change output; only controls
assert exact HTML and AST equivalence. Checksums and post-timing verification
apply to every row. Generated tables retain every row rather than hiding costly
cases in an aggregate. Raw samples, outputs, build hashes, and host observations
are archived with each run. Do not run builds or other tests during timing.
