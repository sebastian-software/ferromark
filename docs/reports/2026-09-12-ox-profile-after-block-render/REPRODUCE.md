# Reproduce the follow-up profiles and screens

The comparison freezes Ferromark at merged PR #312,
`e731dc8162828e0ac2df30b7567107f560159ff8`, and Ox Content at
`026d1859d1c35e5fb1ea65e7e855b428a918b9bb`. It reuses the exact corpus,
normalizer and standalone dependency resolutions from the preceding reports.
The resolved documentation commits and file hashes are in the
[corpus manifest](../2026-09-12-block-render-hotspots/corpus/corpus-manifest.json).
Licenses and attribution accompany this report under `licenses/`.

Run from the repository root. Supply a clean Ox checkout at the pinned revision:

```sh
REPORT=docs/reports/2026-09-12-ox-profile-after-block-render
python3 "$REPORT/check-evidence.py"
python3 "$REPORT/make-results.py" --check
python3 "$REPORT/setup.py" /tmp/ferromark-round7-replay --ox-source /path/to/ox-content
```

The setup verifies source/input identities and reconstructs both standalone
workspaces without changing the repository checkout. It reuses the larger
compressed input and guard archives from the preceding report rather than
copying those datasets into this report. Use Rust 1.97.1 to match this run.
Dependencies must be in Cargo's cache for the offline builds below; the lockfiles
pin their versions. Run all timed operations serially, without builds, tests or
sample collection in parallel.

```sh
cd /tmp/ferromark-round7-replay
python3 build.py release
python3 run.py verify
python3 run.py baseline-1
python3 run.py baseline-2
python3 run.py baseline-3
python3 build.py sample
python3 profile.py
python3 prepare-attribution.py
python3 attribution-profile.py
cargo build --offline --locked --release --manifest-path demangle/Cargo.toml
python3 analyze-profiles.py
```

`sample` is a macOS profiler. Its attachment to the child processes requires
permission outside restrictive sandboxes. The sample builds retain optimized
code, debug information and frame pointers. They are separate from the release
timing binaries. The attribution probe adds only three `inline(never)` boundaries
and verifies all 649 outputs before its four captures. Its percentages are not
latency measurements or a production patch.

For the independent prototypes, first create the new baseline oracles:

```sh
cd experiments
python3 experiment.py baseline
python3 verify-timing.py baseline
python3 verify-blocks.py baseline
python3 verify-extra.py baseline
python3 verify-extended.py baseline
python3 verify-fences.py baseline
```

Use `experiment.build(name)` and `experiment.verify(name)` to build a named
prototype and check the initial 2,695 outputs. Then pass the name to each of the
five guard scripts above. Only time a source after every guard passes. The
rejected `softbreak-ranges` and `softbreak-no-code` variants intentionally fail
those guards; their failures are archived. The scripts preserve the chronological
screening sequence, including the expected rejection that stops `screen.py`.

For example, reproduce the leading candidate:

```sh
python3 -c "from experiment import build, verify; assert build('escape-single-scan') and verify('escape-single-scan')"
python3 verify-timing.py escape-single-scan
python3 verify-blocks.py escape-single-scan
python3 verify-extra.py escape-single-scan
python3 verify-extended.py escape-single-scan
python3 verify-fences.py escape-single-scan
python3 experiment.py escape-single-scan --no-build
```

The default experiment command performs the initial 93-input screen after the
initial oracle check. For the stricter recorded order, run `build`, all six
verification operations, and `run` separately. `confirm.py` requires all four
correct initial prototype binaries and runs two longer pairs on 37 inputs;
`autolink-screen.py` builds, fully checks and measures the later prefilter probe.
All patches in this report are diagnostic; no production parser change is part
of this profiling round.
