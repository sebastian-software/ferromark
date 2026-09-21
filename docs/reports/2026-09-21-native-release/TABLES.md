# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.94× | 0.67× | 0.25× | 0.44× | 0.16× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.52× | 0.31× | 0.41× | 0.18× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.79× | 0.51× | 0.31× | 0.41× | 0.18× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.68× | 0.25× | 0.51× | 0.23× |
| comments (all workloads) | 12 | 1.00× | 0.92× | 0.69× | 0.27× | 0.53× | 0.24× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.31× | 0.25× | 0.31× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.57× | 0.34× | 0.27× | 0.32× | 0.16× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.68× | 0.27× | 0.32× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.06× | 0.68× | 0.27× | 0.32× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.58× | 0.40× | 0.50× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.84× | 0.58× | 0.40× | 0.50× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.58× | 0.41× | 0.57× | 0.28× |
| reference (all workloads) | 4 | 1.00× | 0.88× | 0.58× | 0.41× | 0.57× | 0.28× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.77× | 0.45× | 0.22× | 0.62× | 0.47× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.51× | 0.35× | 0.40× | 0.18× |
| technical-docs (all workloads) | 22 | 1.00× | 0.79× | 0.51× | 0.35× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.40× | 0.26× | 0.31× | 0.12× |
| commonmark (all workloads) | 17 | 1.00× | 0.68× | 0.41× | 0.27× | 0.33× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.56× | 0.33× | 0.45× | 0.21× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.84× | 0.57× | 0.33× | 0.46× | 0.21× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.62× | 0.23× | 0.49× | 0.23× |
| <512 B (all workloads) | 12 | 1.00× | 0.86× | 0.62× | 0.24× | 0.52× | 0.25× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.50× | 0.32× | 0.40× | 0.20× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.72× | 0.50× | 0.32× | 0.40× | 0.20× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.45× | 0.33× | 0.37× | 0.18× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.73× | 0.45× | 0.33× | 0.37× | 0.18× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.55× | 0.41× | 0.46× | 0.21× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.84× | 0.55× | 0.41× | 0.46× | 0.21× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.54× | 0.28× | 0.36× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.80× | 0.49× | 0.30× | 0.36× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.05× | 0.76× | 0.22× | 0.41× | 0.13× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.56× | 0.30× | 0.41× | 0.17× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.82× | 0.55× | 0.30× | 0.41× | 0.17× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.78× | 0.21× | 0.46× | 0.18× |
| comments (all workloads) | 12 | 1.00× | 1.07× | 0.80× | 0.23× | 0.48× | 0.19× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.33× | 0.25× | 0.30× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.58× | 0.36× | 0.27× | 0.32× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.76× | 0.28× | 0.33× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.06× | 0.76× | 0.28× | 0.33× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.62× | 0.40× | 0.51× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.85× | 0.62× | 0.40× | 0.51× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.60× | 0.42× | 0.59× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.87× | 0.60× | 0.42× | 0.59× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.89× | 0.56× | 0.18× | 0.54× | 0.37× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.53× | 0.35× | 0.40× | 0.18× |
| technical-docs (all workloads) | 22 | 1.00× | 0.80× | 0.53× | 0.36× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.44× | 0.26× | 0.31× | 0.12× |
| commonmark (all workloads) | 17 | 1.00× | 0.68× | 0.44× | 0.26× | 0.33× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.61× | 0.31× | 0.44× | 0.19× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.88× | 0.61× | 0.32× | 0.45× | 0.19× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.72× | 0.19× | 0.43× | 0.18× |
| <512 B (all workloads) | 12 | 1.00× | 0.99× | 0.72× | 0.20× | 0.46× | 0.20× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.31× | 0.40× | 0.19× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.74× | 0.54× | 0.31× | 0.40× | 0.19× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.47× | 0.34× | 0.38× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.74× | 0.47× | 0.34× | 0.38× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.58× | 0.41× | 0.47× | 0.21× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.85× | 0.58× | 0.41× | 0.47× | 0.21× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.58× | 0.28× | 0.37× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.79× | 0.52× | 0.30× | 0.37× | 0.12× |

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
| comment-ack | 37 | gfm | yes | yes | 0.118 | 0.157 | 0.160 | 0.788 | 0.185 | 0.296 |
| comment-question | 160 | gfm | yes | yes | 0.130 | 0.166 | 0.184 | 0.881 | 0.292 | 0.773 |
| comment-review | 282 | gfm | yes | yes | 0.179 | 0.216 | 0.297 | 1.068 | 0.490 | 1.310 |
| comment-links | 278 | gfm | yes | yes | 0.360 | 0.475 | 0.699 | 1.439 | 0.754 | 1.598 |
| comment-checklist | 287 | gfm | no | no | 0.714 | 0.612 | 0.831 | 1.551 | 0.903 | 1.792 |
| comment-quote | 290 | gfm | yes | yes | 0.319 | 0.278 | 0.353 | 1.160 | 0.570 | 1.418 |
| comment-unicode | 327 | gfm | yes | yes | 0.411 | 0.436 | 0.708 | 1.358 | 0.805 | 1.685 |
| comment-inline-code | 285 | gfm | yes | yes | 0.247 | 0.285 | 0.603 | 1.212 | 0.677 | 1.370 |
| comment-reproduction | 298 | gfm | yes | yes | 0.259 | 0.324 | 0.475 | 1.339 | 0.591 | 1.384 |
| comment-table | 310 | gfm | yes | yes | 1.016 | 0.994 | 0.983 | 2.158 | 1.205 | 2.139 |
| comment-review-long | 957 | gfm | yes | yes | 0.836 | 0.763 | 1.023 | 2.182 | 1.638 | 4.384 |
| comment-incident | 1124 | gfm | no | yes | 1.393 | 1.329 | 1.522 | 2.846 | 2.223 | 5.428 |
| guard-angle-link | 41 | commonmark | no | no | 0.216 | 0.280 | 0.476 | 0.984 | 0.346 | 0.456 |
| legacy-contributing | 9323 | gfm | no | yes | 9.639 | 10.771 | 16.312 | 22.651 | 21.986 | 45.998 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.639 | 10.886 | 17.463 | 24.639 | 22.205 | 45.993 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.705 | 2.402 | 4.144 | 5.504 | 4.730 | 8.696 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.490 | 3.060 | 4.458 | 6.519 | 5.463 | 10.740 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.884 | 7.534 | 14.087 | 18.719 | 16.467 | 32.200 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.282 | 3.042 | 5.058 | 6.986 | 5.684 | 10.396 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 5.045 | 5.171 | 10.049 | 10.636 | 11.761 | 20.495 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.063 | 2.554 | 4.573 | 6.783 | 6.296 | 13.465 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.152 | 4.003 | 7.469 | 9.331 | 9.268 | 18.344 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.419 | 7.717 | 13.128 | 19.190 | 15.396 | 35.853 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.017 | 4.517 | 5.838 | 8.942 | 6.377 | 12.140 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.181 | 1.600 | 2.529 | 3.844 | 3.462 | 7.659 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.494 | 0.805 | 1.069 | 1.995 | 1.303 | 2.549 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 49.840 | 49.786 | 60.878 | 79.447 | 63.806 | 104.254 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.129 | 13.406 | 17.236 | 24.105 | 26.012 | 59.092 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.351 | 7.011 | 12.800 | 17.850 | 18.022 | 48.314 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.982 | 6.403 | 7.446 | 12.842 | 11.487 | 31.185 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.948 | 9.857 | 19.253 | 22.108 | 20.099 | 42.882 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.909 | 35.392 | 66.188 | 81.200 | 62.392 | 141.782 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.743 | 28.702 | 44.925 | 67.576 | 54.194 | 137.398 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.812 | 3.962 | 4.630 | 8.346 | 7.404 | 18.822 |
| vite-docs-performance | 8184 | gfm | no | yes | 8.004 | 9.102 | 12.336 | 19.192 | 18.112 | 41.768 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 70.795 | 72.658 | 81.381 | 116.067 | 92.461 | 186.294 |
| vite-docs-features | 39739 | gfm | no | no | 53.063 | 69.504 | 105.458 | 140.768 | 120.258 | 244.524 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.526 | 5.672 | 5.825 | 10.574 | 9.348 | 26.166 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 40.077 | 57.397 | 83.148 | 119.841 | 93.107 | 189.319 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.599 | 24.961 | 65.345 | 99.171 | 51.222 | 145.147 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 52.219 | 73.373 | 125.230 | 168.656 | 128.356 | 299.919 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.900 | 1.591 | 2.135 | 3.669 | 2.715 | 5.050 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.445 | 2.422 | 3.161 | 5.740 | 4.684 | 9.813 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.351 | 49.883 | 75.270 | 111.814 | 102.874 | 280.174 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.034 | 8.913 | 15.393 | 42.422 | 36.108 | 188.883 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.620 | 2.984 | 5.430 | 6.235 | 4.815 | 7.991 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.532 | 13.933 | 39.434 | 27.759 | 24.396 | 42.506 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 58.076 | 94.078 | 166.410 | 184.767 | 170.865 | 380.115 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 12.311 | 12.019 | 19.009 | 45.708 | 39.403 | 203.833 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.274 | 2.284 | 3.669 | 5.121 | 4.039 | 7.109 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.786 | 7.144 | 10.848 | 15.377 | 13.178 | 24.923 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 119.757 | 184.501 | 271.722 | 347.103 | 313.137 | 719.353 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 30.951 | 27.832 | 41.363 | 97.751 | 83.580 | 414.837 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.208 | 4.271 | 9.138 | 9.369 | 7.757 | 15.140 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.963 | 5.504 | 10.667 | 12.555 | 10.701 | 22.858 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 65.609 | 107.663 | 165.693 | 211.397 | 196.472 | 434.475 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 17.021 | 15.583 | 22.911 | 56.444 | 49.930 | 237.687 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.068 | 0.067 | 0.084 | 0.744 | 0.151 | 0.292 |
| comment-question | 160 | gfm | yes | yes | 0.083 | 0.077 | 0.103 | 0.842 | 0.260 | 0.765 |
| comment-review | 282 | gfm | yes | yes | 0.131 | 0.128 | 0.189 | 1.008 | 0.439 | 1.311 |
| comment-links | 278 | gfm | yes | yes | 0.300 | 0.377 | 0.510 | 1.348 | 0.663 | 1.582 |
| comment-checklist | 287 | gfm | no | no | 0.658 | 0.511 | 0.663 | 1.427 | 0.819 | 1.788 |
| comment-quote | 290 | gfm | yes | yes | 0.268 | 0.188 | 0.235 | 1.078 | 0.490 | 1.427 |
| comment-unicode | 327 | gfm | yes | yes | 0.353 | 0.335 | 0.450 | 1.292 | 0.730 | 1.681 |
| comment-inline-code | 285 | gfm | yes | yes | 0.199 | 0.196 | 0.452 | 1.125 | 0.601 | 1.376 |
| comment-reproduction | 298 | gfm | yes | yes | 0.210 | 0.237 | 0.357 | 1.250 | 0.522 | 1.404 |
| comment-table | 310 | gfm | yes | yes | 0.970 | 0.907 | 0.847 | 2.082 | 1.127 | 2.143 |
| comment-review-long | 957 | gfm | yes | yes | 0.769 | 0.654 | 0.810 | 2.081 | 1.564 | 4.399 |
| comment-incident | 1124 | gfm | no | yes | 1.324 | 1.211 | 1.322 | 2.719 | 2.079 | 5.410 |
| guard-angle-link | 41 | commonmark | no | no | 0.169 | 0.189 | 0.301 | 0.945 | 0.312 | 0.461 |
| legacy-contributing | 9323 | gfm | no | yes | 9.706 | 10.803 | 15.767 | 22.375 | 21.940 | 47.207 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.519 | 10.748 | 16.292 | 24.364 | 21.531 | 46.086 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.640 | 2.334 | 3.878 | 5.326 | 4.567 | 8.742 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.403 | 2.899 | 4.086 | 6.193 | 5.168 | 10.576 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.831 | 7.371 | 13.529 | 18.562 | 15.898 | 32.437 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.234 | 2.932 | 4.634 | 6.775 | 5.524 | 10.482 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 5.001 | 5.056 | 9.363 | 10.478 | 11.457 | 20.466 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.978 | 2.445 | 4.166 | 6.513 | 6.023 | 13.297 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.118 | 3.939 | 7.080 | 9.124 | 9.089 | 18.567 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.439 | 7.571 | 12.386 | 18.923 | 15.003 | 36.351 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.037 | 4.449 | 5.450 | 8.850 | 6.201 | 12.290 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.099 | 1.473 | 2.207 | 3.642 | 3.290 | 7.600 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.425 | 0.684 | 0.848 | 1.864 | 1.194 | 2.506 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 50.119 | 49.691 | 59.822 | 79.127 | 62.266 | 104.735 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.015 | 13.078 | 15.647 | 22.987 | 24.836 | 57.830 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.353 | 6.951 | 12.104 | 17.760 | 17.864 | 49.206 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.899 | 6.202 | 6.910 | 12.431 | 11.110 | 30.876 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.854 | 9.645 | 18.139 | 21.736 | 19.351 | 42.964 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.851 | 35.332 | 65.413 | 79.539 | 60.883 | 142.355 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.482 | 28.277 | 42.101 | 65.150 | 52.583 | 134.814 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.729 | 3.791 | 4.190 | 8.032 | 7.096 | 18.477 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.930 | 8.935 | 11.774 | 18.729 | 17.458 | 41.784 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 70.963 | 72.738 | 80.029 | 114.980 | 91.087 | 190.386 |
| vite-docs-features | 39739 | gfm | no | no | 54.091 | 71.154 | 100.301 | 139.260 | 119.526 | 246.629 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.525 | 5.539 | 5.457 | 10.457 | 9.165 | 26.448 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 41.332 | 60.874 | 83.379 | 120.719 | 93.342 | 194.545 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.682 | 24.889 | 63.399 | 97.769 | 49.814 | 145.933 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 50.203 | 71.288 | 119.543 | 165.381 | 126.508 | 297.688 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.847 | 1.472 | 1.800 | 3.558 | 2.645 | 5.114 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.400 | 2.314 | 2.795 | 5.622 | 4.546 | 9.815 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.842 | 50.256 | 72.444 | 110.724 | 101.496 | 282.087 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.963 | 8.777 | 13.252 | 40.710 | 34.256 | 188.581 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.533 | 2.805 | 4.985 | 6.002 | 4.626 | 7.920 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.459 | 13.832 | 38.084 | 27.336 | 23.966 | 42.399 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.533 | 90.277 | 156.111 | 177.791 | 165.218 | 374.689 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 12.184 | 11.901 | 16.804 | 44.287 | 37.634 | 203.778 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.218 | 2.171 | 3.251 | 4.974 | 3.924 | 7.150 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.699 | 6.971 | 10.011 | 15.026 | 12.920 | 24.754 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 120.002 | 187.641 | 266.339 | 344.577 | 314.483 | 727.731 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 31.085 | 28.001 | 37.955 | 95.264 | 81.484 | 420.939 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.134 | 4.097 | 8.518 | 9.036 | 7.441 | 14.939 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.910 | 5.437 | 10.209 | 12.413 | 10.601 | 23.141 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 64.968 | 106.376 | 159.969 | 206.282 | 192.539 | 434.364 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 16.794 | 15.325 | 20.496 | 53.672 | 46.884 | 237.893 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 462.64 | 654.14 | 1031.56 | 1279.25 | 1131.72 | 3129.99 |
| rotating-commonmark | reuse | 17 | 444.41 | 630.35 | 966.70 | 1240.85 | 1094.85 | 3110.26 |
| rotating-gfm | fresh | 40 | 697.79 | 869.82 | 1321.48 | 1428.48 | 1175.54 | 2330.52 |
| rotating-gfm | reuse | 40 | 672.99 | 826.46 | 1198.18 | 1398.83 | 1138.90 | 2327.08 |
