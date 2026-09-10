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
