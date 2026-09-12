# Reproduce the HTML block experiment

Use a checkout containing the merged PR #309 baseline identified in
`metadata.json`. The recorded environment is Rust 1.97.1 on Apple M1 Pro,
aarch64-apple-darwin, system allocator. The standalone manifest pins dependencies
and release settings; scripts remove inherited Rust flags and run outside the
repository Cargo configuration.

## Reconstruct and verify

Choose a new scratch directory:

```sh
python3 docs/reports/2026-09-12-html-block-parser/replay.py \
  --work /tmp/ferromark-html-replay --verify
```

The replay checks every archive checksum, reconstructs the frozen Rust files from
Git, validates their hashes, and builds both sources. It checks the rebuilt
baseline against the archived output oracle before comparing production. It then
replays the block/MDX, inline, extended and timing-input guards. Dependency fetching
needs the network only when locked dependencies are absent from the Cargo cache.
The archived input bytes and oracles are authoritative behavior snapshots.

## Timing and memory

After preparing that scratch directory, run from it:

```sh
python3 experiment.py production --no-build --rounds 9 --ms 75 --runs 3
python3 memory.py baseline production
```

Run these suites sequentially, without builds, tests or profilers competing with
timed windows. The production code is `production-parser.rs`; every other parser
file remains at the frozen baseline. `variants.py` reconstructs the exploratory
candidates, and their directories retain source hashes, patches and guard results.
The final production confirmations use freshly built formatted source.

The memory driver and its complete sources, lockfile and frozen dataset are
archived under `memory/`. Its counters track requested allocations with HTML live
at the observation boundary. They exclude input storage and do not measure RSS.
The suite includes large ordinary documents and HTML cases up to 8 MiB.

## Profiles and Ox corpus

`profile.py` builds optimized sampling executables with debug information and
frame pointers. It uses macOS `sample`, which may require permission to attach to
the benchmark child process. The captures and logs are retained under `profiles/`.
`driver/src/baseline-profile.rs` records the baseline sampling harness before the
full MDX snapshot field was added; that operation is outside the render loop.
The baseline and production sample builds both pass the original output guard.

Build the archived demangler with `cargo build --release --locked --manifest-path
demangle/Cargo.toml` before running `analyze-profiles.py`.

The corpus comparison uses a separate driver and normalization boundary. Its
inputs, driver source, lockfile, verification and raw windows are under `corpus/`.
Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; see the preceding
[corpus investigation](../2026-09-12-ox-corpus-profiling/REPORT.md) for its build and
input provenance. `prepare-corpus.py` records the original local staging setup;
reproduction on another machine requires rebuilding that pinned Ox driver and
placing the two native executables under `corpus/bin/`. Run `corpus/run.py verify`
before any timed corpus run. The growing arena is primary; presizing is separate.
Only equal normalized outputs support a ranking.
