# Full-corpus before/after regression measurement

This reuses the runtime-profile worker to compare two frozen Ferromark revisions
on every document in the native comparison corpus. It does not rebuild or rank
other engines. Prepare both revisions with `benchmarks/runtime-profiles/prepare.py`,
then run:

```sh
python3 benchmarks/suite-regression/run.py BEFORE AFTER CORPUS.json OUTPUT
```

The corpus may also be gzip-compressed. The
[recorded run](../../docs/reports/2026-09-15-suite-regression/README.md) retains its
57 inputs, provenance, verification, raw samples, build identities, and summary.
Third-party source licenses and attribution remain in
[the broad corpus licenses](../broad-comparison/licenses/ATTRIBUTION.md) and
[Wikipedia attribution](../broad-comparison/WIKIPEDIA-ATTRIBUTION.md).

CommonMark and GFM-shared use the same syntax flags as the native comparison:
GFM adds tables, task lists, and strikethrough. Both use the strict CommonMark
renderer options. All documents must preserve exact HTML and debug AST before
and after timing, across fresh/reused lifecycles and every process restart.
Checksums validate each timed iteration. Any output change stops the run.

Defaults: three rounds with new processes, shuffled job order, five alternating
pairs, 20 ms warmup, and 40 ms windows. Each document is timed independently, then
all documents of each profile are rotated through a worker. The complete-suite
ratio sums the two profile batch times (one visit to each of the 57 documents).
It is not an unweighted average of document percentages. Raw timing windows,
per-round medians, and their ranges remain available; these are descriptive
workstation measurements, not confidence intervals or cross-machine guarantees.
