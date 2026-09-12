# Reproduce the block/render hotspot experiment

Use a checkout containing merged PR #311, the frozen baseline identified in
`metadata.json`. The recorded environment is Rust 1.97.1 on Apple M1 Pro,
aarch64-apple-darwin, system allocator. The standalone manifest pins dependencies
and release settings. Scripts remove inherited Rust flags and run outside the
repository Cargo configuration.

## Reconstruct and verify

Choose a new scratch directory:

```sh
python3 docs/reports/2026-09-12-block-render-hotspots/replay.py \
  --work /tmp/ferromark-block-render-replay --verify
```

The replay validates archive checksums, reconstructs frozen Rust sources from Git,
checks source hashes and builds both implementations. It verifies the rebuilt
baseline against its archived oracle before replaying all production guards.
Dependency fetching needs the network only when locked dependencies are absent
from the Cargo cache. Archived inputs and oracles are the authoritative behavior
snapshots.

## Timing and memory

From the prepared scratch directory:

```sh
python3 experiment.py production --no-build --rounds 9 --ms 75 --runs 3
python3 memory.py baseline production
python3 large-timings.py
```

Run suites sequentially, without builds, tests or profiles competing with timed
windows. `production-source/` contains the changed production files; every other
Rust source remains at the frozen baseline. `variants.py` reconstructs the
exploratory candidates. Their directories retain patches, hashes, output checks
and raw timings. `scan-screen.py` builds named candidates and checks original outputs plus
every selected timing input before their paired screen. The first four screens
used 81 inputs; subsequent screens and final confirmation use the expanded
93-input selection. The production source runs every guard set, including
12,000 deterministic fence/container/MDX cases. To replay an original 81-input
screen, call `experiment.run(name, indices=selected[:81])`, where `selected` is
loaded from `selected.json`. The appended cases do not change existing indices.

The memory driver includes all sources, its lockfile and the complete frozen
57-input dataset under `memory/`. It measures requested allocations with owned
HTML still live, excludes input storage and does not measure RSS.

## Profiles

From the scratch directory, on macOS:

```sh
python3 profile.py utf8-attribution
python3 profile.py baseline
python3 profile.py production
cargo build --release --locked --manifest-path demangle/Cargo.toml
python3 analyze-profiles.py
```

Sampling executables use optimized code with debug information and frame pointers.
The profiler attaches to its own rendering child process; macOS may require
permission for that operation. `utf8-attribution` only prevents `HtmlWriter::into_string` from
being inlined, so its source-line attribution is diagnostic. Actual baseline and
production captures are separate. All sampling binaries pass the original output
guard before their captures. Raw captures, metadata and logs are under `profiles/`.

## Ox corpus

The corpus comparison uses a separate native driver and normalization boundary.
Inputs, driver source, lockfile, verification and raw windows are under `corpus/`.
Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; see the preceding
[corpus investigation](../2026-09-12-ox-corpus-profiling/REPORT.md) for its build and
input provenance. `prepare-corpus.py` records the original local staging setup.
On another machine, rebuild the pinned Ox driver and place the two executables
under `corpus/bin/`. Run `corpus/run.py verify` before timed corpus runs. The growing
arena is primary; presizing is separate. Only equal normalized outputs support a
comparison.

## Evidence integrity

Run `python3 check-evidence.py` in the archive to check all file hashes and
recompute paired timings, corpus medians and allocation summaries. Logs and patches retain original bytes.
The independent reconstruction/replay log is under `validation/`.
