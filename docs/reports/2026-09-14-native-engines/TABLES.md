# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.97× | 0.72× | 0.38× | 0.51× | 0.23× |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.00× | 0.81× | 0.26× | 0.46× | 0.17× |
| comments | 12 | 1.00× | 1.00× | 0.83× | 0.29× | 0.57× | 0.26× |
| encyclopedia | 12 | 1.00× | 0.90× | 0.55× | 0.42× | 0.50× | 0.25× |
| plain-prose | 4 | 1.00× | 1.00× | 0.80× | 0.25× | 0.30× | 0.06× |
| readme | 2 | 1.00× | 0.98× | 0.76× | 0.46× | 0.58× | 0.29× |
| reference | 4 | 1.00× | 1.01× | 0.97× | 0.47× | 0.66× | 0.31× |
| syntax-guard | 1 | 1.00× | 0.96× | 0.57× | 0.28× | 0.77× | 0.59× |
| technical-docs | 22 | 1.00× | 0.98× | 0.71× | 0.44× | 0.50× | 0.22× |
| commonmark | 17 | 1.00× | 0.93× | 0.60× | 0.37× | 0.45× | 0.18× |
| gfm (shared subset) | 40 | 1.00× | 0.99× | 0.77× | 0.39× | 0.54× | 0.24× |
| <512 B | 12 | 1.00× | 0.99× | 0.79× | 0.28× | 0.59× | 0.29× |
| 512 B–2 KiB | 9 | 1.00× | 0.95× | 0.69× | 0.42× | 0.53× | 0.26× |
| 2–8 KiB | 15 | 1.00× | 0.96× | 0.63× | 0.43× | 0.49× | 0.23× |
| 8–32 KiB | 9 | 1.00× | 0.97× | 0.73× | 0.47× | 0.53× | 0.24× |
| 32–128 KiB | 12 | 1.00× | 0.98× | 0.77× | 0.36× | 0.43× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.97× | 0.75× | 0.36× | 0.48× | 0.20× |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.99× | 0.84× | 0.21× | 0.39× | 0.13× |
| comments | 12 | 1.00× | 0.99× | 0.83× | 0.21× | 0.45× | 0.18× |
| encyclopedia | 12 | 1.00× | 0.90× | 0.57× | 0.42× | 0.49× | 0.24× |
| plain-prose | 4 | 1.00× | 0.99× | 0.92× | 0.26× | 0.31× | 0.06× |
| readme | 2 | 1.00× | 0.99× | 0.80× | 0.46× | 0.58× | 0.28× |
| reference | 4 | 1.00× | 1.01× | 1.00× | 0.47× | 0.67× | 0.31× |
| syntax-guard | 1 | 1.00× | 0.95× | 0.61× | 0.19× | 0.58× | 0.39× |
| technical-docs | 22 | 1.00× | 0.98× | 0.75× | 0.43× | 0.50× | 0.22× |
| commonmark | 17 | 1.00× | 0.92× | 0.64× | 0.36× | 0.45× | 0.18× |
| gfm (shared subset) | 40 | 1.00× | 0.99× | 0.79× | 0.35× | 0.50× | 0.22× |
| <512 B | 12 | 1.00× | 0.98× | 0.79× | 0.20× | 0.46× | 0.20× |
| 512 B–2 KiB | 9 | 1.00× | 0.95× | 0.73× | 0.40× | 0.51× | 0.24× |
| 2–8 KiB | 15 | 1.00× | 0.96× | 0.66× | 0.43× | 0.49× | 0.22× |
| 8–32 KiB | 9 | 1.00× | 0.97× | 0.77× | 0.47× | 0.54× | 0.24× |
| 32–128 KiB | 12 | 1.00× | 0.97× | 0.82× | 0.37× | 0.45× | 0.14× |

## HTML agreement with v2

| Engine | Exact | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 57 | 0 | 0 | 0 |
| OX-Content original | 57 | 0 | 0 | 0 |
| Ferromark v1 | 10 | 5 | 35 | 7 |
| md4c | 11 | 4 | 35 | 7 |
| pulldown-cmark | 9 | 7 | 39 | 2 |
| Bun native bun_md | 10 | 5 | 39 | 3 |

## Per-document timings

Microseconds per complete Markdown→HTML operation; lower is faster. “Agree” requires all six engines.
Original input sizes are UTF-8 bytes. “gfm” is the shared subset described in the harness README.

### fresh

| Document | Bytes | Profile | Agree | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | 0.158 | 0.156 | 0.159 | 0.777 | 0.187 | 0.293 |
| comment-question | 160 | gfm | yes | 0.169 | 0.168 | 0.180 | 0.886 | 0.296 | 0.780 |
| comment-review | 282 | gfm | yes | 0.218 | 0.220 | 0.266 | 1.071 | 0.496 | 1.327 |
| comment-links | 278 | gfm | yes | 0.458 | 0.478 | 0.645 | 1.436 | 0.760 | 1.623 |
| comment-checklist | 287 | gfm | no | 0.615 | 0.621 | 0.751 | 1.583 | 0.917 | 1.824 |
| comment-quote | 290 | gfm | yes | 0.285 | 0.283 | 0.327 | 1.171 | 0.575 | 1.448 |
| comment-unicode | 327 | gfm | yes | 0.439 | 0.440 | 0.643 | 1.370 | 0.821 | 1.709 |
| comment-inline-code | 285 | gfm | yes | 0.293 | 0.293 | 0.483 | 1.241 | 0.692 | 1.410 |
| comment-reproduction | 298 | gfm | yes | 0.332 | 0.330 | 0.412 | 1.371 | 0.608 | 1.413 |
| comment-table | 310 | gfm | yes | 1.014 | 1.014 | 0.913 | 2.209 | 1.250 | 2.198 |
| comment-review-long | 957 | gfm | yes | 0.761 | 0.775 | 0.974 | 2.225 | 1.666 | 4.541 |
| comment-incident | 1124 | gfm | no | 1.353 | 1.332 | 1.397 | 2.871 | 2.232 | 5.515 |
| guard-angle-link | 41 | commonmark | no | 0.278 | 0.289 | 0.485 | 0.993 | 0.360 | 0.468 |
| legacy-contributing | 9323 | gfm | no | 10.951 | 11.081 | 15.459 | 23.413 | 22.455 | 47.691 |
| legacy-node-ferromark-readme | 9075 | gfm | no | 10.871 | 11.116 | 15.802 | 25.642 | 22.613 | 47.523 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | 2.435 | 2.458 | 3.703 | 5.614 | 4.813 | 8.969 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | 3.064 | 3.083 | 3.931 | 6.592 | 5.422 | 10.839 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | 7.684 | 7.690 | 12.246 | 19.258 | 16.476 | 33.070 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | 3.083 | 3.076 | 4.499 | 7.077 | 5.830 | 10.699 |
| legacy-docs-releasing | 4141 | gfm | no | 5.294 | 5.261 | 9.134 | 10.915 | 11.870 | 21.096 |
| legacy-docs-readme-theme | 2848 | gfm | no | 2.551 | 2.607 | 4.253 | 6.894 | 6.427 | 13.748 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | 4.007 | 4.042 | 6.659 | 9.426 | 9.403 | 18.806 |
| legacy-docs-mdx | 7422 | gfm | no | 7.752 | 7.861 | 10.962 | 19.547 | 15.619 | 36.525 |
| legacy-docs-readme | 1825 | gfm | no | 4.561 | 4.639 | 5.482 | 9.192 | 6.574 | 12.420 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | 1.608 | 1.593 | 2.319 | 3.847 | 3.511 | 7.715 |
| rust-book-ch03-04-comments | 393 | gfm | no | 0.757 | 0.810 | 1.010 | 1.992 | 1.335 | 2.572 |
| rust-book-appendix-02-operators | 22595 | gfm | no | 50.702 | 50.416 | 58.751 | 81.347 | 65.423 | 106.477 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | 13.613 | 13.711 | 16.569 | 24.453 | 26.162 | 59.967 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | 7.167 | 7.152 | 11.527 | 18.322 | 18.373 | 50.169 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | 6.220 | 6.515 | 6.873 | 13.059 | 11.693 | 32.237 |
| vue-docs-suspense | 8291 | gfm | no | 9.931 | 10.028 | 15.078 | 22.797 | 20.373 | 44.471 |
| vue-docs-slots | 24211 | gfm | no | 32.394 | 36.232 | 58.008 | 83.572 | 64.299 | 146.248 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | 27.166 | 29.081 | 36.432 | 68.833 | 55.451 | 140.474 |
| vite-docs-philosophy | 3575 | gfm | no | 3.799 | 3.965 | 4.447 | 8.403 | 7.478 | 18.992 |
| vite-docs-performance | 8184 | gfm | no | 9.092 | 9.302 | 11.420 | 19.866 | 18.456 | 43.130 |
| vite-docs-api-plugin | 31890 | gfm | no | 73.112 | 74.183 | 70.671 | 118.385 | 94.336 | 193.190 |
| vite-docs-features | 39739 | gfm | no | 70.235 | 71.949 | 91.332 | 144.616 | 123.623 | 250.898 |
| typescript-handbook-the-handbook | 5337 | gfm | no | 5.609 | 5.793 | 5.582 | 10.812 | 9.559 | 26.790 |
| typescript-handbook-advanced-types | 36745 | gfm | no | 60.404 | 57.066 | 58.488 | 123.694 | 93.532 | 194.750 |
| typescript-handbook-compiler-options | 54026 | gfm | no | 25.669 | 25.910 | 26.900 | 101.742 | 52.712 | 150.341 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | 71.819 | 69.889 | 94.102 | 172.779 | 131.720 | 305.164 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | 1.454 | 1.619 | 2.138 | 3.708 | 2.808 | 5.152 |
| wiki-rainbow-lead | 1859 | commonmark | no | 2.271 | 2.474 | 3.171 | 5.834 | 4.768 | 10.080 |
| wiki-rainbow-article-body | 48422 | commonmark | no | 47.902 | 51.890 | 69.941 | 114.505 | 105.773 | 285.911 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | 9.112 | 9.109 | 12.139 | 43.960 | 37.495 | 193.169 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | 2.702 | 3.015 | 5.590 | 6.294 | 4.873 | 8.095 |
| wiki-tea-lead | 6363 | commonmark | no | 12.776 | 14.305 | 40.251 | 28.496 | 24.923 | 43.663 |
| wiki-tea-article-body | 58814 | commonmark | no | 86.554 | 93.572 | 152.720 | 184.067 | 171.512 | 382.799 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | 12.231 | 12.297 | 14.813 | 46.827 | 40.335 | 207.362 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | 2.071 | 2.310 | 3.699 | 5.192 | 4.110 | 7.207 |
| wiki-chess-lead | 4125 | commonmark | no | 6.458 | 7.307 | 11.073 | 15.717 | 13.536 | 25.380 |
| wiki-chess-article-body | 113609 | commonmark | no | 174.130 | 190.577 | 251.519 | 355.148 | 325.130 | 741.833 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | 28.780 | 29.028 | 34.366 | 101.097 | 86.041 | 429.329 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | 3.758 | 4.301 | 9.117 | 9.420 | 7.784 | 15.266 |
| wiki-volcano-lead | 4232 | commonmark | no | 5.018 | 5.692 | 10.877 | 12.966 | 11.102 | 23.739 |
| wiki-volcano-article-body | 69241 | commonmark | no | 102.358 | 111.366 | 158.773 | 218.510 | 202.814 | 444.716 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | 15.609 | 15.626 | 19.663 | 57.629 | 50.596 | 241.994 |

### reuse

| Document | Bytes | Profile | Agree | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | 0.068 | 0.068 | 0.085 | 0.750 | 0.155 | 0.293 |
| comment-question | 160 | gfm | yes | 0.079 | 0.079 | 0.099 | 0.854 | 0.264 | 0.781 |
| comment-review | 282 | gfm | yes | 0.131 | 0.131 | 0.176 | 1.016 | 0.448 | 1.330 |
| comment-links | 278 | gfm | yes | 0.360 | 0.385 | 0.457 | 1.356 | 0.680 | 1.625 |
| comment-checklist | 287 | gfm | no | 0.518 | 0.523 | 0.619 | 1.444 | 0.832 | 1.815 |
| comment-quote | 290 | gfm | yes | 0.189 | 0.190 | 0.225 | 1.074 | 0.485 | 1.437 |
| comment-unicode | 327 | gfm | yes | 0.340 | 0.341 | 0.405 | 1.300 | 0.749 | 1.702 |
| comment-inline-code | 285 | gfm | yes | 0.202 | 0.201 | 0.334 | 1.147 | 0.613 | 1.406 |
| comment-reproduction | 298 | gfm | yes | 0.239 | 0.238 | 0.317 | 1.250 | 0.535 | 1.398 |
| comment-table | 310 | gfm | yes | 0.916 | 0.922 | 0.789 | 2.128 | 1.149 | 2.190 |
| comment-review-long | 957 | gfm | yes | 0.655 | 0.662 | 0.771 | 2.106 | 1.571 | 4.544 |
| comment-incident | 1124 | gfm | no | 1.218 | 1.216 | 1.218 | 2.711 | 2.081 | 5.434 |
| guard-angle-link | 41 | commonmark | no | 0.181 | 0.191 | 0.295 | 0.944 | 0.314 | 0.466 |
| legacy-contributing | 9323 | gfm | no | 10.867 | 11.027 | 14.718 | 23.076 | 21.910 | 48.130 |
| legacy-node-ferromark-readme | 9075 | gfm | no | 10.790 | 10.993 | 14.739 | 25.241 | 22.107 | 47.602 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | 2.333 | 2.345 | 3.461 | 5.440 | 4.678 | 8.942 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | 2.946 | 2.964 | 3.569 | 6.369 | 5.246 | 10.887 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | 7.567 | 7.521 | 11.750 | 18.976 | 16.051 | 33.367 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | 2.977 | 2.975 | 4.106 | 6.881 | 5.691 | 10.686 |
| legacy-docs-releasing | 4141 | gfm | no | 5.151 | 5.094 | 8.593 | 10.580 | 11.477 | 20.880 |
| legacy-docs-readme-theme | 2848 | gfm | no | 2.442 | 2.474 | 3.835 | 6.680 | 6.150 | 13.823 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | 3.908 | 3.984 | 6.318 | 9.275 | 9.206 | 18.884 |
| legacy-docs-mdx | 7422 | gfm | no | 7.646 | 7.702 | 10.143 | 19.292 | 15.171 | 36.875 |
| legacy-docs-readme | 1825 | gfm | no | 4.427 | 4.476 | 5.082 | 8.947 | 6.337 | 12.362 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | 1.484 | 1.491 | 2.044 | 3.695 | 3.362 | 7.678 |
| rust-book-ch03-04-comments | 393 | gfm | no | 0.639 | 0.695 | 0.817 | 1.880 | 1.210 | 2.556 |
| rust-book-appendix-02-operators | 22595 | gfm | no | 50.395 | 50.129 | 56.769 | 80.899 | 64.441 | 107.427 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | 13.425 | 13.502 | 15.141 | 24.060 | 25.427 | 59.772 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | 6.966 | 6.990 | 10.554 | 18.003 | 17.960 | 49.950 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | 6.054 | 6.352 | 6.461 | 12.761 | 11.350 | 31.885 |
| vue-docs-suspense | 8291 | gfm | no | 9.830 | 9.871 | 14.220 | 22.281 | 19.641 | 44.286 |
| vue-docs-slots | 24211 | gfm | no | 32.367 | 36.553 | 56.081 | 82.720 | 62.150 | 146.386 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | 27.037 | 28.768 | 34.864 | 67.343 | 53.728 | 139.657 |
| vite-docs-philosophy | 3575 | gfm | no | 3.655 | 3.825 | 4.007 | 8.104 | 7.173 | 18.699 |
| vite-docs-performance | 8184 | gfm | no | 8.912 | 9.130 | 10.961 | 19.228 | 17.810 | 43.139 |
| vite-docs-api-plugin | 31890 | gfm | no | 72.130 | 73.151 | 68.968 | 115.253 | 92.567 | 192.008 |
| vite-docs-features | 39739 | gfm | no | 69.709 | 70.377 | 88.147 | 143.093 | 122.306 | 251.273 |
| typescript-handbook-the-handbook | 5337 | gfm | no | 5.378 | 5.591 | 5.089 | 10.538 | 9.199 | 26.770 |
| typescript-handbook-advanced-types | 36745 | gfm | no | 60.601 | 57.720 | 57.145 | 122.273 | 93.320 | 196.219 |
| typescript-handbook-compiler-options | 54026 | gfm | no | 25.420 | 25.612 | 25.251 | 100.040 | 50.990 | 150.339 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | 71.663 | 72.604 | 89.856 | 170.572 | 129.000 | 305.432 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | 1.335 | 1.503 | 1.834 | 3.616 | 2.688 | 5.158 |
| wiki-rainbow-lead | 1859 | commonmark | no | 2.153 | 2.374 | 2.811 | 5.692 | 4.648 | 10.077 |
| wiki-rainbow-article-body | 48422 | commonmark | no | 47.778 | 51.967 | 66.883 | 113.551 | 104.266 | 288.660 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | 8.992 | 9.026 | 10.089 | 42.010 | 35.147 | 193.879 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | 2.583 | 2.866 | 5.222 | 6.059 | 4.741 | 8.063 |
| wiki-tea-lead | 6363 | commonmark | no | 12.652 | 14.136 | 39.310 | 28.123 | 24.627 | 43.563 |
| wiki-tea-article-body | 58814 | commonmark | no | 85.672 | 92.542 | 146.240 | 182.055 | 168.516 | 381.135 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | 12.147 | 12.298 | 12.650 | 45.405 | 38.613 | 207.790 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | 1.959 | 2.215 | 3.409 | 5.078 | 4.005 | 7.266 |
| wiki-chess-lead | 4125 | commonmark | no | 6.287 | 7.107 | 10.229 | 15.239 | 13.079 | 25.340 |
| wiki-chess-article-body | 113609 | commonmark | no | 176.784 | 191.698 | 242.007 | 354.462 | 318.392 | 741.303 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | 28.449 | 28.495 | 30.055 | 96.222 | 81.962 | 427.577 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | 3.664 | 4.193 | 8.685 | 9.341 | 7.692 | 15.404 |
| wiki-volcano-lead | 4232 | commonmark | no | 4.887 | 5.567 | 10.347 | 12.763 | 10.758 | 23.740 |
| wiki-volcano-article-body | 69241 | commonmark | no | 100.335 | 109.849 | 151.604 | 212.542 | 197.140 | 440.857 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | 15.491 | 15.702 | 17.243 | 54.828 | 48.053 | 241.845 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 620.93 | 665.14 | 929.57 | 1304.16 | 1154.52 | 3194.00 |
| rotating-commonmark | reuse | 17 | 600.66 | 642.12 | 877.01 | 1272.02 | 1122.06 | 3176.46 |
| rotating-gfm | fresh | 40 | 861.15 | 869.18 | 1046.61 | 1450.34 | 1194.71 | 2377.71 |
| rotating-gfm | reuse | 40 | 836.74 | 859.43 | 1006.87 | 1442.77 | 1180.33 | 2399.46 |
