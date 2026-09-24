# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.80× | 0.57× | 0.21× | 0.37× | 0.13× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.47× | 0.28× | 0.37× | 0.16× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.70× | 0.46× | 0.28× | 0.37× | 0.16× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.58× | 0.22× | 0.43× | 0.19× |
| comments (all workloads) | 12 | 1.00× | 0.76× | 0.58× | 0.22× | 0.44× | 0.20× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.30× | 0.24× | 0.30× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.55× | 0.33× | 0.26× | 0.30× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.56× | 0.23× | 0.26× | 0.05× |
| plain-prose (all workloads) | 4 | 1.00× | 0.89× | 0.56× | 0.23× | 0.26× | 0.05× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.53× | 0.35× | 0.44× | 0.22× |
| readme (all workloads) | 2 | 1.00× | 0.76× | 0.53× | 0.35× | 0.44× | 0.22× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.50× | 0.35× | 0.49× | 0.23× |
| reference (all workloads) | 4 | 1.00× | 0.75× | 0.50× | 0.35× | 0.49× | 0.23× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.67× | 0.39× | 0.19× | 0.53× | 0.40× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.46× | 0.32× | 0.36× | 0.16× |
| technical-docs (all workloads) | 22 | 1.00× | 0.71× | 0.46× | 0.32× | 0.36× | 0.16× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.37× | 0.24× | 0.29× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.63× | 0.37× | 0.25× | 0.30× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.50× | 0.29× | 0.40× | 0.18× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.73× | 0.50× | 0.29× | 0.40× | 0.18× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.56× | 0.20× | 0.43× | 0.20× |
| <512 B (all workloads) | 12 | 1.00× | 0.74× | 0.54× | 0.21× | 0.44× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.45× | 0.28× | 0.36× | 0.17× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.65× | 0.45× | 0.28× | 0.36× | 0.17× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.41× | 0.30× | 0.34× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.67× | 0.41× | 0.30× | 0.34× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.49× | 0.36× | 0.41× | 0.19× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.75× | 0.49× | 0.36× | 0.41× | 0.19× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.48× | 0.24× | 0.32× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.71× | 0.44× | 0.27× | 0.32× | 0.11× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.88× | 0.63× | 0.18× | 0.34× | 0.11× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.50× | 0.26× | 0.36× | 0.15× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.72× | 0.49× | 0.27× | 0.36× | 0.15× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.64× | 0.18× | 0.38× | 0.15× |
| comments (all workloads) | 12 | 1.00× | 0.85× | 0.64× | 0.18× | 0.38× | 0.16× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.32× | 0.24× | 0.30× | 0.15× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.55× | 0.34× | 0.26× | 0.30× | 0.14× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.63× | 0.24× | 0.28× | 0.05× |
| plain-prose (all workloads) | 4 | 1.00× | 0.90× | 0.63× | 0.24× | 0.28× | 0.05× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.56× | 0.36× | 0.44× | 0.22× |
| readme (all workloads) | 2 | 1.00× | 0.76× | 0.56× | 0.36× | 0.44× | 0.22× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.51× | 0.35× | 0.50× | 0.23× |
| reference (all workloads) | 4 | 1.00× | 0.74× | 0.51× | 0.35× | 0.50× | 0.23× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.73× | 0.46× | 0.15× | 0.44× | 0.29× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.48× | 0.32× | 0.36× | 0.16× |
| technical-docs (all workloads) | 22 | 1.00× | 0.72× | 0.48× | 0.32× | 0.36× | 0.16× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.40× | 0.24× | 0.29× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.63× | 0.40× | 0.24× | 0.30× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.53× | 0.27× | 0.38× | 0.17× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.76× | 0.53× | 0.28× | 0.39× | 0.17× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.62× | 0.16× | 0.37× | 0.15× |
| <512 B (all workloads) | 12 | 1.00× | 0.82× | 0.60× | 0.17× | 0.38× | 0.17× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.48× | 0.28× | 0.35× | 0.16× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.65× | 0.48× | 0.28× | 0.35× | 0.16× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.30× | 0.34× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.67× | 0.43× | 0.30× | 0.34× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.51× | 0.36× | 0.41× | 0.18× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.75× | 0.51× | 0.36× | 0.41× | 0.18× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.52× | 0.25× | 0.33× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.71× | 0.46× | 0.27× | 0.33× | 0.11× |

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
| comment-ack | 37 | gfm | yes | yes | 0.106 | 0.160 | 0.161 | 0.792 | 0.190 | 0.297 |
| comment-question | 160 | gfm | yes | yes | 0.121 | 0.166 | 0.181 | 0.886 | 0.292 | 0.766 |
| comment-review | 282 | gfm | yes | yes | 0.167 | 0.216 | 0.296 | 1.054 | 0.492 | 1.318 |
| comment-links | 278 | gfm | yes | yes | 0.323 | 0.477 | 0.700 | 1.437 | 0.752 | 1.620 |
| comment-checklist | 287 | gfm | no | no | 0.437 | 0.607 | 0.827 | 1.529 | 0.893 | 1.785 |
| comment-quote | 290 | gfm | yes | yes | 0.227 | 0.280 | 0.349 | 1.136 | 0.562 | 1.423 |
| comment-unicode | 327 | gfm | yes | yes | 0.397 | 0.431 | 0.701 | 1.344 | 0.802 | 1.652 |
| comment-inline-code | 285 | gfm | yes | yes | 0.242 | 0.292 | 0.616 | 1.231 | 0.697 | 1.403 |
| comment-reproduction | 298 | gfm | yes | yes | 0.247 | 0.331 | 0.478 | 1.334 | 0.596 | 1.378 |
| comment-table | 310 | gfm | yes | yes | 0.815 | 1.013 | 0.997 | 2.202 | 1.233 | 2.212 |
| comment-review-long | 957 | gfm | yes | yes | 0.549 | 0.758 | 1.016 | 2.159 | 1.637 | 4.401 |
| comment-incident | 1124 | gfm | no | yes | 1.033 | 1.298 | 1.518 | 2.773 | 2.155 | 5.354 |
| guard-angle-link | 41 | commonmark | no | no | 0.187 | 0.278 | 0.478 | 0.968 | 0.352 | 0.467 |
| legacy-contributing | 9323 | gfm | no | yes | 8.723 | 10.965 | 16.429 | 23.045 | 22.363 | 47.208 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.219 | 10.836 | 17.310 | 25.104 | 22.577 | 46.682 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.660 | 2.389 | 4.100 | 5.421 | 4.711 | 8.573 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.284 | 3.027 | 4.447 | 6.513 | 5.378 | 10.681 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.859 | 7.534 | 14.092 | 18.867 | 16.314 | 32.494 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.261 | 3.067 | 5.074 | 6.947 | 5.751 | 10.549 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.199 | 5.118 | 9.773 | 10.595 | 11.658 | 20.448 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.987 | 2.589 | 4.563 | 6.835 | 6.417 | 13.520 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.092 | 3.970 | 7.451 | 9.239 | 9.366 | 18.561 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 5.950 | 7.694 | 12.995 | 19.279 | 15.211 | 35.659 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.381 | 4.440 | 5.729 | 8.831 | 6.408 | 12.097 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.090 | 1.573 | 2.494 | 3.760 | 3.419 | 7.487 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.446 | 0.800 | 1.056 | 1.960 | 1.295 | 2.548 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 37.614 | 49.383 | 60.376 | 78.943 | 63.650 | 104.243 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.164 | 13.119 | 16.902 | 23.703 | 25.413 | 58.108 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.917 | 7.107 | 12.796 | 17.905 | 18.092 | 49.181 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.849 | 6.314 | 7.272 | 12.665 | 11.495 | 31.577 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.482 | 9.806 | 19.168 | 22.159 | 19.980 | 43.360 |
| vue-docs-slots | 24211 | gfm | no | yes | 18.676 | 35.467 | 67.996 | 82.360 | 61.994 | 142.990 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 16.909 | 28.778 | 45.942 | 68.154 | 54.205 | 138.214 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.500 | 3.974 | 4.617 | 8.302 | 7.416 | 18.859 |
| vite-docs-performance | 8184 | gfm | no | yes | 6.688 | 9.124 | 12.225 | 19.251 | 17.971 | 42.181 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 52.295 | 70.995 | 80.656 | 115.213 | 90.513 | 186.709 |
| vite-docs-features | 39739 | gfm | no | no | 47.981 | 73.745 | 102.564 | 141.097 | 120.244 | 246.881 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.772 | 5.627 | 5.776 | 10.503 | 9.298 | 26.360 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 41.282 | 60.612 | 85.547 | 124.490 | 94.666 | 196.479 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 20.921 | 25.687 | 66.182 | 101.465 | 51.909 | 147.503 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 47.956 | 73.207 | 122.063 | 168.056 | 127.059 | 297.231 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.890 | 1.599 | 2.139 | 3.665 | 2.764 | 5.230 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.402 | 2.434 | 3.143 | 5.746 | 4.632 | 9.888 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.955 | 52.754 | 80.272 | 117.089 | 106.635 | 294.065 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.996 | 8.962 | 15.622 | 43.018 | 36.909 | 191.983 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.557 | 2.926 | 5.425 | 6.141 | 4.809 | 8.124 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.478 | 13.983 | 39.749 | 27.952 | 24.425 | 43.830 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 53.961 | 92.576 | 161.290 | 183.240 | 168.241 | 380.922 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 10.427 | 11.793 | 19.305 | 45.794 | 39.542 | 203.720 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.251 | 2.297 | 3.696 | 5.203 | 4.058 | 7.312 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.672 | 7.106 | 10.873 | 15.474 | 13.223 | 25.423 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 107.835 | 185.433 | 271.189 | 349.683 | 315.655 | 731.198 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 24.142 | 28.402 | 42.777 | 99.357 | 84.563 | 418.017 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.165 | 4.193 | 9.008 | 9.290 | 7.666 | 15.278 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.894 | 5.473 | 10.700 | 12.672 | 10.676 | 23.293 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 60.486 | 110.769 | 169.151 | 216.578 | 198.237 | 444.948 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 13.233 | 15.580 | 23.310 | 56.452 | 49.274 | 235.887 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.058 | 0.068 | 0.085 | 0.747 | 0.155 | 0.295 |
| comment-question | 160 | gfm | yes | yes | 0.074 | 0.078 | 0.104 | 0.837 | 0.262 | 0.767 |
| comment-review | 282 | gfm | yes | yes | 0.124 | 0.130 | 0.193 | 1.010 | 0.444 | 1.328 |
| comment-links | 278 | gfm | yes | yes | 0.264 | 0.383 | 0.506 | 1.342 | 0.664 | 1.610 |
| comment-checklist | 287 | gfm | no | no | 0.380 | 0.511 | 0.667 | 1.428 | 0.813 | 1.781 |
| comment-quote | 290 | gfm | yes | yes | 0.174 | 0.188 | 0.236 | 1.063 | 0.479 | 1.426 |
| comment-unicode | 327 | gfm | yes | yes | 0.337 | 0.332 | 0.441 | 1.263 | 0.729 | 1.669 |
| comment-inline-code | 285 | gfm | yes | yes | 0.188 | 0.195 | 0.461 | 1.125 | 0.604 | 1.374 |
| comment-reproduction | 298 | gfm | yes | yes | 0.197 | 0.239 | 0.358 | 1.242 | 0.527 | 1.387 |
| comment-table | 310 | gfm | yes | yes | 0.733 | 0.897 | 0.849 | 2.073 | 1.139 | 2.172 |
| comment-review-long | 957 | gfm | yes | yes | 0.491 | 0.657 | 0.809 | 2.072 | 1.552 | 4.456 |
| comment-incident | 1124 | gfm | no | yes | 0.985 | 1.216 | 1.335 | 2.704 | 2.089 | 5.363 |
| guard-angle-link | 41 | commonmark | no | no | 0.135 | 0.186 | 0.293 | 0.930 | 0.310 | 0.471 |
| legacy-contributing | 9323 | gfm | no | yes | 8.650 | 10.772 | 15.491 | 22.568 | 21.860 | 47.225 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.119 | 10.699 | 16.298 | 24.464 | 22.001 | 46.691 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.608 | 2.301 | 3.832 | 5.292 | 4.560 | 8.691 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.211 | 2.918 | 4.124 | 6.266 | 5.262 | 10.633 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.800 | 7.352 | 13.575 | 18.569 | 15.803 | 32.495 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.195 | 2.949 | 4.636 | 6.812 | 5.592 | 10.742 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.090 | 4.985 | 9.272 | 10.396 | 11.306 | 20.679 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.914 | 2.440 | 4.162 | 6.592 | 6.126 | 13.390 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.028 | 3.882 | 7.131 | 9.082 | 9.066 | 18.568 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 5.827 | 7.508 | 12.193 | 18.710 | 14.746 | 35.792 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.313 | 4.349 | 5.298 | 8.681 | 6.187 | 12.115 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.025 | 1.448 | 2.181 | 3.619 | 3.303 | 7.530 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.377 | 0.682 | 0.846 | 1.846 | 1.180 | 2.556 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 37.645 | 48.518 | 58.032 | 77.878 | 61.830 | 103.289 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.054 | 13.071 | 15.743 | 23.429 | 25.102 | 58.363 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.742 | 6.818 | 11.874 | 17.443 | 17.762 | 49.210 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.824 | 6.218 | 6.818 | 12.424 | 11.098 | 31.517 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.436 | 9.631 | 18.370 | 21.812 | 19.241 | 43.821 |
| vue-docs-slots | 24211 | gfm | no | yes | 18.840 | 35.694 | 66.252 | 81.251 | 61.167 | 143.473 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 16.611 | 28.270 | 43.630 | 65.390 | 52.367 | 136.418 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.369 | 3.762 | 4.133 | 7.968 | 6.989 | 18.455 |
| vite-docs-performance | 8184 | gfm | no | yes | 6.629 | 9.033 | 11.712 | 18.904 | 17.514 | 42.387 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 52.925 | 72.176 | 79.710 | 114.401 | 91.086 | 188.678 |
| vite-docs-features | 39739 | gfm | no | no | 47.664 | 72.812 | 103.743 | 141.470 | 121.139 | 248.732 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.766 | 5.545 | 5.368 | 10.456 | 9.063 | 26.657 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 39.196 | 60.087 | 80.127 | 118.054 | 90.325 | 189.564 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 20.486 | 24.870 | 62.928 | 98.095 | 49.398 | 147.480 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 46.520 | 68.318 | 119.537 | 164.776 | 125.879 | 299.618 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.823 | 1.459 | 1.781 | 3.519 | 2.606 | 5.169 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.355 | 2.328 | 2.780 | 5.642 | 4.526 | 9.932 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 30.537 | 51.066 | 71.993 | 111.213 | 102.019 | 284.596 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.937 | 8.912 | 13.472 | 41.408 | 34.469 | 191.181 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.483 | 2.807 | 5.001 | 5.960 | 4.612 | 8.056 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.397 | 13.899 | 38.361 | 27.594 | 24.205 | 43.972 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 52.371 | 90.519 | 153.600 | 177.952 | 164.059 | 373.029 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 10.667 | 12.024 | 17.390 | 45.719 | 38.952 | 209.819 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.197 | 2.185 | 3.327 | 5.097 | 4.007 | 7.426 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.611 | 6.993 | 9.974 | 15.094 | 12.991 | 25.386 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 107.825 | 184.930 | 262.363 | 343.476 | 311.446 | 723.068 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 23.864 | 27.766 | 38.783 | 95.152 | 81.000 | 418.940 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.108 | 4.119 | 8.601 | 9.211 | 7.577 | 15.442 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.857 | 5.413 | 10.179 | 12.478 | 10.605 | 23.588 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 59.037 | 108.874 | 159.881 | 209.845 | 191.939 | 437.379 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 13.126 | 15.288 | 20.672 | 53.574 | 46.714 | 235.567 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 418.21 | 651.80 | 1033.53 | 1281.23 | 1129.94 | 3152.01 |
| rotating-commonmark | reuse | 17 | 405.93 | 640.36 | 978.25 | 1269.09 | 1110.36 | 3215.81 |
| rotating-gfm | fresh | 40 | 602.75 | 859.27 | 1268.95 | 1440.28 | 1179.35 | 2336.71 |
| rotating-gfm | reuse | 40 | 598.13 | 847.16 | 1218.66 | 1412.46 | 1152.06 | 2364.66 |
