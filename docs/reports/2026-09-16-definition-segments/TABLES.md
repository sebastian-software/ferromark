# Per-case tables — segmented definition pass against main

Ratios are baseline time over candidate time (median of the paired windows); higher is faster.
`rounds` lists the per-round medians. Baseline time is the median nanoseconds per document.

## Issue 320 documents and marker-free variants (9)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.970 | 0.958 | 0.985 | 0.971 / 0.970 / 0.968 | 525 |
| `typescript-handbook-advanced-types-nodef` | reference | 36745 | 0.995 | 0.952 | 1.103 | 0.992 / 0.995 / 1.016 | 40,253 |
| `rust-book-ch00-00-introduction-nodef` | technical-docs | 10839 | 0.998 | 0.914 | 1.009 | 0.999 / 0.996 / 0.997 | 12,555 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.999 | 0.972 | 1.025 | 0.997 / 1.017 / 0.999 | 32,055 |
| `comment-incident-nodef` | comments | 1124 | 1.002 | 0.979 | 1.049 | 1.006 / 1.000 / 1.000 | 1,230 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.061 | 0.971 | 1.127 | 1.070 / 1.071 / 1.051 | 51,276 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.170 | 1.143 | 1.193 | 1.166 / 1.176 / 1.173 | 3,855 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.197 | 1.176 | 1.202 | 1.198 / 1.197 / 1.190 | 16,830 |
| `comment-incident` | comments | 1124 | 1.295 | 1.246 | 1.323 | 1.299 / 1.277 / 1.302 | 1,782 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.979 | 0.971 | 0.988 | 0.979 / 0.979 / 0.982 | 425 |
| `typescript-handbook-advanced-types-nodef` | reference | 36745 | 0.999 | 0.936 | 1.138 | 0.999 / 1.036 / 0.992 | 39,551 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.000 | 0.980 | 1.018 | 1.001 / 0.995 / 1.000 | 31,976 |
| `rust-book-ch00-00-introduction-nodef` | technical-docs | 10839 | 1.003 | 0.995 | 1.057 | 1.002 / 0.999 / 1.004 | 12,293 |
| `comment-incident-nodef` | comments | 1124 | 1.004 | 0.993 | 1.037 | 0.999 / 1.009 / 1.004 | 1,067 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.041 | 0.941 | 1.134 | 1.018 / 1.021 / 1.072 | 50,396 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.172 | 1.159 | 1.227 | 1.171 / 1.181 / 1.176 | 3,553 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.196 | 1.180 | 1.209 | 1.196 / 1.196 / 1.202 | 16,576 |
| `comment-incident` | comments | 1124 | 1.342 | 1.302 | 1.398 | 1.398 / 1.331 / 1.335 | 1,650 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.971 | 0.966 | 0.980 | 0.970 / 0.972 / 0.971 | 391 |
| `typescript-handbook-advanced-types-nodef` | reference | 36745 | 0.986 | 0.950 | 1.035 | 0.978 / 0.986 / 1.000 | 20,820 |
| `rust-book-ch00-00-introduction-nodef` | technical-docs | 10839 | 0.997 | 0.980 | 1.020 | 0.993 / 1.001 / 0.997 | 9,834 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.004 | 0.995 | 1.027 | 1.003 / 1.004 / 1.014 | 29,546 |
| `comment-incident-nodef` | comments | 1124 | 1.006 | 0.999 | 1.033 | 1.007 / 1.007 / 1.002 | 770 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.095 | 1.046 | 1.128 | 1.095 / 1.093 / 1.101 | 31,375 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.218 | 1.181 | 1.239 | 1.218 / 1.222 / 1.213 | 3,046 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.235 | 1.209 | 1.278 | 1.235 / 1.215 / 1.255 | 14,477 |
| `comment-incident` | comments | 1124 | 1.443 | 1.410 | 1.457 | 1.446 / 1.448 / 1.434 | 1,336 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.975 | 0.969 | 0.999 | 0.983 / 0.973 / 0.978 | 40 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.994 | 0.981 | 1.034 | 0.995 / 0.994 / 0.994 | 2,442 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.997 | 0.988 | 1.005 | 0.998 / 0.994 / 0.998 | 474 |
| `comment-incident-nodef` | comments | 1124 | 1.000 | 0.973 | 1.010 | 0.996 / 1.002 / 0.994 | 283 |
| `typescript-handbook-advanced-types-nodef` | reference | 36745 | 1.002 | 0.962 | 1.060 | 0.993 / 1.006 / 1.002 | 15,703 |
| `rust-book-ch00-00-introduction-nodef` | technical-docs | 10839 | 1.005 | 0.995 | 1.012 | 1.007 / 1.003 / 1.008 | 2,336 |
| `comment-incident` | comments | 1124 | 1.007 | 0.997 | 1.021 | 1.002 / 1.004 / 1.010 | 284 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.008 | 0.996 | 1.054 | 1.016 / 1.007 / 1.002 | 2,256 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.013 | 0.935 | 1.082 | 1.051 / 1.003 / 1.013 | 15,848 |

## Original broad documents (57)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-question` | comments | 160 | 0.983 | 0.969 | 0.994 | 0.976 / 0.979 / 0.989 | 166 |
| `comment-ack` | comments | 37 | 0.984 | 0.977 | 0.993 | 0.987 / 0.985 / 0.984 | 145 |
| `vite-docs-api-plugin` | reference | 31890 | 0.988 | 0.947 | 1.090 | 0.984 / 1.001 / 0.987 | 64,731 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.989 | 0.967 | 1.000 | 0.986 / 0.992 / 0.989 | 672 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.989 | 0.977 | 1.001 | 0.989 / 0.989 / 0.998 | 3,833 |
| `comment-reproduction` | comments | 298 | 0.990 | 0.969 | 1.052 | 0.987 / 0.977 / 1.047 | 304 |
| `comment-table` | comments | 310 | 0.990 | 0.985 | 1.017 | 1.007 / 0.990 / 0.990 | 1,132 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.991 | 0.972 | 1.000 | 0.993 / 0.990 / 0.991 | 2,422 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.992 | 0.970 | 0.999 | 0.996 / 0.990 / 0.979 | 5,693 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.993 | 0.970 | 1.009 | 0.993 / 0.994 / 0.991 | 2,944 |
| `legacy-contributing` | technical-docs | 9323 | 0.994 | 0.979 | 1.028 | 0.989 / 0.993 / 1.003 | 10,601 |
| `comment-inline-code` | comments | 285 | 0.994 | 0.989 | 1.003 | 0.992 / 0.995 / 0.997 | 290 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.994 | 0.979 | 0.999 | 0.996 / 0.994 / 0.991 | 1,569 |
| `comment-review` | comments | 282 | 0.995 | 0.955 | 1.022 | 0.994 / 0.995 / 0.994 | 230 |
| `comment-quote` | comments | 290 | 0.995 | 0.979 | 1.005 | 0.995 / 0.992 / 0.997 | 291 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.996 | 0.982 | 1.021 | 0.987 / 1.000 / 0.999 | 9,458 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.996 | 0.985 | 1.006 | 0.993 / 0.995 / 0.998 | 3,837 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.982 | 1.004 | 0.997 / 0.992 / 0.997 | 53,615 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.997 | 0.986 | 1.003 | 0.996 / 0.995 / 0.998 | 10,156 |
| `comment-links` | comments | 278 | 0.997 | 0.991 | 1.050 | 0.993 / 1.046 / 0.997 | 477 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.997 | 0.982 | 1.006 | 0.994 / 1.003 / 0.999 | 5,409 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.997 | 0.985 | 1.000 | 0.997 / 0.993 / 0.998 | 2,910 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.998 | 0.993 | 1.017 | 0.998 / 0.997 / 1.013 | 7,638 |
| `legacy-docs-readme` | readme | 1825 | 0.998 | 0.974 | 1.016 | 0.995 / 0.995 / 1.002 | 4,605 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.998 | 0.968 | 1.011 | 0.998 / 0.999 / 0.997 | 1,026 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.998 | 0.974 | 1.009 | 0.996 / 0.998 / 1.002 | 14,930 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.968 | 1.022 | 1.003 / 0.995 / 0.992 | 27,364 |
| `vue-docs-slots` | technical-docs | 24211 | 0.999 | 0.968 | 1.018 | 0.990 / 1.004 / 0.997 | 23,261 |
| `comment-unicode` | comments | 327 | 0.999 | 0.982 | 1.005 | 0.991 / 1.002 / 1.000 | 454 |
| `guard-angle-link` | syntax-guard | 41 | 0.999 | 0.954 | 1.016 | 0.996 / 0.996 / 1.009 | 269 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.999 | 0.986 | 1.021 | 1.003 / 1.000 / 0.990 | 9,446 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.000 | 0.992 | 1.023 | 0.998 / 0.994 / 1.002 | 3,109 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.000 | 0.989 | 1.017 | 1.001 / 0.999 / 1.000 | 2,526 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.000 | 0.997 | 1.012 | 0.999 / 1.000 / 1.002 | 1,605 |
| `comment-checklist` | comments | 287 | 1.000 | 0.967 | 1.019 | 1.001 / 1.002 / 0.996 | 568 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.000 | 0.948 | 1.080 | 1.000 / 1.008 / 0.991 | 64,357 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.000 | 0.961 | 1.011 | 1.000 / 0.999 / 1.001 | 1,684 |
| `vite-docs-performance` | technical-docs | 8184 | 1.001 | 0.992 | 1.048 | 0.998 / 1.016 / 1.003 | 8,770 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.001 | 0.986 | 1.015 | 0.999 / 0.995 / 1.002 | 12,166 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.001 | 0.986 | 1.017 | 0.998 / 1.006 / 1.001 | 7,436 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.002 | 0.967 | 1.040 | 0.996 / 0.997 / 1.009 | 22,441 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.002 | 0.986 | 1.024 | 0.999 / 1.012 / 1.000 | 2,280 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.002 | 0.957 | 1.132 | 1.031 / 0.997 / 1.014 | 33,014 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.002 | 0.989 | 1.037 | 1.002 / 1.001 / 1.017 | 5,291 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.003 | 0.993 | 1.023 | 1.003 / 1.003 / 1.005 | 77,539 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.003 | 0.989 | 1.030 | 1.002 / 0.998 / 1.008 | 7,456 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.004 | 0.988 | 1.049 | 1.025 / 1.001 / 1.004 | 1,387 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.004 | 0.996 | 1.017 | 1.003 / 1.006 / 1.003 | 8,096 |
| `comment-review-long` | comments | 957 | 1.006 | 0.991 | 1.016 | 1.007 / 0.998 / 1.012 | 762 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.007 | 0.967 | 1.064 | 0.979 / 1.024 / 0.998 | 57,703 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.008 | 0.902 | 1.045 | 1.000 / 1.008 / 1.009 | 116,215 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.009 | 0.994 | 1.019 | 1.009 / 1.001 / 1.002 | 3,400 |
| `vite-docs-features` | technical-docs | 39739 | 1.012 | 0.934 | 1.182 | 1.029 / 1.012 / 1.007 | 60,156 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.024 | 0.936 | 1.132 | 1.009 / 1.024 / 1.053 | 61,454 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.051 | 0.934 | 1.130 | 1.068 / 1.040 / 1.038 | 51,589 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.196 | 1.146 | 1.243 | 1.199 / 1.197 / 1.190 | 17,140 |
| `comment-incident` | comments | 1124 | 1.301 | 1.294 | 1.333 | 1.308 / 1.300 / 1.302 | 1,780 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.982 | 0.971 | 1.013 | 0.982 / 0.982 / 0.983 | 66 |
| `vite-docs-api-plugin` | reference | 31890 | 0.985 | 0.911 | 1.112 | 0.985 / 0.987 / 1.006 | 66,009 |
| `comment-question` | comments | 160 | 0.987 | 0.960 | 1.003 | 0.986 / 0.987 / 0.988 | 87 |
| `comment-inline-code` | comments | 285 | 0.990 | 0.982 | 0.999 | 0.990 / 0.992 / 0.990 | 211 |
| `comment-reproduction` | comments | 298 | 0.991 | 0.958 | 1.017 | 0.997 / 0.987 / 0.992 | 226 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.993 | 0.966 | 1.005 | 0.986 / 0.993 / 0.994 | 7,439 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.993 | 0.978 | 1.016 | 0.998 / 0.989 / 0.993 | 3,632 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.993 | 0.973 | 1.030 | 0.993 / 0.990 / 0.999 | 5,011 |
| `legacy-contributing` | technical-docs | 9323 | 0.994 | 0.983 | 1.010 | 0.991 / 0.996 / 0.993 | 10,175 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.994 | 0.981 | 1.029 | 0.995 / 0.986 / 1.006 | 27,012 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.995 | 0.982 | 1.015 | 0.999 / 0.997 / 0.993 | 11,849 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.995 | 0.976 | 1.017 | 0.995 / 1.008 / 0.992 | 9,003 |
| `comment-quote` | comments | 290 | 0.995 | 0.990 | 1.003 | 0.997 / 1.001 / 0.993 | 212 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.995 | 0.982 | 1.038 | 0.994 / 0.997 / 0.995 | 9,916 |
| `legacy-docs-readme` | readme | 1825 | 0.995 | 0.992 | 1.003 | 0.996 / 0.994 / 0.999 | 4,355 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.996 | 0.992 | 1.014 | 0.996 / 0.996 / 0.999 | 2,778 |
| `comment-review` | comments | 282 | 0.996 | 0.974 | 1.005 | 0.996 / 0.995 / 1.000 | 150 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.991 | 1.009 | 0.996 / 0.997 / 1.000 | 54,201 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.997 | 0.988 | 1.038 | 1.002 / 0.995 / 0.998 | 1,193 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.997 | 0.970 | 1.010 | 0.996 / 0.996 / 0.997 | 2,348 |
| `guard-angle-link` | syntax-guard | 41 | 0.997 | 0.956 | 1.032 | 1.002 / 0.996 / 0.994 | 175 |
| `comment-table` | comments | 310 | 0.997 | 0.990 | 1.002 | 0.997 / 0.992 / 0.997 | 1,022 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.997 | 0.969 | 1.009 | 0.994 / 0.999 / 0.992 | 2,257 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.997 | 0.963 | 1.028 | 0.998 / 1.007 / 0.996 | 512 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.998 | 0.979 | 1.007 | 1.000 / 0.998 / 0.986 | 8,916 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.998 | 0.974 | 1.033 | 0.990 / 1.003 / 0.998 | 21,553 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.999 | 0.987 | 1.061 | 0.999 / 1.006 / 0.995 | 1,413 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.999 | 0.986 | 1.013 | 0.997 / 1.000 / 0.998 | 5,515 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.000 | 0.981 | 1.021 | 1.003 / 0.995 / 1.000 | 14,781 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.000 | 0.989 | 1.015 | 1.000 / 1.000 / 1.000 | 2,860 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.000 | 0.988 | 1.010 | 0.994 / 1.000 / 1.002 | 4,995 |
| `comment-unicode` | comments | 327 | 1.001 | 0.976 | 1.009 | 1.002 / 0.998 / 0.998 | 357 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.001 | 0.992 | 1.018 | 1.006 / 0.999 / 1.001 | 1,514 |
| `vue-docs-slots` | technical-docs | 24211 | 1.001 | 0.965 | 1.034 | 1.000 / 1.000 / 1.009 | 23,185 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.001 | 0.984 | 1.033 | 1.001 / 1.004 / 1.001 | 858 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.001 | 0.987 | 1.022 | 1.001 / 1.003 / 1.001 | 2,101 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.002 | 0.985 | 1.009 | 1.002 / 0.997 / 1.002 | 2,744 |
| `comment-checklist` | comments | 287 | 1.002 | 0.992 | 1.013 | 1.001 / 1.009 / 1.000 | 482 |
| `comment-links` | comments | 278 | 1.002 | 0.989 | 1.017 | 1.007 / 1.001 / 0.995 | 367 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.003 | 0.958 | 1.078 | 1.004 / 0.996 / 1.003 | 32,538 |
| `comment-review-long` | comments | 957 | 1.003 | 0.988 | 1.030 | 1.008 / 1.002 / 1.003 | 662 |
| `vite-docs-performance` | technical-docs | 8184 | 1.004 | 0.980 | 1.018 | 0.996 / 1.004 / 1.005 | 8,453 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.004 | 0.995 | 1.016 | 1.007 / 1.002 / 1.004 | 1,449 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.005 | 0.988 | 1.024 | 1.002 / 1.011 / 1.005 | 7,228 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.005 | 0.998 | 1.010 | 1.004 / 1.003 / 1.006 | 6,908 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.006 | 0.990 | 1.038 | 1.006 / 1.011 / 1.003 | 77,911 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.006 | 1.002 | 1.035 | 1.005 / 1.006 / 1.006 | 3,189 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.007 | 0.987 | 1.018 | 0.994 / 1.009 / 1.007 | 7,879 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.008 | 0.965 | 1.046 | 1.008 / 0.985 / 1.028 | 116,447 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.009 | 0.976 | 1.036 | 1.028 / 1.010 / 1.000 | 3,728 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.011 | 0.959 | 1.075 | 1.024 / 0.997 / 1.011 | 56,191 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.017 | 0.819 | 1.055 | 1.009 / 1.017 / 1.017 | 64,244 |
| `vite-docs-features` | technical-docs | 39739 | 1.027 | 0.931 | 1.118 | 1.027 / 1.065 / 0.996 | 62,418 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.034 | 0.911 | 1.173 | 1.043 / 1.030 / 0.961 | 61,819 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.050 | 0.970 | 1.123 | 1.032 / 1.095 / 1.050 | 50,460 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.203 | 1.182 | 1.214 | 1.203 / 1.204 / 1.208 | 16,694 |
| `comment-incident` | comments | 1124 | 1.338 | 1.311 | 1.349 | 1.338 / 1.338 / 1.341 | 1,625 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.968 | 0.960 | 0.990 | 0.967 / 0.968 / 0.970 | 48 |
| `comment-question` | comments | 160 | 0.981 | 0.955 | 0.986 | 0.981 / 0.983 / 0.979 | 65 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.988 | 0.974 | 1.002 | 0.985 / 0.981 / 0.990 | 1,569 |
| `comment-reproduction` | comments | 298 | 0.988 | 0.981 | 0.995 | 0.988 / 0.988 / 0.983 | 161 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.988 | 0.967 | 1.006 | 0.988 / 0.992 / 0.988 | 1,726 |
| `comment-inline-code` | comments | 285 | 0.989 | 0.978 | 1.015 | 0.993 / 0.987 / 0.987 | 130 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.990 | 0.925 | 1.050 | 0.989 / 1.005 / 1.006 | 33,832 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.990 | 0.985 | 1.032 | 0.990 / 0.991 / 0.987 | 532 |
| `comment-review` | comments | 282 | 0.990 | 0.979 | 0.999 | 0.993 / 0.991 / 0.989 | 108 |
| `comment-quote` | comments | 290 | 0.991 | 0.975 | 1.001 | 0.986 / 0.999 / 0.985 | 165 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.992 | 0.956 | 1.007 | 0.982 / 0.991 / 0.995 | 4,904 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.993 | 0.955 | 1.034 | 0.996 / 0.987 / 0.974 | 36,690 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.993 | 0.972 | 1.017 | 1.003 / 0.993 / 0.987 | 6,220 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.993 | 0.983 | 1.017 | 0.993 / 0.991 / 0.995 | 4,372 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.994 | 0.972 | 1.015 | 0.997 / 1.000 / 0.983 | 19,678 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.994 | 0.987 | 1.004 | 0.995 / 0.994 / 0.992 | 13,940 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.996 | 0.973 | 1.007 | 1.000 / 0.994 / 0.993 | 5,815 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.996 | 0.980 | 1.013 | 0.997 / 0.990 / 0.996 | 1,319 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.996 | 0.976 | 1.007 | 0.996 / 0.996 / 0.996 | 1,623 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.997 | 0.984 | 1.006 | 1.001 / 0.995 / 0.996 | 2,255 |
| `comment-table` | comments | 310 | 0.997 | 0.975 | 1.036 | 0.993 / 0.999 / 0.995 | 843 |
| `legacy-contributing` | technical-docs | 9323 | 0.997 | 0.981 | 1.014 | 1.000 / 0.991 / 0.995 | 7,468 |
| `comment-unicode` | comments | 327 | 0.997 | 0.979 | 1.007 | 0.993 / 1.001 / 0.996 | 273 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.958 | 1.031 | 0.993 / 0.993 / 1.000 | 46,082 |
| `vite-docs-features` | technical-docs | 39739 | 0.997 | 0.931 | 1.070 | 0.992 / 1.000 / 0.997 | 36,505 |
| `vite-docs-api-plugin` | reference | 31890 | 0.997 | 0.949 | 1.024 | 1.001 / 0.995 / 0.995 | 47,118 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.997 | 0.978 | 1.002 | 0.991 / 0.998 / 0.997 | 922 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.998 | 0.992 | 1.004 | 0.999 / 0.997 / 0.995 | 3,716 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.998 | 0.992 | 1.014 | 0.992 / 0.997 / 1.011 | 2,470 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.998 | 0.981 | 1.008 | 1.000 / 0.997 / 0.998 | 4,919 |
| `vue-docs-slots` | technical-docs | 24211 | 0.999 | 0.992 | 1.012 | 1.000 / 0.997 / 0.997 | 13,011 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.999 | 0.945 | 1.012 | 0.999 / 0.996 / 1.004 | 966 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.999 | 0.984 | 1.007 | 0.994 / 1.000 / 0.999 | 744 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.000 | 0.971 | 1.021 | 0.999 / 1.005 / 0.996 | 1,736 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.989 | 1.013 | 0.998 / 0.996 / 1.005 | 3,400 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.000 | 0.972 | 1.024 | 1.006 / 0.997 / 1.000 | 1,306 |
| `legacy-docs-readme` | readme | 1825 | 1.000 | 0.983 | 1.020 | 1.001 / 0.996 / 1.000 | 3,353 |
| `guard-angle-link` | syntax-guard | 41 | 1.001 | 0.978 | 1.042 | 1.001 / 1.001 / 0.998 | 143 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.001 | 0.989 | 1.020 | 0.999 / 0.997 / 1.001 | 881 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.001 | 0.939 | 1.067 | 1.001 / 0.988 / 1.027 | 65,472 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.002 | 0.990 | 1.015 | 0.999 / 1.010 / 0.995 | 350 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.002 | 0.994 | 1.019 | 0.998 / 1.001 / 1.005 | 17,500 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.002 | 0.988 | 1.016 | 1.002 / 1.003 / 1.002 | 8,052 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.002 | 0.989 | 1.012 | 1.002 / 1.005 / 0.998 | 10,006 |
| `comment-links` | comments | 278 | 1.004 | 0.982 | 1.036 | 1.007 / 0.997 / 1.004 | 283 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.004 | 0.996 | 1.021 | 1.004 / 1.002 / 1.007 | 3,549 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.004 | 0.991 | 1.036 | 1.004 / 1.001 / 1.015 | 4,605 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.004 | 0.943 | 1.016 | 1.006 / 1.004 / 1.008 | 1,972 |
| `comment-review-long` | comments | 957 | 1.005 | 1.000 | 1.021 | 1.002 / 1.005 / 1.008 | 508 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.005 | 0.986 | 1.023 | 1.005 / 1.004 / 1.017 | 6,163 |
| `comment-checklist` | comments | 287 | 1.008 | 0.999 | 1.017 | 1.009 / 1.008 / 1.003 | 387 |
| `vite-docs-performance` | technical-docs | 8184 | 1.009 | 0.963 | 1.035 | 1.014 / 0.998 / 1.012 | 5,828 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.013 | 0.963 | 1.072 | 0.980 / 1.027 / 1.016 | 33,079 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.018 | 1.009 | 1.041 | 1.014 / 1.021 / 1.020 | 19,940 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.086 | 1.048 | 1.107 | 1.083 / 1.104 / 1.063 | 31,304 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.238 | 1.222 | 1.257 | 1.241 / 1.235 / 1.231 | 14,352 |
| `comment-incident` | comments | 1124 | 1.446 | 1.423 | 1.453 | 1.448 / 1.443 / 1.438 | 1,339 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-reproduction` | comments | 298 | 0.989 | 0.978 | 0.993 | 0.990 / 0.990 / 0.988 | 64 |
| `comment-links` | comments | 278 | 0.992 | 0.984 | 1.015 | 0.990 / 0.995 / 0.987 | 87 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.993 | 0.933 | 1.037 | 0.986 / 0.993 / 1.012 | 11,201 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.993 | 0.961 | 1.010 | 1.000 / 0.968 / 1.002 | 1,890 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.995 | 0.985 | 1.004 | 0.994 / 0.995 / 0.997 | 149 |
| `comment-table` | comments | 310 | 0.996 | 0.981 | 1.013 | 0.998 / 0.997 / 0.993 | 184 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.996 | 0.976 | 1.015 | 0.998 / 0.994 / 1.001 | 7,616 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.997 | 0.980 | 1.017 | 0.988 / 0.998 / 0.999 | 316 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.997 | 0.954 | 1.002 | 0.996 / 0.999 / 0.996 | 56,581 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.998 | 0.992 | 1.013 | 0.998 / 1.001 / 0.997 | 1,103 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.998 | 0.976 | 1.011 | 0.997 / 1.001 / 0.997 | 2,540 |
| `guard-angle-link` | syntax-guard | 41 | 0.998 | 0.992 | 1.016 | 0.998 / 1.007 / 0.997 | 31 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.999 | 0.993 | 1.013 | 0.998 / 0.998 / 1.002 | 908 |
| `vite-docs-performance` | technical-docs | 8184 | 0.999 | 0.981 | 1.009 | 1.000 / 1.004 / 0.998 | 2,714 |
| `comment-question` | comments | 160 | 0.999 | 0.986 | 1.016 | 0.997 / 1.000 / 0.999 | 25 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.999 | 0.434 | 1.272 | 1.003 / 0.999 / 0.999 | 3,249 |
| `comment-incident` | comments | 1124 | 1.000 | 0.986 | 1.025 | 1.000 / 1.000 / 0.997 | 286 |
| `comment-review-long` | comments | 957 | 1.000 | 0.984 | 1.007 | 0.995 / 1.004 / 1.000 | 150 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.000 | 0.993 | 1.038 | 1.000 / 0.999 / 1.003 | 1,070 |
| `comment-ack` | comments | 37 | 1.000 | 0.989 | 1.015 | 1.000 / 0.999 / 1.001 | 19 |
| `vue-docs-slots` | technical-docs | 24211 | 1.000 | 0.985 | 1.009 | 0.996 / 1.002 / 0.999 | 9,518 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.000 | 0.980 | 1.021 | 1.001 / 1.000 / 0.998 | 536 |
| `comment-checklist` | comments | 287 | 1.000 | 0.987 | 1.009 | 1.002 / 0.994 / 1.000 | 98 |
| `comment-review` | comments | 282 | 1.000 | 0.976 | 1.015 | 1.000 / 1.001 / 1.012 | 42 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.001 | 0.993 | 1.018 | 1.002 / 0.998 / 1.003 | 1,327 |
| `legacy-contributing` | technical-docs | 9323 | 1.001 | 0.993 | 1.015 | 1.007 / 1.001 / 0.998 | 2,713 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.001 | 0.971 | 1.020 | 1.000 / 1.008 / 1.000 | 4,522 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.001 | 0.989 | 1.029 | 0.996 / 1.000 / 1.010 | 3,789 |
| `comment-unicode` | comments | 327 | 1.001 | 0.962 | 1.043 | 1.000 / 1.005 / 0.996 | 94 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.001 | 0.977 | 1.031 | 1.005 / 1.001 / 1.001 | 2,487 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.001 | 0.978 | 1.009 | 0.995 / 0.996 / 1.002 | 2,560 |
| `vite-docs-features` | technical-docs | 39739 | 1.002 | 0.961 | 1.120 | 1.031 / 1.000 / 0.994 | 17,537 |
| `comment-inline-code` | comments | 285 | 1.002 | 0.970 | 1.006 | 0.999 / 1.000 / 1.003 | 80 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.002 | 0.971 | 1.014 | 0.998 / 1.004 / 1.002 | 1,581 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.002 | 0.965 | 1.015 | 0.997 / 1.002 / 1.008 | 441 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.002 | 0.954 | 1.011 | 1.004 / 1.003 / 1.000 | 1,186 |
| `legacy-docs-readme` | readme | 1825 | 1.002 | 0.999 | 1.009 | 1.002 / 1.002 / 1.002 | 1,032 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.002 | 0.986 | 1.025 | 1.002 / 0.999 / 1.002 | 719 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.002 | 0.977 | 1.022 | 1.000 / 1.001 / 1.006 | 786 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.003 | 0.975 | 1.013 | 0.995 / 1.003 / 1.003 | 1,064 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.003 | 0.996 | 1.012 | 1.007 / 0.998 / 1.001 | 1,499 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.003 | 0.943 | 1.053 | 1.005 / 1.017 / 0.971 | 19,063 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.004 | 0.999 | 1.015 | 1.008 / 1.002 / 1.004 | 501 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.004 | 0.989 | 1.024 | 1.002 / 1.008 / 0.997 | 512 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.004 | 0.997 | 1.027 | 1.005 / 1.004 / 1.004 | 3,934 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.005 | 0.996 | 1.022 | 1.007 / 1.003 / 1.004 | 1,173 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.005 | 0.913 | 1.067 | 1.023 / 1.005 / 0.992 | 21,783 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.006 | 0.999 | 1.016 | 1.003 / 1.007 / 1.007 | 2,251 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.006 | 0.989 | 1.014 | 0.992 / 1.008 / 1.006 | 1,225 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.006 | 0.986 | 1.018 | 1.000 / 1.003 / 1.015 | 7,344 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.007 | 0.988 | 1.049 | 1.018 / 1.010 / 0.998 | 8,929 |
| `comment-quote` | comments | 290 | 1.007 | 0.992 | 1.049 | 1.027 / 1.007 / 1.001 | 51 |
| `vite-docs-api-plugin` | reference | 31890 | 1.008 | 0.987 | 1.048 | 1.008 / 1.019 / 1.005 | 12,231 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.009 | 0.986 | 1.024 | 1.011 / 1.009 / 1.009 | 3,539 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.019 | 0.966 | 1.048 | 1.021 / 1.019 / 1.015 | 15,776 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.024 | 0.974 | 1.108 | 1.049 / 1.024 / 1.002 | 37,484 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.028 | 0.873 | 1.310 | 1.004 / 1.033 / 1.022 | 19,374 |

## Authored diagnostics (45)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-empty` | link-scan-diagnostic | 0 | 0.969 | 0.956 | 1.002 | 0.969 / 0.960 / 0.972 | 85 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.969 | 0.953 | 1.006 | 0.969 / 0.969 / 0.964 | 535 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 0.987 | 0.966 | 0.998 | 0.981 / 0.986 / 0.990 | 258 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 0.989 | 0.950 | 1.015 | 0.988 / 0.989 / 1.007 | 5,397 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.990 | 0.976 | 1.016 | 0.989 / 0.986 / 0.995 | 365 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.991 | 0.978 | 1.020 | 0.984 / 0.991 / 0.995 | 268 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 0.991 | 0.968 | 1.021 | 0.996 / 0.986 / 0.992 | 304 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 0.992 | 0.945 | 1.026 | 0.996 / 0.973 / 0.992 | 1,545 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 0.993 | 0.972 | 1.029 | 0.996 / 0.982 / 0.999 | 1,007 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 0.993 | 0.977 | 1.013 | 0.989 / 1.005 / 0.990 | 298 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.994 | 0.968 | 1.006 | 0.987 / 0.991 / 0.994 | 1,623 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 0.995 | 0.968 | 1.023 | 1.000 / 0.984 / 0.995 | 8,276 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.995 | 0.958 | 1.009 | 0.996 / 0.990 / 1.002 | 4,454 |
| `table-plain-256` | table-diagnostic | 299 | 0.996 | 0.982 | 1.009 | 0.997 / 0.995 / 0.989 | 561 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 0.996 | 0.969 | 1.024 | 0.991 / 0.994 / 1.006 | 2,985 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.996 | 0.988 | 1.010 | 0.995 / 0.992 / 0.999 | 487 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 0.997 | 0.959 | 1.020 | 0.999 / 0.985 / 0.997 | 3,292 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 0.997 | 0.981 | 1.014 | 0.994 / 0.997 / 0.994 | 11,247 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 0.997 | 0.966 | 1.004 | 1.001 / 0.971 / 1.000 | 896 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.997 | 0.980 | 1.020 | 0.993 / 0.988 / 1.009 | 439 |
| `table-dense-256` | table-diagnostic | 300 | 0.997 | 0.976 | 1.025 | 0.997 / 1.009 / 0.995 | 2,413 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 0.998 | 0.993 | 1.033 | 1.000 / 0.997 / 0.999 | 2,076 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 0.999 | 0.987 | 1.015 | 1.003 / 0.993 / 0.999 | 313 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 0.999 | 0.986 | 1.015 | 0.991 / 1.001 / 1.002 | 1,001 |
| `table-formatted-256` | table-diagnostic | 278 | 0.999 | 0.991 | 1.007 | 0.999 / 0.999 / 0.996 | 2,053 |
| `scan-short-title` | link-scan-diagnostic | 11 | 0.999 | 0.970 | 1.015 | 0.990 / 1.003 / 0.999 | 252 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.999 | 0.977 | 1.019 | 0.999 / 1.002 / 0.994 | 31,547 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.999 | 0.981 | 1.028 | 0.999 / 1.001 / 0.999 | 4,081 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 0.999 | 0.977 | 1.015 | 0.998 / 1.000 / 0.999 | 984 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.000 | 0.977 | 1.028 | 0.998 / 0.998 / 1.012 | 7,558 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.000 | 0.994 | 1.004 | 0.999 / 1.001 / 1.001 | 121,234 |
| `table-sparse-256` | table-diagnostic | 297 | 1.001 | 0.976 | 1.011 | 1.005 / 1.001 / 0.990 | 591 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.001 | 0.954 | 1.009 | 1.003 / 0.999 / 1.001 | 32,495 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.001 | 0.979 | 1.015 | 0.996 / 1.010 / 1.002 | 4,696 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.002 | 0.981 | 1.029 | 1.002 / 1.008 / 1.001 | 9,763 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.002 | 0.972 | 1.020 | 0.993 / 0.997 / 1.010 | 21,556 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.002 | 0.975 | 1.021 | 1.010 / 1.003 / 1.002 | 242 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.003 | 0.975 | 1.013 | 0.992 / 1.002 / 1.011 | 27,495 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.003 | 0.989 | 1.026 | 1.000 / 1.010 / 1.000 | 105,183 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.005 | 0.990 | 1.009 | 1.005 / 1.006 / 1.000 | 12,910 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.006 | 0.987 | 1.029 | 1.007 / 1.005 / 1.002 | 3,328 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.008 | 0.994 | 1.014 | 1.008 / 1.011 / 1.003 | 14,903 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.009 | 0.975 | 1.048 | 0.997 / 1.016 / 1.010 | 1,527 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 1.010 | 0.978 | 1.014 | 1.009 / 1.006 / 1.011 | 16,921 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.156 | 1.144 | 1.172 | 1.168 / 1.151 / 1.153 | 3,862 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-empty` | link-scan-diagnostic | 0 | 0.918 | 0.914 | 0.922 | 0.918 / 0.918 / 0.919 | 21 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 0.961 | 0.956 | 0.981 | 0.974 / 0.962 / 0.959 | 139 |
| `scan-short-link` | link-scan-diagnostic | 7 | 0.961 | 0.938 | 0.976 | 0.961 / 0.960 / 0.958 | 146 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.979 | 0.973 | 0.997 | 0.984 / 0.978 / 0.978 | 429 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 0.982 | 0.968 | 0.998 | 0.975 / 0.990 / 0.982 | 100 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.983 | 0.980 | 0.995 | 0.983 / 0.985 / 0.985 | 108 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.990 | 0.957 | 1.016 | 0.995 / 0.984 / 0.986 | 267 |
| `table-plain-256` | table-diagnostic | 299 | 0.991 | 0.970 | 1.014 | 0.991 / 0.992 / 0.990 | 475 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.992 | 0.984 | 1.011 | 0.992 / 0.996 / 0.991 | 215 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 0.992 | 0.980 | 1.004 | 0.993 / 0.991 / 0.992 | 141 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 0.993 | 0.969 | 1.004 | 0.990 / 0.993 / 0.998 | 21,173 |
| `table-sparse-256` | table-diagnostic | 297 | 0.994 | 0.960 | 1.020 | 0.995 / 0.985 / 0.994 | 503 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.995 | 0.976 | 1.017 | 0.994 / 1.001 / 0.995 | 370 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 0.995 | 0.987 | 1.009 | 0.998 / 0.993 / 0.992 | 8,132 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 0.995 | 0.964 | 1.005 | 0.997 / 0.994 / 0.996 | 1,331 |
| `scan-short-title` | link-scan-diagnostic | 11 | 0.995 | 0.981 | 1.010 | 0.995 / 0.991 / 0.998 | 156 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 0.996 | 0.986 | 0.999 | 0.997 / 0.996 / 0.996 | 5,082 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.997 | 0.979 | 1.001 | 0.997 / 0.996 / 0.995 | 1,315 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 0.997 | 0.958 | 1.015 | 0.990 / 0.994 / 1.010 | 218 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 0.997 | 0.992 | 1.007 | 0.996 / 0.999 / 0.999 | 11,015 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 0.997 | 0.985 | 1.032 | 0.998 / 0.997 / 0.996 | 806 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 0.997 | 0.972 | 1.024 | 0.991 / 0.995 / 1.014 | 14,642 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 0.998 | 0.982 | 1.010 | 0.996 / 1.002 / 0.990 | 747 |
| `table-dense-256` | table-diagnostic | 300 | 0.998 | 0.976 | 1.009 | 0.997 / 1.004 / 0.998 | 2,317 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.998 | 0.984 | 1.010 | 1.000 / 0.999 / 0.994 | 4,211 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 0.998 | 0.991 | 1.012 | 0.998 / 0.997 / 0.999 | 791 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 0.999 | 0.988 | 1.020 | 1.000 / 0.999 / 0.999 | 1,953 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.999 | 0.991 | 1.029 | 0.999 / 0.998 / 1.002 | 3,850 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.000 | 0.994 | 1.004 | 1.000 / 0.999 / 0.998 | 708 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.000 | 0.973 | 1.142 | 0.996 / 0.999 / 1.000 | 121,073 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.000 | 0.979 | 1.015 | 1.001 / 1.000 / 0.991 | 4,607 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.000 | 0.987 | 1.021 | 1.000 / 0.999 / 1.001 | 31,389 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.000 | 0.985 | 1.006 | 0.997 / 1.004 / 1.000 | 7,419 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.000 | 0.988 | 1.141 | 0.995 / 1.001 / 1.000 | 3,064 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.000 | 0.979 | 1.014 | 1.002 / 0.999 / 1.010 | 3,036 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.000 | 0.995 | 1.021 | 1.000 / 1.004 / 0.998 | 1,437 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.000 | 0.981 | 1.008 | 1.000 / 1.002 / 0.999 | 32,595 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.001 | 0.985 | 1.026 | 1.001 / 0.992 / 1.014 | 107,163 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.001 | 0.995 | 1.019 | 1.001 / 1.003 / 0.998 | 2,730 |
| `table-formatted-256` | table-diagnostic | 278 | 1.001 | 0.992 | 1.018 | 1.001 / 1.004 / 1.000 | 1,979 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.003 | 0.976 | 1.016 | 1.006 / 1.003 / 1.003 | 27,435 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.004 | 0.963 | 1.031 | 0.999 / 1.004 / 1.005 | 12,745 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.005 | 0.985 | 1.014 | 0.999 / 1.005 / 1.009 | 9,563 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 1.008 | 0.983 | 1.022 | 1.007 / 1.008 / 1.010 | 16,755 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.173 | 1.142 | 1.186 | 1.180 / 1.168 / 1.177 | 3,597 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-empty` | link-scan-diagnostic | 0 | 0.908 | 0.893 | 0.937 | 0.907 / 0.910 / 0.906 | 18 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.960 | 0.948 | 0.978 | 0.960 / 0.955 / 0.971 | 45 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.960 | 0.953 | 0.978 | 0.959 / 0.964 / 0.961 | 43 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 0.965 | 0.956 | 0.973 | 0.965 / 0.965 / 0.967 | 44 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.966 | 0.961 | 0.996 | 0.964 / 0.971 / 0.966 | 50 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 0.968 | 0.941 | 0.986 | 0.974 / 0.968 / 0.963 | 43 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.971 | 0.954 | 0.994 | 0.978 / 0.962 / 0.972 | 397 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 0.973 | 0.966 | 0.987 | 0.973 / 0.981 / 0.973 | 47 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 0.974 | 0.963 | 0.994 | 0.977 / 0.982 / 0.971 | 84 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 0.977 | 0.969 | 0.992 | 0.977 / 0.977 / 0.974 | 83 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.979 | 0.966 | 1.003 | 0.984 / 0.979 / 0.977 | 85 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 0.985 | 0.966 | 0.991 | 0.983 / 0.988 / 0.986 | 81 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 0.986 | 0.974 | 0.993 | 0.985 / 0.987 / 0.985 | 81 |
| `table-sparse-256` | table-diagnostic | 297 | 0.987 | 0.958 | 1.003 | 0.982 / 0.984 / 0.990 | 436 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 0.989 | 0.980 | 0.999 | 0.992 / 0.986 / 0.989 | 86 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 0.991 | 0.978 | 1.004 | 0.990 / 0.990 / 0.992 | 212 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 0.991 | 0.964 | 1.011 | 0.994 / 0.989 / 0.998 | 176 |
| `table-plain-256` | table-diagnostic | 299 | 0.991 | 0.980 | 1.007 | 0.995 / 0.991 / 0.990 | 405 |
| `scan-short-title` | link-scan-diagnostic | 11 | 0.992 | 0.980 | 1.008 | 0.987 / 0.997 / 0.994 | 124 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 0.992 | 0.970 | 1.032 | 0.989 / 0.992 / 0.999 | 214 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 0.993 | 0.986 | 1.001 | 0.992 / 0.991 / 0.994 | 210 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 0.993 | 0.983 | 1.003 | 0.990 / 0.991 / 0.997 | 213 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 0.994 | 0.986 | 1.014 | 0.994 / 0.996 / 0.994 | 5,750 |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.995 | 0.978 | 1.008 | 0.999 / 0.998 / 0.986 | 84,780 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.996 | 0.984 | 1.014 | 0.996 / 0.996 / 0.996 | 1,062 |
| `table-formatted-256` | table-diagnostic | 278 | 0.997 | 0.983 | 1.007 | 0.995 / 1.000 / 0.999 | 1,631 |
| `table-dense-16000` | table-diagnostic | 16044 | 0.997 | 0.978 | 1.008 | 1.004 / 0.996 / 1.000 | 121,233 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 0.997 | 0.978 | 1.001 | 0.997 / 0.988 / 0.997 | 211 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 0.997 | 0.983 | 1.005 | 0.985 / 0.997 / 0.998 | 9,179 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.998 | 0.993 | 1.008 | 0.997 / 0.998 / 1.007 | 1,173 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 0.998 | 0.971 | 1.007 | 0.999 / 0.991 / 0.998 | 215 |
| `table-dense-256` | table-diagnostic | 300 | 0.998 | 0.992 | 1.041 | 0.998 / 0.995 / 0.999 | 2,253 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.998 | 0.980 | 1.015 | 1.003 / 0.997 / 0.998 | 29,779 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.999 | 0.994 | 1.036 | 1.002 / 0.998 / 0.999 | 22,100 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.999 | 0.979 | 1.011 | 1.001 / 0.988 / 0.999 | 3,425 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.000 | 0.993 | 1.037 | 0.999 / 0.994 / 1.004 | 3,564 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.000 | 0.982 | 1.028 | 0.991 / 1.005 / 1.001 | 31,782 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.000 | 0.991 | 1.018 | 0.999 / 1.002 / 1.012 | 118 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.001 | 0.980 | 1.035 | 0.997 / 1.008 / 1.001 | 8,975 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.001 | 0.986 | 1.022 | 1.016 / 0.999 / 1.001 | 5,719 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.003 | 0.993 | 1.014 | 1.004 / 1.003 / 0.995 | 3,023 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.005 | 1.000 | 1.048 | 1.003 / 1.008 / 1.009 | 10,548 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.006 | 0.988 | 1.018 | 1.004 / 1.011 / 0.998 | 12,501 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 1.009 | 0.990 | 1.021 | 1.009 / 1.009 / 1.008 | 14,754 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.230 | 1.216 | 1.245 | 1.221 / 1.233 / 1.231 | 3,105 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.973 | 0.959 | 0.980 | 0.979 / 0.966 / 0.971 | 40 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 0.991 | 0.971 | 1.016 | 0.997 / 0.989 / 0.991 | 40 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.992 | 0.977 | 1.010 | 0.995 / 0.985 / 0.993 | 263 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 0.995 | 0.979 | 1.006 | 0.997 / 0.995 / 0.991 | 1,256 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.995 | 0.965 | 1.000 | 0.997 / 0.995 / 0.994 | 170 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.996 | 0.987 | 1.010 | 0.994 / 1.000 / 0.994 | 264 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.996 | 0.983 | 1.013 | 1.000 / 0.996 / 0.990 | 320 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 0.996 | 0.969 | 1.020 | 0.978 / 1.005 / 0.997 | 2,233 |
| `scan-empty` | link-scan-diagnostic | 0 | 0.996 | 0.989 | 1.003 | 0.996 / 0.996 / 0.998 | 7 |
| `table-sparse-256` | table-diagnostic | 297 | 0.996 | 0.830 | 1.123 | 1.000 / 0.880 / 0.999 | 73 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 0.996 | 0.978 | 1.022 | 1.004 / 0.996 / 0.992 | 4,865 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.996 | 0.985 | 1.013 | 1.000 / 1.000 / 0.989 | 2,478 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 0.996 | 0.976 | 1.014 | 0.998 / 0.991 / 0.992 | 2,840 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.997 | 0.972 | 1.010 | 0.995 / 1.001 / 1.001 | 492 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 0.997 | 0.985 | 1.014 | 0.997 / 1.003 / 0.995 | 12,082 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 0.997 | 0.985 | 1.003 | 1.000 / 0.997 / 0.997 | 722 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.998 | 0.974 | 1.007 | 1.000 / 0.998 / 0.992 | 182 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.998 | 0.974 | 1.014 | 1.001 / 0.998 / 0.995 | 803 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 0.998 | 0.972 | 1.027 | 0.994 / 1.005 / 0.993 | 92 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 0.998 | 0.991 | 1.019 | 0.998 / 1.004 / 0.995 | 712 |
| `table-dense-256` | table-diagnostic | 300 | 0.999 | 0.986 | 1.008 | 0.999 / 0.995 / 0.999 | 65 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.999 | 0.986 | 1.005 | 1.000 / 0.995 / 0.999 | 165 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 0.999 | 0.986 | 1.010 | 1.000 / 0.999 / 0.998 | 2,530 |
| `scan-short-link` | link-scan-diagnostic | 7 | 0.999 | 0.973 | 1.016 | 0.998 / 0.998 / 1.001 | 29 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.999 | 0.959 | 1.014 | 1.005 / 1.003 / 0.998 | 67 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 0.999 | 0.997 | 1.025 | 1.001 / 0.999 / 0.998 | 2,402 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 1.000 | 0.980 | 1.024 | 1.003 / 1.000 / 1.000 | 60 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.000 | 0.986 | 1.044 | 1.002 / 1.000 / 1.000 | 825 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.000 | 0.989 | 1.027 | 0.998 / 1.001 / 1.000 | 630 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 1.000 | 0.987 | 1.017 | 0.997 / 0.998 / 1.006 | 2,022 |
| `table-plain-256` | table-diagnostic | 299 | 1.000 | 0.990 | 1.018 | 0.999 / 1.001 / 0.999 | 72 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.001 | 0.993 | 1.040 | 1.001 / 1.006 / 0.998 | 2,849 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.001 | 0.982 | 1.022 | 1.000 / 1.004 / 0.999 | 7,214 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.001 | 0.978 | 1.011 | 1.001 / 1.002 / 0.992 | 2,275 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.001 | 0.979 | 1.021 | 1.000 / 1.004 / 0.991 | 1,038 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.001 | 0.992 | 1.022 | 1.001 / 1.003 / 1.000 | 3,864 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.001 | 0.975 | 1.021 | 1.009 / 0.983 / 1.014 | 1,960 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 1.002 | 0.990 | 1.012 | 1.000 / 1.005 / 1.002 | 1,879 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.002 | 0.991 | 1.019 | 1.002 / 1.003 / 0.997 | 441 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.003 | 0.977 | 1.019 | 1.001 / 1.004 / 1.004 | 533 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.003 | 0.984 | 1.033 | 1.001 / 1.005 / 1.000 | 34 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.004 | 0.989 | 1.023 | 0.999 / 1.004 / 1.017 | 99 |
| `table-formatted-256` | table-diagnostic | 278 | 1.017 | 1.007 | 1.056 | 1.024 / 1.012 / 1.019 | 351 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.019 | 1.008 | 1.036 | 1.019 / 1.019 / 1.019 | 20,001 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.022 | 1.013 | 1.029 | 1.022 / 1.021 / 1.022 | 5,164 |

## A/A control (10)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.993 | 1.023 | 0.999 / 0.996 / 1.010 | 7,396 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.001 | 0.988 | 1.015 | 0.994 / 1.006 / 1.005 | 27,245 |
| `comment-incident` | comments | 1124 | 1.001 | 0.959 | 1.011 | 1.001 / 1.002 / 0.993 | 1,792 |
| `legacy-docs-readme` | readme | 1825 | 1.002 | 0.991 | 1.020 | 0.998 / 1.008 / 1.004 | 4,532 |
| `comment-ack` | comments | 37 | 1.005 | 0.996 | 1.015 | 1.006 / 1.002 / 1.005 | 143 |
| `comment-review` | comments | 282 | 1.005 | 0.987 | 1.022 | 1.006 / 1.006 / 1.005 | 229 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.006 | 1.000 | 1.025 | 1.006 / 1.009 / 1.006 | 8,314 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.009 | 0.936 | 1.052 | 0.999 / 0.996 / 1.009 | 115,815 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.022 | 1.012 | 1.038 | 1.022 / 1.021 / 1.025 | 4,624 |
| `vite-docs-features` | technical-docs | 39739 | 1.040 | 0.937 | 1.148 | 0.989 / 1.051 / 1.040 | 60,459 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.995 | 0.922 | 1.045 | 1.009 / 0.965 / 0.994 | 113,708 |
| `comment-review` | comments | 282 | 0.998 | 0.990 | 1.008 | 0.994 / 0.994 / 0.999 | 149 |
| `comment-incident` | comments | 1124 | 1.000 | 0.977 | 1.022 | 1.000 / 0.996 / 1.002 | 1,635 |
| `comment-ack` | comments | 37 | 1.001 | 0.982 | 1.011 | 1.002 / 1.002 / 0.998 | 66 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.984 | 1.014 | 1.002 / 0.999 / 1.003 | 7,150 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.001 | 0.985 | 1.021 | 1.001 / 1.006 / 1.001 | 27,018 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.002 | 0.987 | 1.017 | 1.002 / 1.002 / 0.998 | 7,885 |
| `legacy-docs-readme` | readme | 1825 | 1.004 | 0.994 | 1.015 | 1.004 / 1.011 / 1.000 | 4,389 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.023 | 1.002 | 1.032 | 1.022 / 1.029 / 1.024 | 4,538 |
| `vite-docs-features` | technical-docs | 39739 | 1.040 | 0.950 | 1.139 | 1.013 / 1.040 / 1.061 | 60,616 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.994 | 0.895 | 1.013 | 0.994 / 0.992 / 0.995 | 21,775 |
| `legacy-docs-readme` | readme | 1825 | 0.997 | 0.973 | 1.022 | 0.997 / 0.998 / 0.996 | 3,347 |
| `comment-incident` | comments | 1124 | 0.998 | 0.982 | 1.012 | 0.992 / 0.998 / 1.004 | 1,331 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.999 | 0.811 | 1.018 | 0.966 / 1.005 / 1.004 | 65,851 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.999 | 0.968 | 1.052 | 0.998 / 0.999 / 1.012 | 3,510 |
| `comment-ack` | comments | 37 | 1.000 | 0.986 | 1.030 | 0.994 / 1.000 / 1.006 | 48 |
| `comment-review` | comments | 282 | 1.001 | 0.989 | 1.011 | 1.002 / 1.001 / 0.997 | 108 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.001 | 0.982 | 1.031 | 1.005 / 1.001 / 0.999 | 6,208 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.992 | 1.016 | 1.001 / 0.998 / 1.001 | 4,566 |
| `vite-docs-features` | technical-docs | 39739 | 1.002 | 0.950 | 1.056 | 1.002 / 1.018 / 1.000 | 36,396 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-review` | comments | 282 | 0.999 | 0.949 | 1.016 | 1.006 / 0.999 / 0.988 | 43 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.000 | 0.989 | 1.008 | 1.002 / 0.998 / 0.997 | 2,509 |
| `comment-incident` | comments | 1124 | 1.000 | 0.990 | 1.016 | 0.998 / 1.007 / 0.995 | 285 |
| `comment-ack` | comments | 37 | 1.002 | 0.990 | 1.011 | 0.998 / 1.002 / 1.002 | 19 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.005 | 0.989 | 1.013 | 1.010 / 1.001 / 1.005 | 1,507 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.020 | 1.009 | 1.033 | 1.025 / 1.026 / 1.017 | 5,097 |
| `legacy-docs-readme` | readme | 1825 | 1.024 | 1.006 | 1.047 | 1.028 / 1.024 / 1.019 | 1,037 |
| `vite-docs-features` | technical-docs | 39739 | 1.031 | 1.004 | 1.179 | 1.046 / 1.020 / 1.025 | 17,607 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.100 | 1.093 | 1.114 | 1.107 / 1.097 / 1.100 | 1,028 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.147 | 1.059 | 1.224 | 1.079 / 1.150 / 1.148 | 37,922 |

