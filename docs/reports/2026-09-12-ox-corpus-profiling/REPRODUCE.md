# Reproducing this profiling round

Use a dedicated directory outside the Ferromark checkout. The helper reconstructs
Ferromark at the merge commit, Ox at its pinned revision, and all documentation
at the exact recorded commits. It verifies source/file hashes and case order.
It checks out fetched Git revisions inside that directory; do not use a working
directory containing unrelated work. Python 3.12+, Git, Cargo, and network access
for the initial source/dependency fetch are needed.

From the Ferromark repository root:

```bash
python3 -B docs/reports/2026-09-12-ox-corpus-profiling/prepare.py /tmp/ox-corpus-replay
cd /tmp/ox-corpus-replay
cargo fetch --locked --manifest-path ferro/Cargo.toml
cargo fetch --locked --manifest-path ox/Cargo.toml
python3 -B build.py
python3 -B run.py verify
python3 -B run.py screen
python3 -B run.py confirm-1
python3 -B run.py confirm-2
```

Do not run timing commands in parallel with builds, profilers, tests, or each
other. `build.py` clears ambient `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS`; its working
directory avoids the repository's CPU-specific `.cargo/config.toml`. The
standalone lockfiles are separate so Ox does not upgrade Ferromark's dependencies.
Original inputs and options are loaded before timing. Verify results before
interpreting ratios: platform/compiler changes can expose new divergences.

On macOS, collect native stack samples and separate work counters:

```bash
python3 -B build.py sample
python3 -B profile.py 442 568 63 34 114 625 648 97 132 509 644
python3 -B profile.py 442 568
cargo build --release --locked --manifest-path demangle/Cargo.toml
python3 -B analyze-profiles.py
python3 -B counters.py
```

`sample` may require permission to inspect the child benchmark process. Each
capture requests six seconds at a 1 ms interval; captures are sequential. The
helper places debug-symbol bundles beside the copied profile binaries. The
indices refer to `case-manifest.json`, whose order is validated during setup.
The main profiling calls use the matched profile, including the concatenations;
the separate `upstream` preset is covered by timing rather than CPU sampling.

To repeat the bounded line-scan experiments:

```bash
python3 -B probe-lines.py
python3 -B make-results.py
```

The experiment copies the frozen source into `probe-source`, builds each
independent patch, checks every case's exact HTML against the baseline, then
runs two paired timing passes. It restores the copied parser source afterward.
It never writes to production source. The measured binaries remain in `bin/`.
These are diagnostic experiments, not automatically shippable changes.

The normalizer tests and standalone harness lint checks are:

```bash
cargo test --offline --locked --manifest-path ferro/Cargo.toml
cargo clippy --offline --locked --manifest-path ferro/Cargo.toml --features counters -- -D warnings
cargo test --offline --locked --manifest-path ox/Cargo.toml
cargo clippy --offline --locked --manifest-path ox/Cargo.toml -- -D warnings
```

The archived `baseline-mismatch-check.json` is a historical comparison against
the pre-optimization binary from the preceding experiment; the reproduction
helper does not rebuild that old binary. Its source revision is in metadata.
The exact probe comparisons and current Ox comparisons are fully rerunnable.

The full corpus is fetched rather than duplicated here. License notices and source attribution are in
[`licenses/ATTRIBUTION.md`](licenses/ATTRIBUTION.md); file and commit provenance is in `corpus-manifest.json`. Complete
mismatch outputs are archived for original files only. Concatenated outputs are
regenerated from the same corpus. `SHA256SUMS` verifies the archived artifacts:

```bash
shasum -a 256 -c SHA256SUMS
```
