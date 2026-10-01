# Native comparison on Linux x86-64 — 2026-10-01

Ferromark **v2 `9af95fc317`** and the five pinned comparison engines (v1 `4e15141`, OX-Content, md4c, pulldown-cmark, and Bun's native `bun_md`) were measured on the frozen **57 documents (37–113,609 UTF-8 bytes)** on one host: **x86_64**, AMD EPYC, 15.4 GB RAM, Linux 6.6.141. Each scored column uses the same input set and equivalent HTML for every included engine. **Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, 14 agreeing across six | Reuse, same 14 | Fresh, 50 agreeing across five | Reuse, same 50 |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.73× | 0.80× | — | — |
| Ferromark v1 | 0.59× | 0.68× | 0.49× | 0.53× |
| md4c | 0.25× | 0.22× | 0.34× | 0.33× |
| pulldown-cmark | 0.42× | 0.39× | 0.43× | 0.41× |
| Bun native bun_md | 0.20× | 0.17× | 0.21× | 0.19× |

The harness, syntax and renderer flags, sources, adapters, and allocator setup are the ones the Apple Silicon reports use; the few platform differences are listed in [PROVENANCE.md](PROVENANCE.md#platform-differences). OX has no score in the five-engine columns because its original renderer cannot disable heading IDs, callouts, or fence metadata cleanup.

## What this run can and cannot claim

- **One host.** The [Blacksmith benchmark run](https://github.com/sebastian-software/ferromark/actions/runs/36822651743) used an isolated committed checkout. All six engines ran in the same process rounds and rotating windows on that host, one worker at a time, so the ratios between engines describe that host's CPU.
- **A managed runner.** The run used a provider-managed virtual machine. The recorded CPU, memory, OS image, and run URL identify this allocation; exclusive physical hardware is not claimed. Compare engine ratios within the run and inspect variation across independent runs.
- **Hypervisor steal** was 0.01% of all CPU time during the timed run (from `/proc/stat`, recorded before and after every process round in `run.json`).
- **Not a universal ranking** and not a statistical significance claim. Positions on x86-64 and Apple Silicon can differ because the engines' SIMD paths, the compilers' code generation, and the cache hierarchies differ; neither platform's figures describe the other.

## Validation

Direct comparison in each process round, using the same agreement sets:

| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |
| --- | ---: | --- | --- |
| OX-Content original | 14 | 1.409×, 1.371×, 1.337× | 1.246×, 1.273×, 1.238× |
| Ferromark v1 | 50 | 1.991×, 2.021×, 2.007× | 1.927×, 1.903×, 1.914× |

These are round aggregates, not confidence intervals. The largest spread between the 3 rounds on any row above is 0.072.

All **6,372 timed windows** passed their output-length checksums and were rechecked from `samples.json.gz` when this directory was assembled. Fresh/reuse equality, repeated transitions, and exact pre/post-timing output checks passed for every engine. Each of the 59 workloads (57 documents and 2 rotating batches) was measured in both lifecycles, with 3 process rounds, 3 windows of at least 10 ms per round, and 30 ms per-engine warmup. The timed run took 2m21s.

**342 of 342 HTML outputs** (57 documents × 6 engines) are byte-identical to the Apple Silicon report [`2026-09-21-native-round-4`](../2026-09-21-native-round-4/README.md), and 57 of 57 agreement classifications are unchanged; details in [checks/commands.json](checks/commands.json).

Harness unit tests: 52 tests passed. Registry resolution: all 71 registry packages are identical to the seed lock in name, version, and checksum.

## Files

- [Full tables](TABLES.md), [per-document timings](timings.csv), [aggregates](aggregates.json), [summary](summary.json), [raw windows](samples.json.gz), [run metadata](run.json).
- [Frozen inputs and attribution](corpus.json.gz), [all HTML](verification.json.gz), [executable option guards](behavior.json.gz).
- [Build metadata](build.json), [dependency lock](Cargo.lock), [build log](build-log.txt), [host description](host.txt), [source audit](source-audit.json.gz), [checks](checks/commands.json), [restore script](restore.py) and [its record](restore.json).
- The harness that produced this run is preserved under [`harness/`](harness/); regenerate the tables with `python3 harness/report.py .`.

Engine sources and corpus inputs retain their original licenses and attribution. This is Bun's native Markdown core, not its JavaScript API.
