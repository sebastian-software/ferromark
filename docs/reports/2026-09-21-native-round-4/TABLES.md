# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.84× | 0.60× | 0.22× | 0.39× | 0.14× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.48× | 0.29× | 0.38× | 0.17× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.72× | 0.47× | 0.29× | 0.38× | 0.17× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.60× | 0.22× | 0.45× | 0.20× |
| comments (all workloads) | 12 | 1.00× | 0.80× | 0.60× | 0.23× | 0.46× | 0.21× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.30× | 0.25× | 0.30× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.55× | 0.33× | 0.26× | 0.31× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.61× | 0.24× | 0.28× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 0.95× | 0.61× | 0.24× | 0.28× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.57× | 0.38× | 0.47× | 0.24× |
| readme (all workloads) | 2 | 1.00× | 0.81× | 0.57× | 0.38× | 0.47× | 0.24× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.55× | 0.38× | 0.53× | 0.26× |
| reference (all workloads) | 4 | 1.00× | 0.81× | 0.55× | 0.38× | 0.53× | 0.26× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.67× | 0.40× | 0.20× | 0.54× | 0.40× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.47× | 0.32× | 0.37× | 0.17× |
| technical-docs (all workloads) | 22 | 1.00× | 0.72× | 0.47× | 0.32× | 0.37× | 0.17× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.38× | 0.24× | 0.29× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.64× | 0.38× | 0.25× | 0.31× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.30× | 0.41× | 0.19× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.76× | 0.52× | 0.30× | 0.41× | 0.19× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.57× | 0.21× | 0.44× | 0.21× |
| <512 B (all workloads) | 12 | 1.00× | 0.77× | 0.56× | 0.22× | 0.46× | 0.22× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.46× | 0.29× | 0.37× | 0.18× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.67× | 0.46× | 0.29× | 0.37× | 0.18× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.41× | 0.31× | 0.34× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.67× | 0.41× | 0.31× | 0.34× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.38× | 0.42× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.78× | 0.52× | 0.38× | 0.42× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.26× | 0.33× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.73× | 0.45× | 0.27× | 0.33× | 0.11× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.92× | 0.67× | 0.19× | 0.36× | 0.12× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.52× | 0.27× | 0.37× | 0.15× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.74× | 0.51× | 0.28× | 0.37× | 0.16× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.68× | 0.19× | 0.40× | 0.16× |
| comments (all workloads) | 12 | 1.00× | 0.90× | 0.68× | 0.20× | 0.41× | 0.16× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.32× | 0.24× | 0.30× | 0.15× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.56× | 0.34× | 0.26× | 0.31× | 0.14× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.68× | 0.25× | 0.30× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 0.96× | 0.68× | 0.25× | 0.30× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.60× | 0.38× | 0.48× | 0.24× |
| readme (all workloads) | 2 | 1.00× | 0.81× | 0.60× | 0.38× | 0.48× | 0.24× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.57× | 0.39× | 0.54× | 0.26× |
| reference (all workloads) | 4 | 1.00× | 0.82× | 0.57× | 0.39× | 0.54× | 0.26× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.76× | 0.48× | 0.15× | 0.46× | 0.30× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.50× | 0.33× | 0.37× | 0.16× |
| technical-docs (all workloads) | 22 | 1.00× | 0.73× | 0.49× | 0.33× | 0.37× | 0.16× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.41× | 0.25× | 0.30× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.64× | 0.41× | 0.25× | 0.31× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.56× | 0.28× | 0.40× | 0.17× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.79× | 0.56× | 0.29× | 0.40× | 0.18× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.65× | 0.17× | 0.39× | 0.16× |
| <512 B (all workloads) | 12 | 1.00× | 0.87× | 0.63× | 0.18× | 0.40× | 0.17× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.50× | 0.29× | 0.36× | 0.17× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.68× | 0.50× | 0.29× | 0.36× | 0.17× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.31× | 0.35× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.68× | 0.43× | 0.31× | 0.35× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.38× | 0.43× | 0.19× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.78× | 0.54× | 0.38× | 0.43× | 0.19× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.55× | 0.26× | 0.34× | 0.09× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.74× | 0.49× | 0.28× | 0.34× | 0.11× |

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
| comment-ack | 37 | gfm | yes | yes | 0.103 | 0.155 | 0.159 | 0.772 | 0.189 | 0.294 |
| comment-question | 160 | gfm | yes | yes | 0.119 | 0.163 | 0.180 | 0.864 | 0.288 | 0.763 |
| comment-review | 282 | gfm | yes | yes | 0.169 | 0.211 | 0.291 | 1.042 | 0.488 | 1.300 |
| comment-links | 278 | gfm | yes | yes | 0.327 | 0.472 | 0.694 | 1.411 | 0.749 | 1.609 |
| comment-checklist | 287 | gfm | no | no | 0.494 | 0.609 | 0.826 | 1.542 | 0.909 | 1.789 |
| comment-quote | 290 | gfm | yes | yes | 0.224 | 0.274 | 0.349 | 1.139 | 0.561 | 1.415 |
| comment-unicode | 327 | gfm | yes | yes | 0.399 | 0.425 | 0.699 | 1.322 | 0.799 | 1.669 |
| comment-inline-code | 285 | gfm | yes | yes | 0.234 | 0.285 | 0.598 | 1.205 | 0.680 | 1.385 |
| comment-reproduction | 298 | gfm | yes | yes | 0.248 | 0.323 | 0.471 | 1.310 | 0.584 | 1.370 |
| comment-table | 310 | gfm | yes | yes | 0.999 | 0.988 | 0.982 | 2.153 | 1.204 | 2.137 |
| comment-review-long | 957 | gfm | yes | yes | 0.592 | 0.762 | 1.010 | 2.133 | 1.620 | 4.359 |
| comment-incident | 1124 | gfm | no | yes | 1.070 | 1.307 | 1.502 | 2.813 | 2.169 | 5.307 |
| guard-angle-link | 41 | commonmark | no | no | 0.188 | 0.282 | 0.470 | 0.959 | 0.352 | 0.465 |
| legacy-contributing | 9323 | gfm | no | yes | 8.806 | 10.905 | 16.163 | 23.064 | 22.298 | 46.354 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.443 | 10.844 | 17.182 | 24.807 | 22.396 | 45.477 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.704 | 2.444 | 4.132 | 5.489 | 4.698 | 8.625 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.324 | 3.030 | 4.372 | 6.440 | 5.369 | 10.483 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.819 | 7.559 | 13.989 | 18.950 | 16.426 | 31.989 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.232 | 3.021 | 5.004 | 6.915 | 5.641 | 10.355 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.234 | 5.167 | 9.929 | 10.651 | 11.738 | 20.118 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.962 | 2.579 | 4.552 | 6.766 | 6.334 | 13.233 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.083 | 3.997 | 7.410 | 9.226 | 9.330 | 18.099 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.232 | 7.633 | 12.904 | 19.215 | 15.332 | 35.682 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.778 | 4.488 | 5.725 | 8.850 | 6.379 | 12.182 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.113 | 1.564 | 2.482 | 3.747 | 3.400 | 7.446 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.458 | 0.791 | 1.052 | 1.963 | 1.298 | 2.546 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 49.051 | 48.935 | 59.711 | 79.211 | 64.395 | 103.949 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.451 | 13.313 | 16.968 | 24.094 | 25.802 | 58.309 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.791 | 6.997 | 12.644 | 17.881 | 18.177 | 48.860 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.122 | 6.375 | 7.367 | 12.835 | 11.443 | 31.012 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.543 | 9.830 | 19.152 | 22.162 | 20.132 | 42.808 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.188 | 35.737 | 66.467 | 82.273 | 63.226 | 143.659 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 17.102 | 28.472 | 44.004 | 67.644 | 54.478 | 137.783 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.481 | 3.917 | 4.583 | 8.250 | 7.324 | 18.236 |
| vite-docs-performance | 8184 | gfm | no | yes | 6.898 | 9.122 | 12.286 | 19.270 | 18.043 | 41.742 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 55.201 | 71.882 | 79.501 | 116.265 | 93.049 | 188.035 |
| vite-docs-features | 39739 | gfm | no | no | 47.118 | 71.149 | 103.722 | 140.923 | 121.903 | 246.200 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.030 | 5.678 | 5.767 | 10.531 | 9.290 | 25.884 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 39.885 | 59.920 | 84.571 | 121.513 | 95.782 | 190.770 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.365 | 25.022 | 65.075 | 99.679 | 51.956 | 145.052 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 46.592 | 71.798 | 120.594 | 165.931 | 128.891 | 296.283 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.887 | 1.584 | 2.115 | 3.607 | 2.718 | 5.131 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.426 | 2.429 | 3.152 | 5.720 | 4.684 | 9.964 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.239 | 51.279 | 76.364 | 112.729 | 103.486 | 284.939 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.905 | 8.957 | 15.334 | 43.079 | 36.301 | 188.445 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.565 | 2.935 | 5.410 | 6.095 | 4.770 | 8.120 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.358 | 13.913 | 39.939 | 27.696 | 24.326 | 43.246 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 54.301 | 91.743 | 160.404 | 180.829 | 166.814 | 378.825 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 11.450 | 12.016 | 18.909 | 45.588 | 39.278 | 203.310 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.243 | 2.273 | 3.697 | 5.096 | 4.042 | 7.291 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.627 | 7.050 | 10.747 | 15.224 | 13.204 | 25.108 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 107.793 | 185.024 | 269.780 | 344.160 | 313.951 | 727.080 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 26.605 | 28.178 | 41.729 | 98.420 | 83.692 | 415.548 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.152 | 4.208 | 9.053 | 9.269 | 7.679 | 15.191 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.896 | 5.510 | 10.686 | 12.533 | 10.718 | 23.058 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 61.399 | 107.757 | 165.060 | 212.271 | 196.479 | 437.894 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 14.214 | 15.330 | 22.803 | 55.731 | 49.534 | 235.145 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.059 | 0.066 | 0.083 | 0.733 | 0.153 | 0.291 |
| comment-question | 160 | gfm | yes | yes | 0.075 | 0.077 | 0.103 | 0.827 | 0.257 | 0.763 |
| comment-review | 282 | gfm | yes | yes | 0.122 | 0.127 | 0.190 | 0.992 | 0.439 | 1.307 |
| comment-links | 278 | gfm | yes | yes | 0.274 | 0.378 | 0.498 | 1.330 | 0.660 | 1.601 |
| comment-checklist | 287 | gfm | no | no | 0.439 | 0.513 | 0.659 | 1.414 | 0.820 | 1.788 |
| comment-quote | 290 | gfm | yes | yes | 0.175 | 0.187 | 0.235 | 1.053 | 0.478 | 1.417 |
| comment-unicode | 327 | gfm | yes | yes | 0.340 | 0.333 | 0.444 | 1.280 | 0.733 | 1.678 |
| comment-inline-code | 285 | gfm | yes | yes | 0.186 | 0.196 | 0.445 | 1.117 | 0.602 | 1.374 |
| comment-reproduction | 298 | gfm | yes | yes | 0.199 | 0.235 | 0.355 | 1.228 | 0.519 | 1.373 |
| comment-table | 310 | gfm | yes | yes | 0.941 | 0.901 | 0.840 | 2.082 | 1.125 | 2.143 |
| comment-review-long | 957 | gfm | yes | yes | 0.539 | 0.664 | 0.810 | 2.088 | 1.560 | 4.406 |
| comment-incident | 1124 | gfm | no | yes | 1.007 | 1.192 | 1.293 | 2.660 | 2.056 | 5.307 |
| guard-angle-link | 41 | commonmark | no | no | 0.140 | 0.185 | 0.293 | 0.914 | 0.308 | 0.465 |
| legacy-contributing | 9323 | gfm | no | yes | 8.750 | 10.771 | 15.426 | 22.326 | 21.611 | 46.196 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.282 | 10.698 | 15.939 | 24.377 | 21.828 | 45.318 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.645 | 2.312 | 3.825 | 5.310 | 4.531 | 8.662 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.252 | 2.902 | 4.064 | 6.253 | 5.105 | 10.489 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.789 | 7.440 | 13.447 | 18.631 | 15.930 | 32.275 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.176 | 2.935 | 4.584 | 6.761 | 5.527 | 10.493 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.167 | 5.049 | 9.234 | 10.389 | 11.361 | 20.215 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 1.889 | 2.440 | 4.112 | 6.467 | 6.024 | 13.162 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.001 | 3.881 | 6.925 | 8.984 | 9.001 | 18.026 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.213 | 7.558 | 12.203 | 18.806 | 14.927 | 35.704 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.699 | 4.387 | 5.282 | 8.691 | 6.171 | 12.205 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.043 | 1.446 | 2.162 | 3.572 | 3.261 | 7.438 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.390 | 0.682 | 0.844 | 1.846 | 1.189 | 2.538 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 48.709 | 48.507 | 58.066 | 78.122 | 63.499 | 103.251 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.362 | 13.143 | 15.784 | 23.466 | 25.157 | 58.112 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.658 | 6.843 | 11.898 | 17.632 | 17.661 | 48.865 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.062 | 6.237 | 6.902 | 12.450 | 11.124 | 31.323 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.451 | 9.696 | 18.104 | 21.774 | 19.320 | 42.891 |
| vue-docs-slots | 24211 | gfm | no | yes | 19.078 | 35.363 | 62.841 | 80.566 | 61.353 | 143.139 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 16.990 | 28.196 | 41.217 | 65.575 | 53.258 | 136.956 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.409 | 3.758 | 4.148 | 7.968 | 7.031 | 18.222 |
| vite-docs-performance | 8184 | gfm | no | yes | 6.795 | 8.967 | 11.528 | 18.819 | 17.347 | 41.941 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 54.972 | 72.374 | 78.548 | 114.361 | 91.797 | 187.658 |
| vite-docs-features | 39739 | gfm | no | no | 48.076 | 73.514 | 101.359 | 141.674 | 121.877 | 250.544 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.961 | 5.520 | 5.299 | 10.316 | 9.083 | 26.245 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 40.702 | 59.668 | 79.589 | 118.735 | 93.937 | 191.965 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 21.348 | 24.888 | 62.845 | 97.226 | 50.495 | 145.542 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 48.726 | 71.695 | 117.763 | 164.762 | 128.515 | 299.258 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.820 | 1.464 | 1.765 | 3.482 | 2.594 | 5.110 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.368 | 2.321 | 2.768 | 5.575 | 4.533 | 9.930 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.372 | 50.107 | 71.527 | 109.976 | 101.382 | 284.023 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 8.810 | 8.796 | 13.255 | 41.135 | 34.203 | 189.145 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.514 | 2.830 | 5.035 | 6.012 | 4.661 | 8.197 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.406 | 13.877 | 38.654 | 27.528 | 24.159 | 43.853 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 52.955 | 90.763 | 153.092 | 177.322 | 164.871 | 375.837 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 11.385 | 11.829 | 16.676 | 44.109 | 37.509 | 203.001 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.189 | 2.145 | 3.254 | 4.944 | 3.915 | 7.325 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.615 | 6.956 | 9.990 | 14.991 | 12.960 | 25.281 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 107.982 | 183.835 | 263.830 | 342.854 | 310.421 | 729.687 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 26.355 | 27.797 | 37.872 | 94.210 | 80.578 | 416.943 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.093 | 4.101 | 8.603 | 9.131 | 7.518 | 15.223 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.850 | 5.415 | 10.271 | 12.537 | 10.510 | 23.457 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 60.737 | 107.735 | 162.336 | 209.939 | 193.919 | 441.415 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 14.128 | 15.227 | 20.484 | 53.758 | 46.827 | 236.855 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 425.98 | 654.26 | 1027.84 | 1288.32 | 1135.39 | 3161.88 |
| rotating-commonmark | reuse | 17 | 409.94 | 633.63 | 971.69 | 1248.93 | 1102.03 | 3152.62 |
| rotating-gfm | fresh | 40 | 622.09 | 857.43 | 1253.38 | 1433.59 | 1175.03 | 2330.51 |
| rotating-gfm | reuse | 40 | 607.35 | 836.03 | 1201.68 | 1408.34 | 1144.34 | 2331.56 |
