# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.91× | 0.67× | 0.25× | 0.52× | 0.24× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.53× | 0.32× | 0.44× | 0.21× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.76× | 0.51× | 0.32× | 0.42× | 0.20× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.71× | 0.28× | 0.53× | 0.24× |
| comments (all workloads) | 6 | 1.00× | 0.93× | 0.71× | 0.28× | 0.53× | 0.24× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.25× | 0.32× | 0.18× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.60× | 0.36× | 0.28× | 0.33× | 0.16× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.70× | 0.45× | 0.63× | 0.33× |
| readme (all workloads) | 1 | 1.00× | 0.90× | 0.70× | 0.45× | 0.63× | 0.33× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.54× | 0.37× | 0.57× | 0.24× |
| reference (all workloads) | 2 | 1.00× | 0.92× | 0.54× | 0.37× | 0.57× | 0.24× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.53× | 0.36× | 0.40× | 0.19× |
| technical-docs (all workloads) | 11 | 1.00× | 0.78× | 0.53× | 0.36× | 0.40× | 0.19× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.25× | 0.32× | 0.18× |
| commonmark (all workloads) | 8 | 1.00× | 0.60× | 0.36× | 0.28× | 0.33× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.58× | 0.34× | 0.46× | 0.21× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.84× | 0.58× | 0.34× | 0.46× | 0.21× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.63× | 0.25× | 0.49× | 0.23× |
| <512 B (all workloads) | 6 | 1.00× | 0.85× | 0.63× | 0.25× | 0.49× | 0.23× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.48× | 0.32× | 0.41× | 0.22× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.70× | 0.48× | 0.32× | 0.41× | 0.22× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.36× | 0.39× | 0.18× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.77× | 0.50× | 0.36× | 0.39× | 0.18× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.58× | 0.43× | 0.48× | 0.22× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.83× | 0.58× | 0.43× | 0.48× | 0.22× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.33× | 0.22× | 0.43× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.69× | 0.40× | 0.30× | 0.37× | 0.15× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 1.06× | 0.77× | 0.21× | 0.47× | 0.19× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.57× | 0.31× | 0.43× | 0.19× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.79× | 0.55× | 0.31× | 0.42× | 0.19× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.81× | 0.25× | 0.49× | 0.20× |
| comments (all workloads) | 6 | 1.00× | 1.07× | 0.81× | 0.25× | 0.49× | 0.20× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.34× | 0.24× | 0.31× | 0.17× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.59× | 0.38× | 0.28× | 0.33× | 0.16× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.74× | 0.45× | 0.65× | 0.33× |
| readme (all workloads) | 1 | 1.00× | 0.91× | 0.74× | 0.45× | 0.65× | 0.33× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.56× | 0.37× | 0.58× | 0.24× |
| reference (all workloads) | 2 | 1.00× | 0.92× | 0.56× | 0.37× | 0.58× | 0.24× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.56× | 0.37× | 0.41× | 0.18× |
| technical-docs (all workloads) | 11 | 1.00× | 0.79× | 0.56× | 0.37× | 0.41× | 0.18× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.34× | 0.24× | 0.31× | 0.17× |
| commonmark (all workloads) | 8 | 1.00× | 0.59× | 0.38× | 0.28× | 0.33× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.63× | 0.33× | 0.46× | 0.20× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.88× | 0.63× | 0.33× | 0.46× | 0.20× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.72× | 0.22× | 0.45× | 0.19× |
| <512 B (all workloads) | 6 | 1.00× | 0.97× | 0.72× | 0.22× | 0.45× | 0.19× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.51× | 0.32× | 0.41× | 0.21× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.71× | 0.51× | 0.32× | 0.41× | 0.21× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.53× | 0.36× | 0.40× | 0.17× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.78× | 0.53× | 0.36× | 0.40× | 0.17× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.61× | 0.44× | 0.49× | 0.22× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.84× | 0.61× | 0.44× | 0.49× | 0.22× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.34× | 0.22× | 0.44× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.68× | 0.40× | 0.30× | 0.37× | 0.15× |

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
| comment-question | 160 | gfm | yes | yes | 0.130 | 0.166 | 0.183 | 0.888 | 0.294 | 0.779 |
| comment-links | 278 | gfm | yes | yes | 0.361 | 0.479 | 0.706 | 1.450 | 0.759 | 1.616 |
| comment-quote | 290 | gfm | yes | yes | 0.324 | 0.280 | 0.358 | 1.167 | 0.579 | 1.438 |
| comment-inline-code | 285 | gfm | yes | yes | 0.250 | 0.287 | 0.608 | 1.219 | 0.683 | 1.384 |
| comment-table | 310 | gfm | yes | yes | 1.031 | 1.006 | 0.997 | 2.187 | 1.224 | 2.164 |
| comment-incident | 1124 | gfm | no | yes | 1.394 | 1.332 | 1.534 | 2.851 | 2.225 | 5.442 |
| legacy-contributing | 9323 | gfm | no | yes | 9.769 | 11.060 | 16.635 | 23.230 | 22.419 | 47.182 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.742 | 2.455 | 4.239 | 5.608 | 4.849 | 8.897 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.994 | 7.663 | 14.250 | 19.170 | 16.840 | 33.046 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 5.063 | 5.220 | 10.139 | 10.822 | 11.936 | 20.624 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.200 | 4.050 | 7.527 | 9.399 | 9.484 | 18.445 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.095 | 4.562 | 5.871 | 9.062 | 6.498 | 12.288 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.492 | 0.809 | 1.078 | 2.000 | 1.303 | 2.558 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.300 | 13.466 | 17.305 | 24.134 | 26.131 | 59.668 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 5.017 | 6.460 | 7.459 | 12.965 | 11.588 | 31.494 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.215 | 35.727 | 68.254 | 82.491 | 63.069 | 144.933 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.821 | 3.978 | 4.663 | 8.398 | 7.470 | 18.656 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 70.369 | 72.309 | 82.022 | 116.020 | 93.132 | 189.720 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.593 | 5.752 | 5.925 | 10.729 | 9.478 | 26.516 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.772 | 25.076 | 65.233 | 98.633 | 50.994 | 144.719 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.928 | 1.622 | 2.205 | 3.736 | 2.798 | 5.165 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 33.861 | 51.705 | 79.179 | 114.106 | 104.962 | 287.350 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.623 | 2.970 | 5.415 | 6.211 | 4.833 | 7.956 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 58.988 | 93.769 | 165.234 | 183.655 | 170.277 | 387.522 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.294 | 2.319 | 3.659 | 5.204 | 4.091 | 7.216 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 125.307 | 187.114 | 273.515 | 348.674 | 318.201 | 728.261 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.221 | 4.300 | 9.191 | 9.440 | 7.775 | 15.296 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 69.046 | 108.646 | 165.393 | 213.362 | 198.605 | 437.248 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.084 | 0.078 | 0.105 | 0.857 | 0.261 | 0.773 |
| comment-links | 278 | gfm | yes | yes | 0.309 | 0.386 | 0.520 | 1.386 | 0.680 | 1.630 |
| comment-quote | 290 | gfm | yes | yes | 0.274 | 0.191 | 0.239 | 1.086 | 0.498 | 1.445 |
| comment-inline-code | 285 | gfm | yes | yes | 0.200 | 0.198 | 0.458 | 1.137 | 0.608 | 1.387 |
| comment-table | 310 | gfm | yes | yes | 0.984 | 0.918 | 0.857 | 2.133 | 1.140 | 2.179 |
| comment-incident | 1124 | gfm | no | yes | 1.338 | 1.222 | 1.333 | 2.739 | 2.103 | 5.416 |
| legacy-contributing | 9323 | gfm | no | yes | 9.822 | 10.829 | 15.887 | 22.510 | 21.922 | 47.565 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.672 | 2.354 | 3.911 | 5.405 | 4.644 | 8.901 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.903 | 7.486 | 13.824 | 18.879 | 16.068 | 32.660 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 5.019 | 5.123 | 9.496 | 10.521 | 11.555 | 20.630 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.134 | 3.967 | 7.133 | 9.220 | 9.176 | 18.801 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.038 | 4.447 | 5.446 | 8.926 | 6.258 | 12.413 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.432 | 0.694 | 0.861 | 1.894 | 1.188 | 2.559 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.073 | 13.245 | 15.910 | 23.325 | 25.120 | 58.829 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 5.015 | 6.348 | 7.058 | 12.775 | 11.313 | 32.102 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.158 | 35.691 | 65.070 | 81.015 | 61.674 | 143.961 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.742 | 3.833 | 4.227 | 8.107 | 7.153 | 18.687 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 71.994 | 74.393 | 79.335 | 115.694 | 92.582 | 189.645 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.579 | 5.573 | 5.420 | 10.501 | 9.196 | 26.667 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.965 | 25.270 | 64.359 | 99.220 | 49.913 | 148.082 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.858 | 1.492 | 1.817 | 3.606 | 2.663 | 5.135 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 33.320 | 51.508 | 74.783 | 111.097 | 103.013 | 284.878 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.587 | 2.892 | 5.121 | 6.162 | 4.745 | 8.128 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 59.762 | 92.485 | 161.046 | 181.235 | 168.823 | 380.627 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.239 | 2.200 | 3.317 | 5.085 | 3.971 | 7.218 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 122.715 | 188.550 | 268.226 | 347.441 | 314.861 | 728.826 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.174 | 4.186 | 8.703 | 9.196 | 7.651 | 15.368 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 67.224 | 109.574 | 165.069 | 211.983 | 196.547 | 442.779 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 343.45 | 526.06 | 796.29 | 941.49 | 839.80 | 1927.96 |
| rotating-commonmark | reuse | 8 | 339.63 | 515.55 | 770.25 | 933.75 | 833.86 | 1934.96 |
| rotating-gfm | fresh | 20 | 245.91 | 294.09 | 442.20 | 532.35 | 414.24 | 903.99 |
| rotating-gfm | reuse | 20 | 239.77 | 285.06 | 421.89 | 522.83 | 407.35 | 904.31 |
