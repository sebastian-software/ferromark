# Reproduce the linear escaping experiment

Run from the repository root. Python 3.12+, Git, `patch` and the recorded Rust
toolchain are needed. This reuses frozen corpora from the earlier block-render
and NEON reports; their manifests preserve source attribution. No mutable upstream
corpus download is required. Cargo dependencies must already be cached for the
locked offline build, or fetched explicitly before measurement.

```bash
python3 docs/reports/2026-09-12-linear-html-escaping/check-evidence.py
python3 docs/reports/2026-09-12-linear-html-escaping/setup.py /tmp/ferromark-linear-replay
cd /tmp/ferromark-linear-replay
python3 run-recorded.py linear/confirm-1
python3 run-recorded.py linear/regime-confirm-1
python3 run-recorded.py linear/attribute-confirm-1
```

`runs.json` lists every archived run and its exact case indices, process/window
order, duration, warmup and operation. Pass any key to `run-recorded.py`.
It builds the baseline and candidate, runs the differential guards, then measures
that pair. Run one timing process at a time, with no compilation or profiler
running concurrently. `follow-selected.json` in the validation directory records
the post-screen counterexample selection; `follow.py` documents the selection rule.
The full procedure uses three fresh primary pairs plus longer follow-ups, not
one favorable replay.

The standalone driver uses native Rust-to-Rust calls. There is no Node wrapper,
external process per document, pre-reserved output buffer, changed allocator or
Ox arena in this before/after experiment. The explicitly named reuse cases retain
their original reusable parser behavior. Options and document strings are built
outside the timed region, and owned output is dropped within it.

For A/A runs, the candidate executable is a byte-identical copy of the baseline.
SHA-256 identities are in `metadata.json`; executable files and build directories
are not committed. Binary hashes describe the original build, and are not a
promise of reproducibility across toolchains or host paths. Source hashes and
complete input/driver hashes are checked independently.

The setup script extracts the frozen baseline revision with `git archive`.
`variants.py` reconstructs the screened changes. The formatted `production`,
`final` and `linear` sources additionally have compressed escape-module snapshots;
the report checker confirms they match their patches and complete source hashes.
The first two names refer to intermediate experiments that were not adopted.
The `linear` snapshot is the original measured implementation. The final
ARM-only scope is described below; its NEON machine code is identical.

`check-evidence.py` validates every manifest entry, reconstructs every source
patch, checks guard summaries, verifies that each requested case has every
baseline/candidate window in the recorded order, recomputes medians and changes,
and checks that `RESULTS.md` is generated from those windows. It does not assert
that future timings must match, or replace the full repository test/lint gates.

The work-count reproducer in `validation/baseline-work-tests.diff` applies to the
frozen baseline. Its recorded test run is deliberately failing: both quote-heavy
inputs exceed the doubling-work budget while still returning the expected HTML.
The proposed source includes the corresponding dense and sparse regression cases
and the scalar-oracle boundary checks. Run the current tests with:

```bash
cargo test --locked --all-features escape::tests
```

Timing conclusions are limited to the recorded Apple M1 Pro. Local cross-target
checks and native CI validate other code paths' correctness; they do not establish
x86, Windows or scalar-target throughput.

`validation/replay-check.log` records a successful independent setup and replay
of one short attribute follow-up. Its timings are a tooling smoke check and are
not included in the recorded comparisons or generated result tables.

## Final ARM-only source

The recorded `linear` candidate remains available for historical replay. The
production PR uses `snapshots/arm-scoped-escape.rs.gz`; non-NEON targets retain
the frozen baseline. Apply `arm-scoped.patch` to the baseline to reconstruct it.
`arm-scope.json` records every resulting source hash and the comparison with the
original ARM executable's machine-code section. Run `check-arm-scope.py` on an
ARM64 macOS machine with a fresh destination to rebuild both sources in the same
harness and verify code identity:

```bash
python3 docs/reports/2026-09-12-linear-html-escaping/check-arm-scope.py /tmp/arm-scope-replay
```
