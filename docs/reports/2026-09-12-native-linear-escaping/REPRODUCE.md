# Reproduce the native Linux escape integration

Use a checkout containing this report and baseline commit
`e731dc8162828e0ac2df30b7567107f560159ff8`. Timing is native Rust library code;
there is no Node or FFI wrapper. The JSON protocol surrounds the timed operation.

Inspect `measurements/*/native-metadata.json` for the exact runner CPU, toolchain,
experiment commit and binary identities. Original orchestration scripts are
archived as `recorded-run.py`; these expect their original experiment checkout.
The experiment-only workflow is available at each recorded commit and is not
part of the production change.

For a fresh replay, from the repository root:

```bash
python3 docs/reports/2026-09-12-native-linear-escaping/check-evidence.py
python3 docs/reports/2026-09-12-native-linear-escaping/setup.py round-3-1 /tmp/escape-replay
cd /tmp/escape-replay
python3 experiment.py baseline masked-direct --rounds 5 --ms 50 --runs 3
python3 verify-all.py baseline masked-direct
python3 direct.py masked-direct --pair 1
python3 - <<'PY'
from regimes import run_regimes
run_regimes('masked-direct', rounds=5, ms=50, tag='regime-replay')
run_regimes('masked-direct', rounds=5, ms=50, tag='attribute-replay',
            casefile='attributes.json', selection='attribute-selected.json')
PY
```

Choose another archived round/variant to replay a rejected alternative. Build
all competing binaries before timing; do not run builds, tests or profilers in
parallel with a timing process. The original orchestration alternates variants
and baseline/candidate window order and runs an identical-binary A/A control
before and after the candidate series. Use the recorded script for the complete
schedule; the commands above are a smaller correctness and timing replay.

The driver manifest uses optimized release code, fat LTO, one codegen unit and
panic abort. Its lockfile pins dependencies. Ambient Rust flags are removed by
the build script, and builds run from the reconstructed directory so repository
`.cargo/config.toml` does not change the target configuration. Downloads happen
before timed execution. Inputs/options are loaded outside the timer; owned
output allocation, rendering/escaping and destruction are inside it.

Direct controls use their recorded capacity flags: default input-length
capacity, doubled capacity for HTML-heavy inputs, and explicitly named
`late-quote-reserved-*` controls with 16 additional bytes. These are different
allocation regimes and are not combined into a memory claim. Ordinary Markdown
and stress controls preserve the original report's allocation policy. The
`window-escape-single` and `window-single` protocols check elapsed time after
one render; other windows check after 16 renders. Very slow baseline renders
can exceed the nominal 50 ms window, and their full elapsed time is retained.

`results.py` generates all result tables directly from archived windows and
verifies the summary arithmetic. `check-evidence.py` also verifies archive
hashes, frozen source identities, exact-output guard counts and driver hashes.
The first native round did not retain executable hashes; later rounds do.
A/A always uses a copied baseline executable, as shown by the recorded scripts.
Native throughput measurements exercise AVX2 on the recorded CPUs. SSE2 is
called directly by the native alignment/oracle test; throughput on a non-AVX2
machine remains unmeasured.
