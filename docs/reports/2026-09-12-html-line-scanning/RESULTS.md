# HTML line scanning measurements

Generated from the archived measurement records. Negative changes mean less elapsed time. Exploratory screens and independent production confirmations are reported separately.

## Profiles

Two six-second native sample captures per document/implementation at one-millisecond intervals. Visible stack shares are diagnostic, not elapsed-time improvements.

| Source | Input | Visible block frames |
| --- | --- | --- |
| probe | corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 74.7%, 73.7% |
| probe | corpus/vue-docs/src/guide/extras/render-function.md | 41.7%, 42.1% |
| baseline | corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 73.4%, 74.0% |
| baseline | corpus/vue-docs/src/guide/extras/render-function.md | 40.9%, 42.4% |
| production | corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 71.0%, 70.9% |
| production | corpus/vue-docs/src/guide/extras/render-function.md | 41.3%, 40.6% |

## Exploratory screens

Five alternating 30 ms windows per implementation/input, following 60 ms warmup. The median is over 81 selected inputs and is not a weighted application workload. The initial state/indentation screens use original and block/MDX guards; scan screens use original and all timing-input guards. Production is checked against every guard set. Complete per-input results remain in each variant directory.

| Variant | Median change | TS compiler options | Root short lines | Long comment lines | Guard failures |
| --- | --- | --- | --- | --- | --- |
| bounded16 | +0.0% | -1.9% | -6.7% | +0.6% | 0 |
| bounded32 | -0.1% | -1.8% | -0.2% | -1.5% | 0 |
| defer-indent | -0.0% | -0.6% | +4.5% | +0.2% | 0 |
| local-indent | +0.1% | +3.5% | +1.4% | +0.5% | 0 |
| newline-all | +0.1% | +28.1% | +52.9% | +41.3% | 0 |
| newline-all+single-byte+const-set | -0.1% | -5.9% | -6.5% | +18.9% | 0 |
| newline128+single-byte+const-set | -0.1% | -6.7% | -7.0% | -2.3% | 0 |
| newline16 | +0.0% | +23.3% | +39.6% | +1.9% | 0 |
| newline16+single-byte+const-set | +0.1% | -2.1% | -8.1% | +0.1% | 0 |
| newline32 | +0.2% | +23.3% | +42.7% | -0.1% | 0 |
| newline32+const-set | +0.1% | +0.6% | +6.6% | -0.8% | 0 |
| newline32+single-byte+const-set | -0.1% | -2.9% | -7.7% | -3.0% | 0 |
| newline32+single-byte+const-set+stable-kind+prefix | -0.1% | -6.2% | -6.0% | -2.9% | 0 |
| newline64+single-byte+const-set | -0.2% | -4.9% | -6.3% | -1.0% | 0 |
| newline64+single-byte+const-set+prefix | -0.5% | -7.0% | -5.5% | -1.8% | 0 |
| prefix | -0.2% | -2.7% | +0.2% | -0.9% | 0 |
| specialize | -0.1% | -1.3% | +0.2% | -1.2% | 0 |
| stable-kind | -0.2% | +0.7% | -1.3% | +0.7% | 0 |
| word16 | -0.1% | -2.1% | -21.9% | +0.5% | 0 |

## Production confirmation

The formatted production source is rebuilt separately. Three fresh process pairs, nine alternating 75 ms windows per implementation/input, following 60 ms warmup. Latencies summarize the three process medians; all paired changes remain visible.

| Input | Baseline µs | Production µs | Three paired changes |
| --- | --- | --- | --- |
| ox-large-cm | 241.961 | 242.703 | +0.2%, +0.6%, +0.5% |
| ox-large-tables | 254.608 | 253.898 | -0.2%, +0.5%, -0.5% |
| ox-huge-tables | 6127.273 | 6160.055 | +0.3%, +0.3%, +1.3% |
| short | 0.486 | 0.493 | +2.2%, +0.4%, +2.0% |
| short-reuse | 0.195 | 0.195 | +1.1%, -0.1%, -1.5% |
| plain | 6.806 | 6.799 | +0.0%, -0.3%, -0.1% |
| multiline | 13.410 | 13.418 | +0.2%, +0.1%, +0.1% |
| inline | 39.631 | 39.605 | -0.1%, -0.1%, -0.2% |
| lists | 45.209 | 45.299 | +0.2%, +0.0%, +0.2% |
| code | 16.869 | 16.930 | +0.2%, +0.2%, +0.4% |
| tables | 41.907 | 41.854 | -0.6%, +0.9%, -0.1% |
| headings | 37.627 | 37.846 | +1.3%, +0.6%, +0.2% |
| unique-headings | 33.918 | 33.994 | +0.5%, +0.2%, +0.2% |
| long-prose | 9.970 | 9.984 | +0.0%, +0.6%, +0.2% |
| late-marker | 12.644 | 12.692 | +0.1%, +0.5%, +0.1% |
| long-code | 20.294 | 20.268 | -0.1%, +0.1%, -0.1% |
| escape-dense | 12.900 | 12.980 | +0.6%, +1.0%, +0.4% |
| commonmark/tiny | 0.467 | 0.466 | +0.9%, -1.3%, +0.1% |
| commonmark/short-100b | 0.869 | 0.871 | -0.1%, +1.3%, +0.9% |
| gfm_overlap/tiny | 0.468 | 0.467 | +1.1%, -0.8%, -0.3% |
| gfm_overlap/short-100b | 0.866 | 0.873 | -0.2%, +1.3%, +0.9% |
| tables/tables-plain | 25.529 | 25.598 | +0.3%, +0.2%, -0.6% |
| tables/tables-commonmark-inline | 32.310 | 32.468 | +0.5%, +0.3%, -0.0% |
| tables/tables-links | 37.543 | 37.600 | +0.2%, -0.0%, +0.2% |
| commonmark/publication-5k | 23.928 | 23.867 | +0.3%, -0.3%, -0.1% |
| guard/plain | 25.390 | 25.426 | +0.1%, +0.2%, +0.0% |
| guard/mixed | 32.491 | 32.297 | -1.0%, -0.6%, +0.0% |
| guard/links | 37.097 | 37.200 | +0.3%, +0.3%, -0.2% |
| guard/mixed-document | 17.817 | 17.833 | -0.1%, +0.3%, +0.1% |
| guard/one-table | 17.449 | 17.396 | +0.8%, -0.2%, -0.3% |
| guard/long-cells | 25.034 | 25.052 | -0.3%, -0.3%, +0.3% |
| guard/wide9 | 38.194 | 38.125 | -0.4%, -0.2%, -0.1% |
| guard/wide16 | 58.892 | 58.664 | -0.1%, +0.0%, -0.9% |
| guard/escapes-code | 67.809 | 67.915 | +0.5%, +0.2%, +0.6% |
| guard/prose | 9.878 | 9.903 | +0.2%, +0.5%, -0.4% |
| guard/links-prose | 36.191 | 36.074 | -0.1%, -0.4%, -0.4% |
| guard/edge-cases | 1338.203 | 1335.514 | +0.2%, -0.3%, +0.1% |
| guard/references | 10.407 | 10.416 | +0.1%, +0.7%, -0.1% |
| guard/empty | 0.143 | 0.143 | -0.1%, -0.6%, +1.8% |
| guard/gfm-document | 24.153 | 24.250 | +0.4%, -0.1%, +0.4% |
| guard/long-delimiters | 13.728 | 13.726 | -0.2%, +0.1%, +0.3% |
| guard/deep-emphasis | 38.620 | 38.563 | -0.2%, +0.3%, -0.3% |
| mdx/short | 1.171 | 1.195 | +2.7%, +2.1%, +0.3% |
| mdx/segments | 162.613 | 161.914 | -0.4%, +0.6%, -0.6% |
| mdx/tables-code | 147.387 | 147.870 | -1.2%, +0.3%, +0.2% |
| corpus/vue-docs/src/api/application.md | 54.147 | 54.303 | -0.4%, +0.4%, -0.3% |
| corpus/vue-docs/src/api/built-in-directives.md | 57.770 | 57.808 | -0.3%, +0.0%, +0.3% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 17.116 | 17.195 | +1.2%, -0.2%, -0.3% |
| corpus/vue-docs/src/guide/extras/render-function.md | 44.465 | 44.425 | -0.1%, +0.5%, -0.1% |
| corpus/vite-docs/docs/config/shared-options.md | 73.569 | 74.173 | +0.0%, +0.8%, +1.6% |
| corpus/vite-docs/docs/guide/api-plugin.md | 74.818 | 74.784 | -0.7%, +0.6%, +0.3% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 101.444 | 100.594 | -0.9%, -0.3%, -0.8% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 64.122 | 63.815 | -0.4%, -0.5%, -1.0% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 75.946 | 75.205 | +0.4%, -1.5%, +0.0% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 18.543 | 17.827 | -3.8%, -3.8%, -4.3% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 33.268 | 30.553 | -8.2%, -8.1%, -8.0% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 105.423 | 104.337 | -0.9%, -1.0%, -1.4% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 81.561 | 82.310 | +0.4%, +0.9%, -0.4% |
| corpus/concat/vue-docs/matched | 2738.099 | 2734.408 | -0.2%, -0.2%, +0.0% |
| corpus/concat/vue-docs/upstream | 3058.344 | 3059.915 | -0.0%, -0.1%, +0.1% |
| corpus/concat/vite-docs/matched | 1475.941 | 1477.938 | +0.5%, -0.3%, +0.1% |
| corpus/concat/vite-docs/upstream | 1761.657 | 1761.065 | -0.2%, -0.0%, -0.1% |
| corpus/concat/rust-book/matched | 4240.711 | 4249.048 | +0.0%, +0.2%, -0.1% |
| corpus/concat/rust-book/upstream | 4600.152 | 4591.995 | -0.3%, -0.2%, +0.2% |
| corpus/concat/typescript-handbook/matched | 5433.888 | 5433.643 | +0.4%, -0.2%, -0.4% |
| corpus/concat/typescript-handbook/upstream | 6012.758 | 5995.828 | -0.2%, -0.3%, -1.0% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.611 | 1.610 | +0.3%, +0.6%, -0.3% |
| corpus/ox-parser/LARGE_MD/1 | 8.316 | 8.352 | -0.2%, +0.6%, +0.2% |
| corpus/ox-parser/LARGE_MD/100 | 758.285 | 762.251 | -0.0%, +0.5%, +0.2% |
| html/root-short-lines | 44.489 | 41.451 | -6.9%, -6.9%, -6.7% |
| html/root-long-lines | 4.529 | 4.467 | -1.3%, -1.8%, -1.8% |
| html/separate-blocks | 166.360 | 167.491 | +1.1%, -0.8%, +0.7% |
| html/comment-long-lines | 5.363 | 5.273 | -1.6%, -1.8%, -1.8% |
| html/script-long-lines | 5.351 | 5.256 | -1.6%, -2.0%, -1.7% |
| html/script-candidates | 59.757 | 59.646 | -0.4%, -0.5%, +0.5% |
| html/comment-short-lines | 91.446 | 84.846 | -7.2%, -7.3%, -7.0% |
| html/processing-long-lines | 5.375 | 5.283 | -1.7%, -1.9%, -1.7% |
| html/cdata-long-lines | 5.362 | 5.305 | -1.2%, -0.1%, -1.7% |
| html/declaration-long-lines | 5.342 | 5.296 | -1.7%, -0.6%, -1.7% |
| html/container-html | 133.211 | 132.911 | -0.2%, -0.4%, -0.4% |
| html/custom-tags | 208.225 | 204.631 | -1.7%, -2.4%, -1.7% |

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
| vue-docs/src/api/application.md | 54.15 | 63.78 | 63.60 | 0.85× |
| vue-docs/src/api/built-in-directives.md | 58.03 | 67.48 | 67.16 | 0.86× |
| vue-docs/src/guide/built-ins/suspense.md | 17.03 | 10.45 | 10.48 | 1.63× |
| vue-docs/src/guide/extras/render-function.md | 44.28 | 31.32 | 31.19 | 1.41× |
| vite-docs/docs/config/shared-options.md | 74.34 | 66.18 | 65.32 | 1.12× |
| vite-docs/docs/guide/api-plugin.md | 75.18 | 74.41 | 74.56 | 1.01× |
| rust-book/src/ch02-00-guessing-game-tutorial.md | 102.15 | 70.32 | 69.07 | 1.45× |
| rust-book/src/ch09-02-recoverable-errors-with-result.md | 65.23 | 39.90 | 39.79 | 1.63× |
| rust-book/src/ch21-02-multithreaded.md | 77.21 | 51.85 | 51.62 | 1.49× |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 17.80 | 14.30 | 14.28 | 1.24× |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 31.03 | 26.55 | 26.30 | 1.17× |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 106.24 | 72.37 | 72.27 | 1.47× |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 83.37 | 59.79 | 59.54 | 1.39× |
| ox-parser/SIMPLE_MD/1 | 1.59 | 1.26 | 1.19 | 1.26× |
| ox-parser/LARGE_MD/1 | 8.29 | 7.16 | 7.06 | 1.16× |
| ox-parser/LARGE_MD/100 | 759.45 | 695.70 | 696.58 | 1.09× |

Normalized equality: 594 / 649 corpus cases. All unequal cases remain diagnostics.

## Exact-output checks

| Guard set | Cases | Failures |
| --- | --- | --- |
| Specification, extensions, reuse, all corpus files | 2695 | 0 |
| Initial inline / MDX events and HTML | 16374 | 0 |
| Independent inline / MDX events and HTML | 60000 | 0 |
| Public block / MDX streams, segments, HTML and metadata | 20740 | 0 |
| Every timed input | 81 | 0 |
