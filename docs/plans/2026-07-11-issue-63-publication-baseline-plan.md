# Issue #63 implementation plan: publication baseline

## Goal

Turn the existing diagnostic harness into a reproducible publication-baseline
runner. It measures the current portable, non-PGO `main` implementation; it
does not change parser or renderer behavior.

## Steps

1. Extend the Criterion profiling matrix with secure-default Extended runs at
   5, 20, and 50 KB while retaining trusted CommonMark parity at 5, 20, 50,
   and 250 KB and profile-cost runs at 50 KB.
2. Add a runner that refuses a dirty checkout, executes three Criterion
   repetitions with 80 samples, a five-second measurement window, and a
   three-second warmup, then copies each run's machine-readable estimates to
   the ignored result directory.
3. Record one JSON environment probe beside every Criterion result set so the
   commit, compiler, target, Rust flags, CPU mode, and machine can be audited.
4. Document the command, lane semantics, artifacts, and publication boundary.
5. Run the harness on the clean committed baseline, summarize the three
   repetitions in the performance report, and apply the full Rust validation
   gates before opening the Issue #63 PR.

## Acceptance checks

- The runner covers every Issue #63 lane and leaves three timestamped result
  directories with Criterion `estimates.json` files and an environment probe.
- The report states the exact lane, toolchain, and result quality; it does not
  promote numbers to the README before the publication protocol is complete.
- The harness passes formatting, locked tests, strict Clippy, and the
  repository's CommonMark/security guardrails.
