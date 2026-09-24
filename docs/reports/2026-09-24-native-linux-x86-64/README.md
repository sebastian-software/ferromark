# Native comparison on Linux x86-64 — 2026-09-24

Ferromark **v2 `09e5e866da`** and the five pinned comparison engines (v1 `4e15141`, the original OX-Content core, md4c, pulldown-cmark, and Bun's native `bun_md`) were measured on the frozen **57 documents (37–113,609 UTF-8 bytes)** on one host: **AMD EPYC 7763 64-Core Processor**, 4 vCPUs, Linux 6.17.0-1022-azure, runner image ubuntu24 20260920.314.1. Each scored column uses the same input set and equivalent HTML for every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.72× | 0.73× | — | — |
| Ferromark v1 | 0.59× | 0.62× | 0.48× | 0.50× |
| md4c | 0.24× | 0.20× | 0.31× | 0.29× |
| pulldown-cmark | 0.42× | 0.36× | 0.40× | 0.39× |
| Bun native bun_md | 0.20× | 0.16× | 0.21× | 0.20× |

The harness, syntax and renderer flags, sources, adapters, and allocator setup are the ones the Apple Silicon reports use; the few platform differences are listed in [PROVENANCE.md](PROVENANCE.md#platform-differences). OX has no score in the five-engine columns because its original renderer cannot disable heading IDs, callouts, or fence metadata cleanup.

## What this run can and cannot claim

- **One host.** The [native comparison workflow](https://github.com/sebastian-software/ferromark/actions/runs/36011572163) ran on a GitHub-hosted ubuntu-latest runner for a workflow_dispatch event. All six engines ran in the same process rounds and rotating windows on that host, one worker at a time, so the ratios between engines describe that host's CPU.
- **A shared machine.** A GitHub-hosted runner is a virtual machine on shared hardware. Neighbors, clock behavior, and the CPU model can change between runs, so absolute nanoseconds are not comparable with any other run, and another run may land on a different CPU. Compare engine ratios, not times.
- **Hypervisor steal** was 0.00% of all CPU time during the timed run (from `/proc/stat`, recorded before and after every process round in `run.json`).
- **Not a universal ranking** and not a statistical significance claim. Positions on x86-64 and Apple Silicon can differ because the engines' SIMD paths, the compilers' code generation, and the cache hierarchies differ; neither platform's figures describe the other.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 1.401×, 1.390×, 1.387× | 1.359×, 1.359×, 1.360× |
| Ferromark v1 | 50 | 2.092×, 2.100×, 2.089× | 1.988×, 1.996×, 2.001× |

These are round aggregates, not confidence intervals. The largest spread between the 3 rounds on any row above is 0.013.

All **12,744 timed windows** passed their output-length checksums and were rechecked from `samples.json.gz` when this directory was assembled. Fresh/reuse equality, repeated transitions, and exact pre/post-timing output checks passed for every engine. Each of the 59 workloads (57 documents and 2 rotating batches) was measured in both lifecycles, with 3 process rounds, 6 windows of at least 40 ms per round, and 60 ms per-engine warmup. The timed run took 11m11s.

**342 of 342 HTML outputs** (57 documents × 6 engines) are byte-identical to the Apple Silicon report [`2026-09-21-native-round-4`](../2026-09-21-native-round-4/README.md), and 57 of 57 agreement classifications are unchanged; details in [checks/commands.json](checks/commands.json).

Harness unit tests: 44 tests passed. Registry resolution: all 67 registry packages are identical to the seed lock in name, version, and checksum.

## Profile-guided optimization, measured on the held-out half

`prepare.py --pgo` applied one recipe to every Rust engine and trained on the documents the round-3 split reserves for training. Both executables were measured on the other **28 documents**, which no profile saw. Per-engine speedup is default-build time over PGO-build time, geometric mean over the held-out documents.

| Engine | Profile data | Fresh | Reuse | Fresh range |
| --- | --- | ---: | ---: | --- |
| Ferromark v2 | rust-pgo | 1.281× | 1.265× | 1.09–1.39× |
| OX-Content original | rust-pgo | 1.231× | 1.247× | 1.06–1.32× |
| Ferromark v1 | rust-pgo | 1.235× | 1.233× | 0.98–1.45× |
| md4c | none | 1.009× | 1.003× | 0.98–1.04× |
| pulldown-cmark | rust-pgo | 1.297× | 1.307× | 1.19–1.35× |
| Bun native bun_md | rust-pgo-partial | 1.043× | 1.044× | 0.87–1.32× |

A PGO row is compared only with PGO rows:

| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |
| --- | ---: | ---: | ---: | ---: | ---: |
| OX-Content original | 5 | 1.30× | 1.26× | 1.43× | 1.20× |
| Ferromark v1 | 24 | 2.07× | 1.98× | 2.10× | 1.98× |
| md4c | 24 | 3.05× | 3.22× | 3.84× | 4.02× |
| pulldown-cmark | 24 | 2.32× | 2.41× | 2.26× | 2.29× |
| Bun native bun_md | 24 | 4.23× | 4.65× | 5.12× | 5.53× |

md4c is C and is not PGO-built; for Bun's engine the recipe reaches only its Rust crates, not the C++ Highway and support objects. The held-out runs reproduce the same HTML from the default and the PGO executable.

## Files

- [Full tables](TABLES.md), [per-document timings](timings.csv), [aggregates](aggregates.json), [summary](summary.json), [raw windows](samples.json.gz), [run metadata](run.json).
- [Frozen inputs and attribution](corpus.json.gz), [all HTML](verification.json.gz), [executable option guards](behavior.json.gz).
- [Build metadata](build.json), [dependency lock](Cargo.lock), [build log](build-log.txt), [host description](host.txt), [source audit](source-audit.json.gz), [checks](checks/commands.json), [restore script](restore.py) and [its record](restore.json).
- Held-out PGO evidence: [PGO build metadata](build-pgo.json), [its build log](build-log-pgo.txt.gz), [comparison](pgo-comparison.json), [default run](run-pgo-default.json) and [`pgo/default/`](pgo/default/), [PGO run](run-pgo-pgo.json) and [`pgo/pgo/`](pgo/pgo/).
- The harness that produced this run is preserved under [`harness/`](harness/); regenerate the tables with `python3 harness/report.py .`.

Engine sources and corpus inputs retain their original licenses and attribution. This is Bun's native Markdown core, not its JavaScript API.
