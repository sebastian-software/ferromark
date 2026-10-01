# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.73× | 0.59× | 0.25× | 0.42× | 0.20× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.49× | 0.34× | 0.43× | 0.21× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.71× | 0.49× | 0.34× | 0.43× | 0.21× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.61× | 0.28× | 0.52× | 0.29× |
| comments (all workloads) | 12 | 1.00× | 0.77× | 0.61× | 0.28× | 0.53× | 0.29× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.36× | 0.32× | 0.39× | 0.19× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.60× | 0.38× | 0.32× | 0.37× | 0.17× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.58× | 0.23× | 0.26× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.62× | 0.58× | 0.23× | 0.26× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.56× | 0.46× | 0.53× | 0.28× |
| readme (all workloads) | 2 | 1.00× | 0.81× | 0.56× | 0.46× | 0.53× | 0.28× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.51× | 0.42× | 0.53× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.77× | 0.51× | 0.42× | 0.53× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.88× | 0.43× | 0.27× | 0.73× | 0.55× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.48× | 0.39× | 0.41× | 0.21× |
| technical-docs (all workloads) | 22 | 1.00× | 0.75× | 0.48× | 0.39× | 0.41× | 0.21× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.43× | 0.29× | 0.34× | 0.14× |
| commonmark (all workloads) | 17 | 1.00× | 0.62× | 0.42× | 0.29× | 0.35× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.36× | 0.46× | 0.24× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.76× | 0.52× | 0.36× | 0.46× | 0.24× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.59× | 0.26× | 0.52× | 0.29× |
| <512 B (all workloads) | 12 | 1.00× | 0.77× | 0.57× | 0.27× | 0.54× | 0.31× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.50× | 0.37× | 0.45× | 0.23× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.69× | 0.50× | 0.37× | 0.45× | 0.23× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.38× | 0.40× | 0.20× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.70× | 0.43× | 0.38× | 0.40× | 0.20× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.53× | 0.44× | 0.46× | 0.23× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.79× | 0.53× | 0.44× | 0.46× | 0.23× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.48× | 0.26× | 0.31× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.64× | 0.46× | 0.29× | 0.33× | 0.13× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.80× | 0.68× | 0.22× | 0.39× | 0.17× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.53× | 0.33× | 0.41× | 0.19× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.73× | 0.52× | 0.33× | 0.41× | 0.19× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.69× | 0.24× | 0.47× | 0.23× |
| comments (all workloads) | 12 | 1.00× | 0.86× | 0.69× | 0.24× | 0.48× | 0.24× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.37× | 0.31× | 0.37× | 0.17× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.59× | 0.39× | 0.32× | 0.36× | 0.16× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.66× | 0.24× | 0.26× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.61× | 0.66× | 0.24× | 0.26× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.60× | 0.46× | 0.53× | 0.27× |
| readme (all workloads) | 2 | 1.00× | 0.81× | 0.60× | 0.46× | 0.53× | 0.27× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.52× | 0.42× | 0.53× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.77× | 0.52× | 0.42× | 0.53× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.98× | 0.58× | 0.22× | 0.61× | 0.40× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.50× | 0.39× | 0.40× | 0.20× |
| technical-docs (all workloads) | 22 | 1.00× | 0.75× | 0.50× | 0.39× | 0.41× | 0.20× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.45× | 0.29× | 0.33× | 0.14× |
| commonmark (all workloads) | 17 | 1.00× | 0.61× | 0.45× | 0.29× | 0.34× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.56× | 0.34× | 0.44× | 0.22× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.79× | 0.56× | 0.34× | 0.44× | 0.22× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.67× | 0.22× | 0.47× | 0.23× |
| <512 B (all workloads) | 12 | 1.00× | 0.88× | 0.67× | 0.23× | 0.49× | 0.25× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.36× | 0.42× | 0.21× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.68× | 0.52× | 0.36× | 0.42× | 0.21× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.44× | 0.37× | 0.39× | 0.19× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.70× | 0.44× | 0.37× | 0.39× | 0.19× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.55× | 0.45× | 0.47× | 0.23× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.79× | 0.55× | 0.45× | 0.47× | 0.23× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.52× | 0.26× | 0.32× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.64× | 0.49× | 0.30× | 0.33× | 0.13× |

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
| comment-ack | 37 | gfm | yes | yes | 0.103 | 0.145 | 0.128 | 0.563 | 0.161 | 0.217 |
| comment-question | 160 | gfm | yes | yes | 0.109 | 0.144 | 0.142 | 0.630 | 0.233 | 0.434 |
| comment-review | 282 | gfm | yes | yes | 0.148 | 0.199 | 0.234 | 0.779 | 0.363 | 0.787 |
| comment-links | 278 | gfm | yes | yes | 0.352 | 0.412 | 0.706 | 1.127 | 0.597 | 1.186 |
| comment-checklist | 287 | gfm | no | no | 0.418 | 0.615 | 0.715 | 1.155 | 0.697 | 1.285 |
| comment-quote | 290 | gfm | yes | yes | 0.200 | 0.263 | 0.312 | 0.855 | 0.446 | 0.909 |
| comment-unicode | 327 | gfm | yes | yes | 0.378 | 0.402 | 0.697 | 1.044 | 0.646 | 1.008 |
| comment-inline-code | 285 | gfm | yes | yes | 0.203 | 0.262 | 0.559 | 0.888 | 0.484 | 0.863 |
| comment-reproduction | 298 | gfm | yes | yes | 0.210 | 0.293 | 0.437 | 0.958 | 0.454 | 0.884 |
| comment-table | 310 | gfm | yes | yes | 0.801 | 0.964 | 0.892 | 1.523 | 0.896 | 1.300 |
| comment-review-long | 957 | gfm | yes | yes | 0.548 | 0.788 | 1.050 | 1.680 | 1.370 | 2.721 |
| comment-incident | 1124 | gfm | no | yes | 1.120 | 1.469 | 1.483 | 2.131 | 1.846 | 3.442 |
| guard-angle-link | 41 | commonmark | no | no | 0.224 | 0.256 | 0.525 | 0.823 | 0.306 | 0.408 |
| legacy-contributing | 9323 | gfm | no | yes | 7.269 | 8.488 | 13.348 | 15.168 | 15.960 | 30.207 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 6.638 | 8.461 | 14.240 | 16.600 | 15.806 | 31.718 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.401 | 1.969 | 3.400 | 3.659 | 3.481 | 6.254 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 1.992 | 2.484 | 3.721 | 4.291 | 3.870 | 7.275 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.562 | 5.689 | 11.384 | 12.258 | 11.866 | 23.012 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 1.682 | 2.330 | 4.354 | 4.554 | 4.257 | 7.584 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.260 | 4.157 | 7.755 | 7.052 | 8.253 | 13.608 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.783 | 2.170 | 3.849 | 4.788 | 4.687 | 8.969 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.487 | 3.222 | 6.195 | 6.349 | 6.998 | 12.282 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 5.030 | 6.414 | 10.590 | 12.839 | 11.161 | 23.809 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.315 | 3.960 | 4.970 | 6.357 | 4.890 | 8.836 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 0.964 | 1.388 | 2.147 | 2.744 | 2.568 | 4.969 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.467 | 0.741 | 1.023 | 1.540 | 1.011 | 1.810 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 30.300 | 35.836 | 43.003 | 50.592 | 44.339 | 62.953 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.880 | 11.269 | 13.170 | 17.170 | 18.856 | 34.988 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 5.591 | 5.918 | 10.055 | 12.616 | 13.288 | 27.037 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.381 | 5.072 | 6.125 | 8.740 | 8.384 | 19.930 |
| vue-docs-suspense | 8291 | gfm | no | yes | 6.041 | 7.582 | 15.885 | 14.582 | 14.239 | 28.689 |
| vue-docs-slots | 24211 | gfm | no | yes | 14.573 | 25.303 | 39.980 | 52.038 | 41.764 | 108.612 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 13.695 | 21.313 | 32.149 | 43.369 | 39.277 | 89.347 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.163 | 3.152 | 3.847 | 6.132 | 5.365 | 11.670 |
| vite-docs-performance | 8184 | gfm | no | yes | 5.639 | 7.432 | 10.063 | 12.948 | 13.372 | 27.366 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 41.816 | 49.227 | 59.051 | 67.871 | 62.842 | 123.999 |
| vite-docs-features | 39739 | gfm | no | no | 35.178 | 47.394 | 72.955 | 86.086 | 82.635 | 167.354 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.441 | 4.777 | 4.764 | 7.525 | 6.880 | 15.706 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 27.658 | 36.331 | 57.490 | 68.939 | 60.435 | 126.626 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 13.453 | 20.932 | 47.429 | 66.454 | 36.617 | 88.176 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 32.728 | 42.659 | 82.979 | 94.724 | 86.848 | 194.628 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.873 | 1.336 | 1.771 | 2.516 | 2.012 | 3.944 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.286 | 2.013 | 2.544 | 4.188 | 3.300 | 7.236 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 23.262 | 39.279 | 51.570 | 78.632 | 71.318 | 169.182 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 6.093 | 10.160 | 10.576 | 29.943 | 25.538 | 78.160 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.504 | 2.365 | 4.351 | 4.268 | 3.362 | 6.515 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 6.511 | 10.530 | 27.366 | 19.363 | 16.573 | 34.792 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 39.699 | 66.018 | 106.873 | 116.367 | 116.046 | 259.957 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 7.563 | 12.215 | 13.270 | 32.013 | 28.928 | 86.395 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.185 | 1.932 | 2.772 | 3.718 | 2.964 | 5.743 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.216 | 5.847 | 7.655 | 10.897 | 9.255 | 19.759 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 73.292 | 131.750 | 174.844 | 237.636 | 230.153 | 504.249 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 17.083 | 26.983 | 27.899 | 66.023 | 60.938 | 183.721 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.964 | 3.442 | 7.260 | 6.399 | 5.344 | 11.714 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.587 | 4.467 | 8.526 | 8.352 | 7.668 | 17.016 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 44.345 | 77.120 | 106.577 | 135.192 | 137.533 | 302.263 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 9.517 | 15.251 | 16.554 | 39.093 | 34.796 | 106.897 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.071 | 0.072 | 0.078 | 0.548 | 0.127 | 0.222 |
| comment-question | 160 | gfm | yes | yes | 0.078 | 0.083 | 0.094 | 0.608 | 0.211 | 0.440 |
| comment-review | 282 | gfm | yes | yes | 0.111 | 0.130 | 0.157 | 0.755 | 0.317 | 0.795 |
| comment-links | 278 | gfm | yes | yes | 0.286 | 0.304 | 0.468 | 1.039 | 0.554 | 1.200 |
| comment-checklist | 287 | gfm | no | no | 0.368 | 0.522 | 0.539 | 1.026 | 0.610 | 1.280 |
| comment-quote | 290 | gfm | yes | yes | 0.165 | 0.182 | 0.201 | 0.774 | 0.384 | 0.906 |
| comment-unicode | 327 | gfm | yes | yes | 0.307 | 0.299 | 0.451 | 0.958 | 0.576 | 0.999 |
| comment-inline-code | 285 | gfm | yes | yes | 0.162 | 0.178 | 0.404 | 0.802 | 0.413 | 0.874 |
| comment-reproduction | 298 | gfm | yes | yes | 0.164 | 0.201 | 0.328 | 0.872 | 0.397 | 0.888 |
| comment-table | 310 | gfm | yes | yes | 0.722 | 0.855 | 0.687 | 1.429 | 0.834 | 1.312 |
| comment-review-long | 957 | gfm | yes | yes | 0.459 | 0.651 | 0.812 | 1.623 | 1.313 | 2.737 |
| comment-incident | 1124 | gfm | no | yes | 1.031 | 1.360 | 1.280 | 2.006 | 1.738 | 3.458 |
| guard-angle-link | 41 | commonmark | no | no | 0.163 | 0.166 | 0.282 | 0.754 | 0.267 | 0.403 |
| legacy-contributing | 9323 | gfm | no | yes | 7.067 | 8.340 | 12.445 | 14.725 | 15.610 | 31.268 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 6.559 | 8.319 | 13.105 | 15.993 | 15.495 | 31.924 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.259 | 1.761 | 3.044 | 3.520 | 3.372 | 6.048 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 1.881 | 2.407 | 3.407 | 4.191 | 3.770 | 7.250 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.370 | 5.527 | 10.684 | 11.966 | 11.765 | 23.875 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 1.579 | 2.214 | 3.948 | 4.389 | 4.196 | 7.684 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.195 | 4.068 | 7.330 | 6.785 | 8.056 | 13.699 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.622 | 2.026 | 3.436 | 4.563 | 4.518 | 8.995 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.373 | 3.077 | 5.725 | 6.125 | 6.767 | 11.893 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 4.939 | 6.307 | 9.855 | 12.445 | 10.909 | 23.426 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.195 | 3.832 | 4.470 | 6.083 | 4.778 | 8.944 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 0.887 | 1.251 | 1.759 | 2.574 | 2.463 | 4.830 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.391 | 0.576 | 0.783 | 1.427 | 0.949 | 1.812 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 30.467 | 35.511 | 42.437 | 49.338 | 44.287 | 62.049 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.835 | 11.163 | 12.113 | 16.644 | 18.412 | 35.089 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 5.420 | 5.794 | 9.269 | 12.380 | 12.922 | 26.822 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.232 | 4.956 | 5.640 | 8.510 | 8.129 | 19.727 |
| vue-docs-suspense | 8291 | gfm | no | yes | 5.982 | 7.401 | 14.807 | 14.159 | 14.090 | 29.593 |
| vue-docs-slots | 24211 | gfm | no | yes | 14.402 | 25.447 | 38.958 | 50.868 | 41.185 | 109.729 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 13.447 | 21.006 | 30.592 | 42.733 | 38.656 | 89.062 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.055 | 2.980 | 3.442 | 5.923 | 5.135 | 11.635 |
| vite-docs-performance | 8184 | gfm | no | yes | 5.440 | 7.299 | 9.327 | 12.939 | 13.252 | 27.037 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 41.496 | 48.883 | 58.187 | 66.979 | 62.251 | 124.926 |
| vite-docs-features | 39739 | gfm | no | no | 35.012 | 46.379 | 70.804 | 84.510 | 81.662 | 165.202 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.373 | 4.673 | 4.366 | 7.564 | 6.850 | 15.925 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 26.816 | 36.367 | 55.857 | 68.409 | 60.073 | 128.517 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 13.496 | 20.259 | 46.655 | 65.487 | 35.401 | 88.604 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 33.053 | 42.367 | 81.665 | 93.861 | 85.338 | 192.332 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.755 | 1.194 | 1.465 | 2.425 | 1.939 | 3.962 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.171 | 1.893 | 2.175 | 3.829 | 3.219 | 7.282 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 23.247 | 39.439 | 48.808 | 73.945 | 70.556 | 166.988 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 6.044 | 9.919 | 8.847 | 28.482 | 24.797 | 78.382 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.378 | 2.270 | 4.260 | 3.884 | 3.265 | 6.491 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 6.325 | 10.434 | 26.462 | 18.981 | 16.067 | 34.494 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 39.510 | 67.528 | 98.412 | 112.953 | 113.317 | 258.106 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 7.393 | 12.198 | 11.765 | 31.320 | 28.682 | 87.018 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.062 | 1.779 | 2.417 | 3.423 | 2.799 | 5.645 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.088 | 5.426 | 6.898 | 10.548 | 9.112 | 19.710 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 73.587 | 130.205 | 165.887 | 222.780 | 226.359 | 505.464 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 16.555 | 26.694 | 24.675 | 64.748 | 57.944 | 179.402 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.813 | 3.106 | 6.911 | 5.871 | 5.252 | 11.701 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.414 | 4.272 | 8.108 | 8.041 | 7.436 | 17.110 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 43.831 | 78.434 | 102.585 | 132.438 | 136.024 | 302.416 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 9.400 | 15.361 | 14.544 | 37.716 | 34.400 | 105.954 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 348.99 | 545.85 | 755.82 | 940.08 | 852.93 | 2050.48 |
| rotating-commonmark | reuse | 17 | 357.93 | 528.98 | 701.67 | 903.60 | 803.42 | 2003.68 |
| rotating-gfm | fresh | 40 | 521.00 | 700.08 | 1017.84 | 1102.96 | 905.10 | 1663.19 |
| rotating-gfm | reuse | 40 | 505.99 | 680.11 | 974.97 | 1096.88 | 876.05 | 1692.63 |
