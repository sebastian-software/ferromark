# HTML block parser measurements

Generated from the archived measurement records. Negative changes mean less elapsed time. Exploratory screens and independent production confirmations are reported separately.

## Profiles

Two six-second native sample captures per document/implementation at one-millisecond intervals. Visible stack shares are diagnostic, not elapsed-time improvements.

| Source | Input | Visible block frames |
| --- | --- | --- |
| baseline | corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 80.5%, 81.6% |
| baseline | corpus/vue-docs/src/guide/extras/render-function.md | 41.2%, 41.6% |
| production | corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 73.4%, 73.3% |
| production | corpus/vue-docs/src/guide/extras/render-function.md | 41.9%, 41.4% |

## Exploratory screens

Five alternating 30 ms windows per implementation/input, following 60 ms warmup. The median is over 81 selected inputs and is not a weighted application workload. All original and block/MDX output guards must pass before timing. Complete per-input results remain in each variant directory.

| Variant | Median change | TS compiler options | Root short lines | Long comment lines | Guard failures |
| --- | --- | --- | --- | --- | --- |
| iter | -0.0% | -19.8% | -47.7% | -0.3% | 0 |
| iter+markers | +0.1% | -19.3% | -47.0% | -84.7% | 0 |
| iter+prefix+markers | +0.3% | -19.7% | -47.1% | -84.2% | 0 |
| markers | +0.3% | +3.0% | +0.3% | -84.4% | 0 |
| markers-lite | +0.1% | -0.7% | +0.3% | -87.8% | 0 |
| prefix | -0.2% | -2.1% | +0.3% | -0.3% | 0 |
| run | +0.0% | -13.1% | -21.6% | -0.0% | 0 |
| scan | +0.0% | -22.9% | -50.7% | +0.2% | 0 |
| scan+markers | -0.3% | -22.4% | -51.0% | -84.2% | 0 |
| scan+markers-lite+tags | -0.2% | -30.3% | -51.1% | -87.9% | 0 |
| scan+markers-lite+tags+entry | -0.3% | -31.1% | -50.8% | -87.9% | 0 |
| tags | -0.0% | -6.9% | +0.5% | +0.1% | 0 |

## Dispatch correction

The first production integration entered the continuation loop from general block dispatch. Three full process pairs exposed a repeated regression on many short escaped code fences. The final integration enters from HTML-block recognition instead. A targeted three-pair confirmation preserves the HTML gain while removing this regression; the complete final confirmation follows below.

| Input | General-dispatch integration | HTML-start integration |
| --- | --- | --- |
| escape-dense | +3.4%, +3.7%, +3.2% | -0.4%, +0.0%, -0.0% |
| ox-huge-tables | +1.2%, +0.2%, +0.5% | +0.6%, -0.4%, +1.0% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | -18.8%, -19.0%, -19.1% | -18.1%, -17.1%, -18.5% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | -30.6%, -31.1%, -30.7% | -30.6%, -30.5%, -30.3% |

## Production confirmation

The formatted production source is rebuilt separately. Three fresh process pairs, nine alternating 75 ms windows per implementation/input, following 60 ms warmup. Latencies summarize the three process medians; all paired changes remain visible.

| Input | Baseline µs | Production µs | Three paired changes |
| --- | --- | --- | --- |
| ox-large-cm | 242.668 | 241.940 | -0.3%, -0.6%, -0.3% |
| ox-large-tables | 253.987 | 253.300 | -0.1%, +0.1%, -0.3% |
| ox-huge-tables | 6179.479 | 6136.773 | -1.0%, -1.1%, -0.4% |
| short | 0.484 | 0.480 | -0.7%, +2.8%, -1.3% |
| short-reuse | 0.195 | 0.197 | +0.1%, +1.8%, +1.4% |
| plain | 6.797 | 6.802 | +0.2%, +0.1%, -0.4% |
| multiline | 13.333 | 13.401 | +0.7%, +0.5%, +0.1% |
| inline | 39.584 | 39.540 | +0.1%, -0.2%, -0.1% |
| lists | 45.005 | 45.054 | -0.1%, +0.1%, +0.5% |
| code | 16.943 | 16.850 | -0.6%, -0.2%, -0.4% |
| tables | 41.843 | 41.823 | -0.1%, +0.1%, +0.3% |
| headings | 37.593 | 37.592 | -0.1%, -0.0%, +0.0% |
| unique-headings | 33.757 | 33.879 | +0.2%, +0.6%, -0.2% |
| long-prose | 9.962 | 9.964 | +0.0%, -0.0%, +0.2% |
| late-marker | 12.670 | 12.649 | -0.2%, -0.0%, -0.3% |
| long-code | 20.383 | 20.316 | -0.8%, -0.0%, +0.4% |
| escape-dense | 12.939 | 12.854 | -0.7%, -0.2%, -0.1% |
| commonmark/tiny | 0.462 | 0.466 | +0.5%, +2.8%, +1.0% |
| commonmark/short-100b | 0.854 | 0.855 | -1.6%, +0.9%, +0.6% |
| gfm_overlap/tiny | 0.457 | 0.467 | +0.4%, +2.9%, +0.7% |
| gfm_overlap/short-100b | 0.847 | 0.850 | -2.0%, +0.8%, +0.2% |
| tables/tables-plain | 25.305 | 25.428 | +0.2%, +0.7%, +0.9% |
| tables/tables-commonmark-inline | 31.946 | 32.028 | +0.3%, +0.2%, +0.1% |
| tables/tables-links | 37.336 | 37.305 | +0.2%, -0.1%, +0.6% |
| commonmark/publication-5k | 24.282 | 23.994 | -1.2%, -1.2%, -0.7% |
| guard/plain | 25.317 | 25.538 | +0.1%, +0.9%, +0.8% |
| guard/mixed | 32.257 | 32.251 | +0.3%, -0.0%, +0.4% |
| guard/links | 37.343 | 37.578 | -0.2%, +0.9%, +0.6% |
| guard/mixed-document | 17.795 | 17.700 | -1.4%, -0.3%, -0.5% |
| guard/one-table | 17.349 | 17.400 | -0.1%, +0.8%, +0.3% |
| guard/long-cells | 25.067 | 25.024 | +0.2%, -0.2%, -0.6% |
| guard/wide9 | 37.955 | 38.106 | +0.4%, +0.4%, +0.5% |
| guard/wide16 | 58.506 | 58.584 | +0.1%, +0.8%, -0.0% |
| guard/escapes-code | 67.863 | 68.233 | -0.3%, -0.6%, +0.7% |
| guard/prose | 9.863 | 9.862 | -0.0%, +0.1%, -0.2% |
| guard/links-prose | 36.467 | 36.019 | -1.3%, -1.3%, -1.0% |
| guard/edge-cases | 1331.812 | 1334.413 | +0.2%, +0.2%, +0.2% |
| guard/references | 10.345 | 10.375 | +0.2%, +0.3%, +0.4% |
| guard/empty | 0.140 | 0.142 | -1.8%, +1.9%, +0.3% |
| guard/gfm-document | 24.247 | 24.024 | -1.0%, -0.2%, -1.0% |
| guard/long-delimiters | 13.680 | 13.678 | +0.0%, +0.3%, -0.0% |
| guard/deep-emphasis | 38.465 | 38.441 | -0.3%, +0.2%, +0.0% |
| mdx/short | 1.168 | 1.179 | -0.5%, +0.9%, +1.0% |
| mdx/segments | 164.976 | 161.879 | -1.6%, -1.5%, -1.9% |
| mdx/tables-code | 147.192 | 146.058 | -0.8%, -0.7%, -0.9% |
| corpus/vue-docs/src/api/application.md | 54.080 | 54.171 | +0.2%, -0.2%, +0.4% |
| corpus/vue-docs/src/api/built-in-directives.md | 57.758 | 57.767 | -0.7%, +0.3%, +0.3% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 17.103 | 17.049 | -0.5%, -0.2%, -0.1% |
| corpus/vue-docs/src/guide/extras/render-function.md | 44.525 | 44.227 | -1.5%, -1.3%, -0.4% |
| corpus/vite-docs/docs/config/shared-options.md | 74.222 | 73.843 | +0.7%, -0.1%, -1.4% |
| corpus/vite-docs/docs/guide/api-plugin.md | 73.966 | 74.619 | +1.4%, -0.9%, +0.9% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 102.760 | 100.955 | -1.6%, -3.4%, -1.8% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 64.550 | 64.231 | -0.4%, -2.1%, -0.9% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 76.446 | 75.574 | -1.2%, +0.4%, -2.0% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 22.699 | 18.497 | -18.4%, -18.8%, -18.5% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 47.883 | 33.281 | -30.5%, -30.6%, -30.4% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 105.856 | 105.963 | +0.2%, -0.5%, -0.4% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 83.210 | 82.580 | -1.0%, -0.8%, +1.2% |
| corpus/concat/vue-docs/matched | 2752.441 | 2734.991 | -0.6%, -0.6%, -0.8% |
| corpus/concat/vue-docs/upstream | 3080.888 | 3042.311 | -0.8%, -1.5%, -1.0% |
| corpus/concat/vite-docs/matched | 1460.608 | 1458.913 | +0.0%, +0.1%, -0.1% |
| corpus/concat/vite-docs/upstream | 1754.023 | 1754.057 | +0.2%, -0.8%, -0.3% |
| corpus/concat/rust-book/matched | 4259.439 | 4222.339 | -0.9%, -0.9%, -0.9% |
| corpus/concat/rust-book/upstream | 4659.422 | 4630.208 | -0.6%, -0.9%, -0.6% |
| corpus/concat/typescript-handbook/matched | 5472.344 | 5439.203 | -0.6%, -0.6%, -0.5% |
| corpus/concat/typescript-handbook/upstream | 6110.727 | 6072.177 | -0.7%, -0.3%, -0.5% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.593 | 1.604 | -0.6%, +2.6%, +0.5% |
| corpus/ox-parser/LARGE_MD/1 | 8.352 | 8.307 | -0.8%, +0.3%, -0.5% |
| corpus/ox-parser/LARGE_MD/100 | 763.826 | 763.140 | -0.3%, -0.1%, -0.6% |
| html/root-short-lines | 90.406 | 44.444 | -50.9%, -50.9%, -50.8% |
| html/root-long-lines | 4.575 | 4.534 | -0.5%, -0.9%, -1.2% |
| html/separate-blocks | 206.819 | 166.938 | -18.8%, -19.5%, -19.7% |
| html/comment-long-lines | 44.222 | 5.368 | -87.9%, -87.9%, -87.7% |
| html/script-long-lines | 142.895 | 5.359 | -96.2%, -96.2%, -96.3% |
| html/script-candidates | 165.449 | 59.813 | -63.9%, -63.9%, -63.8% |
| html/comment-short-lines | 119.276 | 91.459 | -22.9%, -23.3%, -23.4% |
| html/processing-long-lines | 33.005 | 5.366 | -83.7%, -83.8%, -83.8% |
| html/cdata-long-lines | 44.005 | 5.361 | -87.8%, -87.8%, -87.8% |
| html/declaration-long-lines | 33.016 | 5.311 | -83.9%, -83.9%, -83.9% |
| html/container-html | 131.808 | 132.347 | +0.3%, +0.5%, +0.4% |
| html/custom-tags | 279.036 | 207.049 | -26.3%, -25.7%, -25.8% |

Inputs exceeding a 2% regression in all three process pairs: none.

## Requested live heap

Three fresh processes per implementation, ten observations per process/input. This measures allocator requests during rendering with owned output still live, not RSS. The dataset includes 64 KiB and 8 MiB HTML inputs.

| Input | Input bytes | Baseline peak bytes | Production peak bytes | Baseline allocations | Production allocations |
| --- | --- | --- | --- | --- | --- |
| ox-large-cm | 49898 | 356906 | 356906 | 39 | 39 |
| ox-large-tables | 49898 | 356906 | 356906 | 39 | 39 |
| ox-huge-tables | 1072848 | 7480842 | 7480842 | 44 | 44 |
| short | 17 | 3152 | 3152 | 10 | 10 |
| plain | 6300 | 15731 | 15731 | 4 | 4 |
| multiline | 7500 | 26203 | 26203 | 9 | 9 |
| inline | 8100 | 30974 | 30974 | 19 | 19 |
| lists | 5300 | 57714 | 57714 | 22 | 22 |
| code | 6200 | 23228 | 23228 | 4 | 4 |
| tables | 7500 | 80222 | 80222 | 8 | 8 |
| headings | 4200 | 69264 | 69264 | 12 | 12 |
| unique-headings | 4388 | 70972 | 70972 | 12 | 12 |
| long-prose | 65536 | 344096 | 344096 | 5 | 5 |
| late-marker | 65540 | 393653 | 393653 | 11 | 11 |
| long-code | 70712 | 160630 | 160630 | 3 | 3 |
| escape-dense | 2700 | 20412 | 20412 | 6 | 6 |
| commonmark/tiny | 18 | 3152 | 3152 | 10 | 10 |
| commonmark/short-100b | 103 | 3816 | 3816 | 18 | 18 |
| gfm_overlap/tiny | 18 | 3152 | 3152 | 10 | 10 |
| gfm_overlap/short-100b | 103 | 3816 | 3816 | 18 | 18 |
| tables/tables-plain | 4560 | 49448 | 49448 | 8 | 8 |
| tables/tables-commonmark-inline | 4800 | 64528 | 64528 | 16 | 16 |
| tables/tables-links | 6360 | 69092 | 69092 | 21 | 21 |
| commonmark/publication-5k | 5120 | 29092 | 29092 | 45 | 45 |
| guard/plain | 4560 | 49448 | 49448 | 8 | 8 |
| guard/mixed | 4800 | 64528 | 64528 | 16 | 16 |
| guard/links | 6360 | 69092 | 69092 | 21 | 21 |
| guard/mixed-document | 5632 | 33628 | 33628 | 69 | 69 |
| guard/one-table | 3056 | 33656 | 33656 | 8 | 8 |
| guard/long-cells | 53152 | 121160 | 121160 | 4 | 4 |
| guard/wide9 | 6621 | 71000 | 71000 | 110 | 110 |
| guard/wide16 | 11612 | 123414 | 123414 | 110 | 110 |
| guard/escapes-code | 6860 | 33028 | 33028 | 220 | 220 |
| guard/prose | 7050 | 24460 | 24460 | 5 | 5 |
| guard/links-prose | 5800 | 22636 | 22636 | 17 | 17 |
| guard/edge-cases | 175556 | 647479 | 647479 | 1225 | 1225 |
| guard/references | 876 | 7563 | 7563 | 34 | 34 |
| guard/empty | 0 | 2560 | 2560 | 2 | 2 |
| guard/gfm-document | 5120 | 29092 | 29092 | 45 | 45 |
| guard/long-delimiters | 16426 | 38516 | 38516 | 4 | 4 |
| guard/deep-emphasis | 3073 | 132738 | 132738 | 23 | 23 |
| large/mixed-1mib | 1048898 | 7322854 | 7322854 | 44 | 44 |
| large/plain-1mib | 1048576 | 5505056 | 5505056 | 5 | 5 |
| large/mixed-8mib | 8388689 | 58547388 | 58547388 | 47 | 47 |
| large/plain-8mib | 8388608 | 44040224 | 44040224 | 5 | 5 |
| html/root-65536 | 65536 | 148992 | 148992 | 3 | 3 |
| html/comment-65536 | 65536 | 148992 | 148992 | 3 | 3 |
| html/script-65536 | 65536 | 214528 | 214528 | 4 | 4 |
| html/root-8388608 | 8388608 | 18875904 | 18875904 | 3 | 3 |
| html/comment-8388608 | 8388608 | 18875904 | 18875904 | 3 | 3 |
| html/script-8388608 | 8388608 | 27264512 | 27264512 | 4 | 4 |

## Fresh Ox corpus comparison

This separate native harness uses fresh state and owned HTML with heading IDs enabled. Three process groups, nine rotating 60 ms windows per engine/input, following 35 ms warmup. Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; the growing arena is primary. Only equal normalized output supports ranking. Do not subtract these latencies from the optimization harness above.

| Input | Ferromark µs | Ox growing µs | Ox presized µs | F / Ox growing |
| --- | --- | --- | --- | --- |
| vue-docs/src/api/application.md | 54.29 | 64.04 | 64.16 | 0.85× |
| vue-docs/src/api/built-in-directives.md | 57.76 | 67.93 | 67.04 | 0.85× |
| vue-docs/src/guide/built-ins/suspense.md | 17.06 | 10.47 | 10.51 | 1.63× |
| vue-docs/src/guide/extras/render-function.md | 44.30 | 31.44 | 31.30 | 1.41× |
| vite-docs/docs/config/shared-options.md | 74.12 | 65.37 | 65.73 | 1.13× |
| vite-docs/docs/guide/api-plugin.md | 74.96 | 74.91 | 75.07 | 1.00× |
| rust-book/src/ch02-00-guessing-game-tutorial.md | 103.05 | 70.38 | 69.47 | 1.46× |
| rust-book/src/ch09-02-recoverable-errors-with-result.md | 65.74 | 40.25 | 40.52 | 1.63× |
| rust-book/src/ch21-02-multithreaded.md | 77.99 | 52.01 | 51.73 | 1.50× |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 18.39 | 14.31 | 14.30 | 1.29× |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 33.09 | 26.56 | 26.36 | 1.25× |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 106.29 | 74.14 | 73.21 | 1.43× |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 83.42 | 59.62 | 59.45 | 1.40× |
| ox-parser/SIMPLE_MD/1 | 1.60 | 1.26 | 1.21 | 1.27× |
| ox-parser/LARGE_MD/1 | 8.33 | 7.12 | 7.08 | 1.17× |
| ox-parser/LARGE_MD/100 | 761.21 | 697.20 | 696.98 | 1.09× |

Normalized equality: 594 / 649 corpus cases. All unequal cases remain diagnostics.

## Exact-output checks

| Guard set | Cases | Failures |
| --- | --- | --- |
| Specification, extensions, reuse, all corpus files | 2695 | 0 |
| Initial inline / MDX events and HTML | 16374 | 0 |
| Independent inline / MDX events and HTML | 60000 | 0 |
| Public block / MDX streams, segments, HTML and metadata | 20740 | 0 |
| Every timed input | 81 | 0 |
