# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.77× | 0.62× | 0.19× | 0.46× | 0.21× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.49× | 0.25× | 0.40× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.68× | 0.46× | 0.24× | 0.38× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.64× | 0.20× | 0.47× | 0.21× |
| comments (all workloads) | 6 | 1.00× | 0.78× | 0.64× | 0.20× | 0.47× | 0.21× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.20× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.52× | 0.29× | 0.21× | 0.28× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.72× | 0.38× | 0.66× | 0.34× |
| readme (all workloads) | 1 | 1.00× | 0.85× | 0.72× | 0.38× | 0.66× | 0.34× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.51× | 0.28× | 0.52× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.81× | 0.51× | 0.28× | 0.52× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.50× | 0.28× | 0.38× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.72× | 0.50× | 0.28× | 0.38× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.20× | 0.29× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.52× | 0.29× | 0.21× | 0.28× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.55× | 0.26× | 0.43× | 0.19× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.75× | 0.55× | 0.26× | 0.43× | 0.19× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.59× | 0.19× | 0.45× | 0.20× |
| <512 B (all workloads) | 6 | 1.00× | 0.73× | 0.59× | 0.19× | 0.45× | 0.20× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.43× | 0.25× | 0.38× | 0.20× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.64× | 0.43× | 0.25× | 0.38× | 0.20× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.45× | 0.27× | 0.35× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.69× | 0.45× | 0.27× | 0.35× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.55× | 0.32× | 0.45× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.76× | 0.55× | 0.32× | 0.45× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.35× | 0.20× | 0.45× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.59× | 0.33× | 0.22× | 0.30× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.94× | 0.67× | 0.16× | 0.41× | 0.17× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.52× | 0.24× | 0.40× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.71× | 0.49× | 0.24× | 0.38× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.70× | 0.18× | 0.43× | 0.17× |
| comments (all workloads) | 6 | 1.00× | 0.93× | 0.70× | 0.18× | 0.43× | 0.17× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.27× | 0.19× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.52× | 0.30× | 0.21× | 0.28× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.79× | 0.38× | 0.69× | 0.34× |
| readme (all workloads) | 1 | 1.00× | 0.87× | 0.79× | 0.38× | 0.69× | 0.34× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.53× | 0.28× | 0.53× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.80× | 0.53× | 0.28× | 0.53× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.53× | 0.28× | 0.39× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.73× | 0.53× | 0.28× | 0.39× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.27× | 0.19× | 0.29× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.52× | 0.30× | 0.21× | 0.28× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.59× | 0.25× | 0.42× | 0.17× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.80× | 0.59× | 0.25× | 0.42× | 0.17× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.64× | 0.16× | 0.41× | 0.16× |
| <512 B (all workloads) | 6 | 1.00× | 0.87× | 0.64× | 0.16× | 0.41× | 0.16× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.46× | 0.25× | 0.39× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.65× | 0.46× | 0.25× | 0.39× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.27× | 0.36× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.70× | 0.47× | 0.27× | 0.36× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.57× | 0.33× | 0.45× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.76× | 0.57× | 0.33× | 0.45× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.37× | 0.20× | 0.47× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.59× | 0.35× | 0.23× | 0.31× | 0.12× |

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
| comment-question | 160 | gfm | yes | yes | 0.096 | 0.150 | 0.157 | 0.852 | 0.248 | 0.646 |
| comment-links | 278 | gfm | yes | yes | 0.302 | 0.406 | 0.560 | 1.401 | 0.626 | 1.368 |
| comment-quote | 290 | gfm | yes | yes | 0.187 | 0.254 | 0.274 | 1.122 | 0.470 | 1.184 |
| comment-inline-code | 285 | gfm | yes | yes | 0.193 | 0.247 | 0.466 | 1.175 | 0.571 | 1.184 |
| comment-table | 310 | gfm | yes | yes | 0.758 | 0.786 | 0.782 | 2.144 | 0.911 | 1.708 |
| comment-incident | 1124 | gfm | no | yes | 0.871 | 1.032 | 1.118 | 2.790 | 1.783 | 4.397 |
| legacy-contributing | 9323 | gfm | no | yes | 7.157 | 8.613 | 12.460 | 22.525 | 17.585 | 41.474 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.355 | 1.892 | 3.004 | 5.441 | 3.621 | 7.185 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.058 | 6.512 | 11.164 | 18.747 | 13.053 | 26.330 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.497 | 4.132 | 7.609 | 10.579 | 9.283 | 17.307 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.558 | 3.207 | 5.736 | 9.211 | 7.339 | 15.624 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.363 | 3.954 | 4.670 | 8.874 | 5.089 | 9.795 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.391 | 0.664 | 0.853 | 1.915 | 1.026 | 2.136 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.097 | 10.902 | 13.436 | 23.488 | 20.281 | 52.257 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.356 | 5.077 | 5.711 | 12.630 | 9.330 | 26.604 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.812 | 30.320 | 59.634 | 81.164 | 51.927 | 143.956 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.041 | 3.190 | 3.904 | 8.181 | 6.210 | 15.817 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 44.705 | 57.198 | 60.581 | 115.137 | 74.412 | 148.756 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.358 | 4.638 | 4.737 | 10.505 | 7.826 | 21.983 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.596 | 23.335 | 55.574 | 99.925 | 43.529 | 129.178 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.724 | 1.350 | 1.973 | 3.596 | 2.373 | 4.430 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 25.128 | 42.857 | 66.241 | 112.342 | 92.909 | 252.158 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.241 | 2.537 | 5.393 | 6.099 | 4.101 | 7.068 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 40.455 | 77.420 | 148.760 | 179.583 | 149.527 | 337.576 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.987 | 1.985 | 3.426 | 5.073 | 3.449 | 6.462 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 87.534 | 160.976 | 243.836 | 340.422 | 283.637 | 648.275 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.741 | 3.685 | 9.756 | 9.183 | 6.799 | 13.790 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 46.072 | 93.407 | 148.573 | 212.206 | 179.423 | 394.700 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.063 | 0.066 | 0.100 | 0.827 | 0.222 | 0.646 |
| comment-links | 278 | gfm | yes | yes | 0.259 | 0.316 | 0.421 | 1.330 | 0.555 | 1.370 |
| comment-quote | 290 | gfm | yes | yes | 0.149 | 0.158 | 0.195 | 1.052 | 0.412 | 1.191 |
| comment-inline-code | 285 | gfm | yes | yes | 0.156 | 0.161 | 0.358 | 1.107 | 0.511 | 1.185 |
| comment-table | 310 | gfm | yes | yes | 0.712 | 0.695 | 0.661 | 2.080 | 0.859 | 1.708 |
| comment-incident | 1124 | gfm | no | yes | 0.831 | 0.939 | 0.980 | 2.654 | 1.697 | 4.420 |
| legacy-contributing | 9323 | gfm | no | yes | 7.123 | 8.546 | 11.847 | 22.130 | 17.000 | 41.076 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.303 | 1.778 | 2.774 | 5.237 | 3.462 | 7.184 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.014 | 6.364 | 10.671 | 18.406 | 12.599 | 26.108 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.434 | 4.038 | 7.128 | 10.339 | 8.935 | 17.234 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.505 | 3.098 | 5.387 | 9.025 | 7.077 | 15.730 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.335 | 3.822 | 4.224 | 8.708 | 4.867 | 9.790 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.343 | 0.565 | 0.699 | 1.834 | 0.927 | 2.149 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.030 | 10.781 | 12.427 | 23.170 | 19.697 | 52.233 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.314 | 4.923 | 5.283 | 12.295 | 9.013 | 26.938 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.816 | 30.379 | 58.267 | 80.176 | 50.818 | 144.466 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.992 | 3.087 | 3.535 | 7.974 | 5.958 | 15.728 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 44.495 | 58.314 | 58.055 | 113.211 | 73.382 | 149.424 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.309 | 4.444 | 4.261 | 10.265 | 7.519 | 22.005 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.511 | 23.103 | 53.429 | 97.890 | 41.883 | 129.306 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.675 | 1.238 | 1.664 | 3.502 | 2.261 | 4.435 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 25.129 | 42.605 | 62.132 | 110.137 | 91.471 | 251.798 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.203 | 2.429 | 5.059 | 5.945 | 3.940 | 7.060 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 40.917 | 77.364 | 145.572 | 177.393 | 148.182 | 338.479 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.957 | 1.877 | 3.165 | 4.943 | 3.345 | 6.444 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 89.770 | 160.758 | 236.655 | 340.246 | 280.495 | 654.144 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.706 | 3.571 | 9.367 | 8.998 | 6.603 | 13.727 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 46.194 | 92.423 | 143.563 | 206.154 | 173.760 | 392.556 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 282.37 | 462.82 | 756.80 | 923.32 | 762.34 | 1718.84 |
| rotating-commonmark | reuse | 8 | 275.96 | 453.19 | 730.81 | 909.07 | 749.23 | 1724.16 |
| rotating-gfm | fresh | 20 | 185.85 | 260.54 | 385.75 | 528.23 | 355.37 | 804.81 |
| rotating-gfm | reuse | 20 | 178.03 | 245.32 | 365.93 | 514.45 | 345.26 | 803.39 |
