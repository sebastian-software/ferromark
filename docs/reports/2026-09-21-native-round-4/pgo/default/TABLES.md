# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.81× | 0.60× | 0.22× | 0.46× | 0.21× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.48× | 0.29× | 0.40× | 0.19× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.69× | 0.47× | 0.29× | 0.38× | 0.18× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.62× | 0.25× | 0.46× | 0.21× |
| comments (all workloads) | 6 | 1.00× | 0.81× | 0.62× | 0.25× | 0.46× | 0.21× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.24× | 0.31× | 0.17× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.56× | 0.35× | 0.27× | 0.32× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.66× | 0.42× | 0.59× | 0.31× |
| readme (all workloads) | 1 | 1.00× | 0.84× | 0.66× | 0.42× | 0.59× | 0.31× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.47× | 0.32× | 0.50× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.80× | 0.47× | 0.32× | 0.50× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.48× | 0.33× | 0.37× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.71× | 0.48× | 0.33× | 0.37× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.24× | 0.31× | 0.17× |
| commonmark (all workloads) | 8 | 1.00× | 0.56× | 0.35× | 0.27× | 0.32× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.53× | 0.31× | 0.42× | 0.19× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.75× | 0.53× | 0.31× | 0.42× | 0.19× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.57× | 0.23× | 0.44× | 0.20× |
| <512 B (all workloads) | 6 | 1.00× | 0.76× | 0.57× | 0.23× | 0.44× | 0.20× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.45× | 0.30× | 0.39× | 0.20× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.66× | 0.45× | 0.30× | 0.39× | 0.20× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.45× | 0.32× | 0.35× | 0.16× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.69× | 0.45× | 0.32× | 0.35× | 0.16× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.39× | 0.43× | 0.20× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.74× | 0.53× | 0.39× | 0.43× | 0.20× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.33× | 0.22× | 0.41× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.64× | 0.37× | 0.28× | 0.34× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.92× | 0.67× | 0.19× | 0.41× | 0.17× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.52× | 0.28× | 0.39× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.71× | 0.50× | 0.29× | 0.38× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.69× | 0.21× | 0.42× | 0.17× |
| comments (all workloads) | 6 | 1.00× | 0.91× | 0.69× | 0.21× | 0.42× | 0.17× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.33× | 0.24× | 0.31× | 0.16× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.56× | 0.36× | 0.27× | 0.32× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.70× | 0.43× | 0.60× | 0.30× |
| readme (all workloads) | 1 | 1.00× | 0.85× | 0.70× | 0.43× | 0.60× | 0.30× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.49× | 0.33× | 0.51× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.81× | 0.49× | 0.33× | 0.51× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.33× | 0.37× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.72× | 0.51× | 0.33× | 0.37× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.33× | 0.24× | 0.31× | 0.16× |
| commonmark (all workloads) | 8 | 1.00× | 0.56× | 0.36× | 0.27× | 0.32× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.56× | 0.29× | 0.41× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.78× | 0.56× | 0.29× | 0.41× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.63× | 0.19× | 0.39× | 0.16× |
| <512 B (all workloads) | 6 | 1.00× | 0.85× | 0.63× | 0.19× | 0.39× | 0.16× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.48× | 0.30× | 0.39× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.66× | 0.48× | 0.30× | 0.39× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.32× | 0.35× | 0.16× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.69× | 0.47× | 0.32× | 0.35× | 0.16× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.55× | 0.39× | 0.44× | 0.20× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.75× | 0.55× | 0.39× | 0.44× | 0.20× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.34× | 0.22× | 0.43× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.64× | 0.38× | 0.28× | 0.35× | 0.14× |

## HTML agreement with v2

| Engine | Exact | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 28 | 0 | 0 | 0 |
| OX-Content original | 5 | 0 | 22 | 1 |
| Ferromark v1 | 11 | 14 | 0 | 3 |
| md4c | 11 | 13 | 0 | 4 |
| pulldown-cmark | 7 | 21 | 0 | 0 |
| Bun native bun_md | 8 | 20 | 0 | 0 |

## Per-document timings

Microseconds per complete Markdown→HTML operation; lower is faster. Agreement columns show the six/five-engine sets.
Original input sizes are UTF-8 bytes. “gfm” is the shared subset described in the harness README.

### fresh

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.119 | 0.164 | 0.182 | 0.867 | 0.291 | 0.769 |
| comment-links | 278 | gfm | yes | yes | 0.330 | 0.475 | 0.695 | 1.412 | 0.751 | 1.611 |
| comment-quote | 290 | gfm | yes | yes | 0.225 | 0.276 | 0.351 | 1.148 | 0.564 | 1.415 |
| comment-inline-code | 285 | gfm | yes | yes | 0.235 | 0.286 | 0.603 | 1.207 | 0.680 | 1.382 |
| comment-table | 310 | gfm | yes | yes | 1.002 | 0.998 | 0.988 | 2.170 | 1.215 | 2.161 |
| comment-incident | 1124 | gfm | no | yes | 1.081 | 1.317 | 1.516 | 2.824 | 2.196 | 5.363 |
| legacy-contributing | 9323 | gfm | no | yes | 8.909 | 10.955 | 16.290 | 22.999 | 22.286 | 46.572 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.721 | 2.457 | 4.169 | 5.524 | 4.745 | 8.775 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.879 | 7.637 | 14.141 | 19.161 | 16.580 | 32.389 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.268 | 5.202 | 10.026 | 10.728 | 11.871 | 20.373 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.140 | 4.062 | 7.491 | 9.385 | 9.468 | 18.485 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.811 | 4.530 | 5.800 | 8.979 | 6.445 | 12.310 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.461 | 0.801 | 1.061 | 1.958 | 1.304 | 2.565 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.426 | 13.367 | 16.952 | 23.987 | 25.808 | 58.081 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.148 | 6.367 | 7.344 | 12.781 | 11.505 | 31.405 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.322 | 35.918 | 65.943 | 82.600 | 63.340 | 144.976 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.503 | 3.937 | 4.607 | 8.298 | 7.447 | 18.458 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 56.045 | 75.123 | 82.086 | 116.983 | 93.033 | 188.713 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.067 | 5.738 | 5.864 | 10.679 | 9.463 | 26.368 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.609 | 25.380 | 65.724 | 100.110 | 52.197 | 147.210 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.899 | 1.602 | 2.141 | 3.659 | 2.763 | 5.191 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.057 | 51.736 | 77.963 | 114.489 | 104.799 | 286.634 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.577 | 2.962 | 5.433 | 6.168 | 4.803 | 8.156 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 54.852 | 93.235 | 163.608 | 182.503 | 168.800 | 381.262 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.258 | 2.291 | 3.670 | 5.155 | 4.092 | 7.394 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 110.670 | 187.020 | 275.152 | 350.386 | 319.396 | 733.514 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.178 | 4.258 | 9.208 | 9.350 | 7.749 | 15.415 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 62.145 | 109.988 | 169.696 | 215.037 | 198.951 | 442.905 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.075 | 0.078 | 0.104 | 0.832 | 0.259 | 0.768 |
| comment-links | 278 | gfm | yes | yes | 0.276 | 0.380 | 0.505 | 1.343 | 0.669 | 1.617 |
| comment-quote | 290 | gfm | yes | yes | 0.177 | 0.189 | 0.238 | 1.067 | 0.481 | 1.428 |
| comment-inline-code | 285 | gfm | yes | yes | 0.187 | 0.197 | 0.451 | 1.118 | 0.605 | 1.377 |
| comment-table | 310 | gfm | yes | yes | 0.944 | 0.906 | 0.841 | 2.084 | 1.129 | 2.157 |
| comment-incident | 1124 | gfm | no | yes | 1.023 | 1.206 | 1.303 | 2.683 | 2.078 | 5.379 |
| legacy-contributing | 9323 | gfm | no | yes | 8.863 | 10.839 | 15.488 | 22.683 | 21.680 | 46.658 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.656 | 2.346 | 3.870 | 5.353 | 4.575 | 8.721 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.823 | 7.470 | 13.513 | 18.827 | 16.091 | 32.373 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.216 | 5.094 | 9.327 | 10.471 | 11.485 | 20.361 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.050 | 3.928 | 7.005 | 9.092 | 9.125 | 18.416 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.745 | 4.394 | 5.365 | 8.771 | 6.242 | 12.337 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.395 | 0.689 | 0.854 | 1.867 | 1.197 | 2.555 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.364 | 13.195 | 15.806 | 23.549 | 25.200 | 58.633 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.059 | 6.220 | 6.855 | 12.486 | 11.130 | 31.345 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.210 | 35.673 | 65.799 | 81.045 | 62.138 | 143.528 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.426 | 3.784 | 4.177 | 8.025 | 7.113 | 18.610 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 56.188 | 73.593 | 79.063 | 114.795 | 91.917 | 189.514 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.975 | 5.553 | 5.347 | 10.375 | 9.129 | 26.313 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.529 | 25.160 | 63.278 | 98.266 | 50.358 | 146.012 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.835 | 1.481 | 1.788 | 3.541 | 2.649 | 5.207 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.555 | 51.810 | 74.334 | 112.150 | 102.728 | 286.862 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.520 | 2.839 | 5.080 | 6.005 | 4.653 | 8.164 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.077 | 93.686 | 159.046 | 180.534 | 167.541 | 380.633 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.196 | 2.168 | 3.323 | 5.005 | 3.950 | 7.396 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 109.474 | 184.436 | 261.380 | 341.781 | 310.933 | 728.046 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.109 | 4.121 | 8.651 | 9.138 | 7.515 | 15.404 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 62.570 | 109.185 | 164.282 | 211.548 | 194.238 | 443.991 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 318.02 | 524.98 | 800.65 | 941.43 | 839.92 | 1934.77 |
| rotating-commonmark | reuse | 8 | 314.03 | 511.12 | 769.94 | 927.26 | 833.31 | 1937.20 |
| rotating-gfm | fresh | 20 | 217.38 | 296.13 | 443.39 | 534.95 | 413.91 | 899.92 |
| rotating-gfm | reuse | 20 | 211.84 | 285.39 | 419.60 | 524.51 | 403.69 | 904.12 |
