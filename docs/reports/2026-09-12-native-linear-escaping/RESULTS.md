# Native Linux paired results

Generated from the archived windows. Negative values are faster; positive values are slower. Each cell preserves one fresh process pair. No workload weighting or A/A subtraction is applied.

## round-1

AMD EPYC 9V74 80-Core Processor; rustc 1.98.1 (48a229cea 2026-09-01).

### aa-control / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.189 | 0.190 | -0.14% | +0.33% |
| direct-escape/ci-html-heavy | 10.617 | 11.098 | +3.62% | +5.45% |
| direct-escape/plain-128 | 0.021 | 0.021 | +0.21% | -0.45% |
| direct-escape/late-quote-128 | 0.065 | 0.066 | +2.67% | +0.01% |
| direct-escape/plain-512 | 0.036 | 0.036 | -0.24% | -0.18% |
| direct-escape/late-quote-512 | 0.089 | 0.092 | +0.37% | +4.97% |
| direct-escape/plain-1024 | 0.051 | 0.051 | +0.27% | -0.17% |
| direct-escape/late-quote-1024 | 0.104 | 0.107 | +2.91% | +1.24% |
| direct-escape/plain-4096 | 0.164 | 0.165 | +1.18% | +0.40% |
| direct-escape/late-quote-4096 | 0.191 | 0.195 | +3.56% | +0.56% |
| direct-escape/plain-8192 | 0.268 | 0.268 | +1.27% | -0.87% |
| direct-escape/late-quote-8192 | 0.297 | 0.298 | +1.02% | -0.05% |
| direct-escape/plain-65536 | 2.714 | 2.712 | +0.01% | -0.18% |
| direct-escape/late-quote-65536 | 2.738 | 3.080 | +24.86% | +0.12% |
| direct-escape/quotes-4096 | 18.843 | 18.867 | -0.13% | +0.38% |
| direct-escape/attr-quotes-4096 | 77.450 | 77.308 | -0.09% | -0.28% |
| direct-escape/quotes-65536 | 3650.341 | 3646.823 | +0.02% | -0.21% |
| direct-escape/attr-quotes-65536 | 14517.163 | 14478.707 | -0.44% | -0.09% |

### aa-control / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 298.843 | 298.896 | +0.55% | -0.51% |
| ox-huge-tables | 7234.039 | 7164.668 | -1.03% | -0.89% |
| short | 0.519 | 0.515 | -0.17% | -1.35% |
| multiline | 15.335 | 15.326 | -0.00% | -0.12% |
| inline | 55.563 | 55.609 | +0.95% | -0.78% |
| code | 16.260 | 16.187 | -0.56% | -0.34% |
| escape-dense | 15.704 | 15.738 | +0.10% | +0.34% |
| tables/tables-plain | 29.875 | 29.778 | -1.02% | +0.38% |
| guard/one-table | 19.464 | 19.415 | -0.49% | -0.01% |
| mdx/short | 1.424 | 1.436 | +0.72% | +0.91% |
| mdx/tables-code | 183.890 | 183.840 | -0.60% | +0.55% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.370 | 21.392 | +1.33% | -1.11% |
| corpus/vue-docs/src/guide/extras/render-function.md | 46.445 | 46.357 | +0.23% | -0.61% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 68.264 | 68.754 | +0.61% | +0.82% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 19.872 | 19.758 | -0.79% | -0.36% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 30.971 | 30.976 | -0.88% | +0.93% |
| html/declaration-long-lines | 3.287 | 3.289 | -0.16% | +0.27% |
| fence/root-indented | 311.732 | 314.570 | +0.75% | +1.07% |
| fence/root-container | 343.935 | 345.882 | +0.22% | +0.91% |
| tables | 49.714 | 49.806 | +0.32% | +0.05% |
| guard/deep-emphasis | 49.896 | 49.996 | +0.49% | -0.09% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.965 | 1.964 | +0.37% | -0.48% |
| html/comment-long-lines | 3.200 | 3.196 | +0.31% | -0.51% |
| html/script-candidates | 47.474 | 47.528 | +0.44% | -0.21% |
| html/processing-long-lines | 3.310 | 3.306 | +0.15% | -0.44% |
| html/custom-tags | 225.249 | 224.812 | +0.27% | -0.65% |

### current / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.190 | 0.235 | +23.61% | +23.55% |
| direct-escape/ci-html-heavy | 10.995 | 4.333 | -60.36% | -60.82% |
| direct-escape/plain-128 | 0.022 | 0.022 | -1.50% | +5.24% |
| direct-escape/late-quote-128 | 0.066 | 0.064 | -5.33% | -0.11% |
| direct-escape/plain-512 | 0.037 | 0.052 | +37.79% | +44.29% |
| direct-escape/late-quote-512 | 0.090 | 0.104 | +13.57% | +17.80% |
| direct-escape/plain-1024 | 0.051 | 0.073 | +38.24% | +45.81% |
| direct-escape/late-quote-1024 | 0.107 | 0.129 | +17.32% | +24.60% |
| direct-escape/plain-4096 | 0.166 | 0.202 | +20.11% | +23.29% |
| direct-escape/late-quote-4096 | 0.194 | 0.229 | +15.92% | +20.39% |
| direct-escape/plain-8192 | 0.269 | 0.320 | +17.99% | +20.16% |
| direct-escape/late-quote-8192 | 0.299 | 0.349 | +15.82% | +17.89% |
| direct-escape/plain-65536 | 2.714 | 2.553 | -5.86% | -6.03% |
| direct-escape/late-quote-65536 | 3.249 | 3.438 | -4.42% | +19.89% |
| direct-escape/quotes-4096 | 18.922 | 3.071 | -83.76% | -83.78% |
| direct-escape/attr-quotes-4096 | 77.481 | 12.875 | -83.35% | -83.41% |
| direct-escape/quotes-65536 | 3649.621 | 48.418 | -98.68% | -98.67% |
| direct-escape/attr-quotes-65536 | 14493.578 | 205.020 | -98.60% | -98.58% |

### current / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 297.195 | 294.328 | -0.54% | -1.39% |
| ox-huge-tables | 7165.391 | 7002.866 | -2.27% | -2.26% |
| short | 0.522 | 0.521 | +2.32% | -2.30% |
| multiline | 15.285 | 15.195 | -0.24% | -0.94% |
| inline | 55.345 | 54.213 | -3.19% | -0.89% |
| code | 16.234 | 15.537 | -4.41% | -4.18% |
| escape-dense | 15.719 | 15.582 | -1.11% | -0.63% |
| tables/tables-plain | 29.480 | 29.845 | +1.68% | +0.79% |
| guard/one-table | 19.382 | 19.319 | -0.35% | -0.30% |
| mdx/short | 1.426 | 1.408 | -1.89% | -0.61% |
| mdx/tables-code | 183.006 | 178.585 | -2.47% | -2.36% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.255 | 20.233 | -4.38% | -5.23% |
| corpus/vue-docs/src/guide/extras/render-function.md | 46.290 | 44.454 | -3.62% | -4.31% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 68.357 | 68.813 | +0.81% | +0.52% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 19.812 | 18.889 | -4.56% | -4.76% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 30.955 | 30.074 | -3.68% | -2.00% |
| html/declaration-long-lines | 3.290 | 3.302 | +0.60% | +0.14% |
| fence/root-indented | 313.130 | 309.930 | -2.47% | +0.44% |
| fence/root-container | 345.124 | 339.687 | -2.39% | -0.76% |
| tables | 49.541 | 49.433 | +0.13% | -0.57% |
| guard/deep-emphasis | 50.044 | 50.262 | +0.46% | +0.41% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.969 | 1.943 | -2.49% | -0.15% |
| html/comment-long-lines | 3.193 | 3.199 | +1.03% | -0.67% |
| html/script-candidates | 47.485 | 47.676 | +0.54% | +0.26% |
| html/processing-long-lines | 3.313 | 3.323 | +1.04% | -0.44% |
| html/custom-tags | 225.421 | 244.754 | -1.03% | +18.13% |

### no-prefix-1024 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.189 | 0.213 | +12.84% | +12.27% |
| direct-escape/ci-html-heavy | 10.668 | 11.212 | +4.33% | +5.88% |
| direct-escape/plain-128 | 0.022 | 0.023 | +6.75% | -1.57% |
| direct-escape/late-quote-128 | 0.066 | 0.067 | +1.31% | +3.63% |
| direct-escape/plain-512 | 0.037 | 0.039 | +9.34% | +4.51% |
| direct-escape/late-quote-512 | 0.089 | 0.090 | +3.24% | -0.39% |
| direct-escape/plain-1024 | 0.052 | 0.052 | +1.76% | -2.57% |
| direct-escape/late-quote-1024 | 0.105 | 0.107 | +3.30% | +0.28% |
| direct-escape/plain-4096 | 0.163 | 0.182 | +12.69% | +10.98% |
| direct-escape/late-quote-4096 | 0.190 | 0.208 | +9.77% | +8.95% |
| direct-escape/plain-8192 | 0.268 | 0.298 | +11.85% | +10.89% |
| direct-escape/late-quote-8192 | 0.296 | 0.326 | +10.23% | +9.83% |
| direct-escape/plain-65536 | 2.715 | 2.534 | -6.56% | -6.77% |
| direct-escape/late-quote-65536 | 3.595 | 3.408 | -13.93% | +4.45% |
| direct-escape/quotes-4096 | 18.860 | 12.649 | -33.03% | -32.84% |
| direct-escape/attr-quotes-4096 | 78.201 | 50.996 | -35.96% | -33.59% |
| direct-escape/quotes-65536 | 3661.794 | 213.050 | -94.21% | -94.15% |
| direct-escape/attr-quotes-65536 | 14503.583 | 873.008 | -94.03% | -93.94% |

### no-prefix-1024 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 297.522 | 304.892 | +4.32% | +0.62% |
| ox-huge-tables | 7125.283 | 7251.069 | +3.36% | +0.15% |
| short | 0.515 | 0.506 | -1.66% | -1.81% |
| multiline | 15.336 | 16.152 | +10.98% | -0.36% |
| inline | 55.768 | 54.582 | -1.29% | -2.95% |
| code | 16.257 | 16.329 | +1.12% | -0.23% |
| escape-dense | 15.713 | 15.893 | +1.07% | +1.22% |
| tables/tables-plain | 29.518 | 30.543 | +6.88% | +0.07% |
| guard/one-table | 19.525 | 20.074 | +6.52% | -0.91% |
| mdx/short | 1.414 | 1.396 | -1.44% | -0.99% |
| mdx/tables-code | 183.550 | 181.799 | -0.16% | -1.75% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.278 | 21.847 | +3.17% | +2.17% |
| corpus/vue-docs/src/guide/extras/render-function.md | 46.290 | 47.460 | +3.64% | +1.42% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 68.507 | 69.090 | +2.19% | -0.47% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 19.840 | 20.112 | +1.75% | +1.00% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 31.311 | 31.170 | +0.06% | -0.96% |
| html/declaration-long-lines | 3.275 | 3.278 | +0.28% | -0.11% |
| fence/root-indented | 312.397 | 309.637 | -1.22% | -0.55% |
| fence/root-container | 344.578 | 341.474 | -0.96% | -0.84% |
| tables | 49.546 | 50.046 | +4.02% | -2.00% |
| guard/deep-emphasis | 49.998 | 49.882 | -0.03% | -0.44% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.972 | 1.995 | +3.26% | -0.88% |
| html/comment-long-lines | 3.191 | 3.191 | +0.18% | -0.14% |
| html/script-candidates | 47.739 | 48.048 | +2.22% | -0.92% |
| html/processing-long-lines | 3.301 | 3.315 | +1.17% | -0.30% |
| html/custom-tags | 224.790 | 231.738 | +3.78% | +2.40% |

### no-prefix-256 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.189 | 0.231 | +22.57% | +22.40% |
| direct-escape/ci-html-heavy | 10.856 | 11.327 | +3.02% | +5.68% |
| direct-escape/plain-128 | 0.021 | 0.022 | +6.66% | +6.65% |
| direct-escape/late-quote-128 | 0.064 | 0.065 | +2.16% | +1.30% |
| direct-escape/plain-512 | 0.036 | 0.048 | +32.84% | +32.51% |
| direct-escape/late-quote-512 | 0.090 | 0.100 | +10.35% | +13.28% |
| direct-escape/plain-1024 | 0.051 | 0.069 | +40.31% | +32.25% |
| direct-escape/late-quote-1024 | 0.104 | 0.126 | +22.70% | +19.48% |
| direct-escape/plain-4096 | 0.172 | 0.200 | +11.30% | +22.45% |
| direct-escape/late-quote-4096 | 0.191 | 0.229 | +19.78% | +19.92% |
| direct-escape/plain-8192 | 0.269 | 0.317 | +18.35% | +17.27% |
| direct-escape/late-quote-8192 | 0.297 | 0.344 | +16.17% | +15.93% |
| direct-escape/plain-65536 | 2.800 | 2.550 | -11.56% | -6.16% |
| direct-escape/late-quote-65536 | 3.251 | 3.586 | -4.57% | +30.75% |
| direct-escape/quotes-4096 | 18.981 | 9.159 | -51.61% | -51.89% |
| direct-escape/attr-quotes-4096 | 77.269 | 34.007 | -54.88% | -57.09% |
| direct-escape/quotes-65536 | 3653.436 | 147.453 | -95.94% | -95.99% |
| direct-escape/attr-quotes-65536 | 14480.803 | 552.131 | -96.09% | -96.29% |

### no-prefix-256 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 300.364 | 302.469 | +0.47% | +0.94% |
| ox-huge-tables | 7218.598 | 7262.647 | +0.25% | +0.98% |
| short | 0.516 | 0.498 | -3.98% | -3.10% |
| multiline | 15.282 | 15.213 | -0.68% | -0.22% |
| inline | 55.543 | 52.678 | -4.85% | -5.47% |
| code | 16.362 | 16.210 | -0.17% | -1.68% |
| escape-dense | 15.675 | 15.888 | +0.97% | +1.75% |
| tables/tables-plain | 29.535 | 29.647 | +0.52% | +0.25% |
| guard/one-table | 19.343 | 19.537 | +0.91% | +1.09% |
| mdx/short | 1.417 | 1.387 | -1.87% | -2.35% |
| mdx/tables-code | 183.645 | 184.579 | +1.49% | -0.47% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.436 | 21.237 | -0.74% | -1.13% |
| corpus/vue-docs/src/guide/extras/render-function.md | 46.793 | 46.506 | -0.02% | -1.21% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 68.881 | 68.450 | +0.05% | -1.30% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 19.912 | 20.197 | +1.35% | +1.51% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 31.832 | 31.212 | +0.09% | -3.92% |
| html/declaration-long-lines | 3.303 | 3.297 | -0.89% | +0.54% |
| fence/root-indented | 312.512 | 305.995 | -1.63% | -2.54% |
| fence/root-container | 344.501 | 338.049 | -1.54% | -2.21% |
| tables | 49.446 | 48.860 | -1.51% | -0.86% |
| guard/deep-emphasis | 49.848 | 49.802 | -0.15% | -0.03% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.982 | 1.981 | +0.60% | -0.69% |
| html/comment-long-lines | 3.211 | 3.198 | +0.05% | -0.84% |
| html/script-candidates | 47.485 | 47.475 | +0.07% | -0.11% |
| html/processing-long-lines | 3.322 | 3.316 | +0.00% | -0.40% |
| html/custom-tags | 224.795 | 231.615 | +2.94% | +3.13% |

### prefix-8192 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.192 | 0.199 | +4.76% | +2.59% |
| direct-escape/ci-html-heavy | 11.011 | 4.337 | -61.67% | -59.50% |
| direct-escape/plain-128 | 0.021 | 0.022 | +3.23% | +6.06% |
| direct-escape/late-quote-128 | 0.065 | 0.064 | -0.87% | -1.54% |
| direct-escape/plain-512 | 0.037 | 0.042 | +17.70% | +12.40% |
| direct-escape/late-quote-512 | 0.088 | 0.096 | +9.23% | +7.61% |
| direct-escape/plain-1024 | 0.051 | 0.056 | +10.25% | +10.79% |
| direct-escape/late-quote-1024 | 0.104 | 0.113 | +9.69% | +7.54% |
| direct-escape/plain-4096 | 0.163 | 0.169 | +3.45% | +4.09% |
| direct-escape/late-quote-4096 | 0.191 | 0.197 | +3.90% | +2.48% |
| direct-escape/plain-8192 | 0.266 | 0.273 | +2.56% | +2.45% |
| direct-escape/late-quote-8192 | 0.295 | 0.302 | +2.22% | +2.37% |
| direct-escape/plain-65536 | 3.134 | 2.595 | -12.92% | -20.85% |
| direct-escape/late-quote-65536 | 2.739 | 3.220 | +30.28% | +4.90% |
| direct-escape/quotes-4096 | 18.857 | 3.076 | -83.68% | -83.70% |
| direct-escape/attr-quotes-4096 | 79.682 | 12.794 | -83.95% | -83.93% |
| direct-escape/quotes-65536 | 3649.652 | 48.440 | -98.68% | -98.67% |
| direct-escape/attr-quotes-65536 | 14446.890 | 204.240 | -98.59% | -98.58% |

### prefix-8192 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 300.622 | 293.745 | -1.63% | -2.93% |
| ox-huge-tables | 7146.188 | 6978.918 | -2.73% | -1.95% |
| short | 0.517 | 0.520 | +0.46% | +0.74% |
| multiline | 15.298 | 15.158 | -0.84% | -1.00% |
| inline | 55.351 | 53.892 | -2.67% | -2.60% |
| code | 16.227 | 15.524 | -4.14% | -4.52% |
| escape-dense | 15.691 | 15.634 | -0.37% | -0.37% |
| tables/tables-plain | 29.532 | 29.898 | +0.98% | +1.50% |
| guard/one-table | 19.515 | 19.360 | -0.10% | -1.49% |
| mdx/short | 1.413 | 1.413 | -0.63% | +0.61% |
| mdx/tables-code | 183.829 | 178.723 | -2.58% | -2.98% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.373 | 20.241 | -5.42% | -5.16% |
| corpus/vue-docs/src/guide/extras/render-function.md | 46.549 | 44.163 | -5.38% | -4.87% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 68.398 | 68.004 | -0.85% | -0.30% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 19.949 | 18.960 | -4.94% | -4.97% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 31.106 | 30.116 | -5.25% | -1.03% |
| html/declaration-long-lines | 3.319 | 3.340 | +1.14% | +0.11% |
| fence/root-indented | 313.198 | 312.101 | +0.18% | -0.89% |
| fence/root-container | 345.105 | 340.464 | -1.13% | -1.56% |
| tables | 49.649 | 49.934 | +0.71% | +0.43% |
| guard/deep-emphasis | 50.976 | 50.443 | -2.60% | +0.57% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.971 | 1.943 | -1.01% | -1.77% |
| html/comment-long-lines | 3.259 | 3.195 | -2.85% | -1.06% |
| html/script-candidates | 48.286 | 47.724 | -3.55% | +1.31% |
| html/processing-long-lines | 3.361 | 3.313 | -2.54% | -0.25% |
| html/custom-tags | 229.888 | 223.510 | -2.83% | -2.72% |

### threshold-1024 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.189 | 0.231 | +21.80% | +22.28% |
| direct-escape/ci-html-heavy | 11.035 | 11.330 | +2.55% | +2.79% |
| direct-escape/plain-128 | 0.021 | 0.022 | +6.89% | +7.51% |
| direct-escape/late-quote-128 | 0.064 | 0.065 | -0.24% | +2.79% |
| direct-escape/plain-512 | 0.037 | 0.037 | +1.71% | -0.69% |
| direct-escape/late-quote-512 | 0.088 | 0.089 | -0.46% | +1.37% |
| direct-escape/plain-1024 | 0.050 | 0.052 | +3.08% | +2.55% |
| direct-escape/late-quote-1024 | 0.107 | 0.106 | -2.53% | +0.54% |
| direct-escape/plain-4096 | 0.163 | 0.198 | +22.01% | +21.60% |
| direct-escape/late-quote-4096 | 0.191 | 0.228 | +19.17% | +19.91% |
| direct-escape/plain-8192 | 0.267 | 0.324 | +24.13% | +18.74% |
| direct-escape/late-quote-8192 | 0.297 | 0.353 | +20.80% | +17.16% |
| direct-escape/plain-65536 | 2.774 | 2.868 | +5.46% | +1.36% |
| direct-escape/late-quote-65536 | 2.742 | 2.691 | -6.56% | +2.84% |
| direct-escape/quotes-4096 | 18.840 | 9.313 | -50.51% | -50.63% |
| direct-escape/attr-quotes-4096 | 77.301 | 34.627 | -54.82% | -55.59% |
| direct-escape/quotes-65536 | 3650.688 | 146.948 | -95.94% | -96.01% |
| direct-escape/attr-quotes-65536 | 14483.037 | 540.768 | -96.26% | -96.28% |

### threshold-1024 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 296.821 | 291.285 | -1.79% | -1.94% |
| ox-huge-tables | 7111.890 | 6914.827 | -2.47% | -3.08% |
| short | 0.517 | 0.505 | -2.64% | -2.02% |
| multiline | 15.306 | 15.381 | +1.52% | -0.55% |
| inline | 55.509 | 53.036 | -3.46% | -5.44% |
| code | 16.241 | 15.812 | -3.13% | -2.15% |
| escape-dense | 15.726 | 16.209 | +2.72% | +3.44% |
| tables/tables-plain | 29.683 | 29.494 | -0.70% | -0.57% |
| guard/one-table | 19.530 | 19.293 | -1.12% | -1.30% |
| mdx/short | 1.422 | 1.421 | -0.37% | +0.11% |
| mdx/tables-code | 183.224 | 181.234 | -1.36% | -0.81% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.436 | 20.927 | -1.74% | -2.99% |
| corpus/vue-docs/src/guide/extras/render-function.md | 47.102 | 45.174 | -3.64% | -4.54% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 69.433 | 68.656 | -1.16% | -1.08% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 19.814 | 19.566 | -0.79% | -1.71% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 32.256 | 30.259 | -5.52% | -6.86% |
| html/declaration-long-lines | 3.308 | 3.303 | -0.57% | +0.25% |
| fence/root-indented | 310.866 | 311.967 | +0.96% | -0.25% |
| fence/root-container | 342.972 | 342.803 | +0.38% | -0.48% |
| tables | 49.236 | 48.031 | -2.25% | -2.65% |
| guard/deep-emphasis | 49.862 | 50.778 | +1.74% | +1.94% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.965 | 1.921 | -1.17% | -3.27% |
| html/comment-long-lines | 3.193 | 3.197 | +0.38% | -0.10% |
| html/script-candidates | 47.384 | 47.638 | +0.48% | +0.59% |
| html/processing-long-lines | 3.311 | 3.310 | -0.44% | +0.37% |
| html/custom-tags | 225.803 | 232.640 | +3.50% | +2.56% |

### threshold-8192 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.189 | 0.188 | -0.18% | -0.88% |
| direct-escape/ci-html-heavy | 11.033 | 10.646 | -6.50% | -0.42% |
| direct-escape/plain-128 | 0.021 | 0.022 | +6.67% | +7.19% |
| direct-escape/late-quote-128 | 0.065 | 0.065 | -0.68% | +1.71% |
| direct-escape/plain-512 | 0.036 | 0.037 | +1.81% | +0.68% |
| direct-escape/late-quote-512 | 0.089 | 0.088 | -1.21% | -0.05% |
| direct-escape/plain-1024 | 0.050 | 0.051 | +1.03% | -0.32% |
| direct-escape/late-quote-1024 | 0.105 | 0.106 | +0.93% | -0.10% |
| direct-escape/plain-4096 | 0.164 | 0.163 | -0.79% | -0.44% |
| direct-escape/late-quote-4096 | 0.191 | 0.192 | -0.68% | +2.03% |
| direct-escape/plain-8192 | 0.268 | 0.267 | -0.94% | +0.35% |
| direct-escape/late-quote-8192 | 0.298 | 0.296 | -1.76% | +0.07% |
| direct-escape/plain-65536 | 2.798 | 2.539 | -11.84% | -6.49% |
| direct-escape/late-quote-65536 | 3.250 | 3.585 | +5.53% | +14.63% |
| direct-escape/quotes-4096 | 18.854 | 19.152 | +1.53% | +1.64% |
| direct-escape/attr-quotes-4096 | 77.366 | 77.476 | +0.23% | +0.05% |
| direct-escape/quotes-65536 | 3651.799 | 195.189 | -94.63% | -94.68% |
| direct-escape/attr-quotes-65536 | 14546.234 | 741.539 | -94.93% | -94.87% |

### threshold-8192 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 298.025 | 290.363 | -1.58% | -3.55% |
| ox-huge-tables | 7110.760 | 6844.236 | -3.05% | -4.44% |
| short | 0.511 | 0.527 | +5.20% | +0.98% |
| multiline | 15.300 | 15.482 | +3.35% | -0.96% |
| inline | 55.406 | 52.625 | -4.32% | -5.72% |
| code | 16.211 | 15.796 | -2.51% | -2.60% |
| escape-dense | 15.717 | 16.316 | +3.99% | +3.62% |
| tables/tables-plain | 29.708 | 29.229 | -1.49% | -1.74% |
| guard/one-table | 19.544 | 19.165 | -1.64% | -2.24% |
| mdx/short | 1.415 | 1.447 | +2.39% | +2.10% |
| mdx/tables-code | 183.489 | 182.303 | -0.37% | -0.93% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 21.305 | 20.932 | -1.60% | -1.90% |
| corpus/vue-docs/src/guide/extras/render-function.md | 46.682 | 44.969 | -3.97% | -3.36% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 69.047 | 68.865 | +0.71% | -1.24% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 20.140 | 19.648 | -3.61% | -1.23% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 30.751 | 30.498 | +0.27% | -1.91% |
| html/declaration-long-lines | 3.301 | 3.291 | -0.30% | -0.30% |
| fence/root-indented | 313.469 | 312.850 | +0.15% | -0.54% |
| fence/root-container | 344.893 | 342.941 | -0.40% | -0.73% |
| tables | 49.394 | 49.653 | +4.03% | -3.02% |
| guard/deep-emphasis | 49.790 | 50.742 | +1.83% | +1.99% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.982 | 1.971 | +0.76% | -1.87% |
| html/comment-long-lines | 3.194 | 3.187 | -0.23% | -0.21% |
| html/script-candidates | 47.429 | 47.449 | -0.06% | +0.15% |
| html/processing-long-lines | 3.314 | 3.336 | +0.05% | +1.26% |
| html/custom-tags | 224.998 | 233.265 | +3.46% | +3.88% |

## round-2

AMD EPYC 7763 64-Core Processor; rustc 1.98.1 (48a229cea 2026-09-01).

### aa-control / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.276 | 0.281 | -1.41% | +5.21% |
| direct-escape/ci-html-heavy | 11.998 | 12.014 | -1.82% | +2.12% |
| direct-escape/plain-128 | 0.027 | 0.026 | -1.05% | -0.29% |
| direct-escape/late-quote-128 | 0.089 | 0.087 | -1.22% | -1.78% |
| direct-escape/plain-512 | 0.044 | 0.044 | -0.83% | +1.07% |
| direct-escape/late-quote-512 | 0.126 | 0.126 | -0.23% | +0.27% |
| direct-escape/plain-1024 | 0.062 | 0.062 | -0.68% | +0.29% |
| direct-escape/late-quote-1024 | 0.138 | 0.142 | +0.80% | +4.34% |
| direct-escape/plain-4096 | 0.198 | 0.206 | +0.32% | +7.54% |
| direct-escape/late-quote-4096 | 0.238 | 0.242 | +0.16% | +3.95% |
| direct-escape/plain-8192 | 0.363 | 0.371 | +0.18% | +3.89% |
| direct-escape/late-quote-8192 | 0.391 | 0.395 | -1.13% | +3.27% |
| direct-escape/plain-65536 | 3.055 | 3.054 | -0.18% | +0.11% |
| direct-escape/late-quote-65536 | 3.118 | 3.085 | -0.38% | -1.73% |
| direct-escape/quotes-4096 | 26.255 | 26.062 | -1.54% | +0.09% |
| direct-escape/attr-quotes-4096 | 96.933 | 98.160 | +2.62% | -0.09% |
| direct-escape/quotes-65536 | 4528.933 | 4549.930 | +0.46% | +0.47% |
| direct-escape/attr-quotes-65536 | 17961.923 | 17938.000 | -0.15% | -0.12% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.047 | -2.75% | -0.18% |
| direct-escape/late-quote-reserved-8192 | 0.371 | 0.370 | -0.50% | -0.09% |
| direct-escape/late-quote-reserved-65536 | 3.106 | 3.087 | +0.28% | -1.47% |
| direct-escape/html-heavy-65536 | 156.963 | 157.337 | -0.64% | +1.11% |

### aa-control / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 408.882 | 411.501 | +0.55% | +0.73% |
| ox-huge-tables | 9569.441 | 9561.913 | -0.06% | -0.10% |
| short | 0.699 | 0.703 | +1.41% | -0.31% |
| multiline | 20.342 | 20.660 | +2.20% | +0.94% |
| inline | 75.239 | 74.921 | -0.07% | -0.77% |
| code | 22.119 | 22.028 | -0.31% | -0.52% |
| escape-dense | 21.143 | 21.124 | -0.03% | -0.14% |
| tables/tables-plain | 39.615 | 39.592 | -0.18% | +0.07% |
| guard/one-table | 25.378 | 25.346 | +0.61% | -0.86% |
| mdx/short | 1.896 | 1.940 | +1.73% | +2.86% |
| mdx/tables-code | 255.876 | 255.330 | -1.21% | +0.79% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 28.855 | 29.275 | +1.95% | +0.96% |
| corpus/vue-docs/src/guide/extras/render-function.md | 67.581 | 68.707 | +3.36% | -0.04% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 103.185 | 103.970 | +1.28% | +0.23% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.933 | 27.620 | -2.98% | +0.83% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 44.017 | 42.517 | -6.07% | -0.57% |
| html/declaration-long-lines | 3.909 | 3.935 | +2.82% | -1.44% |
| fence/root-indented | 408.851 | 408.954 | +0.30% | -0.25% |
| fence/root-container | 455.179 | 452.403 | -0.03% | -1.18% |
| tables | 65.937 | 65.436 | -0.85% | -0.66% |
| guard/deep-emphasis | 63.843 | 62.443 | -3.33% | -1.08% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.532 | 2.519 | +0.90% | -1.90% |
| html/comment-long-lines | 3.784 | 3.773 | +0.87% | -1.39% |
| html/script-candidates | 56.897 | 56.993 | +0.37% | -0.03% |
| html/processing-long-lines | 3.914 | 3.928 | +1.14% | -0.41% |
| html/custom-tags | 347.966 | 330.969 | -9.24% | -0.10% |

### bulk-cached-8192 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.274 | 0.266 | -2.84% | -2.89% |
| direct-escape/ci-html-heavy | 11.833 | 13.178 | +11.49% | +11.26% |
| direct-escape/plain-128 | 0.026 | 0.027 | +2.07% | +2.34% |
| direct-escape/late-quote-128 | 0.090 | 0.087 | +0.19% | -5.43% |
| direct-escape/plain-512 | 0.044 | 0.043 | -2.85% | -2.88% |
| direct-escape/late-quote-512 | 0.126 | 0.128 | +0.39% | +1.84% |
| direct-escape/plain-1024 | 0.062 | 0.060 | -1.99% | -2.53% |
| direct-escape/late-quote-1024 | 0.138 | 0.134 | -2.54% | -2.73% |
| direct-escape/plain-4096 | 0.198 | 0.193 | -2.25% | -2.57% |
| direct-escape/late-quote-4096 | 0.238 | 0.226 | -4.74% | -5.02% |
| direct-escape/plain-8192 | 0.363 | 0.353 | -2.77% | -2.73% |
| direct-escape/late-quote-8192 | 0.389 | 0.373 | -4.00% | -4.21% |
| direct-escape/plain-65536 | 3.031 | 3.112 | +1.26% | +4.09% |
| direct-escape/late-quote-65536 | 3.094 | 3.107 | +1.19% | -0.36% |
| direct-escape/quotes-4096 | 26.055 | 25.561 | -1.79% | -2.00% |
| direct-escape/attr-quotes-4096 | 96.728 | 95.741 | -0.96% | -1.08% |
| direct-escape/quotes-65536 | 4535.264 | 97.792 | -97.83% | -97.86% |
| direct-escape/attr-quotes-65536 | 17818.042 | 412.406 | -97.68% | -97.69% |
| direct-escape/late-quote-reserved-512 | 0.047 | 0.046 | -3.48% | -2.87% |
| direct-escape/late-quote-reserved-8192 | 0.369 | 0.359 | -2.84% | -2.45% |
| direct-escape/late-quote-reserved-65536 | 3.071 | 3.070 | +0.68% | -0.79% |
| direct-escape/html-heavy-65536 | 155.968 | 121.295 | -22.16% | -22.30% |

### bulk-cached-8192 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 410.914 | 420.248 | +2.87% | +1.68% |
| ox-huge-tables | 9573.350 | 9763.413 | +2.80% | +1.18% |
| short | 0.718 | 0.729 | +1.24% | +1.87% |
| multiline | 20.455 | 21.098 | +3.34% | +2.94% |
| inline | 75.089 | 76.559 | +3.68% | +0.24% |
| code | 22.136 | 23.521 | +6.64% | +5.87% |
| escape-dense | 21.195 | 21.503 | +1.12% | +1.79% |
| tables/tables-plain | 39.887 | 41.409 | +3.92% | +3.71% |
| guard/one-table | 25.435 | 25.609 | +1.26% | +0.11% |
| mdx/short | 1.901 | 1.916 | +2.91% | -1.21% |
| mdx/tables-code | 255.197 | 259.859 | +1.78% | +1.88% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.247 | 29.697 | +1.18% | +1.90% |
| corpus/vue-docs/src/guide/extras/render-function.md | 67.911 | 69.637 | +1.64% | +3.46% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 103.491 | 102.897 | -1.20% | +0.05% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.720 | 27.637 | +1.79% | -2.34% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 43.298 | 43.660 | +4.04% | -2.23% |
| html/declaration-long-lines | 3.896 | 3.866 | -0.35% | -1.22% |
| fence/root-indented | 408.656 | 414.712 | +1.27% | +1.69% |
| fence/root-container | 487.859 | 459.052 | +0.77% | -11.76% |
| tables | 65.767 | 67.057 | +2.12% | +1.80% |
| guard/deep-emphasis | 62.105 | 61.777 | +1.78% | -2.82% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.547 | 2.529 | +0.69% | -2.12% |
| html/comment-long-lines | 3.823 | 3.762 | -0.84% | -2.36% |
| html/script-candidates | 56.904 | 57.296 | +0.56% | +0.82% |
| html/processing-long-lines | 3.899 | 3.886 | +1.21% | -1.84% |
| html/custom-tags | 341.100 | 350.510 | +4.75% | +0.90% |

### bulk-inline-8192 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.274 | 0.273 | -1.65% | +0.26% |
| direct-escape/ci-html-heavy | 11.842 | 13.085 | +10.54% | +10.44% |
| direct-escape/plain-128 | 0.027 | 0.026 | -2.75% | -3.02% |
| direct-escape/late-quote-128 | 0.088 | 0.089 | +1.59% | +0.82% |
| direct-escape/plain-512 | 0.044 | 0.042 | -6.27% | -6.15% |
| direct-escape/late-quote-512 | 0.127 | 0.112 | -11.90% | -11.81% |
| direct-escape/plain-1024 | 0.062 | 0.059 | -5.55% | -4.40% |
| direct-escape/late-quote-1024 | 0.139 | 0.127 | -8.16% | -8.67% |
| direct-escape/plain-4096 | 0.199 | 0.195 | -2.08% | -2.07% |
| direct-escape/late-quote-4096 | 0.239 | 0.239 | +0.77% | -0.15% |
| direct-escape/plain-8192 | 0.366 | 0.356 | -2.32% | -3.19% |
| direct-escape/late-quote-8192 | 0.389 | 0.378 | -2.85% | -3.16% |
| direct-escape/plain-65536 | 3.055 | 3.091 | +0.52% | +1.83% |
| direct-escape/late-quote-65536 | 3.101 | 3.083 | +0.84% | -1.95% |
| direct-escape/quotes-4096 | 26.062 | 26.485 | +1.63% | +1.61% |
| direct-escape/attr-quotes-4096 | 97.000 | 100.279 | +3.17% | +3.59% |
| direct-escape/quotes-65536 | 4526.602 | 107.614 | -97.63% | -97.62% |
| direct-escape/attr-quotes-65536 | 17815.504 | 386.933 | -97.83% | -97.82% |
| direct-escape/late-quote-reserved-512 | 0.047 | 0.046 | -2.26% | -2.02% |
| direct-escape/late-quote-reserved-8192 | 0.371 | 0.363 | -2.43% | -2.29% |
| direct-escape/late-quote-reserved-65536 | 3.121 | 3.058 | -1.59% | -2.48% |
| direct-escape/html-heavy-65536 | 157.480 | 123.888 | -20.58% | -22.07% |

### bulk-inline-8192 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 416.811 | 419.317 | -0.09% | +1.31% |
| ox-huge-tables | 9667.311 | 9699.625 | -0.97% | +1.67% |
| short | 0.724 | 0.725 | -1.09% | +1.32% |
| multiline | 21.231 | 21.253 | -3.69% | +4.20% |
| inline | 75.284 | 82.170 | +8.69% | +9.61% |
| code | 22.204 | 22.151 | +1.35% | -1.80% |
| escape-dense | 21.400 | 21.283 | +0.79% | -1.87% |
| tables/tables-plain | 39.835 | 39.765 | +0.50% | -0.84% |
| guard/one-table | 25.460 | 25.287 | +0.13% | -1.48% |
| mdx/short | 1.908 | 2.068 | +9.16% | +7.61% |
| mdx/tables-code | 257.058 | 254.532 | -0.79% | -1.18% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.552 | 30.091 | +1.21% | +2.45% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.983 | 69.678 | +0.66% | +1.36% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 103.367 | 109.263 | +4.87% | +6.54% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.462 | 27.571 | -0.05% | +0.84% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.608 | 42.827 | +0.44% | +0.58% |
| html/declaration-long-lines | 3.909 | 3.906 | +0.67% | -0.85% |
| fence/root-indented | 407.524 | 415.422 | +2.33% | +1.55% |
| fence/root-container | 454.453 | 458.501 | +0.83% | +0.95% |
| tables | 65.594 | 65.948 | +0.85% | +0.24% |
| guard/deep-emphasis | 63.359 | 69.939 | +10.02% | +10.75% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.589 | 2.586 | +0.03% | -0.26% |
| html/comment-long-lines | 3.786 | 3.799 | +1.40% | -0.67% |
| html/script-candidates | 57.023 | 57.024 | +0.20% | -0.20% |
| html/processing-long-lines | 3.890 | 3.907 | +0.25% | +0.62% |
| html/custom-tags | 329.810 | 333.337 | +1.04% | +1.10% |

### threshold-8192 / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.283 | 0.268 | -2.08% | -7.84% |
| direct-escape/ci-html-heavy | 11.971 | 11.926 | -1.08% | +0.33% |
| direct-escape/plain-128 | 0.027 | 0.027 | -2.10% | +2.53% |
| direct-escape/late-quote-128 | 0.098 | 0.091 | +4.48% | -16.04% |
| direct-escape/plain-512 | 0.045 | 0.045 | +1.38% | +2.81% |
| direct-escape/late-quote-512 | 0.129 | 0.110 | -13.71% | -15.95% |
| direct-escape/plain-1024 | 0.062 | 0.063 | +2.42% | +2.09% |
| direct-escape/late-quote-1024 | 0.145 | 0.131 | -6.26% | -12.83% |
| direct-escape/plain-4096 | 0.206 | 0.190 | -4.55% | -10.71% |
| direct-escape/late-quote-4096 | 0.248 | 0.230 | -4.35% | -10.16% |
| direct-escape/plain-8192 | 0.373 | 0.355 | -2.60% | -6.88% |
| direct-escape/late-quote-8192 | 0.399 | 0.378 | -3.44% | -6.98% |
| direct-escape/plain-65536 | 3.068 | 2.930 | -4.87% | -4.12% |
| direct-escape/late-quote-65536 | 3.106 | 2.937 | -4.95% | -5.94% |
| direct-escape/quotes-4096 | 26.068 | 26.126 | +0.45% | -0.01% |
| direct-escape/attr-quotes-4096 | 97.181 | 98.947 | +1.74% | +1.90% |
| direct-escape/quotes-65536 | 4564.678 | 237.275 | -94.80% | -94.81% |
| direct-escape/attr-quotes-65536 | 18071.053 | 951.942 | -94.79% | -94.67% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.049 | +0.47% | +3.52% |
| direct-escape/late-quote-reserved-8192 | 0.379 | 0.362 | -1.85% | -7.28% |
| direct-escape/late-quote-reserved-65536 | 3.065 | 2.912 | -4.83% | -5.17% |
| direct-escape/html-heavy-65536 | 156.266 | 173.910 | +11.55% | +11.03% |

### threshold-8192 / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 410.177 | 412.530 | +1.23% | -0.08% |
| ox-huge-tables | 9531.417 | 9529.552 | +0.82% | -0.86% |
| short | 0.697 | 0.712 | +2.53% | +1.94% |
| multiline | 20.266 | 21.213 | +4.57% | +4.77% |
| inline | 75.431 | 77.322 | +4.65% | +0.37% |
| code | 21.928 | 22.804 | +4.14% | +3.85% |
| escape-dense | 21.095 | 20.565 | -2.40% | -2.63% |
| tables/tables-plain | 39.738 | 38.899 | -2.33% | -1.89% |
| guard/one-table | 25.162 | 25.087 | +0.38% | -0.96% |
| mdx/short | 1.889 | 1.871 | -0.77% | -1.12% |
| mdx/tables-code | 255.276 | 254.743 | +0.33% | -0.75% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.177 | 29.314 | +0.46% | +0.48% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.694 | 69.186 | +0.23% | +1.21% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 104.748 | 103.233 | -2.17% | -0.72% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.370 | 26.800 | -1.74% | -2.43% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.378 | 41.231 | -1.78% | -3.62% |
| html/declaration-long-lines | 3.950 | 3.876 | +0.41% | -4.08% |
| fence/root-indented | 409.730 | 406.799 | -0.29% | -1.14% |
| fence/root-container | 454.713 | 453.145 | -0.66% | -0.03% |
| tables | 65.476 | 64.869 | -0.58% | -1.27% |
| guard/deep-emphasis | 61.405 | 61.005 | -0.49% | -0.81% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.523 | 2.508 | +0.20% | -1.37% |
| html/comment-long-lines | 3.825 | 3.794 | +0.48% | -2.04% |
| html/script-candidates | 56.865 | 58.270 | +2.80% | +2.14% |
| html/processing-long-lines | 3.861 | 3.890 | +2.16% | -0.68% |
| html/custom-tags | 332.263 | 325.957 | -0.89% | -2.89% |

## round-3-1

AMD EPYC 7763 64-Core Processor; rustc 1.98.1 (48a229cea 2026-09-01).

### aa-control / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.230 | 0.230 | +0.96% | -0.94% |
| direct-escape/ci-html-heavy | 11.819 | 11.782 | +0.11% | -0.73% |
| direct-escape/plain-128 | 0.027 | 0.026 | -1.82% | -0.82% |
| direct-escape/late-quote-128 | 0.071 | 0.070 | +0.11% | -1.19% |
| direct-escape/plain-512 | 0.045 | 0.045 | -0.20% | +0.20% |
| direct-escape/late-quote-512 | 0.083 | 0.084 | +1.12% | +1.26% |
| direct-escape/plain-1024 | 0.060 | 0.060 | -0.05% | +0.11% |
| direct-escape/late-quote-1024 | 0.114 | 0.114 | +0.64% | +0.41% |
| direct-escape/plain-4096 | 0.192 | 0.191 | +0.70% | -1.35% |
| direct-escape/late-quote-4096 | 0.220 | 0.221 | +0.26% | +0.32% |
| direct-escape/plain-8192 | 0.308 | 0.307 | -1.10% | -0.01% |
| direct-escape/late-quote-8192 | 0.332 | 0.331 | -0.81% | +0.20% |
| direct-escape/plain-65536 | 3.030 | 3.050 | +1.25% | +0.08% |
| direct-escape/late-quote-65536 | 3.099 | 3.155 | +2.70% | +0.96% |
| direct-escape/quotes-4096 | 26.081 | 26.022 | -0.35% | -0.10% |
| direct-escape/attr-quotes-4096 | 96.744 | 96.678 | -0.16% | +0.02% |
| direct-escape/quotes-65536 | 4538.645 | 4552.938 | +0.91% | -0.27% |
| direct-escape/attr-quotes-65536 | 17905.413 | 18002.865 | +0.07% | +1.02% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.048 | -0.74% | +0.15% |
| direct-escape/late-quote-reserved-8192 | 0.319 | 0.318 | -0.74% | +0.27% |
| direct-escape/late-quote-reserved-65536 | 3.171 | 3.152 | +0.20% | -1.39% |
| direct-escape/html-heavy-65536 | 156.505 | 156.067 | +0.14% | -0.70% |
| direct-escape/attr-plain-512 | 0.047 | 0.047 | +0.09% | -0.29% |
| direct-escape/attr-plain-8192 | 0.358 | 0.358 | -0.03% | -0.12% |
| direct-escape/attr-plain-65536 | 3.401 | 3.345 | -0.41% | -2.83% |

### aa-control / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 416.800 | 409.696 | -0.44% | -2.94% |
| ox-huge-tables | 9708.470 | 9667.278 | -0.11% | -0.73% |
| short | 0.718 | 0.695 | +0.69% | -7.05% |
| multiline | 20.542 | 20.598 | +0.45% | +0.10% |
| inline | 75.361 | 75.220 | +0.01% | -0.38% |
| code | 22.069 | 22.034 | +0.19% | -0.50% |
| escape-dense | 21.203 | 21.195 | +0.02% | -0.09% |
| tables/tables-plain | 39.597 | 40.005 | +0.95% | +1.11% |
| guard/one-table | 25.294 | 25.300 | +0.91% | -0.86% |
| mdx/short | 1.916 | 1.932 | +0.90% | +0.78% |
| mdx/tables-code | 256.504 | 254.688 | -0.06% | -1.35% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.884 | 29.413 | -0.56% | -2.61% |
| corpus/vue-docs/src/guide/extras/render-function.md | 70.009 | 69.576 | +0.39% | -1.65% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 105.198 | 104.525 | -0.44% | -0.84% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.580 | 27.282 | -0.28% | -1.87% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.553 | 42.458 | +0.15% | -0.59% |
| html/declaration-long-lines | 3.897 | 3.879 | +0.12% | -1.02% |
| fence/root-indented | 409.048 | 406.016 | -0.38% | -1.10% |
| fence/root-container | 453.193 | 450.182 | -0.25% | -1.07% |
| tables | 65.475 | 65.582 | +0.15% | +0.18% |
| guard/deep-emphasis | 61.356 | 63.338 | +3.29% | +3.17% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.543 | 2.509 | -0.83% | -1.91% |
| html/comment-long-lines | 3.834 | 3.823 | -1.04% | +0.50% |
| html/script-candidates | 56.882 | 56.955 | +0.03% | +0.23% |
| html/processing-long-lines | 3.938 | 3.898 | -0.88% | -1.14% |
| html/custom-tags | 328.123 | 329.648 | +0.64% | +0.29% |

### masked-direct / attribute-confirm

| Case | Before µs¹ | After µs¹ | attribute-confirm-1 | attribute-confirm-2 | attribute-confirm-3 |
|---|---:|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.644 | 0.683 | +4.94% | +7.32% | +6.04% |
| attribute-fence/double-quotes-128 | 1.107 | 1.105 | +3.85% | -1.19% | -0.25% |
| attribute-fence/single-quotes-128 | 1.090 | 1.088 | +2.64% | -1.44% | -0.13% |
| attribute-fence/mixed-quotes-128 | 1.251 | 1.271 | +2.85% | -0.04% | +1.66% |
| attribute-fence/plain-4096 | 4.245 | 5.454 | +28.50% | +28.16% | +28.50% |
| attribute-fence/double-quotes-4096 | 71.035 | 15.737 | -77.85% | -78.05% | -77.60% |
| attribute-fence/single-quotes-4096 | 70.194 | 15.628 | -77.74% | -77.96% | -77.63% |
| attribute-fence/mixed-quotes-4096 | 101.668 | 20.671 | -79.67% | -79.81% | -79.09% |
| attribute-fence/plain-65536 | 59.214 | 78.728 | +32.96% | +32.93% | +32.89% |
| attribute-fence/double-quotes-65536 | 12007.737 | 242.083 | -97.99% | -97.98% | -97.98% |
| attribute-fence/single-quotes-65536 | 11932.887 | 240.028 | -97.97% | -97.98% | -98.00% |
| attribute-fence/mixed-quotes-65536 | 18202.430 | 320.235 | -98.22% | -98.23% | -98.24% |
| attribute-fence/plain-1048576 | 988.638 | 1271.853 | +28.61% | +28.17% | +29.39% |
| attribute-fence/sparse-quotes-1048576 | 19115.837 | 1281.474 | -93.29% | -93.35% | -93.30% |

### masked-direct / confirm

| Case | Before µs¹ | After µs¹ | confirm-1 | confirm-2 | confirm-3 |
|---|---:|---:|---:|---:|---:|
| ox-large-cm | 409.635 | 410.629 | +0.55% | -0.05% | -0.16% |
| ox-large-tables | 443.550 | 441.541 | -0.21% | -0.22% | -0.53% |
| ox-huge-tables | 9551.598 | 9544.992 | +0.14% | -0.28% | -0.07% |
| short | 0.688 | 0.715 | +1.82% | +4.98% | +3.68% |
| short-reuse | 0.415 | 0.419 | +0.81% | +3.39% | +0.62% |
| plain | 11.282 | 10.285 | -8.82% | -9.04% | -8.94% |
| multiline | 20.512 | 22.020 | +8.39% | +5.46% | +7.37% |
| inline | 74.847 | 76.986 | +3.37% | +3.02% | +2.05% |
| lists | 77.920 | 75.964 | -0.94% | -2.39% | -2.63% |
| code | 22.068 | 21.850 | -1.35% | +1.05% | -1.13% |
| tables | 64.876 | 64.523 | +1.06% | -1.41% | -0.93% |
| headings | 61.715 | 62.181 | +0.95% | -0.63% | +1.50% |
| unique-headings | 51.357 | 52.287 | +1.81% | +1.79% | +2.36% |
| long-prose | 34.368 | 32.447 | -1.32% | -5.44% | -5.71% |
| late-marker | 41.420 | 41.135 | -0.67% | -0.69% | -1.07% |
| long-code | 34.306 | 34.052 | -0.63% | -0.74% | -1.89% |
| escape-dense | 21.274 | 20.896 | -1.78% | -0.57% | -2.61% |
| commonmark/tiny | 0.640 | 0.661 | +2.20% | +5.08% | +2.61% |
| commonmark/short-100b | 1.250 | 1.274 | +2.54% | +1.31% | +0.64% |
| gfm_overlap/tiny | 0.641 | 0.661 | +2.16% | +5.43% | +2.35% |
| gfm_overlap/short-100b | 1.250 | 1.277 | +2.34% | +1.23% | +1.39% |
| tables/tables-plain | 39.247 | 39.474 | +3.43% | +0.05% | -0.51% |
| tables/tables-commonmark-inline | 54.793 | 53.228 | -1.74% | -1.33% | -3.89% |
| tables/tables-links | 65.223 | 66.889 | +3.38% | +2.86% | +0.94% |
| commonmark/publication-5k | 41.853 | 42.356 | +0.72% | -1.44% | +2.06% |
| guard/plain | 39.266 | 39.576 | +2.44% | +0.73% | -0.39% |
| guard/mixed | 53.963 | 53.396 | +0.45% | -1.05% | -2.38% |
| guard/links | 65.338 | 67.051 | +3.29% | +2.95% | +1.07% |
| guard/mixed-document | 32.412 | 32.142 | -0.84% | -1.06% | +0.47% |
| guard/one-table | 25.014 | 24.693 | -1.08% | -0.99% | -1.33% |
| guard/long-cells | 37.103 | 36.259 | -2.27% | -2.37% | -2.60% |
| guard/wide9 | 52.100 | 50.918 | -2.31% | -1.97% | -2.06% |
| guard/wide16 | 85.166 | 83.977 | -1.40% | -1.23% | -1.83% |
| guard/escapes-code | 125.292 | 123.456 | +0.20% | -0.37% | -8.54% |
| guard/prose | 16.123 | 15.086 | -6.49% | -6.43% | -6.95% |
| guard/links-prose | 63.641 | 64.406 | +1.69% | +2.79% | +0.49% |
| guard/edge-cases | 2065.613 | 2018.150 | -1.29% | -2.15% | -2.71% |
| guard/references | 21.027 | 20.534 | -1.92% | -0.05% | -2.34% |
| guard/empty | 0.191 | 0.201 | +2.00% | +14.98% | +6.34% |
| guard/gfm-document | 41.950 | 42.683 | -0.26% | -0.51% | +2.17% |
| guard/long-delimiters | 18.965 | 13.887 | -26.81% | -26.41% | -26.78% |
| guard/deep-emphasis | 63.000 | 62.588 | -0.71% | -0.77% | -0.42% |
| mdx/short | 1.874 | 1.961 | +2.86% | +1.12% | +5.85% |
| mdx/segments | 258.980 | 260.339 | -0.81% | +1.23% | +0.52% |
| mdx/tables-code | 252.586 | 253.207 | +0.28% | -0.24% | -0.31% |
| corpus/vue-docs/src/api/application.md | 97.962 | 97.434 | -0.61% | +0.54% | -0.29% |
| corpus/vue-docs/src/api/built-in-directives.md | 105.824 | 104.630 | +0.30% | -0.53% | -1.13% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.088 | 28.933 | -1.86% | -2.22% | +0.87% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.464 | 67.022 | -4.06% | -2.47% | -1.31% |
| corpus/vite-docs/docs/config/shared-options.md | 132.439 | 133.644 | +0.64% | +0.54% | +0.92% |
| corpus/vite-docs/docs/guide/api-plugin.md | 126.985 | 128.513 | +0.16% | +0.39% | +1.20% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 163.381 | 162.182 | +0.19% | +1.27% | -0.73% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 105.652 | 103.120 | -1.13% | -0.25% | -2.55% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 124.530 | 123.842 | -1.10% | +0.06% | -0.55% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.273 | 27.197 | -3.02% | -0.50% | +0.10% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.283 | 43.489 | -1.07% | +3.54% | +2.44% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 153.912 | 151.477 | -1.58% | -1.61% | -2.79% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 133.458 | 131.712 | +0.09% | -0.58% | -1.31% |
| corpus/concat/vue-docs/matched | 4288.930 | 4243.475 | -1.11% | +0.97% | -1.18% |
| corpus/concat/vue-docs/upstream | 4807.835 | 4734.409 | -1.62% | +0.78% | -1.22% |
| corpus/concat/vite-docs/matched | 2228.505 | 2227.342 | -0.05% | +0.70% | -0.97% |
| corpus/concat/vite-docs/upstream | 2686.391 | 2681.852 | -0.17% | +0.21% | -1.38% |
| corpus/concat/rust-book/matched | 6506.880 | 6528.565 | +0.25% | +0.64% | +0.25% |
| corpus/concat/rust-book/upstream | 7130.278 | 7162.571 | +0.40% | +0.43% | +1.51% |
| corpus/concat/typescript-handbook/matched | 7878.797 | 7730.893 | -1.75% | -1.88% | -1.28% |
| corpus/concat/typescript-handbook/upstream | 8790.200 | 8624.670 | -1.88% | -1.41% | -1.94% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.522 | 2.565 | +1.26% | +0.07% | +2.91% |
| corpus/ox-parser/LARGE_MD/1 | 13.771 | 13.665 | -1.44% | -1.27% | +0.03% |
| corpus/ox-parser/LARGE_MD/100 | 1257.896 | 1246.460 | -1.16% | -0.91% | -0.80% |
| html/root-short-lines | 41.446 | 47.754 | +14.99% | +15.16% | +15.41% |
| html/root-long-lines | 3.527 | 3.523 | +2.79% | +5.40% | -0.37% |
| html/separate-blocks | 280.303 | 287.174 | -3.81% | +2.21% | +2.45% |
| html/comment-long-lines | 3.838 | 3.837 | -2.52% | +0.06% | +0.82% |
| html/script-long-lines | 4.006 | 3.966 | +4.53% | -0.99% | -2.09% |
| html/script-candidates | 56.971 | 61.342 | +7.26% | +7.76% | +7.64% |
| html/comment-short-lines | 161.456 | 150.177 | -6.24% | -5.69% | -12.14% |
| html/processing-long-lines | 3.924 | 4.072 | +22.73% | +3.78% | -0.53% |
| html/cdata-long-lines | 3.921 | 3.942 | +1.39% | +0.98% | +0.53% |
| html/declaration-long-lines | 3.901 | 3.952 | +2.76% | +0.05% | +2.57% |
| html/container-html | 142.852 | 147.933 | +3.53% | +3.60% | +3.46% |
| html/custom-tags | 330.065 | 349.458 | -0.48% | +5.98% | +6.45% |
| fence/root-short | 344.296 | 188.739 | -45.01% | -45.22% | -44.77% |
| fence/root-long | 5.545 | 5.050 | -8.96% | -7.59% | -8.93% |
| fence/root-indented | 405.820 | 400.533 | -1.50% | -1.08% | -1.32% |
| fence/root-tabs | 315.915 | 170.676 | -46.10% | -45.91% | -46.00% |
| fence/root-container | 451.032 | 446.482 | -1.10% | -0.69% | -1.00% |
| fence/root-crlf | 317.766 | 172.511 | -45.70% | -45.69% | -45.75% |
| utf8/paragraph-0 | 14.507 | 12.699 | -11.44% | -12.63% | -12.49% |
| utf8/paragraph-32768 | 13.633 | 12.385 | -8.97% | -9.16% | -9.37% |
| utf8/paragraph-65536 | 12.745 | 12.118 | -3.45% | -5.22% | -4.62% |
| utf8/html-0 | 5.844 | 4.668 | +0.54% | -20.20% | -20.13% |
| utf8/html-32768 | 4.984 | 4.358 | -11.22% | -12.85% | -12.10% |
| utf8/html-65536 | 4.059 | 4.082 | +2.00% | +2.54% | +0.56% |

### masked-direct / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 | direct-3 |
|---|---:|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.230 | 0.192 | -16.98% | -16.49% | -16.44% |
| direct-escape/ci-html-heavy | 11.814 | 5.483 | -54.58% | -53.30% | -53.90% |
| direct-escape/plain-128 | 0.027 | 0.026 | -1.11% | -1.40% | -2.12% |
| direct-escape/late-quote-128 | 0.071 | 0.070 | -0.18% | -1.12% | -0.81% |
| direct-escape/plain-512 | 0.045 | 0.032 | -27.62% | -27.12% | -28.36% |
| direct-escape/late-quote-512 | 0.083 | 0.066 | -21.30% | -21.24% | -21.16% |
| direct-escape/plain-1024 | 0.060 | 0.046 | -23.13% | -23.15% | -23.42% |
| direct-escape/late-quote-1024 | 0.114 | 0.091 | -21.32% | -19.74% | -21.79% |
| direct-escape/plain-4096 | 0.192 | 0.163 | -15.00% | -15.39% | -15.01% |
| direct-escape/late-quote-4096 | 0.232 | 0.184 | -34.64% | -16.69% | +4.93% |
| direct-escape/plain-8192 | 0.309 | 0.258 | -16.46% | -15.89% | -16.52% |
| direct-escape/late-quote-8192 | 0.331 | 0.278 | -16.02% | -15.99% | -13.90% |
| direct-escape/plain-65536 | 3.043 | 2.546 | -15.23% | -16.62% | -16.71% |
| direct-escape/late-quote-65536 | 3.113 | 2.617 | -17.99% | -14.89% | -15.92% |
| direct-escape/quotes-4096 | 26.003 | 4.188 | -84.16% | -83.77% | -83.99% |
| direct-escape/attr-quotes-4096 | 97.088 | 14.273 | -85.22% | -85.79% | -85.28% |
| direct-escape/quotes-65536 | 4549.802 | 65.955 | -98.55% | -98.54% | -98.55% |
| direct-escape/attr-quotes-65536 | 17964.619 | 227.887 | -98.73% | -98.73% | -98.74% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.035 | -27.57% | -25.68% | -27.10% |
| direct-escape/late-quote-reserved-8192 | 0.318 | 0.259 | -18.50% | -18.34% | -18.73% |
| direct-escape/late-quote-reserved-65536 | 3.172 | 2.653 | -16.28% | -16.85% | -16.27% |
| direct-escape/html-heavy-65536 | 157.590 | 70.241 | -55.06% | -55.27% | -55.82% |
| direct-escape/attr-plain-512 | 0.047 | 0.036 | -22.67% | -13.67% | -22.88% |
| direct-escape/attr-plain-8192 | 0.358 | 0.297 | -16.92% | -16.90% | -16.80% |
| direct-escape/attr-plain-65536 | 3.352 | 2.772 | -16.32% | -17.68% | -17.39% |

### masked-direct / regime-confirm

| Case | Before µs¹ | After µs¹ | regime-confirm-1 | regime-confirm-2 | regime-confirm-3 |
|---|---:|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.371 | 0.371 | -0.35% | -0.11% | +4.80% |
| escape-regime/code-plain-128 | 0.408 | 0.425 | +12.20% | +4.14% | +2.27% |
| escape-regime/code-late-quote-128 | 0.432 | 0.449 | +9.45% | +4.96% | +0.30% |
| escape-regime/code-quotes-128 | 0.677 | 0.662 | +2.11% | -0.69% | -3.97% |
| escape-regime/sparse-amp-128 | 0.368 | 0.372 | +0.90% | +0.93% | +2.35% |
| escape-regime/plain-512 | 0.446 | 0.426 | -4.16% | -3.87% | -4.91% |
| escape-regime/code-plain-512 | 0.441 | 0.447 | +8.50% | +1.57% | -0.35% |
| escape-regime/code-late-quote-512 | 0.464 | 0.474 | +8.60% | +3.07% | -0.63% |
| escape-regime/code-quotes-512 | 1.910 | 1.280 | -31.52% | -32.45% | -33.19% |
| escape-regime/sparse-amp-512 | 0.771 | 0.735 | -4.69% | -4.00% | -4.77% |
| escape-regime/plain-4096 | 1.096 | 1.068 | -2.36% | -3.30% | -2.55% |
| escape-regime/code-plain-4096 | 0.644 | 0.649 | +4.90% | +1.17% | -3.20% |
| escape-regime/code-late-quote-4096 | 0.668 | 0.670 | +1.72% | +1.55% | -3.12% |
| escape-regime/code-quotes-4096 | 28.910 | 6.852 | -75.88% | -76.39% | -76.27% |
| escape-regime/sparse-amp-4096 | 3.774 | 4.159 | +10.74% | +9.65% | +10.40% |
| escape-regime/plain-65536 | 12.642 | 12.086 | -4.29% | -4.40% | -4.47% |
| escape-regime/code-plain-65536 | 5.015 | 4.469 | -10.21% | -10.37% | -11.76% |
| escape-regime/code-late-quote-65536 | 5.020 | 4.547 | -8.54% | -9.63% | -10.62% |
| escape-regime/code-quotes-65536 | 4492.123 | 103.143 | -97.69% | -97.72% | -97.70% |
| escape-regime/sparse-amp-65536 | 53.437 | 60.475 | +12.99% | +13.50% | +13.48% |
| escape-regime/plain-1048576 | 215.297 | 202.444 | -5.30% | +0.32% | -5.97% |
| escape-regime/code-plain-1048576 | 90.422 | 78.149 | -12.80% | +1.32% | -13.76% |
| escape-regime/code-late-quote-1048576 | 90.331 | 78.616 | -12.31% | +1.42% | -13.38% |
| escape-regime/code-quotes-1048576 | 1183347.946 | 1691.260 | -99.86% | -99.86% | -99.86% |
| escape-regime/sparse-amp-1048576 | 875.616 | 1002.663 | +13.94% | +14.47% | +14.55% |
| escape-regime/plain-8388608 | 1896.229 | 1767.612 | -4.70% | -6.78% | -9.90% |
| escape-regime/code-plain-8388608 | 876.061 | 641.732 | -10.63% | -14.61% | -28.22% |

### masked-twin / attribute-confirm

| Case | Before µs¹ | After µs¹ | attribute-confirm-1 | attribute-confirm-2 | attribute-confirm-3 |
|---|---:|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.643 | 0.693 | +6.11% | +8.01% | +8.10% |
| attribute-fence/double-quotes-128 | 1.100 | 1.128 | +1.38% | +2.55% | +1.92% |
| attribute-fence/single-quotes-128 | 1.079 | 1.097 | -0.04% | +1.69% | +2.64% |
| attribute-fence/mixed-quotes-128 | 1.243 | 1.270 | +0.89% | +2.57% | +2.37% |
| attribute-fence/plain-4096 | 4.290 | 5.528 | +29.33% | +26.78% | +30.07% |
| attribute-fence/double-quotes-4096 | 70.858 | 21.898 | -69.60% | -69.10% | -69.57% |
| attribute-fence/single-quotes-4096 | 70.351 | 21.198 | -69.33% | -69.90% | -69.66% |
| attribute-fence/mixed-quotes-4096 | 101.448 | 27.085 | -73.40% | -72.76% | -73.40% |
| attribute-fence/plain-65536 | 59.142 | 79.625 | +34.63% | +34.48% | +34.42% |
| attribute-fence/double-quotes-65536 | 12011.165 | 336.779 | -97.20% | -97.17% | -97.19% |
| attribute-fence/single-quotes-65536 | 12018.633 | 335.563 | -97.27% | -97.21% | -97.17% |
| attribute-fence/mixed-quotes-65536 | 18047.682 | 426.393 | -97.65% | -97.65% | -97.64% |
| attribute-fence/plain-1048576 | 992.432 | 1305.771 | +31.03% | +31.89% | +31.64% |
| attribute-fence/sparse-quotes-1048576 | 19242.791 | 1314.485 | -93.13% | -93.15% | -93.17% |

### masked-twin / confirm

| Case | Before µs¹ | After µs¹ | confirm-1 | confirm-2 | confirm-3 |
|---|---:|---:|---:|---:|---:|
| ox-large-cm | 414.798 | 401.360 | -3.79% | -3.24% | -0.94% |
| ox-large-tables | 446.820 | 435.944 | -2.95% | -2.75% | -1.27% |
| ox-huge-tables | 9586.874 | 9401.636 | -2.34% | -2.91% | -1.05% |
| short | 0.691 | 0.728 | +6.09% | +4.59% | +4.48% |
| short-reuse | 0.411 | 0.410 | -0.24% | -1.25% | +2.00% |
| plain | 11.268 | 10.358 | -7.74% | -8.07% | -11.08% |
| multiline | 20.380 | 20.570 | +0.82% | +1.25% | +0.25% |
| inline | 75.567 | 75.582 | +1.20% | +0.02% | +2.52% |
| lists | 77.425 | 75.131 | -1.13% | -3.97% | -2.96% |
| code | 21.875 | 21.657 | -0.77% | +0.00% | -2.33% |
| tables | 64.987 | 64.366 | +4.25% | -0.97% | -1.37% |
| headings | 61.908 | 62.004 | +0.98% | -0.05% | +0.15% |
| unique-headings | 51.453 | 51.922 | +1.37% | +0.73% | +1.06% |
| long-prose | 33.046 | 33.526 | +2.80% | +0.33% | -1.04% |
| late-marker | 41.536 | 41.603 | +0.13% | +0.10% | +0.16% |
| long-code | 34.358 | 34.647 | +2.03% | -0.24% | +0.84% |
| escape-dense | 21.062 | 20.675 | -1.57% | -0.18% | -2.04% |
| commonmark/tiny | 0.648 | 0.683 | +6.49% | +5.07% | +2.72% |
| commonmark/short-100b | 1.262 | 1.287 | +0.34% | +2.13% | -3.34% |
| gfm_overlap/tiny | 0.644 | 0.682 | +6.38% | +5.64% | +2.72% |
| gfm_overlap/short-100b | 1.267 | 1.287 | +0.21% | +2.06% | -2.79% |
| tables/tables-plain | 39.599 | 39.040 | +5.45% | -1.02% | -2.39% |
| tables/tables-commonmark-inline | 54.756 | 51.892 | -0.75% | -5.22% | -5.81% |
| tables/tables-links | 65.837 | 66.616 | +6.03% | +0.59% | +1.18% |
| commonmark/publication-5k | 42.288 | 42.077 | -2.16% | -1.33% | +1.55% |
| guard/plain | 39.666 | 39.372 | +0.70% | +0.00% | -2.87% |
| guard/mixed | 54.683 | 52.211 | -0.15% | -4.08% | -6.70% |
| guard/links | 65.866 | 67.009 | +5.61% | +1.03% | +1.37% |
| guard/mixed-document | 32.313 | 31.800 | -1.57% | -3.33% | -1.50% |
| guard/one-table | 25.158 | 24.765 | -1.06% | -0.66% | -2.09% |
| guard/long-cells | 37.105 | 37.431 | +0.26% | +1.38% | +0.88% |
| guard/wide9 | 52.246 | 51.229 | -2.35% | -1.53% | -2.13% |
| guard/wide16 | 85.836 | 83.508 | -2.66% | -2.46% | -2.71% |
| guard/escapes-code | 125.098 | 125.428 | +1.74% | -1.47% | +0.14% |
| guard/prose | 15.983 | 15.014 | -6.30% | -5.20% | -6.66% |
| guard/links-prose | 63.586 | 63.373 | +0.44% | -1.54% | +0.55% |
| guard/edge-cases | 2067.696 | 2026.110 | -0.93% | -2.01% | -1.38% |
| guard/references | 20.952 | 20.893 | +1.51% | -0.53% | -0.96% |
| guard/empty | 0.192 | 0.193 | -0.50% | +1.54% | +0.58% |
| guard/gfm-document | 42.159 | 41.945 | -1.92% | -0.68% | +2.01% |
| guard/long-delimiters | 19.044 | 16.493 | -13.49% | -13.36% | -13.32% |
| guard/deep-emphasis | 63.419 | 60.694 | -1.50% | -0.10% | -4.43% |
| mdx/short | 1.941 | 1.906 | -0.14% | -1.28% | -4.75% |
| mdx/segments | 260.886 | 258.690 | -0.43% | -0.90% | -0.86% |
| mdx/tables-code | 253.393 | 253.471 | +3.47% | +0.03% | -1.31% |
| corpus/vue-docs/src/api/application.md | 95.688 | 96.516 | +1.73% | -1.54% | -0.35% |
| corpus/vue-docs/src/api/built-in-directives.md | 104.831 | 104.828 | +0.42% | -2.05% | -0.51% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.290 | 28.975 | -0.58% | -2.44% | -0.42% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.535 | 67.083 | -2.57% | -1.75% | -1.61% |
| corpus/vite-docs/docs/config/shared-options.md | 131.955 | 132.307 | +0.27% | -1.30% | +1.10% |
| corpus/vite-docs/docs/guide/api-plugin.md | 126.998 | 126.741 | -0.20% | -1.64% | -0.08% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 161.587 | 160.794 | -0.15% | -1.91% | -0.80% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 103.617 | 103.212 | +0.03% | -0.57% | -0.61% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 122.578 | 121.971 | -0.48% | -3.07% | -0.26% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.412 | 26.612 | -1.94% | -2.68% | -3.15% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.479 | 41.412 | -2.12% | -2.32% | -2.80% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 155.443 | 152.356 | -1.98% | -3.51% | -2.63% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 132.688 | 132.515 | -0.65% | -0.10% | -0.29% |
| corpus/concat/vue-docs/matched | 4288.851 | 4266.773 | -0.80% | -0.46% | +0.03% |
| corpus/concat/vue-docs/upstream | 4799.650 | 4781.347 | -0.81% | -0.51% | -0.02% |
| corpus/concat/vite-docs/matched | 2230.312 | 2208.519 | -1.28% | -1.05% | -0.00% |
| corpus/concat/vite-docs/upstream | 2691.084 | 2663.100 | -1.36% | -1.04% | +0.12% |
| corpus/concat/rust-book/matched | 6458.545 | 6437.964 | -1.66% | -0.13% | -0.06% |
| corpus/concat/rust-book/upstream | 7112.766 | 7060.797 | -1.60% | -0.57% | -0.06% |
| corpus/concat/typescript-handbook/matched | 7816.803 | 7738.651 | -2.39% | -0.83% | -0.88% |
| corpus/concat/typescript-handbook/upstream | 8750.042 | 8643.805 | -1.51% | -1.14% | -1.07% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.553 | 2.485 | -0.90% | -1.48% | -4.36% |
| corpus/ox-parser/LARGE_MD/1 | 13.797 | 13.494 | -1.18% | +6.02% | -2.81% |
| corpus/ox-parser/LARGE_MD/100 | 1264.228 | 1239.075 | -2.10% | -2.29% | -1.89% |
| html/root-short-lines | 41.256 | 43.974 | +6.59% | +7.63% | +6.72% |
| html/root-long-lines | 3.432 | 3.501 | +1.99% | +1.85% | +1.69% |
| html/separate-blocks | 280.745 | 247.810 | -11.48% | -11.89% | -11.57% |
| html/comment-long-lines | 3.786 | 3.860 | +1.48% | +0.28% | +2.03% |
| html/script-long-lines | 3.903 | 3.925 | +0.55% | -1.55% | +1.23% |
| html/script-candidates | 56.951 | 57.960 | +1.77% | +1.91% | +1.86% |
| html/comment-short-lines | 164.382 | 112.084 | -31.81% | -29.81% | -33.75% |
| html/processing-long-lines | 3.875 | 3.895 | +0.16% | +0.17% | +1.39% |
| html/cdata-long-lines | 3.881 | 3.923 | +1.33% | -0.16% | +1.08% |
| html/declaration-long-lines | 3.862 | 3.886 | +1.60% | +1.20% | -0.04% |
| html/container-html | 142.316 | 146.481 | +3.09% | +2.83% | +2.86% |
| html/custom-tags | 329.433 | 326.132 | -1.41% | -0.96% | -0.32% |
| fence/root-short | 344.116 | 265.841 | -22.75% | -22.67% | -24.23% |
| fence/root-long | 5.533 | 5.646 | +3.21% | +1.31% | +2.46% |
| fence/root-indented | 405.716 | 402.157 | -1.68% | -1.26% | -0.63% |
| fence/root-tabs | 315.706 | 244.295 | -22.55% | -22.64% | -24.24% |
| fence/root-container | 450.349 | 447.492 | -1.01% | -0.69% | -0.63% |
| fence/root-crlf | 317.446 | 246.071 | -22.48% | -22.52% | -24.17% |
| utf8/paragraph-0 | 14.484 | 13.321 | -8.03% | -8.20% | -7.61% |
| utf8/paragraph-32768 | 13.600 | 13.059 | -3.78% | -4.27% | -4.42% |
| utf8/paragraph-65536 | 12.718 | 12.740 | +0.56% | -0.02% | +0.39% |
| utf8/html-0 | 5.821 | 4.630 | -20.39% | -21.57% | -20.51% |
| utf8/html-32768 | 4.967 | 4.354 | -12.21% | -14.17% | -12.40% |
| utf8/html-65536 | 4.000 | 4.068 | +1.70% | -0.56% | +0.26% |

### masked-twin / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 | direct-3 |
|---|---:|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.228 | 0.230 | +0.56% | +0.75% | +0.41% |
| direct-escape/ci-html-heavy | 11.831 | 8.816 | -25.56% | -25.27% | -25.69% |
| direct-escape/plain-128 | 0.027 | 0.027 | +0.07% | +0.46% | -0.18% |
| direct-escape/late-quote-128 | 0.070 | 0.071 | +0.27% | -0.22% | -0.03% |
| direct-escape/plain-512 | 0.045 | 0.041 | -9.40% | -8.40% | -9.42% |
| direct-escape/late-quote-512 | 0.084 | 0.076 | -8.80% | -8.68% | -9.03% |
| direct-escape/plain-1024 | 0.060 | 0.057 | -5.88% | -4.60% | -4.85% |
| direct-escape/late-quote-1024 | 0.114 | 0.101 | -11.64% | -11.07% | -10.71% |
| direct-escape/plain-4096 | 0.192 | 0.190 | -0.95% | -0.84% | -1.26% |
| direct-escape/late-quote-4096 | 0.221 | 0.213 | +1.87% | -23.55% | -3.40% |
| direct-escape/plain-8192 | 0.308 | 0.309 | -0.20% | +0.23% | +0.55% |
| direct-escape/late-quote-8192 | 0.332 | 0.327 | -1.17% | -1.62% | -0.96% |
| direct-escape/plain-65536 | 3.044 | 3.224 | +4.93% | +5.75% | +6.40% |
| direct-escape/late-quote-65536 | 3.114 | 3.221 | +3.44% | +2.97% | +4.60% |
| direct-escape/quotes-4096 | 26.068 | 6.292 | -76.04% | -75.68% | -75.86% |
| direct-escape/attr-quotes-4096 | 96.954 | 21.083 | -78.69% | -78.06% | -78.19% |
| direct-escape/quotes-65536 | 4555.292 | 100.715 | -97.82% | -97.77% | -97.79% |
| direct-escape/attr-quotes-65536 | 18092.297 | 337.498 | -98.13% | -98.14% | -98.15% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.046 | -5.01% | -4.40% | -2.91% |
| direct-escape/late-quote-reserved-8192 | 0.317 | 0.309 | -2.58% | -2.74% | -2.74% |
| direct-escape/late-quote-reserved-65536 | 3.183 | 3.297 | +3.59% | +4.68% | +3.08% |
| direct-escape/html-heavy-65536 | 155.873 | 116.036 | -25.38% | -24.84% | -25.86% |
| direct-escape/attr-plain-512 | 0.047 | 0.048 | +3.41% | +5.69% | +3.46% |
| direct-escape/attr-plain-8192 | 0.358 | 0.401 | +12.27% | +13.37% | +12.20% |
| direct-escape/attr-plain-65536 | 3.378 | 3.572 | +5.76% | +6.10% | +5.21% |

### masked-twin / regime-confirm

| Case | Before µs¹ | After µs¹ | regime-confirm-1 | regime-confirm-2 | regime-confirm-3 |
|---|---:|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.371 | 0.387 | +9.73% | +4.23% | +4.89% |
| escape-regime/code-plain-128 | 0.410 | 0.429 | +11.11% | +4.56% | -2.99% |
| escape-regime/code-late-quote-128 | 0.432 | 0.447 | +11.69% | +3.33% | -1.37% |
| escape-regime/code-quotes-128 | 0.667 | 0.676 | +6.74% | +1.40% | -2.91% |
| escape-regime/sparse-amp-128 | 0.371 | 0.377 | +1.29% | +2.90% | -6.14% |
| escape-regime/plain-512 | 0.448 | 0.467 | +9.99% | +3.82% | +3.27% |
| escape-regime/code-plain-512 | 0.443 | 0.459 | +9.99% | +3.64% | -2.16% |
| escape-regime/code-late-quote-512 | 0.465 | 0.478 | +10.41% | +2.84% | -0.80% |
| escape-regime/code-quotes-512 | 1.907 | 1.508 | -19.07% | -20.92% | -21.22% |
| escape-regime/sparse-amp-512 | 0.777 | 0.770 | +3.24% | +0.02% | -2.63% |
| escape-regime/plain-4096 | 1.097 | 1.117 | +4.28% | +1.84% | +1.43% |
| escape-regime/code-plain-4096 | 0.639 | 0.655 | +6.67% | +2.53% | +0.06% |
| escape-regime/code-late-quote-4096 | 0.666 | 0.665 | +5.47% | -0.13% | -1.50% |
| escape-regime/code-quotes-4096 | 28.991 | 9.162 | -68.36% | -68.30% | -68.59% |
| escape-regime/sparse-amp-4096 | 3.773 | 3.687 | +2.29% | -2.26% | -3.01% |
| escape-regime/plain-65536 | 12.652 | 12.810 | +1.78% | +1.09% | +1.25% |
| escape-regime/code-plain-65536 | 4.975 | 5.169 | +4.70% | +3.14% | +3.89% |
| escape-regime/code-late-quote-65536 | 4.992 | 5.200 | +4.47% | +4.58% | +2.85% |
| escape-regime/code-quotes-65536 | 4474.107 | 137.264 | -96.93% | -96.94% | -96.93% |
| escape-regime/sparse-amp-65536 | 53.454 | 51.935 | -3.10% | -2.55% | -2.84% |
| escape-regime/plain-1048576 | 215.714 | 217.541 | +7.52% | +0.97% | -6.23% |
| escape-regime/code-plain-1048576 | 90.308 | 92.241 | +20.38% | +2.03% | -14.11% |
| escape-regime/code-late-quote-1048576 | 90.273 | 93.371 | +18.94% | +1.93% | -12.93% |
| escape-regime/code-quotes-1048576 | 1214915.504 | 2252.442 | -99.81% | -99.82% | -99.81% |
| escape-regime/sparse-amp-1048576 | 873.071 | 850.961 | -2.18% | -2.53% | -2.70% |
| escape-regime/plain-8388608 | 1717.876 | 1863.415 | +9.57% | +2.65% | +1.13% |
| escape-regime/code-plain-8388608 | 690.795 | 817.047 | +24.87% | +3.86% | +7.12% |

## round-3-2

AMD EPYC 7763 64-Core Processor; rustc 1.98.1 (48a229cea 2026-09-01).

### aa-control / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.228 | 0.230 | -0.33% | +1.51% |
| direct-escape/ci-html-heavy | 11.910 | 11.913 | -0.55% | +0.60% |
| direct-escape/plain-128 | 0.027 | 0.027 | -0.77% | -0.21% |
| direct-escape/late-quote-128 | 0.071 | 0.071 | -0.90% | +0.86% |
| direct-escape/plain-512 | 0.045 | 0.045 | -1.14% | -0.23% |
| direct-escape/late-quote-512 | 0.084 | 0.084 | -0.56% | +0.10% |
| direct-escape/plain-1024 | 0.060 | 0.060 | -1.58% | +1.20% |
| direct-escape/late-quote-1024 | 0.115 | 0.114 | -1.02% | -0.48% |
| direct-escape/plain-4096 | 0.192 | 0.193 | +0.16% | +0.94% |
| direct-escape/late-quote-4096 | 0.250 | 0.221 | -0.31% | -20.43% |
| direct-escape/plain-8192 | 0.308 | 0.308 | -0.09% | -0.14% |
| direct-escape/late-quote-8192 | 0.331 | 0.331 | +0.11% | -0.11% |
| direct-escape/plain-65536 | 3.048 | 3.098 | +1.46% | +1.77% |
| direct-escape/late-quote-65536 | 3.096 | 3.090 | +0.45% | -0.81% |
| direct-escape/quotes-4096 | 26.096 | 26.244 | +0.85% | +0.29% |
| direct-escape/attr-quotes-4096 | 97.285 | 97.187 | -0.23% | +0.03% |
| direct-escape/quotes-65536 | 4578.956 | 4565.342 | -0.27% | -0.32% |
| direct-escape/attr-quotes-65536 | 17985.072 | 17919.225 | -1.17% | +0.44% |
| direct-escape/late-quote-reserved-512 | 0.049 | 0.049 | +1.94% | -0.10% |
| direct-escape/late-quote-reserved-8192 | 0.321 | 0.318 | -1.21% | -0.49% |
| direct-escape/late-quote-reserved-65536 | 3.184 | 3.176 | -0.19% | -0.33% |
| direct-escape/html-heavy-65536 | 157.539 | 157.017 | -1.63% | +0.99% |
| direct-escape/attr-plain-512 | 0.047 | 0.047 | -0.22% | -0.15% |
| direct-escape/attr-plain-8192 | 0.358 | 0.358 | -0.01% | +0.13% |
| direct-escape/attr-plain-65536 | 3.336 | 3.326 | -0.51% | -0.12% |

### aa-control / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 412.022 | 410.366 | -0.24% | -0.56% |
| ox-huge-tables | 9629.901 | 9604.308 | -0.07% | -0.46% |
| short | 0.701 | 0.697 | -1.86% | +0.63% |
| multiline | 20.528 | 20.850 | +4.40% | -1.18% |
| inline | 75.242 | 75.033 | -0.26% | -0.30% |
| code | 22.250 | 22.137 | -0.78% | -0.23% |
| escape-dense | 21.256 | 21.313 | +0.95% | -0.41% |
| tables/tables-plain | 40.035 | 39.725 | -0.52% | -1.03% |
| guard/one-table | 25.496 | 25.326 | -0.36% | -0.97% |
| mdx/short | 1.937 | 1.902 | -0.45% | -3.13% |
| mdx/tables-code | 258.388 | 256.568 | -0.90% | -0.51% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.402 | 29.299 | +0.76% | -1.43% |
| corpus/vue-docs/src/guide/extras/render-function.md | 66.779 | 68.355 | +2.93% | +1.79% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 101.773 | 103.626 | +2.33% | +1.31% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.502 | 28.446 | +2.83% | +4.05% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.426 | 44.163 | +4.13% | +4.06% |
| html/declaration-long-lines | 3.869 | 3.903 | +1.19% | +0.55% |
| fence/root-indented | 409.580 | 409.947 | -0.03% | +0.21% |
| fence/root-container | 452.941 | 453.055 | +0.23% | -0.18% |
| tables | 65.851 | 65.588 | +0.65% | -1.44% |
| guard/deep-emphasis | 61.549 | 61.564 | -0.32% | +0.36% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.502 | 2.492 | +0.38% | -1.22% |
| html/comment-long-lines | 3.800 | 3.802 | +0.58% | -0.51% |
| html/script-candidates | 56.979 | 56.861 | -0.12% | -0.30% |
| html/processing-long-lines | 3.921 | 3.910 | -1.08% | +0.53% |
| html/custom-tags | 328.145 | 353.968 | +7.19% | +8.55% |

### masked-direct / attribute-confirm

| Case | Before µs¹ | After µs¹ | attribute-confirm-1 | attribute-confirm-2 | attribute-confirm-3 |
|---|---:|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.652 | 0.685 | +5.20% | +3.95% | +6.29% |
| attribute-fence/double-quotes-128 | 1.124 | 1.101 | -1.76% | -2.71% | -2.19% |
| attribute-fence/single-quotes-128 | 1.095 | 1.070 | -2.52% | -1.85% | -2.78% |
| attribute-fence/mixed-quotes-128 | 1.255 | 1.251 | -0.54% | +0.53% | -0.35% |
| attribute-fence/plain-4096 | 4.322 | 5.511 | +26.84% | +27.51% | +27.82% |
| attribute-fence/double-quotes-4096 | 71.145 | 15.685 | -78.11% | -77.27% | -77.98% |
| attribute-fence/single-quotes-4096 | 70.485 | 15.574 | -78.13% | -77.69% | -77.90% |
| attribute-fence/mixed-quotes-4096 | 102.016 | 20.593 | -79.89% | -79.57% | -79.86% |
| attribute-fence/plain-65536 | 59.931 | 79.298 | +32.15% | +32.68% | +32.33% |
| attribute-fence/double-quotes-65536 | 11970.299 | 243.510 | -97.94% | -97.98% | -97.97% |
| attribute-fence/single-quotes-65536 | 12061.722 | 242.021 | -97.99% | -98.00% | -97.98% |
| attribute-fence/mixed-quotes-65536 | 18174.434 | 322.186 | -98.23% | -98.22% | -98.23% |
| attribute-fence/plain-1048576 | 994.789 | 1292.880 | +29.97% | +29.78% | +29.77% |
| attribute-fence/sparse-quotes-1048576 | 19225.957 | 1284.932 | -93.27% | -93.23% | -93.36% |

### masked-direct / confirm

| Case | Before µs¹ | After µs¹ | confirm-1 | confirm-2 | confirm-3 |
|---|---:|---:|---:|---:|---:|
| ox-large-cm | 414.036 | 412.049 | -0.48% | -4.47% | +0.34% |
| ox-large-tables | 449.215 | 447.822 | -0.60% | -2.87% | +0.99% |
| ox-huge-tables | 9752.482 | 9750.588 | +0.69% | -2.68% | +1.19% |
| short | 0.699 | 0.718 | +2.52% | +2.72% | +2.17% |
| short-reuse | 0.420 | 0.428 | -1.31% | +2.41% | +2.65% |
| plain | 11.365 | 10.288 | -8.84% | -10.47% | -8.19% |
| multiline | 20.535 | 22.113 | +10.18% | +5.05% | +7.68% |
| inline | 75.112 | 77.648 | +3.66% | +1.22% | +3.42% |
| lists | 77.732 | 76.230 | -1.74% | -2.39% | -0.02% |
| code | 22.073 | 22.041 | -1.21% | +3.99% | +0.57% |
| tables | 65.482 | 64.299 | -0.86% | -3.74% | -1.24% |
| headings | 62.346 | 62.662 | +2.07% | -0.26% | +0.46% |
| unique-headings | 52.011 | 52.436 | +2.23% | +0.82% | +1.62% |
| long-prose | 34.588 | 32.505 | -8.09% | -3.85% | -5.71% |
| late-marker | 41.613 | 41.263 | -0.47% | -1.64% | -0.84% |
| long-code | 34.900 | 34.487 | +0.30% | -1.93% | -1.48% |
| escape-dense | 21.305 | 21.225 | -0.38% | +2.84% | +0.00% |
| commonmark/tiny | 0.656 | 0.664 | +2.68% | +1.11% | +1.64% |
| commonmark/short-100b | 1.285 | 1.279 | +1.79% | -0.45% | +2.88% |
| gfm_overlap/tiny | 0.657 | 0.662 | +1.74% | +0.82% | +1.28% |
| gfm_overlap/short-100b | 1.276 | 1.279 | +1.90% | -0.94% | +3.79% |
| tables/tables-plain | 39.705 | 39.525 | +0.60% | -1.60% | -1.05% |
| tables/tables-commonmark-inline | 54.542 | 53.261 | -2.90% | -2.90% | -2.35% |
| tables/tables-links | 65.826 | 66.669 | +1.61% | -0.16% | +1.28% |
| commonmark/publication-5k | 42.261 | 42.364 | -0.02% | -3.27% | +1.81% |
| guard/plain | 39.705 | 39.608 | +0.34% | -1.20% | -0.24% |
| guard/mixed | 54.557 | 53.109 | -2.00% | -2.37% | -2.65% |
| guard/links | 66.066 | 67.036 | +1.89% | +1.82% | +1.08% |
| guard/mixed-document | 32.474 | 32.308 | -0.87% | -2.48% | +0.95% |
| guard/one-table | 25.092 | 24.881 | +0.11% | -1.45% | -0.03% |
| guard/long-cells | 37.641 | 36.213 | -2.39% | -4.36% | -2.16% |
| guard/wide9 | 52.764 | 51.124 | -2.00% | -3.11% | -2.70% |
| guard/wide16 | 86.030 | 83.823 | -1.72% | -2.97% | -1.68% |
| guard/escapes-code | 125.879 | 125.083 | +0.21% | -5.51% | -0.61% |
| guard/prose | 16.096 | 15.105 | -5.53% | -7.95% | -4.54% |
| guard/links-prose | 64.186 | 65.178 | +1.43% | +0.57% | +2.82% |
| guard/edge-cases | 2061.790 | 2042.474 | -1.04% | -1.14% | -0.40% |
| guard/references | 21.579 | 20.929 | -4.74% | -3.01% | +0.63% |
| guard/empty | 0.192 | 0.209 | +3.33% | +3.63% | +11.26% |
| guard/gfm-document | 42.446 | 42.497 | -0.10% | -3.29% | +0.81% |
| guard/long-delimiters | 19.281 | 15.087 | -22.92% | -21.75% | -20.62% |
| guard/deep-emphasis | 63.046 | 62.943 | -3.46% | -0.28% | +2.26% |
| mdx/short | 1.942 | 1.953 | -2.34% | +1.61% | +2.50% |
| mdx/segments | 263.011 | 260.854 | +0.50% | -1.06% | -0.03% |
| mdx/tables-code | 253.734 | 255.005 | +1.38% | +0.50% | +0.35% |
| corpus/vue-docs/src/api/application.md | 97.241 | 97.925 | -0.81% | +0.45% | +0.70% |
| corpus/vue-docs/src/api/built-in-directives.md | 106.033 | 105.357 | -1.30% | -3.32% | -0.64% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.744 | 28.827 | -2.91% | -3.23% | -3.08% |
| corpus/vue-docs/src/guide/extras/render-function.md | 69.215 | 67.467 | -3.33% | -3.74% | -2.53% |
| corpus/vite-docs/docs/config/shared-options.md | 134.295 | 133.646 | +1.70% | -2.12% | +0.43% |
| corpus/vite-docs/docs/guide/api-plugin.md | 128.303 | 128.363 | -0.24% | -2.19% | +0.31% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 165.353 | 163.132 | +0.48% | +0.13% | -1.34% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 105.081 | 104.055 | +0.66% | +0.21% | -0.98% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 125.457 | 124.388 | -0.93% | -0.96% | -0.85% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.264 | 27.188 | -0.16% | -1.27% | -0.28% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.568 | 43.695 | +2.45% | +1.95% | +2.65% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 156.021 | 151.517 | -2.86% | -4.82% | -2.89% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 134.763 | 131.900 | -0.87% | -1.76% | -2.12% |
| corpus/concat/vue-docs/matched | 4276.126 | 4273.883 | -0.46% | -2.12% | -0.01% |
| corpus/concat/vue-docs/upstream | 4805.756 | 4774.891 | -0.77% | -1.12% | -0.12% |
| corpus/concat/vite-docs/matched | 2237.885 | 2227.951 | -0.27% | -1.52% | -0.50% |
| corpus/concat/vite-docs/upstream | 2700.287 | 2680.193 | -0.43% | -0.97% | -0.33% |
| corpus/concat/rust-book/matched | 6488.728 | 6538.565 | +0.54% | -0.12% | +0.96% |
| corpus/concat/rust-book/upstream | 7182.832 | 7192.531 | +0.91% | -0.35% | +0.95% |
| corpus/concat/typescript-handbook/matched | 7966.904 | 7761.267 | -2.46% | -4.07% | -1.57% |
| corpus/concat/typescript-handbook/upstream | 8850.688 | 8683.777 | -1.66% | -3.23% | -1.30% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.578 | 2.580 | -0.91% | -0.07% | +1.09% |
| corpus/ox-parser/LARGE_MD/1 | 13.863 | 13.735 | -0.92% | -1.17% | -0.01% |
| corpus/ox-parser/LARGE_MD/100 | 1266.312 | 1251.475 | -1.25% | -1.91% | -0.67% |
| html/root-short-lines | 41.394 | 48.087 | +27.49% | +15.59% | +16.09% |
| html/root-long-lines | 3.510 | 3.498 | -2.54% | +0.77% | +3.68% |
| html/separate-blocks | 280.403 | 283.504 | +1.16% | +0.43% | +0.90% |
| html/comment-long-lines | 3.835 | 3.791 | -0.88% | -1.16% | +0.92% |
| html/script-long-lines | 4.011 | 3.904 | -4.72% | -2.33% | -1.83% |
| html/script-candidates | 57.001 | 61.157 | +7.15% | +7.29% | +7.29% |
| html/comment-short-lines | 161.123 | 155.599 | -4.32% | -6.15% | -3.12% |
| html/processing-long-lines | 3.939 | 3.895 | -0.33% | -1.91% | -0.59% |
| html/cdata-long-lines | 3.962 | 3.931 | -2.51% | +0.29% | -0.93% |
| html/declaration-long-lines | 3.935 | 3.887 | -1.23% | -0.03% | -2.04% |
| html/container-html | 142.234 | 148.228 | +4.11% | +4.35% | +4.02% |
| html/custom-tags | 330.888 | 345.195 | +4.62% | +4.03% | +5.82% |
| fence/root-short | 343.434 | 189.419 | -45.22% | -44.81% | -44.42% |
| fence/root-long | 5.543 | 5.036 | +366.53% | -9.78% | -8.87% |
| fence/root-indented | 408.707 | 403.038 | -1.39% | -1.89% | -1.40% |
| fence/root-tabs | 316.087 | 170.634 | -46.07% | -46.62% | -45.53% |
| fence/root-container | 452.213 | 448.574 | -0.71% | -0.97% | -1.09% |
| fence/root-crlf | 317.714 | 173.107 | -45.83% | -45.51% | -45.20% |
| utf8/paragraph-0 | 14.523 | 12.716 | -12.70% | -12.47% | -12.19% |
| utf8/paragraph-32768 | 13.697 | 12.407 | -9.81% | -9.23% | -9.34% |
| utf8/paragraph-65536 | 12.750 | 12.095 | -5.56% | -4.80% | -5.30% |
| utf8/html-0 | 5.881 | 4.713 | -20.60% | -19.71% | -19.53% |
| utf8/html-32768 | 5.014 | 4.379 | -13.12% | -12.81% | -12.51% |
| utf8/html-65536 | 4.114 | 4.070 | -1.34% | +3.33% | +0.04% |

### masked-direct / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 | direct-3 |
|---|---:|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.230 | 0.193 | -15.88% | -16.78% | -16.66% |
| direct-escape/ci-html-heavy | 12.050 | 5.467 | -54.39% | -55.11% | -54.42% |
| direct-escape/plain-128 | 0.027 | 0.026 | -2.46% | -1.10% | -1.39% |
| direct-escape/late-quote-128 | 0.072 | 0.071 | -0.41% | -2.14% | -2.00% |
| direct-escape/plain-512 | 0.045 | 0.032 | -28.26% | -28.06% | -29.08% |
| direct-escape/late-quote-512 | 0.084 | 0.066 | -21.65% | -21.54% | -22.31% |
| direct-escape/plain-1024 | 0.060 | 0.046 | -23.01% | -23.26% | -23.28% |
| direct-escape/late-quote-1024 | 0.115 | 0.091 | -22.54% | -22.15% | -19.26% |
| direct-escape/plain-4096 | 0.195 | 0.163 | -14.99% | -16.42% | -15.77% |
| direct-escape/late-quote-4096 | 0.223 | 0.183 | -17.61% | -17.97% | -21.54% |
| direct-escape/plain-8192 | 0.311 | 0.259 | -15.28% | -16.83% | -16.84% |
| direct-escape/late-quote-8192 | 0.334 | 0.279 | -16.41% | -16.12% | -17.41% |
| direct-escape/plain-65536 | 3.063 | 2.562 | -15.63% | -16.37% | -17.22% |
| direct-escape/late-quote-65536 | 3.122 | 2.612 | -15.80% | -16.02% | -16.80% |
| direct-escape/quotes-4096 | 26.337 | 4.219 | -84.38% | -84.35% | -83.93% |
| direct-escape/attr-quotes-4096 | 97.724 | 14.292 | -85.34% | -86.28% | -85.36% |
| direct-escape/quotes-65536 | 4608.024 | 66.254 | -98.55% | -98.55% | -98.58% |
| direct-escape/attr-quotes-65536 | 18221.443 | 228.763 | -98.73% | -98.75% | -98.74% |
| direct-escape/late-quote-reserved-512 | 0.049 | 0.035 | -28.42% | -27.84% | -25.77% |
| direct-escape/late-quote-reserved-8192 | 0.321 | 0.260 | -18.93% | -18.66% | -19.13% |
| direct-escape/late-quote-reserved-65536 | 3.183 | 2.687 | -15.51% | -15.95% | -16.26% |
| direct-escape/html-heavy-65536 | 158.127 | 70.665 | -55.06% | -55.80% | -55.46% |
| direct-escape/attr-plain-512 | 0.048 | 0.036 | -23.18% | -23.91% | -22.13% |
| direct-escape/attr-plain-8192 | 0.362 | 0.299 | -17.36% | -17.81% | -17.40% |
| direct-escape/attr-plain-65536 | 3.377 | 2.785 | -17.49% | -18.56% | -17.26% |

### masked-direct / regime-confirm

| Case | Before µs¹ | After µs¹ | regime-confirm-1 | regime-confirm-2 | regime-confirm-3 |
|---|---:|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.370 | 0.373 | -0.23% | +0.41% | +0.68% |
| escape-regime/code-plain-128 | 0.422 | 0.424 | -3.43% | +2.15% | +3.01% |
| escape-regime/code-late-quote-128 | 0.438 | 0.440 | -2.87% | +1.99% | +3.77% |
| escape-regime/code-quotes-128 | 0.670 | 0.655 | -6.82% | -2.23% | -1.35% |
| escape-regime/sparse-amp-128 | 0.369 | 0.372 | -0.05% | +1.72% | +0.80% |
| escape-regime/plain-512 | 0.450 | 0.431 | -7.97% | -4.20% | -3.66% |
| escape-regime/code-plain-512 | 0.453 | 0.449 | -1.59% | +2.01% | -1.64% |
| escape-regime/code-late-quote-512 | 0.463 | 0.473 | -0.03% | +2.25% | +0.68% |
| escape-regime/code-quotes-512 | 1.903 | 1.288 | -34.16% | -31.98% | -23.00% |
| escape-regime/sparse-amp-512 | 0.776 | 0.749 | -5.59% | -1.82% | -2.87% |
| escape-regime/plain-4096 | 1.103 | 1.070 | -4.67% | -2.97% | -1.07% |
| escape-regime/code-plain-4096 | 0.652 | 0.638 | -4.13% | -2.27% | +1.07% |
| escape-regime/code-late-quote-4096 | 0.675 | 0.656 | -4.47% | -3.71% | +0.90% |
| escape-regime/code-quotes-4096 | 28.981 | 6.796 | -76.43% | -76.19% | -76.76% |
| escape-regime/sparse-amp-4096 | 3.777 | 4.196 | +10.57% | +10.92% | +12.19% |
| escape-regime/plain-65536 | 12.680 | 12.098 | -4.51% | -4.99% | -4.45% |
| escape-regime/code-plain-65536 | 5.032 | 4.478 | -13.06% | -10.03% | -11.30% |
| escape-regime/code-late-quote-65536 | 5.018 | 4.549 | -9.48% | -9.35% | -3.81% |
| escape-regime/code-quotes-65536 | 4458.766 | 102.714 | -97.71% | -97.67% | -97.70% |
| escape-regime/sparse-amp-65536 | 53.436 | 60.983 | +13.77% | +14.57% | +14.12% |
| escape-regime/plain-1048576 | 215.451 | 202.971 | -5.80% | -0.50% | -5.79% |
| escape-regime/code-plain-1048576 | 89.731 | 77.467 | -13.92% | -1.66% | -13.67% |
| escape-regime/code-late-quote-1048576 | 90.111 | 77.791 | -13.48% | +0.98% | -13.67% |
| escape-regime/code-quotes-1048576 | 1185547.791 | 1706.270 | -99.86% | -99.86% | -99.86% |
| escape-regime/sparse-amp-1048576 | 876.569 | 999.484 | +14.02% | +14.43% | +14.08% |
| escape-regime/plain-8388608 | 1962.054 | 1871.937 | -4.88% | -6.06% | -4.59% |
| escape-regime/code-plain-8388608 | 766.559 | 681.024 | -9.86% | -13.45% | -11.16% |

### masked-twin / attribute-confirm

| Case | Before µs¹ | After µs¹ | attribute-confirm-1 | attribute-confirm-2 | attribute-confirm-3 |
|---|---:|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.647 | 0.690 | +5.97% | +10.04% | +6.25% |
| attribute-fence/double-quotes-128 | 1.120 | 1.130 | -0.13% | +2.30% | +0.09% |
| attribute-fence/single-quotes-128 | 1.095 | 1.098 | -1.57% | +0.67% | +0.10% |
| attribute-fence/mixed-quotes-128 | 1.268 | 1.269 | -0.70% | +1.42% | -0.33% |
| attribute-fence/plain-4096 | 4.301 | 5.573 | +28.95% | +30.89% | +30.23% |
| attribute-fence/double-quotes-4096 | 71.425 | 22.076 | -69.61% | -69.56% | -68.79% |
| attribute-fence/single-quotes-4096 | 70.474 | 21.274 | -69.81% | -70.18% | -69.66% |
| attribute-fence/mixed-quotes-4096 | 101.869 | 27.313 | -73.19% | -73.19% | -73.29% |
| attribute-fence/plain-65536 | 60.038 | 80.432 | +33.86% | +34.05% | +33.97% |
| attribute-fence/double-quotes-65536 | 11984.925 | 342.836 | -97.16% | -97.19% | -97.06% |
| attribute-fence/single-quotes-65536 | 11981.555 | 333.776 | -97.21% | -97.28% | -97.17% |
| attribute-fence/mixed-quotes-65536 | 18154.439 | 427.667 | -97.64% | -97.65% | -97.64% |
| attribute-fence/plain-1048576 | 996.622 | 1313.500 | +32.04% | +31.84% | +30.49% |
| attribute-fence/sparse-quotes-1048576 | 19214.315 | 1323.050 | -93.11% | -93.22% | -93.04% |

### masked-twin / confirm

| Case | Before µs¹ | After µs¹ | confirm-1 | confirm-2 | confirm-3 |
|---|---:|---:|---:|---:|---:|
| ox-large-cm | 410.542 | 409.406 | -0.60% | +5.45% | -0.86% |
| ox-large-tables | 444.771 | 440.182 | -1.03% | +4.40% | -1.62% |
| ox-huge-tables | 9695.189 | 9622.606 | -1.16% | +4.44% | -1.92% |
| short | 0.694 | 0.733 | +5.50% | +5.70% | +4.22% |
| short-reuse | 0.423 | 0.412 | -2.85% | -0.96% | -2.11% |
| plain | 11.277 | 10.353 | -8.19% | +9.99% | -9.07% |
| multiline | 20.491 | 20.527 | +1.18% | -1.21% | +0.18% |
| inline | 75.263 | 77.277 | +2.75% | +2.61% | +1.95% |
| lists | 77.886 | 76.554 | -1.71% | +8.34% | -3.94% |
| code | 22.021 | 21.692 | -1.50% | -0.53% | -3.04% |
| tables | 65.482 | 64.714 | -1.17% | -1.44% | -0.89% |
| headings | 62.283 | 62.019 | -1.28% | +2.35% | -0.58% |
| unique-headings | 51.959 | 51.778 | -0.46% | +1.76% | -0.69% |
| long-prose | 34.351 | 34.559 | +2.74% | +0.17% | +4.36% |
| late-marker | 41.639 | 41.504 | +0.29% | -1.02% | -0.57% |
| long-code | 34.768 | 34.900 | +0.51% | +0.38% | +0.15% |
| escape-dense | 21.143 | 20.681 | -1.12% | -2.35% | -2.25% |
| commonmark/tiny | 0.652 | 0.684 | +4.91% | +7.31% | +2.59% |
| commonmark/short-100b | 1.274 | 1.269 | -0.34% | -1.07% | +0.15% |
| gfm_overlap/tiny | 0.651 | 0.681 | +4.61% | +6.66% | +3.34% |
| gfm_overlap/short-100b | 1.275 | 1.272 | -0.28% | -1.03% | -0.02% |
| tables/tables-plain | 39.653 | 39.697 | +0.47% | -1.74% | -2.58% |
| tables/tables-commonmark-inline | 54.515 | 52.425 | -3.01% | -4.89% | -5.30% |
| tables/tables-links | 65.738 | 66.959 | +2.92% | +1.37% | -0.58% |
| commonmark/publication-5k | 42.054 | 42.715 | +2.17% | +5.84% | +1.19% |
| guard/plain | 39.837 | 39.209 | -0.61% | -2.49% | -2.03% |
| guard/mixed | 54.225 | 51.876 | -4.00% | -4.33% | -4.93% |
| guard/links | 66.187 | 67.213 | +2.75% | +1.55% | +2.77% |
| guard/mixed-document | 32.469 | 32.001 | -1.00% | +1.92% | -2.45% |
| guard/one-table | 25.283 | 24.891 | -0.67% | -1.55% | -2.09% |
| guard/long-cells | 37.503 | 37.584 | +0.99% | -0.56% | +0.22% |
| guard/wide9 | 52.572 | 51.353 | -2.45% | -1.84% | -1.09% |
| guard/wide16 | 85.717 | 83.627 | -2.47% | -2.44% | -1.47% |
| guard/escapes-code | 124.620 | 126.500 | +2.62% | +1.84% | +0.79% |
| guard/prose | 16.144 | 14.997 | -5.47% | +12.75% | -14.57% |
| guard/links-prose | 63.307 | 63.681 | +0.90% | +0.91% | +0.19% |
| guard/edge-cases | 2075.690 | 2042.614 | -0.93% | -1.02% | -2.13% |
| guard/references | 20.960 | 20.960 | +0.83% | +1.45% | -0.84% |
| guard/empty | 0.191 | 0.196 | +3.84% | +2.56% | +2.36% |
| guard/gfm-document | 41.936 | 42.955 | +1.78% | +6.60% | +0.42% |
| guard/long-delimiters | 18.996 | 17.393 | -12.42% | -8.41% | -9.09% |
| guard/deep-emphasis | 63.193 | 61.356 | -1.05% | -2.91% | -1.77% |
| mdx/short | 1.910 | 1.893 | -0.55% | -3.68% | -0.33% |
| mdx/segments | 261.924 | 262.437 | -0.90% | +5.65% | -1.55% |
| mdx/tables-code | 254.608 | 255.272 | +0.24% | +0.26% | +0.75% |
| corpus/vue-docs/src/api/application.md | 97.642 | 98.162 | -0.18% | +5.75% | -1.33% |
| corpus/vue-docs/src/api/built-in-directives.md | 105.932 | 104.664 | -1.20% | +0.53% | -2.69% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.491 | 29.106 | -0.62% | +0.39% | -1.42% |
| corpus/vue-docs/src/guide/extras/render-function.md | 69.502 | 67.218 | -3.29% | -0.51% | -4.27% |
| corpus/vite-docs/docs/config/shared-options.md | 133.394 | 132.610 | -0.29% | +2.30% | -8.19% |
| corpus/vite-docs/docs/guide/api-plugin.md | 128.345 | 128.132 | -0.17% | +1.27% | -0.63% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 163.223 | 161.878 | -0.82% | +1.33% | -1.78% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 105.130 | 103.613 | -2.00% | -0.70% | -2.13% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 124.671 | 123.052 | +1.11% | -1.87% | -2.17% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 28.245 | 26.846 | -4.95% | -1.66% | -5.25% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 44.622 | 41.474 | -6.43% | -3.34% | -7.05% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 156.665 | 153.983 | +0.61% | -1.67% | -2.93% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 133.385 | 133.165 | +1.53% | -1.25% | -1.12% |
| corpus/concat/vue-docs/matched | 4302.689 | 4305.301 | -0.67% | +1.74% | +0.39% |
| corpus/concat/vue-docs/upstream | 4811.549 | 4824.446 | -0.99% | +1.70% | +0.75% |
| corpus/concat/vite-docs/matched | 2225.596 | 2262.777 | +0.67% | +1.67% | -0.26% |
| corpus/concat/vite-docs/upstream | 2690.483 | 2710.257 | -0.16% | +0.73% | +0.04% |
| corpus/concat/rust-book/matched | 6486.743 | 6502.012 | +1.82% | +0.24% | -0.82% |
| corpus/concat/rust-book/upstream | 7132.403 | 7125.949 | +1.31% | +0.16% | -1.05% |
| corpus/concat/typescript-handbook/matched | 7927.619 | 7834.512 | -1.88% | +0.27% | -2.15% |
| corpus/concat/typescript-handbook/upstream | 8896.730 | 8746.590 | -1.90% | -0.19% | -1.18% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.567 | 2.549 | -1.91% | +4.14% | -1.29% |
| corpus/ox-parser/LARGE_MD/1 | 13.756 | 13.477 | -2.06% | +3.53% | -2.33% |
| corpus/ox-parser/LARGE_MD/100 | 1267.961 | 1243.591 | -2.13% | +4.47% | -2.23% |
| html/root-short-lines | 41.390 | 44.128 | +9.28% | +6.16% | +6.37% |
| html/root-long-lines | 3.549 | 3.467 | -5.57% | -0.47% | -85.75% |
| html/separate-blocks | 299.729 | 249.162 | -16.87% | -11.28% | -17.26% |
| html/comment-long-lines | 3.847 | 3.828 | +0.26% | -0.21% | -1.79% |
| html/script-long-lines | 3.979 | 3.927 | -3.72% | -4.38% | -0.43% |
| html/script-candidates | 57.378 | 58.384 | +2.37% | +1.75% | -0.30% |
| html/comment-short-lines | 160.589 | 112.194 | -30.14% | -29.56% | -32.29% |
| html/processing-long-lines | 3.923 | 3.880 | -2.40% | -2.07% | -1.10% |
| html/cdata-long-lines | 4.013 | 3.913 | -4.80% | -2.50% | +0.20% |
| html/declaration-long-lines | 3.969 | 3.917 | -4.73% | -1.13% | +0.64% |
| html/container-html | 142.613 | 147.177 | +3.16% | +3.47% | +3.80% |
| html/custom-tags | 353.907 | 326.992 | -7.61% | -0.50% | -8.04% |
| fence/root-short | 343.566 | 267.070 | -22.27% | -20.19% | -22.84% |
| fence/root-long | 5.493 | 5.734 | +4.46% | +3.64% | +4.05% |
| fence/root-indented | 409.011 | 402.680 | -0.89% | -1.79% | -1.14% |
| fence/root-tabs | 317.481 | 245.497 | -22.22% | -20.64% | -23.00% |
| fence/root-container | 453.843 | 449.209 | -0.75% | -1.02% | -13.63% |
| fence/root-crlf | 317.949 | 247.175 | -22.26% | -21.16% | -22.61% |
| utf8/paragraph-0 | 14.512 | 13.428 | -7.47% | -8.25% | -7.70% |
| utf8/paragraph-32768 | 13.659 | 13.098 | -4.32% | -5.39% | -4.06% |
| utf8/paragraph-65536 | 12.789 | 12.780 | -0.65% | -0.07% | +0.83% |
| utf8/html-0 | 5.878 | 4.625 | -22.81% | -22.09% | -21.16% |
| utf8/html-32768 | 4.998 | 4.330 | -14.77% | -13.82% | -13.32% |
| utf8/html-65536 | 4.056 | 4.030 | -1.91% | +0.31% | -0.28% |

### masked-twin / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 | direct-3 |
|---|---:|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.230 | 0.231 | +0.47% | +0.07% | +0.24% |
| direct-escape/ci-html-heavy | 12.004 | 8.858 | -28.87% | -25.67% | -26.21% |
| direct-escape/plain-128 | 0.027 | 0.027 | +0.81% | +0.74% | -0.22% |
| direct-escape/late-quote-128 | 0.071 | 0.071 | +0.84% | -0.57% | -1.37% |
| direct-escape/plain-512 | 0.045 | 0.041 | -8.84% | -8.78% | -8.78% |
| direct-escape/late-quote-512 | 0.083 | 0.076 | -8.52% | -8.28% | -8.45% |
| direct-escape/plain-1024 | 0.060 | 0.057 | -5.31% | -6.25% | -3.54% |
| direct-escape/late-quote-1024 | 0.114 | 0.102 | -11.64% | -11.87% | -11.10% |
| direct-escape/plain-4096 | 0.192 | 0.190 | -1.04% | -0.75% | -0.93% |
| direct-escape/late-quote-4096 | 0.220 | 0.270 | -3.52% | +22.15% | +22.86% |
| direct-escape/plain-8192 | 0.307 | 0.308 | +0.42% | -0.28% | +0.16% |
| direct-escape/late-quote-8192 | 0.331 | 0.327 | -1.10% | -1.08% | -0.83% |
| direct-escape/plain-65536 | 3.046 | 3.261 | +5.80% | +7.41% | +6.55% |
| direct-escape/late-quote-65536 | 3.116 | 3.247 | +2.97% | +7.76% | +5.22% |
| direct-escape/quotes-4096 | 26.062 | 6.292 | -76.52% | -75.87% | -75.86% |
| direct-escape/attr-quotes-4096 | 97.739 | 21.029 | -78.42% | -78.25% | -79.37% |
| direct-escape/quotes-65536 | 4550.581 | 101.288 | -97.77% | -97.78% | -97.78% |
| direct-escape/attr-quotes-65536 | 17917.170 | 334.421 | -98.11% | -98.13% | -98.14% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.046 | -5.98% | -3.90% | -5.95% |
| direct-escape/late-quote-reserved-8192 | 0.321 | 0.309 | -3.48% | -2.92% | -4.05% |
| direct-escape/late-quote-reserved-65536 | 3.190 | 3.301 | +3.35% | +4.89% | +3.22% |
| direct-escape/html-heavy-65536 | 157.017 | 116.084 | -28.84% | -25.61% | -26.07% |
| direct-escape/attr-plain-512 | 0.047 | 0.048 | +1.25% | +2.49% | +2.56% |
| direct-escape/attr-plain-8192 | 0.362 | 0.402 | +11.06% | +12.58% | +10.89% |
| direct-escape/attr-plain-65536 | 3.359 | 3.578 | +6.62% | +7.47% | +6.53% |

### masked-twin / regime-confirm

| Case | Before µs¹ | After µs¹ | regime-confirm-1 | regime-confirm-2 | regime-confirm-3 |
|---|---:|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.366 | 0.385 | +1.79% | +5.37% | +5.09% |
| escape-regime/code-plain-128 | 0.415 | 0.426 | +2.85% | +6.13% | +1.62% |
| escape-regime/code-late-quote-128 | 0.433 | 0.438 | +1.16% | +3.94% | -0.50% |
| escape-regime/code-quotes-128 | 0.670 | 0.666 | +2.03% | +1.01% | -1.07% |
| escape-regime/sparse-amp-128 | 0.368 | 0.375 | +0.12% | +1.94% | +3.06% |
| escape-regime/plain-512 | 0.452 | 0.464 | +0.95% | +2.75% | +5.21% |
| escape-regime/code-plain-512 | 0.447 | 0.456 | +1.75% | +5.12% | +1.14% |
| escape-regime/code-late-quote-512 | 0.469 | 0.473 | +0.74% | +3.48% | +0.30% |
| escape-regime/code-quotes-512 | 1.908 | 1.519 | -19.89% | -20.81% | -20.05% |
| escape-regime/sparse-amp-512 | 0.773 | 0.763 | -2.20% | -1.23% | -1.25% |
| escape-regime/plain-4096 | 1.106 | 1.114 | +0.76% | +0.47% | +1.54% |
| escape-regime/code-plain-4096 | 0.639 | 0.652 | +2.12% | +1.98% | +1.02% |
| escape-regime/code-late-quote-4096 | 0.665 | 0.662 | -0.10% | +0.18% | -1.22% |
| escape-regime/code-quotes-4096 | 28.930 | 9.238 | -67.85% | -68.04% | -68.51% |
| escape-regime/sparse-amp-4096 | 3.763 | 3.647 | -0.19% | -3.06% | -3.06% |
| escape-regime/plain-65536 | 12.639 | 12.810 | +1.31% | +1.35% | +1.38% |
| escape-regime/code-plain-65536 | 4.974 | 5.148 | +4.30% | +4.11% | +3.32% |
| escape-regime/code-late-quote-65536 | 5.006 | 5.210 | +4.00% | +4.25% | +4.83% |
| escape-regime/code-quotes-65536 | 4492.491 | 138.036 | -96.93% | -96.94% | -96.89% |
| escape-regime/sparse-amp-65536 | 53.521 | 52.076 | -2.71% | -3.35% | -2.45% |
| escape-regime/plain-1048576 | 232.858 | 234.479 | +7.89% | +1.03% | +0.70% |
| escape-regime/code-plain-1048576 | 107.789 | 110.210 | +22.55% | +1.23% | +1.97% |
| escape-regime/code-late-quote-1048576 | 90.504 | 109.569 | +21.20% | +1.78% | +0.85% |
| escape-regime/code-quotes-1048576 | 1194707.039 | 2270.813 | -99.81% | -99.81% | -99.81% |
| escape-regime/sparse-amp-1048576 | 876.866 | 856.041 | -3.28% | -2.47% | -2.24% |
| escape-regime/plain-8388608 | 1950.733 | 1965.656 | +7.24% | +0.48% | -1.18% |
| escape-regime/code-plain-8388608 | 766.705 | 915.457 | +22.51% | +13.44% | -2.82% |

## round-4-1

AMD EPYC 7763 64-Core Processor; rustc 1.98.1 (48a229cea 2026-09-01).

### aa-control / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.229 | 0.229 | -0.17% | +0.48% |
| direct-escape/ci-html-heavy | 11.994 | 11.820 | -1.73% | -1.17% |
| direct-escape/plain-128 | 0.027 | 0.027 | -0.16% | +0.12% |
| direct-escape/late-quote-128 | 0.070 | 0.071 | +0.51% | +2.23% |
| direct-escape/plain-512 | 0.045 | 0.045 | +0.14% | -0.02% |
| direct-escape/late-quote-512 | 0.083 | 0.083 | -0.17% | -0.06% |
| direct-escape/plain-1024 | 0.060 | 0.060 | +0.21% | -0.05% |
| direct-escape/late-quote-1024 | 0.114 | 0.114 | -0.13% | -0.73% |
| direct-escape/plain-4096 | 0.191 | 0.192 | -0.21% | +0.77% |
| direct-escape/late-quote-4096 | 0.220 | 0.224 | +3.47% | +0.06% |
| direct-escape/plain-8192 | 0.308 | 0.307 | -0.79% | +0.11% |
| direct-escape/late-quote-8192 | 0.331 | 0.331 | -0.00% | -0.14% |
| direct-escape/plain-65536 | 3.139 | 3.037 | -3.09% | -3.41% |
| direct-escape/late-quote-65536 | 3.108 | 3.102 | +0.37% | -0.78% |
| direct-escape/quotes-4096 | 26.291 | 25.912 | -0.24% | -2.62% |
| direct-escape/attr-quotes-4096 | 96.660 | 96.599 | -0.15% | +0.03% |
| direct-escape/quotes-65536 | 4579.457 | 4515.763 | -2.25% | -0.52% |
| direct-escape/attr-quotes-65536 | 18005.175 | 17912.295 | -1.14% | +0.11% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.048 | +0.31% | -0.06% |
| direct-escape/late-quote-reserved-8192 | 0.318 | 0.317 | -0.33% | +0.06% |
| direct-escape/late-quote-reserved-65536 | 3.149 | 3.148 | -0.03% | -0.07% |
| direct-escape/html-heavy-65536 | 157.453 | 155.812 | -2.02% | -0.04% |
| direct-escape/attr-plain-512 | 0.047 | 0.047 | -0.01% | -0.35% |
| direct-escape/attr-plain-8192 | 0.358 | 0.358 | -0.18% | +0.34% |
| direct-escape/attr-plain-65536 | 3.329 | 3.360 | +0.41% | +1.48% |

### aa-control / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 409.820 | 410.637 | +0.95% | -0.56% |
| ox-huge-tables | 9617.083 | 9645.951 | +0.92% | -0.31% |
| short | 0.703 | 0.708 | -0.12% | +1.45% |
| multiline | 20.377 | 20.315 | +0.38% | -0.99% |
| inline | 74.760 | 75.803 | +3.57% | -0.76% |
| code | 22.046 | 21.901 | +0.26% | -1.56% |
| escape-dense | 21.191 | 21.074 | +0.37% | -1.46% |
| tables/tables-plain | 39.518 | 39.557 | +0.14% | +0.06% |
| guard/one-table | 25.267 | 25.100 | -0.24% | -1.08% |
| mdx/short | 1.887 | 1.938 | +3.56% | +1.84% |
| mdx/tables-code | 254.342 | 255.170 | +0.75% | -0.09% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.316 | 29.027 | +0.72% | -2.64% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.740 | 67.199 | +0.25% | -4.61% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 104.197 | 102.709 | -0.22% | -2.61% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.834 | 27.794 | +0.48% | -0.74% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 43.519 | 43.247 | +0.13% | -1.34% |
| html/declaration-long-lines | 3.935 | 3.863 | -2.09% | -1.56% |
| fence/root-indented | 409.023 | 407.880 | -0.02% | -0.54% |
| fence/root-container | 452.307 | 451.165 | -0.05% | -0.46% |
| tables | 65.234 | 65.825 | +0.08% | +1.73% |
| guard/deep-emphasis | 61.359 | 63.331 | +3.34% | +3.09% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.523 | 2.553 | -0.08% | +2.52% |
| html/comment-long-lines | 3.771 | 3.806 | -0.32% | +2.19% |
| html/script-candidates | 57.018 | 57.043 | +0.28% | -0.19% |
| html/processing-long-lines | 3.889 | 3.942 | +0.43% | +2.30% |
| html/custom-tags | 341.796 | 341.453 | +0.00% | -0.20% |
| commonmark/tiny | 0.664 | 0.672 | -4.30% | +6.94% |
| html/root-short-lines | 41.504 | 41.572 | +0.30% | +0.03% |
| html/root-long-lines | 3.448 | 3.462 | -0.18% | +0.95% |
| html/separate-blocks | 292.523 | 291.348 | -0.40% | -0.41% |
| html/script-long-lines | 3.887 | 3.962 | +0.80% | +3.07% |
| html/comment-short-lines | 155.811 | 155.074 | +5.49% | -6.28% |
| html/cdata-long-lines | 3.905 | 3.912 | +0.42% | -0.10% |
| html/container-html | 143.517 | 144.271 | +1.06% | -0.00% |
| fence/root-short | 349.191 | 344.509 | -1.76% | -0.92% |
| fence/root-long | 5.557 | 5.573 | +0.55% | +0.03% |
| fence/root-tabs | 321.111 | 316.948 | -2.46% | -0.11% |
| fence/root-crlf | 322.456 | 318.042 | -2.24% | -0.48% |

### attr-outlined / attribute-screen

| Case | Before µs¹ | After µs¹ | attribute-screen-1 | attribute-screen-2 |
|---|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.649 | 0.656 | +0.72% | +1.51% |
| attribute-fence/double-quotes-128 | 1.113 | 1.093 | -1.65% | -1.83% |
| attribute-fence/single-quotes-128 | 1.089 | 1.071 | -1.91% | -1.30% |
| attribute-fence/mixed-quotes-128 | 1.268 | 1.258 | -0.44% | -1.11% |
| attribute-fence/plain-4096 | 4.250 | 4.216 | -0.38% | -1.22% |
| attribute-fence/double-quotes-4096 | 70.939 | 14.374 | -79.53% | -79.95% |
| attribute-fence/single-quotes-4096 | 70.324 | 13.962 | -79.98% | -80.31% |
| attribute-fence/mixed-quotes-4096 | 102.937 | 18.382 | -82.47% | -81.81% |
| attribute-fence/plain-65536 | 59.442 | 59.280 | -0.20% | -0.35% |
| attribute-fence/double-quotes-65536 | 12003.740 | 224.034 | -98.14% | -98.13% |
| attribute-fence/single-quotes-65536 | 11943.033 | 211.546 | -98.24% | -98.22% |
| attribute-fence/mixed-quotes-65536 | 18104.585 | 283.488 | -98.43% | -98.43% |
| attribute-fence/plain-1048576 | 996.155 | 982.721 | -0.58% | -2.10% |
| attribute-fence/sparse-quotes-1048576 | 19520.069 | 986.046 | -94.92% | -94.98% |

### attr-outlined / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.228 | 0.194 | -15.23% | -15.28% |
| direct-escape/ci-html-heavy | 11.914 | 5.469 | -53.68% | -54.50% |
| direct-escape/plain-128 | 0.027 | 0.026 | -0.81% | -1.44% |
| direct-escape/late-quote-128 | 0.070 | 0.073 | +3.41% | +4.28% |
| direct-escape/plain-512 | 0.045 | 0.032 | -27.90% | -27.21% |
| direct-escape/late-quote-512 | 0.085 | 0.071 | -14.63% | -16.80% |
| direct-escape/plain-1024 | 0.060 | 0.046 | -22.96% | -23.14% |
| direct-escape/late-quote-1024 | 0.116 | 0.090 | -20.64% | -23.58% |
| direct-escape/plain-4096 | 0.195 | 0.164 | -14.61% | -17.19% |
| direct-escape/late-quote-4096 | 0.221 | 0.185 | -16.12% | -16.42% |
| direct-escape/plain-8192 | 0.311 | 0.259 | -17.55% | -15.61% |
| direct-escape/late-quote-8192 | 0.334 | 0.285 | -14.55% | -15.13% |
| direct-escape/plain-65536 | 3.123 | 2.545 | -18.95% | -18.06% |
| direct-escape/late-quote-65536 | 3.095 | 2.602 | -15.84% | -16.02% |
| direct-escape/quotes-4096 | 26.031 | 3.860 | -84.92% | -85.42% |
| direct-escape/attr-quotes-4096 | 96.938 | 14.162 | -85.34% | -85.44% |
| direct-escape/quotes-65536 | 4577.423 | 61.128 | -98.66% | -98.67% |
| direct-escape/attr-quotes-65536 | 18000.997 | 224.377 | -98.76% | -98.75% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.035 | -26.57% | -26.66% |
| direct-escape/late-quote-reserved-8192 | 0.319 | 0.260 | -18.25% | -18.42% |
| direct-escape/late-quote-reserved-65536 | 3.149 | 2.680 | -14.82% | -15.00% |
| direct-escape/html-heavy-65536 | 157.123 | 71.058 | -54.76% | -54.79% |
| direct-escape/attr-plain-512 | 0.047 | 0.037 | -18.82% | -22.56% |
| direct-escape/attr-plain-8192 | 0.359 | 0.299 | -16.84% | -16.69% |
| direct-escape/attr-plain-65536 | 3.333 | 2.778 | -16.53% | -16.75% |

### attr-outlined / regime-screen

| Case | Before µs¹ | After µs¹ | regime-screen-1 | regime-screen-2 |
|---|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.375 | 0.380 | +0.03% | +2.31% |
| escape-regime/code-plain-128 | 0.423 | 0.407 | -3.49% | -3.97% |
| escape-regime/code-late-quote-128 | 0.443 | 0.422 | -8.87% | -0.33% |
| escape-regime/code-quotes-128 | 0.673 | 0.651 | -5.19% | -1.24% |
| escape-regime/code-late-quote-512 | 0.476 | 0.455 | -6.87% | -1.47% |
| escape-regime/plain-4096 | 1.105 | 1.072 | -3.37% | -2.52% |
| escape-regime/code-plain-4096 | 0.637 | 0.608 | -6.60% | -2.34% |
| escape-regime/code-late-quote-4096 | 0.674 | 0.624 | -11.44% | -3.06% |
| escape-regime/code-quotes-4096 | 28.977 | 6.559 | -77.55% | -77.18% |
| escape-regime/sparse-amp-4096 | 3.767 | 3.589 | -5.00% | -4.47% |
| escape-regime/plain-65536 | 12.666 | 12.177 | -3.69% | -4.03% |
| escape-regime/code-plain-65536 | 5.003 | 4.438 | -11.34% | -11.22% |
| escape-regime/code-late-quote-65536 | 5.063 | 4.480 | -12.00% | -11.04% |
| escape-regime/code-quotes-65536 | 4563.716 | 97.010 | -97.86% | -97.89% |
| escape-regime/sparse-amp-65536 | 53.374 | 52.188 | -2.49% | -1.95% |
| escape-regime/plain-1048576 | 221.663 | 206.602 | -6.15% | -7.40% |
| escape-regime/code-plain-1048576 | 99.991 | 84.423 | -14.45% | -16.48% |
| escape-regime/code-late-quote-1048576 | 99.998 | 84.055 | -14.82% | -16.85% |

### attr-outlined / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 426.274 | 408.582 | -2.65% | -5.62% |
| ox-huge-tables | 9865.121 | 9518.446 | -2.52% | -4.48% |
| short | 0.715 | 0.709 | -0.09% | -1.39% |
| multiline | 20.492 | 20.517 | +1.13% | -0.87% |
| inline | 75.148 | 77.138 | -0.55% | +5.87% |
| code | 22.148 | 21.852 | -1.23% | -1.45% |
| escape-dense | 21.218 | 21.243 | -1.15% | +1.39% |
| tables/tables-plain | 39.936 | 40.473 | +1.09% | +1.60% |
| guard/one-table | 25.314 | 25.472 | +0.02% | +1.22% |
| mdx/short | 1.925 | 1.907 | +1.06% | -2.86% |
| mdx/tables-code | 255.218 | 256.261 | -0.63% | +1.45% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.398 | 28.236 | -2.68% | -5.18% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.226 | 64.978 | -2.73% | -6.71% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 103.083 | 101.612 | -0.02% | -2.80% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.556 | 26.798 | -2.30% | -3.20% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.518 | 42.550 | +0.30% | -0.15% |
| html/declaration-long-lines | 3.908 | 3.874 | -0.74% | -0.97% |
| fence/root-indented | 406.555 | 401.880 | -1.15% | -1.15% |
| fence/root-container | 450.357 | 446.777 | -0.74% | -0.85% |
| tables | 65.632 | 64.857 | -0.89% | -1.47% |
| guard/deep-emphasis | 63.541 | 63.365 | -0.24% | -0.32% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.556 | 2.498 | -1.59% | -2.96% |
| html/comment-long-lines | 3.787 | 3.794 | +0.20% | +0.17% |
| html/script-candidates | 57.186 | 56.832 | -1.07% | -0.17% |
| html/processing-long-lines | 3.886 | 3.916 | +1.61% | -0.07% |
| html/custom-tags | 330.551 | 337.084 | +0.29% | +3.69% |
| commonmark/tiny | 0.697 | 0.668 | -3.85% | -4.54% |
| html/root-short-lines | 41.413 | 41.432 | -0.01% | +0.10% |
| html/root-long-lines | 3.466 | 3.447 | -0.41% | -0.72% |
| html/separate-blocks | 280.698 | 281.066 | +0.26% | +0.01% |
| html/script-long-lines | 3.875 | 3.916 | +0.85% | +1.28% |
| html/comment-short-lines | 167.550 | 142.830 | -13.98% | -15.54% |
| html/cdata-long-lines | 3.863 | 3.914 | +1.78% | +0.90% |
| html/container-html | 143.162 | 143.315 | +0.03% | +0.19% |
| fence/root-short | 344.580 | 191.777 | -44.59% | -44.10% |
| fence/root-long | 5.524 | 5.004 | -9.70% | -9.13% |
| fence/root-tabs | 316.544 | 173.064 | -45.48% | -45.17% |
| fence/root-crlf | 317.690 | 174.977 | -44.87% | -44.98% |

### masked-direct / attribute-screen

| Case | Before µs¹ | After µs¹ | attribute-screen-1 | attribute-screen-2 |
|---|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.640 | 0.673 | +6.82% | +3.38% |
| attribute-fence/double-quotes-128 | 1.100 | 1.096 | -0.85% | +0.10% |
| attribute-fence/single-quotes-128 | 1.083 | 1.089 | -0.80% | +1.96% |
| attribute-fence/mixed-quotes-128 | 1.247 | 1.274 | +1.54% | +2.78% |
| attribute-fence/plain-4096 | 4.227 | 5.436 | +27.95% | +29.25% |
| attribute-fence/double-quotes-4096 | 70.862 | 15.577 | -77.89% | -78.15% |
| attribute-fence/single-quotes-4096 | 70.196 | 15.532 | -77.85% | -77.90% |
| attribute-fence/mixed-quotes-4096 | 101.776 | 20.505 | -79.80% | -79.91% |
| attribute-fence/plain-65536 | 59.227 | 78.803 | +33.23% | +32.87% |
| attribute-fence/double-quotes-65536 | 12068.674 | 242.777 | -97.99% | -97.99% |
| attribute-fence/single-quotes-65536 | 11952.759 | 240.288 | -97.99% | -97.99% |
| attribute-fence/mixed-quotes-65536 | 18112.078 | 321.604 | -98.25% | -98.20% |
| attribute-fence/plain-1048576 | 993.858 | 1282.228 | +28.80% | +29.23% |
| attribute-fence/sparse-quotes-1048576 | 19242.449 | 1282.203 | -93.37% | -93.31% |

### masked-direct / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.232 | 0.192 | -18.33% | -15.97% |
| direct-escape/ci-html-heavy | 12.057 | 5.459 | -55.95% | -53.45% |
| direct-escape/plain-128 | 0.027 | 0.026 | -1.80% | -0.97% |
| direct-escape/late-quote-128 | 0.070 | 0.070 | -0.70% | -0.08% |
| direct-escape/plain-512 | 0.045 | 0.034 | -27.25% | -18.75% |
| direct-escape/late-quote-512 | 0.084 | 0.065 | -23.54% | -21.47% |
| direct-escape/plain-1024 | 0.060 | 0.047 | -22.84% | -19.57% |
| direct-escape/late-quote-1024 | 0.115 | 0.090 | -21.14% | -21.42% |
| direct-escape/plain-4096 | 0.192 | 0.163 | -14.74% | -15.12% |
| direct-escape/late-quote-4096 | 0.221 | 0.182 | -17.73% | -17.38% |
| direct-escape/plain-8192 | 0.309 | 0.261 | -15.02% | -16.19% |
| direct-escape/late-quote-8192 | 0.333 | 0.278 | -16.82% | -16.09% |
| direct-escape/plain-65536 | 3.122 | 2.568 | -18.62% | -16.91% |
| direct-escape/late-quote-65536 | 3.075 | 2.664 | -14.15% | -12.59% |
| direct-escape/quotes-4096 | 26.028 | 4.190 | -84.29% | -83.52% |
| direct-escape/attr-quotes-4096 | 96.574 | 14.239 | -85.37% | -85.15% |
| direct-escape/quotes-65536 | 4531.311 | 65.943 | -98.54% | -98.55% |
| direct-escape/attr-quotes-65536 | 17947.216 | 228.082 | -98.72% | -98.74% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.038 | -24.67% | -18.78% |
| direct-escape/late-quote-reserved-8192 | 0.320 | 0.260 | -19.18% | -18.41% |
| direct-escape/late-quote-reserved-65536 | 3.166 | 2.690 | -15.51% | -14.56% |
| direct-escape/html-heavy-65536 | 158.805 | 70.330 | -56.56% | -54.83% |
| direct-escape/attr-plain-512 | 0.047 | 0.039 | -23.11% | -11.22% |
| direct-escape/attr-plain-8192 | 0.361 | 0.298 | -17.52% | -17.42% |
| direct-escape/attr-plain-65536 | 3.354 | 2.785 | -17.08% | -16.84% |

### masked-direct / regime-screen

| Case | Before µs¹ | After µs¹ | regime-screen-1 | regime-screen-2 |
|---|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.376 | 0.378 | +0.89% | +0.21% |
| escape-regime/code-plain-128 | 0.419 | 0.392 | -7.95% | -5.10% |
| escape-regime/code-late-quote-128 | 0.437 | 0.410 | -5.99% | -6.13% |
| escape-regime/code-quotes-128 | 0.684 | 0.652 | -4.66% | -4.60% |
| escape-regime/code-late-quote-512 | 0.468 | 0.431 | -7.03% | -8.86% |
| escape-regime/plain-4096 | 1.108 | 1.076 | -2.13% | -3.61% |
| escape-regime/code-plain-4096 | 0.640 | 0.623 | -3.02% | -2.37% |
| escape-regime/code-late-quote-4096 | 0.660 | 0.640 | -3.27% | -2.84% |
| escape-regime/code-quotes-4096 | 30.080 | 7.181 | -75.74% | -76.49% |
| escape-regime/sparse-amp-4096 | 3.762 | 4.169 | +11.65% | +10.01% |
| escape-regime/plain-65536 | 12.697 | 12.132 | -4.09% | -4.80% |
| escape-regime/code-plain-65536 | 5.028 | 4.534 | -10.99% | -8.64% |
| escape-regime/code-late-quote-65536 | 5.048 | 4.600 | -8.20% | -9.56% |
| escape-regime/code-quotes-65536 | 4591.152 | 103.964 | -97.74% | -97.73% |
| escape-regime/sparse-amp-65536 | 53.262 | 60.492 | +13.63% | +13.52% |
| escape-regime/plain-1048576 | 228.510 | 213.485 | -6.37% | -6.78% |
| escape-regime/code-plain-1048576 | 105.676 | 90.528 | -12.75% | -15.88% |
| escape-regime/code-late-quote-1048576 | 106.338 | 89.406 | -15.25% | -16.58% |

### masked-direct / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 411.724 | 412.074 | +0.27% | -0.10% |
| ox-huge-tables | 9583.804 | 9665.367 | +0.40% | +1.30% |
| short | 0.691 | 0.720 | +5.01% | +3.55% |
| multiline | 20.494 | 22.102 | +7.95% | +7.74% |
| inline | 74.990 | 78.872 | +3.66% | +6.71% |
| code | 22.167 | 22.068 | -0.99% | +0.10% |
| escape-dense | 21.290 | 21.329 | -0.35% | +0.73% |
| tables/tables-plain | 39.674 | 39.852 | +0.36% | +0.53% |
| guard/one-table | 25.333 | 25.225 | +0.04% | -0.89% |
| mdx/short | 1.905 | 1.976 | +4.83% | +2.58% |
| mdx/tables-code | 254.516 | 257.534 | +0.52% | +1.85% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.458 | 28.530 | -4.71% | -1.51% |
| corpus/vue-docs/src/guide/extras/render-function.md | 68.394 | 66.500 | -4.28% | -1.17% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 103.802 | 104.014 | +0.51% | -0.11% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.372 | 27.382 | -0.41% | +0.50% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.553 | 43.814 | +3.52% | +2.41% |
| html/declaration-long-lines | 3.939 | 3.887 | -0.49% | -2.15% |
| fence/root-indented | 409.035 | 403.982 | -1.50% | -0.97% |
| fence/root-container | 452.600 | 450.572 | -0.42% | -0.48% |
| tables | 65.857 | 65.302 | -1.22% | -0.46% |
| guard/deep-emphasis | 62.380 | 62.520 | +2.98% | -2.45% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.540 | 2.606 | +1.94% | +3.26% |
| html/comment-long-lines | 3.777 | 3.820 | +1.29% | +1.01% |
| html/script-candidates | 56.929 | 61.146 | +7.66% | +7.16% |
| html/processing-long-lines | 3.908 | 3.917 | -1.00% | +1.44% |
| html/custom-tags | 329.043 | 351.372 | +5.54% | +8.03% |
| commonmark/tiny | 0.670 | 0.686 | -0.31% | +5.20% |
| html/root-short-lines | 41.342 | 51.700 | +15.37% | +34.74% |
| html/root-long-lines | 3.445 | 3.526 | +1.36% | +3.36% |
| html/separate-blocks | 281.757 | 285.593 | -0.05% | +2.76% |
| html/script-long-lines | 3.905 | 3.909 | -0.93% | +1.15% |
| html/comment-short-lines | 153.587 | 136.893 | -15.47% | -5.46% |
| html/cdata-long-lines | 3.901 | 3.887 | -0.42% | -0.29% |
| html/container-html | 142.683 | 148.201 | +4.32% | +3.42% |
| fence/root-short | 344.636 | 194.382 | -44.78% | -42.42% |
| fence/root-long | 5.525 | 5.005 | -9.08% | -9.73% |
| fence/root-tabs | 316.658 | 175.100 | -45.47% | -43.94% |
| fence/root-crlf | 318.235 | 177.740 | -45.12% | -43.17% |

### probe-attr-outlined / attribute-screen

| Case | Before µs¹ | After µs¹ | attribute-screen-1 | attribute-screen-2 |
|---|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.652 | 0.666 | +1.74% | +2.78% |
| attribute-fence/double-quotes-128 | 1.118 | 1.089 | -2.43% | -2.77% |
| attribute-fence/single-quotes-128 | 1.097 | 1.060 | -3.56% | -3.09% |
| attribute-fence/mixed-quotes-128 | 1.258 | 1.247 | -1.02% | -0.73% |
| attribute-fence/plain-4096 | 4.258 | 4.202 | -1.30% | -1.36% |
| attribute-fence/double-quotes-4096 | 71.308 | 14.220 | -80.02% | -80.10% |
| attribute-fence/single-quotes-4096 | 70.757 | 13.849 | -80.38% | -80.48% |
| attribute-fence/mixed-quotes-4096 | 101.668 | 18.375 | -81.99% | -81.86% |
| attribute-fence/plain-65536 | 59.540 | 59.377 | +0.31% | -0.86% |
| attribute-fence/double-quotes-65536 | 11968.640 | 216.891 | -98.18% | -98.20% |
| attribute-fence/single-quotes-65536 | 12052.302 | 212.324 | -98.24% | -98.24% |
| attribute-fence/mixed-quotes-65536 | 18030.773 | 284.927 | -98.42% | -98.42% |
| attribute-fence/plain-1048576 | 992.395 | 976.566 | -1.74% | -1.45% |
| attribute-fence/sparse-quotes-1048576 | 19267.048 | 977.979 | -94.91% | -94.94% |

### probe-attr-outlined / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.240 | 0.191 | -23.81% | -16.65% |
| direct-escape/ci-html-heavy | 11.869 | 5.574 | -51.94% | -54.12% |
| direct-escape/plain-128 | 0.027 | 0.027 | +3.47% | -1.37% |
| direct-escape/late-quote-128 | 0.071 | 0.072 | +0.64% | +2.27% |
| direct-escape/plain-512 | 0.061 | 0.032 | -58.61% | -27.74% |
| direct-escape/late-quote-512 | 0.084 | 0.069 | -17.90% | -16.65% |
| direct-escape/plain-1024 | 0.060 | 0.048 | -19.44% | -19.62% |
| direct-escape/late-quote-1024 | 0.114 | 0.090 | -20.71% | -20.49% |
| direct-escape/plain-4096 | 0.196 | 0.164 | -18.03% | -14.02% |
| direct-escape/late-quote-4096 | 0.220 | 0.215 | -15.61% | +11.07% |
| direct-escape/plain-8192 | 0.307 | 0.253 | -17.89% | -17.67% |
| direct-escape/late-quote-8192 | 0.331 | 0.283 | -13.57% | -15.56% |
| direct-escape/plain-65536 | 3.121 | 2.412 | -22.31% | -23.13% |
| direct-escape/late-quote-65536 | 3.101 | 2.411 | -22.91% | -21.59% |
| direct-escape/quotes-4096 | 26.068 | 3.809 | -85.41% | -85.37% |
| direct-escape/attr-quotes-4096 | 97.023 | 14.046 | -85.66% | -85.39% |
| direct-escape/quotes-65536 | 4561.018 | 60.741 | -98.67% | -98.67% |
| direct-escape/attr-quotes-65536 | 18028.908 | 225.033 | -98.75% | -98.75% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.035 | -27.46% | -27.81% |
| direct-escape/late-quote-reserved-8192 | 0.318 | 0.263 | -17.39% | -17.26% |
| direct-escape/late-quote-reserved-65536 | 3.153 | 2.475 | -21.38% | -21.63% |
| direct-escape/html-heavy-65536 | 157.264 | 71.073 | -54.86% | -54.75% |
| direct-escape/attr-plain-512 | 0.047 | 0.037 | -22.10% | -17.90% |
| direct-escape/attr-plain-8192 | 0.359 | 0.296 | -16.95% | -17.76% |
| direct-escape/attr-plain-65536 | 3.330 | 2.862 | -14.08% | -14.01% |

### probe-attr-outlined / regime-screen

| Case | Before µs¹ | After µs¹ | regime-screen-1 | regime-screen-2 |
|---|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.369 | 0.384 | +4.52% | +3.74% |
| escape-regime/code-plain-128 | 0.424 | 0.413 | -3.86% | -1.35% |
| escape-regime/code-late-quote-128 | 0.442 | 0.428 | -4.26% | -2.08% |
| escape-regime/code-quotes-128 | 0.670 | 0.661 | -2.72% | +0.11% |
| escape-regime/code-late-quote-512 | 0.473 | 0.466 | -2.34% | -0.65% |
| escape-regime/plain-4096 | 1.098 | 1.118 | +2.02% | +1.58% |
| escape-regime/code-plain-4096 | 0.644 | 0.633 | -2.40% | -0.94% |
| escape-regime/code-late-quote-4096 | 0.665 | 0.647 | -3.08% | -2.34% |
| escape-regime/code-quotes-4096 | 28.899 | 6.698 | -76.83% | -76.82% |
| escape-regime/sparse-amp-4096 | 3.771 | 4.211 | +11.37% | +11.97% |
| escape-regime/plain-65536 | 12.682 | 12.708 | +0.14% | +0.27% |
| escape-regime/code-plain-65536 | 5.086 | 4.517 | -10.73% | -11.66% |
| escape-regime/code-late-quote-65536 | 5.144 | 4.551 | -11.90% | -11.16% |
| escape-regime/code-quotes-65536 | 4578.922 | 99.254 | -97.88% | -97.78% |
| escape-regime/sparse-amp-65536 | 53.591 | 62.729 | +16.82% | +17.29% |
| escape-regime/plain-1048576 | 221.934 | 221.123 | -5.72% | +5.30% |
| escape-regime/code-plain-1048576 | 96.569 | 81.738 | -26.75% | -2.05% |
| escape-regime/code-late-quote-1048576 | 96.916 | 82.089 | -26.58% | -2.17% |

### probe-attr-outlined / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 423.735 | 404.879 | -4.69% | -4.21% |
| ox-huge-tables | 9847.936 | 9415.682 | -4.46% | -4.32% |
| short | 0.694 | 0.721 | +9.64% | -1.52% |
| multiline | 20.321 | 20.135 | -0.91% | -0.92% |
| inline | 75.496 | 74.421 | -2.53% | -0.30% |
| code | 22.037 | 21.618 | -2.13% | -1.67% |
| escape-dense | 21.130 | 21.043 | -0.41% | -0.41% |
| tables/tables-plain | 39.776 | 39.070 | -2.01% | -1.54% |
| guard/one-table | 25.527 | 25.038 | -2.85% | -0.96% |
| mdx/short | 1.929 | 1.900 | +0.13% | -3.04% |
| mdx/tables-code | 256.280 | 251.029 | -1.64% | -2.46% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.310 | 28.625 | -2.64% | -2.03% |
| corpus/vue-docs/src/guide/extras/render-function.md | 67.725 | 66.001 | -2.31% | -2.78% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 104.077 | 101.165 | -3.59% | -2.01% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.488 | 26.912 | -2.24% | -1.95% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.443 | 43.007 | +1.54% | +1.12% |
| html/declaration-long-lines | 3.900 | 3.868 | -0.72% | -0.94% |
| fence/root-indented | 408.222 | 403.417 | -0.85% | -1.50% |
| fence/root-container | 452.371 | 450.203 | -0.51% | -0.45% |
| tables | 65.538 | 64.670 | -0.94% | -1.71% |
| guard/deep-emphasis | 62.379 | 62.495 | +3.85% | -3.37% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.583 | 2.531 | -2.46% | -1.50% |
| html/comment-long-lines | 3.779 | 3.801 | +0.98% | +0.19% |
| html/script-candidates | 57.060 | 61.205 | +7.11% | +7.42% |
| html/processing-long-lines | 3.918 | 3.943 | +0.64% | +0.66% |
| html/custom-tags | 331.828 | 345.537 | +3.21% | +5.07% |
| commonmark/tiny | 0.674 | 0.692 | +6.54% | -0.90% |
| html/root-short-lines | 41.481 | 47.814 | +15.15% | +15.39% |
| html/root-long-lines | 3.449 | 3.497 | +1.03% | +1.79% |
| html/separate-blocks | 281.001 | 282.492 | +0.45% | +0.61% |
| html/script-long-lines | 3.882 | 3.988 | +3.49% | +1.97% |
| html/comment-short-lines | 163.656 | 145.517 | -13.82% | -8.32% |
| html/cdata-long-lines | 3.885 | 3.947 | +1.35% | +1.87% |
| html/container-html | 143.133 | 148.414 | +3.71% | +3.67% |
| fence/root-short | 344.406 | 194.369 | -43.36% | -43.77% |
| fence/root-long | 5.504 | 5.035 | -8.59% | -8.46% |
| fence/root-tabs | 316.704 | 175.233 | -44.69% | -44.65% |
| fence/root-crlf | 318.395 | 177.214 | -44.34% | -44.34% |

### probe-isolated / attribute-screen

| Case | Before µs¹ | After µs¹ | attribute-screen-1 | attribute-screen-2 |
|---|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.652 | 0.722 | +11.40% | +10.13% |
| attribute-fence/double-quotes-128 | 1.122 | 1.137 | -0.09% | +2.86% |
| attribute-fence/single-quotes-128 | 1.096 | 1.117 | +1.41% | +2.47% |
| attribute-fence/mixed-quotes-128 | 1.253 | 1.303 | +3.70% | +4.33% |
| attribute-fence/plain-4096 | 4.273 | 6.732 | +57.81% | +57.30% |
| attribute-fence/double-quotes-4096 | 70.503 | 16.942 | -75.92% | -76.02% |
| attribute-fence/single-quotes-4096 | 70.357 | 16.781 | -76.15% | -76.15% |
| attribute-fence/mixed-quotes-4096 | 101.607 | 21.841 | -78.63% | -78.38% |
| attribute-fence/plain-65536 | 59.431 | 99.700 | +68.36% | +67.15% |
| attribute-fence/double-quotes-65536 | 12006.855 | 264.274 | -97.80% | -97.80% |
| attribute-fence/single-quotes-65536 | 12047.608 | 261.762 | -97.84% | -97.82% |
| attribute-fence/mixed-quotes-65536 | 18290.210 | 342.256 | -98.12% | -98.14% |
| attribute-fence/plain-1048576 | 995.341 | 1611.615 | +62.09% | +61.75% |
| attribute-fence/sparse-quotes-1048576 | 19327.758 | 1609.694 | -91.63% | -91.71% |

### probe-isolated / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.230 | 0.193 | -16.38% | -16.14% |
| direct-escape/ci-html-heavy | 11.891 | 5.474 | -53.13% | -54.81% |
| direct-escape/plain-128 | 0.027 | 0.026 | -1.66% | -1.85% |
| direct-escape/late-quote-128 | 0.071 | 0.070 | -0.19% | -1.36% |
| direct-escape/plain-512 | 0.045 | 0.032 | -27.61% | -27.81% |
| direct-escape/late-quote-512 | 0.084 | 0.066 | -21.51% | -21.17% |
| direct-escape/plain-1024 | 0.060 | 0.046 | -22.75% | -22.60% |
| direct-escape/late-quote-1024 | 0.114 | 0.091 | -19.52% | -19.73% |
| direct-escape/plain-4096 | 0.191 | 0.163 | -14.82% | -14.59% |
| direct-escape/late-quote-4096 | 0.222 | 0.183 | -16.83% | -18.71% |
| direct-escape/plain-8192 | 0.308 | 0.258 | -15.68% | -16.41% |
| direct-escape/late-quote-8192 | 0.331 | 0.280 | -14.66% | -16.27% |
| direct-escape/plain-65536 | 3.129 | 2.546 | -18.62% | -18.65% |
| direct-escape/late-quote-65536 | 3.087 | 2.601 | -15.24% | -16.25% |
| direct-escape/quotes-4096 | 26.072 | 4.078 | -84.40% | -84.32% |
| direct-escape/attr-quotes-4096 | 96.513 | 14.125 | -85.36% | -85.37% |
| direct-escape/quotes-65536 | 4562.248 | 65.672 | -98.56% | -98.56% |
| direct-escape/attr-quotes-65536 | 18088.097 | 228.031 | -98.74% | -98.74% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.035 | -27.80% | -25.38% |
| direct-escape/late-quote-reserved-8192 | 0.319 | 0.261 | -17.45% | -18.95% |
| direct-escape/late-quote-reserved-65536 | 3.162 | 2.664 | -15.72% | -15.78% |
| direct-escape/html-heavy-65536 | 160.487 | 70.621 | -55.13% | -56.84% |
| direct-escape/attr-plain-512 | 0.047 | 0.036 | -22.60% | -23.05% |
| direct-escape/attr-plain-8192 | 0.359 | 0.298 | -16.64% | -17.58% |
| direct-escape/attr-plain-65536 | 3.364 | 2.782 | -16.61% | -17.98% |

### probe-isolated / regime-screen

| Case | Before µs¹ | After µs¹ | regime-screen-1 | regime-screen-2 |
|---|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.376 | 0.375 | -0.35% | -0.22% |
| escape-regime/code-plain-128 | 0.418 | 0.410 | -1.92% | -1.66% |
| escape-regime/code-late-quote-128 | 0.437 | 0.424 | -3.49% | -2.40% |
| escape-regime/code-quotes-128 | 0.669 | 0.636 | -5.96% | -3.94% |
| escape-regime/code-late-quote-512 | 0.467 | 0.456 | -3.97% | -1.01% |
| escape-regime/plain-4096 | 1.107 | 1.105 | -0.53% | +0.10% |
| escape-regime/code-plain-4096 | 0.643 | 0.619 | -3.76% | -3.71% |
| escape-regime/code-late-quote-4096 | 0.664 | 0.632 | -6.00% | -3.65% |
| escape-regime/code-quotes-4096 | 29.979 | 6.777 | -76.88% | -77.87% |
| escape-regime/sparse-amp-4096 | 3.769 | 4.328 | +15.31% | +14.34% |
| escape-regime/plain-65536 | 12.676 | 12.734 | +0.34% | +0.57% |
| escape-regime/code-plain-65536 | 5.017 | 4.533 | -9.58% | -9.70% |
| escape-regime/code-late-quote-65536 | 5.135 | 4.590 | -12.03% | -9.18% |
| escape-regime/code-quotes-65536 | 4545.422 | 101.262 | -97.77% | -97.77% |
| escape-regime/sparse-amp-65536 | 53.277 | 60.774 | +14.08% | +14.06% |
| escape-regime/plain-1048576 | 228.908 | 228.831 | -0.17% | +0.11% |
| escape-regime/code-plain-1048576 | 104.998 | 88.885 | -15.39% | -15.30% |
| escape-regime/code-late-quote-1048576 | 105.888 | 88.547 | -16.49% | -16.26% |

### probe-isolated / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 419.528 | 411.492 | +1.36% | -5.01% |
| ox-huge-tables | 9755.115 | 9596.957 | +1.20% | -4.32% |
| short | 0.714 | 0.730 | +4.36% | +0.21% |
| multiline | 21.590 | 21.298 | +3.57% | -5.83% |
| inline | 75.060 | 77.132 | +3.42% | +2.10% |
| code | 22.014 | 22.064 | +0.26% | +0.19% |
| escape-dense | 21.107 | 20.878 | -0.68% | -1.49% |
| tables/tables-plain | 39.707 | 40.696 | +2.51% | +2.47% |
| guard/one-table | 25.424 | 25.469 | +1.21% | -0.84% |
| mdx/short | 1.923 | 1.955 | +1.75% | +1.64% |
| mdx/tables-code | 256.117 | 255.615 | -0.06% | -0.33% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 29.374 | 29.159 | +2.79% | -4.11% |
| corpus/vue-docs/src/guide/extras/render-function.md | 67.675 | 67.403 | +2.60% | -3.34% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 102.725 | 102.454 | +1.31% | -1.82% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.444 | 27.203 | -0.01% | -1.74% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.510 | 43.208 | +1.36% | +1.93% |
| html/declaration-long-lines | 3.940 | 3.892 | -1.93% | -0.52% |
| fence/root-indented | 407.299 | 404.027 | -1.44% | -0.16% |
| fence/root-container | 452.357 | 448.933 | -1.22% | -0.29% |
| tables | 66.131 | 66.162 | -0.99% | +1.10% |
| guard/deep-emphasis | 63.085 | 61.969 | +0.72% | -4.19% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.568 | 2.539 | +0.25% | -2.47% |
| html/comment-long-lines | 3.793 | 3.813 | -0.11% | +1.16% |
| html/script-candidates | 57.127 | 57.634 | +0.80% | +0.98% |
| html/processing-long-lines | 3.898 | 3.891 | -0.16% | -0.20% |
| html/custom-tags | 329.371 | 351.144 | +5.87% | +7.35% |
| commonmark/tiny | 0.665 | 0.690 | +6.42% | +1.13% |
| html/root-short-lines | 41.573 | 47.825 | +15.33% | +14.75% |
| html/root-long-lines | 3.457 | 3.521 | +2.35% | +1.38% |
| html/separate-blocks | 281.364 | 261.905 | -7.73% | -6.10% |
| html/script-long-lines | 3.898 | 3.891 | +0.22% | -0.58% |
| html/comment-short-lines | 163.905 | 127.478 | -20.13% | -24.23% |
| html/cdata-long-lines | 3.901 | 3.899 | -0.63% | +0.53% |
| html/container-html | 143.574 | 148.765 | +3.95% | +3.28% |
| fence/root-short | 344.532 | 186.928 | -45.92% | -45.57% |
| fence/root-long | 5.540 | 4.987 | -10.08% | -9.88% |
| fence/root-tabs | 317.430 | 169.103 | -46.83% | -46.62% |
| fence/root-crlf | 318.683 | 171.474 | -46.19% | -46.20% |

### writers-outlined / attribute-screen

| Case | Before µs¹ | After µs¹ | attribute-screen-1 | attribute-screen-2 |
|---|---:|---:|---:|---:|
| attribute-fence/plain-128 | 0.647 | 0.670 | +4.40% | +2.80% |
| attribute-fence/double-quotes-128 | 1.102 | 1.106 | -0.33% | +1.00% |
| attribute-fence/single-quotes-128 | 1.076 | 1.069 | -1.28% | -0.17% |
| attribute-fence/mixed-quotes-128 | 1.247 | 1.241 | -0.46% | -0.46% |
| attribute-fence/plain-4096 | 4.260 | 4.137 | -3.61% | -2.16% |
| attribute-fence/double-quotes-4096 | 71.183 | 15.145 | -78.53% | -78.91% |
| attribute-fence/single-quotes-4096 | 70.250 | 14.153 | -79.96% | -79.74% |
| attribute-fence/mixed-quotes-4096 | 103.028 | 19.061 | -81.57% | -81.43% |
| attribute-fence/plain-65536 | 59.738 | 57.768 | -3.45% | -3.15% |
| attribute-fence/double-quotes-65536 | 11996.555 | 234.492 | -98.00% | -98.09% |
| attribute-fence/single-quotes-65536 | 11999.189 | 218.581 | -98.16% | -98.19% |
| attribute-fence/mixed-quotes-65536 | 18157.659 | 296.876 | -98.36% | -98.37% |
| attribute-fence/plain-1048576 | 995.339 | 955.747 | -4.33% | -3.63% |
| attribute-fence/sparse-quotes-1048576 | 19314.372 | 951.160 | -95.02% | -95.13% |

### writers-outlined / direct

| Case | Before µs¹ | After µs¹ | direct-1 | direct-2 |
|---|---:|---:|---:|---:|
| direct-escape/ci-plain | 0.228 | 0.190 | -17.10% | -16.71% |
| direct-escape/ci-html-heavy | 11.839 | 5.305 | -55.03% | -55.35% |
| direct-escape/plain-128 | 0.027 | 0.026 | -1.26% | -0.05% |
| direct-escape/late-quote-128 | 0.070 | 0.071 | +0.33% | +0.58% |
| direct-escape/plain-512 | 0.045 | 0.032 | -28.28% | -28.52% |
| direct-escape/late-quote-512 | 0.083 | 0.066 | -21.17% | -21.52% |
| direct-escape/plain-1024 | 0.060 | 0.048 | -20.04% | -20.59% |
| direct-escape/late-quote-1024 | 0.114 | 0.090 | -21.71% | -20.48% |
| direct-escape/plain-4096 | 0.192 | 0.163 | -14.91% | -15.15% |
| direct-escape/late-quote-4096 | 0.221 | 0.184 | -17.20% | -16.68% |
| direct-escape/plain-8192 | 0.308 | 0.253 | -18.10% | -18.07% |
| direct-escape/late-quote-8192 | 0.331 | 0.277 | -16.46% | -16.37% |
| direct-escape/plain-65536 | 3.138 | 2.437 | -22.19% | -22.51% |
| direct-escape/late-quote-65536 | 3.105 | 2.456 | -21.38% | -20.40% |
| direct-escape/quotes-4096 | 26.316 | 4.109 | -84.31% | -84.46% |
| direct-escape/attr-quotes-4096 | 96.901 | 14.758 | -84.79% | -84.75% |
| direct-escape/quotes-65536 | 4559.401 | 65.525 | -98.55% | -98.57% |
| direct-escape/attr-quotes-65536 | 18222.831 | 238.414 | -98.69% | -98.70% |
| direct-escape/late-quote-reserved-512 | 0.048 | 0.035 | -29.16% | -28.16% |
| direct-escape/late-quote-reserved-8192 | 0.318 | 0.263 | -17.38% | -17.47% |
| direct-escape/late-quote-reserved-65536 | 3.160 | 2.519 | -20.16% | -20.38% |
| direct-escape/html-heavy-65536 | 158.841 | 70.276 | -56.08% | -55.44% |
| direct-escape/attr-plain-512 | 0.047 | 0.036 | -20.54% | -26.30% |
| direct-escape/attr-plain-8192 | 0.362 | 0.298 | -18.20% | -16.82% |
| direct-escape/attr-plain-65536 | 3.354 | 2.886 | -14.32% | -13.59% |

### writers-outlined / regime-screen

| Case | Before µs¹ | After µs¹ | regime-screen-1 | regime-screen-2 |
|---|---:|---:|---:|---:|
| escape-regime/plain-128 | 0.373 | 0.366 | -0.58% | -2.92% |
| escape-regime/code-plain-128 | 0.418 | 0.406 | -2.94% | -2.92% |
| escape-regime/code-late-quote-128 | 0.439 | 0.420 | -4.00% | -4.31% |
| escape-regime/code-quotes-128 | 0.674 | 0.652 | -3.04% | -3.43% |
| escape-regime/code-late-quote-512 | 0.472 | 0.446 | -5.71% | -5.64% |
| escape-regime/plain-4096 | 1.098 | 1.091 | -0.84% | -0.59% |
| escape-regime/code-plain-4096 | 0.642 | 0.628 | -1.88% | -2.58% |
| escape-regime/code-late-quote-4096 | 0.667 | 0.632 | -5.14% | -5.21% |
| escape-regime/code-quotes-4096 | 29.300 | 7.009 | -75.92% | -76.24% |
| escape-regime/sparse-amp-4096 | 3.786 | 3.813 | -6.68% | +8.07% |
| escape-regime/plain-65536 | 12.666 | 12.759 | +0.92% | +0.54% |
| escape-regime/code-plain-65536 | 5.015 | 4.496 | -10.38% | -10.31% |
| escape-regime/code-late-quote-65536 | 5.152 | 4.552 | -9.32% | -13.93% |
| escape-regime/code-quotes-65536 | 4593.339 | 103.739 | -97.73% | -97.75% |
| escape-regime/sparse-amp-65536 | 53.730 | 50.742 | -5.25% | -5.87% |
| escape-regime/plain-1048576 | 222.231 | 214.204 | -0.69% | -6.33% |
| escape-regime/code-plain-1048576 | 97.625 | 76.502 | -15.16% | -27.13% |
| escape-regime/code-late-quote-1048576 | 98.457 | 76.509 | -14.73% | -28.62% |

### writers-outlined / screen

| Case | Before µs¹ | After µs¹ | screen-1 | screen-2 |
|---|---:|---:|---:|---:|
| ox-large-cm | 410.172 | 408.917 | -0.02% | -0.59% |
| ox-huge-tables | 9596.109 | 9479.078 | -0.83% | -1.61% |
| short | 0.698 | 0.710 | +1.65% | +1.67% |
| multiline | 20.414 | 20.765 | +1.07% | +2.36% |
| inline | 74.707 | 76.343 | +4.20% | +0.19% |
| code | 21.938 | 21.849 | +0.18% | -0.98% |
| escape-dense | 21.118 | 20.702 | -1.60% | -2.33% |
| tables/tables-plain | 39.916 | 39.305 | -1.24% | -1.82% |
| guard/one-table | 25.249 | 25.338 | +0.18% | +0.52% |
| mdx/short | 1.905 | 1.878 | -1.71% | -1.05% |
| mdx/tables-code | 258.122 | 254.396 | +0.63% | -3.43% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 28.981 | 28.181 | -2.90% | -2.62% |
| corpus/vue-docs/src/guide/extras/render-function.md | 67.837 | 65.565 | -2.91% | -3.79% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 102.843 | 101.260 | -1.46% | -1.62% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 27.371 | 26.191 | -4.33% | -4.29% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 42.453 | 40.971 | -3.57% | -3.41% |
| html/declaration-long-lines | 3.914 | 3.842 | -0.78% | -2.88% |
| fence/root-indented | 407.631 | 400.334 | -1.56% | -2.02% |
| fence/root-container | 451.442 | 446.498 | -0.73% | -1.46% |
| tables | 65.485 | 64.786 | -0.97% | -1.16% |
| guard/deep-emphasis | 62.980 | 61.568 | -3.36% | -1.13% |
| corpus/ox-parser/SIMPLE_MD/1 | 2.551 | 2.490 | -2.38% | -2.40% |
| html/comment-long-lines | 3.789 | 3.751 | -1.50% | -0.48% |
| html/script-candidates | 56.835 | 57.736 | +1.54% | +1.63% |
| html/processing-long-lines | 3.896 | 3.881 | -0.41% | -0.37% |
| html/custom-tags | 328.280 | 331.606 | +0.09% | +1.93% |
| commonmark/tiny | 0.673 | 0.658 | -3.00% | -1.70% |
| html/root-short-lines | 41.292 | 44.525 | +9.22% | +6.44% |
| html/root-long-lines | 3.440 | 3.449 | -1.39% | +1.94% |
| html/separate-blocks | 279.936 | 249.082 | -11.18% | -10.86% |
| html/script-long-lines | 3.888 | 3.875 | -0.03% | -0.60% |
| html/comment-short-lines | 165.826 | 111.705 | -32.61% | -32.66% |
| html/cdata-long-lines | 3.861 | 3.920 | +0.89% | +2.16% |
| html/container-html | 142.412 | 147.875 | +3.72% | +3.95% |
| fence/root-short | 342.479 | 187.995 | -44.90% | -45.31% |
| fence/root-long | 5.511 | 4.897 | -10.75% | -11.56% |
| fence/root-tabs | 315.498 | 173.210 | -45.03% | -45.17% |
| fence/root-crlf | 317.429 | 174.980 | -44.78% | -44.97% |

¹ Median of the pair medians; the per-pair percentage columns are the primary observations.
