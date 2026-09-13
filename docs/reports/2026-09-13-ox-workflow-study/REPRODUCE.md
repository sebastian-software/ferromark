# Reproduce the Ox workflow investigation

Use Ferromark source `0026d4c66eeb4d8a580c15d96fd89976ca9f833a` and the pinned
Ox commit in the archived Cargo manifests. The original investigation used
Rust 1.97.1, Apple M1 Pro, macOS 26.6.2, the system allocator, and an explicit
`-C target-cpu=generic`. Release builds use opt-level 3, fat LTO, one codegen
unit, and panic abort. No production source transformation is applied.

All source and binary identities are in `build.json` and
`audit-provenance.json`; exact dependency locks and generated worker sources
are under `ferro/`, `ox/`, `cache-ferro/`, and `cache-ox/`. Build logs accompany
the commands. The absolute paths record the measured environment; the builder
accepts `FERROMARK_ROOT` for another checkout location.

Copy the reproduction scripts to a fresh writable temporary directory and run
there. Cached dependencies are required because builds use `--offline --locked`;
obtain the dependencies from the pinned manifests first if needed.

```sh
mkdir /private/tmp/ox-workflow-reproduction
cp docs/reports/2026-09-13-ox-workflow-study/*.py /private/tmp/ox-workflow-reproduction/
cp docs/reports/2026-09-13-ox-workflow-study/cache_worker.rs /private/tmp/ox-workflow-reproduction/
export FERROMARK_ROOT=/path/to/ferromark-at-0026d4c
python3 /private/tmp/ox-workflow-reproduction/prepare.py
python3 /private/tmp/ox-workflow-reproduction/run.py
python3 /private/tmp/ox-workflow-reproduction/profile.py
python3 /private/tmp/ox-workflow-reproduction/cache_prepare.py
python3 /private/tmp/ox-workflow-reproduction/cache_run.py
```

Keep the host on AC power and run no competing builds/tests/benchmarks during
measurement. Profiling uses macOS `sample` and may require permission to inspect
the child process. Profiles and counters use separate binaries from timing.
They are never used as timing samples.

The lifecycle experiment uses three fresh process rounds and nine rotating,
reversing windows per variant: at least 63 ms for collections and 30 ms for
individual documents, with 300/100 ms warmups. The cache sensitivity experiment
uses three rounds, nine rotating 63 ms windows, and 300 ms warmups. Every clock
check follows four complete operations; input loading, pool construction,
verification, and IPC are outside timing. Returned HTML and its destruction
are inside timing. Only one worker computes at a time.

These are shorter diagnostic experiments than the public 80-window protocol.
Their independent compiler units, warmed working sets, and measurement sessions
can shift absolute time. Interpret their paired controls within each experiment;
do not splice their numbers into the existing public ranking.

Each engine's state controls render 53 unique inputs in fresh processes and
compare them with 117-element sequences using shared or distinct input addresses.
The timing pools append a fixed-width nonce paragraph to all twelve inputs, then
cycle one repeated collection or 32 separately allocated collections. Changed
nonce values alter contents without changing source length. Both fixed and
changing content receive output work checks; source and state controls are
separate from performance ranking or a cold-start benchmark.

For the existing archive, verify all checksums, raw timing counts, durations,
output sizes, engine ordering, state outputs, pointers, profile summaries, and
generated tables without rebuilding:

```sh
python3 docs/reports/2026-09-13-ox-workflow-study/summarize.py --check
```

`checksums.json` protects the evidence and executed worker/helper sources.
`summarize.py` and the narrative are publication code; they are intentionally
outside that raw-evidence checksum set. The source corpus keeps its original
attribution. Ox code is referenced at pinned upstream locations, not copied into
the production implementation.
