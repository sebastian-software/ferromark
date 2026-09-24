# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.71× | 0.58× | 0.24× | 0.41× | 0.20× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.47× | 0.31× | 0.40× | 0.21× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.68× | 0.47× | 0.31× | 0.40× | 0.21× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.58× | 0.26× | 0.51× | 0.28× |
| comments (all workloads) | 12 | 1.00× | 0.73× | 0.58× | 0.26× | 0.51× | 0.29× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.36× | 0.30× | 0.36× | 0.19× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.60× | 0.37× | 0.30× | 0.36× | 0.18× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.58× | 0.22× | 0.26× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.62× | 0.58× | 0.22× | 0.26× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.54× | 0.41× | 0.50× | 0.29× |
| readme (all workloads) | 2 | 1.00× | 0.75× | 0.54× | 0.41× | 0.50× | 0.29× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.48× | 0.35× | 0.48× | 0.26× |
| reference (all workloads) | 4 | 1.00× | 0.71× | 0.48× | 0.35× | 0.48× | 0.26× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.76× | 0.42× | 0.25× | 0.62× | 0.50× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.45× | 0.35× | 0.38× | 0.21× |
| technical-docs (all workloads) | 22 | 1.00× | 0.69× | 0.45× | 0.35× | 0.38× | 0.21× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.42× | 0.27× | 0.32× | 0.15× |
| commonmark (all workloads) | 17 | 1.00× | 0.61× | 0.42× | 0.28× | 0.34× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.49× | 0.32× | 0.43× | 0.24× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.71× | 0.49× | 0.32× | 0.43× | 0.24× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.57× | 0.24× | 0.50× | 0.29× |
| <512 B (all workloads) | 12 | 1.00× | 0.73× | 0.55× | 0.25× | 0.51× | 0.30× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.47× | 0.34× | 0.42× | 0.23× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.65× | 0.47× | 0.34× | 0.42× | 0.23× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.42× | 0.34× | 0.37× | 0.20× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.66× | 0.42× | 0.34× | 0.37× | 0.20× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.48× | 0.39× | 0.43× | 0.24× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.73× | 0.48× | 0.39× | 0.43× | 0.24× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.47× | 0.23× | 0.30× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.62× | 0.45× | 0.26× | 0.32× | 0.14× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.73× | 0.62× | 0.20× | 0.36× | 0.16× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.50× | 0.29× | 0.39× | 0.20× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.69× | 0.50× | 0.30× | 0.39× | 0.20× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.62× | 0.21× | 0.42× | 0.21× |
| comments (all workloads) | 12 | 1.00× | 0.76× | 0.62× | 0.21× | 0.43× | 0.22× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.39× | 0.30× | 0.36× | 0.19× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.60× | 0.40× | 0.30× | 0.36× | 0.18× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.64× | 0.23× | 0.26× | 0.09× |
| plain-prose (all workloads) | 4 | 1.00× | 0.62× | 0.64× | 0.23× | 0.26× | 0.09× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.58× | 0.42× | 0.51× | 0.29× |
| readme (all workloads) | 2 | 1.00× | 0.77× | 0.58× | 0.42× | 0.51× | 0.29× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.48× | 0.36× | 0.49× | 0.26× |
| reference (all workloads) | 4 | 1.00× | 0.71× | 0.48× | 0.36× | 0.49× | 0.26× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.90× | 0.51× | 0.21× | 0.55× | 0.40× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.47× | 0.35× | 0.38× | 0.20× |
| technical-docs (all workloads) | 22 | 1.00× | 0.69× | 0.47× | 0.35× | 0.39× | 0.20× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.46× | 0.27× | 0.33× | 0.14× |
| commonmark (all workloads) | 17 | 1.00× | 0.62× | 0.45× | 0.28× | 0.34× | 0.16× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.52× | 0.30× | 0.41× | 0.21× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.72× | 0.52× | 0.30× | 0.41× | 0.22× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.60× | 0.19× | 0.42× | 0.21× |
| <512 B (all workloads) | 12 | 1.00× | 0.77× | 0.59× | 0.20× | 0.43× | 0.23× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.51× | 0.33× | 0.41× | 0.22× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.65× | 0.51× | 0.33× | 0.41× | 0.22× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.44× | 0.34× | 0.37× | 0.20× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.67× | 0.44× | 0.34× | 0.37× | 0.20× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.50× | 0.39× | 0.44× | 0.24× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.74× | 0.50× | 0.39× | 0.44× | 0.24× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.50× | 0.24× | 0.30× | 0.12× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.63× | 0.47× | 0.27× | 0.33× | 0.14× |

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
| comment-ack | 37 | gfm | yes | yes | 0.201 | 0.287 | 0.271 | 1.246 | 0.315 | 0.428 |
| comment-question | 160 | gfm | yes | yes | 0.219 | 0.304 | 0.342 | 1.391 | 0.466 | 0.936 |
| comment-review | 282 | gfm | yes | yes | 0.284 | 0.409 | 0.528 | 1.665 | 0.729 | 1.586 |
| comment-links | 278 | gfm | yes | yes | 0.659 | 0.903 | 1.365 | 2.266 | 1.283 | 2.422 |
| comment-checklist | 287 | gfm | no | no | 0.799 | 1.256 | 1.529 | 2.363 | 1.487 | 2.625 |
| comment-quote | 290 | gfm | yes | yes | 0.400 | 0.544 | 0.685 | 1.800 | 0.873 | 1.763 |
| comment-unicode | 327 | gfm | yes | yes | 0.706 | 0.791 | 1.274 | 2.163 | 1.328 | 2.068 |
| comment-inline-code | 285 | gfm | yes | yes | 0.432 | 0.537 | 1.158 | 1.875 | 1.121 | 1.687 |
| comment-reproduction | 298 | gfm | yes | yes | 0.472 | 0.621 | 0.885 | 2.056 | 0.941 | 1.926 |
| comment-table | 310 | gfm | yes | yes | 1.600 | 1.961 | 1.758 | 3.318 | 1.934 | 2.804 |
| comment-review-long | 957 | gfm | yes | yes | 1.068 | 1.591 | 2.044 | 3.445 | 2.565 | 5.200 |
| comment-incident | 1124 | gfm | no | yes | 2.012 | 2.877 | 2.905 | 4.301 | 3.545 | 6.443 |
| guard-angle-link | 41 | commonmark | no | no | 0.418 | 0.551 | 0.997 | 1.644 | 0.671 | 0.841 |
| legacy-contributing | 9323 | gfm | no | yes | 14.277 | 18.413 | 28.084 | 35.331 | 34.407 | 58.115 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 13.351 | 18.255 | 29.526 | 38.377 | 33.905 | 59.275 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.733 | 4.208 | 6.908 | 8.177 | 7.715 | 12.401 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 3.848 | 5.432 | 7.600 | 9.814 | 8.782 | 14.555 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 9.031 | 12.493 | 22.986 | 28.118 | 26.143 | 44.612 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 3.412 | 5.055 | 8.748 | 10.395 | 9.424 | 15.359 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.575 | 8.911 | 16.442 | 16.580 | 18.142 | 27.531 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 3.370 | 4.446 | 7.784 | 10.719 | 9.878 | 18.041 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.886 | 6.849 | 12.576 | 14.184 | 14.938 | 24.925 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 10.127 | 13.481 | 22.265 | 30.268 | 24.074 | 46.059 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.755 | 8.694 | 10.666 | 14.132 | 10.586 | 18.042 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.857 | 2.852 | 4.317 | 5.861 | 5.276 | 9.719 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.927 | 1.566 | 1.987 | 3.070 | 2.156 | 3.695 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 62.617 | 78.830 | 94.415 | 118.814 | 94.051 | 130.589 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.769 | 22.656 | 28.032 | 36.330 | 37.220 | 66.700 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 10.271 | 11.772 | 20.847 | 26.553 | 26.612 | 52.487 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.611 | 11.335 | 12.532 | 19.067 | 16.717 | 36.416 |
| vue-docs-suspense | 8291 | gfm | no | yes | 11.843 | 16.625 | 32.728 | 33.661 | 31.322 | 54.215 |
| vue-docs-slots | 24211 | gfm | no | yes | 28.281 | 51.282 | 94.518 | 111.138 | 87.080 | 194.851 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 27.717 | 45.006 | 69.501 | 95.082 | 80.458 | 166.677 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.320 | 6.900 | 7.902 | 12.521 | 10.827 | 22.204 |
| vite-docs-performance | 8184 | gfm | no | yes | 11.221 | 16.354 | 21.355 | 29.962 | 27.853 | 49.969 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 83.796 | 109.570 | 133.467 | 164.266 | 137.379 | 242.149 |
| vite-docs-features | 39739 | gfm | no | no | 71.986 | 104.895 | 158.532 | 200.641 | 176.925 | 307.714 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.497 | 10.063 | 10.021 | 16.168 | 14.285 | 29.269 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 52.110 | 81.751 | 120.151 | 167.358 | 132.697 | 234.226 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.358 | 36.345 | 82.638 | 126.404 | 68.744 | 192.293 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 66.609 | 93.607 | 175.260 | 235.613 | 185.013 | 358.676 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.610 | 2.587 | 3.524 | 5.333 | 4.106 | 7.332 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.433 | 3.946 | 5.369 | 8.346 | 6.663 | 13.320 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 45.696 | 78.156 | 110.930 | 162.771 | 140.438 | 306.495 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 11.604 | 18.384 | 19.758 | 60.530 | 50.788 | 145.640 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.804 | 4.610 | 7.700 | 8.543 | 6.865 | 11.706 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 12.433 | 20.241 | 50.288 | 38.206 | 33.190 | 62.257 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 80.109 | 134.270 | 210.188 | 253.635 | 223.245 | 461.459 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 14.696 | 23.102 | 25.720 | 64.594 | 55.271 | 168.087 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.187 | 3.738 | 5.450 | 7.381 | 5.723 | 10.334 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 6.220 | 10.788 | 17.000 | 21.674 | 18.460 | 36.108 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 163.326 | 272.820 | 366.781 | 489.379 | 420.821 | 911.838 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 32.549 | 53.362 | 55.148 | 138.758 | 118.127 | 344.927 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.812 | 6.301 | 12.189 | 13.122 | 11.279 | 21.509 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 4.974 | 8.341 | 15.302 | 18.032 | 15.308 | 30.982 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 87.649 | 155.418 | 223.320 | 299.344 | 265.555 | 539.822 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 18.274 | 29.759 | 31.326 | 78.905 | 70.532 | 195.554 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.116 | 0.152 | 0.150 | 1.183 | 0.260 | 0.440 |
| comment-question | 160 | gfm | yes | yes | 0.135 | 0.169 | 0.190 | 1.327 | 0.391 | 0.936 |
| comment-review | 282 | gfm | yes | yes | 0.198 | 0.257 | 0.328 | 1.567 | 0.654 | 1.586 |
| comment-links | 278 | gfm | yes | yes | 0.523 | 0.727 | 0.993 | 2.101 | 1.153 | 2.408 |
| comment-checklist | 287 | gfm | no | no | 0.684 | 1.071 | 1.255 | 2.191 | 1.370 | 2.626 |
| comment-quote | 290 | gfm | yes | yes | 0.289 | 0.378 | 0.441 | 1.669 | 0.755 | 1.770 |
| comment-unicode | 327 | gfm | yes | yes | 0.599 | 0.606 | 0.883 | 2.012 | 1.227 | 2.049 |
| comment-inline-code | 285 | gfm | yes | yes | 0.301 | 0.373 | 0.876 | 1.731 | 0.976 | 1.696 |
| comment-reproduction | 298 | gfm | yes | yes | 0.339 | 0.446 | 0.675 | 1.907 | 0.846 | 1.931 |
| comment-table | 310 | gfm | yes | yes | 1.455 | 1.750 | 1.475 | 3.189 | 1.795 | 2.830 |
| comment-review-long | 957 | gfm | yes | yes | 0.913 | 1.397 | 1.611 | 3.265 | 2.423 | 5.209 |
| comment-incident | 1124 | gfm | no | yes | 1.834 | 2.660 | 2.502 | 4.096 | 3.379 | 6.473 |
| guard-angle-link | 41 | commonmark | no | no | 0.329 | 0.365 | 0.651 | 1.555 | 0.594 | 0.816 |
| legacy-contributing | 9323 | gfm | no | yes | 14.000 | 18.164 | 26.972 | 34.018 | 33.779 | 57.519 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 13.242 | 17.844 | 27.807 | 36.736 | 33.028 | 59.569 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 2.589 | 3.949 | 6.558 | 7.797 | 7.390 | 12.485 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 3.716 | 5.146 | 7.016 | 9.283 | 8.362 | 14.590 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 8.893 | 12.091 | 22.271 | 27.572 | 25.450 | 44.638 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 3.216 | 4.797 | 7.901 | 9.847 | 9.012 | 15.249 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 6.327 | 8.591 | 15.440 | 15.917 | 17.547 | 27.614 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 3.191 | 4.262 | 6.987 | 10.124 | 9.502 | 17.966 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 4.708 | 6.637 | 11.816 | 13.730 | 14.467 | 24.747 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 9.883 | 13.157 | 20.852 | 29.623 | 23.394 | 46.118 |
| legacy-docs-readme | 1825 | gfm | no | yes | 6.632 | 8.364 | 9.491 | 13.641 | 10.236 | 18.037 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.772 | 2.621 | 3.723 | 5.586 | 5.007 | 9.689 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.805 | 1.346 | 1.610 | 2.868 | 1.959 | 3.669 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 61.995 | 78.392 | 92.003 | 117.620 | 91.648 | 131.890 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 18.613 | 22.299 | 26.074 | 35.964 | 36.617 | 67.303 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 10.093 | 11.390 | 19.391 | 25.705 | 26.114 | 53.343 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 6.386 | 10.982 | 11.835 | 18.523 | 16.408 | 36.018 |
| vue-docs-suspense | 8291 | gfm | no | yes | 11.755 | 16.079 | 30.822 | 32.569 | 30.908 | 54.399 |
| vue-docs-slots | 24211 | gfm | no | yes | 27.919 | 50.423 | 91.137 | 109.051 | 85.328 | 194.882 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 27.322 | 44.489 | 66.735 | 92.746 | 78.225 | 166.157 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 4.228 | 6.546 | 7.117 | 11.945 | 10.245 | 22.068 |
| vite-docs-performance | 8184 | gfm | no | yes | 11.174 | 15.909 | 20.276 | 28.864 | 27.044 | 49.902 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 83.846 | 109.451 | 131.273 | 161.416 | 135.544 | 241.194 |
| vite-docs-features | 39739 | gfm | no | no | 71.370 | 105.293 | 155.000 | 195.541 | 172.958 | 308.040 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 6.392 | 9.735 | 9.295 | 15.641 | 13.829 | 29.536 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 52.165 | 80.872 | 116.695 | 162.877 | 130.223 | 233.096 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 23.050 | 35.864 | 80.712 | 122.974 | 67.396 | 190.024 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 66.277 | 93.281 | 173.374 | 231.372 | 182.818 | 357.812 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 1.494 | 2.360 | 2.881 | 5.122 | 3.877 | 7.329 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 2.297 | 3.749 | 4.562 | 7.992 | 6.534 | 13.226 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 46.633 | 77.764 | 104.456 | 158.889 | 138.560 | 305.283 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 11.526 | 18.190 | 17.218 | 58.055 | 48.946 | 145.633 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 2.716 | 4.376 | 6.835 | 8.259 | 6.542 | 11.831 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 12.025 | 19.882 | 46.752 | 37.476 | 32.531 | 62.219 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 78.658 | 132.121 | 203.798 | 249.427 | 219.862 | 462.739 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 14.415 | 22.736 | 22.828 | 63.060 | 53.873 | 168.038 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 2.066 | 3.571 | 4.770 | 7.154 | 5.490 | 10.366 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 6.107 | 10.642 | 15.391 | 21.158 | 17.654 | 36.050 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 162.161 | 270.173 | 353.322 | 482.917 | 412.867 | 909.880 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 31.975 | 52.308 | 50.511 | 132.724 | 113.012 | 346.787 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 3.641 | 6.006 | 11.811 | 12.663 | 10.633 | 21.453 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 4.801 | 8.166 | 14.010 | 17.369 | 14.880 | 31.087 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 87.688 | 154.637 | 217.969 | 293.698 | 253.977 | 537.062 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 17.959 | 29.330 | 28.687 | 76.594 | 67.684 | 195.074 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 636.42 | 969.01 | 1327.45 | 1757.49 | 1504.31 | 3453.51 |
| rotating-commonmark | reuse | 17 | 615.97 | 938.62 | 1271.11 | 1729.07 | 1460.79 | 3440.58 |
| rotating-gfm | fresh | 40 | 872.36 | 1216.82 | 1738.24 | 1956.72 | 1630.06 | 2847.16 |
| rotating-gfm | reuse | 40 | 851.49 | 1164.48 | 1648.66 | 1919.50 | 1559.64 | 2842.53 |
