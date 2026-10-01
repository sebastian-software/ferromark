# Manual comparison

## Manual comparison — macOS arm64

All 20 selected comparisons were remeasured on one host from the same committed
source revision. Factors mean Ferromark throughput relative to the library.
Native and Node.js calls have separate build and allocation contracts.

| Runtime | Project | Timed documents | Fresh | Reuse |
| --- | --- | ---: | ---: | ---: |
| Native | pulldown-cmark | 57/57 | 2.72× | 2.77× |
| Native | md4c* | 57/57 | 3.50× | 3.64× |
| Native | Bun MD* | 57/57 | 6.05× | 6.50× |
| Native | OX-Content* | 57/57 | 1.34× | 1.30× |
| Native | markdown-rs | 57/57 | 72.58× | 87.60× |
| Native | Comrak | 57/57 | 7.14× | 8.45× |
| Native | cmark | 57/57 | 6.66× | 8.00× |
| Native | cmark-gfm | 57/57 | 7.10× | 8.42× |
| Native | Goldmark* | 57/57 | 9.75× | 11.73× |
| Node.js | marked* | 57/57 | 9.49× | 14.03× |
| Node.js | markdown-it* | 57/57 | 10.46× | 15.95× |
| Node.js | remark / unified* | 57/57 | 110.57× | 172.85× |
| Node.js | micromark* | 57/57 | 97.87× | 146.97× |
| Node.js | Showdown* | 57/57 | 43.32× | 65.32× |
| Node.js | commonmark.js | 57/57 | 4.51× | 6.87× |
| Node.js | Remarkable* | 57/57 | 3.96× | 5.93× |
| Node.js | Sätteri* | 57/57 | 1.83× | 2.79× |
| Node.js | MD4X* | 57/57 | 1.51× | 2.24× |
| Node.js | OX-Content* | 57/57 | 1.35× | 2.03× |
| Node.js | TanStack Markdown* | 57/57 | 4.82× | 7.22× |

Machine: Apple M4 Pro (Virtual), 24 GB RAM, macOS 26.3. Measurement dates: 2026-10-01, source `9af95fc3`.

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

[Raw evidence](https://github.com/sebastian-software/ferromark/tree/main/docs/reports/2026-10-01-blacksmith-macos-arm64), [manual workflow](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/manual-comparison/README.md).

[SHA256SUMS](SHA256SUMS) covers all retained evidence.
