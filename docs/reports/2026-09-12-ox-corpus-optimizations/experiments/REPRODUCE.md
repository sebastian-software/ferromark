# Reproduce the corpus optimization experiments

Use Python 3.11+, Rust 1.97.1, and a checkout containing the baseline commit
`a1c308cba63f7bb038699df8560daa8959159b4e`. Run outside the repository to avoid
its CPU-specific Cargo configuration. The Rust target defaults to `apple-m1`; no target-cpu flag is supplied.
The archived driver uses a minimal
manifest with the exact Ferromark direct dependency versions; the package
version in that standalone manifest is only a harness label.

```sh
python3 docs/reports/2026-09-12-ox-corpus-optimizations/experiments/replay.py \
  "$PWD" /tmp/ferromark-corpus-replay
cd /tmp/ferromark-corpus-replay
cargo fetch --locked --manifest-path driver/Cargo.toml
python3 experiment.py baseline lines lazy-sort ordered-code packed-sort \
  code-ranges html-ranges all-ranges lines+all-ranges
python3 experiment.py lines+ranges-after-cells --runs 3 --rounds 9 --ms 75
python3 experiment.py production --runs 3 --rounds 9 --ms 75
```

The baseline command creates reference hashes before any candidate is checked.
Every candidate must match all 2,695 guards before timing begins. Each selected
input, syntax flag, and reuse/MDX mode is in the frozen case files. The transforms
and each resulting patch are archived separately. `implementation.patch` is the
formatted production change; production source identities and executables are
recorded in `metadata.json`. The production run uses the frozen, formatted source in `production-source.json.gz`;
the preceding combination run records the prototype before formatting and comments.

Do not run timers alongside builds, tests, profilers, other timers, or intensive
applications. Options and inputs are prepared before timing. The timer surrounds
fresh rendering and output destruction, except for explicitly named renderer
reuse controls. Screens are exploratory and do not feed the public tables.

For the final corpus comparison, follow the [earlier corpus reproduction](../../2026-09-12-ox-corpus-profiling/REPRODUCE.md)
to reconstruct all 649 inputs and the pinned Ox source. Replace its Ferromark
source with the production revision recorded here and build its release driver.
The archived `corpus/ferro` sources and lockfile identify the final driver.
Use the archived `corpus/run.py` for `verify`, `final-1`, `final-2`, and `final-3`.
The compact mismatch archive keeps original-file outputs; concatenations are
reconstructed from the frozen input and driver. Its only path assumption is that `cases.json`, `selected.json`, `case-manifest.json`,
and `bin/{ferro,ox}-release` are adjacent to it. The growing-arena result is primary;
presized-arena windows are retained as a separate diagnostic.

For allocation counts, use the [existing memory reproduction](../../2026-09-12-ox-memory/REPRODUCE.md)
with the final production source and the frozen `memory/cases.json.gz`.
`memory/ferro` contains the exact counter driver and lock; the Ox executable is
unchanged from that report. Adjust the local executable paths in `memory/run.py`
to the reconstructed workers. The previous and new summaries share identical
inputs, options, and normalization. The counter runs outside throughput timing.

`make-results.py` generates RESULTS.md from the independent screens and repeated
confirmation JSON files. In this compact archive, decompress the corpus and
memory `*.json.gz` summaries beside their gzip originals before regeneration.
All other experiment summaries are already plain JSON. The original files,
outputs, and licensing boundaries remain those of the linked investigations.

## Public table refresh

The two publishers read independent fresh archives in the parent report folder.
Use the [Bun harness](../../../../benchmarks/bun-comparison/README.md) for the
nine displayed inputs, each passed with `--case`, and the
[native pipeline harness](../../../../benchmarks/native-pipeline-comparison/README.md)
for all six additional engines. Each adapter README names its source pin and
three selected eligible documents. Use new build/result directories.

The archive retains complete build and measurement commands in `validation/`.
Those contain machine-local paths for provenance; replace them with local paths
when replaying. Preserve the compiler, dependency locks, options, allocator,
three process runs, warmup, and window counts. Never replace a single parser's
samples inside a published comparison.

```sh
python3 benchmarks/native-pipeline-comparison/publish.py
python3 benchmarks/bun-comparison/publish.py
mise run readme:write
node homepage/scripts/check-benchmark-numbers.mjs
```
