---
name: manual-benchmarks
description: Remeasure all Ferromark homepage comparisons on a manually operated host, validate retained evidence, and update reports and values for the detected OS and CPU architecture. Use for new benchmark runs or importing a completed manual run.
---

# Manual benchmarks

Use the repository's `scripts/benchmark-comparison` commands. Read
[the runbook](../../../benchmarks/manual-comparison/README.md) before executing;
resolve commands from the repository root. Follow root AGENTS.md and
CONTRIBUTING.md for checks and PR conventions.

- Before freezing an official campaign, read [candidate coverage](../../../benchmarks/manual-comparison/candidate-coverage.md).
  Record included/deferred candidates and their runtime/workload contracts. The
  current 14-project executable matrix has documented gaps; do not treat release
  freshness as proof that every relevant alternative is covered. Candidate review
  records are not runnable adapters or benchmark evidence.
- Before an official campaign, check all 14 competitor versions against their
  official registries or released tags, including Node extensions and adapter
  toolchain requirements. Read [dependency readiness](../../../benchmarks/manual-comparison/dependency-readiness.md).
  Refresh the pins and locks in a separate reviewed change when needed; keep
  historical reports and published figures tied to their measured versions.
- After a dependency or adapter refresh, run `doctor`, `prepare OUTPUT`, then
  `verify OUTPUT` locally before allocating managed runner time. This checks
  installation, all native builds, the Node addon, every option guard, and all
  57 inputs. Report which host actually passed; do not infer another platform
  passed. A diagnostic timing smoke must stay separate from publishable evidence.
- For measurements, run `doctor`, then `run OUTPUT` in a new directory outside
  the checkout. Measure the committed branch requested by the user. Preserve
  dirty changes; use a clean worktree or commit only the authorized changes.
  Never reset or clean the user's checkout to satisfy preflight.
- Use the detected host OS and architecture. The native harness supports macOS
  and Linux on arm64/x86-64. Avoid Rosetta or mismatched Python/Node/Rust targets.
  Keep workloads quiet and prevent suspend; disclose observed noise.
- For an explicitly requested Blacksmith run, use the runbook's manual workflow
  in `sebastian-software/ferromark`. Start with the resource/tool probe, then
  output verification, then complete measurements on independent allocations.
  Download the portable evidence and review all trials before importing one.
  Preserve recorded provider/run metadata; a VM is not a local physical host.
- All eight Native and six Node.js projects must finish. Do not substitute
  shortened windows, synthetic results, or copied historical values. Builds,
  downloads, and HTML verification finish before timing.
- Resume interrupted measurements with `measure OUTPUT` on the same host and
  revision. Inspect `OUTPUT/logs` on failure; partial attempts are preserved.
  A failed preparation needs a new output directory.
- Import an existing run using its portable `evidence/` directory under OUTPUT.
  The importing checkout needs the measured commit in Git history. It can be
  on a different host; publication uses the recorded measurement platform.
- Run `publish OUTPUT --check`, inspect agreement exclusions, per-round ranges,
  host observations and source provenance, then `publish OUTPUT` to prepare a
  local update. It selects only the measured platform; other OS/architecture
  values and historical reports stay intact. Do not hand-edit factors, mix
  platform results, or claim a ranking across different agreement sets/contracts.
- Review the new evidence, platform selection, generated values and guide.
  Run both ecosystem publishers with `--check` and applicable repository and
  homepage checks. Report checks that could not run and distinguish platform
  routing tests from full builds or measurements on each hardware type.
- Commit and create a PR when requested. Local publication does not authorize
  merging, deployment, or configuring runners. Include the measured machine,
  OS/architecture, commit, commands, evidence report and material limits.
