# Manual benchmark comparisons

Use an otherwise idle machine to remeasure **all nine Native and thirteen Node.js
execution variants** in the homepage table. The workflow detects the host OS and CPU
architecture. Publishing updates that platform's values and keeps other
platforms' selections and historical reports intact. No runner registration,
CI service, or agent is required.

The native harness supports **macOS and Linux on arm64 or x86-64**. This includes
Apple Silicon and Intel Macs, Intel/AMD Linux hosts, and arm64 Linux hosts.
Use native executables outside Rosetta; Python, Node, Rust, and Go must use the same
host architecture. The full suite has been built and verified on macOS arm64;
existing Linux x86-64 harness evidence is retained separately. New platform
runs still need their own build and output verification before timing.

Before freezing a campaign, review [candidate coverage](candidate-coverage.md).
The executable 22-row matrix includes the seven direct additions identified in
that review. Further subset, runtime, and streaming candidates remain deferred. Before allocating runner time after dependency
changes, follow the [dependency readiness checklist](dependency-readiness.md).
It identifies current pins, release sources, local checks, and the latest
completed preflight.

## First setup

Install Git, curl, clang/clang++, CMake, Python 3.11 or newer, Node.js **24**, npm, Go **1.27.1**,
and Rust through rustup. On macOS, install Xcode Command Line Tools with
`xcode-select --install`. On Linux, install your distribution's compiler and
linker development packages too. Prevent sleep/suspend during Linux runs;
macOS runs use `caffeinate` automatically.

The script installs the pinned Bun Rust nightly through rustup; Cargo installs
the stable toolchain from `rust-toolchain.toml`. First preparation needs network
access to GitHub and the Cargo/npm registries. Dependency versions and source
archive hashes stay pinned. Current native sources come from
`benchmarks/native-comparison/prepare.py` and `restore.py`, not from an old
measurement report. Node competitors and extensions use their private npm
manifest and lockfile; additional Rust adapters use their separate Cargo lock.
Refreshing these inputs does not change historical reports or homepage values;
those still identify the versions actually measured. Node's local addon name follows the OS, architecture,
and Linux libc. Intel Macs use a private benchmark loader with a byte-for-byte
copy of the public JS facade, because no darwin-x64 sidecar is published. This
changes untimed addon selection only; it does not alter published packages.

Clone the repository with its history; the retained six-engine harness also
exports a historical Ferromark v1 commit. For an existing shallow clone, run
`git fetch --unshallow`. Check out the branch to measure and commit its changes.
The script rejects a dirty checkout and never cleans or resets it.

```sh
git switch your-benchmark-branch
./scripts/benchmark-comparison doctor
./scripts/benchmark-comparison run "$HOME/ferromark-runs/run-1"
```

Choose a **new directory outside the checkout**. Paths with spaces are supported.
`FERROMARK_BENCH_PYTHON=/path/to/python3.13` overrides interpreter discovery.
Use `./scripts/benchmark-comparison --help` for the command interface.

## What happens

`run` creates an isolated clone at `OUTPUT/source`, downloads the pinned sources,
checks the harnesses, builds both native harnesses and the plain Node addon,
and audits the exported native sources. Build concurrency is capped at four
jobs to keep memory use reasonable. Your checkout and its addon are untouched.

All HTML/option guards finish before the first timed comparison. The script
waits 60 seconds between lanes and runs one lane at a time. Each uses the
existing **three process rounds, six samples, 40 ms windows, and 60 ms warmup**.
All 57 frozen inputs are timed, including diagnostic disagreements; only
equivalent HTML contributes to the factors. There is no shortened publishable
mode. Logs are under `OUTPUT/logs`; commands and current phases are printed in
the terminal. Allow a long uninterrupted session; duration depends on the host
and the competitor.

Close heavy applications and let OS updates/indexing finish. The script does
not disable system services or claim exclusive CPU access. Host/load/thermal
observations and per-round ranges are retained so unusual noise can be reviewed.
If a result looks unstable, use a fresh output directory and compare full runs;
do not select only the fastest run. `--cooldown-seconds N` changes the pause
(default 60; range 0–600), not the measurement windows.

For separate preparation and measurement:

```sh
./scripts/benchmark-comparison prepare "$HOME/ferromark-runs/run-1"
./scripts/benchmark-comparison measure "$HOME/ferromark-runs/run-1"
```

To check every adapter without timing, run `./scripts/benchmark-comparison verify OUTPUT`
after preparation. Verification-only output cannot be published.

After an interrupted measurement, run `measure` again on the same host and
revision. Fully retained pairs are revalidated and skipped. Incomplete attempts
are renamed with an `-interrupted-` suffix and remeasured. A failed preparation
needs a **new output directory**; its partial files and logs remain for diagnosis.
Do not move a prepared directory before measuring: build metadata contains
absolute binary paths. The final `evidence/` directory is portable for publication.

## Update the measured platform

```sh
./scripts/benchmark-comparison publish "$HOME/ferromark-runs/run-1" --check
./scripts/benchmark-comparison publish "$HOME/ferromark-runs/run-1"
```

The default immutable report name includes the measurement date, detected
platform and source revision. `--name 2026-10-01-linux-x86-64` selects a custom
new report name; the measured platform still comes from verified provenance.
You can copy `evidence/` to another checkout and publish with its parent directory,
even on a different OS or architecture. The publishing checkout must contain
the measured commit in Git history. No build binaries, caches, or Node
installation are needed for this step; Python 3.11+ and Git suffice.

Publication rechecks complete window coverage, checksums, frozen inputs, HTML
classification, aggregates, committed sources/adapters, host/build platform,
runtime and addon identity, and all 22 comparisons. It rejects partial/shortened
runs and nonpositive or nonfinite scores. It retains compressed raw evidence
and provenance under a **new** `docs/reports/<name>/` with SHA256SUMS, updates only
that platform's selection in `benchmarks/manual-comparison/current.json`, and
regenerates homepage values and the benchmark guide. A newly measured platform
adds its own table column, such as macOS x86-64. Existing platforms and reports
are preserved. No Git commit, push, PR, or deployment happens automatically.

The original native harness uses shared mimalloc and a pinned Bun toolchain;
its four displayed libraries retain the historical five/six-engine matching
sets. It still measures v1 internally for those sets but does not publish a v1
homepage row. Other Rust/C pairs use system malloc; Goldmark uses the Go allocator and GC
with GOMAXPROCS=1, GOGC=100, and no memory limit; Node uses the release-node
addon without PGO, including string conversion and GC. cmark/commonmark.js/Remarkable use
CommonMark only in both engines. Different agreement sets and build contracts
remain disclosed; the table does not establish a shared-set ranking.

Review the generated report, agreement exclusions, per-round ranges, source
revision, host observations, and Git diff. Validate generated content:

```sh
python3 benchmarks/markdown-ecosystem/publish_values.py --check
python3 benchmarks/markdown-ecosystem/publish.py --check
```

Then run the [repository checks](../../CONTRIBUTING.md) and homepage checks before
opening a PR. New ratios remain observations for the measured machine and
corpus, not universal performance or statistical significance claims.

## Delegate to an agent

The repository includes
[manual-benchmarks](../../.agents/skills/manual-benchmarks/SKILL.md).
Ask an agent with access to the benchmark host and checkout:

> Use $manual-benchmarks to measure the current committed branch on this host,
> retain the evidence, and prepare a PR updating this platform's homepage values.

An agent that does not discover repository skills can read that SKILL.md directly.
Measuring through a remote terminal is also fine; keep that session alive. The
skill uses the same commands and evidence checks as a manual run.

## Blacksmith managed runners

The same commands can run on Blacksmith in **sebastian-software/ferromark**.
The GitHub App must have access to this repository. Blacksmith provisions an
on-demand runner for each job; no persistent instance or machine definition is
required. The manually dispatched [workflow](../../.github/workflows/blacksmith-benchmarks.yml)
provides these initial profiles:

| Profile | Runner label | Resources |
| --- | --- | --- |
| macOS arm64 | `blacksmith-6vcpu-macos-26` | Apple M4 Pro VM, 6 vCPU, 24 GiB RAM |
| Linux x86-64 | `blacksmith-4vcpu-ubuntu-2404` | Native x64, 4 vCPU, 16 GiB RAM |

The resource gate records the actual CPU model, process architecture, CPU count,
CPU affinity, usable memory, OS/kernel, runner image identifiers and Actions URL.
It rejects unexpected CPU counts (including automatic upgrades), memory sizes,
OS versions, architecture or repositories before setup. Linux kernel memory
reservations allow up to 5% less usable RAM; this is not permission to substitute
a smaller runner. An OS label pins the major image family, not its patch level;
actual versions are retained. Linux CPU models must also be compared between
independent jobs when reviewing results. Virtualization and shared host activity
can still affect measurements; fixed VM resources do not establish exclusive
hardware or stable physical clocks.

Choose the source branch with the workflow's branch selector or `--ref`. The
checkout itself is the measured revision. Start with `mode=probe` (the default):
it validates resources and runs `doctor`, without building the comparison or
timing anything. `mode=verify` prepares every adapter and checks all output and
option guards without timing. `mode=measure` runs all 22 comparisons and checks
complete evidence, but never publishes values, commits, or pushes results.

```sh
gh workflow run blacksmith-benchmarks.yml --repo sebastian-software/ferromark \
  --ref YOUR_BRANCH -f platform=both -f mode=probe -f repetitions=1
```

Before the new workflow reaches the default branch, use the already registered
native workflow as an entry point. Its `revision` and `pgo` inputs apply only to
the original GitHub-hosted lane; Blacksmith measures the selected source branch.

```sh
gh workflow run native-comparison.yml --repo sebastian-software/ferromark \
  --ref codex/manual-macos-benchmarks -f runner_provider=blacksmith \
  -f blacksmith_platform=both -f blacksmith_mode=probe -f blacksmith_repetitions=1
```

After both profiles pass output verification, use `mode=measure` with
`repetitions=3` to assess independent allocations. Jobs run sequentially and
retain each trial separately; do not publish the fastest result or average
incompatible agreement sets. Probe jobs have a 10-minute timeout, verification
jobs 90 minutes, and complete measurement jobs 180 minutes per allocation.
The 22-row suite needs more than 90 minutes for timing windows, warmup, and
default cooldowns alone; builds, output checks, and process starts add overhead.
These limits cap execution rather than prescribe its duration. Runner time,
including setup and cooldown, consumes the provider's billed/free minutes.

Artifacts retain observations, preparation/verification logs, and portable
`evidence/` for complete measurements for 90 days. They omit build binaries and
source/build caches. Download a selected measurement artifact, review all trials,
and point `publish OUTPUT --check` at its `suite/` directory. Then use the normal
local publication/review flow. Source revision and checksum validation still
apply. The suite retains managed-runner metadata and provenance; it never labels
a Blacksmith VM as a local physical Mac or GitHub-hosted hardware.

See Blacksmith's [runner profiles and free-minute conversions](https://docs.blacksmith.sh/blacksmith-runners/overview)
and [GitHub App setup](https://docs.blacksmith.sh/introduction/quickstart).

## Expanded adapter contracts

[comparisons.json](../markdown-ecosystem/comparisons.json) is the required campaign
inventory. Preparation and each pair retain its hash; schema-3 publication verifies
the committed inventory and rejects missing lanes. Historical schema-2 evidence
continues to describe its original 14 rows and cannot fill the new entries.

The new public API contracts are documented in the [ecosystem runbook](../markdown-ecosystem/README.md#expanded-public-apis).
MD4X's fixed extensions, Sätteri's GFM autolinks, and OX-Content's renderer builtins
remain in output verification; disagreement never receives an inferred factor.
Every new pair verifies all inputs in a rotating batch and retains separate
rotating-document timing controls. Single-document factors describe repeated
calls and may include library caching; the controls do not enter those factors.
Review both before making a parsing-throughput claim. Diagnostic short runs and
verification-only output still fail publication.
