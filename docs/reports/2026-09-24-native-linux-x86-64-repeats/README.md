# Native comparison on Linux x86-64: repeat runs — 2026-09-24

The [native comparison workflow](../../../.github/workflows/native-comparison.yml)
measured Ferromark v2 `09e5e866` (release 2.1.2) with PGO three times on
GitHub-hosted `ubuntu-latest` x86-64 runners on 2026-09-24. All three used the
same harness files, v2 revision, five comparison-engine pins, flags and 57
documents. Each run produced 342 of 342 HTML outputs byte-identical to
[`2026-09-21-native-round-4`](../2026-09-21-native-round-4/README.md), so each has
the same 14-document all-six and 50-document five-engine agreement sets.
Hypervisor steal was 0.00% during every timed run.

| Run | Started (UTC) | Runner CPU | Runner image | Role |
| --- | --- | --- | --- | --- |
| [36011572163](https://github.com/sebastian-software/ferromark/actions/runs/36011572163) | 14:24 | AMD EPYC 7763 | ubuntu24 20260920.314.1 | **Primary**, archived unchanged as [`2026-09-24-native-linux-x86-64`](../2026-09-24-native-linux-x86-64/README.md) |
| [36007978254](https://github.com/sebastian-software/ferromark/actions/runs/36007978254) | 13:52 | AMD EPYC 7763 | ubuntu24 20260920.314.1 | Repeat, reduced here |
| [36004415380](https://github.com/sebastian-software/ferromark/actions/runs/36004415380) | 13:22 | AMD EPYC 9V74 | ubuntu24 20260907.300.1 | Repeat, reduced here |

## Why one run is primary

The website and README publish one Linux report so that every Linux figure comes
from a single run on a single host. The primary is the latest run on the runner
CPU two of the three runs landed on, the AMD EPYC 7763. A GitHub-hosted runner
picks its CPU per job, so another run may land on different hardware; the
repeats show how much that moves the figures.

## Headline rows

V2 throughput relative to each engine, higher is faster: the five-engine
agreement set (50 documents) for v1, pulldown-cmark, md4c, and Bun, and the
all-six set (14 documents) for OX. PGO speedup is default-build time over
PGO-build time on the 28 held-out documents.

| Headline row | 36011572163, EPYC 7763 (primary) | 36007978254, EPYC 7763 | 36004415380, EPYC 9V74 |
| --- | ---: | ---: | ---: |
| Ferromark v1, fresh | 2.09× | 2.11× | 2.01× |
| Ferromark v1, reuse | 2.00× | 2.00× | 1.91× |
| pulldown-cmark, fresh | 2.48× | 2.48× | 2.44× |
| pulldown-cmark, reuse | 2.58× | 2.57× | 2.51× |
| md4c, fresh | 3.24× | 3.25× | 3.22× |
| md4c, reuse | 3.41× | 3.40× | 3.34× |
| Bun native bun_md, fresh | 4.68× | 4.70× | 4.52× |
| Bun native bun_md, reuse | 5.12× | 5.12× | 4.88× |
| OX-Content original, fresh | 1.39× | 1.41× | 1.43× |
| OX-Content original, reuse | 1.36× | 1.37× | 1.32× |
| PGO speedup of v2, fresh | 1.281× | 1.295× | 1.235× |
| PGO speedup of v2, reuse | 1.265× | 1.275× | 1.222× |
| PGO speedup of OX, fresh | 1.231× | 1.209× | 1.181× |
| PGO speedup of OX, reuse | 1.247× | 1.220× | 1.189× |

- **Same CPU model.** The EPYC 7763 repeat agrees with the primary within 0.03
  on every row. The largest difference is 0.026, on OX's PGO speedup with
  reuse (2.1%).
- **Different CPU model.** The EPYC 9V74 run is lower than the primary on 13 of
  the 14 rows. The largest differences are 0.24 on Bun with reuse (4.7%) and
  0.09 on v1. Every engine keeps its position, and v2 leads every engine in
  both lifecycles on both CPUs.

The published Linux figures therefore describe the EPYC 7763. The harness
README's
[notes on CI reports](../../../benchmarks/native-comparison/README.md#what-a-ci-report-can-and-cannot-claim)
explain why a run on another runner CPU is an independent measurement, not a
replication.

## Files

Each `run-<id>/` folder keeps four files from that run's
`native-report-linux-x86-64` artifact, unchanged: `TABLES.md` (all aggregate
tables and per-document timings, with each document's agreement-set
membership), `summary.json` (per-document medians), `pgo-comparison.json`
(held-out PGO speedups and standing), and `host.txt` (the runner description).
The rows above are recomputed from `summary.json` over the agreement sets listed
in `TABLES.md`, and from `pgo-comparison.json`. The complete artifacts, including
raw windows, HTML outputs and build logs, remain on the workflow runs until
GitHub expires them. `SHA256SUMS` covers every file in this directory.
