# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.73× | 0.56× | 0.18× | 0.44× | 0.20× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.46× | 0.24× | 0.39× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.66× | 0.44× | 0.24× | 0.37× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.59× | 0.20× | 0.45× | 0.20× |
| comments (all workloads) | 6 | 1.00× | 0.75× | 0.59× | 0.20× | 0.45× | 0.20× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.24× | 0.19× | 0.27× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.52× | 0.29× | 0.21× | 0.28× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.65× | 0.36× | 0.63× | 0.32× |
| readme (all workloads) | 1 | 1.00× | 0.81× | 0.65× | 0.36× | 0.63× | 0.32× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.50× | 0.27× | 0.51× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.79× | 0.50× | 0.27× | 0.51× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.47× | 0.27× | 0.36× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.69× | 0.47× | 0.27× | 0.36× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.24× | 0.19× | 0.27× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.52× | 0.29× | 0.21× | 0.28× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.52× | 0.25× | 0.41× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.73× | 0.52× | 0.25× | 0.41× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.53× | 0.18× | 0.43× | 0.19× |
| <512 B (all workloads) | 6 | 1.00× | 0.69× | 0.53× | 0.18× | 0.43× | 0.19× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.41× | 0.24× | 0.37× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.62× | 0.41× | 0.24× | 0.37× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.42× | 0.26× | 0.34× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.67× | 0.42× | 0.26× | 0.34× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.31× | 0.43× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.53× | 0.31× | 0.43× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.35× | 0.20× | 0.45× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.60× | 0.34× | 0.23× | 0.31× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.90× | 0.64× | 0.15× | 0.40× | 0.16× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.50× | 0.23× | 0.38× | 0.16× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.69× | 0.47× | 0.23× | 0.37× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.67× | 0.17× | 0.41× | 0.16× |
| comments (all workloads) | 6 | 1.00× | 0.89× | 0.67× | 0.17× | 0.41× | 0.16× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.19× | 0.27× | 0.14× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.52× | 0.30× | 0.21× | 0.28× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.73× | 0.36× | 0.64× | 0.32× |
| readme (all workloads) | 1 | 1.00× | 0.82× | 0.73× | 0.36× | 0.64× | 0.32× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.52× | 0.28× | 0.52× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.80× | 0.52× | 0.28× | 0.52× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.27× | 0.37× | 0.15× |
| technical-docs (all workloads) | 11 | 1.00× | 0.70× | 0.51× | 0.27× | 0.37× | 0.15× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.19× | 0.27× | 0.14× |
| commonmark (all workloads) | 8 | 1.00× | 0.52× | 0.30× | 0.21× | 0.28× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.57× | 0.24× | 0.41× | 0.17× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.77× | 0.57× | 0.24× | 0.41× | 0.17× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.61× | 0.16× | 0.39× | 0.16× |
| <512 B (all workloads) | 6 | 1.00× | 0.83× | 0.61× | 0.16× | 0.39× | 0.16× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.43× | 0.24× | 0.37× | 0.18× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.63× | 0.43× | 0.24× | 0.37× | 0.18× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.45× | 0.26× | 0.34× | 0.14× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.67× | 0.45× | 0.26× | 0.34× | 0.14× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.57× | 0.32× | 0.44× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.74× | 0.57× | 0.32× | 0.44× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.37× | 0.20× | 0.46× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.61× | 0.36× | 0.23× | 0.31× | 0.12× |

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
| comment-question | 160 | gfm | yes | yes | 0.093 | 0.151 | 0.157 | 0.850 | 0.250 | 0.643 |
| comment-links | 278 | gfm | yes | yes | 0.272 | 0.406 | 0.558 | 1.392 | 0.633 | 1.361 |
| comment-quote | 290 | gfm | yes | yes | 0.186 | 0.250 | 0.273 | 1.116 | 0.470 | 1.181 |
| comment-inline-code | 285 | gfm | yes | yes | 0.183 | 0.250 | 0.467 | 1.182 | 0.544 | 1.176 |
| comment-table | 310 | gfm | yes | yes | 0.736 | 0.784 | 0.991 | 2.130 | 0.900 | 1.695 |
| comment-incident | 1124 | gfm | no | yes | 0.849 | 1.027 | 1.104 | 2.769 | 1.769 | 4.402 |
| legacy-contributing | 9323 | gfm | no | yes | 6.865 | 8.592 | 12.688 | 22.594 | 17.441 | 40.176 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.317 | 1.887 | 3.078 | 5.426 | 3.602 | 7.143 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.911 | 6.454 | 11.177 | 18.689 | 12.867 | 25.772 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.336 | 4.127 | 7.732 | 10.561 | 9.152 | 17.016 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.421 | 3.188 | 5.775 | 9.141 | 7.280 | 15.638 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.164 | 3.885 | 4.854 | 8.795 | 5.005 | 9.756 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.358 | 0.671 | 0.879 | 1.912 | 1.022 | 2.140 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.886 | 10.919 | 13.557 | 23.747 | 20.394 | 51.410 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.166 | 5.002 | 5.924 | 12.573 | 9.339 | 26.697 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.224 | 30.180 | 57.102 | 80.662 | 52.364 | 146.099 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.933 | 3.171 | 4.016 | 8.126 | 6.192 | 15.645 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 42.977 | 57.958 | 60.369 | 113.993 | 73.449 | 148.170 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.218 | 4.607 | 4.771 | 10.437 | 7.792 | 22.109 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.549 | 23.157 | 55.602 | 99.338 | 43.392 | 131.272 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.683 | 1.337 | 1.974 | 3.590 | 2.386 | 4.419 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.148 | 41.930 | 63.860 | 110.317 | 92.291 | 247.816 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.201 | 2.488 | 5.472 | 6.083 | 4.116 | 7.054 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 42.148 | 74.641 | 145.167 | 175.991 | 148.362 | 334.894 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.943 | 1.941 | 3.412 | 5.037 | 3.442 | 6.425 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 89.452 | 158.634 | 243.717 | 340.945 | 283.163 | 644.731 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.674 | 3.631 | 9.872 | 9.159 | 6.749 | 13.649 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 46.427 | 89.859 | 145.921 | 209.404 | 177.372 | 394.635 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.060 | 0.066 | 0.099 | 0.827 | 0.222 | 0.643 |
| comment-links | 278 | gfm | yes | yes | 0.236 | 0.316 | 0.421 | 1.318 | 0.560 | 1.354 |
| comment-quote | 290 | gfm | yes | yes | 0.148 | 0.158 | 0.194 | 1.048 | 0.412 | 1.185 |
| comment-inline-code | 285 | gfm | yes | yes | 0.147 | 0.163 | 0.362 | 1.109 | 0.486 | 1.181 |
| comment-table | 310 | gfm | yes | yes | 0.694 | 0.696 | 0.663 | 2.059 | 0.855 | 1.698 |
| comment-incident | 1124 | gfm | no | yes | 0.806 | 0.932 | 0.969 | 2.644 | 1.676 | 4.419 |
| legacy-contributing | 9323 | gfm | no | yes | 6.818 | 8.485 | 11.806 | 22.194 | 16.853 | 39.824 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.270 | 1.780 | 2.775 | 5.230 | 3.442 | 7.149 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.838 | 6.313 | 10.606 | 18.274 | 12.384 | 25.931 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.296 | 4.035 | 7.121 | 10.246 | 8.850 | 17.124 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.385 | 3.086 | 5.372 | 8.970 | 6.985 | 15.684 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.111 | 3.793 | 4.255 | 8.689 | 4.843 | 9.795 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.311 | 0.570 | 0.704 | 1.831 | 0.922 | 2.126 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.836 | 10.788 | 12.505 | 23.209 | 19.855 | 52.090 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.112 | 4.894 | 5.314 | 12.238 | 9.035 | 26.550 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.255 | 30.136 | 52.973 | 79.303 | 50.870 | 145.830 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.884 | 3.051 | 3.520 | 7.943 | 5.910 | 15.588 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 42.678 | 57.355 | 57.097 | 112.227 | 72.430 | 147.259 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.185 | 4.457 | 4.291 | 10.265 | 7.563 | 22.168 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.585 | 22.980 | 53.454 | 97.846 | 42.344 | 132.162 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.630 | 1.216 | 1.669 | 3.478 | 2.272 | 4.381 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.176 | 41.816 | 59.935 | 108.793 | 91.089 | 247.913 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.145 | 2.384 | 5.151 | 5.897 | 3.962 | 7.004 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 41.879 | 74.908 | 140.443 | 174.753 | 148.213 | 339.292 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.904 | 1.824 | 3.168 | 4.917 | 3.348 | 6.409 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 90.957 | 158.916 | 237.064 | 339.874 | 279.309 | 647.918 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.638 | 3.539 | 9.459 | 8.984 | 6.622 | 13.769 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 47.147 | 89.433 | 139.137 | 204.565 | 173.192 | 392.060 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 277.98 | 460.31 | 738.08 | 916.21 | 755.99 | 1712.96 |
| rotating-commonmark | reuse | 8 | 269.83 | 449.46 | 719.46 | 910.45 | 748.62 | 1713.03 |
| rotating-gfm | fresh | 20 | 180.48 | 258.50 | 384.60 | 524.61 | 354.87 | 799.55 |
| rotating-gfm | reuse | 20 | 175.87 | 248.54 | 368.57 | 509.32 | 345.21 | 798.97 |
