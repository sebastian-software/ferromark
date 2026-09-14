# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.09× | 0.78× | 0.26× | 0.44× | 0.18× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.66× | 0.37× | 0.46× | 0.23× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 1.00× | 0.66× | 0.38× | 0.47× | 0.24× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.82× | 0.26× | 0.54× | 0.28× |
| comments (all workloads) | 12 | 1.00× | 1.08× | 0.81× | 0.27× | 0.55× | 0.28× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.57× | 0.44× | 0.51× | 0.30× |
| encyclopedia (all workloads) | 12 | 1.00× | 1.00× | 0.60× | 0.46× | 0.51× | 0.27× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.70× | 0.28× | 0.25× | 0.07× |
| plain-prose (all workloads) | 4 | 1.00× | 1.10× | 0.70× | 0.28× | 0.25× | 0.07× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.70× | 0.46× | 0.54× | 0.30× |
| readme (all workloads) | 2 | 1.00× | 0.99× | 0.70× | 0.46× | 0.54× | 0.30× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.66× | 0.46× | 0.62× | 0.31× |
| reference (all workloads) | 4 | 1.00× | 0.98× | 0.66× | 0.46× | 0.62× | 0.31× |
| syntax-guard (all workloads) | 1 | 1.00× | 1.08× | 0.64× | 0.25× | 0.85× | 0.67× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.61× | 0.41× | 0.43× | 0.22× |
| technical-docs (all workloads) | 22 | 1.00× | 0.93× | 0.61× | 0.41× | 0.43× | 0.22× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.61× | 0.38× | 0.40× | 0.18× |
| commonmark (all workloads) | 17 | 1.00× | 1.03× | 0.62× | 0.40× | 0.44× | 0.21× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.67× | 0.36× | 0.48× | 0.25× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.98× | 0.67× | 0.37× | 0.49× | 0.25× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.81× | 0.25× | 0.56× | 0.29× |
| <512 B (all workloads) | 12 | 1.00× | 1.08× | 0.79× | 0.26× | 0.59× | 0.32× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.67× | 0.40× | 0.48× | 0.27× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.97× | 0.67× | 0.40× | 0.48× | 0.27× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.58× | 0.42× | 0.44× | 0.23× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.95× | 0.58× | 0.42× | 0.44× | 0.23× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.65× | 0.47× | 0.49× | 0.25× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.98× | 0.65× | 0.47× | 0.49× | 0.25× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.62× | 0.31× | 0.34× | 0.11× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 1.02× | 0.63× | 0.38× | 0.40× | 0.15× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.12× | 0.82× | 0.21× | 0.37× | 0.14× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.69× | 0.34× | 0.44× | 0.21× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 1.00× | 0.69× | 0.35× | 0.45× | 0.21× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.84× | 0.20× | 0.43× | 0.20× |
| comments (all workloads) | 12 | 1.00× | 1.11× | 0.83× | 0.20× | 0.44× | 0.20× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.60× | 0.44× | 0.50× | 0.29× |
| encyclopedia (all workloads) | 12 | 1.00× | 1.00× | 0.63× | 0.46× | 0.50× | 0.27× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.78× | 0.29× | 0.26× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.10× | 0.78× | 0.29× | 0.26× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.74× | 0.46× | 0.54× | 0.29× |
| readme (all workloads) | 2 | 1.00× | 0.99× | 0.74× | 0.46× | 0.54× | 0.29× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.68× | 0.46× | 0.62× | 0.31× |
| reference (all workloads) | 4 | 1.00× | 0.98× | 0.68× | 0.46× | 0.62× | 0.31× |
| syntax-guard (all workloads) | 1 | 1.00× | 1.10× | 0.70× | 0.18× | 0.64× | 0.45× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.63× | 0.40× | 0.43× | 0.21× |
| technical-docs (all workloads) | 22 | 1.00× | 0.93× | 0.63× | 0.41× | 0.43× | 0.21× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.66× | 0.38× | 0.40× | 0.17× |
| commonmark (all workloads) | 17 | 1.00× | 1.03× | 0.66× | 0.39× | 0.44× | 0.20× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.70× | 0.33× | 0.45× | 0.22× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.99× | 0.70× | 0.34× | 0.46× | 0.22× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.82× | 0.18× | 0.44× | 0.20× |
| <512 B (all workloads) | 12 | 1.00× | 1.11× | 0.81× | 0.19× | 0.46× | 0.23× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.71× | 0.39× | 0.47× | 0.25× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.96× | 0.71× | 0.39× | 0.47× | 0.25× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.61× | 0.42× | 0.44× | 0.23× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.95× | 0.61× | 0.42× | 0.44× | 0.23× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.68× | 0.47× | 0.49× | 0.25× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.98× | 0.68× | 0.47× | 0.49× | 0.25× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.67× | 0.32× | 0.35× | 0.11× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 1.02× | 0.67× | 0.38× | 0.41× | 0.15× |

## HTML agreement with v2

| Engine | Exact | Serialization equivalent | Heading IDs only | Other |
| --- | ---: | ---: | ---: | ---: |
| Ferromark v2 | 57 | 0 | 0 | 0 |
| OX-Content original | 16 | 0 | 39 | 2 |
| Ferromark v1 | 18 | 34 | 0 | 5 |
| md4c | 19 | 33 | 0 | 5 |
| pulldown-cmark | 14 | 43 | 0 | 0 |
| Bun native bun_md | 16 | 40 | 0 | 1 |

## Per-document timings

Microseconds per complete Markdown→HTML operation; lower is faster. Agreement columns show the six/five-engine sets.
Original input sizes are UTF-8 bytes. “gfm” is the shared subset described in the harness README.

### fresh

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.174 | 0.157 | 0.161 | 1.002 | 0.197 | 0.294 |
| comment-question | 160 | gfm | yes | yes | 0.184 | 0.167 | 0.184 | 1.097 | 0.343 | 0.777 |
| comment-review | 282 | gfm | yes | yes | 0.243 | 0.216 | 0.300 | 1.278 | 0.578 | 1.314 |
| comment-links | 278 | gfm | yes | yes | 0.520 | 0.475 | 0.704 | 1.657 | 0.831 | 1.599 |
| comment-checklist | 287 | gfm | no | no | 0.647 | 0.621 | 0.850 | 1.797 | 1.001 | 1.829 |
| comment-quote | 290 | gfm | yes | yes | 0.314 | 0.282 | 0.356 | 1.361 | 0.652 | 1.425 |
| comment-unicode | 327 | gfm | yes | yes | 0.483 | 0.441 | 0.728 | 1.582 | 0.909 | 1.685 |
| comment-inline-code | 285 | gfm | yes | yes | 0.319 | 0.294 | 0.615 | 1.459 | 0.785 | 1.409 |
| comment-reproduction | 298 | gfm | yes | yes | 0.347 | 0.329 | 0.476 | 1.582 | 0.661 | 1.392 |
| comment-table | 310 | gfm | yes | yes | 1.090 | 1.025 | 0.998 | 2.409 | 1.329 | 2.187 |
| comment-review-long | 957 | gfm | yes | yes | 0.815 | 0.772 | 1.038 | 2.425 | 1.937 | 4.405 |
| comment-incident | 1124 | gfm | no | yes | 1.339 | 1.348 | 1.520 | 3.062 | 2.523 | 5.384 |
| guard-angle-link | 41 | commonmark | no | no | 0.308 | 0.284 | 0.480 | 1.211 | 0.364 | 0.461 |
| legacy-contributing | 9323 | gfm | no | yes | 10.782 | 11.157 | 16.624 | 23.635 | 24.756 | 46.725 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 10.633 | 11.061 | 17.538 | 25.511 | 24.655 | 46.434 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.004 | 2.457 | 4.217 | 5.846 | 5.232 | 8.807 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.781 | 3.074 | 4.451 | 6.782 | 5.878 | 10.679 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.963 | 7.616 | 14.091 | 19.264 | 17.744 | 32.359 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.674 | 3.107 | 5.111 | 7.330 | 6.257 | 10.566 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.937 | 5.243 | 9.933 | 11.131 | 12.954 | 20.447 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.550 | 2.611 | 4.649 | 7.139 | 7.161 | 13.393 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.657 | 4.033 | 7.497 | 9.641 | 10.340 | 18.089 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 7.570 | 7.831 | 13.115 | 19.846 | 17.062 | 35.924 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.699 | 4.606 | 5.856 | 9.332 | 7.015 | 12.247 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.402 | 1.608 | 2.526 | 4.050 | 3.947 | 7.561 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.791 | 0.809 | 1.065 | 2.193 | 1.435 | 2.566 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 51.610 | 50.013 | 60.777 | 81.651 | 71.056 | 104.900 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.852 | 13.615 | 17.404 | 24.774 | 28.984 | 58.631 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.388 | 7.167 | 13.030 | 18.520 | 20.822 | 48.437 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 5.822 | 6.428 | 7.370 | 13.189 | 13.237 | 30.841 |
| vue-docs-suspense | 8291 | gfm | no | yes | 9.250 | 9.997 | 19.394 | 22.905 | 22.202 | 42.953 |
| vue-docs-slots | 24211 | gfm | no | yes | 34.848 | 36.161 | 65.299 | 83.411 | 69.004 | 144.748 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 27.139 | 29.104 | 42.706 | 68.573 | 59.947 | 138.501 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 3.622 | 3.986 | 4.621 | 8.626 | 8.454 | 18.080 |
| vite-docs-performance | 8184 | gfm | no | yes | 9.004 | 9.305 | 12.510 | 19.914 | 20.504 | 41.749 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 74.023 | 75.315 | 82.217 | 118.340 | 100.200 | 191.999 |
| vite-docs-features | 39739 | gfm | no | no | 70.619 | 74.481 | 104.917 | 144.873 | 133.035 | 251.392 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.366 | 5.760 | 5.874 | 11.045 | 11.014 | 25.915 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 54.689 | 59.762 | 84.107 | 123.181 | 99.421 | 192.530 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 25.422 | 25.298 | 66.055 | 101.141 | 51.924 | 147.265 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 68.464 | 73.169 | 122.351 | 170.796 | 139.312 | 301.026 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.573 | 1.621 | 2.165 | 3.895 | 3.036 | 5.092 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.453 | 2.442 | 3.157 | 5.966 | 5.237 | 9.766 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 51.950 | 52.108 | 77.949 | 115.082 | 118.066 | 287.740 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 10.226 | 9.131 | 15.773 | 44.255 | 48.722 | 191.307 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.974 | 2.985 | 5.431 | 6.423 | 5.156 | 7.997 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 14.546 | 14.266 | 40.096 | 28.397 | 26.389 | 43.015 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 93.997 | 92.919 | 161.662 | 182.868 | 185.594 | 378.543 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 13.563 | 12.070 | 19.478 | 47.075 | 52.075 | 205.401 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.264 | 2.277 | 3.596 | 5.365 | 4.393 | 7.066 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 7.270 | 7.147 | 10.682 | 15.707 | 14.388 | 24.648 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 189.389 | 186.675 | 274.412 | 349.372 | 350.648 | 732.939 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 31.311 | 28.783 | 42.617 | 101.128 | 109.285 | 422.945 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 4.273 | 4.289 | 9.173 | 9.602 | 8.588 | 15.230 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 5.619 | 5.546 | 10.683 | 12.884 | 12.107 | 22.868 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 110.419 | 111.042 | 167.741 | 216.020 | 220.641 | 443.252 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 17.084 | 15.719 | 23.484 | 58.490 | 64.979 | 239.746 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.080 | 0.068 | 0.086 | 0.971 | 0.167 | 0.294 |
| comment-question | 160 | gfm | yes | yes | 0.093 | 0.079 | 0.106 | 1.068 | 0.311 | 0.773 |
| comment-review | 282 | gfm | yes | yes | 0.155 | 0.130 | 0.194 | 1.236 | 0.530 | 1.324 |
| comment-links | 278 | gfm | yes | yes | 0.422 | 0.376 | 0.508 | 1.578 | 0.760 | 1.603 |
| comment-checklist | 287 | gfm | no | no | 0.544 | 0.519 | 0.681 | 1.672 | 0.910 | 1.817 |
| comment-quote | 290 | gfm | yes | yes | 0.224 | 0.192 | 0.240 | 1.300 | 0.568 | 1.436 |
| comment-unicode | 327 | gfm | yes | yes | 0.382 | 0.340 | 0.458 | 1.516 | 0.834 | 1.685 |
| comment-inline-code | 285 | gfm | yes | yes | 0.225 | 0.200 | 0.451 | 1.365 | 0.693 | 1.392 |
| comment-reproduction | 298 | gfm | yes | yes | 0.257 | 0.240 | 0.362 | 1.494 | 0.590 | 1.389 |
| comment-table | 310 | gfm | yes | yes | 0.996 | 0.923 | 0.851 | 2.334 | 1.241 | 2.194 |
| comment-review-long | 957 | gfm | yes | yes | 0.708 | 0.667 | 0.817 | 2.329 | 1.847 | 4.457 |
| comment-incident | 1124 | gfm | no | yes | 1.216 | 1.232 | 1.317 | 2.927 | 2.384 | 5.392 |
| guard-angle-link | 41 | commonmark | no | no | 0.210 | 0.190 | 0.301 | 1.159 | 0.326 | 0.464 |
| legacy-contributing | 9323 | gfm | no | yes | 10.605 | 10.992 | 15.727 | 23.071 | 24.138 | 46.467 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 10.450 | 10.932 | 16.278 | 25.208 | 24.110 | 46.180 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.891 | 2.343 | 3.912 | 5.628 | 5.045 | 8.811 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.667 | 2.953 | 4.120 | 6.587 | 5.692 | 10.635 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.953 | 7.567 | 13.671 | 19.051 | 17.305 | 32.731 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.578 | 2.991 | 4.647 | 7.141 | 6.055 | 10.677 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.819 | 5.133 | 9.364 | 10.833 | 12.627 | 20.393 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.398 | 2.463 | 4.175 | 6.807 | 6.827 | 13.264 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.612 | 3.969 | 7.108 | 9.485 | 10.134 | 18.218 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 7.472 | 7.726 | 12.416 | 19.313 | 16.600 | 35.821 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.560 | 4.446 | 5.329 | 9.094 | 6.771 | 12.285 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.272 | 1.482 | 2.199 | 3.882 | 3.797 | 7.568 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.676 | 0.699 | 0.861 | 2.098 | 1.322 | 2.552 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 51.341 | 49.320 | 59.562 | 80.310 | 70.257 | 105.471 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.645 | 13.408 | 15.928 | 24.135 | 28.393 | 58.205 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.192 | 6.971 | 12.140 | 18.199 | 20.337 | 48.277 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 5.712 | 6.315 | 6.947 | 12.884 | 12.956 | 30.540 |
| vue-docs-suspense | 8291 | gfm | no | yes | 9.159 | 9.834 | 18.287 | 22.348 | 21.529 | 42.820 |
| vue-docs-slots | 24211 | gfm | no | yes | 34.762 | 36.020 | 63.263 | 81.690 | 66.758 | 143.648 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 26.714 | 28.718 | 40.667 | 66.366 | 58.965 | 137.163 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 3.531 | 3.871 | 4.229 | 8.422 | 8.228 | 18.220 |
| vite-docs-performance | 8184 | gfm | no | yes | 8.891 | 9.115 | 11.784 | 19.437 | 19.886 | 41.607 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 73.193 | 75.020 | 79.028 | 116.842 | 99.023 | 192.233 |
| vite-docs-features | 39739 | gfm | no | no | 69.209 | 73.542 | 101.348 | 141.976 | 129.853 | 250.820 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 5.253 | 5.622 | 5.413 | 10.796 | 10.770 | 26.060 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 54.554 | 59.444 | 80.488 | 121.815 | 99.530 | 195.565 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 25.587 | 25.379 | 64.676 | 100.618 | 51.047 | 149.290 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 69.707 | 73.601 | 120.623 | 169.114 | 138.454 | 303.619 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.453 | 1.491 | 1.806 | 3.786 | 2.897 | 5.109 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.375 | 2.365 | 2.807 | 5.903 | 5.154 | 9.868 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 51.548 | 51.581 | 72.700 | 112.833 | 116.645 | 286.638 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 10.117 | 9.027 | 13.544 | 41.912 | 46.696 | 191.900 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.859 | 2.878 | 5.045 | 6.283 | 5.049 | 8.003 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 14.335 | 14.073 | 38.698 | 28.220 | 26.135 | 43.144 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 92.814 | 91.515 | 155.519 | 178.286 | 181.747 | 378.063 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 13.370 | 12.041 | 17.214 | 45.871 | 50.392 | 206.240 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.149 | 2.163 | 3.262 | 5.240 | 4.262 | 7.117 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 7.227 | 7.117 | 10.043 | 15.510 | 14.309 | 25.041 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 188.485 | 187.233 | 265.928 | 349.251 | 344.452 | 730.807 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 31.084 | 28.662 | 38.464 | 97.262 | 106.257 | 424.426 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 4.151 | 4.164 | 8.635 | 9.399 | 8.391 | 15.199 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 5.564 | 5.516 | 10.272 | 12.756 | 11.911 | 23.203 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 110.949 | 110.152 | 161.599 | 211.842 | 214.067 | 442.498 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 16.882 | 15.622 | 20.906 | 55.423 | 61.965 | 239.722 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 685.17 | 664.18 | 1042.59 | 1316.21 | 1303.53 | 3192.58 |
| rotating-commonmark | reuse | 17 | 658.07 | 645.25 | 990.67 | 1275.03 | 1272.14 | 3183.72 |
| rotating-gfm | fresh | 40 | 831.46 | 878.06 | 1283.60 | 1474.82 | 1276.71 | 2359.31 |
| rotating-gfm | reuse | 40 | 812.30 | 851.93 | 1220.28 | 1438.78 | 1229.24 | 2369.65 |
