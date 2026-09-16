# Per-case tables — round 3 confirmation of the promoted set against f216b8da

Ratios are baseline time over candidate time (median of the paired windows); higher is faster.
`rounds` lists the per-round medians. Baseline time is the median nanoseconds per document.

## Original broad documents (57)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.977 | 0.936 | 1.034 | 0.969 / 0.977 / 0.979 | 55,764 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.996 | 0.957 | 1.049 | 1.013 / 0.994 / 0.994 | 32,815 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.999 | 0.931 | 1.052 | 0.978 / 0.993 / 1.012 | 63,201 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.000 | 0.975 | 1.070 | 1.006 / 0.993 / 1.014 | 114,283 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.006 | 0.992 | 1.015 | 1.005 / 1.006 / 1.007 | 54,612 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.015 | 0.936 | 1.116 | 0.986 / 1.044 / 1.015 | 63,568 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.020 | 1.005 | 1.032 | 1.008 / 1.022 / 1.024 | 17,276 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.024 | 1.005 | 1.038 | 1.026 / 1.005 / 1.022 | 7,543 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.027 | 1.009 | 1.039 | 1.025 / 1.034 / 1.027 | 8,382 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.033 | 1.021 | 1.050 | 1.028 / 1.034 / 1.036 | 9,655 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.034 | 1.025 | 1.049 | 1.034 / 1.040 / 1.030 | 80,656 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.034 | 1.020 | 1.049 | 1.035 / 1.034 / 1.028 | 12,497 |
| `legacy-docs-readme` | readme | 1825 | 1.036 | 1.002 | 1.058 | 1.038 / 1.041 / 1.024 | 4,708 |
| `vue-docs-slots` | technical-docs | 24211 | 1.040 | 1.006 | 1.062 | 1.040 / 1.046 / 1.029 | 24,289 |
| `vite-docs-performance` | technical-docs | 8184 | 1.044 | 1.031 | 1.058 | 1.044 / 1.045 / 1.034 | 9,089 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.045 | 1.034 | 1.057 | 1.052 / 1.045 / 1.044 | 10,642 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.046 | 1.035 | 1.059 | 1.052 / 1.046 / 1.037 | 9,745 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.047 | 1.040 | 1.056 | 1.051 / 1.045 / 1.045 | 6,009 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.048 | 1.022 | 1.059 | 1.033 / 1.050 / 1.034 | 22,986 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.049 | 1.036 | 1.067 | 1.046 / 1.056 / 1.052 | 8,067 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.050 | 1.028 | 1.069 | 1.045 / 1.050 / 1.053 | 4,063 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.050 | 1.016 | 1.068 | 1.054 / 1.049 / 1.049 | 15,757 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.052 | 1.041 | 1.059 | 1.054 / 1.052 / 1.051 | 3,556 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.053 | 1.010 | 1.067 | 1.053 / 1.038 / 1.056 | 5,706 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.055 | 1.017 | 1.092 | 1.057 / 1.050 / 1.061 | 3,248 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.056 | 1.017 | 1.066 | 1.059 / 1.049 / 1.061 | 28,736 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.057 | 1.052 | 1.075 | 1.056 / 1.065 / 1.056 | 7,817 |
| `vite-docs-features` | technical-docs | 39739 | 1.062 | 0.964 | 1.123 | 1.019 / 1.114 / 1.037 | 64,525 |
| `legacy-contributing` | technical-docs | 9323 | 1.063 | 0.831 | 1.084 | 1.063 / 1.065 / 1.060 | 11,138 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.064 | 0.998 | 1.139 | 1.077 / 1.026 / 1.034 | 53,924 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.066 | 1.056 | 1.076 | 1.058 / 1.064 / 1.074 | 5,564 |
| `vite-docs-api-plugin` | reference | 31890 | 1.067 | 1.023 | 1.098 | 1.084 / 1.027 / 1.070 | 68,032 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.073 | 1.053 | 1.094 | 1.071 / 1.074 / 1.067 | 2,404 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.096 | 1.087 | 1.107 | 1.094 / 1.099 / 1.095 | 4,179 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.098 | 1.075 | 1.122 | 1.108 / 1.092 / 1.092 | 2,738 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.106 | 1.095 | 1.114 | 1.106 / 1.111 / 1.103 | 1,780 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.109 | 1.059 | 1.127 | 1.106 / 1.109 / 1.107 | 1,735 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.111 | 1.099 | 1.118 | 1.107 / 1.114 / 1.111 | 3,282 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.113 | 1.086 | 1.119 | 1.114 / 1.113 / 1.115 | 3,226 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.120 | 1.105 | 1.135 | 1.120 / 1.127 / 1.116 | 1,847 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.122 | 1.104 | 1.139 | 1.113 / 1.131 / 1.122 | 2,741 |
| `comment-incident` | comments | 1124 | 1.133 | 1.117 | 1.149 | 1.133 / 1.123 / 1.137 | 2,049 |
| `comment-table` | comments | 310 | 1.146 | 1.137 | 1.150 | 1.146 / 1.149 / 1.141 | 1,266 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.149 | 1.131 | 1.156 | 1.150 / 1.149 / 1.141 | 1,551 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.156 | 1.140 | 1.175 | 1.155 / 1.156 / 1.160 | 1,184 |
| `comment-review-long` | comments | 957 | 1.224 | 1.209 | 1.231 | 1.225 / 1.220 / 1.224 | 919 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.230 | 1.208 | 1.237 | 1.227 / 1.227 / 1.234 | 829 |
| `comment-links` | comments | 278 | 1.328 | 1.313 | 1.343 | 1.318 / 1.338 / 1.331 | 610 |
| `comment-checklist` | comments | 287 | 1.330 | 1.319 | 1.338 | 1.330 / 1.330 / 1.323 | 751 |
| `comment-unicode` | comments | 327 | 1.353 | 1.339 | 1.391 | 1.350 / 1.351 / 1.371 | 612 |
| `comment-inline-code` | comments | 285 | 1.530 | 1.522 | 1.547 | 1.530 / 1.529 / 1.533 | 445 |
| `guard-angle-link` | syntax-guard | 41 | 1.537 | 1.521 | 1.553 | 1.537 / 1.542 / 1.533 | 406 |
| `comment-quote` | comments | 290 | 1.568 | 1.544 | 1.584 | 1.576 / 1.545 / 1.573 | 462 |
| `comment-reproduction` | comments | 298 | 1.594 | 1.556 | 1.620 | 1.594 / 1.595 / 1.594 | 478 |
| `comment-review` | comments | 282 | 1.664 | 1.624 | 1.676 | 1.667 / 1.672 / 1.658 | 381 |
| `comment-question` | comments | 160 | 1.925 | 1.896 | 2.703 | 1.925 / 1.918 / 1.937 | 318 |
| `comment-ack` | comments | 37 | 2.067 | 2.031 | 2.073 | 2.063 / 2.064 / 2.069 | 292 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.976 | 0.955 | 1.034 | 0.995 / 1.019 / 0.958 | 55,222 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.997 | 0.950 | 1.052 | 0.996 / 0.999 / 0.997 | 32,998 |
| `legacy-docs-readme` | readme | 1825 | 1.000 | 0.995 | 1.012 | 1.000 / 1.000 / 0.999 | 4,382 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.001 | 0.992 | 1.006 | 1.003 / 1.001 / 1.000 | 2,836 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.003 | 0.991 | 1.012 | 1.004 / 1.001 / 0.999 | 2,080 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.003 | 0.958 | 1.046 | 0.997 / 1.013 / 0.997 | 63,104 |
| `comment-table` | comments | 310 | 1.003 | 0.999 | 1.023 | 1.002 / 1.003 / 1.009 | 1,033 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.003 | 0.989 | 1.014 | 1.001 / 1.003 / 1.009 | 53,743 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.004 | 0.994 | 1.016 | 1.009 / 1.004 / 1.000 | 7,082 |
| `comment-question` | comments | 160 | 1.005 | 0.999 | 1.014 | 1.001 / 1.003 / 1.007 | 87 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.006 | 0.957 | 1.042 | 1.021 / 1.009 / 1.001 | 116,536 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.007 | 0.996 | 1.015 | 1.009 / 1.008 / 1.005 | 3,202 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.008 | 0.995 | 1.025 | 1.018 / 1.005 / 1.002 | 859 |
| `guard-angle-link` | syntax-guard | 41 | 1.010 | 0.989 | 1.020 | 1.003 / 1.014 / 1.017 | 175 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.011 | 0.998 | 1.022 | 1.011 / 1.004 / 1.011 | 3,632 |
| `comment-review` | comments | 282 | 1.012 | 1.000 | 1.026 | 1.012 / 1.013 / 1.008 | 150 |
| `comment-inline-code` | comments | 285 | 1.012 | 0.987 | 1.024 | 1.008 / 1.012 / 1.014 | 211 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.013 | 0.999 | 1.026 | 1.007 / 1.018 / 1.018 | 16,880 |
| `comment-ack` | comments | 37 | 1.013 | 0.996 | 1.026 | 1.016 / 1.015 / 1.012 | 66 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.014 | 0.994 | 1.031 | 1.010 / 1.012 / 1.018 | 7,857 |
| `comment-unicode` | comments | 327 | 1.014 | 1.001 | 1.023 | 1.017 / 1.010 / 1.013 | 364 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.014 | 1.002 | 1.025 | 1.009 / 1.024 / 1.014 | 1,403 |
| `comment-links` | comments | 278 | 1.016 | 1.011 | 1.030 | 1.015 / 1.020 / 1.016 | 375 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.018 | 0.990 | 1.037 | 1.014 / 1.014 / 1.021 | 518 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.018 | 1.000 | 1.041 | 1.014 / 1.019 / 1.019 | 9,056 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.019 | 1.011 | 1.038 | 1.021 / 1.017 / 1.019 | 1,471 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.021 | 1.004 | 1.037 | 1.026 / 1.021 / 1.015 | 5,602 |
| `comment-quote` | comments | 290 | 1.022 | 0.991 | 1.054 | 0.995 / 1.025 / 1.024 | 217 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.022 | 1.009 | 1.036 | 1.023 / 1.020 / 1.027 | 1,534 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.022 | 1.017 | 1.035 | 1.019 / 1.028 / 1.023 | 12,023 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.024 | 0.972 | 1.229 | 0.988 / 1.027 / 1.024 | 62,571 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.028 | 0.999 | 1.041 | 1.024 / 1.027 / 1.035 | 7,605 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.028 | 1.019 | 1.060 | 1.021 / 1.033 / 1.028 | 9,315 |
| `vite-docs-performance` | technical-docs | 8184 | 1.029 | 1.015 | 1.044 | 1.027 / 1.029 / 1.030 | 8,732 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.030 | 0.980 | 1.039 | 1.032 / 1.021 / 1.030 | 5,128 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.033 | 1.010 | 1.064 | 1.033 / 1.042 / 1.033 | 80,783 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.034 | 1.012 | 1.045 | 1.033 / 1.034 / 1.037 | 1,239 |
| `vite-docs-features` | technical-docs | 39739 | 1.034 | 0.942 | 1.172 | 1.119 / 1.032 / 1.007 | 64,485 |
| `comment-review-long` | comments | 957 | 1.035 | 1.019 | 1.040 | 1.035 / 1.034 / 1.035 | 677 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.036 | 1.022 | 1.066 | 1.041 / 1.042 / 1.034 | 5,135 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.037 | 1.005 | 1.077 | 1.048 / 1.036 / 1.036 | 27,947 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.037 | 1.018 | 1.054 | 1.024 / 1.037 / 1.043 | 2,429 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.038 | 1.021 | 1.049 | 1.030 / 1.044 / 1.038 | 15,302 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.039 | 1.011 | 1.057 | 1.042 / 1.036 / 1.039 | 10,189 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.041 | 1.018 | 1.052 | 1.031 / 1.041 / 1.044 | 22,175 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.041 | 1.027 | 1.070 | 1.043 / 1.039 / 1.044 | 7,170 |
| `vue-docs-slots` | technical-docs | 24211 | 1.044 | 1.016 | 1.057 | 1.042 / 1.051 / 1.045 | 23,838 |
| `legacy-contributing` | technical-docs | 9323 | 1.048 | 1.029 | 1.065 | 1.055 / 1.048 / 1.045 | 10,627 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.051 | 1.038 | 1.056 | 1.045 / 1.051 / 1.052 | 2,945 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.052 | 1.036 | 1.069 | 1.044 / 1.056 / 1.056 | 3,859 |
| `comment-checklist` | comments | 287 | 1.055 | 1.037 | 1.065 | 1.056 / 1.055 / 1.050 | 507 |
| `comment-incident` | comments | 1124 | 1.055 | 1.031 | 1.066 | 1.055 / 1.058 / 1.047 | 1,728 |
| `vite-docs-api-plugin` | reference | 31890 | 1.059 | 0.990 | 1.147 | 1.080 / 1.053 / 1.096 | 68,513 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.066 | 0.963 | 1.101 | 1.066 / 1.041 / 1.068 | 52,755 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.071 | 1.041 | 1.091 | 1.071 / 1.067 / 1.079 | 2,884 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.072 | 1.058 | 1.079 | 1.068 / 1.071 / 1.076 | 2,402 |
| `comment-reproduction` | comments | 298 | 1.134 | 1.104 | 1.143 | 1.125 / 1.134 / 1.138 | 252 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-unicode` | comments | 327 | 1.003 | 0.996 | 1.018 | 1.003 / 1.003 / 1.010 | 270 |
| `comment-question` | comments | 160 | 1.004 | 0.994 | 1.012 | 1.004 / 1.004 / 1.005 | 65 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 0.987 | 1.033 | 1.006 / 1.004 / 1.006 | 45,689 |
| `comment-review` | comments | 282 | 1.005 | 0.993 | 1.018 | 1.005 / 1.004 / 1.008 | 109 |
| `comment-inline-code` | comments | 285 | 1.007 | 0.993 | 1.026 | 1.007 / 1.003 / 1.008 | 131 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 1.008 | 0.987 | 1.012 | 1.008 / 1.006 / 1.008 | 1,749 |
| `legacy-docs-readme` | readme | 1825 | 1.008 | 0.995 | 1.017 | 1.009 / 1.008 / 1.008 | 3,365 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 1.010 | 1.000 | 1.017 | 1.014 / 1.011 / 1.009 | 6,274 |
| `wiki-tea-lead` | encyclopedia | 6363 | 1.010 | 0.995 | 1.098 | 1.013 / 1.006 / 1.010 | 4,622 |
| `comment-table` | comments | 310 | 1.010 | 1.004 | 1.025 | 1.016 / 1.010 / 1.010 | 855 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.011 | 0.993 | 1.031 | 1.011 / 1.011 / 1.005 | 1,299 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.011 | 1.000 | 1.030 | 1.016 / 1.011 / 1.010 | 535 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.015 | 0.988 | 1.032 | 1.024 / 1.015 / 1.015 | 32,452 |
| `guard-angle-link` | syntax-guard | 41 | 1.015 | 1.011 | 1.033 | 1.015 / 1.014 / 1.016 | 145 |
| `comment-links` | comments | 278 | 1.016 | 1.004 | 1.029 | 1.016 / 1.016 / 1.018 | 283 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.017 | 0.991 | 1.049 | 1.018 / 1.004 / 1.023 | 19,917 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.017 | 1.000 | 1.026 | 1.007 / 1.017 / 1.021 | 2,020 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 1.018 | 1.008 | 1.029 | 1.020 / 1.018 / 1.018 | 886 |
| `wiki-chess-lead` | encyclopedia | 4125 | 1.019 | 1.001 | 1.031 | 1.014 / 1.019 / 1.017 | 2,310 |
| `comment-ack` | comments | 37 | 1.019 | 1.002 | 1.045 | 1.017 / 1.023 / 1.019 | 49 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.020 | 1.012 | 1.030 | 1.020 / 1.025 / 1.019 | 14,507 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.020 | 0.979 | 1.070 | 1.020 / 1.028 / 1.010 | 36,234 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.024 | 1.008 | 1.043 | 1.028 / 1.017 / 1.029 | 5,874 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.028 | 1.020 | 1.037 | 1.026 / 1.029 / 1.030 | 360 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.029 | 1.008 | 1.040 | 1.019 / 1.033 / 1.029 | 947 |
| `comment-quote` | comments | 290 | 1.031 | 1.021 | 1.041 | 1.031 / 1.030 / 1.035 | 171 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.032 | 1.011 | 1.048 | 1.032 / 1.038 / 1.032 | 3,676 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.034 | 1.015 | 1.041 | 1.033 / 1.034 / 1.037 | 5,025 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.034 | 1.003 | 1.047 | 1.041 / 1.034 / 1.022 | 6,401 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.036 | 1.025 | 1.058 | 1.033 / 1.036 / 1.046 | 3,877 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.038 | 1.025 | 1.054 | 1.035 / 1.041 / 1.036 | 8,195 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.039 | 1.027 | 1.057 | 1.036 / 1.039 / 1.048 | 14,508 |
| `vite-docs-performance` | technical-docs | 8184 | 1.039 | 1.022 | 1.057 | 1.039 / 1.034 / 1.040 | 5,950 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.039 | 1.013 | 1.051 | 1.034 / 1.042 / 1.042 | 4,519 |
| `vue-docs-slots` | technical-docs | 24211 | 1.039 | 1.019 | 1.066 | 1.040 / 1.039 / 1.039 | 13,376 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.039 | 1.026 | 1.061 | 1.042 / 1.041 / 1.034 | 1,655 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.040 | 0.952 | 1.084 | 1.028 / 1.040 / 1.041 | 67,742 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.041 | 1.026 | 1.056 | 1.041 / 1.040 / 1.046 | 5,156 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.045 | 1.000 | 1.089 | 1.020 / 1.045 / 1.054 | 781 |
| `legacy-contributing` | technical-docs | 9323 | 1.048 | 1.036 | 1.061 | 1.040 / 1.049 / 1.047 | 7,722 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.048 | 1.036 | 1.069 | 1.056 / 1.043 / 1.052 | 2,566 |
| `comment-review-long` | comments | 957 | 1.052 | 1.023 | 1.060 | 1.052 / 1.052 / 1.051 | 531 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.052 | 1.032 | 1.056 | 1.050 / 1.055 / 1.052 | 3,525 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.052 | 1.029 | 1.058 | 1.055 / 1.043 / 1.053 | 1,005 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.054 | 1.045 | 1.064 | 1.055 / 1.049 / 1.058 | 10,606 |
| `vite-docs-features` | technical-docs | 39739 | 1.056 | 1.030 | 1.092 | 1.059 / 1.056 / 1.040 | 37,822 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.056 | 1.039 | 1.067 | 1.056 / 1.056 / 1.056 | 1,805 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.056 | 0.996 | 1.111 | 1.031 / 1.064 / 1.049 | 35,244 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.063 | 1.042 | 1.080 | 1.061 / 1.063 / 1.073 | 18,605 |
| `comment-incident` | comments | 1124 | 1.063 | 1.047 | 1.079 | 1.058 / 1.067 / 1.064 | 1,419 |
| `vite-docs-api-plugin` | reference | 31890 | 1.063 | 1.009 | 1.082 | 1.066 / 1.063 / 1.049 | 49,942 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.064 | 1.033 | 1.082 | 1.065 / 1.057 / 1.062 | 33,330 |
| `comment-checklist` | comments | 287 | 1.070 | 1.049 | 1.090 | 1.054 / 1.070 / 1.075 | 415 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.075 | 1.059 | 1.098 | 1.073 / 1.078 / 1.072 | 1,683 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.087 | 1.070 | 1.096 | 1.083 / 1.089 / 1.089 | 1,418 |
| `comment-reproduction` | comments | 298 | 1.100 | 1.078 | 1.120 | 1.088 / 1.102 / 1.100 | 175 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.154 | 1.138 | 1.171 | 1.144 / 1.158 / 1.151 | 22,902 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.939 | 0.817 | 1.012 | 0.939 / 0.946 / 0.900 | 34,264 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.948 | 0.926 | 1.053 | 0.953 / 0.940 / 0.953 | 17,858 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.951 | 0.876 | 1.085 | 0.961 / 0.932 / 0.961 | 21,110 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.963 | 0.931 | 0.970 | 0.943 / 0.963 / 0.965 | 7,375 |
| `comment-table` | comments | 310 | 0.968 | 0.955 | 0.978 | 0.973 / 0.959 / 0.968 | 177 |
| `legacy-docs-readme` | readme | 1825 | 0.972 | 0.963 | 0.999 | 0.971 / 0.974 / 0.972 | 1,020 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.989 | 0.974 | 0.996 | 0.989 / 0.989 / 0.989 | 310 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.989 | 0.953 | 1.026 | 0.977 / 0.989 / 0.991 | 11,170 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.990 | 0.962 | 1.011 | 0.994 / 0.994 / 0.985 | 780 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.990 | 0.966 | 1.004 | 0.990 / 0.987 / 0.993 | 531 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.990 | 0.968 | 1.010 | 0.984 / 0.993 / 0.981 | 2,496 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.992 | 0.972 | 1.003 | 0.991 / 0.993 / 0.990 | 149 |
| `comment-links` | comments | 278 | 0.993 | 0.986 | 1.017 | 0.991 / 0.993 / 0.995 | 86 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.993 | 0.968 | 1.014 | 0.995 / 0.992 / 0.993 | 1,485 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.993 | 0.976 | 1.011 | 0.993 / 0.994 / 0.992 | 506 |
| `comment-quote` | comments | 290 | 0.994 | 0.988 | 1.006 | 0.991 / 0.998 / 0.995 | 49 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.994 | 0.988 | 1.009 | 0.994 / 0.994 / 0.999 | 1,164 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.994 | 0.989 | 1.005 | 0.996 / 0.993 / 0.993 | 437 |
| `comment-inline-code` | comments | 285 | 0.994 | 0.969 | 1.002 | 0.995 / 0.978 / 0.995 | 79 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.995 | 0.977 | 1.039 | 1.003 / 0.995 / 0.993 | 498 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.995 | 0.987 | 1.007 | 0.996 / 0.995 / 0.993 | 1,307 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.995 | 0.976 | 1.011 | 0.993 / 0.997 / 0.994 | 1,197 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.995 | 0.984 | 1.005 | 0.996 / 0.995 / 0.995 | 1,060 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.996 | 0.990 | 1.005 | 0.998 / 0.994 / 1.002 | 2,278 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.997 | 0.977 | 1.025 | 0.997 / 0.998 / 0.993 | 57,066 |
| `comment-review` | comments | 282 | 0.998 | 0.985 | 1.020 | 1.005 / 0.999 / 0.994 | 43 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.998 | 0.981 | 1.009 | 0.996 / 0.999 / 0.995 | 1,861 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.973 | 1.024 | 0.990 / 0.988 / 0.999 | 8,728 |
| `comment-ack` | comments | 37 | 0.999 | 0.988 | 1.010 | 0.998 / 0.999 / 1.004 | 19 |
| `comment-unicode` | comments | 327 | 1.000 | 0.988 | 1.007 | 0.998 / 1.001 / 1.003 | 93 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.982 | 1.112 | 0.996 / 1.000 / 1.005 | 3,249 |
| `guard-angle-link` | syntax-guard | 41 | 1.000 | 0.994 | 1.018 | 1.000 / 0.999 / 0.997 | 31 |
| `comment-question` | comments | 160 | 1.000 | 0.959 | 1.021 | 0.993 / 1.000 / 1.006 | 25 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.000 | 0.980 | 1.012 | 1.000 / 1.001 / 0.995 | 1,590 |
| `comment-review-long` | comments | 957 | 1.001 | 0.989 | 1.039 | 0.997 / 1.000 / 1.012 | 148 |
| `comment-checklist` | comments | 287 | 1.002 | 0.997 | 1.028 | 1.002 / 1.003 / 1.001 | 97 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 1.003 | 0.976 | 3.980 | 1.003 / 0.999 / 1.015 | 3,770 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.004 | 0.984 | 1.032 | 1.005 / 1.004 / 1.000 | 4,561 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.009 | 0.390 | 1.072 | 0.980 / 1.009 / 1.036 | 18,900 |
| `vite-docs-performance` | technical-docs | 8184 | 1.010 | 0.991 | 1.042 | 1.011 / 1.012 / 1.008 | 2,689 |
| `vue-docs-suspense` | technical-docs | 8291 | 1.016 | 1.002 | 1.027 | 1.019 / 1.013 / 1.021 | 4,062 |
| `vite-docs-features` | technical-docs | 39739 | 1.021 | 0.916 | 1.173 | 1.021 / 1.027 / 1.021 | 18,087 |
| `legacy-docs-mdx` | technical-docs | 7422 | 1.025 | 0.990 | 1.046 | 1.027 / 1.022 / 1.032 | 2,522 |
| `vite-docs-api-plugin` | reference | 31890 | 1.035 | 1.019 | 1.050 | 1.031 / 1.035 / 1.039 | 12,535 |
| `legacy-node-ferromark-readme` | readme | 9075 | 1.036 | 1.027 | 1.044 | 1.036 / 1.036 / 1.030 | 3,666 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.037 | 0.966 | 1.074 | 1.037 / 1.044 / 1.028 | 16,509 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.038 | 1.018 | 1.053 | 1.038 / 1.046 / 1.027 | 7,579 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 1.040 | 1.030 | 1.054 | 1.038 / 1.040 / 1.040 | 737 |
| `comment-incident` | comments | 1124 | 1.048 | 1.021 | 1.067 | 1.048 / 1.047 / 1.056 | 296 |
| `vue-docs-slots` | technical-docs | 24211 | 1.049 | 1.037 | 1.067 | 1.051 / 1.047 / 1.044 | 9,845 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 1.050 | 1.042 | 1.062 | 1.050 / 1.050 / 1.050 | 1,104 |
| `legacy-contributing` | technical-docs | 9323 | 1.052 | 1.006 | 1.069 | 1.051 / 1.051 / 1.053 | 2,802 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.052 | 1.039 | 1.090 | 1.050 / 1.064 / 1.048 | 2,648 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.059 | 1.049 | 1.068 | 1.057 / 1.062 / 1.059 | 1,238 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.071 | 1.057 | 1.093 | 1.066 / 1.087 / 1.067 | 961 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.077 | 1.063 | 1.099 | 1.077 / 1.071 / 1.082 | 1,200 |
| `comment-reproduction` | comments | 298 | 1.202 | 1.196 | 1.234 | 1.202 / 1.203 / 1.203 | 78 |

## Authored diagnostics (45)

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 0.933 | 0.922 | 0.939 | 0.929 / 0.933 / 0.931 | 6,950 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 0.983 | 0.941 | 0.988 | 0.985 / 0.983 / 0.980 | 2,013 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.996 | 0.991 | 1.014 | 0.997 / 0.995 / 0.995 | 16,585 |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.999 | 0.990 | 1.018 | 1.002 / 0.996 / 1.004 | 104,392 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.001 | 0.978 | 1.010 | 1.000 / 1.007 / 0.996 | 27,223 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.001 | 0.984 | 1.005 | 0.993 / 1.002 / 1.001 | 119,034 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.005 | 0.981 | 1.020 | 1.011 / 1.005 / 1.003 | 31,349 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.010 | 0.995 | 1.024 | 1.007 / 1.013 / 1.005 | 14,819 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.014 | 1.003 | 1.020 | 1.015 / 1.013 / 1.013 | 4,669 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.015 | 1.010 | 1.030 | 1.015 / 1.017 / 1.021 | 9,812 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.018 | 0.996 | 1.029 | 1.008 / 1.020 / 1.018 | 11,352 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.020 | 1.002 | 1.036 | 1.019 / 1.020 / 1.024 | 12,998 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.021 | 1.011 | 1.057 | 1.022 / 1.021 / 1.021 | 32,860 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.024 | 1.014 | 1.036 | 1.029 / 1.023 / 1.024 | 8,397 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.024 | 1.013 | 1.036 | 1.022 / 1.026 / 1.024 | 21,656 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.036 | 0.967 | 1.061 | 0.999 / 1.045 / 1.035 | 4,580 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.039 | 1.021 | 1.053 | 1.037 / 1.040 / 1.039 | 4,193 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.051 | 1.013 | 1.079 | 1.051 / 1.050 / 1.055 | 3,432 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.053 | 1.033 | 1.070 | 1.053 / 1.064 / 1.044 | 5,539 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.059 | 1.046 | 1.075 | 1.062 / 1.054 / 1.059 | 4,066 |
| `table-dense-256` | table-diagnostic | 300 | 1.065 | 1.055 | 1.074 | 1.062 / 1.068 / 1.065 | 2,551 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.071 | 1.057 | 1.089 | 1.076 / 1.071 / 1.075 | 3,185 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.073 | 1.067 | 1.087 | 1.070 / 1.075 / 1.078 | 3,494 |
| `table-formatted-256` | table-diagnostic | 278 | 1.084 | 1.059 | 1.099 | 1.081 / 1.085 / 1.088 | 2,203 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.101 | 1.089 | 1.127 | 1.105 / 1.104 / 1.096 | 1,777 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.109 | 1.089 | 1.132 | 1.109 / 1.115 / 1.103 | 1,651 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.111 | 1.090 | 1.157 | 1.110 / 1.120 / 1.106 | 1,681 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 1.156 | 1.134 | 1.164 | 1.160 / 1.156 / 1.149 | 1,127 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.169 | 1.155 | 1.200 | 1.171 / 1.173 / 1.159 | 1,157 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.170 | 1.108 | 1.209 | 1.160 / 1.173 / 1.177 | 1,168 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.190 | 1.180 | 1.194 | 1.190 / 1.191 / 1.191 | 1,055 |
| `table-sparse-256` | table-diagnostic | 297 | 1.279 | 1.261 | 1.296 | 1.279 / 1.275 / 1.288 | 751 |
| `table-plain-256` | table-diagnostic | 299 | 1.284 | 1.277 | 1.304 | 1.283 / 1.283 / 1.293 | 711 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.286 | 1.257 | 1.315 | 1.303 / 1.271 / 1.286 | 670 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 1.296 | 1.290 | 1.327 | 1.296 / 1.324 / 1.296 | 623 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 1.345 | 1.335 | 1.365 | 1.345 / 1.346 / 1.343 | 588 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 1.403 | 1.362 | 1.539 | 1.392 / 1.407 / 1.418 | 506 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.474 | 1.448 | 1.490 | 1.485 / 1.475 / 1.453 | 458 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.498 | 1.461 | 1.512 | 1.480 / 1.498 / 1.508 | 446 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.548 | 1.529 | 1.565 | 1.540 / 1.549 / 1.545 | 453 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 1.560 | 1.543 | 1.580 | 1.561 / 1.560 / 1.556 | 407 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 1.574 | 1.558 | 1.599 | 1.567 / 1.572 / 1.590 | 398 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.577 | 1.519 | 1.602 | 1.586 / 1.587 / 1.529 | 389 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.595 | 1.578 | 1.647 | 1.588 / 1.644 / 1.593 | 385 |
| `scan-empty` | link-scan-diagnostic | 0 | 2.696 | 2.628 | 2.747 | 2.698 / 2.683 / 2.678 | 227 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `autolink-closers-512` | autolink-diagnostic | 545 | 0.907 | 0.903 | 0.921 | 0.907 / 0.906 / 0.911 | 1,731 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 0.912 | 0.899 | 0.922 | 0.914 / 0.912 / 0.908 | 6,627 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.943 | 0.934 | 0.962 | 0.942 / 0.944 / 0.941 | 198 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.978 | 0.967 | 0.996 | 0.978 / 0.980 / 0.976 | 358 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.979 | 0.963 | 0.987 | 0.979 / 0.976 / 0.979 | 4,445 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 0.979 | 0.893 | 0.993 | 0.979 / 0.985 / 0.964 | 98 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.983 | 0.976 | 0.999 | 0.980 / 0.982 / 0.986 | 105 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.988 | 0.971 | 1.008 | 0.988 / 0.987 / 0.996 | 16,491 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 0.994 | 0.969 | 1.001 | 0.995 / 0.988 / 0.996 | 9,380 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.995 | 0.982 | 1.007 | 0.997 / 0.994 / 0.995 | 260 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 0.997 | 0.988 | 1.010 | 0.998 / 0.997 / 0.994 | 14,548 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.997 | 0.987 | 1.025 | 0.999 / 0.993 / 1.005 | 3,791 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.999 | 0.989 | 1.010 | 0.996 / 1.000 / 0.999 | 30,746 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.999 | 0.991 | 1.027 | 0.997 / 1.004 / 0.999 | 1,306 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.000 | 0.985 | 1.008 | 1.002 / 0.992 / 1.001 | 140 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.000 | 0.991 | 1.022 | 0.999 / 1.003 / 1.000 | 4,171 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.000 | 0.987 | 1.008 | 0.998 / 1.000 / 1.001 | 103,353 |
| `table-dense-256` | table-diagnostic | 300 | 1.002 | 0.991 | 1.016 | 1.002 / 1.002 / 0.998 | 2,305 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.002 | 0.991 | 1.011 | 1.003 / 1.002 / 1.000 | 1,431 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.002 | 0.981 | 1.021 | 0.983 / 1.008 / 1.002 | 215 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.003 | 0.992 | 1.012 | 1.004 / 1.006 / 0.998 | 27,138 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 1.003 | 0.993 | 1.007 | 1.003 / 1.002 / 1.003 | 782 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.003 | 0.986 | 1.018 | 0.997 / 1.003 / 1.004 | 120,153 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.004 | 0.983 | 1.017 | 1.004 / 1.005 / 1.004 | 3,012 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.005 | 1.000 | 1.021 | 1.005 / 1.004 / 1.006 | 736 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.006 | 0.993 | 1.017 | 1.006 / 1.004 / 1.011 | 12,549 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.007 | 0.995 | 1.015 | 1.006 / 1.007 / 1.007 | 8,114 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.007 | 0.967 | 1.018 | 1.005 / 1.007 / 1.013 | 144 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.008 | 0.999 | 1.020 | 1.006 / 1.009 / 1.008 | 11,052 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.009 | 0.995 | 1.029 | 1.009 / 1.011 / 1.008 | 156 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.009 | 1.004 | 1.021 | 1.009 / 1.008 / 1.010 | 1,342 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.009 | 1.000 | 1.015 | 1.012 / 1.008 / 1.008 | 423 |
| `table-formatted-256` | table-diagnostic | 278 | 1.010 | 0.999 | 1.019 | 1.005 / 1.010 / 1.010 | 1,945 |
| `table-plain-256` | table-diagnostic | 299 | 1.011 | 0.992 | 1.029 | 1.006 / 1.011 / 1.011 | 473 |
| `table-sparse-256` | table-diagnostic | 297 | 1.016 | 1.007 | 1.025 | 1.015 / 1.016 / 1.019 | 506 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.017 | 1.003 | 1.029 | 1.021 / 1.016 / 1.013 | 21,411 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.021 | 0.997 | 1.029 | 1.022 / 1.021 / 1.020 | 32,530 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.022 | 1.014 | 1.029 | 1.027 / 1.019 / 1.021 | 5,132 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.023 | 1.012 | 1.047 | 1.033 / 1.016 / 1.023 | 810 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.024 | 1.009 | 1.032 | 1.027 / 1.025 / 1.023 | 711 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.024 | 1.009 | 1.046 | 1.017 / 1.024 / 1.041 | 3,667 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.028 | 1.011 | 1.039 | 1.028 / 1.026 / 1.028 | 2,779 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.028 | 1.014 | 1.042 | 1.024 / 1.024 / 1.032 | 3,116 |
| `scan-empty` | link-scan-diagnostic | 0 | 1.040 | 1.035 | 1.048 | 1.040 / 1.040 / 1.041 | 22 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.093 | 1.065 | 1.106 | 1.097 / 1.096 / 1.087 | 146 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.990 | 0.969 | 1.007 | 0.990 / 0.991 / 0.986 | 14,505 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 0.996 | 0.973 | 1.022 | 0.996 / 1.006 / 0.994 | 12,349 |
| `table-plain-16000` | table-diagnostic | 16044 | 1.000 | 0.990 | 1.019 | 0.998 / 1.001 / 1.003 | 3,007 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 1.001 | 0.985 | 1.010 | 1.000 / 1.001 / 1.001 | 3,511 |
| `table-formatted-4096` | table-diagnostic | 4126 | 1.001 | 0.991 | 1.007 | 1.000 / 1.000 / 1.001 | 21,554 |
| `table-dense-16000` | table-diagnostic | 16044 | 1.001 | 0.991 | 1.011 | 1.002 / 1.004 / 0.998 | 119,218 |
| `table-sparse-16000` | table-diagnostic | 16044 | 1.002 | 0.993 | 1.022 | 1.002 / 1.004 / 1.002 | 3,343 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 1.003 | 0.982 | 1.040 | 1.005 / 1.003 / 0.997 | 214 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 1.003 | 0.990 | 1.011 | 1.003 / 1.003 / 1.002 | 5,684 |
| `table-dense-4096` | table-diagnostic | 4140 | 1.003 | 0.989 | 1.015 | 1.004 / 1.000 / 1.004 | 30,872 |
| `table-formatted-16000` | table-diagnostic | 16034 | 1.003 | 0.981 | 1.015 | 1.003 / 1.001 / 1.007 | 84,107 |
| `table-sparse-4096` | table-diagnostic | 4137 | 1.003 | 0.991 | 1.018 | 1.002 / 1.004 / 1.006 | 1,157 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 1.004 | 0.998 | 1.011 | 1.001 / 1.004 / 1.005 | 10,254 |
| `table-dense-256` | table-diagnostic | 300 | 1.004 | 0.997 | 1.012 | 1.004 / 1.005 / 1.003 | 2,246 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.005 | 0.999 | 1.011 | 1.005 / 1.004 / 1.006 | 208 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 1.005 | 1.002 | 1.017 | 1.002 / 1.007 / 1.005 | 211 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.005 | 0.999 | 1.025 | 1.005 / 1.005 / 1.008 | 211 |
| `table-plain-4096` | table-diagnostic | 4139 | 1.005 | 0.992 | 1.021 | 1.000 / 1.006 / 1.006 | 1,047 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.005 | 0.999 | 1.021 | 1.011 / 1.005 / 1.005 | 5,646 |
| `table-formatted-256` | table-diagnostic | 278 | 1.007 | 0.995 | 1.020 | 1.007 / 1.014 / 1.004 | 1,607 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.008 | 0.984 | 1.032 | 1.002 / 1.011 / 1.008 | 212 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.009 | 0.994 | 1.028 | 1.010 / 1.009 / 1.002 | 210 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.010 | 0.981 | 1.023 | 1.010 / 1.009 / 1.014 | 8,958 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.012 | 1.000 | 1.022 | 1.011 / 1.012 / 1.012 | 125 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 1.013 | 1.000 | 1.034 | 1.012 / 1.013 / 1.023 | 174 |
| `table-plain-256` | table-diagnostic | 299 | 1.013 | 0.999 | 1.018 | 1.016 / 1.013 / 1.012 | 404 |
| `autolink-closers-512` | autolink-diagnostic | 545 | 1.014 | 1.001 | 1.023 | 1.014 / 1.014 / 1.014 | 85 |
| `scan-short-link` | link-scan-diagnostic | 7 | 1.015 | 1.003 | 1.024 | 1.008 / 1.015 / 1.016 | 119 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 1.016 | 1.004 | 1.027 | 1.010 / 1.014 / 1.017 | 396 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 1.018 | 1.007 | 1.031 | 1.019 / 1.015 / 1.017 | 81 |
| `table-sparse-256` | table-diagnostic | 297 | 1.020 | 0.987 | 1.037 | 1.021 / 1.017 / 1.014 | 436 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.021 | 1.014 | 1.033 | 1.018 / 1.021 / 1.023 | 81 |
| `scan-empty` | link-scan-diagnostic | 0 | 1.022 | 1.006 | 1.038 | 1.022 / 1.020 / 1.022 | 18 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 1.023 | 1.005 | 1.039 | 1.023 / 1.020 / 1.020 | 85 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 1.023 | 1.002 | 1.041 | 1.027 / 1.024 / 1.022 | 30,150 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.023 | 0.998 | 1.038 | 1.022 / 1.022 / 1.023 | 84 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.024 | 1.013 | 1.048 | 1.027 / 1.022 / 1.024 | 84 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.031 | 1.016 | 1.061 | 1.031 / 1.029 / 1.032 | 48 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 1.031 | 1.017 | 1.054 | 1.024 / 1.031 / 1.042 | 51 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 1.035 | 1.019 | 1.041 | 1.035 / 1.035 / 1.036 | 44 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 1.035 | 1.022 | 1.051 | 1.035 / 1.034 / 1.032 | 43 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.037 | 1.028 | 1.047 | 1.037 / 1.033 / 1.045 | 9,352 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 1.040 | 1.023 | 1.088 | 1.043 / 1.036 / 1.043 | 3,165 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 1.042 | 1.025 | 1.051 | 1.040 / 1.046 / 1.037 | 47 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 1.043 | 1.029 | 1.062 | 1.038 / 1.041 / 1.047 | 46 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `autolink-closers-512` | autolink-diagnostic | 545 | 0.902 | 0.888 | 0.920 | 0.895 / 0.903 / 0.902 | 1,661 |
| `scan-dense-escapes` | link-scan-diagnostic | 1548 | 0.908 | 0.897 | 0.917 | 0.906 / 0.907 / 0.911 | 931 |
| `autolink-closers-2048` | autolink-diagnostic | 2081 | 0.909 | 0.896 | 0.914 | 0.909 / 0.907 / 0.908 | 6,392 |
| `autolink-unicode-64` | autolink-diagnostic | 56 | 0.918 | 0.903 | 0.935 | 0.915 / 0.915 / 0.931 | 156 |
| `autolink-balanced-64` | autolink-diagnostic | 33 | 0.934 | 0.924 | 1.159 | 0.935 / 0.934 / 0.933 | 55 |
| `autolink-mixed-64` | autolink-diagnostic | 35 | 0.944 | 0.930 | 0.951 | 0.941 / 0.946 / 0.942 | 62 |
| `table-dense-256` | table-diagnostic | 300 | 0.961 | 0.926 | 0.992 | 0.962 / 0.956 / 0.961 | 62 |
| `scan-dense-entities` | link-scan-diagnostic | 4108 | 0.963 | 0.957 | 0.976 | 0.963 / 0.963 / 0.963 | 1,915 |
| `table-plain-256` | table-diagnostic | 299 | 0.965 | 0.956 | 0.992 | 0.962 / 0.966 / 0.965 | 68 |
| `autolink-closers-64` | autolink-diagnostic | 98 | 0.972 | 0.960 | 0.983 | 0.967 / 0.975 / 0.974 | 311 |
| `table-sparse-256` | table-diagnostic | 297 | 0.975 | 0.966 | 0.983 | 0.974 / 0.978 / 0.978 | 68 |
| `table-formatted-256` | table-diagnostic | 278 | 0.976 | 0.965 | 0.996 | 0.976 / 0.975 / 0.984 | 338 |
| `table-formatted-4096` | table-diagnostic | 4126 | 0.981 | 0.962 | 0.992 | 0.981 / 0.979 / 0.985 | 4,978 |
| `table-formatted-16000` | table-diagnostic | 16034 | 0.981 | 0.970 | 0.990 | 0.983 / 0.979 / 0.982 | 19,428 |
| `autolink-clean-64` | autolink-diagnostic | 60 | 0.984 | 0.968 | 0.999 | 0.974 / 0.988 / 0.984 | 96 |
| `autolink-long-clean-512` | autolink-diagnostic | 543 | 0.985 | 0.951 | 1.920 | 0.985 / 0.984 / 0.991 | 178 |
| `scan-short-entity` | link-scan-diagnostic | 23 | 0.986 | 0.978 | 0.992 | 0.986 / 0.986 / 0.983 | 38 |
| `scan-many-short-links` | link-scan-diagnostic | 2211 | 0.987 | 0.978 | 1.007 | 0.984 / 0.989 / 0.987 | 3,734 |
| `scan-extension-links` | link-scan-diagnostic | 322 | 0.990 | 0.886 | 1.009 | 0.990 / 0.990 / 0.991 | 475 |
| `scan-short-link` | link-scan-diagnostic | 7 | 0.993 | 0.980 | 1.001 | 0.997 / 0.995 / 0.990 | 29 |
| `table-sparse-16000` | table-diagnostic | 16044 | 0.993 | 0.967 | 1.028 | 0.993 / 1.002 / 0.981 | 808 |
| `table-plain-4096` | table-diagnostic | 4139 | 0.993 | 0.982 | 1.012 | 0.994 / 0.991 / 0.992 | 257 |
| `table-dense-16000` | table-diagnostic | 16044 | 0.994 | 0.985 | 1.004 | 0.994 / 0.992 / 0.996 | 435 |
| `table-plain-16000` | table-diagnostic | 16044 | 0.996 | 0.969 | 1.017 | 0.993 / 0.998 / 0.988 | 801 |
| `scan-long-references` | link-scan-diagnostic | 11862 | 0.997 | 0.984 | 1.004 | 0.997 / 0.998 / 0.994 | 2,427 |
| `table-sparse-4096` | table-diagnostic | 4137 | 0.997 | 0.984 | 1.011 | 0.998 / 0.996 / 0.997 | 258 |
| `table-dense-4096` | table-diagnostic | 4140 | 0.998 | 0.988 | 1.012 | 1.000 / 1.005 / 0.993 | 163 |
| `autolink-mixed-512` | autolink-diagnostic | 490 | 0.998 | 0.986 | 1.007 | 1.002 / 0.997 / 1.000 | 702 |
| `scan-long-clean-links` | link-scan-diagnostic | 12160 | 0.998 | 0.979 | 1.023 | 1.003 / 0.989 / 0.985 | 2,233 |
| `autolink-long-clean-2048` | autolink-diagnostic | 2079 | 0.999 | 0.974 | 1.007 | 0.999 / 0.996 / 0.998 | 525 |
| `scan-short-reference` | link-scan-diagnostic | 20 | 0.999 | 0.992 | 1.010 | 1.003 / 0.998 / 1.000 | 40 |
| `scan-short-title` | link-scan-diagnostic | 11 | 1.001 | 0.990 | 1.047 | 1.038 / 1.001 / 0.997 | 33 |
| `scan-unicode-links` | link-scan-diagnostic | 11728 | 1.002 | 0.994 | 1.019 | 1.000 / 1.002 / 1.005 | 11,791 |
| `scan-escaped-links` | link-scan-diagnostic | 3962 | 1.002 | 0.945 | 1.012 | 1.004 / 1.003 / 0.983 | 2,208 |
| `autolink-mixed-2048` | autolink-diagnostic | 2030 | 1.003 | 0.998 | 1.014 | 1.003 / 1.005 / 1.005 | 2,810 |
| `autolink-unicode-512` | autolink-diagnostic | 504 | 1.009 | 0.994 | 1.061 | 1.006 / 1.038 / 1.009 | 1,243 |
| `scan-malformed-entities` | link-scan-diagnostic | 2832 | 1.010 | 0.997 | 1.036 | 1.016 / 1.009 / 1.015 | 2,395 |
| `scan-mdx-links` | link-scan-diagnostic | 3554 | 1.010 | 0.999 | 1.032 | 1.007 / 1.008 / 1.020 | 1,990 |
| `autolink-unicode-2048` | autolink-diagnostic | 2016 | 1.022 | 1.011 | 1.035 | 1.021 / 1.023 / 1.021 | 4,914 |
| `autolink-clean-512` | autolink-diagnostic | 510 | 1.023 | 1.019 | 1.062 | 1.022 / 1.023 / 1.025 | 732 |
| `autolink-clean-2048` | autolink-diagnostic | 2040 | 1.028 | 1.020 | 1.054 | 1.028 / 1.024 / 1.030 | 2,913 |
| `autolink-balanced-512` | autolink-diagnostic | 495 | 1.029 | 1.017 | 1.075 | 1.030 / 1.029 / 1.025 | 638 |
| `autolink-balanced-2048` | autolink-diagnostic | 2046 | 1.032 | 1.017 | 1.048 | 1.033 / 1.031 / 1.032 | 2,567 |
| `scan-empty` | link-scan-diagnostic | 0 | 1.054 | 1.031 | 1.064 | 1.055 / 1.040 / 1.055 | 7 |
| `autolink-long-clean-64` | autolink-diagnostic | 95 | 1.058 | 1.039 | 1.063 | 1.046 / 1.061 / 1.061 | 94 |

## PGO, trained on 29 broad + 45 diagnostics, measured on the 28 held-out broad documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-question` | comments | 160 | 1.069 | 1.021 | 1.121 | 1.080 / 1.069 / 1.027 | 313 |
| `comment-quote` | comments | 290 | 1.104 | 1.094 | 1.113 | 1.104 / 1.103 / 1.104 | 455 |
| `comment-inline-code` | comments | 285 | 1.124 | 1.091 | 1.132 | 1.125 / 1.124 / 1.124 | 442 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.133 | 1.118 | 1.142 | 1.139 / 1.128 / 1.129 | 5,596 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.146 | 1.139 | 1.151 | 1.146 / 1.146 / 1.143 | 7,727 |
| `comment-links` | comments | 278 | 1.157 | 1.144 | 1.175 | 1.157 / 1.157 / 1.156 | 608 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.174 | 1.153 | 1.203 | 1.172 / 1.176 / 1.159 | 5,556 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.176 | 1.122 | 1.189 | 1.172 / 1.179 / 1.170 | 830 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.179 | 1.162 | 1.189 | 1.179 / 1.182 / 1.171 | 5,971 |
| `vue-docs-slots` | technical-docs | 24211 | 1.179 | 1.155 | 1.189 | 1.184 / 1.176 / 1.184 | 24,065 |
| `comment-incident` | comments | 1124 | 1.188 | 1.154 | 1.196 | 1.186 / 1.192 / 1.188 | 2,037 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.189 | 1.180 | 1.194 | 1.189 / 1.191 / 1.185 | 79,894 |
| `legacy-docs-readme` | readme | 1825 | 1.193 | 1.188 | 1.208 | 1.201 / 1.193 / 1.192 | 4,697 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.195 | 1.183 | 1.201 | 1.199 / 1.194 / 1.193 | 2,726 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.202 | 1.193 | 1.218 | 1.203 / 1.206 / 1.200 | 3,517 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.210 | 1.192 | 1.222 | 1.203 / 1.213 / 1.212 | 1,162 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.215 | 1.207 | 1.241 | 1.224 / 1.217 / 1.210 | 4,172 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.228 | 1.224 | 1.246 | 1.227 / 1.226 / 1.231 | 1,533 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.228 | 1.207 | 1.245 | 1.227 / 1.224 / 1.237 | 2,388 |
| `vite-docs-api-plugin` | reference | 31890 | 1.231 | 1.146 | 1.334 | 1.234 / 1.241 / 1.191 | 67,560 |
| `legacy-contributing` | technical-docs | 9323 | 1.236 | 1.191 | 1.272 | 1.239 / 1.235 / 1.236 | 11,273 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.239 | 1.233 | 1.261 | 1.239 / 1.236 / 1.250 | 1,830 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.244 | 1.236 | 1.262 | 1.243 / 1.245 / 1.247 | 17,078 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.260 | 1.094 | 1.402 | 1.315 / 1.293 / 1.202 | 113,835 |
| `comment-table` | comments | 310 | 1.261 | 1.255 | 1.268 | 1.261 / 1.260 / 1.264 | 1,275 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.268 | 1.226 | 1.325 | 1.263 / 1.257 / 1.271 | 32,748 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.363 | 1.134 | 1.482 | 1.403 / 1.381 / 1.284 | 63,316 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.383 | 1.265 | 1.501 | 1.366 / 1.395 / 1.426 | 55,203 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.148 | 1.136 | 1.163 | 1.148 / 1.146 / 1.158 | 5,089 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.162 | 1.145 | 1.176 | 1.171 / 1.155 / 1.162 | 7,204 |
| `vue-docs-slots` | technical-docs | 24211 | 1.186 | 1.166 | 1.207 | 1.191 / 1.180 / 1.186 | 23,686 |
| `comment-question` | comments | 160 | 1.188 | 1.170 | 1.193 | 1.188 / 1.184 / 1.186 | 87 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.188 | 1.151 | 1.199 | 1.181 / 1.188 / 1.189 | 5,105 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.192 | 1.186 | 1.212 | 1.190 / 1.193 / 1.192 | 79,554 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.192 | 1.169 | 1.215 | 1.194 / 1.192 / 1.185 | 5,583 |
| `comment-quote` | comments | 290 | 1.201 | 1.178 | 1.224 | 1.200 / 1.202 / 1.212 | 215 |
| `legacy-docs-readme` | readme | 1825 | 1.202 | 1.197 | 1.215 | 1.202 / 1.200 / 1.206 | 4,351 |
| `comment-inline-code` | comments | 285 | 1.214 | 1.207 | 1.225 | 1.215 / 1.215 / 1.210 | 210 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.219 | 1.204 | 1.228 | 1.217 / 1.221 / 1.218 | 3,177 |
| `comment-links` | comments | 278 | 1.220 | 1.217 | 1.236 | 1.220 / 1.219 / 1.224 | 372 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.225 | 1.216 | 1.238 | 1.225 / 1.226 / 1.225 | 2,381 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.229 | 1.219 | 1.248 | 1.227 / 1.229 / 1.229 | 3,812 |
| `comment-incident` | comments | 1124 | 1.233 | 1.227 | 1.248 | 1.236 / 1.233 / 1.230 | 1,713 |
| `legacy-contributing` | technical-docs | 9323 | 1.238 | 1.224 | 1.256 | 1.247 / 1.240 / 1.236 | 10,581 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.239 | 1.077 | 1.380 | 1.299 / 1.201 / 1.260 | 115,148 |
| `vite-docs-api-plugin` | reference | 31890 | 1.240 | 1.119 | 1.333 | 1.224 / 1.240 / 1.250 | 66,729 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.252 | 1.213 | 1.292 | 1.250 / 1.248 / 1.268 | 16,696 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.263 | 1.244 | 1.274 | 1.265 / 1.264 / 1.260 | 2,048 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.274 | 1.257 | 1.333 | 1.274 / 1.284 / 1.266 | 32,156 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.279 | 1.269 | 1.296 | 1.285 / 1.278 / 1.274 | 515 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.290 | 1.283 | 1.296 | 1.285 / 1.291 / 1.290 | 852 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.290 | 1.266 | 1.300 | 1.291 / 1.286 / 1.290 | 1,228 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.298 | 1.285 | 1.305 | 1.298 / 1.293 / 1.304 | 1,527 |
| `comment-table` | comments | 310 | 1.313 | 1.300 | 1.326 | 1.317 / 1.313 / 1.308 | 1,031 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.384 | 1.185 | 1.473 | 1.362 / 1.401 / 1.384 | 53,316 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.387 | 1.180 | 1.464 | 1.359 / 1.391 / 1.387 | 62,079 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.033 | 1.028 | 1.039 | 1.033 / 1.030 / 1.035 | 22,718 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.157 | 1.137 | 1.179 | 1.157 / 1.154 / 1.157 | 4,488 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.180 | 1.145 | 1.194 | 1.184 / 1.178 / 1.154 | 3,859 |
| `comment-links` | comments | 278 | 1.199 | 1.188 | 1.212 | 1.198 / 1.201 / 1.197 | 280 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.202 | 1.163 | 1.206 | 1.198 / 1.202 / 1.200 | 3,516 |
| `legacy-docs-readme` | readme | 1825 | 1.207 | 1.194 | 1.223 | 1.211 / 1.204 / 1.208 | 3,325 |
| `comment-question` | comments | 160 | 1.209 | 1.187 | 1.212 | 1.209 / 1.208 / 1.209 | 64 |
| `vite-docs-api-plugin` | reference | 31890 | 1.214 | 1.195 | 1.263 | 1.219 / 1.214 / 1.224 | 49,068 |
| `comment-inline-code` | comments | 285 | 1.216 | 1.202 | 1.235 | 1.216 / 1.215 / 1.227 | 131 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.233 | 1.205 | 1.251 | 1.237 / 1.235 / 1.229 | 2,551 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.238 | 1.139 | 1.467 | 1.238 / 1.156 / 1.353 | 64,853 |
| `comment-quote` | comments | 290 | 1.238 | 1.224 | 1.245 | 1.235 / 1.238 / 1.240 | 168 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.241 | 1.229 | 1.252 | 1.240 / 1.245 / 1.241 | 3,648 |
| `legacy-contributing` | technical-docs | 9323 | 1.253 | 1.239 | 1.272 | 1.248 / 1.248 / 1.266 | 7,730 |
| `comment-incident` | comments | 1124 | 1.257 | 1.253 | 1.264 | 1.257 / 1.259 / 1.256 | 1,411 |
| `vue-docs-slots` | technical-docs | 24211 | 1.266 | 1.256 | 1.276 | 1.269 / 1.268 / 1.258 | 13,453 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.273 | 1.263 | 1.279 | 1.274 / 1.270 / 1.274 | 357 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.282 | 1.267 | 1.298 | 1.279 / 1.286 / 1.283 | 1,996 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.284 | 1.275 | 1.296 | 1.281 / 1.286 / 1.284 | 14,401 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.294 | 1.278 | 1.327 | 1.307 / 1.290 / 1.294 | 1,402 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.301 | 1.286 | 1.327 | 1.299 / 1.317 / 1.301 | 19,703 |
| `comment-table` | comments | 310 | 1.317 | 1.306 | 1.352 | 1.322 / 1.313 / 1.317 | 852 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.322 | 1.309 | 1.358 | 1.325 / 1.322 / 1.318 | 1,286 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.332 | 1.312 | 1.344 | 1.333 / 1.332 / 1.331 | 532 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.338 | 1.311 | 1.343 | 1.323 / 1.338 / 1.341 | 984 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.338 | 1.334 | 1.359 | 1.336 / 1.338 / 1.341 | 775 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.377 | 1.321 | 1.474 | 1.378 / 1.360 / 1.382 | 31,914 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.399 | 1.361 | 1.463 | 1.399 / 1.389 / 1.399 | 35,803 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.076 | 1.059 | 1.085 | 1.066 / 1.077 / 1.080 | 1,197 |
| `vue-docs-slots` | technical-docs | 24211 | 1.078 | 1.054 | 1.083 | 1.075 / 1.080 / 1.077 | 9,800 |
| `comment-incident` | comments | 1124 | 1.106 | 1.098 | 1.127 | 1.109 / 1.103 / 1.106 | 295 |
| `comment-quote` | comments | 290 | 1.106 | 1.096 | 1.110 | 1.105 / 1.106 / 1.106 | 48 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.115 | 1.107 | 1.128 | 1.116 / 1.121 / 1.113 | 1,852 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 1.116 | 1.093 | 1.129 | 1.104 / 1.125 / 1.116 | 2,259 |
| `comment-question` | comments | 160 | 1.123 | 1.108 | 1.138 | 1.125 / 1.123 / 1.123 | 24 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.127 | 1.120 | 1.133 | 1.128 / 1.127 / 1.127 | 1,153 |
| `vite-docs-api-plugin` | reference | 31890 | 1.130 | 1.122 | 1.138 | 1.132 / 1.127 / 1.127 | 12,427 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.138 | 1.102 | 1.152 | 1.128 / 1.146 / 1.140 | 2,606 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.142 | 1.135 | 1.151 | 1.142 / 1.142 / 1.143 | 951 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.160 | 1.148 | 1.176 | 1.160 / 1.162 / 1.159 | 1,584 |
| `legacy-contributing` | technical-docs | 9323 | 1.162 | 1.130 | 1.189 | 1.161 / 1.170 / 1.157 | 2,809 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.164 | 1.128 | 1.193 | 1.176 / 1.172 / 1.148 | 11,026 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 1.180 | 1.165 | 1.203 | 1.180 / 1.179 / 1.180 | 770 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 1.189 | 1.180 | 1.198 | 1.189 / 1.186 / 1.191 | 1,229 |
| `comment-inline-code` | comments | 285 | 1.191 | 1.159 | 1.215 | 1.192 / 1.191 / 1.187 | 79 |
| `comment-links` | comments | 278 | 1.193 | 1.186 | 1.206 | 1.193 / 1.193 / 1.205 | 85 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 1.194 | 1.164 | 1.207 | 1.197 / 1.185 / 1.193 | 149 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 1.201 | 1.183 | 1.213 | 1.201 / 1.199 / 1.201 | 308 |
| `legacy-docs-readme` | readme | 1825 | 1.205 | 1.180 | 1.220 | 1.205 / 1.206 / 1.206 | 1,001 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 1.211 | 1.195 | 1.228 | 1.211 / 1.216 / 1.195 | 434 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 1.241 | 1.224 | 1.263 | 1.239 / 1.245 / 1.236 | 531 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.262 | 1.230 | 1.275 | 1.255 / 1.262 / 1.262 | 17,668 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.274 | 1.267 | 1.281 | 1.277 / 1.275 / 1.273 | 56,275 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.286 | 1.254 | 1.358 | 1.335 / 1.255 / 1.264 | 21,065 |
| `comment-table` | comments | 310 | 1.300 | 1.291 | 1.316 | 1.300 / 1.298 / 1.302 | 176 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.306 | 1.251 | 1.415 | 1.289 / 1.317 / 1.318 | 33,626 |

