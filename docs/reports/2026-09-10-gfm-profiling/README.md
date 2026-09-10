# GFM profiling evidence

`summary.json` contains the compact baseline results. `raw/` preserves the
original probe, generated inputs, timings, allocation counts, and macOS CPU
sampling output from baseline `9c766f11a07cfe65af441d1a147cbfe5703d60b1`.
No experimental parser changes are needed to reproduce this baseline.

To rerun the original probe on that baseline, copy `raw/measurement-probe.rs`
to `examples/gfm_profile_probe.rs`, create `target/gfm-profile/`, and run:

```sh
cargo run --locked --profile release-debug --example gfm_profile_probe -- fixtures
cargo run --locked --profile release-debug --example gfm_profile_probe -- measure
cargo run --locked --profile release-debug --features profiling --example gfm_profile_probe -- counts
```

The source is archived verbatim to preserve its recorded hash. It uses System
allocation counting only in the profiling-feature build; never compare timings
from that instrumented executable against the uninstrumented executable.
The archived Python scripts document the local capture/summary procedure and
its original paths. CPU captures use the existing `profile_harness` example.
The output-equivalence JSON was obtained by a separate verification invocation;
the archived timing probe checks API equivalence within each configuration.

## Paired optimization measurements

`examples/gfm_optimization_probe.rs` is the runnable follow-up probe. Build it
without features under `release-debug`, and retain a copy of the executable
before changing parser code. Build and copy each candidate identically. Then:

```sh
python3 scripts/compare-gfm-probes.py target/gfm-profile/baseline target/gfm-profile/candidate target/gfm-profile/comparison.json
```

The driver first requires identical HTML for all 90 input/preset combinations,
then alternates binary order over seven 150 ms windows per case and API lane.
It includes the repository README as a real-world document. Negative paired
percentage changes mean less elapsed time. Raw windows and executable hashes
are saved with the summary. Do not run builds or other CPU-intensive checks
concurrently with this comparison. Use the separate `profiling` build and
`counts` mode to verify allocation changes.
