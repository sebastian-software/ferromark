# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.71× | 0.57× | 0.17× | 0.43× | 0.19× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.46× | 0.23× | 0.38× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.64× | 0.43× | 0.23× | 0.36× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.60× | 0.19× | 0.43× | 0.19× |
| comments (all workloads) | 6 | 1.00× | 0.73× | 0.60× | 0.19× | 0.43× | 0.19× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.24× | 0.18× | 0.27× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.49× | 0.27× | 0.20× | 0.26× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.63× | 0.33× | 0.57× | 0.30× |
| readme (all workloads) | 1 | 1.00× | 0.74× | 0.63× | 0.33× | 0.57× | 0.30× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.48× | 0.26× | 0.49× | 0.20× |
| reference (all workloads) | 2 | 1.00× | 0.77× | 0.48× | 0.26× | 0.49× | 0.20× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.48× | 0.26× | 0.36× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.69× | 0.48× | 0.26× | 0.36× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.24× | 0.18× | 0.27× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.49× | 0.27× | 0.20× | 0.26× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.52× | 0.24× | 0.40× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.71× | 0.52× | 0.24× | 0.40× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.54× | 0.17× | 0.41× | 0.18× |
| <512 B (all workloads) | 6 | 1.00× | 0.67× | 0.54× | 0.17× | 0.41× | 0.18× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.41× | 0.23× | 0.36× | 0.18× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.61× | 0.41× | 0.23× | 0.36× | 0.18× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.43× | 0.25× | 0.33× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.66× | 0.43× | 0.25× | 0.33× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.53× | 0.31× | 0.42× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.53× | 0.31× | 0.42× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.34× | 0.19× | 0.43× | 0.14× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.56× | 0.31× | 0.21× | 0.29× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.88× | 0.63× | 0.15× | 0.39× | 0.16× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.49× | 0.22× | 0.37× | 0.16× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.67× | 0.46× | 0.22× | 0.35× | 0.15× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.66× | 0.17× | 0.40× | 0.16× |
| comments (all workloads) | 6 | 1.00× | 0.88× | 0.66× | 0.17× | 0.40× | 0.16× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.18× | 0.27× | 0.14× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.49× | 0.28× | 0.20× | 0.26× | 0.12× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.67× | 0.33× | 0.58× | 0.29× |
| readme (all workloads) | 1 | 1.00× | 0.75× | 0.67× | 0.33× | 0.58× | 0.29× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.50× | 0.26× | 0.50× | 0.20× |
| reference (all workloads) | 2 | 1.00× | 0.78× | 0.50× | 0.26× | 0.50× | 0.20× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.26× | 0.36× | 0.15× |
| technical-docs (all workloads) | 11 | 1.00× | 0.70× | 0.51× | 0.26× | 0.36× | 0.15× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.18× | 0.27× | 0.14× |
| commonmark (all workloads) | 8 | 1.00× | 0.49× | 0.28× | 0.20× | 0.26× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.56× | 0.23× | 0.40× | 0.16× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.76× | 0.56× | 0.23× | 0.40× | 0.16× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.59× | 0.15× | 0.38× | 0.15× |
| <512 B (all workloads) | 6 | 1.00× | 0.81× | 0.59× | 0.15× | 0.38× | 0.15× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.43× | 0.23× | 0.36× | 0.18× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.62× | 0.43× | 0.23× | 0.36× | 0.18× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.45× | 0.25× | 0.34× | 0.14× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.67× | 0.45× | 0.25× | 0.34× | 0.14× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.56× | 0.31× | 0.43× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.74× | 0.56× | 0.31× | 0.43× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.35× | 0.19× | 0.44× | 0.14× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.56× | 0.32× | 0.21× | 0.29× | 0.12× |

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
| comment-question | 160 | gfm | yes | yes | 0.095 | 0.155 | 0.160 | 0.879 | 0.256 | 0.667 |
| comment-links | 278 | gfm | yes | yes | 0.258 | 0.421 | 0.570 | 1.441 | 0.647 | 1.399 |
| comment-quote | 290 | gfm | yes | yes | 0.186 | 0.258 | 0.284 | 1.161 | 0.491 | 1.217 |
| comment-inline-code | 285 | gfm | yes | yes | 0.195 | 0.255 | 0.479 | 1.217 | 0.579 | 1.208 |
| comment-table | 310 | gfm | yes | yes | 0.690 | 0.812 | 0.789 | 2.178 | 0.935 | 1.768 |
| comment-incident | 1124 | gfm | no | yes | 0.878 | 1.060 | 1.151 | 2.851 | 1.826 | 4.551 |
| legacy-contributing | 9323 | gfm | no | yes | 7.127 | 8.882 | 12.897 | 23.666 | 18.341 | 41.518 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.409 | 1.934 | 3.091 | 5.635 | 3.700 | 7.433 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.135 | 6.658 | 11.431 | 19.587 | 13.496 | 27.122 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.586 | 4.256 | 7.789 | 10.970 | 9.517 | 18.032 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.585 | 3.312 | 5.866 | 9.497 | 7.512 | 16.051 |
| legacy-docs-readme | 1825 | gfm | no | yes | 2.993 | 4.040 | 4.765 | 9.174 | 5.217 | 10.125 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.342 | 0.680 | 0.856 | 1.952 | 1.037 | 2.171 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.216 | 11.233 | 13.827 | 24.761 | 21.086 | 53.138 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.028 | 5.112 | 5.841 | 12.851 | 9.584 | 27.089 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.424 | 31.198 | 58.305 | 84.097 | 54.178 | 148.786 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.964 | 3.244 | 3.996 | 8.431 | 6.386 | 15.942 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 42.578 | 58.863 | 61.788 | 119.026 | 77.183 | 152.419 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.223 | 4.747 | 4.883 | 10.872 | 8.125 | 22.677 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.415 | 23.873 | 57.213 | 103.956 | 45.190 | 135.873 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.688 | 1.403 | 2.008 | 3.722 | 2.435 | 4.542 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.486 | 43.861 | 67.277 | 117.688 | 97.265 | 259.605 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.213 | 2.573 | 5.568 | 6.323 | 4.269 | 7.276 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 39.864 | 78.616 | 152.988 | 188.609 | 154.449 | 349.773 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.969 | 2.014 | 3.532 | 5.289 | 3.564 | 6.653 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 84.634 | 168.073 | 258.909 | 356.012 | 296.135 | 670.555 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.719 | 3.784 | 10.032 | 9.587 | 7.020 | 14.248 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 44.170 | 94.517 | 153.639 | 221.085 | 187.268 | 406.428 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.065 | 0.069 | 0.105 | 0.852 | 0.229 | 0.672 |
| comment-links | 278 | gfm | yes | yes | 0.222 | 0.322 | 0.431 | 1.362 | 0.574 | 1.411 |
| comment-quote | 290 | gfm | yes | yes | 0.151 | 0.162 | 0.202 | 1.083 | 0.428 | 1.218 |
| comment-inline-code | 285 | gfm | yes | yes | 0.163 | 0.166 | 0.371 | 1.139 | 0.515 | 1.202 |
| comment-table | 310 | gfm | yes | yes | 0.649 | 0.718 | 0.682 | 2.117 | 0.883 | 1.768 |
| comment-incident | 1124 | gfm | no | yes | 0.832 | 0.957 | 1.016 | 2.739 | 1.742 | 4.564 |
| legacy-contributing | 9323 | gfm | no | yes | 7.127 | 8.722 | 12.196 | 23.251 | 17.947 | 41.769 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.357 | 1.829 | 2.843 | 5.463 | 3.568 | 7.404 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 5.076 | 6.527 | 10.854 | 19.210 | 13.138 | 26.844 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.535 | 4.165 | 7.454 | 10.758 | 9.306 | 17.867 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.507 | 3.185 | 5.458 | 9.198 | 7.298 | 16.023 |
| legacy-docs-readme | 1825 | gfm | no | yes | 2.937 | 3.917 | 4.374 | 9.025 | 5.048 | 10.184 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.305 | 0.587 | 0.720 | 1.886 | 0.953 | 2.209 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 10.107 | 11.036 | 12.665 | 24.162 | 20.543 | 53.191 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.026 | 5.046 | 5.483 | 12.720 | 9.410 | 27.026 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.424 | 31.241 | 55.599 | 83.329 | 53.394 | 148.487 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.912 | 3.100 | 3.585 | 8.184 | 6.094 | 15.813 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 43.609 | 58.504 | 60.469 | 118.246 | 77.011 | 155.146 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.153 | 4.583 | 4.429 | 10.686 | 7.827 | 22.714 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.162 | 23.642 | 55.029 | 101.241 | 43.627 | 136.152 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.642 | 1.295 | 1.729 | 3.635 | 2.329 | 4.552 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.149 | 43.179 | 63.227 | 114.823 | 95.643 | 258.451 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.168 | 2.473 | 5.232 | 6.162 | 4.109 | 7.255 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 39.695 | 79.975 | 150.022 | 185.402 | 154.331 | 348.604 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.931 | 1.905 | 3.301 | 5.186 | 3.486 | 6.621 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 84.532 | 167.005 | 249.901 | 353.546 | 290.517 | 669.128 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.671 | 3.676 | 9.583 | 9.370 | 6.817 | 14.101 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 43.777 | 93.914 | 149.580 | 216.953 | 181.081 | 406.452 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 273.30 | 484.44 | 768.80 | 965.34 | 790.43 | 1772.85 |
| rotating-commonmark | reuse | 8 | 267.66 | 468.59 | 743.08 | 951.04 | 776.35 | 1781.96 |
| rotating-gfm | fresh | 20 | 182.87 | 267.92 | 401.78 | 546.78 | 369.56 | 831.73 |
| rotating-gfm | reuse | 20 | 176.57 | 260.01 | 382.15 | 536.53 | 355.15 | 842.41 |
