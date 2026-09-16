# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 1.00× | 0.75× | 0.28× | 0.57× | 0.26× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.53× | 0.32× | 0.44× | 0.20× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.75× | 0.50× | 0.32× | 0.42× | 0.19× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.76× | 0.30× | 0.57× | 0.25× |
| comments (all workloads) | 6 | 1.00× | 0.99× | 0.76× | 0.30× | 0.57× | 0.25× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.24× | 0.31× | 0.17× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.56× | 0.34× | 0.27× | 0.32× | 0.15× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.69× | 0.45× | 0.62× | 0.33× |
| readme (all workloads) | 1 | 1.00× | 0.89× | 0.69× | 0.45× | 0.62× | 0.33× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.53× | 0.36× | 0.56× | 0.23× |
| reference (all workloads) | 2 | 1.00× | 0.90× | 0.53× | 0.36× | 0.56× | 0.23× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.35× | 0.40× | 0.18× |
| technical-docs (all workloads) | 11 | 1.00× | 0.77× | 0.51× | 0.35× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.24× | 0.31× | 0.17× |
| commonmark (all workloads) | 8 | 1.00× | 0.56× | 0.34× | 0.27× | 0.32× | 0.15× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.59× | 0.34× | 0.47× | 0.21× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.85× | 0.59× | 0.34× | 0.47× | 0.21× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.70× | 0.28× | 0.54× | 0.25× |
| <512 B (all workloads) | 6 | 1.00× | 0.94× | 0.70× | 0.28× | 0.54× | 0.25× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.47× | 0.32× | 0.41× | 0.21× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.69× | 0.47× | 0.32× | 0.41× | 0.21× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.33× | 0.37× | 0.17× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.72× | 0.47× | 0.33× | 0.37× | 0.17× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.56× | 0.41× | 0.46× | 0.21× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.79× | 0.56× | 0.41× | 0.46× | 0.21× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.38× | 0.25× | 0.49× | 0.17× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.66× | 0.38× | 0.29× | 0.35× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.98× | 0.71× | 0.20× | 0.43× | 0.18× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.53× | 0.29× | 0.40× | 0.18× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.74× | 0.51× | 0.29× | 0.39× | 0.17× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.73× | 0.22× | 0.45× | 0.18× |
| comments (all workloads) | 6 | 1.00× | 0.97× | 0.73× | 0.22× | 0.45× | 0.18× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.23× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.55× | 0.35× | 0.26× | 0.31× | 0.14× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.72× | 0.44× | 0.63× | 0.32× |
| readme (all workloads) | 1 | 1.00× | 0.89× | 0.72× | 0.44× | 0.63× | 0.32× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.54× | 0.36× | 0.57× | 0.23× |
| reference (all workloads) | 2 | 1.00× | 0.90× | 0.54× | 0.36× | 0.57× | 0.23× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.53× | 0.35× | 0.39× | 0.17× |
| technical-docs (all workloads) | 11 | 1.00× | 0.76× | 0.53× | 0.35× | 0.39× | 0.17× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.31× | 0.23× | 0.29× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.55× | 0.35× | 0.26× | 0.31× | 0.14× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.59× | 0.31× | 0.43× | 0.19× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.84× | 0.59× | 0.31× | 0.43× | 0.19× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.67× | 0.20× | 0.42× | 0.17× |
| <512 B (all workloads) | 6 | 1.00× | 0.91× | 0.67× | 0.20× | 0.42× | 0.17× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.48× | 0.30× | 0.39× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.67× | 0.48× | 0.30× | 0.39× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.48× | 0.33× | 0.37× | 0.16× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.72× | 0.48× | 0.33× | 0.37× | 0.16× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.58× | 0.41× | 0.47× | 0.21× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.79× | 0.58× | 0.41× | 0.47× | 0.21× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.39× | 0.25× | 0.50× | 0.17× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.65× | 0.39× | 0.29× | 0.36× | 0.14× |

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
| comment-question | 160 | gfm | yes | yes | 0.172 | 0.163 | 0.180 | 0.861 | 0.290 | 0.760 |
| comment-links | 278 | gfm | yes | yes | 0.405 | 0.469 | 0.693 | 1.421 | 0.742 | 1.608 |
| comment-quote | 290 | gfm | yes | yes | 0.287 | 0.276 | 0.348 | 1.152 | 0.578 | 1.415 |
| comment-inline-code | 285 | gfm | yes | yes | 0.289 | 0.284 | 0.602 | 1.207 | 0.678 | 1.368 |
| comment-table | 310 | gfm | yes | yes | 1.040 | 0.995 | 0.981 | 2.173 | 1.209 | 2.159 |
| comment-incident | 1124 | gfm | no | yes | 1.220 | 1.303 | 1.502 | 2.807 | 2.181 | 5.301 |
| legacy-contributing | 9323 | gfm | no | yes | 9.300 | 10.788 | 16.352 | 22.930 | 21.906 | 46.489 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.854 | 2.409 | 4.201 | 5.525 | 4.696 | 8.675 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.063 | 7.522 | 14.285 | 18.967 | 16.194 | 32.374 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.538 | 5.130 | 10.027 | 10.705 | 11.568 | 19.962 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.312 | 4.002 | 7.558 | 9.417 | 9.337 | 18.302 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.988 | 4.483 | 5.789 | 8.961 | 6.381 | 12.166 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.539 | 0.788 | 1.053 | 1.959 | 1.301 | 2.534 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.881 | 13.218 | 16.851 | 23.917 | 25.358 | 57.704 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.456 | 6.323 | 7.266 | 12.680 | 11.316 | 31.036 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.392 | 35.464 | 65.085 | 81.974 | 61.788 | 143.584 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.580 | 3.893 | 4.572 | 8.250 | 7.246 | 18.248 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 58.004 | 71.731 | 79.707 | 115.751 | 92.333 | 188.220 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.278 | 5.687 | 5.811 | 10.645 | 9.357 | 26.000 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 24.974 | 24.985 | 64.977 | 98.731 | 50.887 | 144.643 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.898 | 1.584 | 2.122 | 3.640 | 2.709 | 5.144 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.596 | 50.817 | 75.895 | 112.329 | 103.052 | 284.037 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.544 | 2.930 | 5.411 | 6.107 | 4.749 | 8.112 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 52.271 | 90.287 | 159.010 | 179.719 | 164.764 | 376.537 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.251 | 2.268 | 3.604 | 5.121 | 4.046 | 7.309 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 111.135 | 184.447 | 269.773 | 345.199 | 311.667 | 723.306 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.057 | 4.212 | 9.048 | 9.279 | 7.589 | 15.252 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 60.932 | 106.735 | 166.133 | 212.752 | 195.221 | 439.439 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.080 | 0.077 | 0.104 | 0.827 | 0.257 | 0.760 |
| comment-links | 278 | gfm | yes | yes | 0.301 | 0.375 | 0.501 | 1.332 | 0.663 | 1.602 |
| comment-quote | 290 | gfm | yes | yes | 0.193 | 0.185 | 0.235 | 1.062 | 0.493 | 1.417 |
| comment-inline-code | 285 | gfm | yes | yes | 0.192 | 0.196 | 0.463 | 1.119 | 0.600 | 1.362 |
| comment-table | 310 | gfm | yes | yes | 0.938 | 0.895 | 0.835 | 2.061 | 1.121 | 2.142 |
| comment-incident | 1124 | gfm | no | yes | 1.117 | 1.213 | 1.309 | 2.724 | 2.088 | 5.380 |
| legacy-contributing | 9323 | gfm | no | yes | 9.188 | 10.675 | 15.599 | 22.554 | 21.463 | 46.614 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.726 | 2.281 | 3.868 | 5.314 | 4.520 | 8.645 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.938 | 7.352 | 13.571 | 18.685 | 15.701 | 32.469 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.427 | 5.008 | 9.527 | 10.486 | 11.195 | 20.171 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.156 | 3.848 | 7.024 | 9.085 | 8.992 | 18.163 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.853 | 4.340 | 5.335 | 8.772 | 6.159 | 12.144 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.428 | 0.676 | 0.842 | 1.862 | 1.181 | 2.529 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.777 | 13.166 | 15.665 | 23.498 | 24.846 | 58.497 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.336 | 6.205 | 6.841 | 12.448 | 11.066 | 31.179 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.250 | 35.359 | 63.404 | 80.113 | 60.369 | 144.209 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.462 | 3.754 | 4.162 | 8.036 | 7.010 | 18.265 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 57.890 | 71.518 | 77.404 | 113.488 | 90.221 | 187.454 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.151 | 5.485 | 5.301 | 10.341 | 9.059 | 26.039 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 24.767 | 24.806 | 63.349 | 97.920 | 49.344 | 144.591 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.775 | 1.466 | 1.781 | 3.528 | 2.594 | 5.141 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 30.804 | 50.554 | 70.526 | 110.118 | 100.948 | 283.007 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.427 | 2.820 | 5.028 | 5.951 | 4.589 | 8.115 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 52.595 | 90.375 | 154.512 | 177.867 | 163.840 | 376.158 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.132 | 2.156 | 3.264 | 4.964 | 3.909 | 7.303 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 112.875 | 186.427 | 265.011 | 343.704 | 307.662 | 728.506 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.952 | 4.104 | 8.569 | 9.121 | 7.430 | 15.284 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 60.002 | 107.281 | 160.774 | 208.037 | 190.480 | 438.000 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 328.74 | 516.03 | 786.61 | 932.24 | 824.32 | 1912.49 |
| rotating-commonmark | reuse | 8 | 321.06 | 504.00 | 759.41 | 919.05 | 814.49 | 1911.03 |
| rotating-gfm | fresh | 20 | 230.96 | 290.64 | 435.73 | 526.62 | 413.79 | 888.01 |
| rotating-gfm | reuse | 20 | 223.34 | 278.00 | 413.36 | 514.76 | 396.45 | 889.12 |
