# Reproduce the native heap audit

Requires Python 3.12+, Git and Rust 1.97.1. Use a fresh temporary workspace outside
Cargo repositories, without custom Rust flags, profile settings or wrappers.
The primary Ox mode uses its growing arena; presizing is a diagnostic control.

```sh
git clone https://github.com/ubugeeei-prod/ox-content.git /tmp/ox-memory-source
git -C /tmp/ox-memory-source checkout 026d1859d1c35e5fb1ea65e7e855b428a918b9bb
python3 -B docs/reports/2026-09-12-ox-memory/prepare.py /tmp/ferromark-memory-rerun /tmp/ox-memory-source
cd /tmp/ferromark-memory-rerun
cargo build --release --locked --manifest-path ferro/Cargo.toml
cargo build --release --locked --features oxide --manifest-path ox/Cargo.toml
python3 -B run.py
```

The helper reconstructs Ferromark from the preceding report's frozen baseline
and retained patch, then verifies all source hashes. Ox's checkout revision and
tracked crate changes are checked. Both binaries execute their allocator
self-tests at startup. `run.py` verifies exact allocation-counter repetition,
checks normalized HTML and writes raw samples plus summaries. Instrumented
binaries must not be used for performance timing.

To regenerate the archived tables without compiling or measuring:

```sh
python3 -B docs/reports/2026-09-12-ox-memory/make-results.py
```

The generator validates every summary against all 30 underlying observations.
`SHA256SUMS` covers every file except itself. Preserve the recorded data when
reproducing on another host; write new results to the temporary workspace.
