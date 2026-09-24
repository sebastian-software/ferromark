# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.77× | 0.58× | 0.21× | 0.44× | 0.20× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.47× | 0.28× | 0.39× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.67× | 0.45× | 0.29× | 0.37× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.59× | 0.23× | 0.44× | 0.20× |
| comments (all workloads) | 6 | 1.00× | 0.77× | 0.59× | 0.23× | 0.44× | 0.20× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.24× | 0.31× | 0.17× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.55× | 0.34× | 0.26× | 0.31× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.59× | 0.38× | 0.53× | 0.28× |
| readme (all workloads) | 1 | 1.00× | 0.76× | 0.59× | 0.38× | 0.53× | 0.28× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.45× | 0.31× | 0.48× | 0.20× |
| reference (all workloads) | 2 | 1.00× | 0.78× | 0.45× | 0.31× | 0.48× | 0.20× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.47× | 0.32× | 0.36× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.70× | 0.47× | 0.32× | 0.36× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.24× | 0.31× | 0.17× |
| commonmark (all workloads) | 8 | 1.00× | 0.55× | 0.34× | 0.26× | 0.31× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.51× | 0.29× | 0.40× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.73× | 0.51× | 0.29× | 0.40× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.55× | 0.22× | 0.42× | 0.20× |
| <512 B (all workloads) | 6 | 1.00× | 0.73× | 0.55× | 0.22× | 0.42× | 0.20× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.43× | 0.29× | 0.38× | 0.20× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.64× | 0.43× | 0.29× | 0.38× | 0.20× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.44× | 0.31× | 0.34× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.67× | 0.44× | 0.31× | 0.34× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.52× | 0.38× | 0.43× | 0.19× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.52× | 0.38× | 0.43× | 0.19× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.32× | 0.21× | 0.40× | 0.14× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.62× | 0.36× | 0.27× | 0.33× | 0.13× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.86× | 0.63× | 0.18× | 0.38× | 0.16× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.50× | 0.27× | 0.38× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.69× | 0.48× | 0.27× | 0.37× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.65× | 0.20× | 0.40× | 0.16× |
| comments (all workloads) | 6 | 1.00× | 0.85× | 0.65× | 0.20× | 0.40× | 0.16× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.33× | 0.24× | 0.30× | 0.16× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.56× | 0.36× | 0.26× | 0.31× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.62× | 0.38× | 0.53× | 0.27× |
| readme (all workloads) | 1 | 1.00× | 0.76× | 0.62× | 0.38× | 0.53× | 0.27× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.47× | 0.31× | 0.49× | 0.20× |
| reference (all workloads) | 2 | 1.00× | 0.78× | 0.47× | 0.31× | 0.49× | 0.20× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.49× | 0.32× | 0.36× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.70× | 0.49× | 0.32× | 0.36× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.33× | 0.24× | 0.30× | 0.16× |
| commonmark (all workloads) | 8 | 1.00× | 0.56× | 0.36× | 0.26× | 0.31× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.54× | 0.28× | 0.39× | 0.17× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.75× | 0.54× | 0.28× | 0.39× | 0.17× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.60× | 0.18× | 0.37× | 0.15× |
| <512 B (all workloads) | 6 | 1.00× | 0.80× | 0.60× | 0.18× | 0.37× | 0.15× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.46× | 0.29× | 0.37× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.64× | 0.46× | 0.29× | 0.37× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.46× | 0.32× | 0.35× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.68× | 0.46× | 0.32× | 0.35× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.38× | 0.43× | 0.19× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.53× | 0.38× | 0.43× | 0.19× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.33× | 0.21× | 0.41× | 0.14× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.62× | 0.37× | 0.27× | 0.34× | 0.13× |

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
| comment-question | 160 | gfm | yes | yes | 0.121 | 0.165 | 0.180 | 0.877 | 0.290 | 0.770 |
| comment-links | 278 | gfm | yes | yes | 0.324 | 0.479 | 0.702 | 1.430 | 0.757 | 1.618 |
| comment-quote | 290 | gfm | yes | yes | 0.229 | 0.281 | 0.349 | 1.150 | 0.573 | 1.434 |
| comment-inline-code | 285 | gfm | yes | yes | 0.238 | 0.287 | 0.608 | 1.213 | 0.689 | 1.385 |
| comment-table | 310 | gfm | yes | yes | 0.799 | 0.999 | 0.995 | 2.169 | 1.228 | 2.170 |
| comment-incident | 1124 | gfm | no | yes | 1.041 | 1.317 | 1.525 | 2.832 | 2.191 | 5.384 |
| legacy-contributing | 9323 | gfm | no | yes | 8.762 | 10.973 | 16.510 | 23.247 | 22.595 | 47.796 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.687 | 2.425 | 4.144 | 5.540 | 4.806 | 8.735 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.933 | 7.614 | 14.257 | 19.193 | 16.668 | 33.128 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.220 | 5.179 | 9.913 | 10.848 | 11.821 | 20.823 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.112 | 4.028 | 7.518 | 9.352 | 9.426 | 18.740 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.431 | 4.500 | 5.817 | 8.997 | 6.504 | 12.307 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.448 | 0.797 | 1.054 | 1.939 | 1.289 | 2.573 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.313 | 13.431 | 17.004 | 24.122 | 25.875 | 59.092 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.910 | 6.406 | 7.356 | 12.922 | 11.456 | 31.816 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.139 | 36.142 | 67.108 | 83.015 | 63.094 | 146.277 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.473 | 3.950 | 4.616 | 8.333 | 7.404 | 18.641 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 54.297 | 73.986 | 83.214 | 118.056 | 92.910 | 191.612 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.841 | 5.793 | 5.882 | 10.669 | 9.462 | 26.605 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 20.887 | 25.370 | 65.857 | 100.070 | 51.942 | 147.450 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.889 | 1.595 | 2.140 | 3.701 | 2.733 | 5.193 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 30.864 | 51.513 | 77.045 | 113.183 | 103.422 | 284.491 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.575 | 2.961 | 5.476 | 6.210 | 4.838 | 8.128 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 53.623 | 92.469 | 162.731 | 183.098 | 168.408 | 380.959 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.251 | 2.300 | 3.669 | 5.197 | 4.078 | 7.399 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 106.991 | 186.336 | 275.665 | 350.596 | 317.316 | 734.576 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.144 | 4.214 | 9.159 | 9.424 | 7.723 | 15.504 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 60.173 | 110.497 | 168.512 | 216.157 | 199.244 | 444.880 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.074 | 0.078 | 0.104 | 0.844 | 0.261 | 0.767 |
| comment-links | 278 | gfm | yes | yes | 0.266 | 0.385 | 0.507 | 1.351 | 0.673 | 1.620 |
| comment-quote | 290 | gfm | yes | yes | 0.177 | 0.190 | 0.240 | 1.071 | 0.483 | 1.427 |
| comment-inline-code | 285 | gfm | yes | yes | 0.191 | 0.196 | 0.460 | 1.131 | 0.606 | 1.380 |
| comment-table | 310 | gfm | yes | yes | 0.740 | 0.901 | 0.846 | 2.096 | 1.150 | 2.174 |
| comment-incident | 1124 | gfm | no | yes | 0.968 | 1.209 | 1.322 | 2.700 | 2.055 | 5.288 |
| legacy-contributing | 9323 | gfm | no | yes | 8.713 | 10.870 | 15.640 | 22.710 | 21.882 | 48.095 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.618 | 2.305 | 3.886 | 5.343 | 4.626 | 8.794 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.837 | 7.458 | 13.629 | 18.749 | 15.967 | 33.136 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.178 | 5.053 | 9.394 | 10.542 | 11.643 | 20.709 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.046 | 3.910 | 6.999 | 9.124 | 9.059 | 18.636 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.357 | 4.400 | 5.380 | 8.838 | 6.297 | 12.364 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.385 | 0.692 | 0.849 | 1.874 | 1.200 | 2.563 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.172 | 13.154 | 15.875 | 23.626 | 25.126 | 59.317 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.844 | 6.269 | 6.826 | 12.522 | 11.170 | 31.636 |
| vue-docs-slots | 24211 | gfm | no | yes | 18.890 | 35.781 | 66.753 | 81.496 | 61.479 | 144.865 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.410 | 3.823 | 4.195 | 8.109 | 7.109 | 18.730 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 53.918 | 73.990 | 81.100 | 115.570 | 91.621 | 190.111 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.766 | 5.563 | 5.404 | 10.433 | 9.138 | 26.803 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 20.793 | 24.993 | 63.781 | 98.954 | 50.149 | 147.183 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.826 | 1.477 | 1.796 | 3.583 | 2.633 | 5.199 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.358 | 51.366 | 72.927 | 113.414 | 102.856 | 286.796 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.509 | 2.846 | 5.104 | 6.060 | 4.689 | 8.224 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 53.553 | 92.514 | 156.601 | 179.045 | 167.517 | 382.808 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.189 | 2.197 | 3.315 | 5.039 | 3.974 | 7.437 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 109.241 | 189.045 | 267.673 | 346.116 | 313.302 | 735.810 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.118 | 4.132 | 8.690 | 9.277 | 7.572 | 15.502 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 60.016 | 110.241 | 164.301 | 212.991 | 194.390 | 441.757 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 315.67 | 523.62 | 801.69 | 942.39 | 838.67 | 1940.86 |
| rotating-commonmark | reuse | 8 | 312.88 | 513.40 | 770.66 | 932.77 | 832.96 | 1940.11 |
| rotating-gfm | fresh | 20 | 208.51 | 293.28 | 439.63 | 536.07 | 414.94 | 897.16 |
| rotating-gfm | reuse | 20 | 200.44 | 282.93 | 422.67 | 523.62 | 400.88 | 896.44 |
