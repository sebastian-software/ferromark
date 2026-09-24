# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.70× | 0.60× | 0.22× | 0.40× | 0.20× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.50× | 0.31× | 0.41× | 0.22× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.69× | 0.49× | 0.31× | 0.41× | 0.22× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.61× | 0.24× | 0.50× | 0.29× |
| comments (all workloads) | 12 | 1.00× | 0.72× | 0.61× | 0.24× | 0.51× | 0.29× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.36× | 0.30× | 0.38× | 0.20× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.64× | 0.39× | 0.30× | 0.36× | 0.19× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.60× | 0.21× | 0.24× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.64× | 0.60× | 0.21× | 0.24× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.58× | 0.44× | 0.51× | 0.31× |
| readme (all workloads) | 2 | 1.00× | 0.76× | 0.58× | 0.44× | 0.51× | 0.31× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.52× | 0.39× | 0.50× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.71× | 0.52× | 0.39× | 0.50× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.74× | 0.41× | 0.22× | 0.60× | 0.50× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.48× | 0.36× | 0.39× | 0.22× |
| technical-docs (all workloads) | 22 | 1.00× | 0.69× | 0.48× | 0.36× | 0.39× | 0.22× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.43× | 0.27× | 0.33× | 0.15× |
| commonmark (all workloads) | 17 | 1.00× | 0.65× | 0.43× | 0.27× | 0.34× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.33× | 0.44× | 0.25× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.70× | 0.52× | 0.33× | 0.44× | 0.25× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.60× | 0.22× | 0.50× | 0.29× |
| <512 B (all workloads) | 12 | 1.00× | 0.72× | 0.57× | 0.23× | 0.51× | 0.31× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.48× | 0.34× | 0.43× | 0.24× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.66× | 0.48× | 0.34× | 0.43× | 0.24× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.44× | 0.35× | 0.39× | 0.22× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.69× | 0.44× | 0.35× | 0.39× | 0.22× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.41× | 0.45× | 0.25× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.73× | 0.52× | 0.41× | 0.45× | 0.25× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.24× | 0.29× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.65× | 0.48× | 0.27× | 0.31× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.76× | 0.64× | 0.19× | 0.37× | 0.17× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.52× | 0.30× | 0.40× | 0.20× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.71× | 0.52× | 0.30× | 0.40× | 0.20× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.64× | 0.20× | 0.44× | 0.23× |
| comments (all workloads) | 12 | 1.00× | 0.78× | 0.64× | 0.21× | 0.45× | 0.23× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.39× | 0.30× | 0.38× | 0.19× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.65× | 0.41× | 0.30× | 0.37× | 0.18× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.66× | 0.22× | 0.25× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.65× | 0.66× | 0.22× | 0.25× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.60× | 0.44× | 0.51× | 0.31× |
| readme (all workloads) | 2 | 1.00× | 0.77× | 0.60× | 0.44× | 0.51× | 0.31× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.39× | 0.51× | 0.27× |
| reference (all workloads) | 4 | 1.00× | 0.72× | 0.53× | 0.39× | 0.51× | 0.27× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.80× | 0.47× | 0.17× | 0.50× | 0.35× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.50× | 0.36× | 0.39× | 0.21× |
| technical-docs (all workloads) | 22 | 1.00× | 0.70× | 0.50× | 0.36× | 0.39× | 0.21× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.46× | 0.27× | 0.33× | 0.15× |
| commonmark (all workloads) | 17 | 1.00× | 0.66× | 0.46× | 0.27× | 0.34× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.55× | 0.31× | 0.42× | 0.23× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.73× | 0.55× | 0.31× | 0.42× | 0.23× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.63× | 0.18× | 0.43× | 0.23× |
| <512 B (all workloads) | 12 | 1.00× | 0.79× | 0.61× | 0.19× | 0.44× | 0.24× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.33× | 0.42× | 0.23× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.68× | 0.52× | 0.33× | 0.42× | 0.23× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.45× | 0.35× | 0.39× | 0.21× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.69× | 0.45× | 0.35× | 0.39× | 0.21× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.42× | 0.45× | 0.25× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.73× | 0.54× | 0.42× | 0.45× | 0.25× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.53× | 0.25× | 0.30× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.65× | 0.51× | 0.27× | 0.32× | 0.14× |

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
| comment-ack | 37 | gfm | yes | yes | 0.205 | 0.281 | 0.226 | 1.289 | 0.304 | 0.386 |
| comment-question | 160 | gfm | yes | yes | 0.204 | 0.302 | 0.274 | 1.459 | 0.423 | 0.882 |
| comment-review | 282 | gfm | yes | yes | 0.285 | 0.391 | 0.439 | 1.725 | 0.693 | 1.442 |
| comment-links | 278 | gfm | yes | yes | 0.594 | 0.842 | 1.230 | 2.271 | 1.166 | 2.202 |
| comment-checklist | 287 | gfm | no | no | 0.794 | 1.202 | 1.395 | 2.371 | 1.422 | 2.325 |
| comment-quote | 290 | gfm | yes | yes | 0.381 | 0.519 | 0.592 | 1.846 | 0.822 | 1.590 |
| comment-unicode | 327 | gfm | yes | yes | 0.671 | 0.755 | 1.179 | 2.204 | 1.227 | 1.952 |
| comment-inline-code | 285 | gfm | yes | yes | 0.364 | 0.510 | 1.048 | 1.925 | 0.998 | 1.529 |
| comment-reproduction | 298 | gfm | yes | yes | 0.388 | 0.583 | 0.799 | 2.054 | 0.883 | 1.708 |
| comment-table | 310 | gfm | yes | yes | 1.563 | 1.930 | 1.654 | 3.213 | 1.837 | 2.536 |
| comment-review-long | 957 | gfm | yes | yes | 0.966 | 1.512 | 1.915 | 3.402 | 2.421 | 4.890 |
| comment-incident | 1124 | gfm | no | yes | 1.937 | 2.887 | 2.770 | 4.253 | 3.398 | 6.011 |
| guard-angle-link | 41 | commonmark | no | no | 0.367 | 0.494 | 0.897 | 1.666 | 0.611 | 0.735 |
| legacy-contributing | 9323 | gfm | no | yes | 13.650 | 17.585 | 25.001 | 30.268 | 31.498 | 50.361 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 13.010 | 17.498 | 26.364 | 34.466 | 31.888 | 53.194 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.531 | 3.932 | 6.255 | 7.634 | 7.149 | 10.860 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 3.694 | 5.190 | 6.890 | 9.222 | 8.035 | 13.043 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 8.485 | 11.954 | 21.088 | 25.757 | 24.467 | 38.793 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 3.147 | 4.810 | 7.970 | 9.796 | 8.562 | 13.494 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.403 | 8.383 | 15.163 | 15.204 | 17.169 | 24.080 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 3.276 | 4.320 | 7.120 | 9.582 | 9.125 | 15.749 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.725 | 6.429 | 11.576 | 13.044 | 13.433 | 22.232 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 9.668 | 13.036 | 20.326 | 27.255 | 22.617 | 41.334 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.480 | 8.286 | 9.575 | 12.926 | 10.006 | 16.133 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.790 | 2.745 | 3.996 | 5.649 | 4.988 | 8.601 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.857 | 1.460 | 1.900 | 3.002 | 1.989 | 3.375 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 58.243 | 72.212 | 82.354 | 101.300 | 85.569 | 116.637 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.105 | 21.964 | 25.964 | 33.075 | 35.794 | 60.006 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 10.084 | 11.627 | 19.528 | 24.274 | 25.022 | 47.864 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.490 | 10.551 | 11.169 | 17.286 | 16.000 | 33.433 |
| vue-docs-suspense | 8291 | gfm | no | yes | 11.320 | 15.822 | 29.971 | 30.650 | 29.006 | 49.246 |
| vue-docs-slots | 24211 | gfm | no | yes | 26.280 | 50.442 | 75.992 | 108.937 | 83.180 | 185.120 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 26.573 | 42.986 | 58.857 | 87.184 | 75.147 | 150.185 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.239 | 6.368 | 7.260 | 11.677 | 10.221 | 20.703 |
| vite-docs-performance | 8184 | gfm | no | yes | 10.940 | 15.550 | 18.997 | 26.918 | 26.084 | 46.097 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 78.829 | 99.755 | 108.665 | 135.638 | 122.765 | 211.273 |
| vite-docs-features | 39739 | gfm | no | no | 66.284 | 95.560 | 132.445 | 167.419 | 156.144 | 283.427 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.341 | 9.765 | 9.120 | 15.132 | 13.795 | 27.891 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 49.848 | 75.730 | 100.562 | 139.432 | 117.991 | 208.305 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 22.552 | 36.938 | 79.976 | 121.884 | 65.146 | 189.470 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 62.775 | 86.360 | 142.866 | 190.759 | 164.670 | 339.738 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.565 | 2.378 | 3.388 | 5.295 | 3.773 | 7.043 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.376 | 3.633 | 5.026 | 8.068 | 6.338 | 12.699 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 45.423 | 68.139 | 94.517 | 148.785 | 133.099 | 287.044 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 11.363 | 17.871 | 19.854 | 62.829 | 54.699 | 151.146 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.725 | 4.178 | 7.614 | 8.216 | 6.281 | 11.087 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 11.935 | 17.641 | 46.758 | 35.367 | 30.422 | 55.897 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 73.790 | 111.466 | 182.389 | 223.996 | 213.949 | 431.270 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 14.419 | 22.200 | 24.704 | 67.138 | 59.395 | 166.165 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.109 | 3.409 | 5.283 | 7.185 | 5.354 | 9.786 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 6.042 | 9.444 | 15.296 | 20.606 | 16.862 | 33.604 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 135.338 | 235.276 | 310.882 | 492.572 | 431.923 | 889.098 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 32.442 | 50.833 | 52.348 | 140.703 | 125.073 | 357.449 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.646 | 5.655 | 12.289 | 12.452 | 10.115 | 20.111 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 4.888 | 7.503 | 14.969 | 16.793 | 14.070 | 29.183 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 81.755 | 131.280 | 185.896 | 273.093 | 260.734 | 514.191 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 18.566 | 28.619 | 30.205 | 81.023 | 73.496 | 194.677 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.126 | 0.147 | 0.160 | 1.231 | 0.253 | 0.395 |
| comment-question | 160 | gfm | yes | yes | 0.142 | 0.162 | 0.191 | 1.387 | 0.390 | 0.871 |
| comment-review | 282 | gfm | yes | yes | 0.202 | 0.260 | 0.324 | 1.627 | 0.622 | 1.441 |
| comment-links | 278 | gfm | yes | yes | 0.473 | 0.632 | 0.927 | 2.111 | 1.057 | 2.209 |
| comment-checklist | 287 | gfm | no | no | 0.689 | 1.008 | 1.097 | 2.185 | 1.300 | 2.328 |
| comment-quote | 290 | gfm | yes | yes | 0.298 | 0.369 | 0.381 | 1.704 | 0.713 | 1.594 |
| comment-unicode | 327 | gfm | yes | yes | 0.549 | 0.576 | 0.771 | 2.082 | 1.139 | 1.957 |
| comment-inline-code | 285 | gfm | yes | yes | 0.291 | 0.353 | 0.791 | 1.770 | 0.889 | 1.536 |
| comment-reproduction | 298 | gfm | yes | yes | 0.310 | 0.418 | 0.604 | 1.917 | 0.819 | 1.712 |
| comment-table | 310 | gfm | yes | yes | 1.450 | 1.777 | 1.501 | 3.078 | 1.688 | 2.542 |
| comment-review-long | 957 | gfm | yes | yes | 0.880 | 1.326 | 1.525 | 3.241 | 2.245 | 4.841 |
| comment-incident | 1124 | gfm | no | yes | 1.784 | 2.631 | 2.423 | 4.022 | 3.217 | 5.923 |
| guard-angle-link | 41 | commonmark | no | no | 0.271 | 0.340 | 0.577 | 1.581 | 0.547 | 0.775 |
| legacy-contributing | 9323 | gfm | no | yes | 13.531 | 17.254 | 23.682 | 29.509 | 31.061 | 50.367 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 12.737 | 17.218 | 25.294 | 33.635 | 31.213 | 53.029 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.368 | 3.652 | 5.702 | 7.228 | 6.913 | 10.762 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 3.480 | 4.911 | 6.357 | 8.702 | 7.682 | 12.999 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 8.257 | 11.380 | 20.641 | 25.168 | 23.949 | 38.851 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.948 | 4.482 | 7.108 | 9.335 | 8.433 | 13.522 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.127 | 8.101 | 14.159 | 14.573 | 16.705 | 24.077 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 3.080 | 4.070 | 6.393 | 9.140 | 8.811 | 15.750 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.449 | 6.213 | 10.730 | 12.539 | 13.065 | 22.189 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 9.478 | 12.836 | 19.215 | 26.553 | 22.100 | 41.226 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.306 | 7.951 | 8.748 | 12.520 | 9.796 | 16.166 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.656 | 2.485 | 3.440 | 5.287 | 4.783 | 8.522 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.745 | 1.210 | 1.486 | 2.808 | 1.850 | 3.492 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 57.793 | 71.953 | 80.309 | 99.418 | 84.564 | 116.734 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.044 | 21.758 | 24.335 | 32.231 | 35.014 | 60.111 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 9.935 | 11.228 | 18.055 | 23.911 | 24.451 | 48.032 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.334 | 10.392 | 10.560 | 16.760 | 15.593 | 33.559 |
| vue-docs-suspense | 8291 | gfm | no | yes | 11.145 | 15.569 | 28.505 | 29.630 | 28.446 | 49.376 |
| vue-docs-slots | 24211 | gfm | no | yes | 26.208 | 50.109 | 74.011 | 107.127 | 81.709 | 185.472 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 26.133 | 42.712 | 57.590 | 85.479 | 73.861 | 150.217 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 3.997 | 6.034 | 6.504 | 11.309 | 9.837 | 20.662 |
| vite-docs-performance | 8184 | gfm | no | yes | 10.843 | 15.242 | 18.004 | 26.228 | 25.372 | 46.164 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 78.387 | 98.228 | 106.024 | 131.830 | 119.913 | 210.754 |
| vite-docs-features | 39739 | gfm | no | no | 65.905 | 93.840 | 125.617 | 162.996 | 152.763 | 283.687 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.274 | 9.187 | 8.293 | 14.673 | 13.215 | 28.077 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 49.920 | 75.246 | 97.348 | 135.382 | 115.961 | 209.584 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 22.329 | 36.248 | 77.295 | 118.767 | 62.837 | 188.788 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 62.152 | 86.177 | 139.142 | 188.330 | 162.903 | 338.495 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.429 | 2.114 | 2.796 | 5.073 | 3.563 | 7.078 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.258 | 3.400 | 4.357 | 7.830 | 6.102 | 12.779 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 45.226 | 68.146 | 90.095 | 147.054 | 130.073 | 287.337 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 11.200 | 17.664 | 17.544 | 60.859 | 52.708 | 152.217 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.634 | 3.916 | 6.920 | 8.006 | 6.063 | 11.301 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 11.877 | 17.373 | 47.594 | 35.166 | 29.967 | 55.969 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 73.339 | 111.552 | 176.159 | 220.716 | 211.253 | 429.159 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 15.441 | 21.844 | 22.267 | 65.628 | 57.856 | 163.586 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.000 | 3.113 | 4.685 | 6.937 | 5.062 | 9.791 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 5.881 | 9.213 | 13.761 | 20.422 | 16.347 | 33.888 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 135.342 | 233.933 | 296.385 | 494.220 | 421.247 | 887.274 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 31.540 | 50.362 | 48.543 | 135.262 | 120.748 | 355.424 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.614 | 5.410 | 11.538 | 12.130 | 9.656 | 20.033 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 4.744 | 7.247 | 13.971 | 16.366 | 13.670 | 29.406 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 81.665 | 129.607 | 181.440 | 266.707 | 251.594 | 514.560 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 18.039 | 28.543 | 27.668 | 78.539 | 70.908 | 195.125 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 628.36 | 985.63 | 1342.86 | 1830.63 | 1587.28 | 3546.20 |
| rotating-commonmark | reuse | 17 | 600.03 | 950.96 | 1269.48 | 1800.35 | 1536.15 | 3544.14 |
| rotating-gfm | fresh | 40 | 908.26 | 1260.33 | 1751.16 | 2058.03 | 1675.86 | 2948.28 |
| rotating-gfm | reuse | 40 | 879.60 | 1210.95 | 1669.29 | 2003.13 | 1602.17 | 2950.80 |
