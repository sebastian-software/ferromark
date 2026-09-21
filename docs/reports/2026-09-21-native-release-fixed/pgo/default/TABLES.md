# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.84× | 0.63× | 0.20× | 0.48× | 0.22× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.51× | 0.29× | 0.42× | 0.20× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.73× | 0.49× | 0.29× | 0.40× | 0.19× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.64× | 0.22× | 0.48× | 0.22× |
| comments (all workloads) | 6 | 1.00× | 0.84× | 0.64× | 0.22× | 0.48× | 0.22× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.33× | 0.24× | 0.32× | 0.18× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.59× | 0.36× | 0.27× | 0.33× | 0.16× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.72× | 0.45× | 0.64× | 0.34× |
| readme (all workloads) | 1 | 1.00× | 0.91× | 0.72× | 0.45× | 0.64× | 0.34× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.50× | 0.33× | 0.52× | 0.22× |
| reference (all workloads) | 2 | 1.00× | 0.85× | 0.50× | 0.33× | 0.52× | 0.22× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.34× | 0.39× | 0.18× |
| technical-docs (all workloads) | 11 | 1.00× | 0.75× | 0.51× | 0.34× | 0.39× | 0.18× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.33× | 0.24× | 0.32× | 0.18× |
| commonmark (all workloads) | 8 | 1.00× | 0.59× | 0.36× | 0.27× | 0.33× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.56× | 0.30× | 0.44× | 0.20× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.79× | 0.56× | 0.30× | 0.44× | 0.20× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.60× | 0.20× | 0.46× | 0.22× |
| <512 B (all workloads) | 6 | 1.00× | 0.81× | 0.60× | 0.20× | 0.46× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.47× | 0.30× | 0.40× | 0.21× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.68× | 0.47× | 0.30× | 0.40× | 0.21× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.33× | 0.37× | 0.17× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.72× | 0.47× | 0.33× | 0.37× | 0.17× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.56× | 0.40× | 0.45× | 0.21× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.79× | 0.56× | 0.40× | 0.45× | 0.21× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.33× | 0.22× | 0.43× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.66× | 0.38× | 0.29× | 0.35× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.96× | 0.71× | 0.17× | 0.43× | 0.17× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.54× | 0.28× | 0.41× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.75× | 0.52× | 0.28× | 0.40× | 0.18× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.72× | 0.19× | 0.44× | 0.18× |
| comments (all workloads) | 6 | 1.00× | 0.95× | 0.72× | 0.19× | 0.44× | 0.18× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.35× | 0.23× | 0.32× | 0.17× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.59× | 0.38× | 0.27× | 0.33× | 0.16× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.77× | 0.45× | 0.65× | 0.34× |
| readme (all workloads) | 1 | 1.00× | 0.92× | 0.77× | 0.45× | 0.65× | 0.34× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.51× | 0.34× | 0.53× | 0.22× |
| reference (all workloads) | 2 | 1.00× | 0.85× | 0.51× | 0.34× | 0.53× | 0.22× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.53× | 0.34× | 0.39× | 0.18× |
| technical-docs (all workloads) | 11 | 1.00× | 0.76× | 0.53× | 0.34× | 0.39× | 0.18× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.35× | 0.23× | 0.32× | 0.17× |
| commonmark (all workloads) | 8 | 1.00× | 0.59× | 0.38× | 0.27× | 0.33× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.59× | 0.29× | 0.43× | 0.19× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.83× | 0.59× | 0.29× | 0.43× | 0.19× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.67× | 0.17× | 0.42× | 0.17× |
| <512 B (all workloads) | 6 | 1.00× | 0.90× | 0.67× | 0.17× | 0.42× | 0.17× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.50× | 0.29× | 0.40× | 0.20× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.69× | 0.50× | 0.29× | 0.40× | 0.20× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.33× | 0.37× | 0.16× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.73× | 0.50× | 0.33× | 0.37× | 0.16× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.58× | 0.40× | 0.46× | 0.21× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.79× | 0.58× | 0.40× | 0.46× | 0.21× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.34× | 0.22× | 0.44× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.66× | 0.40× | 0.29× | 0.36× | 0.14× |

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
| comment-question | 160 | gfm | yes | yes | 0.122 | 0.162 | 0.180 | 1.080 | 0.288 | 0.763 |
| comment-links | 278 | gfm | yes | yes | 0.374 | 0.465 | 0.691 | 1.621 | 0.742 | 1.575 |
| comment-quote | 290 | gfm | yes | yes | 0.228 | 0.274 | 0.349 | 1.368 | 0.568 | 1.423 |
| comment-inline-code | 285 | gfm | yes | yes | 0.240 | 0.285 | 0.593 | 1.415 | 0.678 | 1.365 |
| comment-table | 310 | gfm | yes | yes | 1.007 | 0.997 | 0.985 | 2.365 | 1.213 | 2.137 |
| comment-incident | 1124 | gfm | no | yes | 1.094 | 1.303 | 1.507 | 3.018 | 2.176 | 5.317 |
| legacy-contributing | 9323 | gfm | no | yes | 9.116 | 10.895 | 16.181 | 23.257 | 22.247 | 46.269 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.738 | 2.396 | 4.108 | 5.680 | 4.685 | 8.602 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.019 | 7.546 | 13.973 | 19.109 | 16.390 | 31.877 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.340 | 5.137 | 9.866 | 10.922 | 11.653 | 19.928 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.210 | 3.989 | 7.348 | 9.475 | 9.259 | 17.932 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.093 | 4.499 | 5.677 | 9.126 | 6.359 | 12.040 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.508 | 0.786 | 1.041 | 2.156 | 1.286 | 2.524 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.634 | 13.206 | 16.870 | 24.047 | 25.533 | 57.324 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.487 | 6.309 | 7.222 | 12.957 | 11.324 | 30.541 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.190 | 35.710 | 64.909 | 82.041 | 62.505 | 141.444 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.654 | 3.898 | 4.561 | 8.461 | 7.298 | 18.153 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 59.527 | 70.039 | 78.118 | 115.641 | 92.211 | 186.533 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.244 | 5.640 | 5.793 | 10.787 | 9.381 | 25.815 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.629 | 25.230 | 65.142 | 99.347 | 50.667 | 144.914 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.924 | 1.590 | 2.104 | 3.829 | 2.707 | 5.035 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.613 | 50.216 | 75.253 | 112.790 | 102.389 | 280.097 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.596 | 2.948 | 5.361 | 6.341 | 4.744 | 7.889 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.687 | 90.000 | 158.701 | 180.760 | 166.523 | 371.677 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.285 | 2.281 | 3.654 | 5.398 | 4.027 | 7.090 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 113.845 | 184.659 | 269.019 | 345.275 | 314.838 | 715.405 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.189 | 4.211 | 8.954 | 9.537 | 7.612 | 14.940 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 65.004 | 107.249 | 164.639 | 212.888 | 196.971 | 432.103 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.075 | 0.077 | 0.103 | 1.041 | 0.257 | 0.761 |
| comment-links | 278 | gfm | yes | yes | 0.318 | 0.376 | 0.500 | 1.547 | 0.662 | 1.579 |
| comment-quote | 290 | gfm | yes | yes | 0.178 | 0.186 | 0.235 | 1.273 | 0.476 | 1.421 |
| comment-inline-code | 285 | gfm | yes | yes | 0.193 | 0.196 | 0.436 | 1.339 | 0.599 | 1.367 |
| comment-table | 310 | gfm | yes | yes | 0.955 | 0.901 | 0.837 | 2.284 | 1.124 | 2.145 |
| comment-incident | 1124 | gfm | no | yes | 1.032 | 1.194 | 1.300 | 2.881 | 2.051 | 5.315 |
| legacy-contributing | 9323 | gfm | no | yes | 8.974 | 10.638 | 15.345 | 22.783 | 21.515 | 45.678 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.675 | 2.278 | 3.817 | 5.531 | 4.526 | 8.624 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.922 | 7.356 | 13.367 | 18.767 | 15.828 | 31.822 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.272 | 5.024 | 9.185 | 10.614 | 11.264 | 20.062 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.134 | 3.856 | 6.903 | 9.271 | 8.979 | 17.979 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.017 | 4.343 | 5.233 | 8.918 | 6.152 | 11.990 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.442 | 0.679 | 0.832 | 2.053 | 1.170 | 2.518 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.552 | 13.001 | 15.578 | 23.528 | 24.778 | 57.240 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.397 | 6.150 | 6.783 | 12.556 | 10.960 | 30.644 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.011 | 35.384 | 64.712 | 80.260 | 61.046 | 141.284 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.595 | 3.764 | 4.130 | 8.205 | 7.012 | 18.136 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 58.270 | 69.327 | 76.941 | 114.103 | 91.271 | 186.427 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.172 | 5.477 | 5.293 | 10.520 | 9.069 | 26.074 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.505 | 24.806 | 62.802 | 96.815 | 48.747 | 144.005 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.856 | 1.458 | 1.758 | 3.733 | 2.600 | 4.989 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.631 | 49.735 | 71.565 | 110.486 | 100.769 | 280.811 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.530 | 2.810 | 4.974 | 6.176 | 4.579 | 7.867 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.717 | 89.457 | 152.920 | 178.183 | 164.846 | 371.335 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.224 | 2.159 | 3.250 | 5.232 | 3.896 | 7.087 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 113.688 | 184.732 | 260.415 | 342.891 | 309.828 | 717.487 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.141 | 4.124 | 8.506 | 9.349 | 7.469 | 14.958 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 63.371 | 106.502 | 159.331 | 209.710 | 191.967 | 432.513 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 333.45 | 514.69 | 782.14 | 933.75 | 829.42 | 1884.02 |
| rotating-commonmark | reuse | 8 | 325.55 | 505.70 | 756.65 | 919.58 | 819.78 | 1889.54 |
| rotating-gfm | fresh | 20 | 220.48 | 288.19 | 434.45 | 532.05 | 406.55 | 880.30 |
| rotating-gfm | reuse | 20 | 213.02 | 277.34 | 412.14 | 522.36 | 398.32 | 884.77 |
