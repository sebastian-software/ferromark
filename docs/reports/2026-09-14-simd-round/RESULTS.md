# Derived measurements

Ratios are baseline time / candidate time; values above 1 favor the candidate.
The aggregate is the geometric mean of each document's median paired ratio.
Broad groups exclude synthetic diagnostics. Wikipedia views overlap.

| Group | Cases | Fresh | Reuse | Parse | Render control |
| --- | ---: | ---: | ---: | ---: | ---: |
| All broad cases | 57 | 1.034 | 1.034 | 1.049 | 1.000 |
| comments | 12 | 1.017 | 1.007 | 1.012 | 1.002 |
| encyclopedia | 12 | 1.096 | 1.106 | 1.146 | 0.993 |
| plain-prose | 4 | 1.001 | 1.002 | 1.005 | 1.006 |
| readme | 2 | 1.021 | 1.019 | 1.026 | 1.005 |
| reference | 4 | 1.002 | 1.003 | 1.014 | 0.999 |
| syntax-guard | 1 | 1.039 | 1.052 | 1.061 | 1.016 |
| technical-docs | 22 | 1.023 | 1.024 | 1.036 | 1.001 |
| <512 B | 12 | 1.021 | 1.015 | 1.022 | 1.003 |
| 512 B–2 KiB | 9 | 1.052 | 1.055 | 1.073 | 1.007 |
| 2–10 KiB | 19 | 1.038 | 1.039 | 1.051 | 1.004 |
| 10–50 KiB | 12 | 1.021 | 1.024 | 1.045 | 0.997 |
| 50–256 KiB | 5 | 1.048 | 1.051 | 1.078 | 0.972 |

## Individual cases

Absolute nanosecond measurements and all paired/round ranges are in `summary.csv`.

| Case | Bytes | Fresh | Reuse | Parse | Render control | Reuse round range |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| comment-ack | 37 | 1.008 | 1.002 | 1.007 | 0.998 | 0.997–1.007 |
| comment-question | 160 | 1.034 | 1.004 | 1.015 | 1.001 | 1.000–1.008 |
| comment-review | 282 | 1.013 | 1.004 | 1.009 | 1.001 | 0.999–1.006 |
| comment-links | 278 | 1.028 | 1.033 | 1.038 | 1.004 | 1.025–1.047 |
| comment-checklist | 287 | 1.016 | 1.012 | 1.010 | 1.001 | 1.009–1.015 |
| comment-quote | 290 | 1.015 | 1.001 | 1.010 | 1.002 | 1.000–1.007 |
| comment-unicode | 327 | 1.018 | 1.001 | 1.007 | 1.014 | 1.000–1.004 |
| comment-inline-code | 285 | 1.021 | 0.988 | 1.003 | 1.002 | 0.987–0.993 |
| comment-reproduction | 298 | 1.014 | 1.011 | 1.011 | 1.009 | 1.007–1.018 |
| comment-table | 310 | 1.002 | 1.003 | 1.006 | 0.977 | 1.000–1.005 |
| comment-review-long | 957 | 1.029 | 1.017 | 1.021 | 0.999 | 1.014–1.022 |
| comment-incident | 1124 | 1.005 | 1.007 | 1.001 | 1.012 | 1.005–1.007 |
| guard-angle-link | 41 | 1.039 | 1.052 | 1.061 | 1.016 | 1.050–1.055 |
| legacy-contributing | 9323 | 1.014 | 1.017 | 1.021 | 1.008 | 1.010–1.017 |
| legacy-node-ferromark-readme | 9075 | 1.026 | 1.025 | 1.040 | 1.006 | 1.020–1.029 |
| legacy-docs-migration-0-2 | 1985 | 1.008 | 1.003 | 1.012 | 1.005 | 1.000–1.008 |
| legacy-docs-migration-0-3 | 2379 | 1.009 | 1.019 | 1.022 | 1.003 | 1.014–1.020 |
| legacy-docs-migration-0-4 | 7045 | 1.008 | 1.001 | 1.002 | 1.004 | 1.000–1.009 |
| legacy-docs-migration-0-8 | 2374 | 1.013 | 1.009 | 1.010 | 1.009 | 1.004–1.012 |
| legacy-docs-releasing | 4141 | 1.001 | 0.997 | 1.001 | 0.996 | 0.994–1.004 |
| legacy-docs-readme-theme | 2848 | 1.023 | 1.018 | 1.028 | 1.010 | 1.015–1.018 |
| legacy-docs-markdown-extensions | 3707 | 1.024 | 1.020 | 1.020 | 1.021 | 1.014–1.027 |
| legacy-docs-mdx | 7422 | 1.008 | 1.009 | 1.014 | 1.001 | 1.002–1.013 |
| legacy-docs-readme | 1825 | 1.015 | 1.013 | 1.012 | 1.004 | 1.009–1.016 |
| legacy-docs-adr-readme-theme-composition | 1532 | 1.004 | 1.007 | 1.004 | 1.002 | 1.004–1.018 |
| rust-book-ch03-04-comments | 393 | 1.047 | 1.066 | 1.085 | 1.010 | 1.057–1.076 |
| rust-book-appendix-02-operators | 22595 | 0.989 | 0.987 | 0.990 | 1.009 | 0.980–0.994 |
| rust-book-ch00-00-introduction | 10839 | 1.005 | 1.007 | 1.013 | 1.004 | 1.002–1.013 |
| rust-book-ch17-00-async-await | 9734 | 0.999 | 1.003 | 0.998 | 0.997 | 0.997–1.006 |
| vue-docs-ways-of-using-vue | 5883 | 1.037 | 1.032 | 1.052 | 1.009 | 1.031–1.039 |
| vue-docs-suspense | 8291 | 1.011 | 1.009 | 1.023 | 1.000 | 1.007–1.020 |
| vue-docs-slots | 24211 | 1.116 | 1.113 | 1.161 | 1.005 | 1.113–1.119 |
| vue-docs-reactivity-in-depth | 24001 | 1.062 | 1.067 | 1.093 | 0.999 | 1.036–1.076 |
| vite-docs-philosophy | 3575 | 1.038 | 1.045 | 1.066 | 0.998 | 1.042–1.047 |
| vite-docs-performance | 8184 | 1.030 | 1.028 | 1.037 | 0.996 | 1.028–1.036 |
| vite-docs-api-plugin | 31890 | 1.020 | 1.019 | 1.051 | 1.000 | 1.014–1.029 |
| vite-docs-features | 39739 | 0.998 | 0.990 | 1.051 | 0.991 | 0.984–1.027 |
| typescript-handbook-the-handbook | 5337 | 1.045 | 1.033 | 1.040 | 1.000 | 1.026–1.035 |
| typescript-handbook-advanced-types | 36745 | 0.992 | 0.992 | 1.014 | 0.989 | 0.976–0.995 |
| typescript-handbook-compiler-options | 54026 | 1.007 | 1.014 | 1.003 | 1.000 | 1.002–1.014 |
| typescript-handbook-typescript-5-0 | 50714 | 1.013 | 1.031 | 1.047 | 0.955 | 0.999–1.034 |
| wiki-rainbow-first-paragraph | 859 | 1.106 | 1.114 | 1.165 | 1.012 | 1.051–1.124 |
| wiki-rainbow-lead | 1859 | 1.093 | 1.104 | 1.140 | 1.008 | 1.096–1.118 |
| wiki-rainbow-article-body | 48422 | 1.071 | 1.080 | 1.119 | 0.987 | 1.075–1.089 |
| wiki-rainbow-plain-prose | 38800 | 0.996 | 0.998 | 1.006 | 0.996 | 0.997–1.002 |
| wiki-tea-first-paragraph | 1126 | 1.106 | 1.116 | 1.156 | 1.008 | 1.116–1.125 |
| wiki-tea-lead | 6363 | 1.114 | 1.116 | 1.143 | 1.008 | 1.114–1.116 |
| wiki-tea-article-body | 58814 | 1.069 | 1.075 | 1.132 | 0.986 | 1.063–1.078 |
| wiki-tea-plain-prose | 40577 | 1.002 | 0.998 | 1.003 | 1.023 | 0.824–1.006 |
| wiki-chess-first-paragraph | 1190 | 1.110 | 1.122 | 1.167 | 1.009 | 1.119–1.126 |
| wiki-chess-lead | 4125 | 1.103 | 1.117 | 1.152 | 1.013 | 1.115–1.121 |
| wiki-chess-article-body | 113609 | 1.076 | 1.081 | 1.123 | 0.914 | 1.074–1.081 |
| wiki-chess-plain-prose | 80966 | 1.005 | 1.004 | 1.009 | 0.996 | 1.004–1.014 |
| wiki-volcano-first-paragraph | 2644 | 1.113 | 1.134 | 1.167 | 1.006 | 1.129–1.138 |
| wiki-volcano-lead | 4232 | 1.114 | 1.125 | 1.161 | 1.002 | 1.124–1.134 |
| wiki-volcano-article-body | 69241 | 1.084 | 1.085 | 1.132 | 0.966 | 1.078–1.090 |
| wiki-volcano-plain-prose | 47303 | 1.001 | 1.007 | 1.003 | 1.010 | 1.001–1.010 |
| scan-empty | 0 | 1.017 | 1.001 | 1.001 | 0.999 | 1.000–1.002 |
| scan-short-link | 7 | 1.019 | 0.997 | 1.003 | 1.007 | 0.996–0.998 |
| scan-short-title | 11 | 1.031 | 1.023 | 1.008 | 1.008 | 1.022–1.024 |
| scan-short-entity | 23 | 1.008 | 0.997 | 1.005 | 1.004 | 0.993–0.999 |
| scan-short-reference | 20 | 1.020 | 1.023 | 1.018 | 1.005 | 1.018–1.026 |
| scan-long-clean-links | 12160 | 1.245 | 1.258 | 1.284 | 1.005 | 1.253–1.259 |
| scan-many-short-links | 2211 | 1.041 | 1.040 | 1.037 | 1.036 | 1.035–1.041 |
| scan-escaped-links | 3962 | 0.995 | 0.993 | 0.995 | 0.996 | 0.985–0.999 |
| scan-unicode-links | 11728 | 1.220 | 1.226 | 1.245 | 1.002 | 1.215–1.238 |
| scan-long-references | 11862 | 1.164 | 1.164 | 1.171 | 1.007 | 1.163–1.175 |
| scan-dense-escapes | 1548 | 1.004 | 1.004 | 0.997 | 0.998 | 1.002–1.009 |
| scan-dense-entities | 4108 | 0.990 | 0.984 | 0.991 | 0.996 | 0.977–0.990 |
| scan-malformed-entities | 2832 | 1.009 | 0.998 | 0.998 | 0.998 | 0.998–1.000 |
| scan-mdx-links | 3554 | 1.066 | 1.066 | 1.076 | 1.001 | 1.066–1.076 |
| scan-extension-links | 322 | 1.018 | 1.014 | 1.016 | 1.021 | 1.014–1.017 |
