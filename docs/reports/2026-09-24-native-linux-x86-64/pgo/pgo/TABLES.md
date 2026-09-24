# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.70× | 0.60× | 0.20× | 0.52× | 0.22× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.48× | 0.26× | 0.44× | 0.20× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.65× | 0.45× | 0.26× | 0.42× | 0.18× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.63× | 0.23× | 0.53× | 0.22× |
| comments (all workloads) | 6 | 1.00× | 0.70× | 0.63× | 0.23× | 0.53× | 0.22× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.30× | 0.22× | 0.36× | 0.19× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.56× | 0.32× | 0.23× | 0.34× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.61× | 0.36× | 0.64× | 0.32× |
| readme (all workloads) | 1 | 1.00× | 0.75× | 0.61× | 0.36× | 0.64× | 0.32× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.43× | 0.27× | 0.49× | 0.22× |
| reference (all workloads) | 2 | 1.00× | 0.70× | 0.43× | 0.27× | 0.49× | 0.22× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.48× | 0.29× | 0.41× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.67× | 0.48× | 0.29× | 0.41× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.30× | 0.22× | 0.36× | 0.19× |
| commonmark (all workloads) | 8 | 1.00× | 0.56× | 0.32× | 0.23× | 0.34× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.52× | 0.27× | 0.46× | 0.20× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.68× | 0.52× | 0.27× | 0.46× | 0.20× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.57× | 0.21× | 0.50× | 0.22× |
| <512 B (all workloads) | 6 | 1.00× | 0.68× | 0.57× | 0.21× | 0.50× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.44× | 0.28× | 0.44× | 0.22× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.63× | 0.44× | 0.28× | 0.44× | 0.22× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.44× | 0.27× | 0.38× | 0.16× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.65× | 0.44× | 0.27× | 0.38× | 0.16× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.55× | 0.34× | 0.49× | 0.21× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.55× | 0.34× | 0.49× | 0.21× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.27× | 0.17× | 0.37× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.56× | 0.33× | 0.22× | 0.34× | 0.13× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.83× | 0.66× | 0.17× | 0.48× | 0.18× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.50× | 0.25× | 0.44× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.67× | 0.48× | 0.25× | 0.42× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.69× | 0.20× | 0.50× | 0.18× |
| comments (all workloads) | 6 | 1.00× | 0.81× | 0.69× | 0.20× | 0.50× | 0.18× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.22× | 0.36× | 0.18× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.56× | 0.34× | 0.22× | 0.35× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.65× | 0.37× | 0.65× | 0.31× |
| readme (all workloads) | 1 | 1.00× | 0.75× | 0.65× | 0.37× | 0.65× | 0.31× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.44× | 0.27× | 0.51× | 0.22× |
| reference (all workloads) | 2 | 1.00× | 0.69× | 0.44× | 0.27× | 0.51× | 0.22× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.50× | 0.28× | 0.41× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.67× | 0.50× | 0.28× | 0.41× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.32× | 0.22× | 0.36× | 0.18× |
| commonmark (all workloads) | 8 | 1.00× | 0.56× | 0.34× | 0.22× | 0.35× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.55× | 0.26× | 0.45× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.72× | 0.55× | 0.26× | 0.45× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.62× | 0.18× | 0.47× | 0.18× |
| <512 B (all workloads) | 6 | 1.00× | 0.79× | 0.62× | 0.18× | 0.47× | 0.18× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.47× | 0.27× | 0.44× | 0.21× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.64× | 0.47× | 0.27× | 0.44× | 0.21× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.46× | 0.27× | 0.38× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.65× | 0.46× | 0.27× | 0.38× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.56× | 0.34× | 0.49× | 0.20× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.72× | 0.56× | 0.34× | 0.49× | 0.20× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.28× | 0.18× | 0.39× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.56× | 0.34× | 0.22× | 0.34× | 0.13× |

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
| comment-question | 160 | gfm | yes | yes | 0.171 | 0.279 | 0.260 | 1.367 | 0.338 | 1.042 |
| comment-links | 278 | gfm | yes | yes | 0.486 | 0.720 | 1.023 | 2.263 | 0.980 | 2.241 |
| comment-quote | 290 | gfm | yes | yes | 0.327 | 0.459 | 0.483 | 1.754 | 0.682 | 1.937 |
| comment-inline-code | 285 | gfm | yes | yes | 0.339 | 0.462 | 0.850 | 1.846 | 0.862 | 1.810 |
| comment-table | 310 | gfm | yes | yes | 1.218 | 1.554 | 1.370 | 3.256 | 1.503 | 2.607 |
| comment-incident | 1124 | gfm | no | yes | 1.603 | 2.283 | 2.001 | 4.204 | 2.721 | 6.985 |
| legacy-contributing | 9323 | gfm | no | yes | 11.623 | 14.964 | 20.575 | 34.747 | 26.450 | 57.077 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.158 | 3.321 | 4.980 | 7.995 | 5.852 | 11.738 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 7.452 | 10.244 | 17.950 | 27.955 | 19.675 | 39.901 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 5.242 | 7.408 | 12.788 | 16.472 | 14.025 | 27.515 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.891 | 5.529 | 9.424 | 13.945 | 11.098 | 25.368 |
| legacy-docs-readme | 1825 | gfm | no | yes | 5.164 | 6.930 | 8.468 | 14.253 | 8.132 | 16.127 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.687 | 1.179 | 1.523 | 3.020 | 1.626 | 3.256 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 15.607 | 18.837 | 21.338 | 36.428 | 28.744 | 76.633 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 5.009 | 8.750 | 9.370 | 19.008 | 12.870 | 37.450 |
| vue-docs-slots | 24211 | gfm | no | yes | 24.900 | 43.163 | 78.960 | 110.517 | 67.611 | 169.475 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 3.290 | 5.225 | 6.329 | 12.382 | 8.291 | 22.333 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 69.034 | 89.866 | 101.604 | 163.153 | 105.984 | 216.944 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.174 | 8.112 | 7.720 | 15.947 | 10.600 | 31.953 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.617 | 34.300 | 79.662 | 126.397 | 57.846 | 145.770 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.172 | 2.009 | 3.031 | 5.299 | 3.103 | 6.146 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 36.044 | 63.470 | 95.402 | 163.442 | 110.925 | 344.344 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.077 | 3.544 | 7.472 | 8.497 | 5.339 | 9.116 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 59.779 | 105.657 | 193.105 | 256.355 | 181.164 | 460.668 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.629 | 2.914 | 5.022 | 7.437 | 4.452 | 8.554 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 123.884 | 227.277 | 320.376 | 491.213 | 347.585 | 899.761 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.836 | 4.898 | 12.275 | 13.141 | 8.658 | 18.137 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 64.000 | 127.361 | 199.704 | 302.887 | 215.428 | 535.787 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.119 | 0.137 | 0.169 | 1.323 | 0.302 | 1.040 |
| comment-links | 278 | gfm | yes | yes | 0.409 | 0.553 | 0.760 | 2.147 | 0.862 | 2.246 |
| comment-quote | 290 | gfm | yes | yes | 0.264 | 0.316 | 0.319 | 1.662 | 0.577 | 1.939 |
| comment-inline-code | 285 | gfm | yes | yes | 0.265 | 0.308 | 0.663 | 1.706 | 0.704 | 1.803 |
| comment-table | 310 | gfm | yes | yes | 1.141 | 1.325 | 1.168 | 3.165 | 1.373 | 2.610 |
| comment-incident | 1124 | gfm | no | yes | 1.484 | 2.103 | 1.712 | 4.032 | 2.540 | 6.984 |
| legacy-contributing | 9323 | gfm | no | yes | 11.542 | 14.704 | 19.535 | 33.857 | 25.636 | 56.887 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.062 | 3.100 | 4.564 | 7.608 | 5.479 | 11.780 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 7.293 | 9.845 | 16.790 | 26.931 | 19.164 | 39.433 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 5.113 | 7.098 | 11.975 | 15.825 | 13.674 | 27.239 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.755 | 5.260 | 8.793 | 13.650 | 10.618 | 25.352 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.992 | 6.694 | 7.676 | 13.676 | 7.645 | 15.907 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.590 | 0.971 | 1.241 | 2.861 | 1.463 | 3.254 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 15.457 | 18.475 | 19.574 | 36.197 | 27.671 | 76.295 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.858 | 8.559 | 8.661 | 18.359 | 12.386 | 37.489 |
| vue-docs-slots | 24211 | gfm | no | yes | 22.945 | 43.022 | 76.316 | 109.038 | 65.924 | 170.022 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 3.074 | 5.006 | 5.659 | 11.963 | 7.831 | 22.122 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 68.246 | 90.023 | 98.673 | 160.634 | 104.319 | 216.557 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.065 | 7.774 | 6.933 | 15.571 | 10.245 | 32.195 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.666 | 34.149 | 77.635 | 123.766 | 55.048 | 146.570 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.103 | 1.817 | 2.525 | 5.163 | 2.924 | 6.155 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 35.543 | 63.182 | 90.192 | 160.013 | 109.118 | 347.854 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.972 | 3.373 | 6.802 | 8.367 | 5.071 | 9.094 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 59.633 | 106.855 | 186.980 | 254.408 | 178.361 | 458.472 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.527 | 2.706 | 4.404 | 7.211 | 4.223 | 8.565 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 122.835 | 226.988 | 306.728 | 485.622 | 337.308 | 895.606 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.695 | 4.687 | 11.657 | 12.700 | 8.336 | 18.323 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 63.670 | 125.380 | 192.082 | 296.832 | 205.813 | 538.905 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 412.21 | 656.21 | 955.12 | 1307.79 | 906.27 | 2365.33 |
| rotating-commonmark | reuse | 8 | 398.67 | 641.21 | 916.43 | 1284.20 | 885.20 | 2363.54 |
| rotating-gfm | fresh | 20 | 248.97 | 377.53 | 547.01 | 736.46 | 474.00 | 1064.27 |
| rotating-gfm | reuse | 20 | 226.06 | 355.84 | 513.10 | 714.58 | 457.74 | 1066.94 |
