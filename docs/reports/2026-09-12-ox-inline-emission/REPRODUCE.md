# Reproduce the inline emission experiment

Run from a checkout containing baseline commit
`96e78323a9d263530e2564cdd7ef8fa3facc0ac0`. The recorded environment uses Rust
1.97.1 on aarch64-apple-darwin. Its default CPU is `apple-m1`. The standalone
manifest pins dependencies and supplies the release profile; scripts remove
inherited Rust flags and build outside the repository Cargo configuration.

## Exact rendering and event checks

Choose a new scratch directory outside the checkout:

```sh
python3 docs/reports/2026-09-12-ox-inline-emission/replay.py \
  --work /tmp/ferromark-inline-replay --verify
```

The replay validates every archive checksum, reconstructs the original Rust
source from Git, checks each source hash, and builds the baseline and production
variants. It checks the rebuilt baseline against the archived oracle before
comparing production. Frozen inputs cover HTML, resource-limit reports, heading
state, reused renderers, and public inline/MDX events. Dependency fetching needs
network access only if the locked dependencies are absent from the Cargo cache.

`prepare-extra.py` and `prepare-extended.py` retain the input-generation method;
the archived input bytes and hashes are authoritative. Their deterministic
combinations are behavior-preservation checks, not a normative Markdown oracle.

## Timing

After preparing the scratch directory, run its scripts from that directory.
Keep builds, tests, profilers, and other timing suites out of measured windows.

```sh
python3 experiment.py production --no-build --rounds 9 --ms 75 --runs 3
```

This starts three new process pairs against the frozen baseline, with nine
alternating windows per implementation/input and a separate warmup. The final
source is `production-inline.rs`; all other parser source remains at baseline.
`variants.py` and `compact_variant.py` reconstruct the exploratory candidates;
each candidate directory contains its patch and output admission results.
Candidates with failed guards are rejected even when a preliminary timing exists.

`confirm.py` retains the separate three-engine comparison of the baseline, the
code/HTML combination, and the combination with compact payloads. Its longer
warmup and rotating order are distinct from the final paired production run.
The report does not mix their absolute times.

## Memory, profiles, and competitor corpus

The `memory/driver` directory contains the original counting allocator,
normalizer, and locked manifest. The frozen 45-case dataset includes large inputs.
The raw records retain ten observations per process and three processes per
implementation. HTML hashes must match before comparing requested live bytes.
Counters describe allocations during a fresh render with output still live;
input storage and process RSS are outside this metric.

The layout probe compiles the actual private point/kind declarations with the
recorded compiler. `layout/sizes.json` records their sizes. Public event types are
not changed.

Native sample captures use release optimization plus debug information and
frame pointers. `profiles/` retains both captures per input and the sampling logs.
The baseline sampling binary predates the additional `events` harness operation;
that operation is never used in the sampled render loop. Both sampling binaries
pass the original frozen output guards. Sampling percentages are diagnostic,
not timing results.

The fresh head-to-head comparison uses the existing Ox corpus driver and
normalization boundary. Its source, lockfile, input manifest, verification, and
raw windows are under `corpus/`. Ox is pinned to
`026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; obtain its build instructions and corpus
provenance from the preceding
[corpus investigation](../2026-09-12-ox-corpus-profiling/REPORT.md). The growing
arena is primary; the presized arena is separately identified. Unequal outputs
remain diagnostics and do not support a speed ranking.
