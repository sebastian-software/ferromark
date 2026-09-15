# Per-case tables — confirmation of `e2a3df70` against `fea50462`

Ratios are baseline time over candidate time (median of 15 paired windows); higher is faster.
`rounds` lists the per-round medians. Baseline time is the median nanoseconds per document.

## Original broad documents (57)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.989 | 0.726 | 1.106 | 1.009 / 0.844 / 0.985 | 70,139 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.001 | 0.992 | 1.012 | 0.999 / 1.006 / 0.998 | 81,850 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.002 | 0.995 | 1.020 | 1.002 / 1.004 / 1.000 | 55,485 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.007 | 0.968 | 1.025 | 1.015 / 1.007 / 1.004 | 29,084 |
| `vite-docs-api-plugin` | reference | 31890 | 1.009 | 0.964 | 1.048 | 1.021 / 0.997 / 1.009 | 72,324 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.012 | 0.873 | 1.392 | 1.012 / 0.943 / 1.014 | 3,405 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.014 | 0.709 | 1.153 | 1.016 / 1.011 / 1.014 | 2,486 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.016 | 0.993 | 1.041 | 1.014 / 1.016 / 1.016 | 10,069 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.016 | 0.216 | 1.738 | 1.016 / 0.944 / 1.017 | 4,242 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.019 | 0.336 | 1.130 | 1.011 / 1.019 / 1.028 | 69,038 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.020 | 1.009 | 1.029 | 1.020 / 1.014 / 1.021 | 7,791 |
| `legacy-docs-readme` | readme | 1825 | 1.020 | 1.004 | 1.033 | 1.028 / 1.020 / 1.017 | 4,925 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.022 | 0.742 | 1.293 | 1.022 / 1.096 / 1.019 | 1,777 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.024 | 0.983 | 1.044 | 1.023 / 1.024 / 1.019 | 16,331 |
| `vite-docs-performance` | technical-docs | 8184 | 1.025 | 0.808 | 1.442 | 1.026 / 1.007 / 1.022 | 9,512 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.025 | 0.607 | 1.529 | 1.025 / 1.214 / 1.019 | 1,646 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.025 | 0.715 | 1.312 | 1.023 / 0.964 / 1.034 | 5,885 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.026 | 1.017 | 1.510 | 1.025 / 1.122 / 1.026 | 2,872 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.027 | 1.008 | 1.039 | 1.036 / 1.027 / 1.017 | 1,944 |
| `legacy-contributing` | technical-docs | 9323 | 1.027 | 0.748 | 1.041 | 1.033 / 0.774 / 1.030 | 11,848 |
| `vue-docs-slots` | technical-docs | 24211 | 1.029 | 1.014 | 1.052 | 1.033 / 1.026 / 1.019 | 25,480 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.032 | 0.911 | 1.226 | 1.032 / 1.032 / 1.032 | 1,257 |
| `vite-docs-features` | technical-docs | 39739 | 1.032 | 0.857 | 1.091 | 1.059 / 0.951 / 1.041 | 67,583 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.033 | 1.013 | 1.047 | 1.039 / 1.028 / 1.033 | 13,002 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.033 | 1.010 | 1.057 | 1.031 / 1.033 / 1.045 | 11,145 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.037 | 0.957 | 1.072 | 1.019 / 1.049 / 1.040 | 60,906 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.038 | 0.759 | 1.069 | 1.045 / 0.955 / 1.047 | 8,700 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.039 | 0.957 | 1.064 | 1.007 / 1.043 / 1.038 | 122,803 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.040 | 0.991 | 1.115 | 1.036 / 1.030 / 1.055 | 36,163 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.041 | 1.027 | 1.061 | 1.035 / 1.043 / 1.044 | 8,498 |
| `comment-table` | comments | 310 | 1.041 | 1.026 | 1.050 | 1.039 / 1.041 / 1.041 | 1,350 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.041 | 1.021 | 1.068 | 1.033 / 1.042 / 1.037 | 4,421 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.041 | 0.736 | 1.278 | 1.047 / 1.035 / 1.036 | 18,303 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.042 | 1.019 | 1.052 | 1.032 / 1.038 / 1.046 | 8,938 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.044 | 1.030 | 1.115 | 1.046 / 1.044 / 1.040 | 887 |
| `comment-incident` | comments | 1124 | 1.046 | 1.018 | 1.058 | 1.046 / 1.054 / 1.044 | 2,179 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.047 | 1.036 | 1.088 | 1.042 / 1.047 / 1.067 | 3,524 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.049 | 0.795 | 1.394 | 1.046 / 1.058 / 1.052 | 6,123 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.050 | 0.720 | 1.086 | 1.050 / 0.921 / 1.055 | 24,781 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.051 | 1.007 | 1.063 | 1.043 / 1.047 / 1.055 | 3,819 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.053 | 1.043 | 1.076 | 1.052 / 1.064 / 1.053 | 10,689 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.056 | 1.013 | 1.080 | 1.056 / 1.077 / 1.035 | 59,293 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.058 | 1.028 | 1.075 | 1.058 / 1.063 / 1.053 | 2,913 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.059 | 1.045 | 1.093 | 1.055 / 1.060 / 1.055 | 6,445 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.065 | 1.056 | 1.081 | 1.064 / 1.064 / 1.068 | 3,515 |
| `comment-review-long` | comments | 957 | 1.075 | 0.882 | 1.164 | 1.062 / 1.082 / 1.076 | 1,000 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.077 | 1.059 | 1.117 | 1.073 / 1.080 / 1.093 | 1,956 |
| `comment-checklist` | comments | 287 | 1.094 | 1.075 | 1.126 | 1.088 / 1.093 / 1.094 | 836 |
| `comment-links` | comments | 278 | 1.094 | 0.991 | 1.410 | 1.093 / 1.140 / 1.106 | 689 |
| `comment-unicode` | comments | 327 | 1.107 | 1.085 | 1.123 | 1.111 / 1.103 / 1.116 | 680 |
| `comment-inline-code` | comments | 285 | 1.114 | 1.066 | 1.122 | 1.114 / 1.116 / 1.110 | 505 |
| `comment-reproduction` | comments | 298 | 1.114 | 1.094 | 1.141 | 1.114 / 1.103 / 1.116 | 548 |
| `comment-quote` | comments | 290 | 1.122 | 1.106 | 1.150 | 1.121 / 1.138 / 1.119 | 531 |
| `comment-review` | comments | 282 | 1.134 | 1.093 | 1.144 | 1.136 / 1.129 / 1.134 | 439 |
| `comment-question` | comments | 160 | 1.166 | 0.532 | 1.828 | 1.171 / 0.962 / 1.166 | 377 |
| `guard-angle-link` | syntax-guard | 41 | 1.178 | 0.867 | 1.235 | 1.178 / 1.117 / 1.187 | 482 |
| `comment-ack` | comments | 37 | 1.189 | 1.176 | 1.209 | 1.189 / 1.190 / 1.185 | 356 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `vite-docs-api-plugin` | reference | 31890 | 1.000 | 0.643 | 1.279 | 1.033 / 0.890 / 1.000 | 71,916 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.002 | 0.988 | 1.020 | 1.001 / 1.007 / 0.994 | 9,460 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.003 | 0.958 | 1.014 | 1.009 / 1.003 / 0.982 | 80,596 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.007 | 0.841 | 1.016 | 1.007 / 1.007 / 1.003 | 54,805 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.007 | 0.762 | 1.651 | 1.012 / 0.815 / 1.007 | 29,572 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.010 | 1.002 | 1.024 | 1.007 / 1.016 / 1.010 | 2,899 |
| `comment-links` | comments | 278 | 1.013 | 0.963 | 1.032 | 1.013 / 1.015 / 1.002 | 382 |
| `comment-table` | comments | 310 | 1.014 | 0.995 | 1.137 | 1.006 / 1.015 / 1.016 | 1,079 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.015 | 0.995 | 1.052 | 1.011 / 1.017 / 1.015 | 15,883 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.017 | 0.993 | 1.056 | 1.017 / 1.020 / 1.016 | 12,434 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.017 | 1.007 | 1.037 | 1.021 / 1.017 / 1.013 | 7,367 |
| `legacy-docs-readme` | readme | 1825 | 1.018 | 0.998 | 1.029 | 1.024 / 1.008 / 1.016 | 4,581 |
| `vite-docs-performance` | technical-docs | 8184 | 1.021 | 1.012 | 1.034 | 1.019 / 1.027 / 1.024 | 9,033 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.022 | 1.006 | 1.031 | 1.022 / 1.024 / 1.021 | 3,765 |
| `comment-review-long` | comments | 957 | 1.023 | 1.004 | 1.049 | 1.017 / 1.027 / 1.020 | 701 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.025 | 0.908 | 1.506 | 1.025 / 1.164 / 1.010 | 127,917 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.025 | 0.670 | 1.368 | 1.028 / 0.824 / 1.023 | 2,158 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.026 | 0.993 | 1.096 | 1.034 / 1.026 / 1.015 | 35,767 |
| `comment-reproduction` | comments | 298 | 1.028 | 1.002 | 1.063 | 1.024 / 1.028 / 1.028 | 265 |
| `comment-inline-code` | comments | 285 | 1.031 | 0.613 | 1.268 | 1.033 / 0.955 / 1.031 | 222 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.031 | 1.015 | 1.039 | 1.026 / 1.033 / 1.036 | 5,418 |
| `legacy-contributing` | technical-docs | 9323 | 1.032 | 1.019 | 1.046 | 1.036 / 1.025 / 1.029 | 11,303 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.033 | 1.024 | 1.039 | 1.031 / 1.033 / 1.033 | 1,485 |
| `vue-docs-slots` | technical-docs | 24211 | 1.034 | 1.013 | 1.042 | 1.037 / 1.032 / 1.034 | 24,565 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.034 | 1.018 | 1.047 | 1.033 / 1.033 / 1.039 | 8,238 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.034 | 0.467 | 1.215 | 1.043 / 0.984 / 1.022 | 68,403 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.034 | 1.014 | 1.074 | 1.043 / 1.036 / 1.026 | 1,607 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.037 | 1.006 | 1.059 | 1.042 / 1.029 / 1.056 | 8,085 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.038 | 1.026 | 1.058 | 1.032 / 1.030 / 1.048 | 2,557 |
| `comment-checklist` | comments | 287 | 1.038 | 1.023 | 1.059 | 1.035 / 1.054 / 1.038 | 541 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.039 | 1.022 | 1.054 | 1.041 / 1.049 / 1.032 | 1,308 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.040 | 0.748 | 1.094 | 1.032 / 1.048 / 1.050 | 7,672 |
| `vite-docs-features` | technical-docs | 39739 | 1.040 | 0.877 | 1.083 | 1.040 / 1.002 / 1.054 | 70,702 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.041 | 1.003 | 1.070 | 1.040 / 1.041 / 1.044 | 17,633 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.041 | 1.029 | 1.051 | 1.042 / 1.040 / 1.042 | 10,660 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.041 | 0.303 | 3.802 | 1.038 / 1.091 / 1.041 | 5,429 |
| `comment-review` | comments | 282 | 1.049 | 0.573 | 2.586 | 1.055 / 0.797 / 1.049 | 163 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.049 | 0.998 | 1.084 | 1.050 / 1.046 / 1.043 | 59,656 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.051 | 1.014 | 1.071 | 1.051 / 1.058 / 1.045 | 23,911 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.052 | 1.000 | 1.090 | 1.047 / 1.061 / 1.061 | 67,198 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.054 | 1.044 | 1.068 | 1.054 / 1.050 / 1.060 | 9,841 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.055 | 0.688 | 1.661 | 1.051 / 1.026 / 1.059 | 6,042 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.056 | 1.032 | 1.066 | 1.056 / 1.043 / 1.058 | 927 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.056 | 1.042 | 1.085 | 1.054 / 1.054 / 1.063 | 4,123 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.056 | 0.974 | 1.089 | 1.042 / 1.062 / 1.061 | 57,208 |
| `comment-quote` | comments | 290 | 1.058 | 1.030 | 1.078 | 1.057 / 1.060 / 1.058 | 233 |
| `comment-incident` | comments | 1124 | 1.059 | 0.996 | 1.164 | 1.061 / 1.059 / 1.058 | 1,865 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.061 | 1.047 | 1.095 | 1.060 / 1.065 / 1.061 | 3,449 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.063 | 1.039 | 1.085 | 1.065 / 1.050 / 1.064 | 556 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.066 | 1.030 | 1.088 | 1.063 / 1.067 / 1.066 | 3,159 |
| `comment-unicode` | comments | 327 | 1.066 | 0.996 | 2.015 | 1.054 / 1.065 / 1.076 | 401 |
| `comment-question` | comments | 160 | 1.070 | 1.053 | 1.093 | 1.074 / 1.071 / 1.066 | 96 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.072 | 1.054 | 1.088 | 1.072 / 1.073 / 1.070 | 3,161 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.080 | 1.034 | 1.092 | 1.072 / 1.083 / 1.085 | 2,613 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.086 | 1.076 | 1.105 | 1.086 / 1.089 / 1.081 | 1,597 |
| `comment-ack` | comments | 37 | 1.107 | 1.090 | 1.712 | 1.112 / 1.100 / 1.107 | 76 |
| `guard-angle-link` | syntax-guard | 41 | 1.170 | 1.132 | 1.254 | 1.170 / 1.231 / 1.152 | 206 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `vite-docs-features` | technical-docs | 39739 | 0.980 | 0.552 | 1.285 | 0.984 / 0.943 / 0.983 | 39,273 |
| `comment-reproduction` | comments | 298 | 0.987 | 0.972 | 1.001 | 0.989 / 0.989 / 0.985 | 177 |
| `comment-table` | comments | 310 | 0.990 | 0.981 | 1.003 | 0.992 / 0.986 / 0.989 | 858 |
| `comment-question` | comments | 160 | 0.990 | 0.942 | 1.006 | 0.990 / 0.979 / 0.993 | 66 |
| `vite-docs-performance` | technical-docs | 8184 | 0.992 | 0.970 | 1.006 | 0.993 / 0.996 / 0.990 | 5,980 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.993 | 0.951 | 1.615 | 0.992 / 1.169 / 0.988 | 3,558 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.994 | 0.860 | 1.524 | 0.988 / 1.138 / 0.997 | 1,888 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.996 | 0.982 | 1.019 | 0.992 / 0.999 / 0.996 | 46,081 |
| `legacy-docs-readme` | readme | 1825 | 0.996 | 0.985 | 1.025 | 0.996 / 0.998 / 0.994 | 3,352 |
| `legacy-contributing` | technical-docs | 9323 | 0.996 | 0.986 | 1.015 | 0.996 / 0.997 / 0.993 | 7,927 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.997 | 0.983 | 1.020 | 1.002 / 0.994 / 0.987 | 1,426 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.998 | 0.984 | 1.007 | 0.990 / 0.997 / 1.001 | 3,743 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.990 | 1.012 | 0.993 / 0.998 / 0.995 | 18,978 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.981 | 1.005 | 0.998 / 0.998 / 0.998 | 8,318 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.998 | 0.966 | 1.013 | 0.995 / 1.005 / 1.000 | 10,896 |
| `comment-links` | comments | 278 | 0.999 | 0.984 | 1.013 | 0.998 / 1.001 / 1.000 | 289 |
| `comment-inline-code` | comments | 285 | 0.999 | 0.984 | 1.109 | 0.995 / 0.995 / 1.002 | 134 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.810 | 1.198 | 1.000 / 1.007 / 1.000 | 5,940 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.001 | 0.989 | 1.015 | 1.007 / 1.000 / 1.001 | 6,487 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.001 | 0.985 | 1.019 | 0.996 / 1.001 / 1.005 | 3,914 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.002 | 0.991 | 1.049 | 1.005 / 0.996 / 1.002 | 4,583 |
| `comment-review-long` | comments | 957 | 1.002 | 0.994 | 1.012 | 1.000 / 1.002 / 1.004 | 546 |
| `comment-ack` | comments | 37 | 1.002 | 0.972 | 1.016 | 1.005 / 0.999 / 1.000 | 50 |
| `comment-review` | comments | 282 | 1.003 | 0.990 | 1.020 | 0.996 / 1.003 / 1.003 | 111 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.004 | 0.992 | 1.025 | 1.009 / 1.004 / 1.003 | 23,224 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.004 | 0.986 | 1.092 | 1.003 / 1.004 / 1.012 | 975 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.007 | 0.847 | 2.197 | 1.010 / 1.006 / 1.007 | 1,686 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.008 | 0.980 | 1.143 | 1.016 / 0.997 / 1.008 | 5,382 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.008 | 0.871 | 1.221 | 1.008 / 0.908 / 1.013 | 37,488 |
| `comment-quote` | comments | 290 | 1.008 | 0.997 | 1.029 | 1.012 / 1.008 / 1.007 | 174 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.008 | 0.896 | 1.485 | 1.012 / 1.004 / 1.008 | 5,199 |
| `comment-checklist` | comments | 287 | 1.008 | 0.986 | 1.024 | 1.009 / 1.008 / 1.008 | 424 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.009 | 0.994 | 1.034 | 1.011 / 1.003 / 1.011 | 2,057 |
| `vite-docs-api-plugin` | reference | 31890 | 1.011 | 0.952 | 1.399 | 1.003 / 1.036 / 1.011 | 52,999 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.011 | 0.995 | 1.255 | 1.010 / 1.017 / 1.009 | 1,792 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.012 | 1.002 | 1.026 | 1.009 / 1.005 / 1.014 | 369 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.014 | 0.804 | 1.514 | 1.003 / 1.053 / 1.009 | 14,909 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.016 | 0.861 | 1.040 | 1.015 / 1.009 / 1.023 | 2,399 |
| `vue-docs-slots` | technical-docs | 24211 | 1.016 | 0.993 | 1.034 | 1.019 / 1.010 / 1.016 | 13,966 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.016 | 1.001 | 1.026 | 1.016 / 1.008 / 1.020 | 2,634 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.018 | 0.555 | 1.226 | 1.029 / 0.759 / 1.023 | 21,492 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.019 | 0.995 | 1.025 | 1.020 / 1.011 / 1.007 | 1,345 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.020 | 0.988 | 1.033 | 1.025 / 1.006 / 1.020 | 918 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.020 | 1.006 | 1.036 | 1.020 / 1.019 / 1.025 | 1,754 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.020 | 0.990 | 1.099 | 1.016 / 1.020 / 1.029 | 71,453 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.022 | 1.008 | 1.033 | 1.018 / 1.023 / 1.026 | 1,044 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.022 | 0.789 | 1.246 | 1.025 / 0.877 / 1.018 | 15,061 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.023 | 1.010 | 1.043 | 1.024 / 1.016 / 1.023 | 6,514 |
| `comment-incident` | comments | 1124 | 1.024 | 1.003 | 1.047 | 1.024 / 1.025 / 1.023 | 1,457 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.028 | 1.003 | 1.240 | 1.020 / 1.026 / 1.035 | 821 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.029 | 0.935 | 1.754 | 1.013 / 1.256 / 1.031 | 34,679 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.029 | 0.774 | 1.293 | 1.029 / 1.259 / 1.033 | 4,787 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.044 | 0.708 | 1.261 | 1.044 / 0.878 / 1.051 | 38,973 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.047 | 1.002 | 1.067 | 1.048 / 1.048 / 1.041 | 578 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.060 | 0.974 | 1.574 | 1.060 / 1.310 / 1.053 | 35,787 |
| `comment-unicode` | comments | 327 | 1.064 | 1.045 | 1.083 | 1.066 / 1.057 / 1.066 | 293 |
| `guard-angle-link` | syntax-guard | 41 | 1.128 | 1.112 | 1.158 | 1.133 / 1.127 / 1.132 | 164 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.992 | 0.727 | 1.007 | 0.989 / 0.990 / 0.996 | 57,295 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.004 | 0.964 | 1.048 | 1.004 / 1.027 / 1.002 | 18,856 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.009 | 0.987 | 1.027 | 1.011 / 1.009 / 1.005 | 9,067 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.014 | 0.998 | 1.021 | 1.010 / 1.011 / 1.017 | 2,531 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.015 | 0.925 | 1.069 | 0.985 / 1.030 / 1.018 | 38,218 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.015 | 1.000 | 1.026 | 1.013 / 1.015 / 1.017 | 1,341 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.019 | 0.875 | 1.090 | 1.030 / 1.025 / 1.014 | 3,427 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.025 | 1.002 | 1.140 | 1.025 / 1.024 / 1.028 | 1,121 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.033 | 0.923 | 1.240 | 1.036 / 1.112 / 1.028 | 814 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.033 | 1.001 | 1.070 | 1.041 / 1.037 / 1.019 | 22,448 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.034 | 1.022 | 1.044 | 1.039 / 1.028 / 1.042 | 557 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.041 | 0.983 | 1.102 | 1.041 / 1.041 / 1.025 | 11,916 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.043 | 0.958 | 1.075 | 1.046 / 1.060 / 1.028 | 20,427 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.048 | 1.009 | 1.105 | 1.051 / 1.046 / 1.041 | 4,794 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.049 | 1.033 | 1.068 | 1.049 / 1.054 / 1.048 | 542 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.049 | 1.035 | 1.061 | 1.045 / 1.052 / 1.047 | 464 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.052 | 1.045 | 1.072 | 1.052 / 1.059 / 1.051 | 4,071 |
| `vue-docs-slots` | technical-docs | 24211 | 1.058 | 1.042 | 1.075 | 1.058 / 1.058 / 1.049 | 10,640 |
| `vite-docs-features` | technical-docs | 39739 | 1.061 | 1.011 | 1.114 | 1.062 / 1.072 / 1.042 | 19,736 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.063 | 1.027 | 1.077 | 1.069 / 1.060 / 1.058 | 7,909 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.068 | 0.845 | 1.888 | 1.074 / 1.251 / 1.067 | 2,740 |
| `comment-review-long` | comments | 957 | 1.072 | 1.064 | 1.078 | 1.072 / 1.071 / 1.074 | 160 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.076 | 1.067 | 1.099 | 1.078 / 1.072 / 1.073 | 341 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.079 | 0.780 | 3.109 | 1.076 / 1.100 / 1.079 | 1,650 |
| `vite-docs-api-plugin` | reference | 31890 | 1.080 | 1.007 | 1.330 | 1.076 / 1.101 / 1.078 | 13,883 |
| `comment-unicode` | comments | 327 | 1.081 | 1.069 | 1.105 | 1.080 / 1.085 / 1.081 | 103 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.094 | 1.063 | 1.114 | 1.094 / 1.102 / 1.078 | 4,031 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.095 | 1.083 | 1.101 | 1.097 / 1.095 / 1.094 | 4,498 |
| `vite-docs-performance` | technical-docs | 8184 | 1.097 | 0.667 | 1.444 | 1.099 / 1.068 / 1.093 | 3,030 |
| `comment-table` | comments | 310 | 1.100 | 0.121 | 1.583 | 1.100 / 1.103 / 1.095 | 199 |
| `comment-links` | comments | 278 | 1.100 | 1.083 | 1.109 | 1.095 / 1.104 / 1.100 | 96 |
| `comment-inline-code` | comments | 285 | 1.105 | 1.092 | 1.112 | 1.103 / 1.108 / 1.104 | 89 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.106 | 1.097 | 1.118 | 1.106 / 1.106 / 1.106 | 2,957 |
| `legacy-docs-readme` | readme | 1825 | 1.106 | 1.097 | 1.119 | 1.108 / 1.105 / 1.106 | 1,130 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.113 | 1.092 | 1.128 | 1.127 / 1.109 / 1.112 | 8,550 |
| `comment-checklist` | comments | 287 | 1.115 | 1.105 | 1.122 | 1.111 / 1.118 / 1.115 | 109 |
| `legacy-contributing` | technical-docs | 9323 | 1.115 | 0.739 | 1.403 | 1.115 / 1.031 / 1.115 | 3,227 |
| `comment-reproduction` | comments | 298 | 1.117 | 1.096 | 1.144 | 1.118 / 1.110 / 1.131 | 89 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.119 | 1.105 | 1.203 | 1.114 / 1.133 / 1.113 | 1,828 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.120 | 1.093 | 1.138 | 1.120 / 1.117 / 1.125 | 831 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.124 | 1.104 | 1.139 | 1.126 / 1.125 / 1.113 | 2,574 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.128 | 1.080 | 1.151 | 1.129 / 1.124 / 1.124 | 18,970 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.149 | 1.119 | 1.161 | 1.151 / 1.149 / 1.146 | 1,374 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.151 | 1.117 | 1.196 | 1.153 / 1.138 / 1.155 | 1,439 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.151 | 1.008 | 1.165 | 1.152 / 1.114 / 1.152 | 1,363 |
| `comment-incident` | comments | 1124 | 1.155 | 0.939 | 1.280 | 1.155 / 0.963 / 1.169 | 349 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.161 | 1.150 | 1.174 | 1.160 / 1.160 / 1.165 | 1,313 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.165 | 1.143 | 1.181 | 1.157 / 1.169 / 1.165 | 2,200 |
| `comment-quote` | comments | 290 | 1.179 | 1.161 | 1.193 | 1.173 / 1.169 / 1.180 | 58 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.179 | 1.168 | 1.193 | 1.181 / 1.174 / 1.172 | 178 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.183 | 0.902 | 1.523 | 1.183 / 1.078 / 1.188 | 1,437 |
| `comment-review` | comments | 282 | 1.187 | 1.174 | 1.199 | 1.190 / 1.187 / 1.187 | 51 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.188 | 1.167 | 1.197 | 1.192 / 1.188 / 1.187 | 1,153 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.241 | 1.225 | 1.263 | 1.237 / 1.245 / 1.235 | 621 |
| `guard-angle-link` | syntax-guard | 41 | 1.257 | 1.244 | 1.264 | 1.257 / 1.259 / 1.253 | 40 |
| `comment-question` | comments | 160 | 1.320 | 1.301 | 1.535 | 1.306 / 1.434 / 1.321 | 33 |
| `comment-ack` | comments | 37 | 1.446 | 0.977 | 5.840 | 1.446 / 1.463 / 1.444 | 28 |

## Authored diagnostics (45)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.994 | 0.973 | 1.004 | 0.989 / 0.994 / 0.996 | 16,753 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.996 | 0.977 | 1.033 | 0.999 / 0.995 / 0.996 | 33,227 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 0.998 | 0.984 | 1.011 | 0.999 / 1.002 / 0.994 | 13,017 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.001 | 0.992 | 1.014 | 0.998 / 1.000 / 1.001 | 31,322 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.001 | 0.993 | 1.014 | 1.003 / 1.001 / 0.999 | 121,136 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.004 | 0.963 | 1.019 | 1.017 / 1.000 / 0.991 | 8,379 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.005 | 0.659 | 1.039 | 1.005 / 0.667 / 1.025 | 4,310 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.006 | 1.000 | 1.077 | 1.012 / 1.001 / 1.006 | 21,721 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.007 | 0.751 | 2.212 | 1.087 / 1.007 / 1.007 | 7,021 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.008 | 0.984 | 1.165 | 1.008 / 1.035 / 1.004 | 11,464 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.009 | 0.998 | 1.020 | 1.003 / 1.009 / 1.010 | 9,895 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.009 | 0.981 | 1.028 | 1.001 / 1.012 / 1.011 | 14,948 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.011 | 0.994 | 1.038 | 1.014 / 1.003 / 1.012 | 5,743 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.013 | 0.756 | 1.034 | 1.002 / 1.017 / 1.019 | 4,754 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.014 | 0.989 | 1.026 | 1.015 / 1.012 / 1.012 | 4,634 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.016 | 1.000 | 1.366 | 1.009 / 1.036 / 1.013 | 3,648 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.017 | 1.002 | 1.027 | 1.013 / 1.014 / 1.021 | 3,480 |
| `table-dense-256` | table-diagnostic | 300 | 1.017 | 1.001 | 1.026 | 1.015 / 1.019 / 1.017 | 2,625 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.018 | 0.996 | 1.032 | 1.015 / 1.018 / 1.021 | 3,254 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.020 | 0.992 | 1.046 | 1.026 / 1.020 / 1.014 | 4,254 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 1.030 | 1.006 | 1.043 | 1.028 / 1.030 / 1.031 | 2,087 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.034 | 1.011 | 1.053 | 1.027 / 1.029 / 1.039 | 1,830 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.039 | 1.021 | 1.049 | 1.042 / 1.037 / 1.036 | 1,707 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.040 | 1.025 | 1.066 | 1.038 / 1.042 / 1.047 | 1,758 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 1.052 | 1.029 | 1.058 | 1.047 / 1.053 / 1.053 | 1,185 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.052 | 1.041 | 1.073 | 1.049 / 1.055 / 1.052 | 1,212 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.056 | 0.608 | 1.129 | 1.055 / 1.056 / 1.057 | 1,152 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.065 | 1.032 | 1.080 | 1.072 / 1.061 / 1.056 | 1,234 |
| `table-plain-256` | table-diagnostic | 299 | 1.068 | 1.035 | 1.082 | 1.074 / 1.051 / 1.068 | 775 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 1.070 | 1.056 | 1.098 | 1.071 / 1.067 / 1.091 | 679 |
| `table-sparse-256` | table-diagnostic | 297 | 1.070 | 1.063 | 1.092 | 1.079 / 1.063 / 1.069 | 811 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 1.096 | 1.084 | 1.115 | 1.096 / 1.102 / 1.096 | 647 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.097 | 1.081 | 1.108 | 1.102 / 1.102 / 1.095 | 30,498 |
| `table-formatted-256` | table-diagnostic | 278 | 1.101 | 0.965 | 1.123 | 1.094 / 1.101 / 1.106 | 2,456 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.101 | 1.073 | 1.129 | 1.096 / 1.101 / 1.118 | 508 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.105 | 1.079 | 1.117 | 1.111 / 1.100 / 1.105 | 115,740 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.115 | 1.105 | 1.145 | 1.115 / 1.112 / 1.123 | 513 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 1.117 | 1.099 | 1.137 | 1.117 / 1.120 / 1.110 | 565 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.121 | 1.100 | 1.145 | 1.125 / 1.117 / 1.121 | 749 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 1.137 | 1.119 | 1.176 | 1.172 / 1.132 / 1.137 | 471 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.139 | 0.858 | 2.597 | 1.139 / 1.141 / 1.133 | 509 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 1.145 | 1.127 | 1.165 | 1.151 / 1.134 / 1.144 | 463 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.247 | 1.218 | 1.302 | 1.221 / 1.247 / 1.278 | 491 |
| `scan-empty` | link-scan-diagnostic | 0 | 1.251 | 1.237 | 1.268 | 1.247 / 1.251 / 1.253 | 287 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.266 | 1.225 | 1.294 | 1.283 / 1.266 / 1.249 | 505 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.993 | 0.955 | 1.019 | 0.995 / 0.991 / 0.989 | 32,627 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.993 | 0.976 | 1.028 | 0.993 / 0.989 / 0.993 | 16,402 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 0.998 | 0.299 | 1.132 | 0.699 / 1.004 / 0.993 | 13,008 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.999 | 0.977 | 1.020 | 1.001 / 0.997 / 0.997 | 4,183 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.000 | 0.990 | 1.011 | 1.000 / 0.996 / 1.002 | 119,295 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.000 | 0.977 | 1.009 | 1.001 / 0.999 / 1.000 | 8,107 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.000 | 0.989 | 1.016 | 1.000 / 1.000 / 1.001 | 9,366 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.001 | 0.988 | 1.016 | 1.001 / 1.000 / 1.003 | 30,809 |
| `table-dense-256` | table-diagnostic | 300 | 1.002 | 0.994 | 1.022 | 1.001 / 1.001 / 1.002 | 2,329 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.002 | 0.977 | 1.017 | 1.004 / 1.002 / 0.996 | 14,882 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.003 | 0.922 | 1.039 | 0.995 / 1.003 / 1.011 | 1,317 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.004 | 0.989 | 1.027 | 1.004 / 1.010 / 1.001 | 11,182 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.004 | 0.985 | 1.018 | 1.004 / 1.003 / 1.007 | 3,855 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.005 | 0.978 | 1.019 | 1.005 / 1.002 / 1.012 | 4,502 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.006 | 0.994 | 1.027 | 1.009 / 1.006 / 1.006 | 6,727 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.008 | 1.000 | 1.019 | 1.014 / 1.004 / 1.008 | 21,762 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.009 | 0.993 | 1.022 | 1.017 / 1.006 / 1.013 | 5,152 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.009 | 1.000 | 1.034 | 1.012 / 1.002 / 1.013 | 3,706 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.010 | 0.993 | 1.029 | 1.010 / 1.010 / 1.023 | 1,455 |
| `table-sparse-256` | table-diagnostic | 297 | 1.012 | 0.979 | 1.043 | 1.017 / 1.010 / 1.006 | 519 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.015 | 1.004 | 1.046 | 1.011 / 1.018 / 1.017 | 3,069 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.015 | 1.003 | 1.024 | 1.012 / 1.015 / 1.016 | 2,821 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.015 | 1.007 | 1.035 | 1.015 / 1.015 / 1.017 | 3,146 |
| `table-plain-256` | table-diagnostic | 299 | 1.017 | 1.004 | 1.036 | 1.023 / 1.017 / 1.018 | 480 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 1.021 | 0.997 | 1.027 | 1.011 / 1.023 / 1.016 | 1,789 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.029 | 1.003 | 1.041 | 1.024 / 1.039 / 1.030 | 1,386 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.032 | 1.000 | 1.069 | 1.035 / 1.028 / 1.048 | 154 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.047 | 1.034 | 1.074 | 1.047 / 1.042 / 1.053 | 872 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 1.052 | 1.036 | 1.063 | 1.049 / 1.056 / 1.052 | 822 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.063 | 1.041 | 1.107 | 1.050 / 1.056 / 1.065 | 781 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.063 | 1.043 | 1.075 | 1.070 / 1.063 / 1.063 | 791 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.066 | 1.049 | 1.095 | 1.058 / 1.066 / 1.068 | 235 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.067 | 1.052 | 1.086 | 1.068 / 1.066 / 1.067 | 461 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.081 | 1.065 | 1.153 | 1.081 / 1.075 / 1.144 | 172 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.087 | 1.075 | 1.116 | 1.095 / 1.087 / 1.087 | 29,324 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.091 | 1.072 | 1.099 | 1.087 / 1.093 / 1.090 | 112,423 |
| `table-formatted-256` | table-diagnostic | 278 | 1.102 | 1.089 | 1.148 | 1.099 / 1.100 / 1.115 | 2,196 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 1.134 | 1.122 | 1.149 | 1.140 / 1.134 / 1.133 | 409 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 1.166 | 1.152 | 1.175 | 1.166 / 1.168 / 1.164 | 303 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 1.212 | 1.201 | 1.240 | 1.216 / 1.210 / 1.217 | 242 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.300 | 1.289 | 1.314 | 1.309 / 1.302 / 1.297 | 185 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.323 | 1.287 | 1.350 | 1.339 / 1.303 / 1.335 | 188 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 1.401 | 1.365 | 1.438 | 1.401 / 1.382 / 1.403 | 149 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 1.433 | 1.417 | 1.450 | 1.433 / 1.433 / 1.431 | 140 |
| `scan-empty` | link-scan-diagnostic | 0 | 1.569 | 1.551 | 1.583 | 1.568 / 1.576 / 1.561 | 34 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-short-link` | link-scan-diagnostic | 7 | 0.971 | 0.952 | 0.981 | 0.971 / 0.971 / 0.972 | 116 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 0.980 | 0.963 | 0.992 | 0.981 / 0.980 / 0.974 | 47 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.987 | 0.976 | 1.000 | 0.987 / 0.992 / 0.981 | 50 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 0.987 | 0.979 | 1.005 | 0.989 / 0.986 / 0.986 | 45 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.987 | 0.974 | 1.008 | 0.985 / 0.988 / 0.985 | 46 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.988 | 0.970 | 1.008 | 0.988 / 0.988 / 0.988 | 44 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 0.988 | 0.511 | 0.997 | 0.988 / 0.988 / 0.989 | 81 |
| `table-plain-256` | table-diagnostic | 299 | 0.989 | 0.984 | 1.018 | 0.993 / 0.989 / 0.988 | 404 |
| `table-sparse-256` | table-diagnostic | 297 | 0.991 | 0.980 | 1.004 | 0.992 / 0.991 / 0.988 | 432 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.991 | 0.975 | 1.000 | 0.988 / 0.991 / 0.991 | 14,463 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.992 | 0.978 | 1.002 | 0.992 / 0.992 / 0.989 | 29,924 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 0.993 | 0.981 | 1.013 | 0.993 / 0.995 / 0.992 | 83 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.994 | 0.990 | 1.019 | 0.996 / 0.996 / 0.994 | 86 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 0.994 | 0.979 | 1.010 | 0.988 / 0.994 / 0.995 | 5,702 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 0.995 | 0.985 | 1.007 | 0.992 / 1.001 / 1.001 | 84 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.995 | 0.984 | 1.021 | 0.994 / 0.995 / 0.999 | 1,162 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 0.996 | 0.980 | 1.007 | 1.004 / 0.988 / 0.996 | 43 |
| `scan-short-title` | link-scan-diagnostic | 11 | 0.996 | 0.984 | 1.010 | 1.005 / 0.996 / 0.996 | 125 |
| `scan-empty` | link-scan-diagnostic | 0 | 0.997 | 0.980 | 1.020 | 1.000 / 0.991 / 0.997 | 18 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.997 | 0.980 | 1.006 | 0.992 / 1.004 / 0.997 | 1,060 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.998 | 0.978 | 1.011 | 0.996 / 1.000 / 0.997 | 3,197 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 0.998 | 0.981 | 1.008 | 0.998 / 0.993 / 1.001 | 82 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.998 | 0.969 | 1.009 | 0.998 / 0.995 / 1.001 | 30,954 |
| `table-dense-256` | table-diagnostic | 300 | 0.999 | 0.884 | 1.022 | 0.976 / 0.999 / 1.003 | 2,267 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.999 | 0.980 | 1.023 | 1.002 / 1.000 / 0.999 | 3,403 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.000 | 0.986 | 1.016 | 1.000 / 1.000 / 1.000 | 209 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.000 | 0.956 | 1.031 | 0.996 / 0.999 / 1.002 | 8,991 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.001 | 0.983 | 1.020 | 1.006 / 1.002 / 0.987 | 3,537 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.001 | 0.981 | 1.009 | 0.997 / 1.001 / 1.006 | 211 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.001 | 0.983 | 1.013 | 1.003 / 1.003 / 0.990 | 10,412 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 1.002 | 0.978 | 1.017 | 1.004 / 1.003 / 1.000 | 86 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.002 | 0.984 | 1.026 | 1.007 / 1.002 / 0.998 | 211 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.003 | 0.583 | 1.152 | 1.013 / 0.996 / 1.003 | 12,831 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.004 | 0.987 | 1.012 | 1.002 / 1.004 / 1.004 | 3,000 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.004 | 0.944 | 1.315 | 1.127 / 1.007 / 1.001 | 214 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.005 | 0.994 | 1.018 | 1.007 / 1.001 / 1.005 | 120,633 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.006 | 0.988 | 1.017 | 1.008 / 0.998 / 1.008 | 215 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.007 | 0.987 | 1.018 | 1.010 / 1.004 / 1.008 | 214 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.009 | 1.003 | 1.026 | 1.022 / 1.009 / 1.006 | 9,565 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.009 | 0.994 | 1.026 | 1.016 / 1.006 / 1.004 | 5,796 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.010 | 0.995 | 1.021 | 1.013 / 1.010 / 0.999 | 401 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.037 | 1.016 | 1.059 | 1.032 / 1.040 / 1.038 | 186 |
| `table-formatted-256` | table-diagnostic | 278 | 1.109 | 1.095 | 1.123 | 1.107 / 1.109 / 1.109 | 1,805 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.114 | 1.060 | 1.131 | 1.105 / 1.115 / 1.116 | 24,215 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.117 | 1.103 | 1.146 | 1.121 / 1.115 / 1.131 | 94,163 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 0.996 | 0.981 | 1.003 | 0.993 / 0.998 / 0.997 | 2,387 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 0.999 | 0.978 | 1.008 | 0.999 / 0.990 / 1.003 | 2,191 |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.999 | 0.992 | 1.026 | 1.001 / 0.999 / 0.999 | 19,421 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.000 | 0.991 | 1.015 | 1.002 / 1.006 / 0.996 | 11,881 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.002 | 0.986 | 1.025 | 0.999 / 1.002 / 1.004 | 4,988 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.003 | 0.987 | 1.015 | 1.003 / 0.996 / 1.004 | 3,732 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 1.006 | 0.996 | 1.017 | 1.002 / 1.006 / 1.009 | 1,959 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.006 | 0.995 | 1.018 | 1.006 / 1.007 / 1.005 | 948 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.009 | 0.990 | 1.978 | 1.010 / 1.009 / 1.005 | 4,956 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.010 | 0.940 | 1.029 | 1.013 / 1.010 / 0.994 | 6,624 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.011 | 0.998 | 1.023 | 1.006 / 1.012 / 1.018 | 2,931 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.011 | 0.987 | 1.026 | 1.010 / 1.013 / 1.016 | 810 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.013 | 0.998 | 1.040 | 1.010 / 1.023 / 1.013 | 822 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.017 | 1.003 | 1.033 | 1.016 / 1.019 / 1.014 | 2,259 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.018 | 0.999 | 1.038 | 1.010 / 1.021 / 1.018 | 2,930 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.020 | 0.947 | 1.871 | 1.109 / 1.018 / 1.019 | 2,611 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.021 | 1.010 | 1.041 | 1.027 / 1.019 / 1.028 | 2,068 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 1.024 | 1.014 | 1.036 | 1.029 / 1.024 / 1.021 | 1,705 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.025 | 1.015 | 1.048 | 1.028 / 1.022 / 1.029 | 2,564 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.029 | 0.996 | 1.036 | 1.022 / 1.029 / 1.030 | 454 |
| `table-formatted-256` | table-diagnostic | 278 | 1.031 | 1.019 | 1.038 | 1.031 / 1.032 / 1.032 | 349 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.036 | 1.009 | 1.048 | 1.031 / 1.037 / 1.035 | 1,296 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.048 | 1.033 | 1.071 | 1.053 / 1.048 / 1.046 | 273 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.049 | 1.030 | 1.077 | 1.049 / 1.049 / 1.045 | 269 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.049 | 1.023 | 1.069 | 1.066 / 1.049 / 1.041 | 504 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.058 | 1.040 | 1.084 | 1.055 / 1.058 / 1.063 | 173 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.058 | 1.048 | 1.073 | 1.063 / 1.056 / 1.062 | 786 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 1.063 | 1.047 | 1.091 | 1.068 / 1.063 / 1.062 | 757 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.072 | 1.041 | 1.174 | 1.064 / 1.077 / 1.064 | 702 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.086 | 0.456 | 1.386 | 0.972 / 1.088 / 1.086 | 570 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 1.130 | 1.115 | 1.160 | 1.130 / 1.155 / 1.122 | 352 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.156 | 1.147 | 1.166 | 1.157 / 1.156 / 1.156 | 46 |
| `table-sparse-256` | table-diagnostic | 297 | 1.170 | 1.165 | 1.214 | 1.167 / 1.167 / 1.202 | 81 |
| `table-plain-256` | table-diagnostic | 299 | 1.175 | 1.160 | 1.196 | 1.176 / 1.175 / 1.173 | 81 |
| `table-dense-256` | table-diagnostic | 300 | 1.187 | 1.167 | 1.211 | 1.180 / 1.201 / 1.188 | 76 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.200 | 1.181 | 1.209 | 1.205 / 1.200 / 1.199 | 46 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.231 | 1.193 | 1.245 | 1.229 / 1.234 / 1.229 | 41 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 1.257 | 1.235 | 1.277 | 1.255 / 1.251 / 1.263 | 223 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.277 | 1.261 | 1.297 | 1.266 / 1.277 / 1.277 | 37 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 1.277 | 1.263 | 1.296 | 1.281 / 1.280 / 1.276 | 202 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.450 | 1.423 | 1.474 | 1.447 / 1.453 / 1.448 | 140 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.461 | 1.421 | 1.499 | 1.475 / 1.477 / 1.431 | 138 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 1.697 | 1.669 | 1.730 | 1.697 / 1.697 / 1.702 | 108 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 1.781 | 1.761 | 1.815 | 1.777 / 1.781 / 1.773 | 102 |
| `scan-empty` | link-scan-diagnostic | 0 | 2.713 | 2.621 | 2.763 | 2.685 / 2.719 / 2.713 | 19 |

