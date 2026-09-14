# Broad Markdown: Ferromark main vs v2

Measured 2026-09-14T06:06:11Z on Apple M1 Pro.

**57 frozen inputs, 37–113,609 UTF-8 bytes.** Same parser versions and exact binaries as the first comparison; broader inputs are the experimental change.

Ratios below are **main time / v2 time: above 1 means v2 is faster**. Category and size tables are equal-document geometric means of admitted cases only. They are overlapping views, not scores to combine. No production-wide winner is inferred from this selection.

Output checks: 17 byte-identical, 19 serialization-equivalent, 14 heading-ID-only differences, 7 other differences. Only the first two statuses enter comparable aggregates. Every input remains measured and visible.

## By content family

| Stratum | Comparable / total | Fresh ratio | Owned ratio | Reuse ratio | Reuse leads: main / close / v2 |
| --- | ---: | ---: | ---: | ---: | --- |
| comments | 12 / 12 | 1.006× | 1.300× | 1.259× | 1 / 0 / 11 |
| encyclopedia | 9 / 12 | 1.773× | 1.645× | 1.618× | 0 / 0 / 9 |
| plain-prose | 4 / 4 | 1.349× | 1.331× | 1.188× | 0 / 0 / 4 |
| readme | 1 / 2 | 1.217× | 1.121× | 1.086× | 0 / 0 / 1 |
| reference | 1 / 4 | 1.094× | 1.079× | 1.066× | 0 / 0 / 1 |
| syntax-guard | 0 / 1 | — | — | — | 0 / 0 / 0 |
| technical-docs | 9 / 22 | 1.453× | 1.381× | 1.341× | 0 / 0 / 9 |

## By input size

| Stratum | Comparable / total | Fresh ratio | Owned ratio | Reuse ratio | Reuse leads: main / close / v2 |
| --- | ---: | ---: | ---: | ---: | --- |
| <512 B | 11 / 12 | 0.996× | 1.318× | 1.276× | 1 / 0 / 10 |
| 512 B–2 KiB | 8 / 9 | 1.430× | 1.308× | 1.280× | 0 / 0 / 8 |
| 2–10 KiB | 10 / 19 | 1.698× | 1.623× | 1.586× | 0 / 0 / 10 |
| 10–50 KiB | 5 / 12 | 1.279× | 1.250× | 1.150× | 0 / 0 / 5 |
| 50–256 KiB | 2 / 5 | 1.444× | 1.436× | 1.356× | 0 / 0 / 2 |

## By source collection

| Stratum | Comparable / total | Fresh ratio | Owned ratio | Reuse ratio | Reuse leads: main / close / v2 |
| --- | ---: | ---: | ---: | ---: | --- |
| authored-comments | 12 / 12 | 1.006× | 1.300× | 1.259× | 1 / 0 / 11 |
| encyclopedia | 9 / 12 | 1.773× | 1.645× | 1.618× | 0 / 0 / 9 |
| encyclopedia-prose | 4 / 4 | 1.349× | 1.331× | 1.188× | 0 / 0 / 4 |
| external-docs | 7 / 16 | 1.327× | 1.269× | 1.228× | 0 / 0 / 7 |
| legacy-docs | 4 / 12 | 1.519× | 1.428× | 1.401× | 0 / 0 / 4 |
| syntax-guards | 0 / 1 | — | — | — | 0 / 0 / 0 |

## Content × size coverage

Full-reuse geometric mean, with comparable/total document counts. Empty cells are unmeasured combinations; no interpolation is implied.

| Content | <512 B | 512 B–2 KiB | 2–10 KiB | 10–50 KiB | 50–256 KiB |
| --- | --- | --- | --- | --- | --- |
| comments | 1.28× (10/10) | 1.15× (2/2) | — | — | — |
| encyclopedia | — | 1.37× (4/4) | 1.93× (4/4) | diagnostic (0/1) | 1.58× (1/3) |
| plain-prose | — | — | — | 1.20× (3/3) | 1.16× (1/1) |
| readme | — | 1.09× (1/1) | diagnostic (0/1) | — | — |
| reference | — | — | — | 1.07× (1/3) | diagnostic (0/1) |
| syntax-guard | diagnostic (0/1) | — | — | — | — |
| technical-docs | 1.21× (1/1) | 1.43× (1/2) | 1.39× (6/14) | 1.10× (1/5) | — |

A lead requires every paired window to exceed a 5% speed advantage. Close/variable includes small differences and overlapping paired ranges; it is not a formal confidence interval.

## Individual inputs

Times are microseconds. Each lifecycle cell is `main / v2 (ratio)`. The final column gives the nine paired ratios' range for full reuse. Diagnostic rows cannot establish equivalent-output speedups.

| Input | Bytes | Output | Fresh | Owned | Reuse | Reuse range |
| --- | ---: | --- | --- | --- | --- | --- |
| comment-ack | 37 | exact | 0.274 / 0.402 (0.69×) | 0.147 / 0.102 (1.45×) | 0.110 / 0.075 (1.46×) | 1.45–1.47× |
| comment-question | 160 | exact | 0.299 / 0.419 (0.71×) | 0.165 / 0.120 (1.37×) | 0.131 / 0.095 (1.38×) | 1.37–1.39× |
| comment-links | 278 | exact | 1.125 / 0.794 (1.42×) | 0.605 / 0.492 (1.23×) | 0.555 / 0.464 (1.20×) | 1.18–1.21× |
| comment-review | 282 | exact | 0.408 / 0.492 (0.83×) | 0.274 / 0.191 (1.42×) | 0.238 / 0.164 (1.45×) | 1.44–1.46× |
| comment-inline-code | 285 | exact | 0.795 / 0.562 (1.42×) | 0.443 / 0.264 (1.69×) | 0.393 / 0.231 (1.70×) | 1.69–1.71× |
| comment-checklist | 287 | serialization-equivalent | 1.022 / 0.914 (1.11×) | 0.854 / 0.615 (1.39×) | 0.762 / 0.579 (1.31×) | 1.30–1.33× |
| comment-quote | 290 | exact | 0.459 / 0.550 (0.83×) | 0.304 / 0.253 (1.20×) | 0.260 / 0.222 (1.17×) | 1.16–1.18× |
| comment-reproduction | 298 | exact | 0.540 / 0.602 (0.90×) | 0.409 / 0.303 (1.35×) | 0.365 / 0.274 (1.33×) | 1.32–1.34× |
| comment-table | 310 | exact | 1.110 / 1.406 (0.79×) | 0.960 / 1.101 (0.87×) | 0.855 / 1.079 (0.79×) | 0.78–0.79× |
| comment-unicode | 327 | exact | 0.958 / 0.710 (1.34×) | 0.603 / 0.411 (1.46×) | 0.475 / 0.379 (1.26×) | 1.24–1.27× |
| comment-review-long | 957 | exact | 1.505 / 1.153 (1.31×) | 1.048 / 0.829 (1.26×) | 0.987 / 0.799 (1.24×) | 1.23–1.24× |
| comment-incident | 1,124 | exact | 2.048 / 1.787 (1.15×) | 1.588 / 1.442 (1.10×) | 1.509 / 1.411 (1.07×) | 1.06–1.08× |
| wiki-rainbow-first-paragraph | 859 | exact | 2.706 / 1.895 (1.43×) | 1.856 / 1.548 (1.20×) | 1.781 / 1.517 (1.17×) | 1.16–1.18× |
| wiki-tea-first-paragraph | 1,126 | exact | 6.245 / 3.287 (1.90×) | 5.096 / 2.931 (1.74×) | 5.002 / 2.893 (1.72×) | 1.71–1.74× |
| wiki-chess-first-paragraph | 1,190 | serialization-equivalent | 4.292 / 2.580 (1.66×) | 3.368 / 2.250 (1.50×) | 3.283 / 2.223 (1.48×) | 1.46–1.49× |
| wiki-rainbow-lead | 1,859 | serialization-equivalent | 3.773 / 2.748 (1.37×) | 2.874 / 2.399 (1.20×) | 2.774 / 2.380 (1.17×) | 1.16–1.18× |
| wiki-volcano-first-paragraph | 2,644 | serialization-equivalent | 9.825 / 4.557 (2.16×) | 8.618 / 4.222 (2.04×) | 8.570 / 4.175 (2.04×) | 2.02–2.06× |
| wiki-chess-lead | 4,125 | serialization-equivalent | 11.682 / 7.602 (1.53×) | 10.205 / 7.150 (1.43×) | 9.977 / 7.124 (1.39×) | 1.39–1.42× |
| wiki-volcano-lead | 4,232 | serialization-equivalent | 11.471 / 5.993 (1.91×) | 10.398 / 5.605 (1.85×) | 10.000 / 5.547 (1.80×) | 1.78–1.82× |
| wiki-tea-lead | 6,363 | exact | 40.373 / 14.781 (2.74×) | 38.234 / 14.084 (2.72×) | 38.050 / 14.111 (2.70×) | 2.66–2.72× |
| wiki-rainbow-article-body | 48,422 | different | 75.209 / 52.869 (1.43×) | 69.423 / 51.167 (1.35×) | 66.213 / 51.360 (1.30×) | 1.23–1.36× |
| wiki-tea-article-body | 58,814 | serialization-equivalent | 151.095 / 92.277 (1.62×) | 147.759 / 92.670 (1.61×) | 144.771 / 91.058 (1.58×) | 1.53–1.61× |
| wiki-volcano-article-body | 69,241 | different | 158.952 / 110.845 (1.42×) | 155.341 / 109.654 (1.42×) | 150.491 / 109.210 (1.39×) | 1.36–1.41× |
| wiki-chess-article-body | 113,609 | different | 251.630 / 188.096 (1.33×) | 245.017 / 185.326 (1.32×) | 241.869 / 188.012 (1.27×) | 1.23–1.32× |
| wiki-rainbow-plain-prose | 38,800 | serialization-equivalent | 13.597 / 9.449 (1.44×) | 13.001 / 9.015 (1.44×) | 11.098 / 8.945 (1.24×) | 1.23–1.25× |
| wiki-tea-plain-prose | 40,577 | serialization-equivalent | 16.532 / 12.657 (1.30×) | 15.569 / 12.224 (1.28×) | 13.803 / 12.128 (1.14×) | 1.13–1.15× |
| wiki-volcano-plain-prose | 47,303 | serialization-equivalent | 22.039 / 16.067 (1.37×) | 20.796 / 15.563 (1.34×) | 18.699 / 15.462 (1.21×) | 1.20–1.22× |
| wiki-chess-plain-prose | 80,966 | serialization-equivalent | 36.928 / 28.666 (1.29×) | 36.667 / 28.362 (1.28×) | 32.691 / 28.125 (1.16×) | 1.15–1.17× |
| legacy-docs-readme | 1,825 | exact | 6.619 / 5.435 (1.22×) | 5.706 / 5.108 (1.12×) | 5.486 / 5.049 (1.09×) | 1.08–1.10× |
| legacy-node-ferromark-readme | 9,075 | heading-id-only | 18.601 / 12.713 (1.46×) | 16.941 / 12.290 (1.37×) | 16.234 / 12.260 (1.32×) | 1.32–1.35× |
| rust-book-appendix-02-operators | 22,595 | serialization-equivalent | 61.586 / 56.200 (1.09×) | 59.727 / 55.266 (1.08×) | 59.033 / 55.162 (1.07×) | 1.05–1.08× |
| vite-docs-api-plugin | 31,890 | heading-id-only | 82.729 / 85.252 (0.98×) | 80.262 / 81.670 (0.96×) | 78.808 / 82.078 (0.96×) | 0.94–1.01× |
| typescript-handbook-advanced-types | 36,745 | heading-id-only | 74.581 / 68.033 (1.09×) | 69.801 / 64.511 (1.09×) | 67.489 / 65.331 (1.03×) | 0.99–1.07× |
| typescript-handbook-compiler-options | 54,026 | heading-id-only | 80.846 / 81.778 (0.99×) | 79.736 / 81.064 (0.99×) | 78.459 / 80.979 (0.97×) | 0.96–0.99× |
| guard-angle-link | 41 | different | 0.887 / 0.536 (1.67×) | 0.390 / 0.233 (1.67×) | 0.310 / 0.209 (1.49×) | 1.48–1.50× |
| rust-book-ch03-04-comments | 393 | serialization-equivalent | 1.540 / 1.148 (1.34×) | 1.006 / 0.809 (1.24×) | 0.940 / 0.774 (1.21×) | 1.20–1.23× |
| legacy-docs-adr-readme-theme-composition | 1,532 | serialization-equivalent | 3.293 / 2.127 (1.54×) | 2.617 / 1.791 (1.46×) | 2.521 / 1.757 (1.43×) | 1.42–1.45× |
| legacy-docs-migration-0-2 | 1,985 | heading-id-only | 4.666 / 3.108 (1.51×) | 4.132 / 2.773 (1.49×) | 3.895 / 2.723 (1.43×) | 1.42–1.44× |
| legacy-docs-migration-0-8 | 2,374 | heading-id-only | 5.769 / 3.653 (1.58×) | 4.835 / 3.317 (1.46×) | 4.613 / 3.292 (1.40×) | 1.39–1.41× |
| legacy-docs-migration-0-3 | 2,379 | heading-id-only | 5.077 / 3.738 (1.36×) | 4.366 / 3.370 (1.29×) | 4.202 / 3.347 (1.24×) | 1.24–1.26× |
| legacy-docs-readme-theme | 2,848 | serialization-equivalent | 5.476 / 3.199 (1.70×) | 4.578 / 2.870 (1.60×) | 4.449 / 2.811 (1.58×) | 1.58–1.60× |
| vite-docs-philosophy | 3,575 | serialization-equivalent | 6.343 / 4.722 (1.34×) | 5.454 / 4.416 (1.24×) | 5.218 / 4.387 (1.19×) | 1.19–1.21× |
| legacy-docs-markdown-extensions | 3,707 | serialization-equivalent | 8.060 / 4.831 (1.67×) | 7.061 / 4.473 (1.58×) | 6.912 / 4.415 (1.56×) | 1.55–1.57× |
| legacy-docs-releasing | 4,141 | heading-id-only | 10.856 / 6.210 (1.75×) | 9.803 / 5.818 (1.68×) | 9.519 / 5.752 (1.67×) | 1.64–1.71× |
| typescript-handbook-the-handbook | 5,337 | heading-id-only | 7.531 / 7.022 (1.07×) | 6.426 / 6.391 (1.00×) | 6.227 / 6.377 (0.98×) | 0.97–0.99× |
| vue-docs-ways-of-using-vue | 5,883 | heading-id-only | 9.215 / 7.827 (1.18×) | 8.321 / 7.386 (1.13×) | 8.124 / 7.307 (1.11×) | 1.10–1.13× |
| legacy-docs-migration-0-4 | 7,045 | heading-id-only | 13.867 / 8.966 (1.54×) | 12.933 / 8.332 (1.55×) | 12.458 / 8.334 (1.50×) | 1.47–1.51× |
| legacy-docs-mdx | 7,422 | heading-id-only | 12.833 / 9.204 (1.39×) | 11.567 / 8.740 (1.32×) | 11.031 / 8.739 (1.26×) | 1.25–1.28× |
| vite-docs-performance | 8,184 | serialization-equivalent | 14.325 / 11.000 (1.30×) | 13.283 / 10.503 (1.27×) | 13.026 / 10.524 (1.25×) | 1.22–1.25× |
| vue-docs-suspense | 8,291 | serialization-equivalent | 17.972 / 11.455 (1.58×) | 16.286 / 10.813 (1.50×) | 15.576 / 10.837 (1.43×) | 1.42–1.44× |
| legacy-contributing | 9,323 | heading-id-only | 17.827 / 12.731 (1.40×) | 16.761 / 12.294 (1.36×) | 16.192 / 12.330 (1.31×) | 1.28–1.33× |
| rust-book-ch17-00-async-await | 9,734 | exact | 13.040 / 8.795 (1.48×) | 11.944 / 8.223 (1.45×) | 11.320 / 8.183 (1.39×) | 1.37–1.39× |
| rust-book-ch00-00-introduction | 10,839 | exact | 18.927 / 15.513 (1.22×) | 17.391 / 15.004 (1.16×) | 16.420 / 14.928 (1.10×) | 1.09–1.11× |
| vue-docs-reactivity-in-depth | 24,001 | different | 41.466 / 33.062 (1.26×) | 38.747 / 32.062 (1.21×) | 38.063 / 32.068 (1.19×) | 1.16–1.27× |
| vue-docs-slots | 24,211 | different | 60.701 / 39.799 (1.52×) | 56.521 / 39.271 (1.45×) | 56.992 / 39.545 (1.40×) | 1.29–1.62× |
| vite-docs-features | 39,739 | different | 113.495 / 84.277 (1.34×) | 109.323 / 81.046 (1.33×) | 109.307 / 82.381 (1.34×) | 1.23–1.38× |
| typescript-handbook-typescript-5-0 | 50,714 | heading-id-only | 112.459 / 84.337 (1.34×) | 111.590 / 83.438 (1.34×) | 105.888 / 81.079 (1.28×) | 1.19–1.32× |

## Rotating collections

Fixed cohorts visit each input once per traversal. Times are total microseconds per complete collection, rather than per-document averages. A cohort with any output mismatch is diagnostic; membership is never silently reduced.

| Collection | Mode | Documents | Comparable | Main µs | v2 µs | Ratio |
| --- | --- | ---: | --- | ---: | ---: | ---: |
| rotating-authored-comments-gfm | fresh | 12 | True | 11.410 | 10.285 | 1.109× |
| rotating-authored-comments-gfm | owned | 12 | True | 8.105 | 6.462 | 1.255× |
| rotating-authored-comments-gfm | reuse | 12 | True | 7.200 | 5.998 | 1.196× |
| rotating-encyclopedia-commonmark | fresh | 12 | False | 823.447 | 558.580 | 1.473× |
| rotating-encyclopedia-commonmark | owned | 12 | False | 798.237 | 548.061 | 1.440× |
| rotating-encyclopedia-commonmark | reuse | 12 | False | 775.627 | 552.039 | 1.399× |
| rotating-encyclopedia-prose-commonmark | fresh | 4 | True | 99.515 | 74.639 | 1.344× |
| rotating-encyclopedia-prose-commonmark | owned | 4 | True | 96.581 | 74.052 | 1.315× |
| rotating-encyclopedia-prose-commonmark | reuse | 4 | True | 85.652 | 72.435 | 1.176× |
| rotating-external-docs-gfm | fresh | 16 | False | 1089.704 | 858.658 | 1.269× |
| rotating-external-docs-gfm | owned | 16 | False | 1048.829 | 848.647 | 1.235× |
| rotating-external-docs-gfm | reuse | 16 | False | 1027.447 | 844.473 | 1.223× |
| rotating-legacy-docs-gfm | fresh | 12 | False | 165.787 | 102.101 | 1.647× |
| rotating-legacy-docs-gfm | owned | 12 | False | 149.125 | 98.491 | 1.511× |
| rotating-legacy-docs-gfm | reuse | 12 | False | 142.593 | 96.647 | 1.475× |
| rotating-syntax-guards-commonmark | fresh | 1 | False | 0.893 | 0.538 | 1.657× |
| rotating-syntax-guards-commonmark | owned | 1 | False | 0.388 | 0.234 | 1.650× |
| rotating-syntax-guards-commonmark | reuse | 1 | False | 0.308 | 0.208 | 1.488× |

## Evidence and limits

- Main: `a6e9906f7b4a01355d336f209fd12534796419df`. v2: `d1481ca7687e94d30e56f473d3044a175dc6c9a6`. No parser changes.
- 3 rounds × 3 alternating pairs; 1,701 pairs total. At least 75 ms/window plus 75 ms warmups. Every raw window and round median is retained.
- CommonMark for encyclopedia prose; GFM for comments/docs, trusted HTML, heading IDs enabled, footnotes disabled. Conversion, I/O, process startup, bindings, highlighting, and MDX execution are not timed.
- Fresh includes each API's actual setup costs. Reuse still parses every document; no AST or rendered-result cache is reused.
- Source families, sizes, and comments are deliberately varied; this is not a sampled production traffic distribution. Multiple excerpts from one article are correlated.
- Strict output admission can leave sparse or empty cells. Mismatching cases remain diagnostics; their ratios do not prove compatibility.
- Local Apple Silicon measurement on an active desktop, with browser/backup activity observed at preflight. Host power/load observations are archived; unavailable thermal telemetry is recorded. Small differences are not portable claims.

[Methodology and reproduction](../../../benchmarks/broad-comparison/README.md) · [CSV](summary.csv) · [Stratified data](strata.json) · [Per-case summaries and round medians](summary.json)

[Frozen corpus and provenance](corpus.json.gz) · [Full HTML and diffs](verification.json.gz) · [Raw timing pairs](samples.json.gz) · [Run configuration](run.json) · [Build hashes](build.json) · [Harness hashes](harness-hashes.json)

[Independent source/archive byte verification](source-validation.json)
