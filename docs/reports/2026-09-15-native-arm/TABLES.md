# Native benchmark tables

Speed relative to Ferromark v2 in the same lifecycle: **higher is faster**; v2 = 1.00×.
Geometric mean of per-document time ratios; median of three process-round medians per engine/document.
All-workload aggregates include different HTML and are diagnostic. Strict agreement excludes heading-ID differences.
The configurable-five subset requires agreement among v2, v1, md4c, pulldown-cmark, and Bun; OX has no score in it.
Bun uses its fresh owned-output API in both lifecycle schedules.

## fresh

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 1.00× | 0.71× | 0.26× | 0.46× | 0.17× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.53× | 0.31× | 0.42× | 0.18× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.80× | 0.52× | 0.31× | 0.42× | 0.18× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.74× | 0.28× | 0.55× | 0.25× |
| comments (all workloads) | 12 | 1.00× | 0.99× | 0.74× | 0.28× | 0.56× | 0.25× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.30× | 0.24× | 0.29× | 0.15× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.56× | 0.33× | 0.26× | 0.31× | 0.15× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.65× | 0.26× | 0.30× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.01× | 0.65× | 0.26× | 0.30× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.59× | 0.39× | 0.50× | 0.25× |
| readme (all workloads) | 2 | 1.00× | 0.85× | 0.59× | 0.39× | 0.50× | 0.25× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.61× | 0.42× | 0.59× | 0.28× |
| reference (all workloads) | 4 | 1.00× | 0.90× | 0.61× | 0.42× | 0.59× | 0.28× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.97× | 0.57× | 0.28× | 0.77× | 0.58× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.51× | 0.35× | 0.40× | 0.18× |
| technical-docs (all workloads) | 22 | 1.00× | 0.79× | 0.51× | 0.35× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.39× | 0.24× | 0.30× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.66× | 0.40× | 0.26× | 0.32× | 0.13× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.58× | 0.33× | 0.46× | 0.21× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.86× | 0.58× | 0.34× | 0.47× | 0.21× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.71× | 0.26× | 0.55× | 0.26× |
| <512 B (all workloads) | 12 | 1.00× | 0.97× | 0.70× | 0.27× | 0.57× | 0.28× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.49× | 0.31× | 0.39× | 0.19× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.71× | 0.49× | 0.31× | 0.39× | 0.19× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.43× | 0.31× | 0.36× | 0.17× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.70× | 0.43× | 0.31× | 0.36× | 0.17× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.55× | 0.39× | 0.45× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.82× | 0.55× | 0.39× | 0.45× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.57× | 0.29× | 0.37× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.82× | 0.50× | 0.30× | 0.36× | 0.12× |

## reuse

| Group | N | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| All-six HTML agreement (14 cases) | 14 | 1.00× | 0.98× | 0.71× | 0.21× | 0.38× | 0.12× |
| Configurable-five HTML agreement (50 cases) | 50 | 1.00× | — | 0.54× | 0.28× | 0.39× | 0.16× |
| All 57 native workloads (output differences included) | 57 | 1.00× | 0.78× | 0.53× | 0.29× | 0.39× | 0.16× |
| comments (five-engine agreement) | 11 | 1.00× | — | 0.72× | 0.20× | 0.42× | 0.17× |
| comments (all workloads) | 12 | 1.00× | 0.96× | 0.72× | 0.21× | 0.43× | 0.18× |
| encyclopedia (five-engine agreement) | 8 | 1.00× | — | 0.30× | 0.23× | 0.28× | 0.14× |
| encyclopedia (all workloads) | 12 | 1.00× | 0.54× | 0.33× | 0.25× | 0.30× | 0.14× |
| plain-prose (five-engine agreement) | 4 | 1.00× | — | 0.72× | 0.27× | 0.31× | 0.06× |
| plain-prose (all workloads) | 4 | 1.00× | 1.02× | 0.72× | 0.27× | 0.31× | 0.06× |
| readme (five-engine agreement) | 2 | 1.00× | — | 0.62× | 0.39× | 0.50× | 0.24× |
| readme (all workloads) | 2 | 1.00× | 0.85× | 0.62× | 0.39× | 0.50× | 0.24× |
| reference (five-engine agreement) | 4 | 1.00× | — | 0.62× | 0.42× | 0.60× | 0.28× |
| reference (all workloads) | 4 | 1.00× | 0.91× | 0.62× | 0.42× | 0.60× | 0.28× |
| syntax-guard (all workloads) | 1 | 1.00× | 0.94× | 0.59× | 0.19× | 0.57× | 0.38× |
| technical-docs (five-engine agreement) | 21 | 1.00× | — | 0.52× | 0.34× | 0.40× | 0.17× |
| technical-docs (all workloads) | 22 | 1.00× | 0.78× | 0.53× | 0.35× | 0.40× | 0.18× |
| commonmark (five-engine agreement) | 12 | 1.00× | — | 0.41× | 0.24× | 0.29× | 0.11× |
| commonmark (all workloads) | 17 | 1.00× | 0.65× | 0.41× | 0.25× | 0.31× | 0.12× |
| gfm (shared subset) (five-engine agreement) | 38 | 1.00× | — | 0.59× | 0.30× | 0.42× | 0.18× |
| gfm (shared subset) (all workloads) | 40 | 1.00× | 0.85× | 0.59× | 0.30× | 0.43× | 0.19× |
| <512 B (five-engine agreement) | 10 | 1.00× | — | 0.68× | 0.18× | 0.41× | 0.17× |
| <512 B (all workloads) | 12 | 1.00× | 0.93× | 0.68× | 0.19× | 0.43× | 0.19× |
| 512 B–2 KiB (five-engine agreement) | 9 | 1.00× | — | 0.51× | 0.29× | 0.37× | 0.17× |
| 512 B–2 KiB (all workloads) | 9 | 1.00× | 0.69× | 0.51× | 0.29× | 0.37× | 0.17× |
| 2–8 KiB (five-engine agreement) | 15 | 1.00× | — | 0.44× | 0.31× | 0.36× | 0.16× |
| 2–8 KiB (all workloads) | 15 | 1.00× | 0.69× | 0.44× | 0.31× | 0.36× | 0.16× |
| 8–32 KiB (five-engine agreement) | 9 | 1.00× | — | 0.56× | 0.39× | 0.45× | 0.20× |
| 8–32 KiB (all workloads) | 9 | 1.00× | 0.82× | 0.56× | 0.39× | 0.45× | 0.20× |
| 32–128 KiB (five-engine agreement) | 7 | 1.00× | — | 0.61× | 0.29× | 0.38× | 0.10× |
| 32–128 KiB (all workloads) | 12 | 1.00× | 0.82× | 0.53× | 0.30× | 0.37× | 0.12× |

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
| comment-ack | 37 | gfm | yes | yes | 0.152 | 0.150 | 0.155 | 0.757 | 0.181 | 0.282 |
| comment-question | 160 | gfm | yes | yes | 0.173 | 0.165 | 0.184 | 0.877 | 0.294 | 0.772 |
| comment-review | 282 | gfm | yes | yes | 0.223 | 0.216 | 0.302 | 1.066 | 0.497 | 1.322 |
| comment-links | 278 | gfm | yes | yes | 0.397 | 0.462 | 0.684 | 1.404 | 0.733 | 1.583 |
| comment-checklist | 287 | gfm | no | no | 0.591 | 0.611 | 0.838 | 1.564 | 0.915 | 1.802 |
| comment-quote | 290 | gfm | yes | yes | 0.286 | 0.279 | 0.356 | 1.162 | 0.586 | 1.432 |
| comment-unicode | 327 | gfm | yes | yes | 0.460 | 0.434 | 0.714 | 1.359 | 0.807 | 1.681 |
| comment-inline-code | 285 | gfm | yes | yes | 0.288 | 0.282 | 0.596 | 1.203 | 0.686 | 1.386 |
| comment-reproduction | 298 | gfm | yes | yes | 0.305 | 0.325 | 0.475 | 1.340 | 0.600 | 1.399 |
| comment-table | 310 | gfm | yes | yes | 1.042 | 1.001 | 0.992 | 2.195 | 1.224 | 2.181 |
| comment-review-long | 957 | gfm | yes | yes | 0.711 | 0.768 | 1.027 | 2.184 | 1.661 | 4.435 |
| comment-incident | 1124 | gfm | no | yes | 1.238 | 1.318 | 1.526 | 2.840 | 2.209 | 5.366 |
| guard-angle-link | 41 | commonmark | no | no | 0.271 | 0.280 | 0.480 | 0.982 | 0.353 | 0.471 |
| legacy-contributing | 9323 | gfm | no | yes | 9.371 | 10.949 | 16.394 | 23.266 | 22.156 | 47.814 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.774 | 10.858 | 17.210 | 25.134 | 22.060 | 47.139 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.881 | 2.444 | 4.177 | 5.584 | 4.768 | 8.773 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.434 | 2.971 | 4.293 | 6.349 | 5.191 | 10.340 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.149 | 7.630 | 14.158 | 19.134 | 16.535 | 33.176 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.427 | 3.064 | 5.047 | 7.046 | 5.728 | 10.491 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.626 | 5.196 | 9.916 | 10.879 | 11.686 | 20.395 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.090 | 2.509 | 4.506 | 6.742 | 6.162 | 13.313 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.229 | 3.928 | 7.303 | 9.246 | 9.144 | 18.188 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.419 | 7.513 | 13.027 | 19.606 | 15.232 | 35.895 |
| legacy-docs-readme | 1825 | gfm | no | yes | 4.017 | 4.521 | 5.841 | 9.149 | 6.454 | 12.260 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.235 | 1.579 | 2.510 | 3.796 | 3.426 | 7.478 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.553 | 0.799 | 1.070 | 1.986 | 1.303 | 2.575 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 48.859 | 47.942 | 58.349 | 78.352 | 61.839 | 101.211 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 13.025 | 13.380 | 17.029 | 24.259 | 25.700 | 58.742 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 7.187 | 7.071 | 12.879 | 18.128 | 18.100 | 49.440 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.516 | 6.361 | 7.380 | 12.946 | 11.473 | 31.068 |
| vue-docs-suspense | 8291 | gfm | no | yes | 8.017 | 9.900 | 19.236 | 22.463 | 20.034 | 43.890 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.615 | 35.643 | 68.187 | 82.526 | 62.445 | 145.046 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.508 | 28.484 | 41.888 | 67.705 | 53.928 | 138.199 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.620 | 3.929 | 4.644 | 8.393 | 7.324 | 18.316 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.414 | 9.191 | 12.366 | 19.529 | 18.105 | 42.361 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 57.798 | 70.605 | 78.820 | 116.076 | 92.366 | 188.945 |
| vite-docs-features | 39739 | gfm | no | no | 52.718 | 69.120 | 103.561 | 142.154 | 120.888 | 246.679 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.357 | 5.832 | 5.871 | 10.752 | 9.464 | 26.307 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 46.262 | 57.288 | 80.193 | 118.790 | 90.122 | 187.468 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 24.647 | 24.841 | 63.734 | 97.575 | 50.273 | 143.607 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 57.992 | 66.420 | 119.141 | 167.183 | 127.660 | 298.136 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.905 | 1.587 | 2.153 | 3.713 | 2.734 | 5.203 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.438 | 2.443 | 3.196 | 5.872 | 4.705 | 10.039 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 32.351 | 49.687 | 77.920 | 111.194 | 100.856 | 279.495 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.105 | 8.929 | 15.148 | 42.635 | 35.899 | 187.224 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.564 | 2.962 | 5.460 | 6.245 | 4.798 | 8.220 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 7.012 | 14.027 | 39.727 | 28.194 | 24.329 | 43.686 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 57.218 | 88.959 | 154.370 | 176.537 | 162.239 | 368.963 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 12.339 | 12.112 | 19.245 | 46.459 | 39.895 | 207.474 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.265 | 2.294 | 3.656 | 5.252 | 4.102 | 7.399 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.591 | 7.166 | 10.931 | 15.704 | 13.281 | 25.635 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 112.667 | 184.308 | 269.807 | 348.113 | 314.618 | 731.470 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 27.819 | 27.513 | 41.620 | 96.570 | 82.255 | 410.107 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 2.035 | 4.147 | 8.929 | 9.244 | 7.483 | 15.036 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.815 | 5.576 | 10.885 | 12.864 | 10.872 | 23.656 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 65.117 | 108.060 | 163.630 | 212.347 | 195.220 | 436.285 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 15.261 | 15.205 | 22.710 | 55.486 | 48.833 | 232.359 |

### reuse

| Document | Bytes | Profile | Agree 6 | Agree 5 | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | gfm | yes | yes | 0.065 | 0.068 | 0.084 | 0.744 | 0.155 | 0.290 |
| comment-question | 160 | gfm | yes | yes | 0.079 | 0.077 | 0.103 | 0.826 | 0.256 | 0.754 |
| comment-review | 282 | gfm | yes | yes | 0.127 | 0.126 | 0.189 | 0.983 | 0.435 | 1.286 |
| comment-links | 278 | gfm | yes | yes | 0.304 | 0.382 | 0.509 | 1.356 | 0.673 | 1.623 |
| comment-checklist | 287 | gfm | no | no | 0.489 | 0.516 | 0.673 | 1.437 | 0.828 | 1.802 |
| comment-quote | 290 | gfm | yes | yes | 0.193 | 0.189 | 0.238 | 1.074 | 0.499 | 1.434 |
| comment-unicode | 327 | gfm | yes | yes | 0.353 | 0.336 | 0.450 | 1.293 | 0.736 | 1.670 |
| comment-inline-code | 285 | gfm | yes | yes | 0.197 | 0.200 | 0.456 | 1.137 | 0.610 | 1.384 |
| comment-reproduction | 298 | gfm | yes | yes | 0.214 | 0.237 | 0.358 | 1.248 | 0.530 | 1.393 |
| comment-table | 310 | gfm | yes | yes | 0.938 | 0.905 | 0.849 | 2.112 | 1.139 | 2.188 |
| comment-review-long | 957 | gfm | yes | yes | 0.603 | 0.662 | 0.811 | 2.091 | 1.570 | 4.411 |
| comment-incident | 1124 | gfm | no | yes | 1.112 | 1.210 | 1.313 | 2.704 | 2.092 | 5.353 |
| guard-angle-link | 41 | commonmark | no | no | 0.178 | 0.189 | 0.300 | 0.937 | 0.312 | 0.473 |
| legacy-contributing | 9323 | gfm | no | yes | 9.216 | 10.852 | 15.567 | 23.014 | 21.575 | 47.471 |
| legacy-node-ferromark-readme | 9075 | gfm | no | yes | 8.680 | 10.743 | 16.150 | 24.796 | 21.684 | 46.727 |
| legacy-docs-migration-0-2 | 1985 | gfm | no | yes | 1.763 | 2.330 | 3.885 | 5.421 | 4.631 | 8.841 |
| legacy-docs-migration-0-3 | 2379 | gfm | no | yes | 2.383 | 2.923 | 4.091 | 6.314 | 5.112 | 10.595 |
| legacy-docs-migration-0-4 | 7045 | gfm | no | yes | 6.021 | 7.471 | 13.517 | 18.851 | 15.953 | 32.394 |
| legacy-docs-migration-0-8 | 2374 | gfm | no | yes | 2.305 | 2.950 | 4.612 | 6.849 | 5.545 | 10.531 |
| legacy-docs-releasing | 4141 | gfm | no | yes | 4.504 | 5.069 | 9.297 | 10.538 | 11.290 | 20.153 |
| legacy-docs-readme-theme | 2848 | gfm | no | yes | 2.015 | 2.452 | 4.185 | 6.628 | 6.101 | 13.494 |
| legacy-docs-markdown-extensions | 3707 | gfm | no | yes | 3.196 | 3.920 | 7.038 | 9.207 | 9.062 | 18.028 |
| legacy-docs-mdx | 7422 | gfm | no | yes | 6.485 | 7.576 | 12.336 | 19.221 | 14.988 | 36.464 |
| legacy-docs-readme | 1825 | gfm | no | yes | 3.893 | 4.387 | 5.376 | 9.020 | 6.232 | 12.377 |
| legacy-docs-adr-readme-theme-composition | 1532 | gfm | no | yes | 1.121 | 1.469 | 2.198 | 3.643 | 3.292 | 7.501 |
| rust-book-ch03-04-comments | 393 | gfm | no | yes | 0.434 | 0.687 | 0.857 | 1.893 | 1.195 | 2.571 |
| rust-book-appendix-02-operators | 22595 | gfm | no | yes | 49.758 | 49.196 | 58.364 | 79.951 | 62.827 | 103.822 |
| rust-book-ch00-00-introduction | 10839 | gfm | no | yes | 12.827 | 13.243 | 15.799 | 23.584 | 25.011 | 59.371 |
| rust-book-ch17-00-async-await | 9734 | gfm | no | yes | 6.900 | 6.766 | 11.802 | 17.501 | 17.308 | 47.948 |
| vue-docs-ways-of-using-vue | 5883 | gfm | no | yes | 4.290 | 6.087 | 6.755 | 12.295 | 10.901 | 30.826 |
| vue-docs-suspense | 8291 | gfm | no | yes | 7.849 | 9.731 | 18.176 | 21.939 | 19.309 | 44.112 |
| vue-docs-slots | 24211 | gfm | no | yes | 20.545 | 35.553 | 64.805 | 81.247 | 61.267 | 145.205 |
| vue-docs-reactivity-in-depth | 24001 | gfm | no | yes | 18.295 | 28.580 | 43.027 | 66.300 | 52.869 | 139.848 |
| vite-docs-philosophy | 3575 | gfm | no | yes | 2.503 | 3.811 | 4.208 | 8.167 | 7.075 | 18.409 |
| vite-docs-performance | 8184 | gfm | no | yes | 7.328 | 9.037 | 11.710 | 19.045 | 17.599 | 42.557 |
| vite-docs-api-plugin | 31890 | gfm | no | yes | 55.867 | 69.021 | 76.302 | 111.948 | 88.810 | 184.212 |
| vite-docs-features | 39739 | gfm | no | no | 56.663 | 67.518 | 97.201 | 137.148 | 116.082 | 240.766 |
| typescript-handbook-the-handbook | 5337 | gfm | no | yes | 4.209 | 5.582 | 5.384 | 10.486 | 9.160 | 26.111 |
| typescript-handbook-advanced-types | 36745 | gfm | no | yes | 46.489 | 56.226 | 79.326 | 119.660 | 91.477 | 191.937 |
| typescript-handbook-compiler-options | 54026 | gfm | no | yes | 24.380 | 24.327 | 61.824 | 95.840 | 48.669 | 142.303 |
| typescript-handbook-typescript-5-0 | 50714 | gfm | no | yes | 56.795 | 66.654 | 115.723 | 165.154 | 126.455 | 298.543 |
| wiki-rainbow-first-paragraph | 859 | commonmark | no | yes | 0.786 | 1.466 | 1.797 | 3.604 | 2.614 | 5.209 |
| wiki-rainbow-lead | 1859 | commonmark | no | yes | 1.290 | 2.270 | 2.736 | 5.576 | 4.450 | 9.806 |
| wiki-rainbow-article-body | 48422 | commonmark | no | no | 30.822 | 50.582 | 77.038 | 111.895 | 102.035 | 285.810 |
| wiki-rainbow-plain-prose | 38800 | commonmark | yes | yes | 9.283 | 8.893 | 13.455 | 41.497 | 34.792 | 192.322 |
| wiki-tea-first-paragraph | 1126 | commonmark | no | yes | 1.447 | 2.840 | 5.039 | 6.093 | 4.647 | 8.192 |
| wiki-tea-lead | 6363 | commonmark | no | yes | 6.716 | 13.548 | 37.952 | 27.385 | 23.434 | 42.766 |
| wiki-tea-article-body | 58814 | commonmark | no | no | 55.879 | 91.006 | 154.223 | 180.047 | 165.107 | 380.496 |
| wiki-tea-plain-prose | 40577 | commonmark | yes | yes | 12.262 | 12.005 | 16.969 | 45.042 | 38.184 | 206.473 |
| wiki-chess-first-paragraph | 1190 | commonmark | no | yes | 1.147 | 2.180 | 3.291 | 5.070 | 3.961 | 7.398 |
| wiki-chess-lead | 4125 | commonmark | no | yes | 3.473 | 7.032 | 10.203 | 15.444 | 12.973 | 25.684 |
| wiki-chess-article-body | 113609 | commonmark | no | no | 113.433 | 183.552 | 262.407 | 345.357 | 309.603 | 730.772 |
| wiki-chess-plain-prose | 80966 | commonmark | yes | yes | 28.275 | 27.966 | 38.433 | 95.577 | 81.406 | 421.419 |
| wiki-volcano-first-paragraph | 2644 | commonmark | no | yes | 1.967 | 4.147 | 8.707 | 9.298 | 7.548 | 15.497 |
| wiki-volcano-lead | 4232 | commonmark | no | yes | 2.695 | 5.457 | 10.306 | 12.598 | 10.529 | 23.520 |
| wiki-volcano-article-body | 69241 | commonmark | no | no | 63.900 | 109.214 | 160.497 | 210.964 | 193.381 | 441.924 |
| wiki-volcano-plain-prose | 47303 | commonmark | yes | yes | 15.617 | 15.380 | 20.747 | 54.387 | 47.410 | 239.245 |

## Rotating batches

Microseconds for a complete batch of all inputs in the named profile; lower is faster.
This total weights large documents more heavily and is not included in per-document geometric means.

| Batch | Mode | Documents | Ferromark v2 | OX-Content original | Ferromark v1 | md4c | pulldown-cmark | Bun native bun_md |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rotating-commonmark | fresh | 17 | 438.71 | 644.66 | 1021.40 | 1286.61 | 1123.66 | 3152.82 |
| rotating-commonmark | reuse | 17 | 418.04 | 613.42 | 951.54 | 1224.13 | 1072.36 | 3084.00 |
| rotating-gfm | fresh | 40 | 679.89 | 848.03 | 1261.08 | 1436.06 | 1172.66 | 2343.41 |
| rotating-gfm | reuse | 40 | 663.55 | 830.11 | 1212.64 | 1407.23 | 1141.56 | 2342.73 |
