# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.77× | 0.59× | 0.26× | 0.52× | 0.30× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.48× | 0.33× | 0.43× | 0.24× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.67× | 0.47× | 0.32× | 0.42× | 0.22× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.60× | 0.29× | 0.52× | 0.30× |
| comments (all workloads) | 6 | 1.00× | 0.76× | 0.60× | 0.29× | 0.52× | 0.30× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.39× | 0.30× | 0.38× | 0.21× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.60× | 0.40× | 0.30× | 0.37× | 0.19× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.64× | 0.48× | 0.63× | 0.38× |
| readme (all workloads) | 1 | 1.00× | 0.78× | 0.64× | 0.48× | 0.63× | 0.38× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.42× | 0.31× | 0.46× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.70× | 0.42× | 0.31× | 0.46× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.46× | 0.35× | 0.39× | 0.21× |
| technical-docs (all workloads) | 11 | 1.00× | 0.67× | 0.46× | 0.35× | 0.39× | 0.21× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.39× | 0.30× | 0.38× | 0.21× |
| commonmark (all workloads) | 8 | 1.00× | 0.60× | 0.40× | 0.30× | 0.37× | 0.19× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.51× | 0.33× | 0.44× | 0.24× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.71× | 0.51× | 0.33× | 0.44× | 0.24× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.57× | 0.27× | 0.50× | 0.29× |
| <512 B (all workloads) | 6 | 1.00× | 0.74× | 0.57× | 0.27× | 0.50× | 0.29× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.48× | 0.36× | 0.44× | 0.26× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.66× | 0.48× | 0.36× | 0.44× | 0.26× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.44× | 0.35× | 0.37× | 0.20× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.66× | 0.44× | 0.35× | 0.37× | 0.20× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.50× | 0.40× | 0.45× | 0.24× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.72× | 0.50× | 0.40× | 0.45× | 0.24× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.28× | 0.19× | 0.34× | 0.12× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.60× | 0.38× | 0.28× | 0.35× | 0.16× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.79× | 0.61× | 0.20× | 0.43× | 0.22× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.51× | 0.31× | 0.42× | 0.21× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.68× | 0.49× | 0.31× | 0.41× | 0.21× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.63× | 0.23× | 0.45× | 0.23× |
| comments (all workloads) | 6 | 1.00× | 0.77× | 0.63× | 0.23× | 0.45× | 0.23× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.41× | 0.30× | 0.38× | 0.20× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.60× | 0.41× | 0.30× | 0.37× | 0.18× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.68× | 0.48× | 0.63× | 0.36× |
| readme (all workloads) | 1 | 1.00× | 0.78× | 0.68× | 0.48× | 0.63× | 0.36× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.43× | 0.31× | 0.47× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.70× | 0.43× | 0.31× | 0.47× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.48× | 0.35× | 0.39× | 0.21× |
| technical-docs (all workloads) | 11 | 1.00× | 0.68× | 0.48× | 0.35× | 0.39× | 0.21× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.41× | 0.30× | 0.38× | 0.20× |
| commonmark (all workloads) | 8 | 1.00× | 0.60× | 0.41× | 0.30× | 0.37× | 0.18× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.53× | 0.31× | 0.42× | 0.22× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.71× | 0.53× | 0.31× | 0.42× | 0.22× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.59× | 0.22× | 0.43× | 0.22× |
| <512 B (all workloads) | 6 | 1.00× | 0.76× | 0.59× | 0.22× | 0.43× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.51× | 0.35× | 0.44× | 0.24× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.66× | 0.51× | 0.35× | 0.44× | 0.24× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.46× | 0.35× | 0.37× | 0.19× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.66× | 0.46× | 0.35× | 0.37× | 0.19× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.52× | 0.41× | 0.45× | 0.24× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.72× | 0.52× | 0.41× | 0.45× | 0.24× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.29× | 0.19× | 0.35× | 0.12× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.60× | 0.39× | 0.28× | 0.35× | 0.16× |

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
| comment-question | 160 | gfm | yes | yes | 0.233 | 0.305 | 0.329 | 1.390 | 0.453 | 0.933 |
| comment-links | 278 | gfm | yes | yes | 0.674 | 0.916 | 1.350 | 2.262 | 1.289 | 2.433 |
| comment-quote | 290 | gfm | yes | yes | 0.406 | 0.542 | 0.673 | 1.803 | 0.890 | 1.778 |
| comment-inline-code | 285 | gfm | yes | yes | 0.432 | 0.538 | 1.148 | 1.885 | 1.134 | 1.679 |
| comment-table | 310 | gfm | yes | yes | 1.553 | 1.958 | 1.760 | 3.346 | 1.932 | 2.802 |
| comment-incident | 1124 | gfm | no | yes | 1.990 | 2.884 | 2.904 | 4.367 | 3.603 | 6.510 |
| legacy-contributing | 9323 | gfm | no | yes | 14.039 | 18.409 | 27.807 | 35.193 | 34.791 | 58.490 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.775 | 4.204 | 6.843 | 8.166 | 7.731 | 12.476 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 9.047 | 12.407 | 23.421 | 28.024 | 25.787 | 44.400 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.411 | 8.855 | 16.587 | 16.505 | 18.314 | 27.787 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.882 | 6.867 | 12.551 | 14.309 | 15.006 | 24.810 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.755 | 8.633 | 10.536 | 14.136 | 10.712 | 18.009 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.942 | 1.557 | 1.984 | 3.101 | 2.186 | 3.672 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.686 | 22.702 | 28.088 | 37.102 | 37.396 | 66.779 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.688 | 11.139 | 12.493 | 19.172 | 16.737 | 36.075 |
| vue-docs-slots | 24211 | gfm | no | yes | 28.398 | 51.011 | 94.168 | 110.665 | 87.367 | 196.097 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.275 | 6.779 | 7.843 | 12.391 | 10.812 | 22.032 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 84.167 | 110.315 | 131.388 | 163.549 | 137.934 | 242.242 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.494 | 9.936 | 10.100 | 16.116 | 14.260 | 29.279 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.473 | 36.218 | 84.493 | 126.367 | 69.001 | 192.110 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.631 | 2.584 | 3.496 | 5.374 | 4.066 | 7.295 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 45.828 | 78.605 | 109.609 | 161.300 | 140.116 | 306.205 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.854 | 4.597 | 7.587 | 8.586 | 7.227 | 11.863 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 80.688 | 133.206 | 209.784 | 254.779 | 222.612 | 465.177 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.228 | 3.782 | 5.447 | 7.527 | 5.705 | 10.363 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 163.060 | 273.216 | 364.388 | 486.335 | 423.190 | 912.583 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.763 | 6.281 | 12.089 | 13.355 | 11.064 | 21.488 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 87.476 | 155.703 | 223.601 | 298.228 | 263.590 | 538.258 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.139 | 0.163 | 0.190 | 1.330 | 0.393 | 0.931 |
| comment-links | 278 | gfm | yes | yes | 0.522 | 0.731 | 0.990 | 2.117 | 1.169 | 2.439 |
| comment-quote | 290 | gfm | yes | yes | 0.295 | 0.376 | 0.438 | 1.680 | 0.761 | 1.778 |
| comment-inline-code | 285 | gfm | yes | yes | 0.304 | 0.376 | 0.875 | 1.728 | 0.989 | 1.682 |
| comment-table | 310 | gfm | yes | yes | 1.425 | 1.755 | 1.477 | 3.171 | 1.748 | 2.786 |
| comment-incident | 1124 | gfm | no | yes | 1.846 | 2.665 | 2.481 | 4.113 | 3.394 | 6.520 |
| legacy-contributing | 9323 | gfm | no | yes | 14.040 | 18.313 | 26.544 | 34.424 | 34.252 | 58.032 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.599 | 3.930 | 6.464 | 7.774 | 7.483 | 12.408 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 8.956 | 12.094 | 22.395 | 27.332 | 25.950 | 44.847 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.302 | 8.451 | 15.362 | 16.054 | 17.541 | 27.632 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.758 | 6.612 | 11.839 | 13.683 | 14.515 | 24.835 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.519 | 8.408 | 9.578 | 13.520 | 10.282 | 18.161 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.821 | 1.351 | 1.608 | 2.884 | 1.950 | 3.693 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.648 | 22.342 | 25.955 | 35.742 | 36.776 | 66.859 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.327 | 10.966 | 11.804 | 18.553 | 16.260 | 35.922 |
| vue-docs-slots | 24211 | gfm | no | yes | 27.941 | 50.693 | 91.300 | 110.051 | 85.440 | 194.830 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.083 | 6.532 | 7.058 | 11.995 | 10.255 | 22.220 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 83.700 | 109.083 | 128.704 | 160.685 | 134.592 | 242.710 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.458 | 9.660 | 9.219 | 15.722 | 13.875 | 29.242 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.231 | 35.910 | 80.618 | 123.778 | 66.047 | 190.237 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.515 | 2.387 | 2.864 | 5.166 | 3.844 | 7.298 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 46.360 | 77.774 | 104.688 | 158.979 | 138.554 | 306.666 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.734 | 4.417 | 6.801 | 8.315 | 6.594 | 11.781 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 78.824 | 132.504 | 203.854 | 250.887 | 220.412 | 464.335 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.043 | 3.537 | 4.789 | 7.131 | 5.450 | 10.416 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 161.880 | 270.345 | 352.733 | 482.786 | 413.927 | 909.187 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.625 | 6.085 | 11.800 | 12.923 | 10.739 | 21.511 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 86.962 | 154.776 | 218.083 | 295.812 | 254.647 | 538.667 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 487.02 | 762.53 | 1033.96 | 1302.40 | 1104.89 | 2400.69 |
| rotating-commonmark | reuse | 8 | 481.46 | 744.61 | 1000.75 | 1272.81 | 1080.71 | 2394.64 |
| rotating-gfm | fresh | 20 | 298.17 | 438.26 | 629.56 | 734.82 | 575.65 | 1104.59 |
| rotating-gfm | reuse | 20 | 281.11 | 413.05 | 605.56 | 719.37 | 565.08 | 1111.78 |
