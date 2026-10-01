# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.90× | 0.60× | 0.22× | 0.36× | 0.13× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.48× | 0.29× | 0.37× | 0.16× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.75× | 0.47× | 0.29× | 0.37× | 0.17× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.61× | 0.23× | 0.43× | 0.19× |
| comments (all workloads) | 12 | 1.00× | 0.86× | 0.61× | 0.23× | 0.44× | 0.20× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.30× | 0.26× | 0.31× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.54× | 0.33× | 0.27× | 0.31× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.59× | 0.23× | 0.23× | 0.05× |
| plain-prose (all workloads) | 4 | 1.00× | 1.00× | 0.59× | 0.23× | 0.23× | 0.05× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.52× | 0.37× | 0.48× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.74× | 0.52× | 0.37× | 0.48× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.51× | 0.36× | 0.49× | 0.24× |
| reference (all workloads) | 4 | 1.00× | 0.81× | 0.51× | 0.36× | 0.49× | 0.24× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.77× | 0.40× | 0.19× | 0.50× | 0.37× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.47× | 0.33× | 0.37× | 0.17× |
| technical-docs (all workloads) | 22 | 1.00× | 0.77× | 0.48× | 0.33× | 0.37× | 0.17× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.38× | 0.25× | 0.28× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.64× | 0.39× | 0.25× | 0.29× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.30× | 0.40× | 0.19× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.80× | 0.52× | 0.30× | 0.40× | 0.19× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.59× | 0.21× | 0.43× | 0.20× |
| <512 B (all workloads) | 12 | 1.00× | 0.83× | 0.57× | 0.22× | 0.44× | 0.21× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.45× | 0.30× | 0.37× | 0.18× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.69× | 0.45× | 0.30× | 0.37× | 0.18× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.41× | 0.31× | 0.35× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.69× | 0.41× | 0.31× | 0.35× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.51× | 0.37× | 0.41× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.79× | 0.51× | 0.37× | 0.41× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.25× | 0.29× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.76× | 0.46× | 0.27× | 0.30× | 0.11× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.98× | 0.63× | 0.18× | 0.32× | 0.11× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.51× | 0.28× | 0.36× | 0.15× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.77× | 0.50× | 0.27× | 0.36× | 0.15× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.64× | 0.18× | 0.37× | 0.14× |
| comments (all workloads) | 12 | 1.00× | 0.95× | 0.64× | 0.19× | 0.38× | 0.15× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.33× | 0.26× | 0.31× | 0.15× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.54× | 0.35× | 0.27× | 0.30× | 0.14× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.63× | 0.23× | 0.25× | 0.05× |
| plain-prose (all workloads) | 4 | 1.00× | 1.00× | 0.63× | 0.23× | 0.25× | 0.05× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.56× | 0.37× | 0.47× | 0.24× |
| readme (all workloads) | 2 | 1.00× | 0.82× | 0.56× | 0.37× | 0.47× | 0.24× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.38× | 0.52× | 0.25× |
| reference (all workloads) | 4 | 1.00× | 0.81× | 0.53× | 0.38× | 0.52× | 0.25× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.91× | 0.51× | 0.16× | 0.49× | 0.31× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.50× | 0.34× | 0.37× | 0.17× |
| technical-docs (all workloads) | 22 | 1.00× | 0.78× | 0.50× | 0.33× | 0.37× | 0.17× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.41× | 0.25× | 0.29× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.65× | 0.41× | 0.25× | 0.30× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.54× | 0.29× | 0.39× | 0.17× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.83× | 0.54× | 0.29× | 0.39× | 0.17× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.62× | 0.17× | 0.37× | 0.15× |
| <512 B (all workloads) | 12 | 1.00× | 0.94× | 0.61× | 0.18× | 0.39× | 0.16× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.49× | 0.28× | 0.36× | 0.17× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.69× | 0.49× | 0.28× | 0.36× | 0.17× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.32× | 0.35× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.71× | 0.43× | 0.32× | 0.35× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.38× | 0.42× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.78× | 0.54× | 0.38× | 0.42× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.54× | 0.26× | 0.31× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.77× | 0.49× | 0.27× | 0.31× | 0.11× |

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
| comment-ack | 37 | gfm | yes | yes | 0.103 | 0.125 | 0.147 | 0.621 | 0.171 | 0.263 |
| comment-question | 160 | gfm | yes | yes | 0.104 | 0.124 | 0.141 | 0.695 | 0.254 | 0.648 |
| comment-review | 282 | gfm | yes | yes | 0.140 | 0.159 | 0.233 | 0.910 | 0.405 | 1.112 |
| comment-links | 278 | gfm | yes | yes | 0.272 | 0.367 | 0.564 | 1.145 | 0.650 | 1.402 |
| comment-checklist | 287 | gfm | no | no | 0.380 | 0.461 | 0.673 | 1.280 | 0.762 | 1.613 |
| comment-quote | 290 | gfm | yes | yes | 0.194 | 0.197 | 0.271 | 0.961 | 0.486 | 1.254 |
| comment-unicode | 327 | gfm | yes | yes | 0.308 | 0.350 | 0.562 | 1.113 | 0.658 | 1.422 |
| comment-inline-code | 285 | gfm | yes | yes | 0.201 | 0.228 | 0.492 | 0.995 | 0.577 | 1.235 |
| comment-reproduction | 298 | gfm | yes | yes | 0.204 | 0.231 | 0.398 | 1.053 | 0.511 | 1.119 |
| comment-table | 310 | gfm | yes | yes | 0.703 | 0.841 | 0.861 | 1.895 | 1.085 | 1.976 |
| comment-review-long | 957 | gfm | yes | yes | 0.494 | 0.572 | 0.784 | 1.840 | 1.333 | 3.838 |
| comment-incident | 1124 | gfm | no | yes | 0.870 | 1.017 | 1.240 | 2.138 | 1.988 | 4.648 |
| guard-angle-link | 41 | commonmark | no | no | 0.161 | 0.210 | 0.400 | 0.843 | 0.320 | 0.438 |
| legacy-contributing | 9323 | gfm | no | yes | 7.609 | 8.411 | 13.263 | 17.822 | 18.255 | 35.196 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 7.606 | 10.091 | 16.458 | 22.632 | 19.931 | 39.197 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.374 | 1.711 | 3.312 | 4.691 | 3.747 | 6.666 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.028 | 2.405 | 3.802 | 5.829 | 4.582 | 9.421 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.540 | 6.001 | 11.539 | 14.201 | 12.630 | 25.517 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 1.762 | 2.243 | 4.384 | 5.746 | 4.745 | 8.705 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.700 | 4.239 | 8.868 | 8.970 | 8.923 | 16.333 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.609 | 2.014 | 3.723 | 5.345 | 5.117 | 11.063 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.388 | 2.983 | 5.799 | 7.704 | 7.332 | 14.011 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 5.210 | 6.891 | 10.300 | 16.348 | 12.543 | 27.206 |
| legacy-docs-readme | 1825 | gfm | no | yes | 2.972 | 4.058 | 5.097 | 7.320 | 4.864 | 9.446 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 0.994 | 1.337 | 2.191 | 3.250 | 2.931 | 6.785 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.414 | 0.658 | 0.850 | 1.584 | 1.131 | 2.307 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 30.758 | 41.435 | 48.310 | 64.786 | 53.790 | 89.767 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.916 | 10.630 | 14.242 | 20.017 | 22.190 | 46.839 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 5.713 | 5.339 | 10.872 | 14.250 | 14.591 | 36.769 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.326 | 4.425 | 5.944 | 10.089 | 9.106 | 26.106 |
| vue-docs-suspense | 8291 | gfm | no | yes | 6.719 | 8.488 | 17.627 | 20.221 | 18.223 | 36.608 |
| vue-docs-slots | 24211 | gfm | no | yes | 15.655 | 27.448 | 42.693 | 63.306 | 50.944 | 113.631 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 13.687 | 22.628 | 32.821 | 53.316 | 46.800 | 104.530 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.098 | 2.883 | 3.855 | 6.470 | 6.341 | 15.639 |
| vite-docs-performance | 8184 | gfm | no | yes | 5.421 | 7.318 | 10.788 | 15.124 | 13.969 | 31.321 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 39.149 | 45.500 | 56.257 | 81.306 | 65.534 | 129.562 |
| vite-docs-features | 39739 | gfm | no | no | 34.271 | 48.145 | 68.852 | 104.894 | 94.171 | 179.279 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.249 | 4.402 | 4.546 | 8.460 | 8.528 | 20.250 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 30.090 | 37.737 | 59.412 | 86.287 | 64.784 | 134.811 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 16.933 | 19.937 | 56.076 | 76.955 | 45.176 | 111.720 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 35.805 | 46.847 | 86.953 | 123.910 | 104.951 | 237.075 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.791 | 1.353 | 1.784 | 2.866 | 2.261 | 4.275 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.222 | 2.003 | 2.620 | 4.168 | 3.799 | 8.250 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 23.304 | 40.256 | 55.481 | 83.828 | 77.139 | 204.337 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 7.241 | 6.876 | 12.422 | 36.047 | 34.864 | 161.467 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.250 | 2.394 | 4.668 | 5.127 | 4.008 | 6.428 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 6.733 | 12.721 | 33.169 | 22.479 | 20.079 | 38.091 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 42.090 | 68.358 | 109.173 | 141.581 | 139.159 | 297.350 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 7.481 | 7.867 | 14.748 | 36.247 | 35.234 | 148.727 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.980 | 1.767 | 3.210 | 4.017 | 3.173 | 6.073 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 2.941 | 6.367 | 8.754 | 11.644 | 10.358 | 20.291 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 72.495 | 140.322 | 171.803 | 254.255 | 246.404 | 594.712 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 17.981 | 20.252 | 29.851 | 78.791 | 78.210 | 341.129 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.737 | 3.543 | 7.779 | 7.115 | 5.831 | 12.339 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.329 | 4.357 | 8.472 | 9.169 | 8.244 | 16.984 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 45.871 | 86.247 | 119.396 | 163.002 | 160.999 | 367.203 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 11.204 | 10.091 | 16.088 | 41.309 | 41.179 | 177.596 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.056 | 0.051 | 0.083 | 0.650 | 0.135 | 0.235 |
| comment-question | 160 | gfm | yes | yes | 0.057 | 0.055 | 0.091 | 0.672 | 0.239 | 0.627 |
| comment-review | 282 | gfm | yes | yes | 0.095 | 0.091 | 0.155 | 0.798 | 0.359 | 1.124 |
| comment-links | 278 | gfm | yes | yes | 0.217 | 0.305 | 0.431 | 1.055 | 0.543 | 1.276 |
| comment-checklist | 287 | gfm | no | no | 0.319 | 0.365 | 0.545 | 1.121 | 0.661 | 1.461 |
| comment-quote | 290 | gfm | yes | yes | 0.140 | 0.125 | 0.186 | 0.866 | 0.389 | 1.199 |
| comment-unicode | 327 | gfm | yes | yes | 0.288 | 0.276 | 0.352 | 1.079 | 0.632 | 1.587 |
| comment-inline-code | 285 | gfm | yes | yes | 0.149 | 0.148 | 0.347 | 0.911 | 0.504 | 1.262 |
| comment-reproduction | 298 | gfm | yes | yes | 0.149 | 0.160 | 0.273 | 0.979 | 0.418 | 1.135 |
| comment-table | 310 | gfm | yes | yes | 0.657 | 0.723 | 0.719 | 1.630 | 0.924 | 2.060 |
| comment-review-long | 957 | gfm | yes | yes | 0.409 | 0.485 | 0.643 | 1.686 | 1.397 | 3.963 |
| comment-incident | 1124 | gfm | no | yes | 0.760 | 0.900 | 1.044 | 2.247 | 1.726 | 4.609 |
| guard-angle-link | 41 | commonmark | no | no | 0.118 | 0.129 | 0.230 | 0.725 | 0.238 | 0.376 |
| legacy-contributing | 9323 | gfm | no | yes | 7.017 | 8.503 | 11.975 | 16.342 | 17.978 | 35.851 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 6.455 | 7.892 | 13.385 | 19.834 | 17.596 | 34.673 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.207 | 1.643 | 2.979 | 4.132 | 3.485 | 6.566 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.014 | 2.546 | 3.742 | 5.829 | 4.648 | 9.490 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.394 | 5.490 | 12.103 | 14.927 | 11.969 | 26.165 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 1.959 | 2.388 | 4.042 | 5.953 | 4.976 | 9.484 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.532 | 3.774 | 7.455 | 8.357 | 9.097 | 15.994 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.630 | 1.843 | 3.333 | 4.848 | 4.680 | 11.654 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.410 | 2.809 | 5.756 | 7.361 | 7.308 | 15.376 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 4.635 | 5.855 | 10.213 | 15.101 | 12.387 | 27.432 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.000 | 3.616 | 4.555 | 7.208 | 4.889 | 9.945 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 0.901 | 1.162 | 1.810 | 3.117 | 2.877 | 6.095 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.335 | 0.504 | 0.668 | 1.458 | 0.928 | 2.063 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 32.099 | 44.175 | 49.573 | 64.390 | 51.291 | 82.225 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.116 | 10.470 | 11.943 | 19.681 | 20.584 | 41.236 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 5.752 | 5.556 | 9.763 | 13.657 | 15.242 | 37.004 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.436 | 5.083 | 6.079 | 9.542 | 8.536 | 23.953 |
| vue-docs-suspense | 8291 | gfm | no | yes | 6.409 | 8.210 | 15.857 | 18.264 | 16.539 | 34.430 |
| vue-docs-slots | 24211 | gfm | no | yes | 14.475 | 26.656 | 41.233 | 56.929 | 45.634 | 109.204 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 15.690 | 24.062 | 36.666 | 56.553 | 46.705 | 113.999 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.002 | 2.835 | 3.447 | 6.367 | 6.054 | 15.315 |
| vite-docs-performance | 8184 | gfm | no | yes | 5.929 | 7.270 | 10.493 | 15.824 | 15.058 | 33.557 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 37.453 | 45.000 | 55.516 | 78.233 | 65.080 | 127.358 |
| vite-docs-features | 39739 | gfm | no | no | 33.733 | 45.209 | 66.474 | 115.235 | 86.900 | 170.308 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 2.993 | 4.285 | 4.250 | 8.427 | 7.607 | 22.049 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 31.342 | 42.495 | 56.222 | 83.213 | 66.202 | 138.486 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 17.468 | 18.123 | 54.518 | 71.763 | 39.952 | 109.696 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 39.505 | 50.323 | 87.467 | 119.274 | 102.697 | 247.983 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.683 | 1.233 | 1.551 | 2.821 | 2.139 | 4.061 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.126 | 1.776 | 2.283 | 4.425 | 3.709 | 9.010 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 25.054 | 40.685 | 53.876 | 86.310 | 77.825 | 204.182 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 6.613 | 6.370 | 10.145 | 31.897 | 29.617 | 138.827 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.227 | 2.241 | 3.948 | 4.468 | 3.359 | 6.245 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 6.097 | 11.985 | 34.189 | 22.382 | 21.098 | 34.558 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 41.766 | 77.401 | 107.871 | 147.362 | 145.505 | 317.876 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 7.612 | 8.441 | 12.661 | 34.772 | 32.273 | 161.889 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.941 | 1.719 | 2.610 | 3.802 | 2.934 | 5.928 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 2.842 | 5.882 | 8.173 | 12.304 | 10.218 | 20.248 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 75.718 | 131.112 | 178.231 | 257.818 | 259.003 | 584.667 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 19.417 | 18.206 | 29.440 | 78.495 | 70.779 | 354.281 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.866 | 3.378 | 6.702 | 6.522 | 5.974 | 12.852 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.387 | 4.768 | 7.779 | 9.646 | 8.574 | 18.838 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 44.790 | 89.758 | 113.072 | 149.906 | 150.142 | 355.025 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 10.396 | 10.290 | 17.151 | 43.659 | 40.259 | 185.227 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 315.43 | 457.92 | 768.91 | 998.58 | 944.90 | 2756.53 |
| rotating-commonmark | reuse | 17 | 283.62 | 438.72 | 686.55 | 952.16 | 848.15 | 2555.96 |
| rotating-gfm | fresh | 40 | 426.62 | 558.46 | 978.62 | 1093.00 | 955.42 | 2012.50 |
| rotating-gfm | reuse | 40 | 421.54 | 564.93 | 936.88 | 1027.46 | 888.31 | 1888.95 |
