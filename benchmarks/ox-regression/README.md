# OX regression diagnostics

This harness localizes the v2/OX gap on the 14 documents whose native HTML
already agrees across all six engines. It is a diagnostic companion to the
[native comparison](../native-comparison/README.md), not a new leaderboard.
The [recorded investigation](../../docs/reports/2026-09-14-ox-regression/README.md)
contains the results and limits.

`worker.rs` retains the six-engine native worker's fresh/reuse paths and adds
three v2/OX stages: parser construction, full parsing, and rendering a prebuilt
AST. Keeping the existing paths provides a control for the added harness code.
These stages have different optimization boundaries and are not additive.

Build the frozen native reference first, using its documented setup. Then:

```sh
python3 benchmarks/ox-regression/prepare.py /private/tmp/ox-stages \
  --native-build /private/tmp/ferromark-v2-native-matched-build
python3 benchmarks/ox-regression/run.py /private/tmp/ox-stages /private/tmp/ox-stages-results
python3 -m unittest discover -s benchmarks/ox-regression -p 'test_*.py'
```

Each build directory must be new. `prepare.py --revision COMMIT` uses a Git
archive without touching the checkout, reuses the frozen native support
libraries, and checks that the shared dependency lock stays unchanged. It records
any API adaptation required by older commits. Exact HTML equality is still
required for those historical builds before timing.

For several builds, supply `run.py --variants variants.json`, for example:

```json
{
  "ox": {"build": "/private/tmp/ox-stages", "engine": "ox-content"},
  "control": {"build": "/private/tmp/ox-stages", "engine": "v2"},
  "candidate": {"build": "/private/tmp/ox-candidate", "engine": "v2"}
}
```

Use `--same-ast control candidate` for modifications to the same core revision.
The gate compares exact HTML and full AST Debug, including source spans, before
any timing. Historical AST structures can differ, so the history comparisons
require exact HTML across revisions and AST consistency within each revision.

`patches/` contains deliberately incomplete diagnostic bypasses. They are
applied only to extracted snapshots with `prepare.py --patch FILE`. They must
not be used as production optimizations. The normalization experiment also
uses `--worker benchmarks/ox-regression/normalization-worker.rs`: its environment
switch is read once at process startup, with an atomic branch in the normalizer.
Set `FERROMARK_DIAGNOSTIC_SKIP_NORMALIZATION=1` only in the candidate's `env`
object. Control and bypass then run in the same executable. The bypass fails
NUL/BOM correctness guards by design.

Run timings serially, after builds and tests finish. The runner alternates engine
order, verifies output before and after each job, checks every timing checksum,
and saves every window, per-round medians, corpus provenance, binary/source
hashes, and host observations. `--all-cases` disables the existing reference-HTML
gate for custom diagnostic corpora; use an explicit candidate equality gate when
doing so. It does not make those cases eligible for the native leaderboard.
