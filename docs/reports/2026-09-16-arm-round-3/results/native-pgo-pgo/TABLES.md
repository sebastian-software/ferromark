# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.98× | 0.79× | 0.24× | 0.59× | 0.26× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.53× | 0.27× | 0.44× | 0.20× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.72× | 0.49× | 0.26× | 0.41× | 0.18× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.80× | 0.25× | 0.58× | 0.26× |
| comments (all workloads) | 6 | 1.00× | 0.97× | 0.80× | 0.25× | 0.58× | 0.26× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.20× | 0.29× | 0.16× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.51× | 0.29× | 0.21× | 0.28× | 0.14× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.70× | 0.37× | 0.65× | 0.33× |
| readme (all workloads) | 1 | 1.00× | 0.84× | 0.70× | 0.37× | 0.65× | 0.33× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.56× | 0.30× | 0.57× | 0.24× |
| reference (all workloads) | 2 | 1.00× | 0.88× | 0.56× | 0.30× | 0.57× | 0.24× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.53× | 0.29× | 0.40× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.76× | 0.53× | 0.29× | 0.40× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.20× | 0.29× | 0.16× |
| commonmark (all workloads) | 8 | 1.00× | 0.51× | 0.29× | 0.21× | 0.28× | 0.14× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.61× | 0.28× | 0.47× | 0.21× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.83× | 0.61× | 0.28× | 0.47× | 0.21× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.73× | 0.24× | 0.56× | 0.25× |
| <512 B (all workloads) | 6 | 1.00× | 0.91× | 0.73× | 0.24× | 0.56× | 0.25× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.45× | 0.26× | 0.40× | 0.21× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.67× | 0.45× | 0.26× | 0.40× | 0.21× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.46× | 0.28× | 0.36× | 0.16× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.72× | 0.46× | 0.28× | 0.36× | 0.16× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.56× | 0.33× | 0.45× | 0.19× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.77× | 0.56× | 0.33× | 0.45× | 0.19× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.42× | 0.23× | 0.54× | 0.18× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.58× | 0.34× | 0.23× | 0.31× | 0.13× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.98× | 0.70× | 0.16× | 0.43× | 0.17× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.52× | 0.24× | 0.40× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.71× | 0.49× | 0.24× | 0.38× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.72× | 0.18× | 0.44× | 0.18× |
| comments (all workloads) | 6 | 1.00× | 0.97× | 0.72× | 0.18× | 0.44× | 0.18× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.18× | 0.27× | 0.14× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.49× | 0.29× | 0.20× | 0.27× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.74× | 0.36× | 0.65× | 0.32× |
| readme (all workloads) | 1 | 1.00× | 0.83× | 0.74× | 0.36× | 0.65× | 0.32× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.58× | 0.31× | 0.58× | 0.23× |
| reference (all workloads) | 2 | 1.00× | 0.88× | 0.58× | 0.31× | 0.58× | 0.23× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.54× | 0.28× | 0.39× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.75× | 0.54× | 0.28× | 0.39× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.18× | 0.27× | 0.14× |
| commonmark (all workloads) | 8 | 1.00× | 0.49× | 0.29× | 0.20× | 0.27× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.60× | 0.25× | 0.43× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.83× | 0.60× | 0.25× | 0.43× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.65× | 0.17× | 0.42× | 0.17× |
| <512 B (all workloads) | 6 | 1.00× | 0.89× | 0.65× | 0.17× | 0.42× | 0.17× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.45× | 0.24× | 0.38× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.65× | 0.45× | 0.24× | 0.38× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.27× | 0.36× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.71× | 0.47× | 0.27× | 0.36× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.59× | 0.33× | 0.46× | 0.19× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.77× | 0.59× | 0.33× | 0.46× | 0.19× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.43× | 0.24× | 0.55× | 0.18× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.58× | 0.35× | 0.23× | 0.31× | 0.13× |

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
| comment-question | 160 | gfm | yes | yes | 0.153 | 0.148 | 0.157 | 0.854 | 0.246 | 0.645 |
| comment-links | 278 | gfm | yes | yes | 0.337 | 0.402 | 0.565 | 1.411 | 0.631 | 1.363 |
| comment-quote | 290 | gfm | yes | yes | 0.262 | 0.265 | 0.276 | 1.123 | 0.475 | 1.182 |
| comment-inline-code | 285 | gfm | yes | yes | 0.250 | 0.245 | 0.473 | 1.192 | 0.567 | 1.193 |
| comment-table | 310 | gfm | yes | yes | 0.794 | 0.789 | 0.767 | 2.149 | 0.905 | 1.701 |
| comment-incident | 1124 | gfm | no | yes | 0.949 | 1.032 | 1.117 | 2.774 | 1.782 | 4.368 |
| legacy-contributing | 9323 | gfm | no | yes | 7.392 | 8.654 | 12.587 | 22.871 | 17.473 | 40.231 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.519 | 1.882 | 3.035 | 5.452 | 3.626 | 7.161 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.417 | 6.527 | 11.329 | 18.987 | 13.166 | 26.103 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.717 | 4.150 | 7.693 | 10.671 | 9.212 | 17.036 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.699 | 3.194 | 5.779 | 9.204 | 7.283 | 15.563 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.286 | 3.926 | 4.667 | 8.973 | 5.027 | 9.808 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.428 | 0.662 | 0.851 | 1.922 | 1.024 | 2.133 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.360 | 10.834 | 13.466 | 23.754 | 20.278 | 51.789 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.411 | 5.057 | 5.738 | 12.671 | 9.371 | 26.455 |
| vue-docs-slots | 24211 | gfm | no | yes | 17.123 | 30.241 | 57.210 | 81.260 | 52.027 | 143.627 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.049 | 3.178 | 3.920 | 8.241 | 6.177 | 15.444 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 45.331 | 58.132 | 60.358 | 115.533 | 75.634 | 147.820 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.482 | 4.613 | 4.715 | 10.579 | 7.846 | 21.888 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.107 | 23.235 | 55.380 | 98.959 | 43.057 | 128.039 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.751 | 1.358 | 1.976 | 3.672 | 2.390 | 4.459 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.149 | 44.827 | 64.422 | 112.771 | 93.474 | 250.282 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.275 | 2.531 | 5.429 | 6.148 | 4.118 | 7.088 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 41.462 | 83.257 | 151.662 | 181.471 | 152.441 | 341.873 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.034 | 1.979 | 3.447 | 5.165 | 3.445 | 6.460 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 90.880 | 164.867 | 244.195 | 343.316 | 285.739 | 648.104 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.742 | 3.691 | 9.803 | 9.359 | 6.890 | 13.658 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 45.188 | 98.590 | 149.146 | 212.226 | 180.619 | 392.324 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.072 | 0.067 | 0.101 | 0.836 | 0.223 | 0.649 |
| comment-links | 278 | gfm | yes | yes | 0.245 | 0.314 | 0.422 | 1.330 | 0.556 | 1.359 |
| comment-quote | 290 | gfm | yes | yes | 0.164 | 0.158 | 0.195 | 1.052 | 0.415 | 1.184 |
| comment-inline-code | 285 | gfm | yes | yes | 0.163 | 0.162 | 0.367 | 1.118 | 0.503 | 1.186 |
| comment-table | 310 | gfm | yes | yes | 0.707 | 0.696 | 0.663 | 2.076 | 0.857 | 1.704 |
| comment-incident | 1124 | gfm | no | yes | 0.850 | 0.931 | 0.979 | 2.666 | 1.688 | 4.383 |
| legacy-contributing | 9323 | gfm | no | yes | 7.244 | 8.498 | 11.796 | 22.247 | 17.034 | 40.409 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.420 | 1.776 | 2.794 | 5.283 | 3.478 | 7.109 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.278 | 6.367 | 10.728 | 18.408 | 12.665 | 26.289 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.637 | 4.073 | 7.231 | 10.407 | 8.998 | 17.140 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.618 | 3.091 | 5.434 | 9.032 | 7.091 | 15.798 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.151 | 3.819 | 4.273 | 8.791 | 4.857 | 9.858 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.326 | 0.563 | 0.707 | 1.843 | 0.926 | 2.140 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.303 | 10.792 | 12.469 | 23.471 | 20.031 | 52.292 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.276 | 4.931 | 5.334 | 12.431 | 9.076 | 26.415 |
| vue-docs-slots | 24211 | gfm | no | yes | 17.103 | 30.394 | 55.788 | 80.165 | 51.267 | 144.999 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.934 | 3.052 | 3.540 | 8.087 | 5.947 | 15.631 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 44.803 | 58.587 | 58.245 | 113.129 | 74.039 | 147.842 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.341 | 4.472 | 4.358 | 10.349 | 7.555 | 22.073 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.014 | 22.985 | 53.446 | 97.711 | 41.723 | 127.988 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.631 | 1.227 | 1.677 | 3.526 | 2.263 | 4.403 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 23.863 | 44.582 | 61.893 | 110.072 | 91.559 | 249.132 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.155 | 2.405 | 5.083 | 6.002 | 3.944 | 7.004 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 40.109 | 81.144 | 142.336 | 177.127 | 148.677 | 333.993 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.922 | 1.865 | 3.162 | 5.048 | 3.345 | 6.449 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 87.925 | 164.363 | 236.136 | 337.678 | 279.833 | 650.409 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.630 | 3.587 | 9.519 | 9.147 | 6.645 | 13.694 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 44.504 | 97.084 | 143.019 | 208.221 | 175.452 | 394.270 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 288.13 | 461.74 | 755.72 | 935.29 | 765.11 | 1720.35 |
| rotating-commonmark | reuse | 8 | 279.10 | 452.56 | 724.71 | 916.09 | 752.33 | 1723.97 |
| rotating-gfm | fresh | 20 | 196.62 | 256.28 | 385.05 | 519.54 | 354.51 | 794.92 |
| rotating-gfm | reuse | 20 | 192.10 | 244.45 | 368.84 | 513.40 | 344.05 | 799.31 |
