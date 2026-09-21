# Numbers

Ratios are baseline time over candidate time (median of the paired windows); **higher is faster, below 1.000 the candidate is slower**.
The baseline is `23a59bdf` for every candidate; each candidate is one branch on its own, so the rows are independent, not cumulative.

## Screens: 20 documents and 16 diagnostics, 3 rounds × 3 pairs

| Candidate | fresh | reuse | parse | render | fresh rounds | parse rounds |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| A/A control (baseline against itself) | 1.0035 | 1.0025 | 1.0035 | 0.9983 | 1.006 / 1.001 / 1.004 | 1.003 / 1.003 / 1.005 |
| `norm` source normalization, cold path outlined | 1.0002 | 0.9978 | 0.9997 | 0.9917 | 0.995 / 1.001 / 0.999 | 1.000 / 0.999 / 0.999 |
| `depth` inline depth as a plain cell | 1.0056 | 1.0056 | 1.0086 | 0.9990 | 1.006 / 1.004 / 1.007 | 1.010 / 1.009 / 1.008 |
| `lines` container line facts once per line | 1.0078 | 1.0054 | 1.0090 | 0.9985 | 1.011 / 1.008 / 1.008 | 1.009 / 1.009 / 1.009 |
| `subctx` borrowed sub-parser context | 0.9988 | 0.9970 | 0.9968 | 0.9956 | 0.999 / 0.997 / 1.001 | 0.997 / 1.000 / 0.997 |
| `arena` 2 KB reservation floor | 1.0054 | 1.0040 | 1.0054 | 0.9998 | 1.004 / 1.008 / 1.005 | 1.005 / 1.005 / 1.007 |
| `emph` one delimiter buffer per parse | 1.0030 | 0.9977 | 1.0001 | 0.9963 | 1.005 / 1.005 / 1.002 | 0.999 / 0.999 / 0.998 |
| `closer` forward window for has_closer_from | 1.0142 | 1.0131 | 1.0168 | 0.9980 | 1.018 / 1.014 / 1.010 | 1.017 / 1.017 / 1.017 |
| `tables` NEON pipe cursor | 1.0233 | 1.0230 | 1.0222 | 0.9995 | 1.023 / 1.025 / 1.022 | 1.021 / 1.021 / 1.022 |

### Per-document fresh ratios, screen

| Document | aa | norm | depth | lines | subctx | arena | emph | closer | tables |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `comment-ack` | 1.004 | 0.996 | 1.008 | 1.003 | 0.976 | 1.042 | 0.990 | 0.992 | 1.005 |
| `comment-checklist` | 1.004 | 0.998 | 0.998 | 1.024 | 0.996 | 1.017 | 0.987 | 0.979 | 1.006 |
| `comment-incident` | 1.006 | 1.002 | 0.998 | 0.998 | 0.991 | 0.996 | 1.011 | 0.989 | 1.000 |
| `comment-links` | 1.000 | 0.990 | 1.011 | 1.008 | 0.987 | 1.015 | 0.981 | 1.108 | 0.998 |
| `comment-question` | 0.996 | 0.993 | 1.004 | 0.996 | 0.979 | 1.035 | 0.993 | 0.991 | 0.999 |
| `comment-quote` | 1.003 | 0.994 | 0.999 | 0.971 | 0.987 | 1.022 | 0.991 | 0.987 | 1.010 |
| `comment-review-long` | 0.998 | 0.989 | 1.003 | 1.015 | 0.998 | 1.000 | 0.997 | 1.051 | 0.992 |
| `legacy-docs-migration-0-2` | 1.017 | 0.997 | 1.008 | 1.007 | 1.008 | 1.018 | 1.001 | 1.010 | 1.002 |
| `legacy-docs-releasing` | 1.000 | 0.990 | 1.001 | 1.013 | 0.997 | 1.001 | 0.981 | 0.997 | 1.000 |
| `rust-book-appendix-02-operators` | 1.003 | 1.003 | 1.000 | 1.001 | 1.007 | 1.007 | 1.000 | 1.004 | 0.997 |
| `scan-dense-escapes` | 1.000 | 1.000 | 1.003 | 1.003 | 0.972 | 1.000 | 0.999 | 1.017 | 0.999 |
| `scan-extension-links` | 1.001 | 1.001 | 1.000 | 0.999 | 1.009 | 0.997 | 0.993 | 1.040 | 0.982 |
| `scan-long-references` | 0.967 | 0.972 | 0.997 | 1.007 | 0.998 | 1.003 | 0.973 | 1.041 | 1.001 |
| `scan-short-reference` | 0.993 | 1.006 | 1.007 | 1.007 | 0.993 | 1.018 | 1.006 | 1.098 | 1.007 |
| `table-dense-16000` | 1.004 | 1.001 | 0.999 | 0.997 | 1.000 | 1.002 | 1.000 | 1.000 | 1.430 |
| `table-dense-256` | 1.002 | 1.005 | 1.006 | 1.005 | 1.001 | 1.007 | 1.003 | 0.998 | 1.285 |
| `table-dense-4096` | 1.000 | 1.002 | 1.001 | 1.001 | 1.001 | 1.000 | 1.001 | 0.998 | 1.423 |
| `table-formatted-16000` | 0.997 | 0.998 | 1.000 | 0.997 | 1.004 | 1.003 | 1.026 | 1.003 | 0.912 |
| `table-formatted-256` | 1.001 | 1.005 | 1.003 | 1.004 | 1.007 | 0.973 | 1.004 | 0.998 | 0.941 |
| `table-formatted-4096` | 0.996 | 1.002 | 0.987 | 0.999 | 1.002 | 1.001 | 1.020 | 1.001 | 0.915 |
| `table-plain-16000` | 0.998 | 1.010 | 1.002 | 1.005 | 1.010 | 1.005 | 1.014 | 0.999 | 1.053 |
| `table-plain-256` | 1.022 | 1.019 | 1.027 | 1.012 | 1.018 | 1.037 | 1.016 | 0.991 | 0.991 |
| `table-plain-4096` | 1.012 | 1.020 | 1.004 | 1.003 | 1.016 | 1.006 | 1.004 | 0.988 | 1.030 |
| `table-sparse-16000` | 1.001 | 1.008 | 1.008 | 1.006 | 1.000 | 1.013 | 1.003 | 0.998 | 1.028 |
| `table-sparse-256` | 1.019 | 1.017 | 1.024 | 1.022 | 1.017 | 1.033 | 1.016 | 0.992 | 0.976 |
| `table-sparse-4096` | 1.010 | 1.006 | 1.019 | 1.005 | 0.997 | 1.009 | 1.010 | 0.996 | 0.995 |
| `typescript-handbook-advanced-types` | 1.030 | 0.998 | 1.028 | 0.989 | 1.011 | 0.990 | 0.989 | 1.012 | 1.021 |
| `typescript-handbook-the-handbook` | 1.005 | 0.999 | 1.007 | 1.030 | 1.000 | 1.002 | 1.000 | 1.028 | 1.000 |
| `vite-docs-api-plugin` | 1.023 | 1.005 | 1.020 | 1.087 | 1.003 | 0.990 | 1.004 | 1.019 | 1.009 |
| `vite-docs-performance` | 1.003 | 0.991 | 0.998 | 1.013 | 0.996 | 0.999 | 0.992 | 1.031 | 1.001 |
| `vue-docs-ways-of-using-vue` | 1.004 | 0.989 | 1.001 | 1.018 | 0.995 | 0.997 | 1.001 | 1.048 | 0.999 |
| `wiki-chess-plain-prose` | 0.995 | 0.992 | 0.999 | 0.999 | 0.987 | 0.988 | 0.997 | 0.996 | 0.991 |
| `wiki-tea-article-body` | 0.999 | 1.010 | 1.019 | 1.022 | 1.011 | 1.000 | 1.010 | 1.038 | 1.007 |
| `wiki-tea-first-paragraph` | 1.004 | 0.998 | 1.009 | 1.006 | 0.995 | 1.002 | 1.004 | 1.014 | 1.006 |
| `wiki-tea-lead` | 1.001 | 0.990 | 1.010 | 1.001 | 0.992 | 0.992 | 1.008 | 1.010 | 1.000 |
| `wiki-volcano-article-body` | 1.009 | 1.012 | 0.996 | 1.015 | 1.001 | 0.976 | 1.089 | 1.065 | 1.010 |

### Per-document parse ratios, screen

| Document | aa | norm | depth | lines | subctx | arena | emph | closer | tables |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `comment-ack` | 1.002 | 1.004 | 1.016 | 0.995 | 0.961 | 1.004 | 0.994 | 0.971 | 1.000 |
| `comment-checklist` | 1.002 | 1.001 | 1.004 | 1.027 | 0.993 | 0.997 | 0.996 | 0.975 | 1.002 |
| `comment-incident` | 1.005 | 0.993 | 1.004 | 1.007 | 0.991 | 1.001 | 0.996 | 0.982 | 0.999 |
| `comment-links` | 1.009 | 0.993 | 1.009 | 1.016 | 0.996 | 1.016 | 0.994 | 1.172 | 0.999 |
| `comment-question` | 1.002 | 0.999 | 1.016 | 0.996 | 0.965 | 1.001 | 0.991 | 0.980 | 1.000 |
| `comment-quote` | 1.002 | 0.992 | 1.007 | 0.951 | 0.985 | 1.000 | 0.990 | 0.978 | 1.000 |
| `comment-review-long` | 1.001 | 0.989 | 1.006 | 1.011 | 0.985 | 0.998 | 1.005 | 1.070 | 1.008 |
| `legacy-docs-migration-0-2` | 1.013 | 0.996 | 1.006 | 1.005 | 1.006 | 1.011 | 0.997 | 1.010 | 0.996 |
| `legacy-docs-releasing` | 0.995 | 0.990 | 0.996 | 1.012 | 0.989 | 0.994 | 0.980 | 0.995 | 1.001 |
| `rust-book-appendix-02-operators` | 1.002 | 1.007 | 1.007 | 1.005 | 1.001 | 1.011 | 1.000 | 1.006 | 0.998 |
| `scan-dense-escapes` | 1.001 | 1.004 | 1.001 | 1.003 | 0.989 | 1.002 | 1.003 | 1.019 | 0.996 |
| `scan-extension-links` | 0.991 | 1.000 | 1.013 | 1.008 | 1.015 | 1.020 | 1.001 | 1.052 | 1.005 |
| `scan-long-references` | 0.970 | 0.974 | 0.996 | 1.005 | 0.999 | 1.002 | 0.970 | 1.039 | 1.007 |
| `scan-short-reference` | 0.998 | 1.008 | 1.015 | 1.015 | 0.995 | 1.011 | 1.012 | 1.119 | 1.007 |
| `table-dense-16000` | 1.004 | 0.998 | 0.998 | 1.000 | 0.999 | 1.000 | 0.999 | 1.001 | 1.433 |
| `table-dense-256` | 1.002 | 1.004 | 1.006 | 1.004 | 1.005 | 1.005 | 1.003 | 0.998 | 1.308 |
| `table-dense-4096` | 0.999 | 1.001 | 1.001 | 1.000 | 1.001 | 1.000 | 1.000 | 0.999 | 1.429 |
| `table-formatted-16000` | 1.001 | 1.000 | 1.002 | 1.001 | 1.003 | 1.002 | 1.039 | 1.003 | 0.896 |
| `table-formatted-256` | 1.002 | 1.007 | 1.003 | 1.006 | 1.008 | 1.007 | 0.999 | 0.997 | 0.926 |
| `table-formatted-4096` | 1.000 | 1.000 | 1.003 | 1.000 | 1.001 | 1.001 | 1.030 | 1.001 | 0.897 |
| `table-plain-16000` | 1.002 | 1.001 | 1.004 | 1.003 | 1.000 | 1.002 | 0.999 | 0.999 | 1.070 |
| `table-plain-256` | 1.018 | 1.014 | 1.025 | 1.007 | 1.007 | 1.022 | 1.006 | 0.988 | 0.968 |
| `table-plain-4096` | 1.008 | 1.008 | 1.010 | 1.004 | 1.003 | 1.007 | 1.001 | 0.995 | 1.034 |
| `table-sparse-16000` | 1.008 | 1.006 | 1.010 | 1.005 | 1.001 | 1.004 | 1.001 | 0.996 | 1.029 |
| `table-sparse-256` | 1.031 | 1.023 | 1.033 | 1.021 | 1.016 | 1.021 | 1.011 | 0.985 | 0.961 |
| `table-sparse-4096` | 1.006 | 1.008 | 1.011 | 1.008 | 1.005 | 1.010 | 1.004 | 0.991 | 0.995 |
| `typescript-handbook-advanced-types` | 1.006 | 1.012 | 1.012 | 1.016 | 1.002 | 1.021 | 0.983 | 1.011 | 0.995 |
| `typescript-handbook-the-handbook` | 0.996 | 0.990 | 1.010 | 1.037 | 1.001 | 1.003 | 1.001 | 1.031 | 1.010 |
| `vite-docs-api-plugin` | 1.007 | 1.007 | 1.001 | 1.105 | 0.995 | 0.998 | 0.993 | 1.016 | 0.993 |
| `vite-docs-performance` | 1.006 | 0.992 | 1.002 | 1.021 | 0.990 | 1.005 | 0.996 | 1.044 | 1.000 |
| `vue-docs-ways-of-using-vue` | 1.005 | 0.992 | 1.008 | 1.024 | 0.995 | 1.007 | 1.007 | 1.086 | 1.004 |
| `wiki-chess-plain-prose` | 1.003 | 0.993 | 1.003 | 1.001 | 0.983 | 0.993 | 0.996 | 0.994 | 0.995 |
| `wiki-tea-article-body` | 1.005 | 0.985 | 1.021 | 1.002 | 1.016 | 1.011 | 1.010 | 1.050 | 1.012 |
| `wiki-tea-first-paragraph` | 1.004 | 1.001 | 1.013 | 1.002 | 0.991 | 1.000 | 1.000 | 1.021 | 1.006 |
| `wiki-tea-lead` | 1.007 | 0.999 | 1.017 | 1.004 | 0.996 | 1.003 | 1.004 | 1.011 | 1.003 |
| `wiki-volcano-article-body` | 1.016 | 0.999 | 1.023 | 1.008 | 0.999 | 1.007 | 0.994 | 1.050 | 1.010 |

## Broad confirmations: 57 documents, 3 rounds × 5 pairs

| Candidate | fresh | reuse | parse | render | fresh rounds | parse rounds |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| `depth` inline depth as a plain cell | 1.0048 | 1.0064 | 1.0087 | 1.0005 | 1.004 / 1.003 / 1.007 | 1.009 / 1.010 / 1.008 |
| `lines` container line facts once per line | 1.0053 | 1.0026 | 1.0055 | 1.0006 | 1.006 / 1.005 / 1.006 | 1.005 / 1.006 / 1.006 |
| `arena` 2 KB reservation floor | 1.0074 | 1.0024 | 1.0035 | 1.0012 | 1.007 / 1.008 / 1.008 | 1.004 / 1.003 / 1.004 |
| `closer` forward window for has_closer_from | 1.0205 | 1.0233 | 1.0289 | 0.9991 | 1.011 / 1.022 / 1.022 | 1.030 / 1.029 / 1.029 |
| `tables` NEON pipe cursor | 1.0004 | 1.0015 | 1.0011 | 0.9996 | 1.001 / 1.001 / 1.000 | 0.999 / 1.002 / 1.002 |

### Per-document fresh ratios, broad

| Document | depth | lines | arena | closer | tables |
| --- | ---: | ---: | ---: | ---: | ---: |
| `comment-ack` | 1.006 | 1.000 | 1.048 | 0.989 | 1.000 |
| `comment-checklist` | 1.003 | 1.023 | 1.008 | 0.976 | 1.008 |
| `comment-incident` | 1.000 | 1.004 | 0.997 | 0.995 | 0.997 |
| `comment-inline-code` | 0.994 | 1.006 | 1.021 | 1.001 | 1.002 |
| `comment-links` | 1.008 | 1.004 | 1.013 | 1.109 | 0.992 |
| `comment-question` | 1.005 | 0.999 | 1.032 | 0.989 | 1.000 |
| `comment-quote` | 1.006 | 0.970 | 1.023 | 0.984 | 0.996 |
| `comment-reproduction` | 1.002 | 0.992 | 1.019 | 0.994 | 1.014 |
| `comment-review` | 1.009 | 0.999 | 1.027 | 0.995 | 1.001 |
| `comment-review-long` | 1.009 | 1.010 | 1.002 | 1.051 | 0.995 |
| `comment-table` | 1.015 | 1.001 | 1.012 | 0.992 | 0.992 |
| `comment-unicode` | 1.004 | 1.008 | 1.019 | 0.986 | 1.000 |
| `guard-angle-link` | 1.010 | 1.001 | 1.028 | 1.185 | 0.996 |
| `legacy-contributing` | 1.001 | 1.009 | 1.009 | 1.011 | 1.001 |
| `legacy-docs-adr-readme-theme-composition` | 0.998 | 1.001 | 1.003 | 1.027 | 0.996 |
| `legacy-docs-markdown-extensions` | 1.000 | 0.999 | 1.003 | 1.022 | 1.001 |
| `legacy-docs-mdx` | 1.002 | 1.007 | 1.000 | 1.012 | 1.008 |
| `legacy-docs-migration-0-2` | 1.007 | 1.006 | 1.019 | 1.011 | 1.005 |
| `legacy-docs-migration-0-3` | 1.008 | 0.996 | 1.005 | 0.998 | 1.001 |
| `legacy-docs-migration-0-4` | 1.001 | 0.995 | 1.008 | 1.009 | 1.000 |
| `legacy-docs-migration-0-8` | 1.002 | 1.005 | 1.010 | 1.006 | 1.003 |
| `legacy-docs-readme` | 1.008 | 1.005 | 1.011 | 1.060 | 1.020 |
| `legacy-docs-readme-theme` | 1.006 | 1.012 | 1.010 | 1.030 | 1.003 |
| `legacy-docs-releasing` | 0.999 | 1.010 | 1.009 | 0.993 | 1.000 |
| `legacy-node-ferromark-readme` | 1.003 | 1.008 | 1.009 | 1.018 | 1.000 |
| `rust-book-appendix-02-operators` | 1.003 | 1.006 | 1.003 | 1.005 | 1.001 |
| `rust-book-ch00-00-introduction` | 1.000 | 0.997 | 0.999 | 1.016 | 0.997 |
| `rust-book-ch03-04-comments` | 1.011 | 1.011 | 1.013 | 1.079 | 1.000 |
| `rust-book-ch17-00-async-await` | 1.008 | 1.007 | 0.993 | 0.998 | 0.999 |
| `typescript-handbook-advanced-types` | 0.996 | 1.019 | 1.032 | 1.020 | 1.007 |
| `typescript-handbook-compiler-options` | 0.999 | 1.000 | 0.996 | 0.994 | 1.001 |
| `typescript-handbook-the-handbook` | 1.005 | 1.024 | 1.009 | 1.026 | 0.997 |
| `typescript-handbook-typescript-5-0` | 0.981 | 1.042 | 1.040 | 1.058 | 1.012 |
| `vite-docs-api-plugin` | 1.001 | 1.059 | 0.992 | 1.025 | 1.003 |
| `vite-docs-features` | 1.027 | 1.007 | 1.014 | 1.038 | 0.984 |
| `vite-docs-performance` | 1.001 | 1.013 | 0.997 | 1.026 | 0.999 |
| `vite-docs-philosophy` | 1.002 | 1.010 | 0.995 | 1.039 | 0.997 |
| `vue-docs-reactivity-in-depth` | 0.995 | 1.004 | 1.006 | 1.037 | 1.002 |
| `vue-docs-slots` | 1.003 | 1.002 | 1.005 | 1.037 | 0.997 |
| `vue-docs-suspense` | 0.999 | 1.010 | 1.004 | 1.014 | 0.997 |
| `vue-docs-ways-of-using-vue` | 1.003 | 1.022 | 0.997 | 1.052 | 1.003 |
| `wiki-chess-article-body` | 1.032 | 0.985 | 1.011 | 1.056 | 1.001 |
| `wiki-chess-first-paragraph` | 1.006 | 1.001 | 1.002 | 1.025 | 1.005 |
| `wiki-chess-lead` | 1.005 | 1.002 | 0.993 | 1.030 | 1.005 |
| `wiki-chess-plain-prose` | 1.005 | 0.997 | 1.001 | 0.998 | 0.997 |
| `wiki-rainbow-article-body` | 1.001 | 0.991 | 0.988 | 1.041 | 0.985 |
| `wiki-rainbow-first-paragraph` | 1.008 | 1.004 | 1.000 | 1.020 | 1.000 |
| `wiki-rainbow-lead` | 1.005 | 1.005 | 1.002 | 1.009 | 1.001 |
| `wiki-rainbow-plain-prose` | 1.000 | 0.994 | 0.995 | 0.998 | 1.002 |
| `wiki-tea-article-body` | 1.035 | 1.002 | 1.002 | 1.035 | 1.004 |
| `wiki-tea-first-paragraph` | 1.003 | 1.007 | 1.001 | 1.012 | 1.002 |
| `wiki-tea-lead` | 1.009 | 1.003 | 1.005 | 1.010 | 1.006 |
| `wiki-tea-plain-prose` | 1.000 | 1.001 | 0.993 | 0.993 | 0.994 |
| `wiki-volcano-article-body` | 1.000 | 1.011 | 1.001 | 1.056 | 1.005 |
| `wiki-volcano-first-paragraph` | 1.012 | 0.994 | 0.999 | 1.006 | 0.998 |
| `wiki-volcano-lead` | 1.018 | 1.005 | 1.001 | 1.010 | 1.000 |
| `wiki-volcano-plain-prose` | 1.001 | 0.997 | 0.993 | 0.993 | 0.991 |

### Per-document parse ratios, broad

| Document | depth | lines | arena | closer | tables |
| --- | ---: | ---: | ---: | ---: | ---: |
| `comment-ack` | 1.011 | 0.996 | 1.004 | 0.972 | 1.002 |
| `comment-checklist` | 1.000 | 1.030 | 1.006 | 0.973 | 1.006 |
| `comment-incident` | 0.999 | 1.001 | 1.001 | 0.985 | 0.998 |
| `comment-inline-code` | 1.018 | 1.001 | 1.001 | 0.999 | 1.013 |
| `comment-links` | 1.012 | 1.017 | 1.020 | 1.171 | 0.999 |
| `comment-question` | 1.012 | 0.995 | 1.002 | 0.976 | 0.999 |
| `comment-quote` | 1.009 | 0.949 | 0.999 | 0.976 | 0.999 |
| `comment-reproduction` | 0.983 | 0.974 | 0.977 | 0.968 | 1.011 |
| `comment-review` | 1.029 | 1.005 | 1.011 | 0.998 | 1.017 |
| `comment-review-long` | 1.006 | 1.013 | 1.003 | 1.065 | 0.999 |
| `comment-table` | 1.020 | 0.999 | 1.010 | 0.986 | 0.986 |
| `comment-unicode` | 1.006 | 0.997 | 1.000 | 0.992 | 0.999 |
| `guard-angle-link` | 1.013 | 0.997 | 1.001 | 1.342 | 0.991 |
| `legacy-contributing` | 1.005 | 1.011 | 1.005 | 1.013 | 0.997 |
| `legacy-docs-adr-readme-theme-composition` | 1.004 | 1.009 | 1.006 | 1.048 | 0.999 |
| `legacy-docs-markdown-extensions` | 1.003 | 1.007 | 1.007 | 1.020 | 1.001 |
| `legacy-docs-mdx` | 1.009 | 1.011 | 1.007 | 1.027 | 1.008 |
| `legacy-docs-migration-0-2` | 1.005 | 1.010 | 1.013 | 1.006 | 0.994 |
| `legacy-docs-migration-0-3` | 1.006 | 0.991 | 1.011 | 0.998 | 1.002 |
| `legacy-docs-migration-0-4` | 1.001 | 1.005 | 0.999 | 1.014 | 1.000 |
| `legacy-docs-migration-0-8` | 1.006 | 0.998 | 1.007 | 1.003 | 0.995 |
| `legacy-docs-readme` | 1.014 | 1.008 | 1.009 | 1.079 | 1.027 |
| `legacy-docs-readme-theme` | 1.009 | 1.014 | 1.014 | 1.034 | 0.999 |
| `legacy-docs-releasing` | 0.996 | 1.014 | 0.997 | 0.991 | 0.999 |
| `legacy-node-ferromark-readme` | 1.003 | 1.006 | 1.006 | 1.026 | 1.009 |
| `rust-book-appendix-02-operators` | 1.004 | 1.002 | 1.007 | 1.003 | 0.994 |
| `rust-book-ch00-00-introduction` | 1.000 | 0.999 | 1.002 | 1.011 | 0.998 |
| `rust-book-ch03-04-comments` | 1.016 | 1.016 | 1.013 | 1.139 | 1.000 |
| `rust-book-ch17-00-async-await` | 1.008 | 1.001 | 0.998 | 0.995 | 0.998 |
| `typescript-handbook-advanced-types` | 1.004 | 1.014 | 1.016 | 1.022 | 0.995 |
| `typescript-handbook-compiler-options` | 0.998 | 1.002 | 0.996 | 1.003 | 1.003 |
| `typescript-handbook-the-handbook` | 1.010 | 1.039 | 1.006 | 1.030 | 1.004 |
| `typescript-handbook-typescript-5-0` | 1.010 | 1.016 | 0.987 | 1.030 | 0.992 |
| `vite-docs-api-plugin` | 1.008 | 1.084 | 0.997 | 1.029 | 0.993 |
| `vite-docs-features` | 0.996 | 1.004 | 1.000 | 1.059 | 0.987 |
| `vite-docs-performance` | 1.001 | 1.020 | 0.999 | 1.049 | 0.999 |
| `vite-docs-philosophy` | 1.009 | 1.015 | 1.004 | 1.056 | 1.006 |
| `vue-docs-reactivity-in-depth` | 1.012 | 1.005 | 1.005 | 1.045 | 1.002 |
| `vue-docs-slots` | 1.006 | 1.012 | 1.002 | 1.059 | 1.004 |
| `vue-docs-suspense` | 1.000 | 1.009 | 1.000 | 1.033 | 0.998 |
| `vue-docs-ways-of-using-vue` | 1.012 | 1.026 | 1.002 | 1.084 | 1.002 |
| `wiki-chess-article-body` | 1.027 | 1.002 | 1.015 | 1.077 | 1.017 |
| `wiki-chess-first-paragraph` | 1.012 | 1.006 | 1.000 | 1.041 | 1.003 |
| `wiki-chess-lead` | 1.008 | 1.000 | 1.002 | 1.036 | 1.003 |
| `wiki-chess-plain-prose` | 1.004 | 0.998 | 1.001 | 0.994 | 0.995 |
| `wiki-rainbow-article-body` | 1.008 | 1.003 | 1.004 | 1.061 | 1.001 |
| `wiki-rainbow-first-paragraph` | 1.015 | 1.004 | 1.006 | 1.035 | 1.002 |
| `wiki-rainbow-lead` | 1.013 | 0.996 | 1.000 | 1.020 | 1.000 |
| `wiki-rainbow-plain-prose` | 1.006 | 0.987 | 0.998 | 0.999 | 1.000 |
| `wiki-tea-article-body` | 1.027 | 0.977 | 1.010 | 1.054 | 1.005 |
| `wiki-tea-first-paragraph` | 1.014 | 1.001 | 1.000 | 1.021 | 1.000 |
| `wiki-tea-lead` | 1.019 | 1.005 | 1.002 | 1.007 | 1.000 |
| `wiki-tea-plain-prose` | 1.005 | 1.006 | 0.998 | 0.997 | 1.000 |
| `wiki-volcano-article-body` | 1.018 | 1.015 | 1.016 | 1.061 | 1.007 |
| `wiki-volcano-first-paragraph` | 1.021 | 0.998 | 1.000 | 1.014 | 1.008 |
| `wiki-volcano-lead` | 1.018 | 0.998 | 0.995 | 1.010 | 1.000 |
| `wiki-volcano-plain-prose` | 1.006 | 1.006 | 1.001 | 0.995 | 0.998 |

