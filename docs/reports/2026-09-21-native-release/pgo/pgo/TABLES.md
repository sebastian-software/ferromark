# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.84× | 0.68× | 0.20× | 0.50× | 0.23× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.51× | 0.26× | 0.42× | 0.19× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.72× | 0.49× | 0.26× | 0.40× | 0.18× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.72× | 0.23× | 0.52× | 0.23× |
| comments (all workloads) | 6 | 1.00× | 0.88× | 0.72× | 0.23× | 0.52× | 0.23× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.19× | 0.28× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.55× | 0.31× | 0.22× | 0.30× | 0.14× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.70× | 0.37× | 0.64× | 0.33× |
| readme (all workloads) | 1 | 1.00× | 0.85× | 0.70× | 0.37× | 0.64× | 0.33× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.56× | 0.30× | 0.56× | 0.24× |
| reference (all workloads) | 2 | 1.00× | 0.89× | 0.56× | 0.30× | 0.56× | 0.24× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.53× | 0.29× | 0.40× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.76× | 0.53× | 0.29× | 0.40× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.19× | 0.28× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.55× | 0.31× | 0.22× | 0.30× | 0.14× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.59× | 0.27× | 0.46× | 0.20× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.81× | 0.59× | 0.27× | 0.46× | 0.20× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.63× | 0.20× | 0.48× | 0.22× |
| <512 B (all workloads) | 6 | 1.00× | 0.79× | 0.63× | 0.20× | 0.48× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.44× | 0.26× | 0.39× | 0.20× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.66× | 0.44× | 0.26× | 0.39× | 0.20× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.48× | 0.29× | 0.38× | 0.17× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.74× | 0.48× | 0.29× | 0.38× | 0.17× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.60× | 0.34× | 0.47× | 0.20× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.81× | 0.60× | 0.34× | 0.47× | 0.20× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.35× | 0.20× | 0.45× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.64× | 0.37× | 0.24× | 0.33× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 1.06× | 0.76× | 0.18× | 0.46× | 0.19× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.55× | 0.25× | 0.42× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.76× | 0.52× | 0.25× | 0.40× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.80× | 0.20× | 0.49× | 0.19× |
| comments (all workloads) | 6 | 1.00× | 1.07× | 0.80× | 0.20× | 0.49× | 0.19× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.27× | 0.19× | 0.28× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.55× | 0.32× | 0.22× | 0.29× | 0.14× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.76× | 0.37× | 0.66× | 0.33× |
| readme (all workloads) | 1 | 1.00× | 0.86× | 0.76× | 0.37× | 0.66× | 0.33× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.58× | 0.30× | 0.58× | 0.24× |
| reference (all workloads) | 2 | 1.00× | 0.89× | 0.58× | 0.30× | 0.58× | 0.24× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.56× | 0.29× | 0.40× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.77× | 0.56× | 0.29× | 0.40× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.27× | 0.19× | 0.28× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.55× | 0.32× | 0.22× | 0.29× | 0.14× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.64× | 0.26× | 0.45× | 0.19× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.87× | 0.64× | 0.26× | 0.45× | 0.19× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.70× | 0.18× | 0.44× | 0.18× |
| <512 B (all workloads) | 6 | 1.00× | 0.96× | 0.70× | 0.18× | 0.44× | 0.18× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.47× | 0.25× | 0.40× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.68× | 0.47× | 0.25× | 0.40× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.51× | 0.29× | 0.38× | 0.17× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.75× | 0.51× | 0.29× | 0.38× | 0.17× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.62× | 0.34× | 0.48× | 0.20× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.81× | 0.62× | 0.34× | 0.48× | 0.20× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.37× | 0.20× | 0.47× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.64× | 0.37× | 0.24× | 0.33× | 0.13× |

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
| comment-question | 160 | gfm | yes | yes | 0.105 | 0.148 | 0.155 | 0.847 | 0.243 | 0.643 |
| comment-links | 278 | gfm | yes | yes | 0.286 | 0.396 | 0.543 | 1.385 | 0.621 | 1.350 |
| comment-quote | 290 | gfm | yes | yes | 0.257 | 0.249 | 0.272 | 1.114 | 0.470 | 1.175 |
| comment-inline-code | 285 | gfm | yes | yes | 0.200 | 0.245 | 0.464 | 1.175 | 0.564 | 1.160 |
| comment-table | 310 | gfm | yes | yes | 0.757 | 0.775 | 0.773 | 2.114 | 0.896 | 1.711 |
| comment-incident | 1124 | gfm | no | yes | 1.076 | 1.016 | 1.105 | 2.756 | 1.766 | 4.351 |
| legacy-contributing | 9323 | gfm | no | yes | 7.472 | 8.550 | 12.370 | 22.521 | 17.289 | 39.596 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.341 | 1.861 | 2.974 | 5.405 | 3.551 | 7.081 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.988 | 6.423 | 10.997 | 18.610 | 13.072 | 25.470 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.910 | 4.099 | 7.539 | 10.543 | 9.142 | 16.680 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.508 | 3.163 | 5.764 | 9.102 | 7.215 | 15.198 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.240 | 3.832 | 4.621 | 8.764 | 5.050 | 9.678 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.373 | 0.659 | 0.842 | 1.899 | 1.019 | 2.115 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.194 | 10.762 | 13.316 | 23.440 | 20.156 | 49.936 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.634 | 5.033 | 5.684 | 12.574 | 9.328 | 25.862 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.557 | 29.958 | 54.581 | 80.753 | 52.067 | 143.217 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.123 | 3.177 | 3.856 | 8.122 | 6.221 | 15.055 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 52.292 | 55.779 | 58.440 | 114.122 | 74.791 | 144.224 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.281 | 4.565 | 4.808 | 10.434 | 7.730 | 21.398 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.389 | 23.038 | 55.158 | 98.632 | 43.098 | 126.317 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.705 | 1.333 | 1.958 | 3.558 | 2.342 | 4.378 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 25.255 | 42.304 | 65.462 | 111.679 | 92.468 | 242.812 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.225 | 2.491 | 5.363 | 6.012 | 4.115 | 6.973 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 45.686 | 76.503 | 143.185 | 177.837 | 148.322 | 330.382 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.976 | 1.960 | 3.403 | 5.074 | 3.444 | 6.416 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 99.599 | 156.517 | 239.992 | 339.852 | 283.234 | 635.399 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.720 | 3.638 | 9.726 | 9.259 | 6.782 | 13.594 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 53.556 | 91.944 | 142.690 | 210.226 | 177.230 | 385.372 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.072 | 0.065 | 0.099 | 0.821 | 0.218 | 0.643 |
| comment-links | 278 | gfm | yes | yes | 0.248 | 0.310 | 0.410 | 1.312 | 0.553 | 1.349 |
| comment-quote | 290 | gfm | yes | yes | 0.220 | 0.156 | 0.192 | 1.050 | 0.413 | 1.173 |
| comment-inline-code | 285 | gfm | yes | yes | 0.163 | 0.159 | 0.357 | 1.100 | 0.501 | 1.157 |
| comment-table | 310 | gfm | yes | yes | 0.714 | 0.688 | 0.653 | 2.035 | 0.852 | 1.713 |
| comment-incident | 1124 | gfm | no | yes | 1.035 | 0.919 | 0.970 | 2.639 | 1.673 | 4.347 |
| legacy-contributing | 9323 | gfm | no | yes | 7.438 | 8.415 | 11.688 | 22.531 | 17.075 | 39.392 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.299 | 1.756 | 2.746 | 5.236 | 3.426 | 7.038 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.964 | 6.283 | 10.582 | 18.500 | 12.597 | 25.354 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.849 | 3.991 | 7.081 | 10.303 | 8.815 | 16.611 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.462 | 3.054 | 5.317 | 8.931 | 6.981 | 15.178 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.188 | 3.722 | 4.198 | 8.596 | 4.866 | 9.718 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.330 | 0.559 | 0.697 | 1.824 | 0.921 | 2.114 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.184 | 10.654 | 12.326 | 23.179 | 19.628 | 49.853 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.592 | 4.908 | 5.288 | 12.221 | 9.030 | 25.771 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.508 | 29.881 | 53.602 | 79.290 | 51.022 | 143.731 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.061 | 3.058 | 3.494 | 7.895 | 6.001 | 15.152 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 52.298 | 55.844 | 56.872 | 112.614 | 73.430 | 145.006 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.240 | 4.417 | 4.270 | 10.197 | 7.517 | 21.418 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.324 | 22.739 | 52.869 | 96.721 | 41.489 | 126.114 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.660 | 1.223 | 1.659 | 3.469 | 2.242 | 4.372 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.593 | 42.122 | 60.710 | 109.593 | 90.654 | 243.505 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.192 | 2.388 | 5.129 | 5.877 | 3.961 | 6.970 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 45.605 | 76.614 | 139.444 | 175.855 | 147.128 | 329.628 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.939 | 1.835 | 3.144 | 4.946 | 3.341 | 6.393 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 94.719 | 156.361 | 235.758 | 337.177 | 278.771 | 636.241 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.687 | 3.535 | 9.325 | 9.072 | 6.589 | 13.543 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 53.537 | 91.162 | 141.134 | 205.914 | 173.161 | 385.008 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 288.02 | 458.71 | 744.04 | 923.20 | 760.83 | 1704.16 |
| rotating-commonmark | reuse | 8 | 282.22 | 446.15 | 712.18 | 906.58 | 747.61 | 1681.53 |
| rotating-gfm | fresh | 20 | 199.10 | 249.95 | 378.52 | 520.15 | 350.94 | 782.50 |
| rotating-gfm | reuse | 20 | 193.15 | 244.49 | 364.30 | 509.34 | 343.04 | 782.65 |
