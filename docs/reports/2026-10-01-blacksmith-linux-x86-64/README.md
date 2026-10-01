# Manual comparison

## Manual comparison — Linux x86-64

All 20 selected comparisons were remeasured on one host from the same committed
source revision. Factors mean Ferromark throughput relative to the library.
Native and Node.js calls have separate build and allocation contracts.

| Runtime | Project | Timed documents | Fresh | Reuse |
| --- | --- | ---: | ---: | ---: |
| Native | pulldown-cmark | 57/57 | 2.35× | 2.43× |
| Native | md4c* | 57/57 | 2.95× | 3.06× |
| Native | Bun MD* | 57/57 | 4.74× | 5.15× |
| Native | OX-Content* | 57/57 | 1.40× | 1.37× |
| Native | markdown-rs | 57/57 | 106.55× | 118.08× |
| Native | Comrak | 57/57 | 5.10× | 5.51× |
| Native | cmark | 57/57 | 5.70× | 6.25× |
| Native | cmark-gfm | 57/57 | 6.15× | 6.67× |
| Native | Goldmark* | 57/57 | 8.77× | 9.49× |
| Node.js | marked* | 57/57 | 9.74× | 14.46× |
| Node.js | markdown-it* | 57/57 | 10.49× | 15.63× |
| Node.js | remark / unified* | 57/57 | 132.49× | 192.30× |
| Node.js | micromark* | 57/57 | 108.26× | 165.69× |
| Node.js | Showdown* | 57/57 | 47.32× | 69.00× |
| Node.js | commonmark.js | 57/57 | 4.76× | 7.02× |
| Node.js | Remarkable* | 57/57 | 3.88× | 5.77× |
| Node.js | Sätteri* | 57/57 | 1.68× | 2.49× |
| Node.js | MD4X* | 57/57 | 1.33× | 1.96× |
| Node.js | OX-Content* | 57/57 | 1.27× | 1.86× |
| Node.js | TanStack Markdown* | 57/57 | 5.72× | 8.53× |

Machine: AMD EPYC, 15.4 GB RAM, Linux 6.6.141. Measurement dates: 2026-10-01, source `9af95fc3`.

*Output differs for some inputs. Every factor includes all 57 frozen documents;
HTML agreement is retained as descriptive metadata, not a scoring filter or a
conformance test. Public syntax, output, allocator, runtime and GC contracts
remain documented; these figures compare the public APIs on the same inputs,
not identical implemented features. Timing profile: **balanced**; 3 process rounds,
3 alternating 10 ms windows and 30 ms warmup.
Equal-document geometric means use per-engine medians of round medians.
Rotating-document cache controls remain separate. Native and Node binding costs
remain distinct. Inspect round spread before making a performance claim; the
balanced profile has less timing evidence than standard.

Builds and downloads finish before verification and timing. Lanes run
sequentially with a cooldown; host observations and per-round ranges remain
in the raw evidence. Background system activity can still add noise. These
results describe this machine and corpus, with no significance claim. Other platforms
and earlier reports remain separate.

[Raw evidence](https://github.com/sebastian-software/ferromark/tree/main/docs/reports/2026-10-01-blacksmith-linux-x86-64), [manual workflow](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/manual-comparison/README.md).

[SHA256SUMS](SHA256SUMS) covers all retained evidence.
