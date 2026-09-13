# Reproduce the ARM sort comparison

The benchmark compares the recorded PR head with three rejected variants.
The baseline git object in `metadata.json` must be available locally. The
recorded Cargo dependencies must be present in the local cache for offline
builds. Reproducing the original machine/compiler is necessary for comparable
timings; other machines can still verify outputs and identities.

```sh
python3 docs/reports/2026-09-13-emit-sort-arm/setup.py /tmp/ferromark-sort-replay
cd /tmp/ferromark-sort-replay
python3 - <<'PY'
from experiment import build
for name in ['baseline', 'unstable-packed', 'stable-packed', 'stable-tuple']:
    assert build(name)
PY
python3 verify-cases.py
python3 memory.py
python3 screen.py
python3 confirm.py
```

`confirm.py` copies the baseline binary for the A/A control, then uses the
frozen 17-case follow-up selection. All workers use the same harness, profile
and input files. Run timing alone, without concurrent builds or allocation
probes. Memory binaries are instrumented independently and never used for
timing. Each candidate's source snapshot and full source hashes identify what
was measured; rebuilding does not regenerate a candidate with a future formatter.

`python3 docs/reports/2026-09-13-emit-sort-arm/check-evidence.py` checks hashes,
reconstructs the frozen sources, recalculates medians from all raw windows,
checks allocation samples and output identities, and checks generated tables.
It validates the archived evidence, not the timing behavior of the current
checkout.
