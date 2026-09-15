# Session harness copies

These are the exact driver scripts used for this round, copied from the session
scratchpad. They wrap the repository harness in `benchmarks/optimization-rounds`
and are kept for reproduction, not as maintained tooling.

- `prepare_local.py` — the repository `prepare.py` with the `--baseline-revision`
  and `--symbols` additions used during the session (the former has since been
  ported to the repository script).
- `screen.sh` — runs `run.py` on a case filter and prints geometric means per
  stage and per case (`ROUNDS`, `PAIRS`, `WINDOW` environment variables).
- `confirm2.sh` — waits for the absence of compiler/profiler processes, runs an
  A/A control, then the 57 broad documents and the 45 diagnostics in all four
  stages with five pairs per round.
- `percase.py` — side-by-side per-case ratio tables across result directories.
- `profile.sh` / `aggregate_profile.py` — `xctrace` Time Profiler recording of a
  worker's timed loop and a self/inclusive summary of the XML export.
