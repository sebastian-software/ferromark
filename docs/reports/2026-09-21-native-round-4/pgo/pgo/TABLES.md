# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.74× | 0.60× | 0.18× | 0.44× | 0.20× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.47× | 0.24× | 0.39× | 0.17× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.65× | 0.44× | 0.23× | 0.37× | 0.16× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.63× | 0.20× | 0.45× | 0.20× |
| comments (all workloads) | 6 | 1.00× | 0.75× | 0.63× | 0.20× | 0.45× | 0.20× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.19× | 0.27× | 0.15× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.50× | 0.28× | 0.20× | 0.27× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.67× | 0.35× | 0.62× | 0.32× |
| readme (all workloads) | 1 | 1.00× | 0.80× | 0.67× | 0.35× | 0.62× | 0.32× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.50× | 0.27× | 0.51× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.78× | 0.50× | 0.27× | 0.51× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.49× | 0.27× | 0.37× | 0.16× |
| technical-docs (all workloads) | 11 | 1.00× | 0.70× | 0.49× | 0.27× | 0.37× | 0.16× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.25× | 0.19× | 0.27× | 0.15× |
| commonmark (all workloads) | 8 | 1.00× | 0.50× | 0.28× | 0.20× | 0.27× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.53× | 0.25× | 0.41× | 0.18× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.73× | 0.53× | 0.25× | 0.41× | 0.18× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.56× | 0.18× | 0.42× | 0.19× |
| <512 B (all workloads) | 6 | 1.00× | 0.70× | 0.56× | 0.18× | 0.42× | 0.19× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.41× | 0.24× | 0.37× | 0.19× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.62× | 0.41× | 0.24× | 0.37× | 0.19× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.43× | 0.26× | 0.34× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.67× | 0.43× | 0.26× | 0.34× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.54× | 0.31× | 0.43× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.54× | 0.31× | 0.43× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.35× | 0.19× | 0.44× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.57× | 0.33× | 0.22× | 0.30× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (5 cases) | 5 | 1.00× | 0.91× | 0.66× | 0.15× | 0.40× | 0.16× |
| Configurable-five HTML agreement (24 cases) | 24 | 1.00× | — | 0.50× | 0.23× | 0.38× | 0.16× |
| All 28 native workloads (output differences included) | 28 | 1.00× | 0.68× | 0.47× | 0.23× | 0.36× | 0.15× |
| comments (five-engine agreement) | 6 | 1.00× | — | 0.68× | 0.17× | 0.41× | 0.16× |
| comments (all workloads) | 6 | 1.00× | 0.90× | 0.68× | 0.17× | 0.41× | 0.16× |
| encyclopedia (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.18× | 0.27× | 0.14× |
| encyclopedia (all workloads) | 8 | 1.00× | 0.50× | 0.29× | 0.20× | 0.27× | 0.13× |
| readme (five-engine agreement) | 1 | 1.00× | — | 0.73× | 0.35× | 0.63× | 0.31× |
| readme (all workloads) | 1 | 1.00× | 0.81× | 0.73× | 0.35× | 0.63× | 0.31× |
| reference (five-engine agreement) | 2 | 1.00× | — | 0.53× | 0.28× | 0.52× | 0.21× |
| reference (all workloads) | 2 | 1.00× | 0.79× | 0.53× | 0.28× | 0.52× | 0.21× |
| technical-docs (five-engine agreement) | 11 | 1.00× | — | 0.51× | 0.27× | 0.37× | 0.15× |
| technical-docs (all workloads) | 11 | 1.00× | 0.70× | 0.51× | 0.27× | 0.37× | 0.15× |
| commonmark (five-engine agreement) | 4 | 1.00× | — | 0.26× | 0.18× | 0.27× | 0.14× |
| commonmark (all workloads) | 8 | 1.00× | 0.50× | 0.29× | 0.20× | 0.27× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 20 | 1.00× | — | 0.57× | 0.24× | 0.41× | 0.17× |
| gfm (shared subset) (all workloads) | 20 | 1.00× | 0.77× | 0.57× | 0.24× | 0.41× | 0.17× |
| <512 B (five-engine agreement) | 6 | 1.00× | — | 0.61× | 0.16× | 0.39× | 0.16× |
| <512 B (all workloads) | 6 | 1.00× | 0.83× | 0.61× | 0.16× | 0.39× | 0.16× |
| 512 B–2 KiB (five-engine agreement) | 6 | 1.00× | — | 0.44× | 0.23× | 0.37× | 0.18× |
| 512 B–2 KiB (all workloads) | 6 | 1.00× | 0.63× | 0.44× | 0.23× | 0.37× | 0.18× |
| 2–8 KiB (five-engine agreement) | 7 | 1.00× | — | 0.46× | 0.26× | 0.35× | 0.15× |
| 2–8 KiB (all workloads) | 7 | 1.00× | 0.67× | 0.46× | 0.26× | 0.35× | 0.15× |
| 8–32 KiB (five-engine agreement) | 4 | 1.00× | — | 0.57× | 0.32× | 0.44× | 0.18× |
| 8–32 KiB (all workloads) | 4 | 1.00× | 0.73× | 0.57× | 0.32× | 0.44× | 0.18× |
| 32–128 KiB (five-engine agreement) | 1 | 1.00× | — | 0.37× | 0.20× | 0.46× | 0.15× |
| 32–128 KiB (all workloads) | 5 | 1.00× | 0.57× | 0.34× | 0.22× | 0.30× | 0.12× |

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
| comment-question | 160 | gfm | yes | yes | 0.095 | 0.151 | 0.157 | 0.861 | 0.249 | 0.648 |
| comment-links | 278 | gfm | yes | yes | 0.263 | 0.405 | 0.548 | 1.402 | 0.631 | 1.369 |
| comment-quote | 290 | gfm | yes | yes | 0.185 | 0.251 | 0.274 | 1.139 | 0.474 | 1.191 |
| comment-inline-code | 285 | gfm | yes | yes | 0.190 | 0.251 | 0.467 | 1.189 | 0.571 | 1.190 |
| comment-table | 310 | gfm | yes | yes | 0.747 | 0.786 | 0.777 | 2.126 | 0.904 | 1.706 |
| comment-incident | 1124 | gfm | no | yes | 0.871 | 1.033 | 1.122 | 2.807 | 1.795 | 4.429 |
| legacy-contributing | 9323 | gfm | no | yes | 7.024 | 8.666 | 12.515 | 22.927 | 17.581 | 40.076 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.342 | 1.889 | 3.002 | 5.477 | 3.616 | 7.173 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.978 | 6.518 | 11.129 | 18.815 | 12.897 | 26.236 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.486 | 4.193 | 7.619 | 10.631 | 9.232 | 17.284 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.514 | 3.215 | 5.709 | 9.210 | 7.304 | 15.642 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.115 | 3.908 | 4.661 | 8.900 | 5.048 | 9.805 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.352 | 0.670 | 0.849 | 1.927 | 1.029 | 2.155 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.961 | 10.939 | 13.475 | 23.680 | 20.410 | 51.515 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.119 | 5.081 | 5.731 | 12.707 | 9.382 | 26.263 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.191 | 30.553 | 56.009 | 82.347 | 52.955 | 147.418 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.936 | 3.208 | 3.898 | 8.263 | 6.193 | 15.634 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 43.183 | 58.783 | 60.070 | 115.780 | 74.682 | 149.789 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.211 | 4.619 | 4.843 | 10.512 | 7.809 | 21.712 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.493 | 23.337 | 55.873 | 100.197 | 44.188 | 130.959 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.678 | 1.341 | 1.956 | 3.628 | 2.377 | 4.433 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 24.073 | 43.455 | 65.235 | 112.820 | 92.817 | 250.904 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.202 | 2.510 | 5.413 | 6.107 | 4.130 | 7.092 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 40.169 | 78.553 | 146.568 | 181.525 | 149.064 | 340.676 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.950 | 1.942 | 3.456 | 5.124 | 3.490 | 6.431 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 83.671 | 162.368 | 247.672 | 345.993 | 286.796 | 656.655 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.704 | 3.710 | 9.819 | 9.283 | 6.878 | 13.824 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 45.269 | 92.754 | 146.776 | 211.794 | 178.351 | 394.892 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-question | 160 | gfm | yes | yes | 0.063 | 0.066 | 0.100 | 0.831 | 0.224 | 0.650 |
| comment-links | 278 | gfm | yes | yes | 0.225 | 0.316 | 0.415 | 1.333 | 0.559 | 1.368 |
| comment-quote | 290 | gfm | yes | yes | 0.149 | 0.158 | 0.195 | 1.056 | 0.412 | 1.190 |
| comment-inline-code | 285 | gfm | yes | yes | 0.155 | 0.163 | 0.359 | 1.114 | 0.505 | 1.190 |
| comment-table | 310 | gfm | yes | yes | 0.710 | 0.697 | 0.662 | 2.063 | 0.860 | 1.717 |
| comment-incident | 1124 | gfm | no | yes | 0.824 | 0.936 | 0.988 | 2.657 | 1.698 | 4.447 |
| legacy-contributing | 9323 | gfm | no | yes | 6.992 | 8.533 | 11.844 | 22.288 | 16.998 | 40.355 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.294 | 1.779 | 2.780 | 5.286 | 3.471 | 7.146 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 4.965 | 6.397 | 10.686 | 18.567 | 12.483 | 26.110 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 3.434 | 4.074 | 7.173 | 10.335 | 8.975 | 17.352 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 2.459 | 3.106 | 5.370 | 8.987 | 7.080 | 15.574 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.075 | 3.788 | 4.230 | 8.736 | 4.873 | 9.876 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.304 | 0.565 | 0.701 | 1.850 | 0.927 | 2.154 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 9.809 | 10.718 | 12.327 | 23.240 | 19.564 | 51.201 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 3.050 | 4.972 | 5.315 | 12.371 | 9.076 | 26.092 |
| vue-docs-slots | 24211 | gfm | no | yes | 16.122 | 30.458 | 53.701 | 80.396 | 51.483 | 146.860 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 1.879 | 3.087 | 3.525 | 8.016 | 5.972 | 15.607 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 43.704 | 59.929 | 57.391 | 114.294 | 73.604 | 150.085 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 3.187 | 4.496 | 4.317 | 10.294 | 7.564 | 21.843 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 19.613 | 23.146 | 53.438 | 97.628 | 42.731 | 131.351 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.633 | 1.227 | 1.668 | 3.544 | 2.275 | 4.435 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 23.968 | 43.041 | 60.680 | 110.717 | 91.523 | 251.939 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.162 | 2.406 | 5.106 | 5.978 | 3.993 | 7.090 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 40.202 | 78.282 | 142.670 | 178.907 | 147.621 | 339.201 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 0.907 | 1.839 | 3.166 | 5.008 | 3.382 | 6.457 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 84.571 | 161.680 | 240.763 | 342.624 | 281.650 | 655.564 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.669 | 3.608 | 9.411 | 9.108 | 6.678 | 13.798 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 44.177 | 92.607 | 142.135 | 209.541 | 175.196 | 397.026 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 8 | 267.17 | 467.02 | 749.92 | 938.19 | 766.28 | 1738.24 |
| rotating-commonmark | reuse | 8 | 263.85 | 453.72 | 725.72 | 921.70 | 755.16 | 1741.03 |
| rotating-gfm | fresh | 20 | 181.17 | 259.19 | 386.51 | 529.62 | 357.57 | 810.82 |
| rotating-gfm | reuse | 20 | 172.24 | 249.52 | 371.66 | 519.65 | 347.53 | 813.53 |
