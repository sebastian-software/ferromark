# Reproduce the optimization experiments

Requires Python 3.12+, Git and Rust 1.97.1. Use a new directory outside Cargo
repositories, and remove custom `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, build-target,
profile or compiler-wrapper environment overrides. The helper refuses ancestor
Cargo configuration files. The archived runs used the generic ARM target,
not the repository's Apple M1 flags. Keep builds/tests separate from timing.

```sh
python3 -B docs/reports/2026-09-12-ox-inspired-optimizations/prepare.py /tmp/ferromark-ox-rerun
cd /tmp/ferromark-ox-rerun/confirmation
python3 -B confirm.py
```

This restores frozen baseline source, applies `retained.patch`, checks every
retained source hash, builds both binaries with the same archived driver/lock,
verifies outputs, and runs all three confirmation pairs. Its output goes into
the temporary directory. Do not replace the archived results with a different
host's measurements.

To reproduce screens instead, use the top-level workspace. Baseline setup must
precede any candidate build:

```sh
cd /tmp/ferromark-ox-rerun
cargo build --release --locked --manifest-path driver/Cargo.toml
python3 -B experiment.py baseline
python3 -B experiment.py nibble-locate borrow-contiguous code-info-borrow headings-count direct-text-event
```

All 13 variant names and the three combination strings are in `metadata.json`.
`variants.py` recreates them independently from `baseline/src`; combinations
join names with `+`. The original `recorded-setup.py` is historical provenance
and depends on the original checkout; use `prepare.py` for reproduction.
The archived hashes, patch and raw windows identify each measured prototype.

The optional allocation/Ox stage follows the confirmation stage. Checkout the
pinned Ox source first (the restored manifest points to this location):

```sh
cd /tmp/ferromark-ox-rerun
git clone https://github.com/ubugeeei-prod/ox-content.git ox-current
git -C ox-current checkout 026d1859d1c35e5fb1ea65e7e855b428a918b9bb
cargo fetch --locked --manifest-path ox-driver/Cargo.toml
python3 -B extra.py build
python3 -B extra.py run
```

`extra.py build` records allocations with separate counting binaries, then builds
Ox without instrumentation. `extra.py run` measures three rotating native
comparisons against the preserved baseline/candidate timing binaries. Allocation
calls include reallocations; requested bytes are cumulative, not peak memory.

Generate the archived tables without rebuilding or remeasuring:

```sh
python3 -B docs/reports/2026-09-12-ox-inspired-optimizations/make-results.py
```

The generator validates summary statistics against every raw timing window
before writing `RESULTS.md`. `SHA256SUMS` covers every archive file except itself.
Source files and tests in the main repository remain the implementation under
review; the compressed source snapshot is only the frozen benchmark baseline.
