# Per-case tables

Ratios are baseline time over candidate time (median of the paired windows); higher is faster.
`rounds` lists the per-round medians. Baseline time is the median nanoseconds per document.

## A/A control

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.967 | 0.947 | 0.996 | 0.967 / 0.973 / 0.959 | 33,076 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.993 | 0.976 | 1.003 | 0.992 / 0.995 / 0.993 | 563 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.995 | 0.985 | 1.028 | 0.991 / 0.997 / 0.995 | 28,195 |
| `comment-question` | comments | 160 | 0.996 | 0.984 | 1.035 | 0.997 / 0.996 / 0.996 | 169 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.996 | 0.986 | 1.009 | 0.996 / 0.989 / 0.997 | 29,833 |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.997 | 0.986 | 1.003 | 0.992 / 0.997 / 0.998 | 115,078 |
| `comment-review-long` | comments | 957 | 0.998 | 0.983 | 1.004 | 1.004 / 0.993 / 0.999 | 763 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.998 | 0.945 | 1.006 | 1.004 / 0.998 / 0.954 | 4,254 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.999 | 0.914 | 1.027 | 1.019 / 0.999 / 0.982 | 58,204 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.000 | 0.994 | 1.037 | 1.005 / 0.996 / 1.000 | 4,711 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.000 | 0.968 | 1.019 | 1.000 / 0.999 / 1.014 | 31,275 |
| `comment-links` | comments | 278 | 1.000 | 0.985 | 1.003 | 1.001 / 0.998 / 0.997 | 477 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.986 | 1.092 | 0.999 / 1.000 / 1.041 | 5,577 |
| `table-formatted-256` | table-diagnostic | 278 | 1.001 | 0.994 | 1.004 | 1.003 / 0.998 / 0.996 | 2,160 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.001 | 0.985 | 1.015 | 0.998 / 1.000 / 1.011 | 4,508 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.970 | 1.009 | 1.004 / 0.998 / 1.001 | 7,890 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.001 | 0.994 | 1.016 | 1.001 / 0.995 / 1.011 | 3,524 |
| `table-dense-256` | table-diagnostic | 300 | 1.002 | 0.995 | 1.016 | 1.002 / 1.011 / 0.998 | 2,448 |
| `comment-quote` | comments | 290 | 1.003 | 0.992 | 1.008 | 0.998 / 1.003 / 1.006 | 292 |
| `vite-docs-performance` | technical-docs | 8184 | 1.003 | 0.988 | 1.016 | 1.003 / 1.009 / 0.988 | 9,234 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.003 | 0.991 | 1.036 | 1.026 / 1.003 / 0.999 | 55,683 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.004 | 0.993 | 1.022 | 1.006 / 0.994 / 1.002 | 1,770 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.004 | 0.989 | 1.014 | 1.004 / 1.009 / 0.993 | 6,037 |
| `comment-checklist` | comments | 287 | 1.004 | 0.980 | 1.010 | 1.006 / 0.995 / 0.993 | 587 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.004 | 0.988 | 1.024 | 1.003 / 1.005 / 0.999 | 122,120 |
| `comment-ack` | comments | 37 | 1.004 | 0.998 | 1.015 | 1.004 / 1.003 / 1.004 | 153 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.005 | 0.991 | 1.021 | 1.007 / 1.012 / 1.000 | 5,607 |
| `comment-incident` | comments | 1124 | 1.006 | 0.979 | 1.044 | 1.006 / 0.985 / 1.008 | 1,453 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.009 | 0.961 | 1.042 | 1.012 / 1.006 / 1.024 | 66,382 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.010 | 0.986 | 1.040 | 1.003 / 1.021 / 0.990 | 1,644 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.012 | 0.996 | 1.019 | 1.005 / 1.006 / 1.016 | 1,534 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.017 | 1.000 | 1.095 | 1.017 / 1.032 / 1.007 | 2,586 |
| `table-sparse-256` | table-diagnostic | 297 | 1.019 | 1.009 | 1.067 | 1.037 / 1.010 / 1.024 | 632 |
| `table-plain-256` | table-diagnostic | 299 | 1.022 | 1.011 | 1.028 | 1.025 / 1.016 / 1.022 | 577 |
| `vite-docs-api-plugin` | reference | 31890 | 1.023 | 0.984 | 1.056 | 1.042 / 1.002 / 1.035 | 71,263 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.030 | 0.853 | 1.093 | 1.030 / 1.010 / 1.091 | 53,190 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.974 | 0.957 | 0.991 | 0.974 / 0.978 / 0.967 | 33,665 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.985 | 0.962 | 1.045 | 0.989 / 0.985 / 0.985 | 67,253 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.986 | 0.972 | 0.996 | 0.975 / 0.984 / 0.991 | 28,166 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.990 | 0.980 | 1.012 | 0.990 / 0.995 / 0.986 | 3,323 |
| `vite-docs-performance` | technical-docs | 8184 | 0.993 | 0.528 | 1.013 | 1.001 / 0.897 / 1.010 | 8,948 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.994 | 0.978 | 1.038 | 1.009 / 0.989 / 0.990 | 57,373 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.996 | 0.980 | 1.014 | 1.005 / 0.999 / 0.995 | 458 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.997 | 0.990 | 1.008 | 0.997 / 1.003 / 0.995 | 29,354 |
| `comment-question` | comments | 160 | 0.997 | 0.974 | 1.079 | 0.999 / 0.991 / 1.012 | 93 |
| `comment-ack` | comments | 37 | 0.997 | 0.988 | 1.006 | 0.997 / 1.003 / 0.995 | 70 |
| `table-dense-16000` | table-diagnostic | 16044 | 0.998 | 0.991 | 1.002 | 0.998 / 0.999 / 1.002 | 120,690 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.999 | 0.990 | 1.017 | 0.999 / 0.998 / 1.004 | 7,510 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.949 | 1.029 | 1.000 / 0.996 / 1.018 | 5,167 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.000 | 0.991 | 1.018 | 0.998 / 1.016 / 0.999 | 30,999 |
| `comment-checklist` | comments | 287 | 1.001 | 0.993 | 1.015 | 1.000 / 1.006 / 1.001 | 508 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.001 | 0.593 | 1.008 | 1.001 / 1.005 / 0.611 | 3,856 |
| `table-dense-256` | table-diagnostic | 300 | 1.001 | 0.992 | 1.016 | 1.001 / 0.997 / 1.006 | 2,402 |
| `table-formatted-256` | table-diagnostic | 278 | 1.001 | 0.961 | 1.016 | 0.968 / 1.009 / 1.004 | 2,099 |
| `comment-quote` | comments | 290 | 1.002 | 0.992 | 1.033 | 1.002 / 1.026 / 0.999 | 209 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.002 | 0.991 | 1.024 | 1.002 / 1.007 / 1.001 | 1,604 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.002 | 1.000 | 1.215 | 1.002 / 1.002 / 1.133 | 4,199 |
| `comment-incident` | comments | 1124 | 1.003 | 0.960 | 1.191 | 0.998 / 1.006 / 0.969 | 1,277 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.003 | 0.937 | 1.055 | 0.989 / 1.004 / 1.003 | 114,610 |
| `comment-links` | comments | 278 | 1.004 | 0.994 | 1.021 | 0.996 / 1.004 / 1.008 | 392 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.004 | 0.709 | 1.183 | 1.002 / 1.027 / 1.011 | 5,356 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.004 | 0.993 | 1.024 | 1.005 / 1.004 / 0.999 | 1,327 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.005 | 0.985 | 1.019 | 1.015 / 1.005 / 1.003 | 1,470 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.005 | 0.957 | 1.021 | 1.011 / 0.987 / 1.004 | 5,835 |
| `comment-review-long` | comments | 957 | 1.005 | 0.988 | 1.032 | 1.005 / 1.020 / 1.010 | 702 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 0.993 | 1.008 | 0.997 / 1.007 / 1.005 | 55,406 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.007 | 0.989 | 1.012 | 0.994 / 1.009 / 1.008 | 4,536 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.015 | 1.005 | 1.020 | 1.015 / 1.005 / 1.017 | 2,301 |
| `table-plain-256` | table-diagnostic | 299 | 1.020 | 1.000 | 1.036 | 1.031 / 1.019 / 1.025 | 490 |
| `vite-docs-api-plugin` | reference | 31890 | 1.023 | 0.960 | 1.061 | 1.014 / 1.027 / 1.024 | 69,336 |
| `table-sparse-256` | table-diagnostic | 297 | 1.026 | 1.019 | 1.040 | 1.026 / 1.035 / 1.026 | 520 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.049 | 0.903 | 1.152 | 1.051 / 1.049 / 1.049 | 52,284 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.970 | 0.953 | 1.029 | 0.989 / 0.967 / 0.970 | 31,686 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.991 | 0.981 | 0.999 | 0.989 / 0.991 / 0.998 | 2,698 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.995 | 0.959 | 1.164 | 0.999 / 0.981 / 0.988 | 3,454 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.996 | 0.988 | 1.011 | 0.996 / 1.004 / 0.994 | 3,988 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.998 | 0.992 | 1.047 | 1.002 / 0.998 / 0.994 | 433 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.999 | 0.993 | 1.016 | 0.999 / 1.004 / 0.996 | 30,973 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.000 | 0.971 | 1.007 | 1.000 / 0.990 / 1.004 | 23,948 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.001 | 0.995 | 1.010 | 1.000 / 1.003 / 1.004 | 91,680 |
| `comment-review-long` | comments | 957 | 1.001 | 0.981 | 1.014 | 0.982 / 1.001 / 1.002 | 534 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.001 | 0.994 | 1.010 | 1.002 / 0.998 / 1.003 | 3,559 |
| `comment-ack` | comments | 37 | 1.002 | 0.991 | 1.017 | 0.996 / 1.002 / 1.011 | 52 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.002 | 0.995 | 1.010 | 0.998 / 1.006 / 1.002 | 2,990 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.002 | 0.952 | 1.013 | 1.002 / 1.007 / 0.990 | 46,786 |
| `comment-question` | comments | 160 | 1.002 | 0.985 | 1.008 | 1.002 / 0.989 / 1.002 | 67 |
| `comment-quote` | comments | 290 | 1.002 | 0.992 | 1.015 | 1.000 / 1.005 / 0.993 | 162 |
| `table-dense-256` | table-diagnostic | 300 | 1.002 | 0.984 | 1.011 | 1.002 / 1.002 / 0.998 | 2,265 |
| `table-formatted-256` | table-diagnostic | 278 | 1.002 | 0.996 | 1.011 | 1.002 / 1.009 / 1.000 | 1,697 |
| `comment-checklist` | comments | 287 | 1.002 | 0.987 | 1.056 | 1.002 / 1.013 / 0.991 | 424 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.003 | 0.984 | 1.018 | 1.005 / 0.998 / 1.004 | 18,206 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.004 | 0.990 | 1.015 | 1.004 / 1.004 / 1.008 | 120,406 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.004 | 0.942 | 1.008 | 1.004 / 0.993 / 1.004 | 1,054 |
| `comment-incident` | comments | 1124 | 1.005 | 0.997 | 1.024 | 1.005 / 1.005 / 0.999 | 955 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.005 | 0.992 | 1.016 | 1.005 / 1.000 / 1.010 | 3,803 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.005 | 0.974 | 1.014 | 1.009 / 1.011 / 0.986 | 35,678 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.006 | 0.994 | 1.022 | 1.006 / 1.003 / 1.006 | 30,409 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.006 | 0.980 | 1.016 | 1.005 / 1.007 / 1.014 | 1,175 |
| `vite-docs-performance` | technical-docs | 8184 | 1.006 | 0.993 | 1.018 | 1.000 / 1.006 / 1.010 | 6,002 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.007 | 1.001 | 1.015 | 1.004 / 1.009 / 1.007 | 4,963 |
| `vite-docs-api-plugin` | reference | 31890 | 1.007 | 0.976 | 1.089 | 1.007 / 1.002 / 1.023 | 51,321 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.008 | 1.001 | 1.032 | 1.008 / 1.008 / 1.013 | 3,409 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.008 | 0.999 | 1.018 | 1.003 / 1.010 / 1.008 | 1,081 |
| `comment-links` | comments | 278 | 1.009 | 0.998 | 1.015 | 1.005 / 1.011 / 1.009 | 301 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.013 | 1.005 | 1.055 | 1.013 / 1.013 / 1.013 | 1,366 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.016 | 0.979 | 1.054 | 1.009 / 1.015 / 1.044 | 38,835 |
| `table-plain-256` | table-diagnostic | 299 | 1.018 | 0.779 | 1.027 | 1.026 / 1.018 / 0.969 | 435 |
| `table-sparse-256` | table-diagnostic | 297 | 1.031 | 1.012 | 1.168 | 1.028 / 1.034 / 1.130 | 472 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.983 | 0.974 | 1.007 | 0.992 / 0.978 / 0.983 | 7,577 |
| `table-dense-256` | table-diagnostic | 300 | 0.985 | 0.982 | 0.994 | 0.985 / 0.983 / 0.985 | 64 |
| `table-plain-256` | table-diagnostic | 299 | 0.987 | 0.983 | 1.003 | 0.985 / 0.989 / 0.986 | 69 |
| `table-sparse-256` | table-diagnostic | 297 | 0.990 | 0.976 | 1.001 | 0.991 / 0.987 / 0.976 | 70 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.991 | 0.985 | 1.003 | 0.989 / 0.994 / 0.988 | 260 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.992 | 0.971 | 1.000 | 0.992 / 0.993 / 0.992 | 264 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.992 | 0.909 | 1.000 | 0.991 / 0.997 / 0.994 | 161 |
| `vite-docs-api-plugin` | reference | 31890 | 0.993 | 0.983 | 1.007 | 0.989 / 1.007 / 0.997 | 12,147 |
| `table-dense-16000` | table-diagnostic | 16044 | 0.994 | 0.987 | 1.005 | 0.995 / 0.994 / 0.994 | 437 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.996 | 0.977 | 1.005 | 1.000 / 0.987 / 1.001 | 846 |
| `vite-docs-performance` | technical-docs | 8184 | 0.997 | 0.989 | 1.005 | 0.997 / 0.999 / 0.994 | 2,687 |
| `comment-ack` | comments | 37 | 0.997 | 0.979 | 1.026 | 0.998 / 0.990 / 1.026 | 19 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.999 | 0.989 | 1.003 | 1.001 / 0.994 / 0.998 | 488 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.999 | 0.992 | 1.013 | 0.995 / 0.999 / 1.003 | 1,887 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.000 | 0.988 | 1.010 | 1.000 / 1.002 / 0.997 | 5,266 |
| `comment-incident` | comments | 1124 | 1.000 | 0.987 | 1.007 | 1.007 / 1.002 / 0.998 | 295 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.000 | 0.934 | 1.074 | 1.002 / 1.000 / 0.936 | 980 |
| `comment-quote` | comments | 290 | 1.000 | 0.995 | 1.009 | 0.999 / 1.000 / 1.000 | 49 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.000 | 0.984 | 1.006 | 1.000 / 1.001 / 0.998 | 20,530 |
| `comment-review-long` | comments | 957 | 1.000 | 0.991 | 1.009 | 1.000 / 0.992 / 1.004 | 150 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.000 | 0.996 | 1.010 | 1.000 / 1.004 / 0.998 | 1,215 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.997 | 1.021 | 1.000 / 1.000 / 1.014 | 1,598 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.000 | 0.987 | 1.019 | 1.002 / 0.988 / 0.998 | 815 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.001 | 0.991 | 1.005 | 0.998 / 1.003 / 0.999 | 910 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.001 | 0.979 | 1.029 | 1.001 / 1.003 / 0.996 | 15,831 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.991 | 1.016 | 1.001 / 1.001 / 1.004 | 2,607 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.002 | 0.977 | 1.014 | 1.010 / 1.004 / 1.000 | 2,500 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.002 | 0.985 | 1.032 | 0.999 / 1.014 / 1.012 | 41 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.002 | 0.994 | 1.025 | 1.012 / 1.003 / 0.996 | 8,914 |
| `comment-links` | comments | 278 | 1.002 | 0.968 | 1.008 | 0.999 / 1.003 / 0.998 | 87 |
| `comment-checklist` | comments | 287 | 1.002 | 0.991 | 1.004 | 0.998 / 1.002 / 1.003 | 101 |
| `comment-question` | comments | 160 | 1.003 | 0.990 | 1.021 | 1.003 / 1.008 / 1.000 | 25 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.004 | 0.967 | 1.073 | 1.026 / 1.000 / 1.004 | 18,626 |
| `table-formatted-256` | table-diagnostic | 278 | 1.005 | 0.982 | 1.009 | 1.003 / 1.005 / 1.006 | 356 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.006 | 0.974 | 1.045 | 0.989 / 1.007 / 1.006 | 569 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.013 | 0.963 | 1.062 | 1.013 / 0.977 / 1.013 | 21,566 |

## closer, screen

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.979 | 0.968 | 0.984 | 0.983 / 0.979 / 0.973 | 576 |
| `comment-quote` | comments | 290 | 0.987 | 0.982 | 0.991 | 0.985 / 0.986 / 0.987 | 286 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.988 | 0.959 | 1.021 | 0.976 / 0.986 / 1.009 | 1,468 |
| `comment-incident` | comments | 1124 | 0.989 | 0.986 | 0.996 | 0.987 / 0.992 / 0.989 | 1,363 |
| `comment-question` | comments | 160 | 0.991 | 0.984 | 0.994 | 0.991 / 0.992 / 0.990 | 164 |
| `table-plain-256` | table-diagnostic | 299 | 0.991 | 0.987 | 1.002 | 0.988 / 0.991 / 0.997 | 569 |
| `comment-ack` | comments | 37 | 0.992 | 0.970 | 1.000 | 0.992 / 0.987 / 0.994 | 144 |
| `table-sparse-256` | table-diagnostic | 297 | 0.992 | 0.988 | 0.996 | 0.991 / 0.992 / 0.994 | 599 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.996 | 0.982 | 1.035 | 0.995 / 0.996 / 1.003 | 26,887 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.996 | 0.977 | 1.012 | 1.007 / 0.998 / 0.994 | 1,588 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.997 | 0.988 | 1.008 | 0.990 / 0.994 / 1.007 | 5,246 |
| `table-formatted-256` | table-diagnostic | 278 | 0.998 | 0.985 | 1.030 | 0.998 / 0.997 / 0.998 | 2,104 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.998 | 0.993 | 1.004 | 1.001 / 0.998 / 0.997 | 30,744 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.998 | 0.986 | 1.034 | 0.987 / 1.014 / 1.012 | 4,389 |
| `table-dense-256` | table-diagnostic | 300 | 0.998 | 0.992 | 0.999 | 0.999 / 0.999 / 0.997 | 2,372 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.999 | 0.991 | 1.077 | 0.999 / 1.010 / 0.998 | 4,025 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.000 | 0.996 | 1.004 | 1.000 / 1.001 / 0.998 | 118,528 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.001 | 0.999 | 1.006 | 1.003 / 1.001 / 0.999 | 28,940 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.003 | 0.992 | 1.010 | 0.999 / 1.006 / 1.003 | 112,216 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.004 | 0.995 | 1.007 | 1.000 / 1.004 / 1.005 | 53,567 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.010 | 0.995 | 1.026 | 1.016 / 1.010 / 1.004 | 7,686 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.010 | 0.995 | 1.016 | 1.011 / 1.008 / 1.013 | 2,444 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.012 | 0.821 | 1.096 | 1.023 / 1.015 / 0.854 | 47,032 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.014 | 1.003 | 1.032 | 1.013 / 1.014 / 1.014 | 1,729 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.017 | 1.001 | 1.084 | 1.018 / 1.013 / 1.017 | 4,494 |
| `vite-docs-api-plugin` | reference | 31890 | 1.019 | 1.016 | 1.064 | 1.019 / 1.027 / 1.018 | 65,358 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.028 | 1.007 | 1.033 | 1.028 / 1.031 / 1.023 | 5,479 |
| `vite-docs-performance` | technical-docs | 8184 | 1.031 | 1.026 | 1.037 | 1.034 / 1.031 / 1.028 | 8,845 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.038 | 0.972 | 1.136 | 1.071 / 1.014 / 1.023 | 55,746 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.040 | 1.027 | 1.050 | 1.047 / 1.040 / 1.030 | 3,454 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.041 | 1.008 | 1.042 | 1.041 / 1.038 / 1.041 | 32,564 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.048 | 1.042 | 1.055 | 1.051 / 1.047 / 1.053 | 5,905 |
| `comment-review-long` | comments | 957 | 1.051 | 1.048 | 1.056 | 1.051 / 1.053 / 1.048 | 747 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.065 | 0.946 | 1.170 | 1.155 / 1.062 / 1.065 | 69,319 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.098 | 1.071 | 1.118 | 1.112 / 1.081 / 1.098 | 553 |
| `comment-links` | comments | 278 | 1.108 | 1.103 | 1.116 | 1.116 / 1.110 / 1.107 | 466 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.980 | 0.961 | 0.984 | 0.973 / 0.980 / 0.980 | 68 |
| `comment-quote` | comments | 290 | 0.982 | 0.979 | 0.991 | 0.983 / 0.983 / 0.981 | 205 |
| `comment-checklist` | comments | 287 | 0.982 | 0.971 | 0.994 | 0.980 / 0.978 / 0.987 | 496 |
| `comment-question` | comments | 160 | 0.983 | 0.977 | 0.986 | 0.978 / 0.983 / 0.983 | 88 |
| `table-sparse-256` | table-diagnostic | 297 | 0.984 | 0.983 | 0.989 | 0.986 / 0.983 / 0.984 | 513 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.985 | 0.982 | 0.997 | 0.984 / 0.985 / 0.990 | 26,344 |
| `comment-incident` | comments | 1124 | 0.986 | 0.984 | 0.994 | 0.987 / 0.984 / 0.986 | 1,212 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.993 | 0.991 | 0.996 | 0.994 / 0.993 / 0.991 | 1,421 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.993 | 0.988 | 1.002 | 0.993 / 0.990 / 0.994 | 4,969 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.993 | 0.990 | 0.996 | 0.993 / 0.993 / 0.992 | 1,299 |
| `table-plain-256` | table-diagnostic | 299 | 0.996 | 0.991 | 1.006 | 0.996 / 0.993 / 0.997 | 479 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.997 | 0.982 | 1.010 | 0.996 / 0.997 / 1.001 | 3,778 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.999 | 0.996 | 1.007 | 0.998 / 1.003 / 0.999 | 28,649 |
| `table-formatted-256` | table-diagnostic | 278 | 0.999 | 0.997 | 1.001 | 0.998 / 1.000 / 0.999 | 2,017 |
| `table-dense-256` | table-diagnostic | 300 | 0.999 | 0.994 | 1.005 | 0.998 / 1.001 / 1.005 | 2,291 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.999 | 0.991 | 1.006 | 1.000 / 0.997 / 0.999 | 30,571 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.999 | 0.994 | 1.030 | 0.999 / 1.003 / 0.999 | 4,156 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.000 | 0.997 | 1.002 | 0.999 / 1.001 / 0.999 | 118,149 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.001 | 0.998 | 1.008 | 1.002 / 1.002 / 1.000 | 110,244 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.992 | 1.036 | 1.027 / 0.993 / 1.005 | 7,314 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.003 | 0.915 | 1.172 | 1.024 / 0.946 / 0.974 | 47,976 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.004 | 1.001 | 1.010 | 1.004 / 1.004 / 1.002 | 53,194 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.011 | 1.000 | 1.018 | 1.012 / 1.007 / 1.011 | 2,259 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.015 | 1.013 | 1.045 | 1.015 / 1.023 / 1.015 | 1,562 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.016 | 1.013 | 1.020 | 1.014 / 1.017 / 1.015 | 4,409 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.018 | 1.011 | 1.028 | 1.015 / 1.017 / 1.023 | 5,100 |
| `vite-docs-api-plugin` | reference | 31890 | 1.026 | 0.979 | 1.064 | 1.016 / 1.046 / 1.026 | 65,426 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.026 | 0.928 | 1.068 | 1.046 / 0.996 / 0.969 | 64,296 |
| `vite-docs-performance` | technical-docs | 8184 | 1.031 | 0.999 | 1.052 | 1.043 / 1.032 / 1.026 | 8,590 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.039 | 1.034 | 1.043 | 1.040 / 1.039 / 1.038 | 32,398 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.040 | 1.035 | 1.086 | 1.038 / 1.082 / 1.040 | 3,158 |
| `comment-review-long` | comments | 957 | 1.047 | 1.031 | 1.063 | 1.044 / 1.047 / 1.051 | 670 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.052 | 1.045 | 1.055 | 1.052 / 1.055 / 1.045 | 5,635 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.069 | 0.998 | 1.205 | 1.074 / 1.061 / 1.054 | 56,247 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.105 | 1.100 | 1.142 | 1.114 / 1.105 / 1.100 | 445 |
| `comment-links` | comments | 278 | 1.133 | 1.119 | 1.137 | 1.125 / 1.136 / 1.135 | 381 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.971 | 0.965 | 0.982 | 0.971 / 0.969 / 0.971 | 51 |
| `comment-checklist` | comments | 287 | 0.975 | 0.951 | 1.005 | 0.994 / 0.957 / 0.975 | 412 |
| `comment-quote` | comments | 290 | 0.978 | 0.966 | 0.984 | 0.978 / 0.969 / 0.979 | 159 |
| `comment-question` | comments | 160 | 0.980 | 0.976 | 1.068 | 0.990 / 0.978 / 0.980 | 66 |
| `comment-incident` | comments | 1124 | 0.982 | 0.964 | 0.985 | 0.972 / 0.985 / 0.982 | 927 |
| `table-sparse-256` | table-diagnostic | 297 | 0.985 | 0.980 | 0.988 | 0.984 / 0.986 / 0.987 | 443 |
| `table-plain-256` | table-diagnostic | 299 | 0.988 | 0.984 | 0.992 | 0.989 / 0.989 / 0.987 | 410 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.991 | 0.988 | 0.992 | 0.989 / 0.990 / 0.992 | 1,160 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.994 | 0.983 | 1.004 | 0.995 / 0.988 / 0.994 | 17,609 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.995 | 0.986 | 1.001 | 0.995 / 0.996 / 0.994 | 3,376 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.995 | 0.993 | 1.000 | 0.999 / 0.995 / 0.996 | 1,048 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.996 | 0.994 | 1.054 | 0.999 / 0.995 / 1.000 | 3,363 |
| `table-formatted-256` | table-diagnostic | 278 | 0.997 | 0.989 | 1.002 | 0.997 / 0.997 / 0.997 | 1,670 |
| `table-dense-256` | table-diagnostic | 300 | 0.998 | 0.983 | 1.003 | 0.999 / 0.993 / 0.999 | 2,239 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.999 | 0.994 | 1.009 | 1.001 / 1.001 / 0.998 | 2,983 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.999 | 0.996 | 1.002 | 0.999 / 0.999 / 0.996 | 30,428 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.001 | 0.999 | 1.004 | 1.003 / 1.000 / 1.000 | 23,318 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.001 | 0.999 | 1.018 | 1.000 / 1.006 / 1.000 | 117,772 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.003 | 0.910 | 1.018 | 1.001 / 0.999 / 1.018 | 90,569 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.006 | 1.001 | 1.011 | 1.004 / 1.006 / 1.008 | 45,295 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.010 | 1.002 | 1.018 | 1.018 / 1.005 / 1.004 | 1,331 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.011 | 1.008 | 1.015 | 1.010 / 1.012 / 1.008 | 4,779 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.011 | 0.994 | 1.034 | 0.999 / 1.018 / 1.011 | 29,013 |
| `vite-docs-api-plugin` | reference | 31890 | 1.016 | 1.006 | 1.044 | 1.025 / 1.016 / 1.016 | 48,870 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.019 | 1.017 | 1.031 | 1.018 / 1.019 / 1.019 | 3,491 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.021 | 1.017 | 1.034 | 1.021 / 1.027 / 1.023 | 1,018 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.031 | 1.018 | 1.035 | 1.019 / 1.031 / 1.033 | 3,869 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.039 | 1.035 | 1.047 | 1.039 / 1.041 / 1.039 | 29,844 |
| `vite-docs-performance` | technical-docs | 8184 | 1.044 | 1.031 | 1.051 | 1.044 / 1.046 / 1.045 | 5,873 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.050 | 1.038 | 1.096 | 1.057 / 1.040 / 1.051 | 33,999 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.050 | 0.994 | 1.113 | 1.049 / 1.076 / 1.047 | 37,674 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.052 | 1.044 | 1.056 | 1.055 / 1.051 / 1.048 | 2,665 |
| `comment-review-long` | comments | 957 | 1.070 | 1.067 | 1.095 | 1.067 / 1.073 / 1.070 | 531 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.086 | 1.063 | 1.090 | 1.086 / 1.086 / 1.080 | 3,754 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.119 | 1.091 | 1.133 | 1.105 / 1.118 / 1.133 | 422 |
| `comment-links` | comments | 278 | 1.172 | 1.165 | 1.176 | 1.172 / 1.172 / 1.170 | 294 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `table-dense-4096` | table-diagnostic | 4140 | 0.967 | 0.963 | 0.970 | 0.964 / 0.967 / 0.969 | 155 |
| `table-dense-256` | table-diagnostic | 300 | 0.989 | 0.986 | 0.993 | 0.990 / 0.987 / 0.989 | 62 |
| `table-sparse-256` | table-diagnostic | 297 | 0.993 | 0.969 | 0.999 | 0.993 / 0.992 / 0.998 | 68 |
| `table-plain-256` | table-diagnostic | 299 | 0.993 | 0.980 | 0.995 | 0.994 / 0.991 / 0.993 | 68 |
| `vite-docs-api-plugin` | reference | 31890 | 0.994 | 0.977 | 1.003 | 0.990 / 0.992 / 0.994 | 11,973 |
| `comment-checklist` | comments | 287 | 0.995 | 0.975 | 0.999 | 0.991 / 0.995 / 0.995 | 95 |
| `comment-incident` | comments | 1124 | 0.995 | 0.987 | 1.002 | 0.993 / 0.999 / 0.997 | 279 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.995 | 0.981 | 1.008 | 0.991 / 1.001 / 0.996 | 796 |
| `vite-docs-performance` | technical-docs | 8184 | 0.996 | 0.994 | 1.005 | 0.995 / 0.996 / 0.999 | 2,650 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.996 | 0.928 | 1.052 | 1.022 / 1.007 / 0.994 | 20,568 |
| `comment-links` | comments | 278 | 0.996 | 0.984 | 1.017 | 0.996 / 0.997 / 0.988 | 86 |
| `comment-quote` | comments | 290 | 0.996 | 0.988 | 1.013 | 0.992 / 1.000 / 1.007 | 49 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.996 | 0.990 | 1.003 | 0.997 / 0.995 / 0.998 | 1,574 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.997 | 0.989 | 1.004 | 0.999 / 0.997 / 0.995 | 255 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.974 | 1.006 | 0.997 / 0.997 / 0.997 | 7,446 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.998 | 0.974 | 1.008 | 1.001 / 0.989 / 0.989 | 793 |
| `table-formatted-256` | table-diagnostic | 278 | 0.998 | 0.982 | 1.004 | 0.997 / 0.998 / 0.998 | 346 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.998 | 0.992 | 1.000 | 0.997 / 0.998 / 0.996 | 255 |
| `comment-review-long` | comments | 957 | 0.998 | 0.996 | 1.002 | 0.999 / 0.998 / 0.996 | 147 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.998 | 0.993 | 1.000 | 0.994 / 0.999 / 0.997 | 39 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.999 | 0.998 | 1.000 | 1.000 / 0.998 / 0.999 | 921 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.999 | 0.988 | 1.001 | 0.999 / 0.994 / 1.000 | 5,157 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.000 | 0.996 | 1.002 | 0.999 / 1.000 / 1.002 | 20,034 |
| `comment-ack` | comments | 37 | 1.000 | 0.997 | 1.008 | 0.999 / 1.001 / 0.998 | 19 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.000 | 0.994 | 1.007 | 1.000 / 0.998 / 1.002 | 8,654 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.001 | 0.995 | 1.011 | 1.000 / 1.001 / 1.001 | 1,853 |
| `comment-question` | comments | 160 | 1.001 | 0.992 | 1.006 | 1.000 / 1.001 / 0.998 | 24 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.001 | 0.997 | 1.017 | 1.001 / 1.003 / 0.999 | 1,193 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.001 | 0.995 | 1.026 | 1.001 / 1.014 / 1.001 | 541 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.002 | 0.976 | 1.034 | 1.008 / 1.000 / 0.999 | 429 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.003 | 0.988 | 1.012 | 1.003 / 1.003 / 1.002 | 2,429 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.004 | 0.996 | 1.008 | 1.005 / 1.005 / 1.000 | 894 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.006 | 1.001 | 1.012 | 1.008 / 1.004 / 1.006 | 476 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.009 | 0.916 | 1.018 | 1.000 / 1.009 / 1.011 | 17,589 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.009 | 1.001 | 1.076 | 1.013 / 1.002 / 1.005 | 15,499 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.010 | 0.997 | 1.022 | 0.999 / 1.020 / 1.007 | 2,544 |

## closer, broad

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.976 | 0.642 | 0.992 | 0.937 / 0.978 / 0.989 | 591 |
| `comment-quote` | comments | 290 | 0.984 | 0.976 | 1.013 | 0.980 / 0.985 / 0.988 | 289 |
| `comment-unicode` | comments | 327 | 0.986 | 0.973 | 1.045 | 0.983 / 0.986 / 0.986 | 459 |
| `comment-question` | comments | 160 | 0.989 | 0.981 | 1.000 | 0.985 / 0.989 / 0.991 | 166 |
| `comment-ack` | comments | 37 | 0.989 | 0.983 | 1.014 | 0.993 / 0.989 / 0.989 | 147 |
| `comment-table` | comments | 310 | 0.992 | 0.972 | 1.002 | 0.993 / 0.990 / 0.997 | 1,149 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.993 | 0.968 | 1.017 | 0.982 / 0.994 / 0.993 | 15,180 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.993 | 0.988 | 1.000 | 0.992 / 0.994 / 0.992 | 5,306 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.993 | 0.986 | 1.015 | 0.992 / 0.993 / 0.994 | 12,184 |
| `comment-reproduction` | comments | 298 | 0.994 | 0.860 | 1.039 | 0.903 / 0.994 / 1.005 | 323 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.994 | 0.950 | 1.193 | 0.974 / 1.001 / 0.996 | 76,988 |
| `comment-incident` | comments | 1124 | 0.995 | 0.988 | 1.002 | 0.995 / 0.997 / 0.992 | 1,383 |
| `comment-review` | comments | 282 | 0.995 | 0.973 | 1.008 | 0.986 / 0.996 / 0.994 | 231 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.986 | 1.022 | 1.003 / 0.994 / 1.000 | 27,174 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.998 | 0.992 | 1.003 | 0.997 / 0.998 / 0.998 | 9,282 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.998 | 0.991 | 1.019 | 1.004 / 0.996 / 0.996 | 3,019 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.998 | 0.766 | 1.014 | 0.999 / 0.998 / 0.996 | 8,184 |
| `comment-inline-code` | comments | 285 | 1.001 | 0.980 | 1.022 | 1.000 / 1.005 / 0.998 | 300 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 0.984 | 1.013 | 1.004 / 1.005 / 1.005 | 54,363 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.006 | 0.998 | 1.015 | 1.004 / 1.011 / 1.004 | 2,324 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.006 | 0.987 | 1.023 | 1.010 / 0.996 / 1.002 | 2,974 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.009 | 0.988 | 1.041 | 1.010 / 1.008 / 1.009 | 1,608 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.009 | 1.001 | 1.019 | 1.009 / 1.009 / 1.008 | 7,392 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.010 | 1.002 | 1.024 | 1.012 / 1.013 / 1.009 | 7,619 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.010 | 0.984 | 1.039 | 1.013 / 1.011 / 1.005 | 3,312 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.011 | 0.980 | 1.026 | 1.011 / 1.010 / 1.012 | 2,484 |
| `legacy-contributing` | technical-docs | 9323 | 1.011 | 0.999 | 1.661 | 1.028 / 1.007 / 1.027 | 10,821 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.012 | 0.918 | 1.103 | 1.015 / 1.010 / 0.999 | 1,741 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.012 | 0.938 | 1.030 | 1.012 / 1.005 / 1.019 | 7,875 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.014 | 0.994 | 1.032 | 1.017 / 1.014 / 1.017 | 9,555 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.016 | 1.000 | 1.045 | 1.009 / 1.013 / 1.020 | 14,559 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.018 | 1.011 | 1.036 | 1.016 / 1.024 / 1.020 | 10,228 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.020 | 0.962 | 1.111 | 1.017 / 1.034 / 0.992 | 48,243 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.020 | 1.013 | 1.032 | 1.024 / 1.019 / 1.020 | 1,043 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.022 | 0.992 | 1.050 | 1.017 / 1.018 / 1.039 | 3,919 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.025 | 1.004 | 1.038 | 1.033 / 1.015 / 1.029 | 1,414 |
| `vite-docs-api-plugin` | reference | 31890 | 1.025 | 0.948 | 1.171 | 1.021 / 1.088 / 1.048 | 67,661 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.026 | 1.019 | 1.031 | 1.025 / 1.026 / 1.026 | 5,556 |
| `vite-docs-performance` | technical-docs | 8184 | 1.026 | 0.989 | 1.039 | 1.030 / 1.026 / 1.026 | 8,927 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.027 | 0.745 | 2.731 | 0.845 / 1.026 / 1.028 | 1,609 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.030 | 1.010 | 1.049 | 1.032 / 1.029 / 1.031 | 2,537 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.030 | 1.014 | 1.044 | 1.025 / 1.033 / 1.026 | 3,977 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.035 | 0.999 | 1.072 | 1.032 / 1.060 / 1.042 | 56,734 |
| `vue-docs-slots` | technical-docs | 24211 | 1.037 | 0.841 | 1.102 | 1.042 / 1.036 / 1.055 | 24,664 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.037 | 0.997 | 1.050 | 1.038 / 1.034 / 1.037 | 22,514 |
| `vite-docs-features` | technical-docs | 39739 | 1.038 | 0.937 | 1.102 | 1.008 / 1.049 / 1.036 | 61,061 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.039 | 1.014 | 1.061 | 1.039 / 1.042 / 1.031 | 3,478 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.041 | 1.014 | 1.086 | 1.024 / 1.061 / 1.047 | 33,638 |
| `comment-review-long` | comments | 957 | 1.051 | 1.039 | 1.060 | 1.056 / 1.048 / 1.052 | 757 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.052 | 1.024 | 1.065 | 1.045 / 1.052 / 1.061 | 6,003 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.056 | 0.909 | 1.102 | 1.060 / 1.050 / 1.056 | 65,005 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.056 | 0.456 | 1.384 | 0.938 / 1.060 / 1.056 | 119,372 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.058 | 0.949 | 1.137 | 1.056 / 1.062 / 1.063 | 60,343 |
| `legacy-docs-readme` | readme | 1825 | 1.060 | 1.051 | 1.084 | 1.063 / 1.056 / 1.062 | 4,828 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.079 | 1.062 | 1.091 | 1.083 / 1.079 / 1.078 | 701 |
| `comment-links` | comments | 278 | 1.109 | 1.093 | 1.137 | 1.103 / 1.102 / 1.118 | 472 |
| `guard-angle-link` | syntax-guard | 41 | 1.185 | 1.159 | 1.212 | 1.185 / 1.185 / 1.186 | 273 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.974 | 0.917 | 1.005 | 0.972 / 0.978 / 0.942 | 68 |
| `comment-reproduction` | comments | 298 | 0.976 | 0.963 | 1.031 | 0.966 / 0.979 / 0.971 | 234 |
| `comment-checklist` | comments | 287 | 0.980 | 0.972 | 0.986 | 0.981 / 0.980 / 0.981 | 497 |
| `comment-quote` | comments | 290 | 0.983 | 0.967 | 0.995 | 0.979 / 0.983 / 0.983 | 206 |
| `comment-question` | comments | 160 | 0.984 | 0.973 | 1.022 | 0.980 / 0.982 / 1.011 | 90 |
| `comment-incident` | comments | 1124 | 0.985 | 0.971 | 0.995 | 0.985 / 0.984 / 0.985 | 1,233 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.989 | 0.980 | 1.002 | 0.989 / 0.981 / 0.991 | 5,008 |
| `comment-table` | comments | 310 | 0.989 | 0.926 | 1.015 | 0.987 / 0.990 / 0.990 | 1,065 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.991 | 0.984 | 0.999 | 0.989 / 0.989 / 0.996 | 14,656 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.992 | 0.975 | 1.007 | 0.995 / 0.992 / 0.994 | 11,786 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.993 | 0.976 | 1.006 | 0.992 / 0.994 / 0.994 | 2,811 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.993 | 0.966 | 1.016 | 0.994 / 0.992 / 0.993 | 7,902 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.993 | 0.983 | 1.016 | 0.993 / 0.998 / 0.991 | 27,065 |
| `comment-unicode` | comments | 327 | 0.994 | 0.885 | 1.012 | 0.999 / 0.994 / 0.994 | 374 |
| `comment-review` | comments | 282 | 0.996 | 0.937 | 1.038 | 0.997 / 0.995 / 0.997 | 153 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.979 | 1.016 | 1.004 / 0.998 / 1.003 | 9,182 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.000 | 0.977 | 1.007 | 1.000 / 1.004 / 1.000 | 77,798 |
| `comment-inline-code` | comments | 285 | 1.001 | 0.993 | 1.010 | 0.995 / 1.001 / 1.001 | 216 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.004 | 0.988 | 1.037 | 1.002 / 1.004 / 1.010 | 53,721 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.005 | 0.875 | 1.264 | 1.007 / 0.995 / 1.036 | 48,902 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.006 | 0.991 | 1.019 | 1.007 / 1.006 / 1.004 | 2,908 |
| `vite-docs-api-plugin` | reference | 31890 | 1.006 | 0.960 | 1.102 | 1.006 / 0.973 / 1.026 | 65,897 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.007 | 0.978 | 1.019 | 1.016 / 1.005 / 1.004 | 13,935 |
| `legacy-contributing` | technical-docs | 9323 | 1.007 | 0.987 | 1.018 | 1.010 / 1.003 / 1.007 | 10,396 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.009 | 0.973 | 1.022 | 1.007 / 1.014 / 1.009 | 2,150 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.010 | 1.000 | 1.069 | 1.012 / 1.006 / 1.004 | 7,466 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.010 | 0.900 | 1.724 | 1.011 / 1.010 / 1.010 | 2,793 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.013 | 1.001 | 1.030 | 1.018 / 1.013 / 1.012 | 1,576 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.013 | 1.003 | 1.025 | 1.017 / 1.010 / 1.015 | 2,281 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.014 | 0.998 | 1.043 | 1.014 / 1.018 / 1.012 | 6,896 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.018 | 1.007 | 1.049 | 1.020 / 1.018 / 1.023 | 7,653 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.019 | 1.003 | 1.036 | 1.024 / 1.015 / 1.021 | 10,058 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.020 | 0.996 | 1.045 | 1.021 / 1.018 / 1.020 | 9,194 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.020 | 1.011 | 1.040 | 1.021 / 1.019 / 1.029 | 1,454 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.022 | 1.012 | 1.028 | 1.018 / 1.023 / 1.022 | 5,132 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.022 | 1.017 | 1.037 | 1.022 / 1.020 / 1.023 | 3,709 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.022 | 0.982 | 1.038 | 1.024 / 1.020 / 1.021 | 886 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.026 | 1.014 | 1.048 | 1.026 / 1.022 / 1.027 | 1,251 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.029 | 1.017 | 1.044 | 1.031 / 1.028 / 1.022 | 3,749 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.033 | 1.024 | 1.039 | 1.032 / 1.031 / 1.036 | 2,371 |
| `vite-docs-performance` | technical-docs | 8184 | 1.035 | 0.993 | 1.088 | 1.039 / 1.034 / 1.035 | 8,859 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.036 | 0.972 | 1.054 | 1.041 / 1.040 / 1.027 | 21,891 |
| `vue-docs-slots` | technical-docs | 24211 | 1.037 | 1.015 | 1.047 | 1.040 / 1.045 / 1.029 | 23,338 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.039 | 1.028 | 1.053 | 1.042 / 1.037 / 1.039 | 3,261 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.039 | 1.024 | 1.821 | 1.035 / 1.039 / 1.063 | 1,471 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.047 | 1.021 | 1.054 | 1.047 / 1.053 / 1.046 | 5,685 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.051 | 0.950 | 1.116 | 1.051 / 1.034 / 1.067 | 57,090 |
| `comment-review-long` | comments | 957 | 1.052 | 1.035 | 1.065 | 1.054 / 1.051 / 1.048 | 679 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.058 | 0.943 | 1.123 | 1.021 / 1.059 / 1.070 | 65,655 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.058 | 0.916 | 1.102 | 1.018 / 1.050 / 1.058 | 34,123 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.058 | 0.950 | 1.143 | 1.053 / 1.034 / 1.069 | 117,688 |
| `legacy-docs-readme` | readme | 1825 | 1.066 | 1.028 | 1.127 | 1.070 / 1.066 / 1.066 | 4,589 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.066 | 0.949 | 1.141 | 1.055 / 1.088 / 1.059 | 61,006 |
| `vite-docs-features` | technical-docs | 39739 | 1.075 | 0.939 | 2.098 | 1.106 / 1.056 / 1.042 | 61,291 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.101 | 1.089 | 1.106 | 1.096 / 1.101 / 1.104 | 535 |
| `comment-links` | comments | 278 | 1.136 | 1.099 | 1.156 | 1.128 / 1.141 / 1.130 | 387 |
| `guard-angle-link` | syntax-guard | 41 | 1.281 | 1.257 | 1.316 | 1.273 / 1.281 / 1.302 | 192 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-reproduction` | comments | 298 | 0.968 | 0.950 | 0.978 | 0.968 / 0.969 / 0.962 | 172 |
| `comment-ack` | comments | 37 | 0.972 | 0.748 | 0.993 | 0.974 / 0.972 / 0.970 | 52 |
| `comment-checklist` | comments | 287 | 0.973 | 0.960 | 0.982 | 0.975 / 0.974 / 0.970 | 402 |
| `comment-quote` | comments | 290 | 0.976 | 0.962 | 0.986 | 0.974 / 0.977 / 0.976 | 160 |
| `comment-question` | comments | 160 | 0.976 | 0.967 | 0.994 | 0.976 / 0.982 / 0.976 | 66 |
| `comment-incident` | comments | 1124 | 0.985 | 0.970 | 0.992 | 0.987 / 0.983 / 0.979 | 940 |
| `comment-table` | comments | 310 | 0.986 | 0.982 | 1.018 | 0.989 / 0.985 / 0.987 | 876 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.991 | 0.500 | 1.015 | 0.997 / 0.988 / 0.992 | 3,399 |
| `comment-unicode` | comments | 327 | 0.992 | 0.981 | 0.996 | 0.992 / 0.985 / 0.994 | 276 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.994 | 0.978 | 1.004 | 0.979 / 0.997 / 0.993 | 17,824 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.995 | 0.986 | 1.007 | 0.998 / 0.994 / 0.999 | 6,231 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.995 | 0.973 | 1.000 | 0.996 / 0.990 / 0.994 | 10,168 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.997 | 0.984 | 1.019 | 1.007 / 0.995 / 0.993 | 8,053 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.998 | 0.975 | 1.025 | 0.997 / 0.998 / 0.999 | 1,756 |
| `comment-review` | comments | 282 | 0.998 | 0.987 | 1.005 | 0.998 / 0.999 / 0.997 | 112 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.999 | 0.983 | 1.022 | 0.994 / 0.997 / 1.002 | 5,800 |
| `comment-inline-code` | comments | 285 | 0.999 | 0.985 | 1.008 | 0.998 / 0.998 / 1.001 | 134 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.003 | 0.990 | 1.024 | 1.001 / 1.004 / 1.003 | 1,603 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.003 | 0.995 | 1.012 | 1.001 / 1.007 / 1.003 | 19,534 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.003 | 0.994 | 1.017 | 1.004 / 1.001 / 1.003 | 45,521 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.006 | 0.997 | 1.012 | 1.007 / 1.006 / 1.006 | 1,339 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.007 | 0.991 | 1.029 | 1.006 / 1.008 / 1.005 | 4,815 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.010 | 0.982 | 1.134 | 1.014 / 1.007 / 1.010 | 1,829 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.011 | 1.002 | 1.020 | 1.011 / 1.010 / 1.012 | 11,582 |
| `legacy-contributing` | technical-docs | 9323 | 1.013 | 1.003 | 1.065 | 1.025 / 1.013 / 1.010 | 7,560 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.014 | 0.996 | 1.048 | 1.036 / 1.011 / 1.017 | 4,435 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.014 | 0.991 | 1.040 | 1.011 / 1.020 / 1.014 | 1,360 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.020 | 1.005 | 1.049 | 1.011 / 1.023 / 1.018 | 930 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.020 | 1.004 | 1.042 | 1.013 / 1.020 / 1.024 | 2,480 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.021 | 1.006 | 1.055 | 1.019 / 1.021 / 1.021 | 1,043 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.022 | 0.968 | 1.044 | 1.011 / 1.016 / 1.025 | 29,209 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.026 | 1.016 | 1.039 | 1.029 / 1.029 / 1.020 | 6,314 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.027 | 1.015 | 1.039 | 1.022 / 1.027 / 1.032 | 5,062 |
| `vite-docs-api-plugin` | reference | 31890 | 1.029 | 1.006 | 1.059 | 1.029 / 1.024 / 1.033 | 50,192 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.030 | 1.017 | 1.064 | 1.026 / 1.036 / 1.030 | 3,901 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.030 | 0.368 | 1.328 | 0.968 / 1.030 / 1.032 | 35,678 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.033 | 1.014 | 1.096 | 1.031 / 1.033 / 1.034 | 5,099 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.034 | 1.008 | 1.059 | 1.034 / 1.036 / 1.039 | 1,644 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.035 | 1.027 | 1.055 | 1.035 / 1.038 / 1.033 | 565 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.036 | 1.012 | 1.062 | 1.031 / 1.036 / 1.039 | 2,404 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.041 | 1.014 | 1.064 | 1.033 / 1.057 / 1.041 | 798 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.045 | 1.014 | 1.055 | 1.045 / 1.041 / 1.047 | 14,279 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.048 | 0.903 | 1.078 | 1.058 / 1.048 / 1.046 | 955 |
| `vite-docs-performance` | technical-docs | 8184 | 1.049 | 1.025 | 1.056 | 1.052 / 1.045 / 1.047 | 5,920 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.054 | 1.032 | 1.094 | 1.057 / 1.067 / 1.044 | 34,394 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.056 | 1.035 | 1.067 | 1.053 / 1.056 / 1.065 | 2,069 |
| `vue-docs-slots` | technical-docs | 24211 | 1.059 | 1.047 | 1.073 | 1.057 / 1.067 / 1.060 | 13,361 |
| `vite-docs-features` | technical-docs | 39739 | 1.059 | 1.021 | 1.157 | 1.080 / 1.064 / 1.055 | 38,403 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.061 | 1.046 | 1.081 | 1.049 / 1.072 / 1.060 | 20,741 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.061 | 1.031 | 1.127 | 1.081 / 1.056 / 1.061 | 39,988 |
| `comment-review-long` | comments | 957 | 1.065 | 1.041 | 1.083 | 1.064 / 1.055 / 1.067 | 525 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.077 | 1.045 | 1.119 | 1.085 / 1.074 / 1.092 | 69,802 |
| `legacy-docs-readme` | readme | 1825 | 1.079 | 1.073 | 1.125 | 1.090 / 1.078 / 1.079 | 3,512 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.084 | 1.070 | 1.118 | 1.092 / 1.087 / 1.079 | 3,808 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.139 | 1.099 | 1.187 | 1.162 / 1.137 / 1.143 | 377 |
| `comment-links` | comments | 278 | 1.171 | 1.158 | 1.212 | 1.209 / 1.170 / 1.168 | 297 |
| `guard-angle-link` | syntax-guard | 41 | 1.342 | 1.308 | 1.374 | 1.347 / 1.330 / 1.350 | 155 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.986 | 0.889 | 1.101 | 1.008 / 1.009 / 0.975 | 33,643 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.991 | 0.976 | 1.000 | 0.991 / 0.984 / 0.998 | 7,514 |
| `comment-reproduction` | comments | 298 | 0.991 | 0.985 | 1.008 | 0.989 / 0.991 / 0.998 | 64 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.992 | 0.896 | 1.069 | 0.992 / 0.985 / 0.997 | 8,740 |
| `comment-checklist` | comments | 287 | 0.993 | 0.970 | 1.015 | 0.994 / 0.993 / 0.987 | 97 |
| `legacy-docs-readme` | readme | 1825 | 0.993 | 0.985 | 1.008 | 0.993 / 0.993 / 0.996 | 1,027 |
| `comment-table` | comments | 310 | 0.994 | 0.979 | 1.012 | 0.998 / 0.991 / 0.992 | 183 |
| `vite-docs-api-plugin` | reference | 31890 | 0.994 | 0.974 | 1.025 | 0.995 / 0.993 / 1.001 | 12,186 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.994 | 0.694 | 1.017 | 0.942 / 0.998 / 0.995 | 1,518 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.994 | 0.982 | 1.003 | 0.995 / 0.988 / 0.998 | 56,438 |
| `comment-links` | comments | 278 | 0.995 | 0.965 | 1.007 | 0.996 / 0.990 / 0.996 | 88 |
| `comment-unicode` | comments | 327 | 0.995 | 0.902 | 1.118 | 1.012 / 0.981 / 0.994 | 96 |
| `comment-incident` | comments | 1124 | 0.995 | 0.948 | 1.018 | 0.995 / 1.000 / 0.991 | 283 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.995 | 0.916 | 1.054 | 0.995 / 0.995 / 0.997 | 1,589 |
| `legacy-contributing` | technical-docs | 9323 | 0.995 | 0.981 | 1.010 | 1.004 / 0.995 / 0.995 | 2,717 |
| `comment-question` | comments | 160 | 0.996 | 0.982 | 1.007 | 1.004 / 0.995 / 0.991 | 25 |
| `comment-review-long` | comments | 957 | 0.996 | 0.988 | 1.010 | 0.996 / 0.996 / 0.997 | 149 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.996 | 0.976 | 1.005 | 0.993 / 0.995 / 1.003 | 1,109 |
| `vite-docs-performance` | technical-docs | 8184 | 0.996 | 0.984 | 1.011 | 1.004 / 0.995 / 1.000 | 2,697 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.996 | 0.957 | 1.001 | 0.994 / 0.998 / 0.997 | 1,054 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.997 | 0.975 | 1.007 | 0.994 / 0.997 / 0.998 | 1,197 |
| `guard-angle-link` | syntax-guard | 41 | 0.997 | 0.626 | 1.011 | 0.980 / 0.999 / 0.997 | 31 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.997 | 0.974 | 1.007 | 0.998 / 0.992 / 0.998 | 2,277 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.998 | 0.984 | 1.002 | 1.000 / 0.992 / 0.999 | 1,200 |
| `comment-ack` | comments | 37 | 0.998 | 0.954 | 1.018 | 0.992 / 0.998 / 1.001 | 19 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.998 | 0.986 | 1.002 | 0.996 / 0.998 / 1.000 | 2,433 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.998 | 0.986 | 1.017 | 1.000 / 1.000 / 0.994 | 1,873 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.984 | 1.008 | 0.997 / 0.997 / 1.005 | 3,759 |
| `comment-inline-code` | comments | 285 | 0.999 | 0.964 | 1.020 | 1.000 / 0.983 / 0.999 | 84 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.999 | 0.988 | 1.026 | 0.998 / 1.002 / 0.999 | 3,502 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.000 | 0.442 | 1.006 | 0.987 / 1.001 / 0.998 | 1,157 |
| `vue-docs-slots` | technical-docs | 24211 | 1.000 | 0.989 | 1.010 | 0.997 / 1.000 / 1.000 | 9,379 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.986 | 1.014 | 0.999 / 1.000 / 1.003 | 3,242 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.000 | 0.975 | 1.013 | 1.000 / 1.002 / 0.997 | 715 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.000 | 0.984 | 1.012 | 1.000 / 1.004 / 1.000 | 2,515 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.000 | 0.567 | 1.010 | 0.978 / 1.004 / 0.999 | 3,985 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.000 | 0.980 | 1.007 | 1.002 / 1.000 / 0.995 | 903 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.001 | 0.980 | 1.047 | 1.006 / 0.995 / 1.001 | 7,274 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.001 | 0.635 | 1.204 | 1.007 / 0.991 / 1.002 | 1,100 |
| `comment-quote` | comments | 290 | 1.001 | 0.874 | 1.013 | 0.959 / 1.005 / 1.005 | 49 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.001 | 0.998 | 1.008 | 1.001 / 1.003 / 1.001 | 315 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.001 | 0.989 | 1.029 | 1.001 / 1.009 / 0.997 | 15,605 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.002 | 0.998 | 1.022 | 1.005 / 1.001 / 1.002 | 546 |
| `comment-review` | comments | 282 | 1.002 | 0.990 | 1.010 | 1.002 / 1.000 / 1.005 | 43 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.002 | 0.984 | 1.034 | 0.999 / 1.003 / 1.002 | 786 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.002 | 0.976 | 1.066 | 1.030 / 1.000 / 1.004 | 444 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.003 | 0.991 | 1.013 | 1.003 / 1.003 / 1.002 | 512 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.003 | 0.975 | 1.016 | 1.003 / 1.004 / 0.999 | 2,551 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.003 | 0.990 | 1.026 | 1.003 / 0.997 / 1.005 | 152 |
| `vite-docs-features` | technical-docs | 39739 | 1.003 | 0.967 | 1.062 | 1.005 / 1.003 / 0.992 | 17,184 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.003 | 0.984 | 1.022 | 1.004 / 1.005 / 1.001 | 496 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.003 | 0.877 | 1.110 | 1.003 / 0.974 / 1.006 | 21,279 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.005 | 0.948 | 1.032 | 1.009 / 1.005 / 1.003 | 1,333 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.007 | 0.988 | 1.019 | 1.009 / 1.009 / 1.003 | 4,582 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.012 | 0.961 | 1.051 | 1.011 / 1.012 / 1.016 | 11,224 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.019 | 0.940 | 1.058 | 1.016 / 1.026 / 1.033 | 18,202 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.023 | 0.992 | 1.120 | 1.081 / 1.018 / 1.012 | 18,003 |

## depth, broad

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.981 | 0.870 | 1.090 | 1.030 / 0.926 / 0.981 | 62,194 |
| `comment-inline-code` | comments | 285 | 0.994 | 0.975 | 1.015 | 0.994 / 0.987 / 1.010 | 302 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.995 | 0.961 | 1.029 | 0.993 / 1.007 / 0.992 | 22,577 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.996 | 0.921 | 1.171 | 0.995 / 0.964 / 1.046 | 50,686 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.998 | 0.713 | 1.005 | 1.000 / 0.982 / 0.998 | 1,618 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.999 | 0.986 | 1.024 | 1.000 / 0.998 / 0.999 | 76,811 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.999 | 0.951 | 1.039 | 0.988 / 0.995 / 1.003 | 9,647 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.999 | 0.986 | 1.019 | 0.995 / 0.994 / 1.002 | 5,414 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.000 | 0.988 | 1.011 | 1.000 / 1.002 / 0.995 | 14,450 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.988 | 1.008 | 0.999 / 1.001 / 1.001 | 9,191 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.000 | 0.991 | 1.004 | 1.002 / 0.999 / 1.000 | 3,899 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.000 | 0.910 | 1.046 | 0.994 / 1.025 / 0.996 | 65,187 |
| `comment-incident` | comments | 1124 | 1.000 | 0.980 | 1.772 | 0.995 / 1.002 / 1.000 | 1,387 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.000 | 0.992 | 1.010 | 0.998 / 1.001 / 1.007 | 12,290 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.001 | 0.979 | 1.023 | 1.001 / 1.010 / 0.996 | 15,259 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.001 | 0.922 | 1.106 | 1.005 / 0.955 / 0.993 | 34,951 |
| `vite-docs-api-plugin` | reference | 31890 | 1.001 | 0.947 | 1.068 | 1.001 / 1.000 / 1.011 | 67,396 |
| `legacy-contributing` | technical-docs | 9323 | 1.001 | 0.977 | 1.020 | 1.002 / 0.990 / 0.997 | 10,763 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.001 | 0.991 | 1.009 | 1.002 / 0.998 / 1.004 | 7,512 |
| `vite-docs-performance` | technical-docs | 8184 | 1.001 | 0.969 | 1.013 | 0.999 / 0.994 / 1.004 | 9,060 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.002 | 0.975 | 1.033 | 1.001 / 1.003 / 1.009 | 3,485 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.002 | 0.972 | 1.009 | 1.004 / 1.005 / 0.994 | 7,833 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.002 | 0.975 | 1.014 | 1.002 / 1.006 / 0.998 | 2,956 |
| `comment-reproduction` | comments | 298 | 1.002 | 0.996 | 1.009 | 1.001 / 1.005 / 1.007 | 322 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.003 | 0.986 | 1.018 | 1.003 / 1.008 / 1.002 | 1,758 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.003 | 0.987 | 1.014 | 0.993 / 1.001 / 1.004 | 6,073 |
| `comment-checklist` | comments | 287 | 1.003 | 0.991 | 1.020 | 1.003 / 1.008 / 1.003 | 593 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.003 | 0.985 | 1.030 | 1.019 / 0.999 / 1.006 | 55,275 |
| `vue-docs-slots` | technical-docs | 24211 | 1.003 | 0.984 | 1.013 | 1.001 / 1.004 / 1.003 | 23,589 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.003 | 0.994 | 1.020 | 1.003 / 1.005 / 0.999 | 10,428 |
| `comment-unicode` | comments | 327 | 1.004 | 0.995 | 1.025 | 1.005 / 0.997 / 1.008 | 465 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.005 | 0.995 | 1.033 | 1.000 / 1.013 / 1.005 | 4,007 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.005 | 0.994 | 1.009 | 1.005 / 1.006 / 1.003 | 1,622 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.005 | 0.973 | 1.025 | 1.001 / 1.003 / 1.011 | 5,692 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.005 | 0.965 | 1.080 | 0.994 / 1.002 / 1.009 | 27,770 |
| `comment-question` | comments | 160 | 1.005 | 0.996 | 1.029 | 1.022 / 1.005 / 1.003 | 169 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.006 | 0.999 | 1.019 | 1.006 / 1.007 / 1.002 | 2,590 |
| `comment-quote` | comments | 290 | 1.006 | 0.987 | 1.024 | 1.007 / 1.014 / 0.993 | 293 |
| `comment-ack` | comments | 37 | 1.006 | 1.001 | 1.016 | 1.005 / 1.006 / 1.008 | 146 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.006 | 0.754 | 1.016 | 0.997 / 1.006 / 1.007 | 1,431 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.007 | 1.002 | 1.020 | 1.007 / 1.011 / 1.007 | 2,500 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.008 | 0.982 | 1.016 | 1.009 / 0.998 / 1.008 | 3,045 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.008 | 0.997 | 1.014 | 1.008 / 1.007 / 1.007 | 8,184 |
| `comment-links` | comments | 278 | 1.008 | 0.987 | 1.015 | 1.005 / 1.007 / 1.012 | 477 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.008 | 0.997 | 1.015 | 1.008 / 1.009 / 1.006 | 1,058 |
| `legacy-docs-readme` | readme | 1825 | 1.008 | 0.996 | 1.018 | 1.005 / 1.010 / 1.008 | 4,770 |
| `comment-review-long` | comments | 957 | 1.009 | 1.001 | 1.022 | 1.013 / 1.002 / 1.009 | 771 |
| `comment-review` | comments | 282 | 1.009 | 1.001 | 1.019 | 1.012 / 1.017 / 1.006 | 234 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.009 | 0.981 | 1.036 | 1.008 / 1.009 / 1.011 | 7,788 |
| `guard-angle-link` | syntax-guard | 41 | 1.010 | 1.003 | 1.023 | 1.009 / 1.017 / 1.009 | 269 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.011 | 1.001 | 1.022 | 1.007 / 1.011 / 1.012 | 706 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.012 | 0.993 | 1.023 | 1.012 / 1.013 / 1.012 | 2,333 |
| `comment-table` | comments | 310 | 1.015 | 1.005 | 1.020 | 1.011 / 1.013 / 1.017 | 1,142 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.018 | 1.009 | 1.045 | 1.011 / 1.020 / 1.030 | 3,223 |
| `vite-docs-features` | technical-docs | 39739 | 1.027 | 0.909 | 1.148 | 1.027 / 1.018 / 1.050 | 65,871 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.032 | 0.942 | 1.086 | 0.995 / 1.036 / 1.033 | 120,881 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.035 | 0.952 | 1.155 | 1.021 / 1.035 / 1.048 | 57,982 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-reproduction` | comments | 298 | 0.991 | 0.987 | 0.995 | 0.990 / 0.992 / 0.991 | 232 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.995 | 0.893 | 1.044 | 0.995 / 1.014 / 0.983 | 33,577 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.995 | 0.981 | 1.015 | 1.000 / 0.991 / 0.994 | 5,096 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.997 | 0.979 | 1.014 | 0.994 / 1.000 / 0.994 | 1,470 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.997 | 0.951 | 1.192 | 1.004 / 0.980 / 1.035 | 50,088 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.999 | 0.988 | 1.013 | 1.005 / 0.997 / 0.997 | 9,053 |
| `vite-docs-api-plugin` | reference | 31890 | 0.999 | 0.926 | 1.105 | 0.986 / 0.999 / 1.024 | 67,656 |
| `vite-docs-performance` | technical-docs | 8184 | 0.999 | 0.983 | 1.015 | 0.994 / 0.997 / 1.002 | 8,806 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.999 | 0.984 | 1.012 | 1.003 / 1.000 / 0.998 | 77,836 |
| `comment-incident` | comments | 1124 | 0.999 | 0.991 | 1.018 | 1.000 / 0.999 / 0.997 | 1,249 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.999 | 0.985 | 1.011 | 0.995 / 1.003 / 1.001 | 2,408 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.000 | 0.986 | 1.012 | 1.003 / 1.000 / 0.995 | 5,758 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.000 | 0.974 | 1.011 | 0.995 / 1.003 / 1.002 | 11,975 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.001 | 0.994 | 1.012 | 1.005 / 1.001 / 0.998 | 3,718 |
| `comment-checklist` | comments | 287 | 1.001 | 0.967 | 1.024 | 1.004 / 0.996 / 0.995 | 505 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.002 | 0.977 | 1.031 | 1.013 / 1.003 / 0.993 | 15,005 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.002 | 0.992 | 1.015 | 1.000 / 1.004 / 1.001 | 6,966 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.002 | 0.967 | 1.013 | 1.008 / 0.999 / 1.005 | 9,243 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.003 | 0.993 | 1.014 | 1.002 / 1.005 / 1.003 | 2,861 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.003 | 0.984 | 1.007 | 1.003 / 1.003 / 1.002 | 10,087 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.003 | 0.987 | 1.011 | 1.005 / 0.993 / 1.008 | 3,269 |
| `comment-unicode` | comments | 327 | 1.003 | 0.987 | 1.014 | 1.006 / 1.005 / 1.002 | 377 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.004 | 0.993 | 1.012 | 1.002 / 1.001 / 1.006 | 3,780 |
| `legacy-contributing` | technical-docs | 9323 | 1.004 | 0.988 | 1.010 | 0.993 / 1.004 / 1.004 | 10,513 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.004 | 0.991 | 1.043 | 1.006 / 1.004 / 1.004 | 14,198 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.004 | 0.990 | 1.022 | 1.005 / 1.012 / 1.004 | 2,788 |
| `comment-review-long` | comments | 957 | 1.004 | 0.989 | 1.022 | 1.003 / 1.004 / 1.006 | 675 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.005 | 0.999 | 1.018 | 1.005 / 1.000 / 1.017 | 5,141 |
| `comment-links` | comments | 278 | 1.005 | 0.993 | 1.016 | 1.005 / 1.005 / 1.010 | 384 |
| `comment-quote` | comments | 290 | 1.005 | 0.974 | 1.106 | 1.005 / 1.004 / 1.005 | 211 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 0.983 | 1.036 | 1.006 / 1.003 / 1.006 | 54,503 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.005 | 0.996 | 1.014 | 1.004 / 1.010 / 1.005 | 7,580 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.005 | 0.982 | 1.032 | 1.007 / 1.003 / 0.991 | 27,185 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.006 | 0.978 | 1.025 | 0.989 / 1.005 / 1.012 | 1,265 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.007 | 0.999 | 1.110 | 1.007 / 1.004 / 1.008 | 2,324 |
| `comment-ack` | comments | 37 | 1.008 | 1.000 | 1.014 | 1.008 / 1.003 / 1.008 | 70 |
| `vue-docs-slots` | technical-docs | 24211 | 1.008 | 0.981 | 1.029 | 1.009 / 0.999 / 1.008 | 23,499 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.008 | 0.971 | 2.746 | 0.991 / 1.111 / 1.007 | 22,397 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.008 | 0.961 | 1.041 | 1.008 / 1.006 / 1.009 | 7,934 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.009 | 1.001 | 1.020 | 1.005 / 1.011 / 1.009 | 1,438 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.009 | 0.942 | 1.096 | 1.009 / 1.012 / 0.974 | 60,665 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.010 | 1.000 | 1.033 | 1.007 / 1.010 / 1.027 | 2,899 |
| `comment-inline-code` | comments | 285 | 1.010 | 0.993 | 1.033 | 0.995 / 1.010 / 1.018 | 219 |
| `comment-question` | comments | 160 | 1.011 | 0.999 | 1.161 | 1.009 / 1.017 / 1.012 | 91 |
| `legacy-docs-readme` | readme | 1825 | 1.011 | 1.007 | 1.030 | 1.010 / 1.011 / 1.016 | 4,549 |
| `guard-angle-link` | syntax-guard | 41 | 1.013 | 0.986 | 1.029 | 1.014 / 1.013 / 1.010 | 186 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.013 | 0.923 | 1.167 | 1.016 / 1.013 / 0.973 | 56,252 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.014 | 1.001 | 1.024 | 1.010 / 1.014 / 1.018 | 7,526 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.015 | 0.998 | 1.042 | 1.015 / 1.015 / 1.006 | 1,593 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.015 | 1.005 | 1.028 | 1.010 / 1.015 / 1.018 | 2,191 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.015 | 1.010 | 1.026 | 1.019 / 1.019 / 1.014 | 545 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.016 | 0.961 | 1.087 | 1.016 / 1.016 / 1.031 | 65,328 |
| `comment-table` | comments | 310 | 1.017 | 1.005 | 1.225 | 1.025 / 1.014 / 1.017 | 1,083 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.019 | 1.003 | 1.044 | 1.019 / 1.013 / 1.020 | 895 |
| `comment-review` | comments | 282 | 1.020 | 1.003 | 1.035 | 1.020 / 1.013 / 1.027 | 153 |
| `vite-docs-features` | technical-docs | 39739 | 1.031 | 0.886 | 1.117 | 0.965 / 1.027 / 1.057 | 63,614 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.036 | 0.979 | 1.110 | 1.031 / 1.061 / 1.029 | 124,463 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-reproduction` | comments | 298 | 0.983 | 0.961 | 1.014 | 0.984 / 0.982 / 0.992 | 173 |
| `vite-docs-features` | technical-docs | 39739 | 0.996 | 0.900 | 1.119 | 0.996 / 1.000 / 0.989 | 38,903 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.996 | 0.982 | 1.012 | 0.996 / 0.995 / 0.996 | 3,439 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.998 | 0.983 | 1.031 | 0.997 / 0.995 / 0.999 | 19,786 |
| `comment-incident` | comments | 1124 | 0.999 | 0.985 | 1.029 | 0.993 / 1.007 / 0.988 | 955 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.000 | 0.987 | 1.008 | 1.003 / 1.000 / 0.992 | 5,043 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.000 | 0.982 | 1.012 | 1.006 / 1.002 / 0.996 | 11,738 |
| `comment-checklist` | comments | 287 | 1.000 | 0.994 | 1.015 | 1.006 / 1.000 / 0.995 | 407 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.001 | 0.992 | 1.009 | 1.002 / 0.997 / 1.001 | 4,466 |
| `vite-docs-performance` | technical-docs | 8184 | 1.001 | 0.991 | 1.019 | 0.998 / 1.001 / 1.005 | 6,064 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.003 | 0.976 | 1.015 | 1.003 / 1.005 / 1.002 | 2,474 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.003 | 0.993 | 1.024 | 1.003 / 1.014 / 0.999 | 6,375 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.004 | 0.971 | 1.018 | 1.008 / 0.997 / 1.013 | 18,197 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.004 | 0.994 | 1.014 | 1.001 / 1.002 / 1.005 | 956 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.004 | 0.971 | 1.039 | 1.008 / 1.009 / 0.988 | 29,261 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.004 | 0.974 | 1.012 | 1.006 / 1.004 / 1.001 | 46,454 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.005 | 0.996 | 1.019 | 1.004 / 1.008 / 1.001 | 8,159 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.005 | 0.990 | 1.017 | 1.008 / 1.005 / 0.998 | 1,338 |
| `legacy-contributing` | technical-docs | 9323 | 1.005 | 0.990 | 1.013 | 1.004 / 1.005 / 1.009 | 7,765 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.006 | 0.996 | 1.035 | 1.005 / 1.003 / 1.011 | 10,317 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.006 | 1.001 | 1.032 | 1.008 / 1.016 / 1.003 | 1,637 |
| `vue-docs-slots` | technical-docs | 24211 | 1.006 | 0.986 | 1.013 | 1.006 / 1.008 / 0.996 | 13,339 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.006 | 0.906 | 1.070 | 1.008 / 1.014 / 1.000 | 1,781 |
| `comment-unicode` | comments | 327 | 1.006 | 0.998 | 1.020 | 1.006 / 1.009 / 1.006 | 283 |
| `comment-review-long` | comments | 957 | 1.006 | 0.996 | 1.014 | 1.005 / 1.002 / 1.010 | 523 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.006 | 0.994 | 1.028 | 1.001 / 1.009 / 1.009 | 5,842 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.008 | 0.998 | 1.020 | 1.006 / 1.010 / 1.008 | 2,395 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.008 | 0.981 | 1.040 | 1.022 / 1.014 / 1.003 | 21,062 |
| `vite-docs-api-plugin` | reference | 31890 | 1.008 | 0.971 | 1.030 | 1.008 / 1.011 / 1.008 | 51,491 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.008 | 0.997 | 1.018 | 1.008 / 1.008 / 1.009 | 6,292 |
| `comment-quote` | comments | 290 | 1.009 | 0.991 | 1.020 | 1.016 / 1.009 / 1.009 | 163 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.009 | 0.961 | 1.014 | 1.010 / 1.007 / 1.010 | 2,071 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.009 | 0.986 | 1.013 | 1.009 / 1.010 / 1.005 | 5,116 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.009 | 0.984 | 1.017 | 1.009 / 1.010 / 1.009 | 1,664 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.010 | 1.000 | 1.022 | 1.008 / 1.010 / 1.011 | 3,971 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.010 | 0.961 | 1.137 | 1.003 / 1.087 / 0.988 | 36,491 |
| `comment-ack` | comments | 37 | 1.011 | 1.004 | 1.016 | 1.011 / 1.010 / 1.013 | 52 |
| `comment-question` | comments | 160 | 1.012 | 1.005 | 2.340 | 1.012 / 1.010 / 1.012 | 66 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.012 | 0.998 | 1.025 | 1.012 / 1.021 / 1.010 | 3,848 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.012 | 0.994 | 1.034 | 1.012 / 1.017 / 1.010 | 14,540 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.012 | 0.995 | 1.025 | 1.012 / 1.010 / 1.014 | 802 |
| `comment-links` | comments | 278 | 1.012 | 1.000 | 1.024 | 1.017 / 1.012 / 1.008 | 295 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.013 | 1.000 | 1.019 | 1.015 / 1.013 / 1.013 | 927 |
| `guard-angle-link` | syntax-guard | 41 | 1.013 | 1.004 | 1.027 | 1.014 / 1.013 / 1.012 | 154 |
| `legacy-docs-readme` | readme | 1825 | 1.014 | 1.002 | 1.019 | 1.012 / 1.015 / 1.014 | 3,544 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.014 | 1.002 | 1.026 | 1.008 / 1.019 / 1.015 | 1,023 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.015 | 1.002 | 1.032 | 1.014 / 1.015 / 1.015 | 577 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.016 | 1.006 | 1.021 | 1.016 / 1.016 / 1.017 | 376 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.018 | 0.936 | 1.092 | 1.010 / 1.010 / 1.033 | 39,677 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.018 | 1.009 | 1.041 | 1.017 / 1.015 / 1.020 | 1,841 |
| `comment-inline-code` | comments | 285 | 1.018 | 1.008 | 1.027 | 1.020 / 1.017 / 1.018 | 134 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.019 | 1.002 | 1.028 | 1.021 / 1.017 / 1.018 | 4,887 |
| `comment-table` | comments | 310 | 1.020 | 1.007 | 1.043 | 1.027 / 1.015 / 1.024 | 895 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.021 | 1.002 | 1.029 | 1.016 / 1.018 / 1.023 | 1,368 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.027 | 0.976 | 1.064 | 1.032 / 1.024 / 1.036 | 34,995 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.027 | 0.979 | 1.075 | 1.025 / 1.003 / 1.033 | 70,659 |
| `comment-review` | comments | 282 | 1.029 | 1.018 | 1.034 | 1.029 / 1.028 / 1.032 | 112 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.989 | 0.980 | 0.997 | 0.988 / 0.989 / 0.996 | 7,592 |
| `comment-table` | comments | 310 | 0.991 | 0.976 | 1.009 | 0.988 / 0.992 / 0.979 | 183 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.991 | 0.850 | 1.046 | 0.992 / 0.991 / 0.977 | 21,805 |
| `vite-docs-api-plugin` | reference | 31890 | 0.994 | 0.976 | 1.012 | 0.991 / 0.987 / 1.003 | 12,297 |
| `comment-links` | comments | 278 | 0.994 | 0.983 | 1.115 | 0.994 / 0.993 / 0.997 | 88 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.995 | 0.981 | 1.060 | 0.989 / 1.000 / 0.988 | 1,131 |
| `comment-incident` | comments | 1124 | 0.996 | 0.989 | 1.013 | 0.997 / 0.994 / 0.996 | 286 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.997 | 0.981 | 1.012 | 1.007 / 0.989 / 0.997 | 3,831 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.997 | 0.991 | 1.028 | 0.994 / 1.004 / 0.996 | 4,025 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.998 | 0.980 | 1.007 | 0.996 / 0.998 / 1.001 | 2,540 |
| `comment-unicode` | comments | 327 | 0.998 | 0.977 | 1.003 | 0.999 / 0.995 / 0.998 | 95 |
| `guard-angle-link` | syntax-guard | 41 | 0.998 | 0.744 | 1.339 | 0.995 / 1.009 / 0.998 | 32 |
| `legacy-docs-readme` | readme | 1825 | 0.998 | 0.986 | 1.008 | 0.992 / 1.005 / 0.998 | 1,053 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.998 | 0.976 | 1.007 | 1.003 / 0.998 / 0.994 | 2,312 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.998 | 0.990 | 1.009 | 1.001 / 0.999 / 0.998 | 495 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.998 | 0.985 | 1.011 | 0.998 / 1.002 / 0.998 | 1,506 |
| `comment-review` | comments | 282 | 0.998 | 0.977 | 1.054 | 1.000 / 0.996 / 1.001 | 43 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.999 | 0.986 | 1.013 | 1.000 / 0.999 / 0.994 | 3,485 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.999 | 0.993 | 1.022 | 1.002 / 0.995 / 0.999 | 915 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.999 | 0.994 | 1.016 | 0.999 / 0.997 / 1.001 | 2,491 |
| `vue-docs-slots` | technical-docs | 24211 | 0.999 | 0.973 | 1.009 | 0.998 / 1.004 / 0.999 | 9,551 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.999 | 0.980 | 1.012 | 0.999 / 1.000 / 0.996 | 1,164 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.999 | 0.983 | 1.043 | 1.001 / 0.999 / 0.997 | 1,345 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.999 | 0.971 | 1.012 | 0.998 / 1.001 / 1.002 | 8,725 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.999 | 0.979 | 1.026 | 0.992 / 1.004 / 1.001 | 722 |
| `comment-review-long` | comments | 957 | 1.000 | 0.988 | 1.006 | 1.000 / 1.000 / 0.999 | 151 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.000 | 0.989 | 1.007 | 1.000 / 1.001 / 0.998 | 320 |
| `vite-docs-performance` | technical-docs | 8184 | 1.000 | 0.984 | 1.014 | 1.006 / 1.005 / 0.997 | 2,729 |
| `comment-ack` | comments | 37 | 1.000 | 0.993 | 1.011 | 1.002 / 1.000 / 1.002 | 19 |
| `comment-reproduction` | comments | 298 | 1.000 | 0.992 | 1.015 | 1.000 / 1.000 / 1.004 | 64 |
| `comment-checklist` | comments | 287 | 1.000 | 0.975 | 1.007 | 1.003 / 0.998 / 0.996 | 98 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.995 | 1.006 | 0.999 / 1.002 / 1.000 | 1,617 |
| `comment-inline-code` | comments | 285 | 1.001 | 0.989 | 1.021 | 1.008 / 1.000 / 0.996 | 84 |
| `comment-question` | comments | 160 | 1.001 | 0.994 | 1.011 | 1.001 / 0.996 / 1.004 | 25 |
| `legacy-contributing` | technical-docs | 9323 | 1.001 | 0.994 | 1.027 | 1.001 / 1.001 / 1.000 | 2,742 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.001 | 0.988 | 1.009 | 1.001 / 1.001 / 1.002 | 1,223 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.001 | 0.989 | 1.048 | 0.996 / 1.007 / 1.001 | 57,776 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.001 | 0.985 | 1.019 | 0.998 / 1.010 / 1.001 | 15,950 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.001 | 0.996 | 1.009 | 0.996 / 1.004 / 1.001 | 154 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.002 | 0.986 | 1.012 | 1.000 / 1.003 / 1.002 | 553 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.002 | 0.987 | 1.014 | 1.003 / 0.996 / 0.998 | 442 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.002 | 0.996 | 1.021 | 1.000 / 1.002 / 1.005 | 1,195 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.003 | 0.993 | 1.014 | 0.999 / 1.006 / 1.002 | 1,087 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.003 | 0.980 | 1.010 | 0.996 / 1.007 / 1.003 | 1,073 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.003 | 0.980 | 1.024 | 0.993 / 1.007 / 1.003 | 521 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.004 | 0.988 | 1.032 | 1.004 / 0.998 / 1.018 | 2,598 |
| `comment-quote` | comments | 290 | 1.004 | 0.994 | 1.020 | 1.007 / 1.003 / 1.000 | 50 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.004 | 0.991 | 1.015 | 1.001 / 1.008 / 0.996 | 7,339 |
| `vite-docs-features` | technical-docs | 39739 | 1.004 | 0.947 | 1.096 | 0.995 / 1.062 / 1.004 | 17,514 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.004 | 0.993 | 1.026 | 1.001 / 1.005 / 1.004 | 1,902 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.004 | 0.529 | 1.039 | 1.004 / 0.999 / 1.007 | 3,299 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.005 | 0.984 | 1.011 | 1.005 / 1.002 / 1.006 | 795 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.005 | 0.977 | 1.047 | 0.997 / 1.017 / 1.006 | 11,213 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.006 | 0.976 | 1.020 | 1.004 / 0.997 / 1.010 | 4,639 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.008 | 0.983 | 1.034 | 1.008 / 1.018 / 0.999 | 17,833 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.014 | 0.973 | 1.111 | 0.997 / 1.014 / 1.026 | 18,436 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.024 | 0.885 | 1.078 | 1.002 / 1.025 / 1.015 | 34,130 |

## lines, broad

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-quote` | comments | 290 | 0.970 | 0.952 | 0.979 | 0.970 / 0.971 / 0.970 | 294 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.985 | 0.949 | 1.064 | 0.997 / 0.977 / 1.008 | 118,694 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.991 | 0.912 | 1.047 | 0.982 / 1.016 / 0.991 | 34,567 |
| `comment-reproduction` | comments | 298 | 0.992 | 0.975 | 1.000 | 0.994 / 0.985 / 0.994 | 323 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.994 | 0.981 | 1.259 | 0.997 / 0.995 / 0.989 | 9,402 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.994 | 0.981 | 1.017 | 1.008 / 0.987 / 0.995 | 2,337 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.995 | 0.980 | 1.008 | 0.991 / 0.998 / 0.991 | 7,555 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.996 | 0.987 | 1.009 | 0.992 / 0.990 / 0.997 | 3,063 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.997 | 0.985 | 1.015 | 0.997 / 1.002 / 0.992 | 15,323 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.997 | 0.970 | 1.018 | 0.998 / 0.998 / 0.991 | 27,760 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.997 | 0.992 | 1.014 | 0.997 / 1.005 / 0.997 | 14,330 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.999 | 0.979 | 1.008 | 0.997 / 0.994 / 1.002 | 3,939 |
| `comment-review` | comments | 282 | 0.999 | 0.992 | 1.004 | 0.999 / 0.996 / 0.999 | 235 |
| `comment-question` | comments | 160 | 0.999 | 0.979 | 1.008 | 0.999 / 1.000 / 0.995 | 170 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.000 | 0.986 | 1.006 | 1.000 / 0.997 / 1.001 | 78,536 |
| `comment-ack` | comments | 37 | 1.000 | 0.982 | 1.026 | 1.002 / 0.999 / 1.000 | 150 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.001 | 0.989 | 1.017 | 1.001 / 0.999 / 1.003 | 1,653 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.001 | 0.989 | 1.016 | 1.002 / 0.991 / 1.001 | 12,298 |
| `guard-angle-link` | syntax-guard | 41 | 1.001 | 0.982 | 1.011 | 0.985 / 1.003 / 1.003 | 268 |
| `comment-table` | comments | 310 | 1.001 | 0.993 | 1.033 | 0.999 / 1.000 / 1.010 | 1,162 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.001 | 0.956 | 1.023 | 1.001 / 1.005 / 0.998 | 1,422 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.002 | 0.993 | 1.027 | 1.000 / 1.001 / 1.004 | 4,050 |
| `vue-docs-slots` | technical-docs | 24211 | 1.002 | 0.993 | 1.018 | 1.011 / 0.999 / 0.999 | 23,824 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.002 | 0.910 | 1.068 | 0.999 / 0.997 / 1.024 | 59,349 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.003 | 0.977 | 1.024 | 0.989 / 1.021 / 1.003 | 7,875 |
| `comment-incident` | comments | 1124 | 1.004 | 0.975 | 1.059 | 1.012 / 1.000 / 1.005 | 1,409 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.004 | 0.977 | 1.022 | 1.007 / 0.994 / 1.015 | 22,848 |
| `comment-links` | comments | 278 | 1.004 | 0.980 | 1.014 | 0.999 / 1.006 / 1.004 | 479 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.004 | 0.999 | 1.022 | 1.013 / 1.007 / 1.003 | 1,062 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.005 | 0.987 | 1.024 | 1.010 / 1.004 / 1.004 | 2,990 |
| `legacy-docs-readme` | readme | 1825 | 1.005 | 0.996 | 1.015 | 1.002 / 1.012 / 1.005 | 4,792 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.005 | 0.996 | 1.021 | 1.005 / 1.008 / 1.005 | 3,212 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.005 | 0.992 | 1.023 | 0.996 / 1.007 / 1.007 | 1,632 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.006 | 0.996 | 1.014 | 1.007 / 1.005 / 1.004 | 2,498 |
| `comment-inline-code` | comments | 285 | 1.006 | 0.993 | 1.018 | 1.002 / 1.006 / 1.006 | 302 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.006 | 0.990 | 1.036 | 1.008 / 1.006 / 1.012 | 55,263 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.007 | 0.997 | 1.023 | 1.007 / 0.999 / 1.009 | 8,366 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.007 | 1.003 | 1.022 | 1.005 / 1.011 / 1.006 | 7,991 |
| `vite-docs-features` | technical-docs | 39739 | 1.007 | 0.893 | 1.122 | 1.007 / 0.996 / 1.037 | 61,690 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.007 | 0.991 | 1.035 | 1.007 / 1.010 / 1.007 | 1,766 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.008 | 0.991 | 1.021 | 1.006 / 1.018 / 1.006 | 10,478 |
| `comment-unicode` | comments | 327 | 1.008 | 0.992 | 1.023 | 1.008 / 1.004 / 1.013 | 467 |
| `legacy-contributing` | technical-docs | 9323 | 1.009 | 0.943 | 1.033 | 0.999 / 1.017 / 1.008 | 11,024 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.010 | 0.998 | 1.023 | 1.011 / 1.005 / 1.006 | 3,497 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.010 | 0.993 | 1.035 | 1.012 / 1.011 / 1.008 | 9,674 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.010 | 1.003 | 1.022 | 1.010 / 1.013 / 1.007 | 5,360 |
| `comment-review-long` | comments | 957 | 1.010 | 0.993 | 2.010 | 1.011 / 1.010 / 1.004 | 763 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.011 | 0.997 | 1.019 | 1.017 / 1.014 / 1.010 | 711 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.011 | 0.946 | 1.160 | 1.025 / 1.006 / 0.973 | 67,627 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.012 | 1.004 | 1.025 | 1.015 / 1.012 / 1.010 | 2,590 |
| `vite-docs-performance` | technical-docs | 8184 | 1.013 | 0.994 | 1.038 | 1.021 / 1.009 / 1.013 | 9,127 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.019 | 0.949 | 1.212 | 1.026 / 1.005 / 1.027 | 51,832 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.022 | 1.016 | 1.042 | 1.021 / 1.022 / 1.030 | 6,087 |
| `comment-checklist` | comments | 287 | 1.023 | 0.984 | 1.033 | 1.023 / 1.020 / 1.023 | 594 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.024 | 1.011 | 1.036 | 1.034 / 1.021 / 1.024 | 5,580 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.042 | 0.937 | 1.194 | 1.053 / 1.042 / 1.035 | 64,538 |
| `vite-docs-api-plugin` | reference | 31890 | 1.059 | 0.921 | 1.132 | 1.069 / 1.059 / 1.056 | 68,993 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-quote` | comments | 290 | 0.960 | 0.949 | 0.981 | 0.958 / 0.965 / 0.960 | 211 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.973 | 0.907 | 1.019 | 0.962 / 0.962 / 1.013 | 33,721 |
| `comment-reproduction` | comments | 298 | 0.976 | 0.971 | 1.008 | 0.983 / 0.974 / 0.975 | 237 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.979 | 0.840 | 1.080 | 1.025 / 0.979 / 0.925 | 60,524 |
| `comment-inline-code` | comments | 285 | 0.988 | 0.912 | 0.999 | 0.985 / 0.995 / 0.987 | 219 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.993 | 0.942 | 1.101 | 1.015 / 0.981 / 0.993 | 49,437 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.994 | 0.978 | 1.011 | 1.003 / 0.990 / 0.996 | 9,228 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.996 | 0.969 | 1.017 | 0.999 / 0.996 / 0.979 | 27,319 |
| `comment-table` | comments | 310 | 0.996 | 0.969 | 1.014 | 0.997 / 0.996 / 0.993 | 1,078 |
| `comment-ack` | comments | 37 | 0.996 | 0.988 | 1.023 | 0.999 / 0.995 / 0.996 | 69 |
| `comment-links` | comments | 278 | 0.996 | 0.971 | 1.018 | 1.008 / 0.975 / 0.998 | 393 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.997 | 0.976 | 1.008 | 0.998 / 1.002 / 0.982 | 14,937 |
| `comment-incident` | comments | 1124 | 0.998 | 0.954 | 1.013 | 0.998 / 0.993 / 0.998 | 1,254 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.987 | 1.004 | 0.993 / 0.998 / 1.000 | 11,867 |
| `comment-unicode` | comments | 327 | 0.999 | 0.984 | 1.021 | 1.000 / 0.996 / 0.999 | 378 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.999 | 0.990 | 1.043 | 0.996 / 1.000 / 1.005 | 2,884 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.999 | 0.929 | 1.059 | 1.031 / 0.999 / 0.987 | 119,944 |
| `guard-angle-link` | syntax-guard | 41 | 0.999 | 0.982 | 1.011 | 1.000 / 0.996 / 0.999 | 186 |
| `comment-review` | comments | 282 | 0.999 | 0.987 | 1.006 | 0.999 / 1.000 / 0.998 | 155 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.000 | 0.984 | 1.019 | 1.001 / 0.994 / 0.998 | 14,298 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.001 | 0.989 | 1.013 | 0.999 / 1.002 / 1.003 | 77,706 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.001 | 0.974 | 1.016 | 1.001 / 1.005 / 0.994 | 7,482 |
| `comment-question` | comments | 160 | 1.001 | 0.991 | 1.009 | 1.000 / 1.001 / 1.002 | 91 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.001 | 0.990 | 1.012 | 1.007 / 1.001 / 0.996 | 1,463 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.002 | 0.994 | 1.027 | 1.001 / 1.007 / 0.998 | 3,815 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.002 | 0.991 | 1.019 | 1.000 / 1.012 / 1.002 | 54,857 |
| `vue-docs-slots` | technical-docs | 24211 | 1.002 | 0.988 | 1.022 | 1.002 / 1.004 / 1.002 | 23,183 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.002 | 0.979 | 1.032 | 1.000 / 0.999 / 1.008 | 2,957 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.002 | 0.984 | 1.024 | 1.000 / 0.999 / 1.004 | 7,182 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.003 | 0.996 | 1.020 | 1.006 / 0.999 / 1.004 | 1,601 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.003 | 0.979 | 1.021 | 1.005 / 1.003 / 1.002 | 7,930 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.004 | 0.968 | 1.058 | 1.002 / 1.015 / 0.984 | 22,277 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.004 | 0.996 | 1.028 | 1.000 / 1.004 / 1.009 | 2,190 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.004 | 0.991 | 1.010 | 0.999 / 1.006 / 1.008 | 10,085 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.005 | 0.990 | 1.011 | 1.002 / 1.009 / 1.005 | 7,720 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.005 | 0.998 | 1.016 | 1.007 / 1.002 / 1.005 | 1,479 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.005 | 0.996 | 1.022 | 1.001 / 1.003 / 1.008 | 899 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.006 | 0.919 | 1.075 | 1.010 / 0.999 / 0.977 | 58,880 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.006 | 0.984 | 1.017 | 1.003 / 1.008 / 1.013 | 2,322 |
| `legacy-docs-readme` | readme | 1825 | 1.007 | 0.999 | 1.017 | 1.010 / 1.007 / 1.006 | 4,649 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.007 | 0.993 | 1.015 | 1.007 / 1.007 / 1.006 | 3,776 |
| `legacy-contributing` | technical-docs | 9323 | 1.007 | 0.994 | 1.025 | 1.009 / 1.007 / 1.005 | 10,542 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.007 | 0.995 | 1.016 | 1.013 / 1.007 / 1.004 | 5,150 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.007 | 0.994 | 1.031 | 1.006 / 1.015 / 1.005 | 2,817 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.007 | 0.915 | 1.067 | 0.999 / 1.016 / 1.007 | 67,027 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.008 | 0.983 | 1.019 | 1.007 / 1.001 / 1.010 | 1,275 |
| `comment-review-long` | comments | 957 | 1.008 | 0.994 | 1.027 | 1.000 / 1.004 / 1.019 | 685 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.008 | 0.967 | 1.043 | 1.007 / 1.008 / 1.009 | 543 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.009 | 1.002 | 1.015 | 1.009 / 1.011 / 1.008 | 3,327 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.012 | 0.996 | 1.015 | 1.008 / 1.009 / 1.013 | 9,183 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.013 | 0.995 | 1.039 | 1.029 / 1.013 / 1.011 | 2,421 |
| `vite-docs-performance` | technical-docs | 8184 | 1.013 | 1.005 | 1.030 | 1.020 / 1.014 / 1.012 | 8,837 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.017 | 1.001 | 1.030 | 1.019 / 1.010 / 1.009 | 5,776 |
| `comment-checklist` | comments | 287 | 1.020 | 1.003 | 1.035 | 1.013 / 1.020 / 1.024 | 507 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.028 | 1.019 | 1.045 | 1.026 / 1.028 / 1.030 | 5,281 |
| `vite-docs-features` | technical-docs | 39739 | 1.037 | 0.926 | 1.115 | 1.013 / 1.057 / 1.027 | 66,985 |
| `vite-docs-api-plugin` | reference | 31890 | 1.048 | 0.848 | 1.225 | 1.048 / 1.044 / 1.072 | 70,230 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-quote` | comments | 290 | 0.949 | 0.941 | 0.966 | 0.950 / 0.947 / 0.954 | 162 |
| `comment-reproduction` | comments | 298 | 0.974 | 0.934 | 1.033 | 0.974 / 0.964 / 0.975 | 177 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.977 | 0.943 | 1.025 | 0.978 / 0.976 / 0.977 | 34,849 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.987 | 0.974 | 0.998 | 0.990 / 0.981 / 0.987 | 5,815 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.991 | 0.980 | 1.005 | 0.991 / 0.990 / 0.996 | 1,769 |
| `comment-question` | comments | 160 | 0.995 | 0.963 | 1.002 | 0.997 / 0.995 / 0.997 | 67 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.996 | 0.991 | 1.004 | 0.996 / 0.999 / 0.994 | 934 |
| `comment-ack` | comments | 37 | 0.996 | 0.982 | 1.011 | 0.997 / 0.995 / 0.996 | 52 |
| `guard-angle-link` | syntax-guard | 41 | 0.997 | 0.987 | 1.013 | 0.996 / 0.992 / 1.003 | 156 |
| `comment-unicode` | comments | 327 | 0.997 | 0.988 | 1.000 | 0.996 / 1.000 / 0.998 | 282 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.998 | 0.988 | 1.013 | 0.998 / 0.997 / 1.009 | 1,627 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.998 | 0.630 | 2.493 | 0.998 / 0.994 / 1.001 | 1,852 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.998 | 0.976 | 1.014 | 0.998 / 0.988 / 1.004 | 1,379 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.989 | 1.012 | 0.998 / 1.004 / 0.997 | 18,093 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.999 | 0.988 | 1.016 | 0.996 / 1.006 / 0.999 | 11,765 |
| `comment-table` | comments | 310 | 0.999 | 0.979 | 1.007 | 0.994 / 1.000 / 1.000 | 893 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.000 | 0.944 | 1.048 | 0.999 / 1.010 / 0.999 | 2,454 |
| `comment-inline-code` | comments | 285 | 1.001 | 0.985 | 1.020 | 0.996 / 1.002 / 1.002 | 137 |
| `comment-incident` | comments | 1124 | 1.001 | 0.983 | 1.017 | 1.000 / 1.001 / 1.005 | 955 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.001 | 0.990 | 1.024 | 1.004 / 0.999 / 0.999 | 6,390 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.001 | 0.983 | 1.016 | 0.992 / 1.001 / 1.003 | 1,038 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.002 | 0.951 | 1.049 | 1.012 / 0.987 / 0.990 | 69,960 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.002 | 0.994 | 1.014 | 0.999 / 1.006 / 1.002 | 47,053 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.002 | 0.995 | 1.009 | 1.001 / 1.002 / 1.000 | 19,779 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.003 | 0.978 | 1.021 | 1.006 / 0.995 / 1.001 | 21,513 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.004 | 0.994 | 1.039 | 1.004 / 1.006 / 1.003 | 573 |
| `vite-docs-features` | technical-docs | 39739 | 1.004 | 0.888 | 1.083 | 1.004 / 1.031 / 0.983 | 38,312 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.005 | 0.950 | 1.024 | 1.015 / 1.007 / 0.993 | 14,558 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.005 | 0.982 | 1.014 | 1.005 / 1.005 / 1.001 | 4,872 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.005 | 0.992 | 1.016 | 1.005 / 1.008 / 1.002 | 4,479 |
| `comment-review` | comments | 282 | 1.005 | 0.996 | 1.027 | 1.005 / 1.003 / 1.006 | 114 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.006 | 0.989 | 1.028 | 1.011 / 0.998 / 1.006 | 8,200 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.006 | 0.994 | 1.042 | 1.005 / 1.001 / 1.016 | 10,293 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.006 | 0.986 | 1.035 | 1.006 / 1.009 / 0.998 | 6,482 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.006 | 0.996 | 1.040 | 1.009 / 1.006 / 1.005 | 809 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.007 | 0.994 | 1.022 | 1.001 / 1.009 / 1.007 | 2,521 |
| `legacy-docs-readme` | readme | 1825 | 1.008 | 0.984 | 1.030 | 1.007 / 1.013 / 1.008 | 3,557 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.009 | 1.000 | 1.020 | 1.007 / 1.010 / 1.009 | 959 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.009 | 1.000 | 1.036 | 1.006 / 1.009 / 1.010 | 5,151 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.010 | 0.995 | 1.017 | 1.008 / 1.000 / 1.017 | 1,370 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.011 | 1.003 | 1.032 | 1.009 / 1.012 / 1.011 | 5,100 |
| `legacy-contributing` | technical-docs | 9323 | 1.011 | 0.996 | 1.022 | 1.006 / 1.007 / 1.012 | 7,695 |
| `vue-docs-slots` | technical-docs | 24211 | 1.012 | 0.990 | 1.019 | 1.009 / 1.012 / 1.012 | 13,677 |
| `comment-review-long` | comments | 957 | 1.013 | 1.001 | 1.022 | 1.010 / 1.015 / 1.013 | 530 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.014 | 1.002 | 1.046 | 1.013 / 1.028 / 1.005 | 1,671 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.014 | 1.005 | 1.022 | 1.012 / 1.015 / 1.017 | 3,461 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.014 | 0.982 | 1.057 | 1.014 / 1.014 / 1.030 | 30,113 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.015 | 1.002 | 1.027 | 1.015 / 1.018 / 1.014 | 2,112 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.015 | 0.952 | 1.057 | 0.994 / 1.041 / 1.011 | 39,182 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.016 | 0.944 | 1.185 | 1.033 / 0.991 / 1.030 | 35,742 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.016 | 1.004 | 1.038 | 1.021 / 1.015 / 1.024 | 378 |
| `comment-links` | comments | 278 | 1.017 | 1.011 | 1.028 | 1.017 / 1.016 / 1.018 | 300 |
| `vite-docs-performance` | technical-docs | 8184 | 1.020 | 1.008 | 1.031 | 1.021 / 1.022 / 1.015 | 6,022 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.026 | 1.015 | 1.048 | 1.027 / 1.024 / 1.029 | 3,880 |
| `comment-checklist` | comments | 287 | 1.030 | 1.018 | 1.041 | 1.034 / 1.029 / 1.027 | 410 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.039 | 1.012 | 1.053 | 1.037 / 1.041 / 1.039 | 3,986 |
| `vite-docs-api-plugin` | reference | 31890 | 1.084 | 1.034 | 1.135 | 1.072 / 1.087 / 1.084 | 50,497 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `vite-docs-features` | technical-docs | 39739 | 0.976 | 0.950 | 1.055 | 0.956 / 0.976 / 0.986 | 17,526 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.984 | 0.882 | 1.124 | 1.004 / 0.963 / 0.976 | 21,131 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.988 | 0.974 | 1.006 | 0.983 / 0.985 / 0.997 | 7,581 |
| `comment-table` | comments | 310 | 0.991 | 0.983 | 1.009 | 0.994 / 0.992 / 0.989 | 186 |
| `comment-links` | comments | 278 | 0.994 | 0.987 | 1.010 | 0.992 / 0.993 / 0.997 | 89 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.995 | 0.987 | 1.002 | 0.995 / 0.996 / 0.990 | 3,537 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.996 | 0.900 | 1.046 | 0.997 / 0.996 / 0.987 | 4,059 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.996 | 0.973 | 1.023 | 0.995 / 0.998 / 0.983 | 1,525 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.996 | 0.974 | 1.008 | 0.991 / 0.995 / 1.003 | 1,134 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.996 | 0.992 | 1.008 | 0.996 / 0.998 / 0.995 | 57,587 |
| `vite-docs-api-plugin` | reference | 31890 | 0.997 | 0.979 | 1.019 | 0.993 / 0.996 / 1.001 | 12,356 |
| `vue-docs-slots` | technical-docs | 24211 | 0.997 | 0.946 | 1.030 | 1.003 / 0.984 / 0.996 | 9,632 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.997 | 0.985 | 1.006 | 1.005 / 0.995 / 0.988 | 2,314 |
| `comment-quote` | comments | 290 | 0.997 | 0.989 | 1.011 | 0.999 / 0.997 / 0.991 | 50 |
| `legacy-docs-readme` | readme | 1825 | 0.997 | 0.988 | 1.010 | 0.998 / 0.995 / 1.002 | 1,054 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.998 | 0.980 | 1.022 | 0.998 / 0.995 / 1.005 | 2,600 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.998 | 0.986 | 1.003 | 0.996 / 0.998 / 0.999 | 2,540 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.999 | 0.986 | 1.020 | 0.999 / 0.997 / 1.004 | 1,183 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.999 | 0.988 | 1.009 | 0.998 / 0.996 / 1.001 | 721 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.999 | 0.990 | 1.008 | 1.001 / 1.005 / 0.997 | 1,910 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.999 | 0.987 | 1.008 | 0.997 / 0.997 / 1.002 | 2,484 |
| `legacy-contributing` | technical-docs | 9323 | 0.999 | 0.981 | 1.013 | 1.007 / 1.001 / 0.995 | 2,746 |
| `comment-review` | comments | 282 | 0.999 | 0.993 | 1.004 | 1.001 / 1.000 / 0.996 | 43 |
| `comment-inline-code` | comments | 285 | 0.999 | 0.989 | 1.011 | 1.001 / 0.995 / 1.004 | 84 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.999 | 0.988 | 1.008 | 1.002 / 0.994 / 1.002 | 1,075 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.000 | 0.992 | 1.014 | 0.999 / 1.008 / 1.000 | 3,831 |
| `comment-ack` | comments | 37 | 1.000 | 0.987 | 1.009 | 1.000 / 1.001 / 0.998 | 20 |
| `comment-question` | comments | 160 | 1.000 | 0.992 | 1.018 | 1.016 / 1.000 / 1.000 | 25 |
| `comment-unicode` | comments | 327 | 1.000 | 0.984 | 1.007 | 0.997 / 1.001 / 1.001 | 97 |
| `comment-review-long` | comments | 957 | 1.001 | 0.984 | 1.007 | 1.001 / 0.998 / 1.004 | 151 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.001 | 0.991 | 1.027 | 0.997 / 1.012 / 1.001 | 8,904 |
| `comment-checklist` | comments | 287 | 1.001 | 0.987 | 1.008 | 1.000 / 1.004 / 0.998 | 98 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.001 | 0.969 | 1.010 | 1.000 / 1.002 / 1.001 | 504 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.001 | 0.993 | 1.011 | 0.995 / 1.005 / 1.005 | 3,330 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.001 | 0.995 | 1.013 | 0.999 / 1.001 / 1.003 | 902 |
| `guard-angle-link` | syntax-guard | 41 | 1.001 | 0.993 | 1.018 | 0.999 / 1.009 / 0.999 | 32 |
| `comment-reproduction` | comments | 298 | 1.001 | 0.982 | 1.025 | 1.001 / 0.998 / 1.002 | 64 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.001 | 0.989 | 1.018 | 0.993 / 1.005 / 1.000 | 1,197 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.002 | 0.994 | 1.010 | 0.998 / 1.002 / 1.005 | 553 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.002 | 0.987 | 1.021 | 0.999 / 1.003 / 1.005 | 4,617 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.002 | 0.994 | 1.039 | 1.009 / 1.000 / 1.002 | 1,629 |
| `vite-docs-performance` | technical-docs | 8184 | 1.002 | 0.988 | 1.026 | 0.997 / 1.002 / 1.002 | 2,733 |
| `comment-incident` | comments | 1124 | 1.002 | 0.952 | 1.093 | 1.003 / 0.999 / 1.011 | 287 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.002 | 0.994 | 1.015 | 1.002 / 0.999 / 1.006 | 523 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.002 | 0.986 | 1.335 | 1.002 / 1.001 / 1.003 | 791 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.003 | 0.990 | 1.019 | 1.004 / 1.003 / 1.001 | 1,222 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.004 | 0.994 | 1.015 | 1.004 / 1.002 / 1.004 | 321 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.004 | 0.993 | 1.011 | 1.004 / 1.006 / 1.003 | 1,102 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.004 | 0.991 | 1.010 | 0.999 / 1.007 / 1.010 | 1,351 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.004 | 0.975 | 1.008 | 1.003 / 1.005 / 1.008 | 7,349 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.004 | 0.983 | 1.040 | 0.991 / 1.026 / 1.004 | 11,107 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.004 | 0.995 | 1.020 | 1.015 / 1.000 / 1.004 | 449 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.006 | 0.989 | 1.026 | 1.005 / 1.006 / 1.009 | 155 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.006 | 0.979 | 1.077 | 0.988 / 1.014 / 1.015 | 15,869 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.010 | 0.919 | 1.093 | 0.987 / 1.039 / 1.010 | 18,305 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.015 | 0.905 | 1.129 | 1.069 / 1.015 / 1.015 | 18,533 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.063 | 0.946 | 1.221 | 1.035 / 1.047 / 1.076 | 36,289 |

## arena, broad

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.988 | 0.974 | 1.064 | 0.988 / 0.986 / 0.999 | 33,853 |
| `vite-docs-api-plugin` | reference | 31890 | 0.992 | 0.923 | 1.063 | 0.971 / 1.028 / 0.999 | 65,975 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.993 | 0.973 | 1.002 | 0.990 / 0.990 / 0.998 | 12,144 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.993 | 0.978 | 1.015 | 0.992 / 0.983 / 0.993 | 15,043 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.993 | 0.979 | 1.032 | 0.993 / 1.004 / 0.993 | 8,156 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.993 | 0.976 | 1.007 | 0.992 / 1.000 / 0.993 | 4,033 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.995 | 0.985 | 1.007 | 0.999 / 0.995 / 0.995 | 3,511 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.995 | 0.983 | 1.003 | 0.991 / 1.001 / 0.998 | 9,361 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.996 | 0.988 | 1.008 | 0.999 / 0.997 / 0.990 | 77,163 |
| `comment-incident` | comments | 1124 | 0.997 | 0.989 | 1.007 | 0.996 / 1.002 / 1.002 | 1,399 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.997 | 0.969 | 1.023 | 0.996 / 0.984 / 1.002 | 5,977 |
| `vite-docs-performance` | technical-docs | 8184 | 0.997 | 0.964 | 1.009 | 0.996 / 0.998 / 0.997 | 9,014 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.999 | 0.989 | 1.012 | 0.994 / 1.000 / 0.998 | 14,255 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.999 | 0.971 | 1.014 | 1.006 / 1.001 / 0.996 | 2,364 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.000 | 0.974 | 1.016 | 1.001 / 1.000 / 0.999 | 1,041 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.000 | 0.984 | 1.017 | 0.997 / 1.003 / 1.003 | 7,804 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.001 | 0.979 | 1.013 | 1.001 / 0.991 / 1.001 | 27,203 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.001 | 0.990 | 1.020 | 0.999 / 1.003 / 0.998 | 1,751 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.001 | 0.990 | 1.031 | 1.001 / 1.003 / 1.002 | 3,185 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.001 | 0.903 | 1.126 | 0.999 / 1.037 / 0.991 | 66,263 |
| `comment-review-long` | comments | 957 | 1.002 | 0.994 | 1.024 | 1.001 / 1.002 / 1.002 | 760 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.002 | 0.985 | 1.017 | 1.011 / 0.998 / 1.001 | 1,425 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.002 | 0.958 | 1.047 | 0.991 / 1.002 / 1.006 | 57,459 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.002 | 0.992 | 1.012 | 1.001 / 1.006 / 1.002 | 1,604 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.003 | 0.978 | 1.006 | 1.003 / 1.003 / 1.002 | 1,618 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.003 | 0.983 | 1.008 | 1.003 / 1.004 / 1.002 | 3,843 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.003 | 0.974 | 1.029 | 1.002 / 0.993 / 1.009 | 54,945 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.004 | 0.989 | 1.011 | 1.002 / 1.004 / 0.998 | 9,434 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.005 | 0.988 | 1.012 | 1.005 / 1.004 / 1.004 | 2,982 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.005 | 0.994 | 1.023 | 1.003 / 1.005 / 1.007 | 7,716 |
| `vue-docs-slots` | technical-docs | 24211 | 1.005 | 0.995 | 1.014 | 1.003 / 1.008 / 1.012 | 23,586 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.006 | 0.985 | 1.026 | 0.998 / 1.006 / 1.012 | 22,478 |
| `comment-checklist` | comments | 287 | 1.008 | 0.984 | 1.022 | 1.004 / 1.018 / 1.007 | 589 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.008 | 0.993 | 1.019 | 1.013 / 1.010 / 0.996 | 7,553 |
| `legacy-contributing` | technical-docs | 9323 | 1.009 | 0.988 | 1.074 | 1.010 / 1.012 / 1.007 | 10,820 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.009 | 0.997 | 1.039 | 1.012 / 1.009 / 1.007 | 10,282 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.009 | 0.991 | 1.026 | 1.003 / 1.016 / 1.010 | 5,368 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.009 | 0.995 | 1.018 | 1.009 / 1.005 / 1.011 | 5,578 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.010 | 0.989 | 1.014 | 1.008 / 1.009 / 1.013 | 2,924 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.010 | 0.982 | 1.019 | 1.013 / 1.010 / 0.999 | 2,546 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.011 | 0.899 | 1.091 | 1.020 / 0.975 / 1.002 | 116,118 |
| `legacy-docs-readme` | readme | 1825 | 1.011 | 1.002 | 1.026 | 1.016 / 1.010 / 1.007 | 4,806 |
| `comment-table` | comments | 310 | 1.012 | 0.987 | 1.019 | 1.012 / 1.009 / 1.013 | 1,150 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.013 | 1.004 | 1.020 | 1.011 / 1.011 / 1.016 | 698 |
| `comment-links` | comments | 278 | 1.013 | 0.989 | 1.051 | 1.016 / 1.002 / 1.013 | 473 |
| `vite-docs-features` | technical-docs | 39739 | 1.014 | 0.888 | 1.143 | 1.014 / 1.009 / 1.069 | 64,354 |
| `comment-reproduction` | comments | 298 | 1.019 | 0.983 | 1.024 | 1.011 / 1.020 / 1.022 | 317 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.019 | 1.012 | 1.037 | 1.020 / 1.019 / 1.017 | 2,474 |
| `comment-unicode` | comments | 327 | 1.019 | 1.009 | 1.029 | 1.020 / 1.019 / 1.016 | 460 |
| `comment-inline-code` | comments | 285 | 1.021 | 1.011 | 1.034 | 1.020 / 1.021 / 1.023 | 298 |
| `comment-quote` | comments | 290 | 1.023 | 1.009 | 1.054 | 1.024 / 1.021 / 1.023 | 294 |
| `comment-review` | comments | 282 | 1.027 | 1.018 | 1.033 | 1.027 / 1.025 / 1.029 | 231 |
| `guard-angle-link` | syntax-guard | 41 | 1.028 | 1.012 | 1.039 | 1.025 / 1.032 / 1.032 | 273 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.032 | 0.942 | 1.212 | 1.074 / 1.039 / 1.027 | 50,381 |
| `comment-question` | comments | 160 | 1.032 | 1.020 | 1.040 | 1.032 / 1.031 / 1.038 | 167 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.040 | 0.918 | 1.162 | 1.040 / 1.066 / 1.039 | 62,371 |
| `comment-ack` | comments | 37 | 1.048 | 1.023 | 1.063 | 1.053 / 1.051 / 1.042 | 149 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-reproduction` | comments | 298 | 0.987 | 0.984 | 0.994 | 0.987 / 0.989 / 0.987 | 233 |
| `vite-docs-api-plugin` | reference | 31890 | 0.992 | 0.890 | 1.363 | 0.997 / 0.988 / 0.986 | 67,268 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.992 | 0.955 | 1.028 | 0.987 / 0.992 / 0.986 | 26,627 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.992 | 0.954 | 1.095 | 1.020 / 0.979 / 1.018 | 33,657 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.993 | 0.975 | 1.001 | 0.989 / 0.994 / 0.993 | 14,744 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.993 | 0.971 | 1.030 | 0.993 / 1.002 / 0.991 | 11,787 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.993 | 0.976 | 1.023 | 0.993 / 0.999 / 0.992 | 7,468 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.994 | 0.861 | 1.096 | 0.982 / 1.045 / 1.028 | 48,421 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.995 | 0.985 | 0.999 | 0.998 / 0.991 / 0.993 | 5,083 |
| `comment-review-long` | comments | 957 | 0.996 | 0.974 | 1.020 | 0.987 / 0.998 / 0.997 | 676 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.997 | 0.995 | 1.008 | 0.996 / 0.998 / 0.998 | 5,647 |
| `comment-incident` | comments | 1124 | 0.998 | 0.993 | 1.021 | 1.002 / 0.998 / 0.995 | 1,229 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.998 | 0.982 | 1.012 | 0.997 / 1.006 / 0.998 | 3,763 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.998 | 0.946 | 1.128 | 0.985 / 1.008 / 0.994 | 64,728 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.998 | 0.972 | 1.015 | 0.999 / 0.997 / 0.995 | 2,150 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.998 | 0.969 | 1.006 | 1.000 / 1.001 / 0.989 | 7,746 |
| `comment-ack` | comments | 37 | 0.998 | 0.993 | 1.004 | 0.997 / 1.001 / 0.997 | 68 |
| `comment-quote` | comments | 290 | 0.999 | 0.983 | 1.024 | 1.002 / 0.999 / 0.998 | 210 |
| `comment-question` | comments | 160 | 0.999 | 0.994 | 1.019 | 0.999 / 1.002 / 0.999 | 90 |
| `guard-angle-link` | syntax-guard | 41 | 0.999 | 0.988 | 1.029 | 1.020 / 0.995 / 0.996 | 183 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.995 | 1.005 | 0.999 / 1.001 / 1.001 | 9,007 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.000 | 0.992 | 1.010 | 1.006 / 0.997 / 1.000 | 5,157 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.001 | 0.993 | 1.009 | 1.000 / 1.002 / 1.001 | 3,253 |
| `comment-unicode` | comments | 327 | 1.001 | 0.976 | 1.005 | 1.000 / 1.001 / 1.001 | 373 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.001 | 0.994 | 1.007 | 1.003 / 1.001 / 1.001 | 76,546 |
| `comment-review` | comments | 282 | 1.001 | 0.992 | 1.069 | 1.001 / 1.004 / 0.993 | 152 |
| `vite-docs-performance` | technical-docs | 8184 | 1.001 | 0.990 | 1.015 | 1.002 / 1.001 / 1.001 | 8,740 |
| `vue-docs-slots` | technical-docs | 24211 | 1.002 | 0.980 | 1.034 | 0.997 / 1.007 / 1.002 | 23,056 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.002 | 0.995 | 1.010 | 1.004 / 1.000 / 1.002 | 1,445 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.002 | 0.873 | 1.033 | 1.008 / 0.994 / 1.011 | 1,255 |
| `comment-checklist` | comments | 287 | 1.003 | 0.988 | 1.025 | 1.002 / 1.000 / 1.008 | 510 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.003 | 0.982 | 1.011 | 1.003 / 1.008 / 0.997 | 1,578 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.003 | 0.994 | 1.023 | 1.018 / 1.002 / 1.005 | 22,052 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.003 | 0.994 | 1.016 | 1.002 / 1.012 / 1.003 | 1,467 |
| `comment-inline-code` | comments | 285 | 1.004 | 0.990 | 1.025 | 1.005 / 0.999 / 1.005 | 217 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.004 | 0.989 | 1.013 | 1.006 / 1.003 / 1.004 | 9,287 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.004 | 0.965 | 1.012 | 1.006 / 1.009 / 1.001 | 13,937 |
| `legacy-contributing` | technical-docs | 9323 | 1.005 | 0.976 | 1.027 | 1.007 / 1.005 / 1.004 | 10,454 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.005 | 0.994 | 1.034 | 1.011 / 1.008 / 1.002 | 2,969 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.005 | 0.981 | 1.021 | 1.007 / 1.002 / 1.005 | 6,976 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.005 | 0.995 | 1.015 | 0.998 / 1.005 / 1.005 | 536 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.005 | 0.993 | 1.029 | 1.008 / 1.015 / 1.004 | 7,655 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.005 | 0.940 | 1.053 | 0.990 / 1.019 / 1.016 | 117,521 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.006 | 1.001 | 1.032 | 1.004 / 1.007 / 1.006 | 9,921 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.007 | 0.991 | 1.021 | 1.007 / 1.008 / 1.007 | 2,370 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.007 | 0.979 | 1.015 | 1.002 / 1.012 / 1.004 | 889 |
| `comment-table` | comments | 310 | 1.009 | 0.994 | 1.021 | 1.012 / 1.008 / 1.009 | 1,061 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.009 | 1.003 | 1.030 | 1.009 / 1.020 / 1.007 | 2,834 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.009 | 1.000 | 1.025 | 1.014 / 1.009 / 1.004 | 53,963 |
| `legacy-docs-readme` | readme | 1825 | 1.009 | 0.999 | 1.023 | 1.009 / 1.006 / 1.013 | 4,547 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.010 | 0.996 | 1.017 | 1.014 / 1.010 / 1.009 | 3,696 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.010 | 0.999 | 1.034 | 1.008 / 1.010 / 1.012 | 2,758 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.011 | 0.948 | 1.125 | 0.989 / 1.028 / 1.006 | 58,134 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.012 | 1.006 | 1.028 | 1.012 / 1.008 / 1.015 | 2,282 |
| `vite-docs-features` | technical-docs | 39739 | 1.012 | 0.895 | 1.070 | 1.042 / 1.009 / 1.011 | 62,675 |
| `comment-links` | comments | 278 | 1.014 | 1.004 | 1.024 | 1.007 / 1.013 / 1.014 | 385 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.037 | 0.968 | 1.095 | 1.037 / 1.007 / 1.056 | 60,188 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-reproduction` | comments | 298 | 0.977 | 0.945 | 1.002 | 0.974 / 0.977 / 0.989 | 173 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.987 | 0.952 | 1.029 | 0.978 / 0.988 / 0.995 | 34,092 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.995 | 0.952 | 1.015 | 0.977 / 0.998 / 0.997 | 1,822 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.996 | 0.983 | 1.019 | 1.003 / 0.998 / 0.995 | 19,549 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.997 | 0.988 | 1.008 | 0.997 / 0.997 / 1.003 | 3,422 |
| `vite-docs-api-plugin` | reference | 31890 | 0.997 | 0.963 | 1.031 | 1.009 / 0.992 / 0.997 | 49,693 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.991 | 1.010 | 0.995 / 0.998 / 0.999 | 8,016 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.998 | 0.981 | 1.014 | 0.998 / 0.997 / 1.001 | 6,208 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.998 | 0.980 | 1.019 | 1.001 / 0.993 / 1.011 | 5,827 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.999 | 0.988 | 1.011 | 0.991 / 1.001 / 0.999 | 4,416 |
| `comment-quote` | comments | 290 | 0.999 | 0.989 | 1.007 | 1.000 / 0.999 / 0.996 | 160 |
| `vite-docs-performance` | technical-docs | 8184 | 0.999 | 0.994 | 1.013 | 0.997 / 1.002 / 0.999 | 5,981 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.000 | 0.983 | 1.044 | 0.993 / 1.000 / 1.000 | 1,367 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.000 | 0.994 | 1.008 | 0.996 / 1.002 / 1.000 | 921 |
| `vite-docs-features` | technical-docs | 39739 | 1.000 | 0.947 | 1.045 | 1.007 / 0.998 / 1.000 | 38,245 |
| `comment-unicode` | comments | 327 | 1.000 | 0.991 | 1.012 | 1.005 / 1.000 / 1.000 | 284 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.000 | 0.992 | 1.016 | 1.004 / 1.000 / 1.000 | 1,032 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.000 | 0.972 | 1.013 | 1.003 / 0.996 / 1.005 | 5,047 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.000 | 0.990 | 1.012 | 1.000 / 1.000 / 1.003 | 801 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.001 | 0.986 | 1.008 | 1.003 / 1.000 / 0.998 | 10,279 |
| `comment-incident` | comments | 1124 | 1.001 | 0.991 | 1.016 | 1.002 / 0.999 / 1.001 | 938 |
| `guard-angle-link` | syntax-guard | 41 | 1.001 | 0.991 | 1.013 | 1.009 / 1.003 / 1.000 | 155 |
| `comment-inline-code` | comments | 285 | 1.001 | 0.988 | 1.027 | 1.000 / 1.007 / 1.000 | 134 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.001 | 0.984 | 1.022 | 0.997 / 1.005 / 1.002 | 17,813 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.002 | 0.980 | 1.012 | 1.004 / 1.002 / 1.000 | 2,390 |
| `comment-question` | comments | 160 | 1.002 | 0.991 | 1.029 | 1.002 / 1.003 / 1.000 | 67 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.002 | 0.996 | 1.022 | 1.002 / 1.005 / 1.001 | 11,567 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.002 | 0.906 | 1.283 | 0.998 / 1.007 / 1.002 | 4,821 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.002 | 0.990 | 1.011 | 1.000 / 1.003 / 0.999 | 3,805 |
| `vue-docs-slots` | technical-docs | 24211 | 1.002 | 0.993 | 1.036 | 1.007 / 0.999 / 1.000 | 13,498 |
| `comment-review-long` | comments | 957 | 1.003 | 0.995 | 1.011 | 1.009 / 1.000 / 1.003 | 527 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.004 | 0.995 | 1.006 | 1.004 / 1.004 / 1.001 | 2,069 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.004 | 0.975 | 1.050 | 1.010 / 0.991 / 1.008 | 21,476 |
| `comment-ack` | comments | 37 | 1.004 | 0.998 | 1.015 | 1.005 / 1.005 / 1.003 | 51 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.005 | 0.980 | 1.016 | 1.007 / 1.006 / 0.995 | 14,267 |
| `legacy-contributing` | technical-docs | 9323 | 1.005 | 0.989 | 1.376 | 1.007 / 1.005 / 1.003 | 7,583 |
| `comment-checklist` | comments | 287 | 1.006 | 0.976 | 1.016 | 1.009 / 0.998 / 1.006 | 404 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.006 | 0.998 | 1.017 | 1.006 / 1.005 / 1.007 | 564 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.006 | 0.995 | 1.009 | 1.006 / 1.002 / 1.007 | 3,895 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.006 | 1.001 | 1.030 | 1.011 / 1.007 / 1.005 | 6,324 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.006 | 0.996 | 1.015 | 1.006 / 1.001 / 1.006 | 948 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.007 | 0.989 | 1.020 | 1.009 / 1.010 / 1.005 | 5,083 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.007 | 1.000 | 1.046 | 1.006 / 1.007 / 1.008 | 1,613 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.007 | 0.978 | 1.024 | 1.007 / 1.001 / 1.007 | 2,478 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.007 | 0.966 | 1.042 | 1.006 / 1.009 / 1.007 | 45,941 |
| `legacy-docs-readme` | readme | 1825 | 1.009 | 1.001 | 1.021 | 1.009 / 1.007 / 1.009 | 3,503 |
| `comment-table` | comments | 310 | 1.010 | 0.991 | 1.020 | 1.010 / 1.010 / 1.004 | 879 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.010 | 0.978 | 1.082 | 1.011 / 0.996 / 1.015 | 34,584 |
| `comment-review` | comments | 282 | 1.011 | 1.004 | 1.018 | 1.011 / 1.009 / 1.011 | 112 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.011 | 0.987 | 1.016 | 1.012 / 1.003 / 1.005 | 1,745 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.013 | 1.002 | 1.020 | 1.014 / 1.013 / 1.008 | 376 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.013 | 1.004 | 1.031 | 1.011 / 1.020 / 1.013 | 1,349 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.014 | 1.006 | 1.024 | 1.011 / 1.019 / 1.014 | 1,649 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.015 | 0.910 | 1.076 | 1.024 / 1.008 / 1.017 | 70,718 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.016 | 0.986 | 1.041 | 0.996 / 1.023 / 1.019 | 39,765 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.016 | 0.990 | 1.045 | 1.017 / 1.020 / 1.008 | 29,221 |
| `comment-links` | comments | 278 | 1.020 | 0.942 | 1.109 | 1.020 / 1.018 / 1.027 | 298 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-table` | comments | 310 | 0.989 | 0.977 | 1.002 | 0.990 / 0.983 / 0.988 | 181 |
| `comment-ack` | comments | 37 | 0.991 | 0.978 | 1.001 | 0.989 / 0.991 / 0.995 | 19 |
| `comment-links` | comments | 278 | 0.992 | 0.987 | 1.011 | 0.991 / 0.994 / 0.992 | 87 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.992 | 0.972 | 1.007 | 0.988 / 0.985 / 0.995 | 8,783 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.995 | 0.983 | 1.051 | 0.995 / 0.994 / 0.996 | 2,324 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.996 | 0.978 | 1.002 | 0.992 / 0.998 / 0.997 | 1,875 |
| `comment-question` | comments | 160 | 0.997 | 0.983 | 1.017 | 0.997 / 0.997 / 0.996 | 25 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.998 | 0.989 | 1.024 | 1.004 / 0.997 / 0.997 | 7,473 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.998 | 0.991 | 1.022 | 0.995 / 1.007 / 0.998 | 56,456 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.987 | 1.008 | 0.998 / 0.999 / 0.997 | 3,756 |
| `legacy-docs-readme` | readme | 1825 | 0.998 | 0.991 | 1.009 | 0.993 / 0.996 / 0.999 | 1,040 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.998 | 0.906 | 1.077 | 0.971 / 1.027 / 0.993 | 34,514 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.998 | 0.961 | 1.025 | 1.001 / 0.998 / 0.994 | 3,267 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.999 | 0.989 | 1.012 | 1.000 / 0.999 / 0.999 | 787 |
| `comment-checklist` | comments | 287 | 0.999 | 0.984 | 1.014 | 1.003 / 1.000 / 0.997 | 97 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.999 | 0.990 | 1.023 | 1.000 / 0.999 / 0.998 | 1,083 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.999 | 0.988 | 1.007 | 0.999 / 1.004 / 0.997 | 1,324 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.999 | 0.994 | 1.040 | 1.000 / 0.997 / 0.999 | 320 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.000 | 0.993 | 1.002 | 1.000 / 1.001 / 0.999 | 1,167 |
| `comment-reproduction` | comments | 298 | 1.000 | 0.989 | 1.020 | 1.006 / 1.001 / 0.992 | 63 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.000 | 0.993 | 1.012 | 0.999 / 1.001 / 1.002 | 543 |
| `comment-review-long` | comments | 957 | 1.000 | 0.997 | 1.005 | 1.000 / 0.999 / 1.001 | 149 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.000 | 0.985 | 1.004 | 1.002 / 0.997 / 1.002 | 152 |
| `comment-incident` | comments | 1124 | 1.000 | 0.978 | 1.008 | 0.993 / 1.000 / 1.002 | 282 |
| `guard-angle-link` | syntax-guard | 41 | 1.000 | 0.991 | 1.016 | 0.997 / 1.012 / 1.000 | 32 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.000 | 0.994 | 1.010 | 1.001 / 0.997 / 1.003 | 513 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.001 | 0.918 | 1.049 | 1.015 / 0.983 / 1.006 | 18,036 |
| `comment-review` | comments | 282 | 1.001 | 0.996 | 1.011 | 1.000 / 1.002 / 1.001 | 43 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.001 | 0.996 | 1.006 | 1.002 / 1.002 / 0.998 | 497 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.001 | 0.973 | 1.040 | 1.001 / 0.989 / 1.007 | 17,967 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.001 | 0.981 | 1.009 | 1.003 / 1.001 / 1.001 | 3,465 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.001 | 0.990 | 1.011 | 1.000 / 1.002 / 1.002 | 1,503 |
| `vue-docs-slots` | technical-docs | 24211 | 1.002 | 0.984 | 1.036 | 1.007 / 1.000 / 1.001 | 9,371 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.002 | 0.982 | 1.021 | 1.001 / 0.997 / 1.006 | 4,548 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.002 | 0.985 | 1.028 | 1.007 / 0.999 / 1.001 | 1,171 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.002 | 0.987 | 1.019 | 1.004 / 0.998 / 1.004 | 1,217 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.002 | 0.994 | 1.018 | 0.997 / 1.003 / 1.002 | 712 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.002 | 0.883 | 1.064 | 1.001 / 1.013 / 1.023 | 21,036 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.003 | 0.993 | 1.014 | 1.007 / 1.002 / 1.003 | 2,447 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.003 | 0.986 | 1.022 | 1.010 / 1.001 / 1.000 | 2,536 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.003 | 0.978 | 1.021 | 1.000 / 1.006 / 1.004 | 11,213 |
| `vite-docs-features` | technical-docs | 39739 | 1.003 | 0.926 | 1.026 | 0.988 / 1.004 / 1.007 | 17,220 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.003 | 0.995 | 1.014 | 1.004 / 0.999 / 1.003 | 7,230 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.004 | 0.993 | 1.015 | 1.009 / 1.001 / 1.009 | 2,484 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.004 | 0.993 | 1.021 | 1.009 / 1.003 / 1.009 | 1,115 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.004 | 0.988 | 1.014 | 1.001 / 1.005 / 1.005 | 445 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.004 | 0.997 | 1.008 | 1.001 / 1.004 / 1.005 | 1,054 |
| `legacy-contributing` | technical-docs | 9323 | 1.004 | 0.990 | 1.016 | 1.004 / 1.004 / 1.003 | 2,711 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.006 | 0.999 | 1.034 | 1.005 / 1.014 / 1.022 | 917 |
| `comment-unicode` | comments | 327 | 1.006 | 1.000 | 1.021 | 1.005 / 1.010 / 1.006 | 95 |
| `vite-docs-api-plugin` | reference | 31890 | 1.007 | 0.995 | 1.017 | 1.001 / 1.005 / 1.014 | 12,259 |
| `comment-quote` | comments | 290 | 1.007 | 0.992 | 1.018 | 1.007 / 1.011 / 1.006 | 49 |
| `vite-docs-performance` | technical-docs | 8184 | 1.008 | 0.998 | 1.041 | 1.010 / 1.004 / 1.008 | 2,679 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.009 | 0.986 | 1.046 | 1.019 / 1.009 / 1.008 | 4,030 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.012 | 0.982 | 1.049 | 1.008 / 1.019 / 1.003 | 15,894 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.012 | 0.988 | 1.021 | 1.012 / 1.016 / 1.011 | 1,588 |
| `comment-inline-code` | comments | 285 | 1.014 | 1.009 | 1.020 | 1.014 / 1.012 / 1.016 | 83 |

## tables, screen

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.912 | 0.906 | 0.917 | 0.915 / 0.912 / 0.912 | 112,973 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.915 | 0.906 | 0.921 | 0.915 / 0.917 / 0.909 | 29,169 |
| `table-formatted-256` | table-diagnostic | 278 | 0.941 | 0.934 | 0.945 | 0.939 / 0.942 / 0.943 | 2,135 |
| `table-sparse-256` | table-diagnostic | 297 | 0.976 | 0.969 | 0.980 | 0.973 / 0.978 / 0.976 | 603 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.982 | 0.940 | 0.991 | 0.982 / 0.991 / 0.943 | 3,462 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.991 | 0.974 | 1.006 | 0.991 / 0.999 / 0.986 | 26,995 |
| `table-plain-256` | table-diagnostic | 299 | 0.991 | 0.985 | 0.999 | 0.991 / 0.994 / 0.987 | 571 |
| `comment-review-long` | comments | 957 | 0.992 | 0.988 | 1.000 | 1.000 / 0.993 / 0.990 | 756 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.995 | 0.951 | 1.026 | 1.003 / 0.973 / 0.995 | 1,616 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.989 | 1.024 | 0.998 / 0.994 / 1.007 | 54,370 |
| `comment-links` | comments | 278 | 0.998 | 0.990 | 1.005 | 1.001 / 0.991 / 0.995 | 468 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.999 | 0.994 | 1.001 | 0.998 / 1.000 / 0.999 | 4,521 |
| `comment-question` | comments | 160 | 0.999 | 0.976 | 1.002 | 1.000 / 0.999 / 0.994 | 165 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.999 | 0.961 | 1.011 | 0.999 / 1.001 / 0.998 | 5,973 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.993 | 1.005 | 0.996 / 1.002 / 1.005 | 5,270 |
| `comment-incident` | comments | 1124 | 1.000 | 0.985 | 1.001 | 0.997 / 1.001 / 1.000 | 1,387 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.000 | 0.993 | 1.011 | 1.001 / 0.999 / 0.998 | 7,649 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.000 | 0.990 | 1.006 | 1.003 / 0.993 / 1.000 | 5,519 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.001 | 0.995 | 1.009 | 1.000 / 1.001 / 1.001 | 32,849 |
| `vite-docs-performance` | technical-docs | 8184 | 1.001 | 0.999 | 1.019 | 0.999 / 1.003 / 1.001 | 8,854 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.002 | 0.995 | 1.006 | 1.002 / 0.999 / 1.004 | 2,467 |
| `comment-ack` | comments | 37 | 1.005 | 0.991 | 1.013 | 1.010 / 0.995 / 1.005 | 146 |
| `comment-checklist` | comments | 287 | 1.006 | 0.997 | 1.012 | 1.006 / 1.011 / 0.999 | 584 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.006 | 0.995 | 1.013 | 1.005 / 1.007 / 1.006 | 1,738 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.007 | 1.000 | 1.012 | 1.007 / 1.012 / 1.004 | 555 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.007 | 0.908 | 1.037 | 1.024 / 1.009 / 1.005 | 56,691 |
| `vite-docs-api-plugin` | reference | 31890 | 1.009 | 0.905 | 1.039 | 1.015 / 0.985 / 1.010 | 66,252 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.010 | 0.899 | 1.100 | 0.981 / 1.043 / 1.010 | 64,527 |
| `comment-quote` | comments | 290 | 1.010 | 0.995 | 1.072 | 1.010 / 1.000 / 1.069 | 292 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.021 | 0.972 | 1.084 | 1.025 / 1.064 / 0.986 | 48,478 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.028 | 1.011 | 1.044 | 1.028 / 1.031 / 1.027 | 4,423 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.030 | 1.013 | 1.051 | 1.020 / 1.042 / 1.031 | 1,499 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.053 | 1.041 | 1.070 | 1.050 / 1.059 / 1.053 | 4,040 |
| `table-dense-256` | table-diagnostic | 300 | 1.285 | 1.277 | 1.292 | 1.286 / 1.287 / 1.284 | 2,399 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.423 | 1.415 | 1.435 | 1.419 / 1.424 / 1.427 | 31,093 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.430 | 1.409 | 1.437 | 1.435 / 1.428 / 1.430 | 119,743 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.913 | 0.908 | 0.918 | 0.915 / 0.913 / 0.916 | 111,510 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.915 | 0.911 | 0.919 | 0.915 / 0.915 / 0.913 | 28,885 |
| `table-formatted-256` | table-diagnostic | 278 | 0.937 | 0.931 | 1.009 | 0.939 / 0.936 / 0.951 | 2,067 |
| `table-sparse-256` | table-diagnostic | 297 | 0.967 | 0.962 | 0.972 | 0.967 / 0.966 / 0.970 | 517 |
| `table-plain-256` | table-diagnostic | 299 | 0.981 | 0.971 | 0.988 | 0.984 / 0.981 / 0.978 | 485 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.988 | 0.983 | 1.008 | 0.986 / 0.988 / 1.003 | 3,182 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.990 | 0.898 | 1.036 | 0.990 / 0.946 / 0.992 | 64,170 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.994 | 0.986 | 0.997 | 0.995 / 0.994 / 0.993 | 1,433 |
| `comment-review-long` | comments | 957 | 0.996 | 0.990 | 1.020 | 0.999 / 0.995 / 0.997 | 675 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.996 | 0.992 | 1.005 | 0.994 / 0.997 / 0.994 | 5,686 |
| `comment-links` | comments | 278 | 0.997 | 0.989 | 1.008 | 0.997 / 0.999 / 0.996 | 384 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.999 | 0.987 | 1.004 | 0.997 / 0.999 / 0.999 | 5,011 |
| `comment-question` | comments | 160 | 0.999 | 0.985 | 1.008 | 0.999 / 1.000 / 1.003 | 89 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.000 | 0.995 | 1.007 | 1.001 / 1.000 / 1.000 | 4,457 |
| `comment-incident` | comments | 1124 | 1.000 | 0.996 | 1.002 | 1.001 / 1.000 / 0.999 | 1,229 |
| `comment-ack` | comments | 37 | 1.001 | 0.989 | 1.011 | 1.002 / 0.999 / 0.992 | 68 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.001 | 0.996 | 1.007 | 1.001 / 1.005 / 0.998 | 32,762 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.001 | 0.990 | 1.007 | 0.998 / 0.996 / 1.005 | 2,281 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.001 | 0.997 | 1.014 | 1.006 / 0.999 / 1.004 | 54,104 |
| `comment-quote` | comments | 290 | 1.002 | 0.988 | 1.007 | 1.002 / 1.005 / 1.003 | 207 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.003 | 0.968 | 1.092 | 1.016 / 0.990 / 1.023 | 55,838 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.004 | 0.984 | 1.009 | 1.005 / 0.998 / 0.996 | 26,500 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.004 | 0.992 | 1.018 | 1.004 / 1.005 / 1.002 | 7,393 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.005 | 1.001 | 1.011 | 1.003 / 1.004 / 1.007 | 5,131 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.005 | 0.996 | 1.007 | 1.005 / 1.006 / 1.001 | 1,566 |
| `comment-checklist` | comments | 287 | 1.005 | 0.995 | 1.028 | 1.007 / 1.004 / 1.005 | 499 |
| `vite-docs-api-plugin` | reference | 31890 | 1.006 | 0.980 | 1.049 | 0.985 / 1.016 / 0.999 | 65,266 |
| `vite-docs-performance` | technical-docs | 8184 | 1.006 | 0.982 | 1.014 | 1.006 / 1.010 / 0.988 | 8,733 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.013 | 0.995 | 1.034 | 1.002 / 1.007 / 1.028 | 451 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.021 | 1.014 | 1.027 | 1.015 / 1.021 / 1.023 | 4,193 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.028 | 1.023 | 1.030 | 1.028 / 1.028 / 1.025 | 1,316 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.034 | 0.877 | 1.076 | 1.034 / 1.061 / 0.995 | 47,763 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.055 | 1.050 | 1.058 | 1.053 / 1.056 / 1.051 | 3,807 |
| `table-dense-256` | table-diagnostic | 300 | 1.297 | 1.286 | 1.307 | 1.297 / 1.295 / 1.307 | 2,317 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.422 | 1.415 | 1.426 | 1.424 / 1.424 / 1.421 | 30,671 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.432 | 1.428 | 1.441 | 1.430 / 1.432 / 1.431 | 118,864 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.896 | 0.888 | 0.908 | 0.896 / 0.892 / 0.897 | 91,397 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.897 | 0.890 | 0.901 | 0.898 / 0.896 / 0.900 | 23,651 |
| `table-formatted-256` | table-diagnostic | 278 | 0.926 | 0.923 | 0.930 | 0.923 / 0.928 / 0.926 | 1,679 |
| `table-sparse-256` | table-diagnostic | 297 | 0.961 | 0.955 | 0.984 | 0.961 / 0.961 / 0.963 | 447 |
| `table-plain-256` | table-diagnostic | 299 | 0.968 | 0.960 | 0.972 | 0.963 / 0.969 / 0.966 | 415 |
| `vite-docs-api-plugin` | reference | 31890 | 0.993 | 0.910 | 1.111 | 0.993 / 0.999 / 0.988 | 50,029 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.995 | 0.987 | 1.005 | 0.995 / 0.995 / 0.998 | 17,682 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.995 | 0.963 | 1.034 | 0.995 / 0.999 / 0.988 | 29,207 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.995 | 0.994 | 0.999 | 0.995 / 0.995 / 0.997 | 1,168 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.996 | 0.988 | 1.005 | 0.996 / 0.998 / 0.995 | 1,344 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.996 | 0.991 | 1.005 | 0.993 / 1.000 / 0.995 | 3,527 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.998 | 0.995 | 1.006 | 0.998 / 0.998 / 0.999 | 46,035 |
| `comment-links` | comments | 278 | 0.999 | 0.984 | 1.015 | 1.000 / 1.004 / 0.987 | 297 |
| `comment-incident` | comments | 1124 | 0.999 | 0.987 | 1.010 | 0.998 / 0.994 / 1.003 | 932 |
| `vite-docs-performance` | technical-docs | 8184 | 1.000 | 0.992 | 1.004 | 0.998 / 0.998 / 1.000 | 5,883 |
| `comment-question` | comments | 160 | 1.000 | 0.991 | 1.006 | 1.000 / 1.001 / 0.997 | 66 |
| `comment-quote` | comments | 290 | 1.000 | 0.997 | 1.003 | 1.000 / 0.999 / 1.000 | 159 |
| `comment-ack` | comments | 37 | 1.000 | 0.994 | 1.004 | 1.000 / 1.001 / 1.000 | 51 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.001 | 0.992 | 1.004 | 1.001 / 1.003 / 0.997 | 3,399 |
| `comment-checklist` | comments | 287 | 1.002 | 0.596 | 1.011 | 1.008 / 0.935 / 1.002 | 403 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.003 | 0.995 | 1.012 | 1.002 / 1.002 / 1.005 | 4,789 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.004 | 0.993 | 1.010 | 1.005 / 1.004 / 1.000 | 3,769 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.005 | 0.999 | 1.052 | 1.003 / 1.013 / 1.000 | 2,710 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.006 | 1.003 | 1.009 | 1.007 / 1.006 / 1.003 | 1,030 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.007 | 0.996 | 1.011 | 0.999 / 1.007 / 1.011 | 30,287 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.007 | 0.980 | 1.013 | 1.005 / 1.006 / 1.012 | 421 |
| `comment-review-long` | comments | 957 | 1.008 | 0.995 | 1.041 | 0.999 / 1.008 / 1.026 | 527 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.010 | 0.997 | 1.016 | 1.010 / 1.009 / 1.011 | 3,887 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.010 | 0.988 | 1.032 | 1.010 / 1.019 / 1.009 | 38,147 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.012 | 0.987 | 1.038 | 1.006 / 1.009 / 1.030 | 34,505 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.029 | 1.018 | 1.039 | 1.031 / 1.029 / 1.026 | 3,393 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.034 | 1.027 | 1.035 | 1.034 / 1.034 / 1.034 | 1,056 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.070 | 1.063 | 1.076 | 1.070 / 1.071 / 1.067 | 2,998 |
| `table-dense-256` | table-diagnostic | 300 | 1.308 | 1.304 | 1.331 | 1.311 / 1.307 / 1.309 | 2,231 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.429 | 1.413 | 1.443 | 1.426 / 1.441 / 1.425 | 30,767 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.433 | 1.428 | 1.445 | 1.431 / 1.434 / 1.432 | 118,883 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.989 | 0.966 | 1.072 | 1.006 / 0.971 / 1.015 | 17,951 |
| `vite-docs-api-plugin` | reference | 31890 | 0.993 | 0.989 | 1.011 | 0.993 / 0.991 / 0.998 | 12,104 |
| `comment-links` | comments | 278 | 0.996 | 0.987 | 1.001 | 0.992 / 0.996 / 0.999 | 87 |
| `table-dense-256` | table-diagnostic | 300 | 0.997 | 0.981 | 1.011 | 0.989 / 1.001 / 0.997 | 63 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.986 | 1.000 | 0.997 / 0.994 / 0.997 | 7,432 |
| `comment-incident` | comments | 1124 | 0.997 | 0.926 | 1.003 | 1.002 / 0.985 / 0.997 | 283 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.998 | 0.992 | 1.002 | 1.001 / 0.996 / 0.998 | 925 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.998 | 0.991 | 1.002 | 0.999 / 0.996 / 0.998 | 1,869 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.996 | 1.010 | 1.004 / 0.998 / 0.997 | 8,738 |
| `table-formatted-256` | table-diagnostic | 278 | 0.998 | 0.987 | 1.003 | 0.998 / 0.998 / 0.999 | 350 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.998 | 0.990 | 1.005 | 1.004 / 0.998 / 0.993 | 544 |
| `comment-checklist` | comments | 287 | 0.999 | 0.986 | 1.027 | 1.006 / 0.998 / 0.993 | 96 |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.999 | 0.981 | 1.002 | 0.988 / 1.001 / 0.999 | 20,211 |
| `table-dense-16000` | table-diagnostic | 16044 | 0.999 | 0.996 | 1.012 | 0.996 / 0.999 / 0.999 | 435 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.999 | 0.996 | 1.004 | 1.000 / 1.001 / 0.997 | 481 |
| `comment-review-long` | comments | 957 | 0.999 | 0.983 | 1.010 | 1.000 / 0.999 / 0.988 | 148 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.999 | 0.991 | 1.009 | 1.002 / 0.999 / 0.994 | 1,588 |
| `table-sparse-256` | table-diagnostic | 297 | 0.999 | 0.995 | 1.002 | 1.000 / 0.999 / 1.000 | 69 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.000 | 0.992 | 1.006 | 1.000 / 0.998 / 1.002 | 39 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.000 | 0.989 | 1.009 | 1.005 / 0.996 / 1.000 | 2,557 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.000 | 0.996 | 1.004 | 0.996 / 1.002 / 1.000 | 257 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.000 | 0.984 | 1.013 | 1.006 / 0.998 / 1.003 | 15,634 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.000 | 0.999 | 1.015 | 1.000 / 1.009 / 1.000 | 5,211 |
| `comment-question` | comments | 160 | 1.001 | 0.991 | 1.004 | 0.998 / 1.003 / 1.001 | 25 |
| `table-plain-256` | table-diagnostic | 299 | 1.001 | 0.994 | 1.015 | 1.002 / 0.999 / 1.001 | 69 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.001 | 0.997 | 1.010 | 1.001 / 1.001 / 1.002 | 258 |
| `comment-ack` | comments | 37 | 1.001 | 0.993 | 1.003 | 0.998 / 1.002 / 1.002 | 19 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.001 | 0.996 | 1.005 | 1.001 / 1.001 / 1.001 | 156 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.002 | 0.994 | 1.016 | 0.998 / 1.005 / 1.002 | 901 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.002 | 0.997 | 1.011 | 1.009 / 1.001 / 1.001 | 1,203 |
| `vite-docs-performance` | technical-docs | 8184 | 1.002 | 0.997 | 1.004 | 1.000 / 1.002 / 1.003 | 2,672 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.002 | 0.993 | 1.008 | 1.004 / 1.000 / 1.000 | 797 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.002 | 0.990 | 1.016 | 1.002 / 1.009 / 0.998 | 810 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.003 | 0.998 | 1.013 | 1.000 / 1.009 / 1.002 | 2,452 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.006 | 0.964 | 1.043 | 1.006 / 1.029 / 1.002 | 21,118 |
| `comment-quote` | comments | 290 | 1.006 | 0.999 | 1.012 | 1.009 / 1.004 / 1.006 | 49 |

## tables, broad

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `vite-docs-features` | technical-docs | 39739 | 0.984 | 0.857 | 1.091 | 0.978 / 0.950 / 1.019 | 61,442 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.985 | 0.885 | 1.065 | 0.985 / 1.027 / 0.968 | 34,265 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.991 | 0.973 | 1.003 | 0.987 / 0.989 / 1.000 | 15,327 |
| `comment-table` | comments | 310 | 0.992 | 0.982 | 1.019 | 0.992 / 0.992 / 0.990 | 1,148 |
| `comment-links` | comments | 278 | 0.992 | 0.975 | 1.009 | 0.996 / 0.998 / 0.978 | 466 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.994 | 0.978 | 1.024 | 0.993 / 1.000 / 1.000 | 12,428 |
| `comment-review-long` | comments | 957 | 0.995 | 0.984 | 1.011 | 0.995 / 0.996 / 0.994 | 753 |
| `guard-angle-link` | syntax-guard | 41 | 0.996 | 0.980 | 1.009 | 1.001 / 0.992 / 0.994 | 266 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.996 | 0.991 | 1.007 | 0.995 / 0.998 / 0.999 | 1,605 |
| `comment-quote` | comments | 290 | 0.996 | 0.988 | 1.025 | 0.996 / 0.998 / 0.996 | 288 |
| `vue-docs-slots` | technical-docs | 24211 | 0.997 | 0.985 | 1.117 | 0.996 / 0.999 / 0.993 | 23,757 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.997 | 0.986 | 1.011 | 1.001 / 0.990 / 0.997 | 9,652 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.997 | 0.991 | 1.010 | 0.999 / 0.997 / 0.996 | 27,799 |
| `comment-incident` | comments | 1124 | 0.997 | 0.984 | 1.014 | 1.002 / 0.997 / 0.996 | 1,367 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.997 | 0.911 | 1.014 | 0.997 / 1.001 / 0.995 | 5,499 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.997 | 0.982 | 1.050 | 0.994 / 0.997 / 0.998 | 3,428 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.997 | 0.984 | 1.008 | 0.995 / 1.001 / 0.991 | 14,277 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.998 | 0.924 | 1.067 | 0.998 / 0.992 / 0.998 | 2,347 |
| `vite-docs-performance` | technical-docs | 8184 | 0.999 | 0.993 | 1.008 | 0.994 / 1.000 / 1.003 | 8,869 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.999 | 0.996 | 1.003 | 0.999 / 1.000 / 0.999 | 8,084 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.000 | 0.987 | 1.014 | 0.999 / 1.000 / 1.001 | 7,304 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.000 | 0.995 | 1.009 | 1.004 / 0.999 / 1.000 | 710 |
| `comment-ack` | comments | 37 | 1.000 | 0.973 | 1.008 | 1.001 / 0.973 / 1.002 | 145 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.000 | 0.984 | 1.016 | 1.004 / 1.000 / 0.998 | 3,150 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.971 | 1.008 | 1.000 / 1.003 / 0.999 | 5,254 |
| `comment-unicode` | comments | 327 | 1.000 | 0.993 | 1.020 | 1.004 / 0.999 / 0.996 | 465 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.000 | 0.989 | 1.012 | 1.000 / 0.999 / 1.003 | 1,058 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.000 | 0.982 | 1.011 | 1.000 / 0.999 / 1.002 | 10,291 |
| `comment-question` | comments | 160 | 1.000 | 0.987 | 1.005 | 1.000 / 1.004 / 0.995 | 166 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.001 | 0.996 | 1.013 | 1.000 / 1.003 / 1.000 | 76,404 |
| `legacy-contributing` | technical-docs | 9323 | 1.001 | 0.969 | 1.005 | 1.000 / 1.001 / 0.999 | 10,571 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.001 | 0.986 | 1.014 | 1.005 / 1.001 / 1.000 | 3,826 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.001 | 0.977 | 1.008 | 0.999 / 1.004 / 1.001 | 3,061 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.001 | 0.986 | 1.019 | 1.004 / 0.992 / 1.005 | 1,587 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.001 | 0.992 | 1.031 | 0.996 / 1.001 / 1.003 | 54,282 |
| `comment-review` | comments | 282 | 1.001 | 0.986 | 1.027 | 1.005 / 0.999 / 0.992 | 228 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.001 | 0.917 | 1.055 | 0.993 / 1.016 / 1.001 | 121,517 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.002 | 0.985 | 1.013 | 1.001 / 0.997 / 1.008 | 9,188 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.002 | 0.989 | 1.035 | 0.993 / 1.002 / 1.003 | 22,340 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.002 | 0.992 | 1.018 | 1.002 / 1.003 / 1.002 | 1,731 |
| `comment-inline-code` | comments | 285 | 1.002 | 0.997 | 1.026 | 1.006 / 1.001 / 1.002 | 295 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.003 | 0.987 | 1.012 | 0.996 / 1.005 / 1.002 | 6,081 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.003 | 0.992 | 1.020 | 1.004 / 1.004 / 1.002 | 2,970 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.003 | 0.999 | 1.019 | 1.003 / 1.004 / 1.007 | 2,522 |
| `vite-docs-api-plugin` | reference | 31890 | 1.003 | 0.971 | 1.097 | 1.002 / 1.010 / 0.994 | 68,419 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.004 | 0.961 | 1.042 | 1.004 / 1.013 / 0.979 | 56,347 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.005 | 0.992 | 1.015 | 1.005 / 1.003 / 1.010 | 1,414 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.005 | 0.987 | 1.021 | 1.013 / 1.005 / 1.003 | 4,097 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.005 | 0.914 | 1.075 | 1.007 / 1.005 / 1.002 | 68,244 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.005 | 0.999 | 1.014 | 1.005 / 1.003 / 1.007 | 2,472 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.006 | 0.978 | 1.020 | 1.012 / 1.006 / 1.003 | 7,762 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.007 | 0.900 | 1.114 | 1.015 / 1.007 / 1.007 | 48,339 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.008 | 1.003 | 1.010 | 1.004 / 1.009 / 1.008 | 7,771 |
| `comment-checklist` | comments | 287 | 1.008 | 0.996 | 1.037 | 1.029 / 1.006 / 1.005 | 595 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.012 | 0.937 | 1.053 | 1.012 / 1.032 / 1.001 | 59,858 |
| `comment-reproduction` | comments | 298 | 1.014 | 1.007 | 1.027 | 1.010 / 1.015 / 1.014 | 314 |
| `legacy-docs-readme` | readme | 1825 | 1.020 | 1.003 | 1.032 | 1.017 / 1.021 / 1.018 | 4,693 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.986 | 0.974 | 1.003 | 0.985 / 0.986 / 0.991 | 14,602 |
| `comment-table` | comments | 310 | 0.990 | 0.976 | 1.000 | 0.993 / 0.984 / 0.990 | 1,054 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.992 | 0.966 | 1.009 | 1.003 / 0.988 / 0.993 | 14,145 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.993 | 0.982 | 1.016 | 0.993 / 0.990 / 1.003 | 9,298 |
| `comment-ack` | comments | 37 | 0.995 | 0.978 | 1.006 | 0.992 / 1.000 / 0.998 | 69 |
| `vue-docs-slots` | technical-docs | 24211 | 0.996 | 0.961 | 1.012 | 1.002 / 0.981 / 1.002 | 23,002 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.996 | 0.988 | 1.005 | 1.001 / 0.992 / 0.997 | 5,666 |
| `legacy-contributing` | technical-docs | 9323 | 0.996 | 0.985 | 1.017 | 0.999 / 0.990 / 0.996 | 10,332 |
| `comment-incident` | comments | 1124 | 0.996 | 0.986 | 1.009 | 1.005 / 0.995 / 0.996 | 1,251 |
| `comment-links` | comments | 278 | 0.996 | 0.943 | 1.010 | 0.997 / 0.950 / 1.004 | 392 |
| `comment-review-long` | comments | 957 | 0.996 | 0.967 | 1.013 | 0.996 / 0.997 / 0.996 | 682 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.997 | 0.991 | 1.025 | 0.998 / 0.997 / 0.996 | 2,424 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.963 | 1.010 | 0.998 / 0.989 / 1.003 | 27,083 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.998 | 0.986 | 1.013 | 0.989 / 1.004 / 0.993 | 7,723 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.983 | 1.005 | 0.994 / 0.994 / 1.003 | 12,074 |
| `vite-docs-features` | technical-docs | 39739 | 0.998 | 0.952 | 1.034 | 1.008 / 1.008 / 0.971 | 60,486 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.998 | 0.942 | 1.024 | 0.997 / 1.002 / 0.997 | 1,482 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.998 | 0.987 | 1.018 | 0.995 / 1.001 / 1.002 | 2,173 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.998 | 0.991 | 1.010 | 0.998 / 0.999 / 0.998 | 2,820 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.998 | 0.991 | 1.009 | 0.994 / 0.998 / 1.008 | 6,905 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.999 | 0.988 | 1.013 | 0.994 / 1.000 / 1.003 | 75,879 |
| `comment-question` | comments | 160 | 1.000 | 0.995 | 1.013 | 0.999 / 1.001 / 1.002 | 89 |
| `comment-unicode` | comments | 327 | 1.000 | 0.989 | 1.012 | 0.998 / 1.000 / 1.000 | 376 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.000 | 0.927 | 1.129 | 1.003 / 0.993 / 1.009 | 55,635 |
| `guard-angle-link` | syntax-guard | 41 | 1.000 | 0.980 | 1.018 | 1.015 / 0.993 / 0.994 | 188 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.000 | 0.989 | 1.021 | 1.002 / 0.999 / 1.000 | 3,285 |
| `comment-quote` | comments | 290 | 1.000 | 0.981 | 1.008 | 1.003 / 0.999 / 0.999 | 208 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.993 | 1.005 | 1.001 / 0.998 / 0.999 | 5,119 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.000 | 0.986 | 1.015 | 1.005 / 0.998 / 1.000 | 3,754 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.000 | 0.982 | 1.005 | 1.002 / 1.000 / 0.996 | 2,769 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.001 | 0.977 | 1.014 | 1.002 / 0.997 / 1.002 | 1,606 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.001 | 0.994 | 1.016 | 0.999 / 1.002 / 1.000 | 2,267 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.001 | 0.888 | 1.138 | 1.001 / 0.990 / 1.014 | 34,054 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.001 | 0.996 | 1.029 | 1.001 / 1.000 / 1.012 | 53,263 |
| `comment-checklist` | comments | 287 | 1.001 | 0.982 | 1.013 | 1.007 / 0.997 / 1.001 | 494 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.002 | 0.977 | 1.029 | 1.002 / 1.000 / 0.999 | 21,555 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.002 | 0.999 | 1.014 | 1.002 / 1.005 / 1.001 | 7,509 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.002 | 0.994 | 1.011 | 1.002 / 0.996 / 1.007 | 551 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.003 | 0.987 | 1.011 | 1.003 / 0.997 / 1.004 | 3,701 |
| `vite-docs-performance` | technical-docs | 8184 | 1.003 | 0.996 | 1.014 | 1.003 / 1.003 / 1.003 | 8,620 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.003 | 0.993 | 1.018 | 1.000 / 0.999 / 1.010 | 8,972 |
| `comment-inline-code` | comments | 285 | 1.004 | 0.991 | 1.023 | 1.002 / 1.002 / 1.006 | 215 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.005 | 0.992 | 1.015 | 1.010 / 1.005 / 1.001 | 1,467 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.005 | 0.995 | 1.014 | 1.010 / 1.002 / 1.005 | 10,175 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.006 | 0.988 | 1.017 | 1.006 / 1.016 / 1.002 | 2,941 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.006 | 1.000 | 1.020 | 1.006 / 1.005 / 1.007 | 883 |
| `vite-docs-api-plugin` | reference | 31890 | 1.007 | 0.969 | 1.064 | 1.003 / 1.008 / 0.987 | 65,651 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.007 | 0.985 | 1.013 | 1.009 / 0.994 / 1.006 | 7,359 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.008 | 0.998 | 1.012 | 1.008 / 1.008 / 1.006 | 5,088 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.008 | 0.967 | 1.082 | 1.001 / 0.997 / 1.024 | 49,034 |
| `comment-review` | comments | 282 | 1.009 | 0.995 | 1.064 | 1.009 / 1.007 / 1.015 | 153 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.010 | 1.001 | 1.459 | 1.011 / 1.012 / 1.006 | 1,265 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.012 | 0.969 | 1.070 | 0.996 / 0.998 / 1.033 | 118,628 |
| `comment-reproduction` | comments | 298 | 1.012 | 1.008 | 1.016 | 1.013 / 1.013 / 1.010 | 234 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.015 | 0.905 | 1.170 | 0.998 / 1.145 / 1.013 | 64,142 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.021 | 0.918 | 1.131 | 1.017 / 1.025 / 1.049 | 59,143 |
| `legacy-docs-readme` | readme | 1825 | 1.021 | 1.012 | 1.037 | 1.015 / 1.022 / 1.020 | 4,644 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-table` | comments | 310 | 0.986 | 0.961 | 1.014 | 0.988 / 0.987 / 0.986 | 876 |
| `vite-docs-features` | technical-docs | 39739 | 0.987 | 0.902 | 1.048 | 0.984 / 0.989 / 0.987 | 36,788 |
| `guard-angle-link` | syntax-guard | 41 | 0.991 | 0.957 | 0.999 | 0.990 / 0.989 / 0.997 | 153 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.992 | 0.850 | 1.134 | 0.964 / 1.004 / 1.014 | 33,913 |
| `vite-docs-api-plugin` | reference | 31890 | 0.993 | 0.938 | 1.017 | 0.995 / 0.968 / 1.007 | 49,355 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.994 | 0.973 | 1.008 | 1.000 / 0.991 / 0.994 | 1,328 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.994 | 0.952 | 1.001 | 0.995 / 0.994 / 0.994 | 45,502 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.995 | 0.971 | 1.009 | 0.995 / 1.005 / 0.976 | 17,619 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.995 | 0.982 | 1.007 | 0.994 / 0.994 / 1.000 | 1,634 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.995 | 0.988 | 1.057 | 0.994 / 0.995 / 0.996 | 28,741 |
| `legacy-contributing` | technical-docs | 9323 | 0.997 | 0.975 | 1.633 | 0.997 / 0.989 / 0.998 | 7,585 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.998 | 0.994 | 1.017 | 0.998 / 0.999 / 0.998 | 10,111 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.998 | 0.974 | 1.008 | 1.003 / 0.997 / 0.998 | 11,516 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.998 | 0.986 | 1.010 | 0.987 / 1.000 / 1.007 | 5,058 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.998 | 0.992 | 1.004 | 0.998 / 1.000 / 0.995 | 6,332 |
| `comment-incident` | comments | 1124 | 0.998 | 0.989 | 1.004 | 1.000 / 0.996 / 0.998 | 935 |
| `comment-links` | comments | 278 | 0.999 | 0.986 | 1.029 | 0.999 / 1.000 / 0.996 | 294 |
| `comment-question` | comments | 160 | 0.999 | 0.978 | 1.316 | 0.999 / 0.999 / 1.005 | 66 |
| `vite-docs-performance` | technical-docs | 8184 | 0.999 | 0.983 | 1.009 | 0.998 / 0.999 / 1.004 | 6,064 |
| `comment-review-long` | comments | 957 | 0.999 | 0.984 | 1.017 | 0.999 / 1.000 / 0.999 | 524 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.999 | 0.977 | 1.015 | 0.999 / 1.006 / 0.996 | 3,365 |
| `comment-unicode` | comments | 327 | 0.999 | 0.990 | 1.007 | 0.998 / 1.003 / 0.998 | 283 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.999 | 0.990 | 1.011 | 1.000 / 0.999 / 1.003 | 940 |
| `comment-quote` | comments | 290 | 0.999 | 0.992 | 1.009 | 0.999 / 1.002 / 0.998 | 163 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.999 | 0.992 | 1.631 | 0.997 / 0.999 / 1.007 | 1,624 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.000 | 0.969 | 1.023 | 1.009 / 0.995 / 0.986 | 4,922 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.000 | 0.993 | 1.025 | 1.000 / 1.003 / 0.998 | 1,020 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.000 | 0.980 | 1.023 | 0.994 / 1.000 / 1.000 | 938 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.000 | 0.498 | 1.014 | 1.000 / 0.999 / 1.000 | 1,813 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.000 | 0.993 | 1.011 | 1.005 / 0.999 / 1.000 | 4,396 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.000 | 0.988 | 1.011 | 0.995 / 1.000 / 1.008 | 372 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.000 | 0.987 | 1.016 | 0.998 / 1.001 / 1.003 | 7,966 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.981 | 1.020 | 0.987 / 1.009 / 0.996 | 5,783 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.001 | 0.987 | 1.012 | 0.994 / 1.002 / 0.997 | 2,529 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.001 | 0.947 | 1.039 | 0.992 / 1.001 / 1.020 | 21,155 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.002 | 0.980 | 1.009 | 1.004 / 0.999 / 1.002 | 1,734 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.002 | 0.982 | 1.008 | 1.002 / 1.004 / 1.001 | 574 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.002 | 0.911 | 1.183 | 1.003 / 1.001 / 1.002 | 14,270 |
| `comment-ack` | comments | 37 | 1.002 | 0.984 | 1.014 | 1.002 / 1.002 / 1.002 | 52 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.002 | 0.989 | 1.010 | 0.998 / 1.006 / 1.002 | 3,844 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.003 | 0.991 | 1.023 | 0.996 / 1.006 / 1.003 | 2,379 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.003 | 0.986 | 1.018 | 1.003 / 1.004 / 0.998 | 793 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.003 | 0.975 | 1.021 | 1.005 / 1.003 / 1.002 | 19,890 |
| `vue-docs-slots` | technical-docs | 24211 | 1.004 | 0.983 | 1.016 | 1.004 / 0.990 / 1.010 | 13,298 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.004 | 0.998 | 1.020 | 1.002 / 1.009 / 1.001 | 4,007 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.005 | 0.930 | 1.053 | 0.959 / 1.014 / 1.005 | 34,037 |
| `comment-checklist` | comments | 287 | 1.006 | 0.976 | 1.020 | 1.003 / 1.018 / 0.994 | 404 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.006 | 0.979 | 1.023 | 1.003 / 1.008 / 1.000 | 2,070 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.007 | 0.930 | 1.077 | 1.010 / 1.007 / 1.004 | 37,849 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.008 | 0.996 | 1.039 | 1.011 / 1.011 / 1.002 | 1,354 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.008 | 0.991 | 1.021 | 1.004 / 1.010 / 1.007 | 5,066 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.009 | 0.997 | 1.015 | 1.008 / 1.009 / 1.006 | 6,297 |
| `comment-reproduction` | comments | 298 | 1.011 | 0.803 | 1.337 | 1.020 / 1.010 / 1.011 | 173 |
| `comment-inline-code` | comments | 285 | 1.013 | 0.993 | 1.025 | 0.997 / 1.019 / 1.010 | 135 |
| `comment-review` | comments | 282 | 1.017 | 0.998 | 1.035 | 1.016 / 1.026 / 1.010 | 113 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.017 | 0.973 | 1.091 | 1.008 / 1.005 / 1.075 | 70,055 |
| `legacy-docs-readme` | readme | 1825 | 1.027 | 1.016 | 1.048 | 1.033 / 1.029 / 1.020 | 3,583 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.989 | 0.945 | 1.058 | 1.000 / 0.963 / 1.017 | 11,297 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.990 | 0.924 | 1.004 | 0.995 / 0.980 / 0.974 | 8,796 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.993 | 0.986 | 1.003 | 0.992 / 0.996 / 0.992 | 3,465 |
| `vite-docs-api-plugin` | reference | 31890 | 0.994 | 0.983 | 1.022 | 1.009 / 0.987 / 0.994 | 12,312 |
| `comment-table` | comments | 310 | 0.994 | 0.987 | 1.023 | 0.998 / 1.000 / 0.992 | 182 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.995 | 0.978 | 1.010 | 1.000 / 0.996 / 0.986 | 15,421 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.996 | 0.983 | 1.012 | 1.000 / 0.992 / 0.993 | 4,680 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.996 | 0.982 | 1.003 | 0.998 / 0.994 / 0.998 | 1,152 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.996 | 0.983 | 1.010 | 0.994 / 0.996 / 0.997 | 439 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.997 | 0.903 | 1.014 | 1.000 / 0.997 / 0.996 | 1,108 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.998 | 0.871 | 1.089 | 0.903 / 0.984 / 1.022 | 17,556 |
| `comment-links` | comments | 278 | 0.998 | 0.991 | 1.016 | 0.994 / 0.998 / 1.001 | 88 |
| `guard-angle-link` | syntax-guard | 41 | 0.998 | 0.991 | 1.029 | 0.998 / 0.998 / 0.995 | 31 |
| `comment-reproduction` | comments | 298 | 0.998 | 0.990 | 1.003 | 0.997 / 0.998 / 1.000 | 65 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.998 | 0.984 | 1.006 | 1.001 / 0.997 / 1.002 | 1,617 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.999 | 0.979 | 1.014 | 0.999 / 0.988 / 1.001 | 7,708 |
| `comment-unicode` | comments | 327 | 0.999 | 0.983 | 1.006 | 0.997 / 1.000 / 1.000 | 94 |
| `vite-docs-performance` | technical-docs | 8184 | 0.999 | 0.991 | 1.008 | 0.999 / 0.999 / 0.998 | 2,670 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.999 | 0.989 | 1.007 | 1.003 / 1.002 / 0.992 | 3,856 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.999 | 0.993 | 1.018 | 0.997 / 1.001 / 0.998 | 553 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.999 | 0.992 | 1.010 | 0.994 / 1.000 / 0.999 | 155 |
| `comment-checklist` | comments | 287 | 0.999 | 0.987 | 1.019 | 1.002 / 1.002 / 0.990 | 97 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.992 | 1.041 | 0.998 / 1.004 / 0.999 | 3,237 |
| `comment-incident` | comments | 1124 | 1.000 | 0.985 | 1.015 | 0.998 / 1.005 / 0.997 | 279 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.000 | 0.986 | 1.014 | 1.000 / 0.999 / 1.003 | 1,903 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.000 | 0.949 | 1.026 | 1.001 / 0.998 / 1.000 | 801 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.000 | 0.988 | 1.010 | 0.993 / 0.999 / 1.003 | 2,297 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.000 | 0.987 | 1.028 | 1.003 / 0.996 / 1.000 | 901 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.000 | 0.977 | 1.019 | 1.000 / 1.002 / 0.998 | 511 |
| `comment-review-long` | comments | 957 | 1.000 | 0.988 | 1.027 | 1.004 / 0.998 / 0.999 | 148 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.000 | 0.983 | 1.905 | 1.001 / 0.998 / 0.995 | 324 |
| `vue-docs-slots` | technical-docs | 24211 | 1.000 | 0.979 | 1.008 | 1.001 / 1.000 / 0.999 | 9,541 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.000 | 0.962 | 1.095 | 1.000 / 1.019 / 1.004 | 17,889 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.000 | 0.898 | 1.029 | 0.983 / 1.010 / 0.939 | 20,679 |
| `comment-question` | comments | 160 | 1.001 | 0.998 | 1.004 | 1.001 / 1.000 / 1.001 | 25 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.001 | 0.995 | 1.018 | 1.001 / 1.000 / 1.001 | 729 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.001 | 0.986 | 1.012 | 1.006 / 0.998 / 1.000 | 1,196 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.001 | 0.984 | 1.007 | 0.999 / 1.002 / 1.000 | 1,179 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.001 | 0.988 | 1.036 | 0.996 / 1.011 / 1.001 | 1,106 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.001 | 0.995 | 1.008 | 0.998 / 1.001 / 1.001 | 3,931 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.001 | 0.997 | 1.008 | 0.998 / 1.002 / 1.001 | 1,529 |
| `comment-ack` | comments | 37 | 1.001 | 0.998 | 1.018 | 1.002 / 1.001 / 1.001 | 19 |
| `legacy-contributing` | technical-docs | 9323 | 1.002 | 0.983 | 1.016 | 1.003 / 1.000 / 1.002 | 2,673 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.002 | 0.989 | 1.013 | 1.002 / 1.000 / 1.008 | 1,067 |
| `comment-review` | comments | 282 | 1.002 | 0.991 | 1.013 | 1.001 / 1.001 / 1.006 | 43 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.002 | 0.996 | 1.018 | 1.002 / 1.009 / 1.000 | 2,476 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.002 | 0.986 | 1.034 | 1.000 / 1.005 / 0.987 | 1,323 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.002 | 0.995 | 1.027 | 1.003 / 1.003 / 1.000 | 497 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.003 | 0.985 | 1.425 | 1.005 / 1.001 / 1.005 | 7,339 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.003 | 0.949 | 1.033 | 1.002 / 1.002 / 1.007 | 2,490 |
| `comment-inline-code` | comments | 285 | 1.003 | 0.985 | 1.031 | 1.009 / 1.000 / 1.005 | 82 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.003 | 0.991 | 1.014 | 1.004 / 1.002 / 0.998 | 56,156 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.003 | 0.974 | 1.026 | 1.014 / 0.993 / 0.999 | 2,590 |
| `legacy-docs-readme` | readme | 1825 | 1.003 | 0.965 | 1.063 | 1.003 / 1.009 / 0.997 | 1,044 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.004 | 0.937 | 1.195 | 1.053 / 0.993 / 1.004 | 33,957 |
| `comment-quote` | comments | 290 | 1.005 | 1.000 | 1.013 | 1.005 / 1.002 / 1.008 | 50 |
| `vite-docs-features` | technical-docs | 39739 | 1.007 | 0.985 | 1.131 | 1.004 / 1.007 / 1.010 | 16,995 |

