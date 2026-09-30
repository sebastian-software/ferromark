# Manual macOS benchmarks

Use an otherwise idle Apple Silicon Mac, including a Mac mini M1 with 16 GB,
to remeasure **all eight Native and six Node.js libraries** in the homepage
table. No runner registration, CI service, or agent is required.

## First setup

Install Xcode Command Line Tools (`xcode-select --install`), Git, curl, CMake,
Python 3.11 or newer, Node.js **24**, npm, and Rust through rustup. Use native
arm64 executables, outside Rosetta. The script installs the pinned Bun Rust
nightly through rustup; Cargo installs the stable toolchain from
`rust-toolchain.toml`. First preparation needs network access to GitHub and the
Cargo/npm registries. Dependency versions and source archive hashes stay pinned.

Clone the repository with its history; the retained six-engine harness also
exports a historical Ferromark v1 commit. For an existing shallow clone, run
`git fetch --unshallow`. Check out the branch to measure and commit its changes.
The script rejects a dirty checkout and never cleans or resets it.

```sh
git switch your-benchmark-branch
./scripts/benchmark-macos doctor
./scripts/benchmark-macos run "$HOME/ferromark-runs/m1-2026-10-01"
```

Choose a **new directory outside the checkout**. Paths with spaces are supported.
`FERROMARK_BENCH_PYTHON=/path/to/python3.13` overrides interpreter discovery.
Use `./scripts/benchmark-macos --help` for the command interface.

## What happens

`run` creates an isolated clone at `OUTPUT/source`, downloads the pinned sources,
checks the harnesses, builds both native harnesses and the plain Node addon,
and audits the exported native sources. Build concurrency is capped at four
jobs to keep memory use reasonable. Your checkout and its addon are untouched.

All HTML/option guards finish before the first timed comparison. The script
keeps the Mac awake with `caffeinate`, waits 60 seconds between lanes, and runs
one lane at a time. Each uses the existing **three process rounds, six samples,
40 ms windows, and 60 ms warmup**. All 57 frozen inputs are timed, including
diagnostic disagreements; only equivalent HTML contributes to the factors.
There is no shortened publishable mode. Logs are under `OUTPUT/logs`; commands
and current phases are printed in the terminal. Allow a long uninterrupted
session; duration depends on the Mac and the competitor.

Close heavy applications and let OS updates/indexing finish. The script does
not disable system services or claim exclusive CPU access. Host/load/thermal
observations and per-round ranges are retained so unusual noise can be reviewed.
If a result looks unstable, use a fresh output directory and compare full runs;
do not select only the fastest run. `--cooldown-seconds N` changes the pause
(default 60; range 0–600), not the measurement windows.

For separate preparation and measurement:

```sh
./scripts/benchmark-macos prepare "$HOME/ferromark-runs/m1-2026-10-01"
./scripts/benchmark-macos measure "$HOME/ferromark-runs/m1-2026-10-01"
```

To check every adapter without timing, run `./scripts/benchmark-macos verify OUTPUT`
after preparation. Verification-only output cannot be published.

After an interrupted measurement, run `measure` again on the same Mac and
revision. Fully retained pairs are revalidated and skipped. Incomplete attempts
are renamed with an `-interrupted-` suffix and remeasured. A failed preparation
needs a **new output directory**; its partial files and logs remain for diagnosis.
Do not move a prepared directory before measuring: build metadata contains
absolute binary paths. The final `evidence/` directory is portable for publication.

## Update the report and homepage

```sh
./scripts/benchmark-macos publish "$HOME/ferromark-runs/m1-2026-10-01" --check
./scripts/benchmark-macos publish "$HOME/ferromark-runs/m1-2026-10-01" \
  --name 2026-10-01-mac-mini-m1
```

You can also copy `evidence/` to another checkout of this repository and run
`publish` with its parent directory. The publishing checkout must contain the
measured commit in Git history. No build binaries, caches, or Node installation
are needed for this step; Python 3.11+ and Git suffice.

Publication rechecks complete window coverage, checksums, frozen inputs, HTML
classification, aggregates, committed sources/adapters, runtime and addon
identity, and all 14 projects. It rejects partial/shortened runs and nonpositive
or nonfinite scores. It retains compressed raw evidence and provenance under a
**new** `docs/reports/<name>/` with SHA256SUMS, selects it in
`benchmarks/manual-macos/current.json`, and regenerates the homepage values and
benchmark guide. Existing reports are immutable and Linux values are preserved.
No Git commit, push, PR, or deployment happens automatically.

The original native harness uses shared mimalloc and a pinned Bun toolchain;
its four displayed libraries retain the historical five/six-engine matching
sets. It still measures v1 internally for those sets but does not publish a v1
homepage row. Other native pairs use system malloc; Node uses the release-node
addon without PGO, including string conversion and GC. cmark/commonmark.js use
CommonMark only in both engines. Different agreement sets and build contracts
remain disclosed; the table does not establish a shared-set ranking.

Review the generated report, agreement exclusions, per-round ranges, source
revision, host observations, and Git diff. Validate generated content:

```sh
python3 benchmarks/markdown-ecosystem/publish_values.py --check
python3 benchmarks/markdown-ecosystem/publish.py --check
```

Then run the [repository checks](../../CONTRIBUTING.md) and homepage checks before
opening a PR. New ratios remain observations for this machine and corpus,
not universal performance or statistical significance claims.

## Delegate to an agent

The repository includes
[manual-macos-benchmarks](../../.agents/skills/manual-macos-benchmarks/SKILL.md).
On the benchmark Mac, ask an agent with access to this checkout:

> Use $manual-macos-benchmarks to measure the current committed branch on this
> Mac, retain the evidence, and prepare a PR updating the macOS homepage values.

An agent that does not discover repository skills can read that SKILL.md directly.
Measuring through a remote terminal is also fine; keep that session alive. The
skill directs the agent to the same commands and evidence checks as a manual run.
