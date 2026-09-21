# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.87× | 0.62× | 0.20× | 0.41× | 0.14× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.50× | 0.28× | 0.40× | 0.17× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.76× | 0.50× | 0.28× | 0.40× | 0.18× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.62× | 0.20× | 0.46× | 0.21× |
| comments (all workloads) | 12 | 1.00× | 0.83× | 0.62× | 0.20× | 0.47× | 0.21× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.32× | 0.24× | 0.31× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.57× | 0.34× | 0.26× | 0.32× | 0.16× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.63× | 0.25× | 0.29× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.00× | 0.63× | 0.25× | 0.29× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.61× | 0.39× | 0.50× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.86× | 0.61× | 0.39× | 0.50× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.57× | 0.40× | 0.56× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.86× | 0.57× | 0.40× | 0.56× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.78× | 0.48× | 0.19× | 0.65× | 0.49× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.50× | 0.33× | 0.39× | 0.18× |
| technical-docs (all workloads) | 22 | 1.00× | 0.77× | 0.50× | 0.33× | 0.39× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.40× | 0.25× | 0.30× | 0.12× |
| commonmark (all workloads) | 17 | 1.00× | 0.67× | 0.40× | 0.26× | 0.32× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.54× | 0.29× | 0.43× | 0.20× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.80× | 0.54× | 0.30× | 0.43× | 0.20× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.60× | 0.18× | 0.46× | 0.22× |
| <512 B (all workloads) | 12 | 1.00× | 0.80× | 0.59× | 0.19× | 0.48× | 0.24× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.48× | 0.29× | 0.38× | 0.19× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.70× | 0.48× | 0.29× | 0.38× | 0.19× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.31× | 0.36× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.71× | 0.43× | 0.31× | 0.36× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.39× | 0.44× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.81× | 0.54× | 0.39× | 0.44× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.53× | 0.27× | 0.35× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.79× | 0.48× | 0.29× | 0.35× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.96× | 0.70× | 0.18× | 0.37× | 0.12× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.54× | 0.27× | 0.39× | 0.16× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.78× | 0.53× | 0.27× | 0.39× | 0.17× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.70× | 0.16× | 0.41× | 0.16× |
| comments (all workloads) | 12 | 1.00× | 0.93× | 0.70× | 0.17× | 0.42× | 0.17× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.33× | 0.24× | 0.31× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.58× | 0.36× | 0.26× | 0.32× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.70× | 0.26× | 0.31× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.00× | 0.70× | 0.26× | 0.31× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.65× | 0.40× | 0.51× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.87× | 0.65× | 0.40× | 0.51× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.59× | 0.40× | 0.56× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.86× | 0.59× | 0.40× | 0.56× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.93× | 0.59× | 0.15× | 0.57× | 0.38× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.52× | 0.33× | 0.39× | 0.17× |
| technical-docs (all workloads) | 22 | 1.00× | 0.77× | 0.52× | 0.34× | 0.39× | 0.17× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.43× | 0.25× | 0.31× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.67× | 0.43× | 0.25× | 0.33× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.58× | 0.28× | 0.42× | 0.18× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.83× | 0.58× | 0.28× | 0.42× | 0.18× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.67× | 0.15× | 0.40× | 0.17× |
| <512 B (all workloads) | 12 | 1.00× | 0.91× | 0.66× | 0.16× | 0.42× | 0.18× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.28× | 0.38× | 0.18× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.71× | 0.52× | 0.28× | 0.38× | 0.18× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.45× | 0.31× | 0.36× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.71× | 0.45× | 0.31× | 0.36× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.57× | 0.39× | 0.45× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.82× | 0.57× | 0.39× | 0.45× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.57× | 0.27× | 0.36× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.78× | 0.51× | 0.29× | 0.36× | 0.12× |

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
| comment-ack | 37 | gfm | yes | yes | 0.103 | 0.155 | 0.157 | 0.981 | 0.182 | 0.289 |
| comment-question | 160 | gfm | yes | yes | 0.122 | 0.163 | 0.180 | 1.080 | 0.288 | 0.764 |
| comment-review | 282 | gfm | yes | yes | 0.174 | 0.212 | 0.296 | 1.266 | 0.488 | 1.306 |
| comment-links | 278 | gfm | yes | yes | 0.374 | 0.466 | 0.692 | 1.627 | 0.746 | 1.576 |
| comment-checklist | 287 | gfm | no | no | 0.503 | 0.604 | 0.825 | 1.761 | 0.898 | 1.779 |
| comment-quote | 290 | gfm | yes | yes | 0.229 | 0.274 | 0.349 | 1.358 | 0.565 | 1.428 |
| comment-unicode | 327 | gfm | yes | yes | 0.401 | 0.432 | 0.696 | 1.550 | 0.804 | 1.665 |
| comment-inline-code | 285 | gfm | yes | yes | 0.241 | 0.285 | 0.597 | 1.419 | 0.677 | 1.366 |
| comment-reproduction | 298 | gfm | yes | yes | 0.251 | 0.321 | 0.473 | 1.530 | 0.589 | 1.385 |
| comment-table | 310 | gfm | yes | yes | 1.001 | 0.989 | 0.974 | 2.354 | 1.205 | 2.126 |
| comment-review-long | 957 | gfm | yes | yes | 0.644 | 0.757 | 1.014 | 2.369 | 1.626 | 4.368 |
| comment-incident | 1124 | gfm | no | yes | 1.090 | 1.303 | 1.508 | 3.019 | 2.177 | 5.307 |
| guard-angle-link | 41 | commonmark | no | no | 0.224 | 0.286 | 0.471 | 1.180 | 0.346 | 0.456 |
| legacy-contributing | 9323 | gfm | no | yes | 9.125 | 10.867 | 16.216 | 23.296 | 22.320 | 46.431 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.702 | 10.802 | 17.025 | 25.125 | 22.148 | 45.422 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.744 | 2.405 | 4.112 | 5.727 | 4.718 | 8.639 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.358 | 2.996 | 4.350 | 6.695 | 5.392 | 10.459 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.030 | 7.550 | 14.010 | 19.139 | 16.340 | 31.709 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.282 | 3.027 | 5.023 | 7.171 | 5.710 | 10.377 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.343 | 5.143 | 9.875 | 10.949 | 11.656 | 19.999 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.081 | 2.526 | 4.502 | 6.934 | 6.301 | 13.203 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.213 | 3.989 | 7.373 | 9.506 | 9.265 | 17.942 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.510 | 7.678 | 12.926 | 19.450 | 15.298 | 35.484 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.083 | 4.491 | 5.691 | 9.132 | 6.395 | 12.037 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.178 | 1.559 | 2.474 | 3.962 | 3.401 | 7.465 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.507 | 0.784 | 1.042 | 2.155 | 1.282 | 2.516 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 50.111 | 49.049 | 59.307 | 80.034 | 64.218 | 102.985 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.697 | 13.168 | 16.892 | 24.182 | 25.462 | 57.642 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.912 | 7.045 | 12.634 | 18.114 | 18.001 | 48.348 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.475 | 6.296 | 7.219 | 12.907 | 11.295 | 30.726 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.828 | 9.804 | 19.011 | 22.335 | 20.035 | 42.447 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.147 | 35.697 | 65.268 | 82.132 | 62.557 | 141.638 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.102 | 28.248 | 42.027 | 67.302 | 53.800 | 135.124 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.676 | 3.905 | 4.553 | 8.494 | 7.316 | 18.258 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.333 | 9.058 | 12.106 | 19.573 | 17.848 | 41.383 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 58.536 | 69.792 | 78.786 | 116.252 | 92.132 | 186.175 |
| vite-docs-features | 39739 | gfm | no | no | 51.187 | 69.680 | 102.877 | 141.839 | 122.323 | 244.246 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.258 | 5.645 | 5.796 | 10.804 | 9.365 | 25.698 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 43.371 | 57.611 | 83.274 | 122.203 | 94.958 | 190.743 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.659 | 25.215 | 65.302 | 99.168 | 50.649 | 144.875 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 53.724 | 68.349 | 119.442 | 166.898 | 129.407 | 296.024 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.921 | 1.590 | 2.117 | 3.847 | 2.726 | 5.034 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.486 | 2.450 | 3.154 | 5.985 | 4.691 | 9.777 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.725 | 50.555 | 77.477 | 113.711 | 103.383 | 281.430 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.202 | 8.892 | 15.435 | 43.042 | 36.313 | 189.545 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.593 | 2.945 | 5.367 | 6.356 | 4.739 | 7.881 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.514 | 13.947 | 39.365 | 27.992 | 24.196 | 42.238 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 56.613 | 90.605 | 158.476 | 181.212 | 166.642 | 372.014 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 11.849 | 11.828 | 19.022 | 46.064 | 39.476 | 203.336 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.282 | 2.272 | 3.635 | 5.406 | 4.043 | 7.103 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.815 | 7.131 | 10.764 | 15.639 | 13.112 | 24.547 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 114.437 | 184.468 | 269.262 | 345.422 | 313.704 | 716.398 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 27.172 | 27.890 | 41.701 | 98.172 | 83.513 | 414.156 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.198 | 4.244 | 8.988 | 9.541 | 7.606 | 14.949 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.974 | 5.554 | 10.635 | 12.904 | 10.718 | 22.762 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 63.970 | 107.300 | 165.232 | 213.076 | 196.877 | 432.803 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 14.758 | 15.150 | 22.773 | 56.120 | 49.635 | 235.610 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.059 | 0.066 | 0.084 | 0.948 | 0.151 | 0.289 |
| comment-question | 160 | gfm | yes | yes | 0.076 | 0.078 | 0.104 | 1.051 | 0.260 | 0.764 |
| comment-review | 282 | gfm | yes | yes | 0.125 | 0.128 | 0.194 | 1.210 | 0.442 | 1.307 |
| comment-links | 278 | gfm | yes | yes | 0.318 | 0.374 | 0.499 | 1.546 | 0.662 | 1.578 |
| comment-checklist | 287 | gfm | no | no | 0.447 | 0.510 | 0.670 | 1.643 | 0.817 | 1.789 |
| comment-quote | 290 | gfm | yes | yes | 0.178 | 0.186 | 0.234 | 1.270 | 0.476 | 1.427 |
| comment-unicode | 327 | gfm | yes | yes | 0.343 | 0.335 | 0.440 | 1.483 | 0.728 | 1.666 |
| comment-inline-code | 285 | gfm | yes | yes | 0.194 | 0.197 | 0.438 | 1.342 | 0.603 | 1.369 |
| comment-reproduction | 298 | gfm | yes | yes | 0.203 | 0.234 | 0.361 | 1.447 | 0.524 | 1.386 |
| comment-table | 310 | gfm | yes | yes | 0.954 | 0.899 | 0.837 | 2.282 | 1.122 | 2.143 |
| comment-review-long | 957 | gfm | yes | yes | 0.582 | 0.653 | 0.801 | 2.284 | 1.539 | 4.354 |
| comment-incident | 1124 | gfm | no | yes | 1.030 | 1.187 | 1.293 | 2.876 | 2.048 | 5.301 |
| guard-angle-link | 41 | commonmark | no | no | 0.174 | 0.187 | 0.294 | 1.136 | 0.307 | 0.455 |
| legacy-contributing | 9323 | gfm | no | yes | 9.052 | 10.715 | 15.388 | 22.819 | 21.537 | 45.621 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.630 | 10.663 | 15.961 | 24.882 | 21.755 | 45.181 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.679 | 2.295 | 3.826 | 5.525 | 4.553 | 8.655 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.284 | 2.875 | 4.032 | 6.489 | 5.125 | 10.476 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.943 | 7.346 | 13.410 | 18.784 | 16.011 | 32.023 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.209 | 2.914 | 4.559 | 6.982 | 5.539 | 10.380 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.268 | 5.012 | 9.178 | 10.613 | 11.318 | 20.046 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.014 | 2.431 | 4.136 | 6.733 | 6.023 | 13.213 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.137 | 3.864 | 6.934 | 9.250 | 8.966 | 17.987 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.435 | 7.553 | 12.207 | 19.137 | 14.925 | 35.645 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.050 | 4.358 | 5.253 | 8.981 | 6.173 | 12.108 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.109 | 1.450 | 2.165 | 3.796 | 3.262 | 7.467 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.443 | 0.676 | 0.831 | 2.050 | 1.166 | 2.512 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 49.983 | 48.946 | 57.924 | 79.187 | 63.565 | 102.841 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.638 | 13.111 | 15.664 | 23.561 | 24.836 | 57.354 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.840 | 6.872 | 11.875 | 17.848 | 17.597 | 48.474 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.411 | 6.174 | 6.767 | 12.556 | 10.969 | 30.644 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.765 | 9.660 | 18.046 | 21.834 | 19.359 | 42.721 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.141 | 35.473 | 63.184 | 80.696 | 61.159 | 141.667 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.105 | 28.070 | 40.464 | 66.059 | 53.014 | 135.617 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.595 | 3.763 | 4.131 | 8.226 | 7.048 | 18.154 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.267 | 8.969 | 11.519 | 19.098 | 17.397 | 41.284 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 59.015 | 70.519 | 77.162 | 114.383 | 91.556 | 187.119 |
| vite-docs-features | 39739 | gfm | no | no | 51.025 | 70.086 | 99.744 | 140.459 | 121.019 | 244.352 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.169 | 5.492 | 5.280 | 10.553 | 9.056 | 25.620 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 41.959 | 56.764 | 79.421 | 119.858 | 92.908 | 190.392 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.526 | 24.948 | 63.292 | 97.909 | 49.148 | 144.628 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 53.178 | 69.383 | 115.791 | 165.577 | 127.554 | 295.163 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.853 | 1.468 | 1.768 | 3.729 | 2.608 | 5.049 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.419 | 2.334 | 2.761 | 5.822 | 4.551 | 9.766 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.961 | 50.011 | 71.344 | 111.363 | 101.346 | 281.236 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.122 | 8.762 | 13.383 | 41.102 | 34.355 | 188.718 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.535 | 2.823 | 4.978 | 6.189 | 4.588 | 7.885 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.446 | 13.785 | 37.941 | 27.745 | 23.877 | 42.099 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.571 | 90.094 | 153.196 | 178.337 | 165.840 | 370.833 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 11.786 | 11.697 | 16.768 | 44.638 | 37.717 | 203.327 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.226 | 2.161 | 3.248 | 5.257 | 3.903 | 7.091 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.744 | 7.017 | 9.947 | 15.394 | 12.874 | 24.597 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 113.782 | 183.135 | 261.790 | 342.937 | 309.821 | 719.799 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 27.043 | 27.485 | 37.879 | 94.419 | 80.570 | 413.484 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.132 | 4.129 | 8.487 | 9.349 | 7.454 | 14.933 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.928 | 5.443 | 10.094 | 12.611 | 10.454 | 22.761 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 63.928 | 106.318 | 159.662 | 209.539 | 192.657 | 431.017 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 14.745 | 15.052 | 20.514 | 53.809 | 46.781 | 235.107 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 439.56 | 641.86 | 1012.13 | 1280.29 | 1122.87 | 3097.19 |
| rotating-commonmark | reuse | 17 | 429.01 | 628.27 | 960.68 | 1254.32 | 1094.74 | 3093.70 |
| rotating-gfm | fresh | 40 | 653.49 | 843.57 | 1236.37 | 1433.39 | 1160.50 | 2309.45 |
| rotating-gfm | reuse | 40 | 639.47 | 825.21 | 1192.16 | 1407.42 | 1132.19 | 2306.82 |
