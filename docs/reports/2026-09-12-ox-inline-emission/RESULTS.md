# Inline emission measurements

Generated from the archived raw samples. Negative changes mean less elapsed time. Screening is exploratory; production confirmation is independent.

## Fresh baseline profiles

Each input has two six-second native sample captures at a one-millisecond interval. Percentages are visible stack shares, not additive bottleneck budgets or elapsed-time improvements.

| Input | Visible inline frames | Visible emit-point sorting |
| --- | --- | --- |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 48.8–50.7% | 2.9–3.2% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 48.5–49.4% | 9.7–10.1% |
| corpus/vue-docs/src/guide/extras/render-function.md | 20.9–21.0% | 1.5–1.8% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 2.5–2.8% | 0.0–0.1% |
| inline | 58.8–59.7% | 7.3–7.8% |

## Production profiles

These are new diagnostic captures of the final source, with the same sampling settings. A smaller sample share is not by itself an absolute timing improvement; use the paired timing results below.

| Input | Visible inline frames | Visible emit-point sorting |
| --- | --- | --- |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 41.8–41.8% | 1.3–1.4% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 43.1–44.1% | 6.9–6.9% |
| corpus/vue-docs/src/guide/extras/render-function.md | 19.7–20.8% | 1.3–1.6% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 2.6–2.7% | 0.1–0.2% |
| inline | 57.6–58.8% | 7.2–7.9% |

## Independent screens

Five alternating 30 ms windows per implementation/input after 60 ms warmup. The median covers 69 selected inputs and is not a weighted application workload. Failed output guards exclude a variant from production regardless of its timing.

| Variant | Median time change | Original guard failures | Additional event guard failures |
| --- | --- | --- | --- |
| code-one | not admitted | 3 | 39 |
| code-safe | -0.4% | 0 | 0 |
| code-safe+compact-all+compact-code | -0.9% | 0 | 0 |
| code-safe+compact-all+compact-code+html-search | -0.8% | 0 | 0 |
| code-safe+html-search | -0.8% | 0 | 0 |
| code-two | -0.8% | 0 | 4 |
| code-two+html-search | not admitted | 0 | 4 |
| compact-all | -0.2% | 0 | 0 |
| compact-links | -0.4% | 0 | 0 |
| compact-links+code-two | not admitted | 0 | 4 |
| html-search | -0.5% | 0 | 0 |
| insert-marks | +0.2% | 0 | 0 |
| merge-marks | +0.0% | 0 | 0 |
| merge-slices | +0.2% | 0 | 0 |

## Private point layout

The layout probe compiles the actual declarations with the recorded Rust compiler. Public event types are unchanged.

| Variant | EmitPoint bytes | EmitKind bytes |
| --- | --- | --- |
| baseline | 36 | 28 |
| compact-links | 20 | 12 |
| compact-all | 16 | 8 |
| production | 16 | 8 |

## Combination confirmation

Three fresh three-engine process groups, nine rotating 75 ms windows per engine/input after 100 ms warmup. These are prototype combinations before the final shared-padding cleanup. The additional compact-payload benefit is workload-dependent; complete results remain in the archived JSON.

| Input | Code/HTML vs baseline | With compact payloads vs baseline |
| --- | --- | --- |
| multiline | +0.1%, -0.7%, -0.5% | -0.1%, -0.1%, -0.5% |
| inline | -1.0%, -0.7%, -0.9% | -1.5%, -0.9%, -1.6% |
| guard/deep-emphasis | -0.1%, +0.2%, -0.1% | -15.9%, -15.9%, -16.6% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | -14.2%, -14.3%, -14.6% | -15.4%, -14.6%, -14.6% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | -10.1%, -11.1%, -10.2% | -10.6%, -10.8%, -9.0% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | +0.4%, +0.3%, +0.3% | -0.5%, +0.5%, +0.2% |

## Production confirmation

The formatted production source is rebuilt separately. Three fresh process pairs, nine alternating 75 ms windows per implementation/input after 60 ms warmup. Latencies summarize the three per-process medians; each paired change remains visible.

| Input | Baseline µs | Production µs | Paired changes |
| --- | --- | --- | --- |
| ox-large-cm | 246.285 | 241.029 | -0.6%, -2.6%, -1.5% |
| ox-large-tables | 254.892 | 252.600 | -0.5%, -2.0%, -0.9% |
| ox-huge-tables | 6166.828 | 6135.966 | -0.1%, -2.6%, -0.5% |
| short | 0.488 | 0.488 | -2.4%, +0.2%, +1.8% |
| short-reuse | 0.199 | 0.195 | -2.1%, -0.8%, -2.1% |
| plain | 6.712 | 6.705 | -0.0%, -0.3%, -0.5% |
| multiline | 13.298 | 13.327 | +0.1%, +0.1%, +0.4% |
| inline | 39.929 | 39.315 | -1.2%, -1.5%, -1.5% |
| lists | 45.490 | 45.402 | -0.3%, +0.3%, -0.1% |
| code | 16.764 | 16.771 | -0.0%, +0.0%, -0.4% |
| tables | 41.886 | 41.632 | -0.3%, -0.3%, -0.8% |
| headings | 37.604 | 37.735 | +0.3%, +0.1%, +0.1% |
| unique-headings | 33.717 | 33.900 | -0.3%, +0.5%, +0.6% |
| long-prose | 9.930 | 9.918 | +0.2%, -0.1%, -0.3% |
| late-marker | 12.636 | 12.602 | -0.0%, -0.3%, +0.0% |
| long-code | 20.181 | 20.170 | +0.3%, +0.4%, -0.3% |
| escape-dense | 12.978 | 13.019 | +0.3%, +0.8%, +0.3% |
| commonmark/tiny | 0.472 | 0.467 | -1.7%, -0.9%, -0.8% |
| commonmark/short-100b | 0.869 | 0.853 | -2.0%, -2.8%, -0.4% |
| gfm_overlap/tiny | 0.473 | 0.466 | -1.4%, -1.4%, -0.6% |
| gfm_overlap/short-100b | 0.867 | 0.846 | -2.2%, -2.5%, -0.3% |
| tables/tables-plain | 25.719 | 25.742 | -0.2%, +0.1%, +0.1% |
| tables/tables-commonmark-inline | 32.482 | 32.530 | +0.3%, +0.3%, +0.1% |
| tables/tables-links | 37.491 | 37.465 | +0.1%, -0.3%, -0.2% |
| commonmark/publication-5k | 24.564 | 24.099 | -1.9%, -2.0%, -1.4% |
| guard/plain | 25.631 | 25.504 | -0.5%, -0.4%, -0.5% |
| guard/mixed | 32.399 | 32.479 | +0.5%, +0.2%, +0.4% |
| guard/links | 37.669 | 37.778 | -0.0%, +0.7%, +0.1% |
| guard/mixed-document | 18.026 | 17.829 | -2.0%, -1.1%, -1.2% |
| guard/one-table | 17.317 | 17.300 | -0.8%, +0.3%, -0.2% |
| guard/long-cells | 24.939 | 24.962 | -0.4%, +0.1%, -0.4% |
| guard/wide9 | 37.964 | 37.755 | -0.4%, -0.1%, -0.9% |
| guard/wide16 | 58.517 | 58.406 | +0.0%, -0.2%, -1.0% |
| guard/escapes-code | 67.669 | 66.580 | -1.1%, -2.0%, -1.6% |
| guard/prose | 9.711 | 9.673 | -0.1%, +0.3%, -0.4% |
| guard/links-prose | 35.484 | 35.555 | +0.2%, +0.7%, +0.2% |
| guard/edge-cases | 1388.389 | 1328.434 | -3.9%, -4.1%, -5.1% |
| guard/references | 10.438 | 10.267 | -1.6%, -1.4%, -1.5% |
| guard/empty | 0.142 | 0.142 | +0.1%, +0.3%, -0.1% |
| guard/gfm-document | 24.328 | 23.950 | -1.6%, -1.2%, -1.1% |
| guard/long-delimiters | 13.642 | 13.657 | +0.0%, -0.4%, +0.1% |
| guard/deep-emphasis | 45.746 | 38.300 | -16.4%, -16.1%, -16.1% |
| mdx/short | 1.160 | 1.169 | -0.6%, -0.2%, +3.3% |
| mdx/segments | 163.810 | 164.909 | +0.6%, +0.6%, +0.7% |
| mdx/tables-code | 148.030 | 148.142 | +0.0%, -0.2%, +0.3% |
| corpus/vue-docs/src/api/application.md | 54.344 | 53.779 | -1.6%, -0.8%, -1.1% |
| corpus/vue-docs/src/api/built-in-directives.md | 59.081 | 57.662 | -2.8%, -2.6%, -1.9% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 19.990 | 16.982 | -15.2%, -15.0%, -15.1% |
| corpus/vue-docs/src/guide/extras/render-function.md | 45.086 | 44.488 | -1.2%, -1.3%, -0.6% |
| corpus/vite-docs/docs/config/shared-options.md | 74.930 | 73.048 | -3.4%, -2.5%, -2.5% |
| corpus/vite-docs/docs/guide/api-plugin.md | 75.645 | 74.026 | -2.1%, -3.0%, -2.7% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 109.885 | 102.524 | -6.5%, -6.8%, -6.6% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 71.672 | 64.964 | -9.2%, -9.4%, -10.3% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 82.349 | 75.920 | -6.1%, -9.5%, -7.8% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 22.903 | 22.543 | -1.5%, -1.8%, -1.3% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 46.606 | 46.767 | -0.2%, -0.3%, +0.8% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 108.352 | 105.686 | -2.5%, -1.1%, -3.2% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 85.564 | 81.970 | -4.4%, -3.8%, -2.9% |
| corpus/concat/vue-docs/matched | 2786.079 | 2742.875 | -1.7%, -1.3%, -1.7% |
| corpus/concat/vue-docs/upstream | 3114.466 | 3055.275 | -1.3%, -2.0%, -1.8% |
| corpus/concat/vite-docs/matched | 1482.892 | 1459.398 | -1.6%, -1.7%, -1.3% |
| corpus/concat/vite-docs/upstream | 1767.068 | 1751.115 | -0.7%, -0.9%, -0.9% |
| corpus/concat/rust-book/matched | 4409.409 | 4236.495 | -4.0%, -3.9%, -3.3% |
| corpus/concat/rust-book/upstream | 4797.240 | 4640.871 | -3.3%, -3.3%, -3.1% |
| corpus/concat/typescript-handbook/matched | 5518.042 | 5434.180 | -0.6%, -1.6%, -1.6% |
| corpus/concat/typescript-handbook/upstream | 6140.948 | 6007.625 | -2.2%, -3.2%, -1.7% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.602 | 1.607 | +0.2%, +0.4%, +0.8% |
| corpus/ox-parser/LARGE_MD/1 | 8.300 | 8.315 | -0.0%, +0.2%, +0.7% |
| corpus/ox-parser/LARGE_MD/100 | 761.858 | 759.573 | -0.5%, -0.2%, -0.3% |

Controls exceeding a 2% regression in all three runs: none.

## Requested live heap

The existing counting allocator measures 45 inputs, three fresh processes per implementation, and ten identical observations per process/input. HTML remains live at the observation boundary. This is not RSS.

Peak requested live bytes change in 26 of 45 cases; 0 increase. All corresponding HTML hashes match.

| Input | Input bytes | Baseline peak | Production peak | Change |
| --- | --- | --- | --- | --- |
| ox-large-cm | 49898 | 357226 | 356906 | -0.0896% |
| ox-large-tables | 49898 | 357226 | 356906 | -0.0896% |
| ox-huge-tables | 1072848 | 7481162 | 7480842 | -0.0043% |
| short | 17 | 3312 | 3152 | -4.8309% |
| plain | 6300 | 15731 | 15731 | +0.0000% |
| multiline | 7500 | 26363 | 26203 | -0.6069% |
| inline | 8100 | 31354 | 30974 | -1.2120% |
| lists | 5300 | 57874 | 57714 | -0.2765% |
| code | 6200 | 23228 | 23228 | +0.0000% |
| tables | 7500 | 80222 | 80222 | +0.0000% |
| headings | 4200 | 69264 | 69264 | +0.0000% |
| unique-headings | 4388 | 70972 | 70972 | +0.0000% |
| long-prose | 65536 | 344096 | 344096 | +0.0000% |
| late-marker | 65540 | 393813 | 393653 | -0.0406% |
| long-code | 70712 | 160630 | 160630 | +0.0000% |
| escape-dense | 2700 | 20412 | 20412 | +0.0000% |
| commonmark/tiny | 18 | 3312 | 3152 | -4.8309% |
| commonmark/short-100b | 103 | 4116 | 3816 | -7.2886% |
| gfm_overlap/tiny | 18 | 3312 | 3152 | -4.8309% |
| gfm_overlap/short-100b | 103 | 4116 | 3816 | -7.2886% |
| tables/tables-plain | 4560 | 49448 | 49448 | +0.0000% |
| tables/tables-commonmark-inline | 4800 | 64688 | 64528 | -0.2473% |
| tables/tables-links | 6360 | 69252 | 69092 | -0.2310% |
| commonmark/publication-5k | 5120 | 29352 | 29092 | -0.8858% |
| guard/plain | 4560 | 49448 | 49448 | +0.0000% |
| guard/mixed | 4800 | 64688 | 64528 | -0.2473% |
| guard/links | 6360 | 69252 | 69092 | -0.2310% |
| guard/mixed-document | 5632 | 34268 | 33628 | -1.8676% |
| guard/one-table | 3056 | 33656 | 33656 | +0.0000% |
| guard/long-cells | 53152 | 121160 | 121160 | +0.0000% |
| guard/wide9 | 6621 | 71000 | 71000 | +0.0000% |
| guard/wide16 | 11612 | 123414 | 123414 | +0.0000% |
| guard/escapes-code | 6860 | 33208 | 33028 | -0.5420% |
| guard/prose | 7050 | 24460 | 24460 | +0.0000% |
| guard/links-prose | 5800 | 22836 | 22636 | -0.8758% |
| guard/edge-cases | 175556 | 658079 | 647479 | -1.6107% |
| guard/references | 876 | 7883 | 7563 | -4.0594% |
| guard/empty | 0 | 2560 | 2560 | +0.0000% |
| guard/gfm-document | 5120 | 29352 | 29092 | -0.8858% |
| guard/long-delimiters | 16426 | 38516 | 38516 | +0.0000% |
| guard/deep-emphasis | 3073 | 173698 | 132738 | -23.5812% |
| large/mixed-1mib | 1048898 | 7323174 | 7322854 | -0.0044% |
| large/plain-1mib | 1048576 | 5505056 | 5505056 | +0.0000% |
| large/mixed-8mib | 8388689 | 58547708 | 58547388 | -0.0005% |
| large/plain-8mib | 8388608 | 44040224 | 44040224 | +0.0000% |

## Fresh Ox corpus comparison

This is a separate native harness: do not subtract its times from the optimization experiment. Three process groups, nine 60 ms windows per engine/input. Both engines create and destroy fresh state and owned HTML with heading IDs enabled. Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`; its growing arena is primary. Only equal normalized outputs support a ranking.

| Input | Ferromark µs | Ox growing µs | Ox presized µs | F / Ox growing |
| --- | --- | --- | --- | --- |
| vue-docs/src/api/application.md | 54.05 | 64.15 | 63.64 | 0.84× |
| vue-docs/src/api/built-in-directives.md | 57.80 | 67.23 | 67.00 | 0.86× |
| vue-docs/src/guide/built-ins/suspense.md | 17.03 | 10.46 | 10.48 | 1.63× |
| vue-docs/src/guide/extras/render-function.md | 44.58 | 31.44 | 31.32 | 1.42× |
| vite-docs/docs/config/shared-options.md | 74.05 | 65.18 | 64.81 | 1.14× |
| vite-docs/docs/guide/api-plugin.md | 74.77 | 75.09 | 74.45 | 1.00× |
| rust-book/src/ch02-00-guessing-game-tutorial.md | 103.85 | 70.13 | 68.99 | 1.48× |
| rust-book/src/ch09-02-recoverable-errors-with-result.md | 66.55 | 40.00 | 39.88 | 1.66× |
| rust-book/src/ch21-02-multithreaded.md | 78.82 | 52.08 | 51.24 | 1.51× |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 22.57 | 14.29 | 14.20 | 1.58× |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 46.89 | 26.45 | 26.26 | 1.77× |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 106.27 | 72.36 | 71.56 | 1.47× |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 82.35 | 59.11 | 58.96 | 1.39× |
| ox-parser/SIMPLE_MD/1 | 1.61 | 1.26 | 1.19 | 1.28× |
| ox-parser/LARGE_MD/1 | 8.34 | 7.03 | 6.96 | 1.19× |
| ox-parser/LARGE_MD/100 | 756.80 | 687.90 | 683.84 | 1.10× |

Normalized equality: 594 / 649 corpus cases. Unequal concatenations remain in the raw diagnostics and are excluded from this ranking table.

## Exact-output verification

| Guard set | Cases | Failures |
| --- | --- | --- |
| Specification, extensions, renderer reuse, and Ox corpus | 2695 | 0 |
| Initial deterministic HTML / inline / MDX event guards | 16374 | 0 |
| Independent deterministic HTML / inline / MDX event guards | 60000 | 0 |

