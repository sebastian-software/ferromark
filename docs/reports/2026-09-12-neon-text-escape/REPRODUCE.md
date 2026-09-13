# Reproduce the escape integration investigation

Run from the repository root. Python 3.12+, Rust/Cargo and `patch` are required.
No Ox build or Node wrapper is used for these before/after experiments. The
preceding native-profile report contains its separate Ox reproducer.

```bash
python3 docs/reports/2026-09-12-neon-text-escape/check-evidence.py
python3 docs/reports/2026-09-12-neon-text-escape/setup.py /tmp/ferromark-escape-replay
cd /tmp/ferromark-escape-replay
python3 experiment.py baseline --runs 1
python3 verify-all.py baseline
```

Build and verify candidates explicitly, then choose the measurement you want:

```bash
python3 - <<'PY'
from experiment import build, verify
for name in ['single-neon', 'direct-neon', 'outlined-neon', 'tail-neon', 'bulk-neon', 'bulk-inline-neon']:
    assert build(name) and verify(name)
PY
python3 verify-all.py single-neon
python3 verify-timing.py direct-neon outlined-neon tail-neon bulk-neon bulk-inline-neon
python3 experiment.py direct-neon --no-build --runs 3 --rounds 7 --ms 50
python3 - <<'PY'
from regimes import run_regimes
run_regimes('single-neon', tag='regime-screen-complete')
PY
```

Cargo builds run offline with the archived lockfile. If the exact dependencies
are not cached, first run `cargo fetch --locked --manifest-path driver/Cargo.toml`
on a machine permitted to fetch them. Build all candidates and finish output
guards before timing. Keep tests, builds, profilers and other CPU-heavy work out
of timing windows. Binary hashes are the recorded machine's identities, not a
promise of bit-identical compilation across machines and toolchains.

`setup.py` reconstructs the merged source from Git, reuses the preceding
block-render report's archived ordinary/differential fixtures, appends this
report's 27 stress cases and validates input/source hashes. It applies the
archived direct-scan patch separately because that measured source includes a
formatted proposed test. `variants.py` constructs the other exact patches.
`direct-neon` was originally named `production`; its archived raw engine label
is retained. It is a rejected experiment, not a shipped version.

To repeat a recorded run with its exact case list and timing settings:

```bash
python3 run-recorded.py direct-neon/confirm-1
python3 run-recorded.py tail-neon/follow-1
python3 run-recorded.py single-neon/regime-screen-complete
```

`runs.json` contains every recorded selection, window count/duration, warmup and
initial engine order. `run-recorded.py` verifies the candidate's initial output
guards before dispatch. Use a fresh replay directory, and never move an output
directory while its runner is active. The stress runner writes `complete.json`
only after all 27 cases finish.

`results.py` regenerates all displayed figures from raw windows and independently
checks the recorded summaries. `check-evidence.py` checks the file manifest,
source patches, driver identity, window membership/counts and output guard
records. These checks validate the archived evidence; rerunning the native guards
and timings reproduces the actual operations. New timings belong in a new
snapshot rather than replacing the recorded observations or their checksums.
