# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.84× | 0.60× | 0.22× | 0.39× | 0.13× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.48× | 0.29× | 0.38× | 0.16× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.72× | 0.47× | 0.29× | 0.38× | 0.16× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.60× | 0.22× | 0.45× | 0.19× |
| comments (all workloads) | 12 | 1.00× | 0.80× | 0.60× | 0.23× | 0.45× | 0.20× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.30× | 0.24× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.54× | 0.32× | 0.26× | 0.30× | 0.14× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.61× | 0.24× | 0.28× | 0.05× |
| plain-prose (all workloads) | 4 | 1.00× | 0.96× | 0.61× | 0.24× | 0.28× | 0.05× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.58× | 0.39× | 0.48× | 0.23× |
| readme (all workloads) | 2 | 1.00× | 0.82× | 0.58× | 0.39× | 0.48× | 0.23× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.55× | 0.38× | 0.54× | 0.25× |
| reference (all workloads) | 4 | 1.00× | 0.82× | 0.55× | 0.38× | 0.54× | 0.25× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.75× | 0.44× | 0.22× | 0.59× | 0.44× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.47× | 0.33× | 0.37× | 0.16× |
| technical-docs (all workloads) | 22 | 1.00× | 0.73× | 0.47× | 0.33× | 0.37× | 0.16× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.38× | 0.24× | 0.29× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.63× | 0.38× | 0.25× | 0.31× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.30× | 0.41× | 0.18× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.77× | 0.52× | 0.30× | 0.41× | 0.18× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.58× | 0.21× | 0.44× | 0.20× |
| <512 B (all workloads) | 12 | 1.00× | 0.77× | 0.56× | 0.22× | 0.46× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.46× | 0.29× | 0.36× | 0.17× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.66× | 0.46× | 0.29× | 0.36× | 0.17× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.41× | 0.31× | 0.34× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.68× | 0.41× | 0.31× | 0.34× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.38× | 0.43× | 0.19× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.79× | 0.52× | 0.38× | 0.43× | 0.19× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.51× | 0.26× | 0.33× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.74× | 0.46× | 0.28× | 0.33× | 0.11× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.92× | 0.67× | 0.19× | 0.36× | 0.11× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.52× | 0.27× | 0.37× | 0.15× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.74× | 0.51× | 0.28× | 0.37× | 0.15× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.67× | 0.19× | 0.39× | 0.15× |
| comments (all workloads) | 12 | 1.00× | 0.90× | 0.67× | 0.19× | 0.40× | 0.16× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.31× | 0.23× | 0.29× | 0.14× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.54× | 0.34× | 0.26× | 0.30× | 0.14× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.68× | 0.25× | 0.30× | 0.05× |
| plain-prose (all workloads) | 4 | 1.00× | 0.96× | 0.68× | 0.25× | 0.30× | 0.05× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.62× | 0.39× | 0.49× | 0.23× |
| readme (all workloads) | 2 | 1.00× | 0.83× | 0.62× | 0.39× | 0.49× | 0.23× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.57× | 0.39× | 0.55× | 0.25× |
| reference (all workloads) | 4 | 1.00× | 0.82× | 0.57× | 0.39× | 0.55× | 0.25× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.84× | 0.53× | 0.17× | 0.50× | 0.33× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.50× | 0.33× | 0.37× | 0.16× |
| technical-docs (all workloads) | 22 | 1.00× | 0.74× | 0.50× | 0.33× | 0.37× | 0.16× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.41× | 0.24× | 0.29× | 0.10× |
| commonmark (all workloads) | 17 | 1.00× | 0.64× | 0.41× | 0.25× | 0.31× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.56× | 0.29× | 0.40× | 0.17× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.80× | 0.56× | 0.29× | 0.40× | 0.17× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.64× | 0.17× | 0.38× | 0.15× |
| <512 B (all workloads) | 12 | 1.00× | 0.87× | 0.63× | 0.18× | 0.40× | 0.17× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.49× | 0.28× | 0.36× | 0.16× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.67× | 0.49× | 0.28× | 0.36× | 0.16× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.31× | 0.35× | 0.15× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.68× | 0.43× | 0.31× | 0.35× | 0.15× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.39× | 0.44× | 0.19× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.79× | 0.54× | 0.39× | 0.44× | 0.19× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.55× | 0.26× | 0.34× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.74× | 0.49× | 0.28× | 0.34× | 0.11× |

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
| comment-ack | 37 | gfm | yes | yes | 0.101 | 0.152 | 0.158 | 0.765 | 0.188 | 0.299 |
| comment-question | 160 | gfm | yes | yes | 0.117 | 0.162 | 0.180 | 0.864 | 0.287 | 0.802 |
| comment-review | 282 | gfm | yes | yes | 0.164 | 0.212 | 0.290 | 1.049 | 0.488 | 1.368 |
| comment-links | 278 | gfm | yes | yes | 0.343 | 0.466 | 0.681 | 1.404 | 0.744 | 1.655 |
| comment-checklist | 287 | gfm | no | no | 0.481 | 0.599 | 0.824 | 1.525 | 0.901 | 1.852 |
| comment-quote | 290 | gfm | yes | yes | 0.226 | 0.276 | 0.346 | 1.127 | 0.566 | 1.483 |
| comment-unicode | 327 | gfm | yes | yes | 0.388 | 0.424 | 0.699 | 1.325 | 0.800 | 1.750 |
| comment-inline-code | 285 | gfm | yes | yes | 0.232 | 0.281 | 0.595 | 1.195 | 0.681 | 1.434 |
| comment-reproduction | 298 | gfm | yes | yes | 0.244 | 0.319 | 0.469 | 1.323 | 0.591 | 1.418 |
| comment-table | 310 | gfm | yes | yes | 0.989 | 0.981 | 0.974 | 2.128 | 1.201 | 2.190 |
| comment-review-long | 957 | gfm | yes | yes | 0.606 | 0.754 | 1.006 | 2.147 | 1.624 | 4.645 |
| comment-incident | 1124 | gfm | no | yes | 1.056 | 1.297 | 1.496 | 2.793 | 2.174 | 5.594 |
| guard-angle-link | 41 | commonmark | no | no | 0.207 | 0.275 | 0.471 | 0.958 | 0.352 | 0.475 |
| legacy-contributing | 9323 | gfm | no | yes | 8.701 | 10.768 | 16.111 | 22.720 | 22.173 | 48.244 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.350 | 10.734 | 17.045 | 24.536 | 22.171 | 48.067 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.676 | 2.372 | 4.090 | 5.452 | 4.692 | 9.024 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.262 | 2.982 | 4.337 | 6.376 | 5.298 | 10.944 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.803 | 7.437 | 13.935 | 18.768 | 16.408 | 33.427 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.227 | 3.040 | 4.995 | 6.899 | 5.704 | 10.851 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.187 | 5.102 | 9.891 | 10.613 | 11.627 | 21.157 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.979 | 2.532 | 4.519 | 6.673 | 6.281 | 14.001 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.047 | 3.953 | 7.350 | 9.225 | 9.254 | 19.246 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.233 | 7.582 | 12.884 | 19.076 | 15.380 | 37.340 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.867 | 4.437 | 5.695 | 8.831 | 6.371 | 12.470 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.115 | 1.549 | 2.452 | 3.715 | 3.428 | 7.867 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.466 | 0.788 | 1.040 | 1.934 | 1.295 | 2.587 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 49.591 | 49.158 | 58.927 | 78.930 | 62.786 | 105.317 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.229 | 13.144 | 16.749 | 23.829 | 25.471 | 60.697 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.736 | 6.975 | 12.659 | 17.838 | 17.958 | 50.464 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.149 | 6.241 | 7.245 | 12.521 | 11.280 | 32.252 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.601 | 9.767 | 18.953 | 22.019 | 19.929 | 44.893 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.488 | 35.428 | 66.602 | 81.655 | 62.748 | 146.673 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 17.312 | 28.270 | 44.954 | 66.994 | 54.001 | 139.556 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.502 | 3.865 | 4.533 | 8.166 | 7.336 | 19.289 |
| vite-docs-performance | 8184 | gfm | no | yes | 6.904 | 9.052 | 12.184 | 19.031 | 17.985 | 43.626 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 55.311 | 71.890 | 81.487 | 115.380 | 92.144 | 190.360 |
| vite-docs-features | 39739 | gfm | no | no | 48.239 | 70.966 | 106.864 | 140.472 | 121.366 | 248.784 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.069 | 5.613 | 5.767 | 10.494 | 9.282 | 27.369 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 40.176 | 58.096 | 84.378 | 121.569 | 92.971 | 195.109 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.567 | 25.009 | 64.683 | 99.418 | 51.751 | 145.316 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 48.318 | 70.849 | 121.678 | 165.109 | 127.542 | 301.023 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.857 | 1.581 | 2.108 | 3.622 | 2.723 | 5.247 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.378 | 2.419 | 3.139 | 5.708 | 4.660 | 10.301 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.127 | 51.318 | 76.214 | 111.859 | 102.827 | 291.243 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.826 | 8.864 | 15.240 | 42.701 | 36.105 | 195.956 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.516 | 2.933 | 5.329 | 6.063 | 4.753 | 8.159 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.113 | 13.908 | 39.519 | 27.457 | 24.231 | 43.797 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 53.520 | 93.453 | 159.324 | 180.626 | 167.205 | 384.144 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 11.471 | 11.934 | 18.906 | 45.631 | 39.181 | 209.390 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.221 | 2.270 | 3.593 | 5.086 | 4.051 | 7.461 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.579 | 7.058 | 10.763 | 15.202 | 13.154 | 25.658 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 111.309 | 184.994 | 269.841 | 344.151 | 314.548 | 740.207 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 26.265 | 27.703 | 41.592 | 98.115 | 83.203 | 429.533 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.066 | 4.171 | 8.952 | 9.193 | 7.598 | 15.744 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.833 | 5.502 | 10.692 | 12.528 | 10.697 | 24.084 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 61.833 | 108.957 | 164.907 | 211.529 | 197.783 | 445.120 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 14.302 | 15.272 | 22.906 | 56.334 | 50.057 | 247.182 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.056 | 0.066 | 0.083 | 0.729 | 0.152 | 0.297 |
| comment-question | 160 | gfm | yes | yes | 0.072 | 0.077 | 0.103 | 0.822 | 0.257 | 0.801 |
| comment-review | 282 | gfm | yes | yes | 0.118 | 0.127 | 0.190 | 0.990 | 0.440 | 1.370 |
| comment-links | 278 | gfm | yes | yes | 0.290 | 0.372 | 0.498 | 1.320 | 0.660 | 1.645 |
| comment-checklist | 287 | gfm | no | no | 0.426 | 0.507 | 0.660 | 1.406 | 0.813 | 1.842 |
| comment-quote | 290 | gfm | yes | yes | 0.177 | 0.186 | 0.233 | 1.049 | 0.479 | 1.485 |
| comment-unicode | 327 | gfm | yes | yes | 0.326 | 0.327 | 0.440 | 1.256 | 0.727 | 1.730 |
| comment-inline-code | 285 | gfm | yes | yes | 0.187 | 0.195 | 0.452 | 1.115 | 0.599 | 1.433 |
| comment-reproduction | 298 | gfm | yes | yes | 0.194 | 0.233 | 0.348 | 1.235 | 0.520 | 1.414 |
| comment-table | 310 | gfm | yes | yes | 0.930 | 0.892 | 0.836 | 2.066 | 1.119 | 2.189 |
| comment-review-long | 957 | gfm | yes | yes | 0.541 | 0.653 | 0.799 | 2.049 | 1.532 | 4.645 |
| comment-incident | 1124 | gfm | no | yes | 0.988 | 1.181 | 1.292 | 2.655 | 2.031 | 5.651 |
| guard-angle-link | 41 | commonmark | no | no | 0.155 | 0.184 | 0.292 | 0.911 | 0.311 | 0.472 |
| legacy-contributing | 9323 | gfm | no | yes | 8.673 | 10.662 | 15.398 | 22.233 | 21.526 | 48.211 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.318 | 10.615 | 15.958 | 24.038 | 21.586 | 48.086 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.603 | 2.265 | 3.767 | 5.250 | 4.561 | 8.991 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.203 | 2.880 | 4.027 | 6.190 | 5.087 | 10.927 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.743 | 7.294 | 13.314 | 18.402 | 15.984 | 33.754 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.150 | 2.893 | 4.535 | 6.698 | 5.499 | 10.794 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.111 | 4.987 | 9.165 | 10.300 | 11.401 | 21.020 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.929 | 2.438 | 4.116 | 6.435 | 6.026 | 14.116 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.997 | 3.847 | 6.891 | 8.975 | 8.981 | 19.189 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.158 | 7.472 | 12.070 | 18.737 | 14.808 | 37.384 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.855 | 4.349 | 5.287 | 8.650 | 6.193 | 12.496 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.047 | 1.439 | 2.148 | 3.575 | 3.248 | 7.846 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.408 | 0.680 | 0.830 | 1.835 | 1.176 | 2.615 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 49.349 | 48.674 | 57.688 | 77.691 | 62.489 | 105.099 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.168 | 13.025 | 15.607 | 23.235 | 24.854 | 60.440 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.647 | 6.803 | 11.931 | 17.674 | 17.568 | 50.467 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.098 | 6.160 | 6.832 | 12.223 | 10.987 | 32.599 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.509 | 9.533 | 17.952 | 21.480 | 19.302 | 44.865 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.453 | 35.204 | 64.679 | 79.687 | 60.752 | 145.737 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 17.298 | 28.003 | 42.639 | 65.436 | 52.681 | 139.466 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.436 | 3.751 | 4.131 | 7.959 | 7.058 | 19.304 |
| vite-docs-performance | 8184 | gfm | no | yes | 6.905 | 8.993 | 11.544 | 18.665 | 17.361 | 43.380 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 55.898 | 72.956 | 78.746 | 113.343 | 90.943 | 190.449 |
| vite-docs-features | 39739 | gfm | no | no | 47.039 | 70.836 | 101.296 | 139.160 | 119.726 | 246.810 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.983 | 5.461 | 5.268 | 10.259 | 9.046 | 27.300 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 39.041 | 58.695 | 80.719 | 117.954 | 91.335 | 193.956 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.601 | 24.819 | 62.163 | 97.534 | 49.420 | 145.394 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 47.498 | 71.405 | 118.445 | 163.774 | 127.025 | 302.118 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.791 | 1.461 | 1.770 | 3.501 | 2.604 | 5.244 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.315 | 2.309 | 2.756 | 5.566 | 4.504 | 10.288 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.487 | 50.968 | 72.463 | 110.385 | 101.376 | 289.864 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.750 | 8.681 | 13.200 | 40.772 | 34.126 | 195.785 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.438 | 2.798 | 4.942 | 5.884 | 4.592 | 8.125 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.016 | 13.748 | 38.107 | 27.171 | 23.871 | 43.890 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 54.627 | 92.207 | 154.942 | 177.352 | 165.683 | 382.149 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 11.386 | 11.999 | 16.912 | 44.296 | 37.490 | 211.192 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.138 | 2.150 | 3.235 | 4.922 | 3.884 | 7.407 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.498 | 6.926 | 9.867 | 14.930 | 12.844 | 25.746 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 112.269 | 185.041 | 260.936 | 340.912 | 310.451 | 743.691 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 26.316 | 27.576 | 37.547 | 93.673 | 80.038 | 426.489 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.012 | 4.074 | 8.455 | 9.016 | 7.427 | 15.702 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.746 | 5.394 | 10.056 | 12.249 | 10.423 | 24.188 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 61.857 | 110.200 | 161.420 | 206.599 | 193.044 | 451.576 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 14.135 | 15.145 | 20.325 | 53.235 | 46.580 | 243.778 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 427.14 | 650.29 | 1020.64 | 1266.53 | 1125.10 | 3207.47 |
| rotating-commonmark | reuse | 17 | 414.47 | 630.01 | 972.03 | 1232.43 | 1089.50 | 3204.21 |
| rotating-gfm | fresh | 40 | 632.75 | 850.79 | 1256.89 | 1426.53 | 1166.79 | 2357.60 |
| rotating-gfm | reuse | 40 | 616.17 | 831.81 | 1196.53 | 1403.09 | 1137.81 | 2352.28 |
