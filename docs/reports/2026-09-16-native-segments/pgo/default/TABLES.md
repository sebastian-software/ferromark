# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.82× | 0.61× | 0.23× | 0.46× | 0.20× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.48× | 0.29× | 0.40× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.70× | 0.47× | 0.29× | 0.38× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.62× | 0.25× | 0.46× | 0.20× |
| comments (all workloads) | 6 | 1.00× | 0.82× | 0.62× | 0.25× | 0.46× | 0.20× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.24× | 0.30× | 0.16× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.55× | 0.34× | 0.26× | 0.31× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.68× | 0.44× | 0.61× | 0.31× |
| readme (all workloads) | 1 | 1.00× | 0.87× | 0.68× | 0.44× | 0.61× | 0.31× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.48× | 0.32× | 0.50× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.81× | 0.48× | 0.32× | 0.50× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.48× | 0.33× | 0.37× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.72× | 0.48× | 0.33× | 0.37× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.24× | 0.30× | 0.16× |
| commonmark (all workloads) | 8 | 1.00× | 0.55× | 0.34× | 0.26× | 0.31× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.53× | 0.31× | 0.42× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.76× | 0.53× | 0.31× | 0.42× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.58× | 0.23× | 0.44× | 0.20× |
| <512 B (all workloads) | 6 | 1.00× | 0.77× | 0.58× | 0.23× | 0.44× | 0.20× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.44× | 0.30× | 0.38× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.65× | 0.44× | 0.30× | 0.38× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.45× | 0.32× | 0.35× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.69× | 0.45× | 0.32× | 0.35× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.39× | 0.43× | 0.19× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.75× | 0.53× | 0.39× | 0.43× | 0.19× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.34× | 0.22× | 0.42× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.63× | 0.37× | 0.28× | 0.34× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.93× | 0.68× | 0.19× | 0.41× | 0.16× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.52× | 0.28× | 0.39× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.72× | 0.50× | 0.29× | 0.38× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.69× | 0.21× | 0.42× | 0.16× |
| comments (all workloads) | 6 | 1.00× | 0.91× | 0.69× | 0.21× | 0.42× | 0.16× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.23× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.55× | 0.36× | 0.26× | 0.31× | 0.14× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.73× | 0.44× | 0.62× | 0.31× |
| readme (all workloads) | 1 | 1.00× | 0.88× | 0.73× | 0.44× | 0.62× | 0.31× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.49× | 0.33× | 0.51× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.81× | 0.49× | 0.33× | 0.51× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.33× | 0.37× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.72× | 0.51× | 0.33× | 0.37× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.23× | 0.29× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.55× | 0.36× | 0.26× | 0.31× | 0.14× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.57× | 0.29× | 0.41× | 0.17× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.79× | 0.57× | 0.29× | 0.41× | 0.17× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.64× | 0.19× | 0.40× | 0.16× |
| <512 B (all workloads) | 6 | 1.00× | 0.86× | 0.64× | 0.19× | 0.40× | 0.16× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.47× | 0.29× | 0.38× | 0.18× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.65× | 0.47× | 0.29× | 0.38× | 0.18× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.32× | 0.35× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.70× | 0.47× | 0.32× | 0.35× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.55× | 0.39× | 0.44× | 0.19× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.75× | 0.55× | 0.39× | 0.44× | 0.19× |
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
| comment-question | 160 | gfm | yes | yes | 0.116 | 0.161 | 0.181 | 0.859 | 0.286 | 0.799 |
| comment-links | 278 | gfm | yes | yes | 0.342 | 0.464 | 0.686 | 1.396 | 0.744 | 1.642 |
| comment-quote | 290 | gfm | yes | yes | 0.228 | 0.281 | 0.356 | 1.154 | 0.578 | 1.499 |
| comment-inline-code | 285 | gfm | yes | yes | 0.232 | 0.280 | 0.593 | 1.198 | 0.679 | 1.435 |
| comment-table | 310 | gfm | yes | yes | 0.991 | 0.982 | 0.970 | 2.131 | 1.199 | 2.181 |
| comment-incident | 1124 | gfm | no | yes | 1.049 | 1.291 | 1.496 | 2.777 | 2.167 | 5.579 |
| legacy-contributing | 9323 | gfm | no | yes | 8.757 | 10.886 | 16.318 | 22.890 | 22.237 | 48.667 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.680 | 2.383 | 4.108 | 5.452 | 4.723 | 9.020 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.784 | 7.488 | 13.892 | 18.777 | 16.389 | 33.410 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.201 | 5.132 | 9.799 | 10.719 | 11.807 | 21.270 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.075 | 3.962 | 7.369 | 9.281 | 9.316 | 19.199 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.899 | 4.463 | 5.739 | 8.918 | 6.414 | 12.484 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.466 | 0.784 | 1.044 | 1.950 | 1.283 | 2.595 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.303 | 13.120 | 16.907 | 23.861 | 25.597 | 60.789 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.172 | 6.291 | 7.209 | 12.558 | 11.304 | 32.556 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.467 | 35.314 | 67.590 | 81.680 | 62.682 | 145.861 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.502 | 3.891 | 4.554 | 8.201 | 7.295 | 19.385 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 55.442 | 73.359 | 81.456 | 114.961 | 92.395 | 191.407 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.057 | 5.646 | 5.766 | 10.519 | 9.317 | 27.318 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.494 | 24.888 | 64.044 | 98.490 | 50.992 | 143.887 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.864 | 1.587 | 2.116 | 3.631 | 2.738 | 5.272 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 30.994 | 51.087 | 76.584 | 112.536 | 102.812 | 291.395 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.502 | 2.915 | 5.306 | 6.059 | 4.733 | 8.100 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 54.659 | 94.222 | 161.079 | 181.374 | 169.314 | 385.552 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.201 | 2.277 | 3.631 | 5.096 | 4.047 | 7.424 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 110.363 | 185.723 | 268.740 | 345.671 | 316.721 | 740.879 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.065 | 4.170 | 8.963 | 9.170 | 7.603 | 15.676 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 61.382 | 108.524 | 163.722 | 210.673 | 195.875 | 443.885 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.073 | 0.077 | 0.104 | 0.831 | 0.259 | 0.802 |
| comment-links | 278 | gfm | yes | yes | 0.291 | 0.375 | 0.498 | 1.324 | 0.662 | 1.664 |
| comment-quote | 290 | gfm | yes | yes | 0.177 | 0.186 | 0.233 | 1.048 | 0.475 | 1.478 |
| comment-inline-code | 285 | gfm | yes | yes | 0.186 | 0.195 | 0.454 | 1.114 | 0.603 | 1.429 |
| comment-table | 310 | gfm | yes | yes | 0.937 | 0.895 | 0.840 | 2.091 | 1.130 | 2.224 |
| comment-incident | 1124 | gfm | no | yes | 0.995 | 1.181 | 1.293 | 2.658 | 2.042 | 5.661 |
| legacy-contributing | 9323 | gfm | no | yes | 8.690 | 10.685 | 15.381 | 22.197 | 21.599 | 48.508 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.616 | 2.275 | 3.786 | 5.260 | 4.541 | 9.064 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.754 | 7.327 | 13.360 | 18.307 | 15.896 | 33.628 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.130 | 5.010 | 9.196 | 10.354 | 11.292 | 21.176 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.985 | 3.826 | 6.889 | 8.999 | 8.950 | 19.320 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.842 | 4.385 | 5.271 | 8.721 | 6.186 | 12.552 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.405 | 0.681 | 0.833 | 1.844 | 1.179 | 2.597 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.184 | 13.059 | 15.606 | 23.148 | 24.861 | 60.513 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.166 | 6.197 | 6.850 | 12.433 | 11.172 | 33.030 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.565 | 35.699 | 65.088 | 80.608 | 60.902 | 146.876 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.458 | 3.820 | 4.214 | 8.058 | 7.141 | 19.610 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 55.018 | 71.821 | 78.027 | 113.329 | 91.276 | 189.667 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.961 | 5.436 | 5.244 | 10.230 | 8.978 | 27.216 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.486 | 24.897 | 62.604 | 98.341 | 50.210 | 145.967 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.794 | 1.462 | 1.763 | 3.489 | 2.606 | 5.262 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 30.939 | 50.517 | 72.557 | 110.505 | 101.098 | 289.787 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.447 | 2.800 | 4.965 | 5.919 | 4.616 | 8.166 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 53.085 | 91.632 | 154.780 | 176.236 | 164.286 | 380.375 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.130 | 2.166 | 3.235 | 4.921 | 3.921 | 7.436 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 113.097 | 184.611 | 260.922 | 340.383 | 310.329 | 739.039 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.021 | 4.080 | 8.489 | 8.969 | 7.429 | 15.767 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 61.095 | 108.091 | 159.878 | 206.239 | 192.202 | 443.275 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 324.04 | 516.21 | 792.01 | 925.45 | 828.56 | 1944.57 |
| rotating-commonmark | reuse | 8 | 316.84 | 503.00 | 764.29 | 914.32 | 819.32 | 1942.69 |
| rotating-gfm | fresh | 20 | 214.51 | 288.29 | 433.07 | 527.13 | 407.06 | 893.07 |
| rotating-gfm | reuse | 20 | 209.26 | 279.60 | 410.72 | 514.61 | 399.03 | 900.56 |
