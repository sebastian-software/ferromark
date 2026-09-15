# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.01× | 0.72× | 0.27× | 0.47× | 0.17× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.54× | 0.32× | 0.42× | 0.19× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.81× | 0.53× | 0.32× | 0.42× | 0.19× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.77× | 0.29× | 0.58× | 0.26× |
| comments (all workloads) | 12 | 1.00× | 1.03× | 0.77× | 0.30× | 0.58× | 0.27× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.31× | 0.25× | 0.30× | 0.16× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.56× | 0.33× | 0.26× | 0.31× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.66× | 0.26× | 0.31× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.03× | 0.66× | 0.26× | 0.31× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.60× | 0.40× | 0.49× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.86× | 0.60× | 0.40× | 0.49× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.60× | 0.42× | 0.57× | 0.28× |
| reference (all workloads) | 4 | 1.00× | 0.90× | 0.60× | 0.42× | 0.57× | 0.28× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.98× | 0.58× | 0.28× | 0.79× | 0.60× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.51× | 0.35× | 0.40× | 0.18× |
| technical-docs (all workloads) | 22 | 1.00× | 0.80× | 0.51× | 0.36× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.40× | 0.25× | 0.31× | 0.12× |
| commonmark (all workloads) | 17 | 1.00× | 0.67× | 0.40× | 0.26× | 0.33× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.59× | 0.34× | 0.47× | 0.22× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.87× | 0.59× | 0.34× | 0.47× | 0.22× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.73× | 0.26× | 0.56× | 0.26× |
| <512 B (all workloads) | 12 | 1.00× | 0.98× | 0.71× | 0.27× | 0.58× | 0.29× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.52× | 0.33× | 0.41× | 0.20× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.75× | 0.52× | 0.33× | 0.41× | 0.20× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.44× | 0.32× | 0.36× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.72× | 0.44× | 0.32× | 0.36× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.56× | 0.41× | 0.46× | 0.21× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.85× | 0.56× | 0.41× | 0.46× | 0.21× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.56× | 0.28× | 0.36× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.79× | 0.49× | 0.29× | 0.35× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.01× | 0.73× | 0.21× | 0.39× | 0.13× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.56× | 0.29× | 0.40× | 0.17× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.80× | 0.55× | 0.30× | 0.40× | 0.17× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.76× | 0.21× | 0.45× | 0.18× |
| comments (all workloads) | 12 | 1.00× | 1.02× | 0.76× | 0.22× | 0.46× | 0.19× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.32× | 0.24× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.55× | 0.34× | 0.26× | 0.30× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.74× | 0.27× | 0.32× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.04× | 0.74× | 0.27× | 0.32× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.63× | 0.40× | 0.50× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.86× | 0.63× | 0.40× | 0.50× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.61× | 0.42× | 0.58× | 0.28× |
| reference (all workloads) | 4 | 1.00× | 0.90× | 0.61× | 0.42× | 0.58× | 0.28× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.98× | 0.62× | 0.20× | 0.59× | 0.40× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.53× | 0.35× | 0.40× | 0.18× |
| technical-docs (all workloads) | 22 | 1.00× | 0.79× | 0.53× | 0.35× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.42× | 0.25× | 0.30× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.66× | 0.43× | 0.26× | 0.32× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.61× | 0.31× | 0.43× | 0.19× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.87× | 0.61× | 0.31× | 0.44× | 0.19× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.70× | 0.19× | 0.42× | 0.18× |
| <512 B (all workloads) | 12 | 1.00× | 0.96× | 0.70× | 0.20× | 0.44× | 0.20× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.54× | 0.31× | 0.39× | 0.19× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.73× | 0.54× | 0.31× | 0.39× | 0.19× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.45× | 0.32× | 0.36× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.71× | 0.45× | 0.32× | 0.36× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.59× | 0.41× | 0.46× | 0.21× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.85× | 0.59× | 0.41× | 0.46× | 0.21× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.60× | 0.29× | 0.37× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.79× | 0.52× | 0.30× | 0.36× | 0.12× |

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
| comment-ack | 37 | gfm | yes | yes | 0.161 | 0.157 | 0.160 | 0.787 | 0.187 | 0.295 |
| comment-question | 160 | gfm | yes | yes | 0.175 | 0.165 | 0.184 | 0.874 | 0.292 | 0.772 |
| comment-review | 282 | gfm | yes | yes | 0.228 | 0.217 | 0.300 | 1.074 | 0.499 | 1.330 |
| comment-links | 278 | gfm | yes | yes | 0.416 | 0.480 | 0.710 | 1.446 | 0.767 | 1.603 |
| comment-checklist | 287 | gfm | no | no | 0.619 | 0.619 | 0.855 | 1.577 | 0.923 | 1.823 |
| comment-quote | 290 | gfm | yes | yes | 0.298 | 0.280 | 0.354 | 1.168 | 0.589 | 1.452 |
| comment-unicode | 327 | gfm | yes | yes | 0.463 | 0.437 | 0.710 | 1.361 | 0.824 | 1.669 |
| comment-inline-code | 285 | gfm | yes | yes | 0.297 | 0.289 | 0.616 | 1.230 | 0.691 | 1.393 |
| comment-reproduction | 298 | gfm | yes | yes | 0.313 | 0.332 | 0.483 | 1.359 | 0.602 | 1.404 |
| comment-table | 310 | gfm | yes | yes | 1.071 | 1.016 | 1.004 | 2.205 | 1.250 | 2.199 |
| comment-review-long | 957 | gfm | yes | yes | 0.732 | 0.777 | 1.043 | 2.206 | 1.682 | 4.469 |
| comment-incident | 1124 | gfm | no | yes | 1.708 | 1.354 | 1.541 | 2.869 | 2.251 | 5.434 |
| guard-angle-link | 41 | commonmark | no | no | 0.283 | 0.288 | 0.487 | 0.994 | 0.360 | 0.472 |
| legacy-contributing | 9323 | gfm | no | yes | 9.630 | 11.165 | 16.665 | 23.500 | 22.819 | 48.634 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 9.026 | 11.056 | 17.625 | 25.376 | 23.075 | 47.505 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.887 | 2.453 | 4.234 | 5.597 | 4.862 | 8.772 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.559 | 3.074 | 4.495 | 6.624 | 5.479 | 10.694 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.292 | 7.696 | 14.484 | 19.410 | 17.194 | 33.391 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.509 | 3.096 | 5.131 | 7.130 | 5.867 | 10.610 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.691 | 5.223 | 10.001 | 10.847 | 11.984 | 20.501 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.195 | 2.590 | 4.659 | 6.909 | 6.450 | 13.420 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.367 | 4.065 | 7.562 | 9.525 | 9.536 | 18.424 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.742 | 7.865 | 13.361 | 19.749 | 15.847 | 36.943 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.110 | 4.558 | 5.880 | 9.089 | 6.610 | 12.376 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.264 | 1.604 | 2.544 | 3.846 | 3.522 | 7.616 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.556 | 0.808 | 1.075 | 2.005 | 1.337 | 2.566 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 51.126 | 50.210 | 60.872 | 80.687 | 67.581 | 106.359 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 16.354 | 13.597 | 17.365 | 24.389 | 26.471 | 59.991 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.290 | 7.187 | 13.029 | 18.213 | 18.417 | 49.698 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.672 | 6.448 | 7.520 | 12.986 | 11.667 | 31.378 |
| vue-docs-suspense | 8291 | gfm | no | yes | 8.306 | 10.084 | 19.569 | 22.659 | 20.684 | 44.320 |
| vue-docs-slots | 24211 | gfm | no | yes | 21.140 | 36.682 | 66.577 | 83.981 | 64.884 | 147.675 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.992 | 29.111 | 43.499 | 68.859 | 55.545 | 139.820 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.734 | 3.981 | 4.713 | 8.453 | 7.530 | 18.527 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.686 | 9.308 | 12.579 | 19.766 | 18.604 | 43.188 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 59.667 | 73.437 | 81.415 | 118.541 | 96.659 | 191.952 |
| vite-docs-features | 39739 | gfm | no | no | 53.307 | 73.101 | 105.506 | 144.553 | 128.654 | 252.211 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.489 | 5.772 | 5.949 | 10.838 | 9.618 | 26.067 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 46.923 | 59.542 | 85.495 | 124.077 | 98.148 | 195.535 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 25.537 | 25.626 | 66.748 | 102.234 | 52.922 | 149.126 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 51.664 | 72.674 | 122.989 | 170.225 | 134.633 | 304.297 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.962 | 1.622 | 2.180 | 3.725 | 2.781 | 5.159 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.519 | 2.493 | 3.240 | 5.914 | 4.805 | 10.038 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.223 | 52.789 | 76.547 | 114.393 | 105.322 | 286.841 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.607 | 9.125 | 15.753 | 44.264 | 37.130 | 195.211 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.644 | 2.997 | 5.463 | 6.207 | 4.799 | 8.052 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.379 | 14.313 | 40.440 | 28.458 | 24.822 | 43.612 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 54.982 | 93.288 | 161.849 | 185.008 | 171.760 | 381.650 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 12.642 | 12.257 | 19.551 | 46.878 | 40.364 | 208.291 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.318 | 2.347 | 3.693 | 5.294 | 4.140 | 7.265 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.758 | 7.295 | 11.108 | 15.832 | 13.551 | 25.335 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 116.438 | 188.668 | 276.603 | 354.441 | 323.272 | 737.793 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 29.409 | 29.123 | 42.871 | 100.807 | 85.891 | 426.353 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.192 | 4.317 | 9.351 | 9.587 | 7.852 | 15.390 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.961 | 5.667 | 11.020 | 12.970 | 11.055 | 23.526 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 63.629 | 111.805 | 169.401 | 217.324 | 203.597 | 444.716 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 16.262 | 15.739 | 23.680 | 57.905 | 51.161 | 241.836 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.068 | 0.068 | 0.086 | 0.750 | 0.155 | 0.296 |
| comment-question | 160 | gfm | yes | yes | 0.085 | 0.079 | 0.105 | 0.846 | 0.264 | 0.772 |
| comment-review | 282 | gfm | yes | yes | 0.139 | 0.130 | 0.194 | 1.016 | 0.446 | 1.327 |
| comment-links | 278 | gfm | yes | yes | 0.312 | 0.385 | 0.507 | 1.358 | 0.681 | 1.607 |
| comment-checklist | 287 | gfm | no | no | 0.514 | 0.524 | 0.687 | 1.457 | 0.840 | 1.834 |
| comment-quote | 290 | gfm | yes | yes | 0.204 | 0.190 | 0.240 | 1.088 | 0.499 | 1.456 |
| comment-unicode | 327 | gfm | yes | yes | 0.362 | 0.339 | 0.457 | 1.298 | 0.749 | 1.682 |
| comment-inline-code | 285 | gfm | yes | yes | 0.205 | 0.201 | 0.466 | 1.142 | 0.613 | 1.395 |
| comment-reproduction | 298 | gfm | yes | yes | 0.221 | 0.240 | 0.365 | 1.282 | 0.532 | 1.400 |
| comment-table | 310 | gfm | yes | yes | 0.966 | 0.917 | 0.861 | 2.120 | 1.159 | 2.191 |
| comment-review-long | 957 | gfm | yes | yes | 0.625 | 0.667 | 0.830 | 2.116 | 1.586 | 4.470 |
| comment-incident | 1124 | gfm | no | yes | 1.600 | 1.221 | 1.329 | 2.726 | 2.113 | 5.426 |
| guard-angle-link | 41 | commonmark | no | no | 0.187 | 0.190 | 0.304 | 0.942 | 0.319 | 0.471 |
| legacy-contributing | 9323 | gfm | no | yes | 9.501 | 11.059 | 15.836 | 22.885 | 22.457 | 48.148 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.901 | 10.940 | 16.468 | 24.912 | 22.557 | 47.477 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.769 | 2.346 | 3.929 | 5.429 | 4.719 | 8.867 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.434 | 2.966 | 4.157 | 6.413 | 5.319 | 10.746 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.181 | 7.568 | 13.876 | 19.033 | 16.474 | 32.950 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.404 | 3.001 | 4.708 | 6.947 | 5.694 | 10.645 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.602 | 5.127 | 9.480 | 10.649 | 11.614 | 20.695 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.073 | 2.483 | 4.235 | 6.694 | 6.217 | 13.527 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.279 | 3.977 | 7.139 | 9.248 | 9.269 | 18.456 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.606 | 7.666 | 12.505 | 19.231 | 15.287 | 36.900 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.975 | 4.411 | 5.411 | 8.822 | 6.310 | 12.347 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.150 | 1.498 | 2.227 | 3.670 | 3.378 | 7.635 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.443 | 0.697 | 0.861 | 1.886 | 1.218 | 2.567 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 51.407 | 50.341 | 59.734 | 80.312 | 67.143 | 107.248 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 16.164 | 13.433 | 16.090 | 23.794 | 25.714 | 59.911 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.176 | 6.996 | 12.172 | 17.887 | 18.086 | 49.718 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.531 | 6.332 | 7.067 | 12.686 | 11.374 | 31.409 |
| vue-docs-suspense | 8291 | gfm | no | yes | 8.153 | 9.817 | 18.480 | 21.969 | 19.774 | 44.068 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.930 | 36.157 | 63.504 | 82.092 | 62.482 | 146.709 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.808 | 28.956 | 41.817 | 67.143 | 54.688 | 139.201 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.613 | 3.848 | 4.285 | 8.209 | 7.258 | 18.408 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.567 | 9.203 | 11.979 | 19.283 | 18.033 | 42.757 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 59.778 | 72.698 | 80.199 | 115.851 | 94.578 | 191.183 |
| vite-docs-features | 39739 | gfm | no | no | 53.391 | 71.488 | 103.877 | 142.434 | 126.139 | 250.598 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.362 | 5.603 | 5.450 | 10.597 | 9.320 | 26.231 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 46.364 | 60.262 | 82.538 | 121.827 | 97.512 | 197.249 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 25.478 | 25.511 | 64.615 | 101.319 | 51.244 | 149.398 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 51.523 | 70.315 | 120.172 | 167.537 | 132.656 | 304.356 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.833 | 1.502 | 1.817 | 3.598 | 2.664 | 5.142 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.403 | 2.381 | 2.843 | 5.752 | 4.647 | 9.983 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 31.624 | 51.521 | 71.607 | 113.073 | 104.054 | 286.214 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.492 | 8.967 | 13.535 | 41.919 | 35.079 | 193.119 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.529 | 2.883 | 5.125 | 6.131 | 4.728 | 8.117 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.220 | 14.107 | 38.951 | 28.019 | 24.461 | 43.456 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.678 | 93.655 | 156.946 | 182.260 | 170.421 | 381.264 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 12.643 | 12.234 | 17.165 | 45.394 | 38.780 | 208.022 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.201 | 2.230 | 3.328 | 5.136 | 4.033 | 7.272 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.624 | 7.120 | 10.138 | 15.346 | 13.210 | 25.223 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 117.625 | 188.363 | 269.468 | 350.874 | 316.856 | 737.083 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 29.214 | 28.630 | 38.376 | 96.983 | 82.570 | 423.769 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.069 | 4.207 | 8.749 | 9.299 | 7.630 | 15.337 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.848 | 5.560 | 10.383 | 12.643 | 10.666 | 23.383 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 62.902 | 110.745 | 162.375 | 212.003 | 197.740 | 443.161 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 15.971 | 15.458 | 20.847 | 54.700 | 47.928 | 240.406 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 457.67 | 667.04 | 1047.65 | 1301.01 | 1160.99 | 3186.06 |
| rotating-commonmark | reuse | 17 | 447.61 | 639.92 | 989.98 | 1267.05 | 1121.71 | 3170.90 |
| rotating-gfm | fresh | 40 | 711.94 | 874.87 | 1296.84 | 1471.33 | 1211.12 | 2381.22 |
| rotating-gfm | reuse | 40 | 694.08 | 851.34 | 1227.91 | 1431.53 | 1172.70 | 2377.19 |
