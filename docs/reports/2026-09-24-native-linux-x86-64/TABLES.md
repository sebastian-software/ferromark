# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.72× | 0.59× | 0.24× | 0.42× | 0.20× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.48× | 0.31× | 0.40× | 0.21× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.68× | 0.47× | 0.31× | 0.40× | 0.22× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.59× | 0.26× | 0.51× | 0.29× |
| comments (all workloads) | 12 | 1.00× | 0.74× | 0.59× | 0.27× | 0.51× | 0.29× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.37× | 0.30× | 0.37× | 0.20× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.60× | 0.38× | 0.30× | 0.36× | 0.18× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.59× | 0.22× | 0.26× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.63× | 0.59× | 0.22× | 0.26× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.54× | 0.41× | 0.50× | 0.29× |
| readme (all workloads) | 2 | 1.00× | 0.76× | 0.54× | 0.41× | 0.50× | 0.29× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.48× | 0.35× | 0.48× | 0.26× |
| reference (all workloads) | 4 | 1.00× | 0.71× | 0.48× | 0.35× | 0.48× | 0.26× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.80× | 0.44× | 0.26× | 0.64× | 0.52× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.45× | 0.34× | 0.38× | 0.21× |
| technical-docs (all workloads) | 22 | 1.00× | 0.69× | 0.45× | 0.34× | 0.38× | 0.21× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.43× | 0.27× | 0.32× | 0.15× |
| commonmark (all workloads) | 17 | 1.00× | 0.62× | 0.42× | 0.28× | 0.34× | 0.17× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.49× | 0.32× | 0.43× | 0.24× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.71× | 0.49× | 0.32× | 0.43× | 0.24× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.58× | 0.25× | 0.51× | 0.29× |
| <512 B (all workloads) | 12 | 1.00× | 0.75× | 0.56× | 0.25× | 0.52× | 0.31× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.48× | 0.34× | 0.42× | 0.23× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.65× | 0.48× | 0.34× | 0.42× | 0.23× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.42× | 0.33× | 0.37× | 0.20× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.67× | 0.42× | 0.33× | 0.37× | 0.20× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.48× | 0.39× | 0.43× | 0.24× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.73× | 0.48× | 0.39× | 0.43× | 0.24× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.48× | 0.23× | 0.30× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.63× | 0.45× | 0.26× | 0.32× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.73× | 0.62× | 0.20× | 0.36× | 0.16× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.50× | 0.29× | 0.39× | 0.20× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.68× | 0.49× | 0.29× | 0.39× | 0.20× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.62× | 0.21× | 0.42× | 0.21× |
| comments (all workloads) | 12 | 1.00× | 0.76× | 0.62× | 0.21× | 0.43× | 0.22× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.39× | 0.29× | 0.36× | 0.19× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.60× | 0.40× | 0.30× | 0.36× | 0.18× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.64× | 0.22× | 0.26× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.62× | 0.64× | 0.22× | 0.26× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.57× | 0.41× | 0.51× | 0.28× |
| readme (all workloads) | 2 | 1.00× | 0.76× | 0.57× | 0.41× | 0.51× | 0.28× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.49× | 0.36× | 0.49× | 0.26× |
| reference (all workloads) | 4 | 1.00× | 0.71× | 0.49× | 0.36× | 0.49× | 0.26× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.88× | 0.50× | 0.21× | 0.54× | 0.41× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.47× | 0.34× | 0.38× | 0.20× |
| technical-docs (all workloads) | 22 | 1.00× | 0.69× | 0.47× | 0.35× | 0.38× | 0.20× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.46× | 0.27× | 0.32× | 0.14× |
| commonmark (all workloads) | 17 | 1.00× | 0.62× | 0.45× | 0.27× | 0.34× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.30× | 0.41× | 0.21× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.72× | 0.51× | 0.30× | 0.41× | 0.22× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.61× | 0.19× | 0.42× | 0.21× |
| <512 B (all workloads) | 12 | 1.00× | 0.77× | 0.59× | 0.20× | 0.43× | 0.23× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.51× | 0.33× | 0.41× | 0.22× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.65× | 0.51× | 0.33× | 0.41× | 0.22× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.44× | 0.34× | 0.37× | 0.20× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.66× | 0.44× | 0.34× | 0.37× | 0.20× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.50× | 0.39× | 0.44× | 0.24× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.73× | 0.50× | 0.39× | 0.44× | 0.24× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.24× | 0.30× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.62× | 0.47× | 0.27× | 0.33× | 0.14× |

## HTML agreement with v2

| Engine | Exact | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 57 | 0 | 0 | 0 |
| OX-Content original | 16 | 0 | 39 | 2 |
| Ferromark v1 | 18 | 34 | 0 | 5 |
| md4c | 19 | 33 | 0 | 5 |
| pulldown-cmark | 14 | 43 | 0 | 0 |
| Bun native bun_md | 16 | 40 | 0 | 1 |

## Per-document timings

Microseconds per complete Markdown→HTML operation; lower is faster. Agreement columns show the six/five-engine sets.
Original input sizes are UTF-8 bytes. “gfm” is the shared subset described in the harness README.

### fresh

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.212 | 0.289 | 0.274 | 1.257 | 0.320 | 0.427 |
| comment-question | 160 | gfm | yes | yes | 0.231 | 0.304 | 0.329 | 1.394 | 0.453 | 0.944 |
| comment-review | 282 | gfm | yes | yes | 0.300 | 0.408 | 0.520 | 1.651 | 0.735 | 1.587 |
| comment-links | 278 | gfm | yes | yes | 0.674 | 0.916 | 1.360 | 2.259 | 1.288 | 2.413 |
| comment-checklist | 287 | gfm | no | no | 0.801 | 1.241 | 1.549 | 2.435 | 1.477 | 2.610 |
| comment-quote | 290 | gfm | yes | yes | 0.406 | 0.546 | 0.670 | 1.810 | 0.881 | 1.759 |
| comment-unicode | 327 | gfm | yes | yes | 0.720 | 0.799 | 1.287 | 2.140 | 1.330 | 2.052 |
| comment-inline-code | 285 | gfm | yes | yes | 0.430 | 0.541 | 1.146 | 1.893 | 1.142 | 1.691 |
| comment-reproduction | 298 | gfm | yes | yes | 0.476 | 0.630 | 0.896 | 2.110 | 0.969 | 1.918 |
| comment-table | 310 | gfm | yes | yes | 1.548 | 1.937 | 1.767 | 3.337 | 1.907 | 2.806 |
| comment-review-long | 957 | gfm | yes | yes | 1.051 | 1.590 | 2.074 | 3.450 | 2.555 | 5.187 |
| comment-incident | 1124 | gfm | no | yes | 2.001 | 2.868 | 2.874 | 4.369 | 3.586 | 6.538 |
| guard-angle-link | 41 | commonmark | no | no | 0.436 | 0.547 | 0.999 | 1.648 | 0.683 | 0.844 |
| legacy-contributing | 9323 | gfm | no | yes | 14.070 | 18.599 | 28.218 | 35.094 | 34.463 | 57.800 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 13.460 | 18.177 | 29.592 | 38.094 | 33.934 | 59.322 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.764 | 4.183 | 6.949 | 8.116 | 7.782 | 12.508 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 3.816 | 5.443 | 7.629 | 9.885 | 8.628 | 14.531 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 9.073 | 12.467 | 23.078 | 28.431 | 26.224 | 44.330 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 3.414 | 5.035 | 8.694 | 10.392 | 9.355 | 15.379 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.509 | 8.856 | 16.579 | 16.662 | 18.282 | 27.593 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 3.453 | 4.490 | 7.948 | 10.755 | 9.984 | 17.897 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.932 | 6.866 | 12.565 | 14.361 | 14.958 | 24.876 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 10.056 | 13.417 | 22.334 | 30.369 | 23.984 | 46.357 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.739 | 8.643 | 10.651 | 14.100 | 10.613 | 18.041 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.863 | 2.874 | 4.303 | 5.954 | 5.294 | 9.673 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.941 | 1.566 | 2.001 | 3.104 | 2.146 | 3.683 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 61.847 | 78.919 | 94.851 | 119.138 | 96.230 | 131.671 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.566 | 22.700 | 28.002 | 36.307 | 37.770 | 66.667 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 10.124 | 11.780 | 21.044 | 26.829 | 26.513 | 52.355 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.655 | 11.234 | 12.515 | 19.068 | 17.025 | 35.831 |
| vue-docs-suspense | 8291 | gfm | no | yes | 11.819 | 16.412 | 32.495 | 33.363 | 31.605 | 54.320 |
| vue-docs-slots | 24211 | gfm | no | yes | 28.299 | 50.996 | 95.079 | 111.591 | 87.554 | 196.376 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 27.550 | 44.850 | 68.411 | 95.352 | 80.084 | 165.992 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.275 | 6.857 | 7.883 | 12.410 | 10.815 | 22.230 |
| vite-docs-performance | 8184 | gfm | no | yes | 11.150 | 16.269 | 21.367 | 30.111 | 27.878 | 49.527 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 84.071 | 109.960 | 130.340 | 164.089 | 139.626 | 243.446 |
| vite-docs-features | 39739 | gfm | no | no | 71.444 | 105.700 | 158.287 | 200.855 | 175.246 | 307.677 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.505 | 9.964 | 10.045 | 16.128 | 14.361 | 29.266 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 52.542 | 81.254 | 118.502 | 168.252 | 133.012 | 235.319 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.561 | 36.334 | 82.783 | 128.090 | 69.266 | 189.964 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 66.793 | 93.489 | 174.494 | 235.835 | 185.136 | 358.703 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.646 | 2.594 | 3.487 | 5.435 | 4.127 | 7.366 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.475 | 3.974 | 5.312 | 8.401 | 6.770 | 13.221 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 46.325 | 77.785 | 109.949 | 162.849 | 140.394 | 304.393 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 11.780 | 18.502 | 19.758 | 60.757 | 51.243 | 145.731 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.860 | 4.657 | 7.615 | 8.646 | 6.884 | 11.868 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 12.333 | 20.266 | 49.800 | 38.396 | 33.545 | 62.214 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 79.368 | 133.170 | 211.288 | 255.379 | 222.539 | 466.886 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 14.518 | 23.129 | 25.457 | 64.699 | 55.522 | 167.354 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.197 | 3.751 | 5.426 | 7.504 | 5.781 | 10.358 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 6.269 | 10.849 | 17.119 | 21.942 | 18.655 | 36.200 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 162.099 | 272.093 | 364.756 | 490.676 | 425.427 | 914.573 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 33.146 | 53.239 | 54.431 | 139.125 | 120.495 | 349.539 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.821 | 6.263 | 12.187 | 13.422 | 11.141 | 21.619 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 5.014 | 8.363 | 15.070 | 18.191 | 15.379 | 31.085 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 88.018 | 155.719 | 223.999 | 298.851 | 266.004 | 537.459 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 18.359 | 29.675 | 31.414 | 79.332 | 70.265 | 194.952 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.117 | 0.150 | 0.153 | 1.187 | 0.273 | 0.431 |
| comment-question | 160 | gfm | yes | yes | 0.137 | 0.172 | 0.192 | 1.325 | 0.410 | 0.932 |
| comment-review | 282 | gfm | yes | yes | 0.200 | 0.256 | 0.327 | 1.559 | 0.649 | 1.580 |
| comment-links | 278 | gfm | yes | yes | 0.522 | 0.734 | 0.989 | 2.099 | 1.161 | 2.420 |
| comment-checklist | 287 | gfm | no | no | 0.675 | 1.062 | 1.236 | 2.192 | 1.353 | 2.609 |
| comment-quote | 290 | gfm | yes | yes | 0.291 | 0.379 | 0.441 | 1.672 | 0.756 | 1.766 |
| comment-unicode | 327 | gfm | yes | yes | 0.599 | 0.614 | 0.879 | 2.015 | 1.232 | 2.068 |
| comment-inline-code | 285 | gfm | yes | yes | 0.307 | 0.378 | 0.869 | 1.735 | 0.984 | 1.690 |
| comment-reproduction | 298 | gfm | yes | yes | 0.341 | 0.449 | 0.693 | 1.916 | 0.843 | 1.915 |
| comment-table | 310 | gfm | yes | yes | 1.438 | 1.747 | 1.469 | 3.168 | 1.764 | 2.795 |
| comment-review-long | 957 | gfm | yes | yes | 0.900 | 1.394 | 1.619 | 3.291 | 2.420 | 5.198 |
| comment-incident | 1124 | gfm | no | yes | 1.865 | 2.670 | 2.529 | 4.090 | 3.410 | 6.537 |
| guard-angle-link | 41 | commonmark | no | no | 0.328 | 0.371 | 0.658 | 1.571 | 0.605 | 0.808 |
| legacy-contributing | 9323 | gfm | no | yes | 13.873 | 18.254 | 26.844 | 33.920 | 33.868 | 57.459 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 13.293 | 17.946 | 27.382 | 37.168 | 32.984 | 59.150 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.644 | 3.933 | 6.333 | 7.777 | 7.437 | 12.354 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 3.692 | 5.183 | 7.126 | 9.448 | 8.329 | 14.613 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 8.882 | 12.158 | 22.515 | 27.306 | 25.436 | 44.589 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 3.273 | 4.811 | 7.798 | 9.899 | 8.927 | 15.386 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.263 | 8.564 | 15.632 | 16.046 | 17.758 | 27.560 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 3.174 | 4.278 | 7.058 | 10.080 | 9.455 | 17.999 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.710 | 6.652 | 11.821 | 13.669 | 14.539 | 24.991 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 9.834 | 13.156 | 20.956 | 29.615 | 23.221 | 45.804 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.515 | 8.455 | 9.610 | 13.835 | 10.284 | 18.068 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.706 | 2.635 | 3.659 | 5.600 | 5.124 | 9.702 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.801 | 1.356 | 1.606 | 2.868 | 1.950 | 3.677 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 62.447 | 77.863 | 93.131 | 118.667 | 92.408 | 129.628 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.534 | 22.313 | 26.368 | 35.622 | 36.668 | 67.055 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 9.948 | 11.531 | 19.349 | 25.914 | 26.488 | 53.116 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.404 | 10.930 | 11.750 | 18.599 | 16.262 | 36.176 |
| vue-docs-suspense | 8291 | gfm | no | yes | 11.847 | 16.118 | 30.717 | 32.516 | 30.372 | 54.454 |
| vue-docs-slots | 24211 | gfm | no | yes | 27.791 | 50.829 | 91.478 | 109.884 | 85.830 | 194.131 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 27.423 | 44.348 | 66.144 | 92.934 | 78.160 | 165.811 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.135 | 6.556 | 7.070 | 11.886 | 10.171 | 22.233 |
| vite-docs-performance | 8184 | gfm | no | yes | 10.966 | 15.825 | 20.330 | 29.286 | 26.993 | 49.799 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 83.902 | 109.742 | 128.497 | 160.541 | 133.418 | 241.633 |
| vite-docs-features | 39739 | gfm | no | no | 71.352 | 105.241 | 154.512 | 197.895 | 174.293 | 307.909 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.295 | 9.658 | 9.181 | 15.645 | 13.719 | 29.197 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 51.901 | 80.814 | 117.247 | 164.590 | 131.447 | 235.147 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.498 | 35.919 | 80.743 | 124.285 | 66.939 | 190.041 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 65.961 | 92.876 | 171.703 | 232.550 | 181.870 | 361.783 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.488 | 2.360 | 2.916 | 5.133 | 3.915 | 7.311 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.351 | 3.719 | 4.554 | 8.063 | 6.548 | 13.298 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 45.618 | 77.639 | 104.354 | 159.141 | 137.578 | 305.835 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 11.391 | 18.302 | 17.400 | 58.587 | 48.921 | 144.153 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.679 | 4.410 | 6.935 | 8.245 | 6.584 | 11.702 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 12.273 | 20.071 | 45.590 | 37.735 | 32.581 | 63.195 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 78.674 | 132.701 | 205.623 | 251.770 | 221.246 | 463.753 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 14.320 | 22.822 | 23.044 | 63.215 | 54.340 | 170.320 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.052 | 3.600 | 4.762 | 7.139 | 5.494 | 10.423 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 6.083 | 10.551 | 14.955 | 21.235 | 17.898 | 36.203 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 161.188 | 272.030 | 354.015 | 481.728 | 411.280 | 912.811 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 32.582 | 52.052 | 50.536 | 133.428 | 114.851 | 347.686 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.622 | 6.023 | 11.766 | 13.008 | 10.789 | 21.836 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 4.916 | 8.301 | 14.311 | 17.616 | 15.036 | 31.120 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 86.742 | 156.421 | 218.206 | 295.594 | 255.104 | 540.328 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 18.209 | 29.324 | 28.535 | 76.844 | 66.882 | 195.040 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 634.69 | 973.80 | 1331.91 | 1784.49 | 1520.89 | 3451.38 |
| rotating-commonmark | reuse | 17 | 625.43 | 943.25 | 1270.95 | 1733.83 | 1460.98 | 3452.19 |
| rotating-gfm | fresh | 40 | 882.10 | 1222.01 | 1735.18 | 1957.08 | 1620.56 | 2848.78 |
| rotating-gfm | reuse | 40 | 848.59 | 1166.27 | 1644.24 | 1915.69 | 1560.13 | 2834.00 |
