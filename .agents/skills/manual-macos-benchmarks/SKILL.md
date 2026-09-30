---
name: manual-macos-benchmarks
description: Remeasure all Ferromark homepage comparisons on a manually operated Apple Silicon Mac, validate retained evidence, and update macOS reports and homepage values. Use for new benchmark runs or importing a completed manual run.
---

# Manual macOS benchmarks

Use the repository's `scripts/benchmark-macos` commands. Read
[the runbook](../../../benchmarks/manual-macos/README.md) before executing;
resolve all commands from the repository root. Follow root AGENTS.md and
CONTRIBUTING.md for checks and PR conventions.

- For measurements, run `doctor`, then `run OUTPUT` in a new directory outside
  the checkout. Measure the committed branch requested by the user. Preserve
  dirty changes; use a clean worktree or commit only the authorized changes.
  Never reset or clean the user's checkout to satisfy preflight.
- All eight Native and six Node.js projects must finish. Do not substitute
  shortened windows, synthetic results, or copied historical values. Builds,
  downloads, and HTML verification finish before timing. Keep workloads quiet;
  disclose observed noise instead of promising an idle machine.
- To resume an interrupted measurement, use `measure OUTPUT` on the same Mac
  and revision. Inspect `OUTPUT/logs` on failure; the script preserves partial
  attempts. A failed preparation needs a new output directory.
- To import an existing run, use its portable `evidence/` directory under OUTPUT.
  The importing checkout needs the measured commit in its Git history.
- Run `publish OUTPUT --check`, inspect agreement exclusions, per-round ranges,
  host observations and source provenance, then run `publish OUTPUT --name
  YYYY-MM-DD-mac-mini-m1` to prepare a local update. Choose a new report name.
  Do not hand-edit factors, rewrite old reports, mix Linux into the macOS run,
  or claim a ranking across different agreement sets and build contracts.
- Review the diff: only the new evidence, current-report pointer, generated
  platform values and guide should change. Run both ecosystem publishers with
  `--check` and the applicable repository/homepage checks. Report any checks
  that could not run and distinguish infrastructure validation from a full
  measurement on the benchmark Mac.
- Commit and create a PR when requested. Local publication does not authorize
  merging, deployment, or configuring the Mac as a GitHub runner. Include the
  machine, measured commit, commands, evidence report, and material limits in
  the result. Link the new PR and retained report.
