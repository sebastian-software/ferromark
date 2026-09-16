# Every document against the preceding native comparison

Positive percentages mean an increase in v2 processing time relative to the
peer, compared with the previous run. These are relative time changes, whereas
`previous-comparison.md` reports throughput changes; they are reciprocals,
not interchangeable percentages. Separate runs do not establish causality.

Every one of the 57 documents is listed in both lifecycles, ordered by the
change relative to v1. A dagger marks differing HTML: that cell is diagnostic,
not an equivalent-output competitive score. JSON retains exact classifications.

| Document | Lifecycle | Bytes | vs OX | vs v1 | vs md4c | vs pulldown | vs Bun |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| wiki-volcano-first-paragraph | reuse | 2644 | +0.43%† | +0.62% | +0.30% | -0.10% | -5.01% |
| rust-book-appendix-02-operators | fresh | 22595 | -0.93%† | +0.20% | -0.84% | +4.41% | -2.04% |
| wiki-volcano-article-body | fresh | 69241 | -0.28%† | -0.17%† | -0.16%† | +0.04% | -2.91% |
| wiki-volcano-lead | reuse | 4232 | -0.61%† | -0.44% | -0.47% | -1.33% | -6.79% |
| rust-book-appendix-02-operators | reuse | 22595 | -0.72%† | -0.60% | -0.76% | +3.14% | -2.04% |
| wiki-tea-article-body | reuse | 58814 | -0.35%† | -0.62% | +0.83%† | +0.92% | -2.12% |
| wiki-tea-lead | reuse | 6363 | -0.29%† | -0.66% | +0.21% | -0.41% | -3.78% |
| legacy-docs-readme | reuse | 1825 | -1.64%† | -0.74% | -1.08% | -1.16% | -4.16% |
| wiki-chess-lead | reuse | 4125 | -0.80%† | -0.85% | -0.80% | -0.74% | -5.45% |
| comment-table | reuse | 310 | -1.03% | -0.89% | -1.21% | -0.27% | -3.65% |
| wiki-volcano-article-body | reuse | 69241 | -1.17%† | -1.08%† | +0.91%† | +0.73% | -3.49% |
| wiki-tea-article-body | fresh | 58814 | -2.83%† | -1.12% | -0.30%† | -0.01% | -3.29% |
| wiki-tea-lead | fresh | 6363 | -0.79%† | -1.35% | -0.08% | -1.25% | -4.00% |
| wiki-volcano-lead | fresh | 4232 | -1.48%† | -1.42% | -0.97% | -1.14% | -6.56% |
| wiki-chess-article-body | reuse | 113609 | -2.84%† | -1.43%† | -1.76%† | -2.58% | -5.40% |
| wiki-volcano-first-paragraph | fresh | 2644 | -2.46%† | -1.56% | -1.72% | -2.60% | -7.88% |
| wiki-rainbow-article-body | reuse | 48422 | +0.64%† | -1.61%† | +1.99%† | +2.20% | -1.69% |
| wiki-chess-lead | fresh | 4125 | -1.55%† | -1.71% | -0.81% | -1.88% | -5.95% |
| wiki-chess-article-body | fresh | 113609 | -2.51%† | -2.01%† | -1.55%† | -1.75% | -4.72% |
| wiki-tea-first-paragraph | reuse | 1126 | -3.13%† | -2.48% | -2.03% | -3.18% | -6.06% |
| wiki-chess-first-paragraph | reuse | 1190 | -1.72%† | -2.56% | -1.13% | -1.62% | -6.99% |
| wiki-rainbow-first-paragraph | reuse | 859 | -2.43%† | -2.60% | -2.47% | -2.92% | -6.92% |
| legacy-docs-readme | fresh | 1825 | -3.37%† | -2.87% | -3.18% | -2.40% | -6.63% |
| wiki-rainbow-article-body | fresh | 48422 | -0.63%† | -2.98%† | -1.21%† | -1.05% | -4.86% |
| legacy-docs-migration-0-4 | reuse | 7045 | -3.60%† | -3.17% | -3.90% | -4.24% | -9.30% |
| vite-docs-philosophy | reuse | 3575 | -4.39%† | -3.33% | -3.87% | -4.16% | -11.13% |
| wiki-rainbow-lead | reuse | 1859 | -3.37%† | -3.41% | -3.21% | -3.37% | -9.11% |
| legacy-docs-mdx | reuse | 7422 | -4.35%† | -3.42% | -4.32% | -3.76% | -7.98% |
| legacy-node-ferromark-readme | reuse | 9075 | -3.69%† | -3.57% | -3.15% | -2.35% | -7.74% |
| legacy-docs-mdx | fresh | 7422 | -4.10%† | -4.13% | -4.29% | -4.74% | -8.54% |
| legacy-docs-migration-0-4 | fresh | 7045 | -4.57%† | -4.15% | -4.63% | -3.37% | -7.89% |
| legacy-docs-readme-theme | reuse | 2848 | -5.21%† | -4.28% | -3.21% | -3.99% | -10.83% |
| legacy-node-ferromark-readme | fresh | 9075 | -4.70%† | -4.34% | -4.32% | -3.71% | -8.56% |
| rust-book-ch03-04-comments | reuse | 393 | -5.50%† | -4.40% | -5.29% | -4.60% | -9.54% |
| vite-docs-api-plugin | reuse | 31890 | -6.82%† | -4.77% | -4.42% | -2.75% | -6.13% |
| wiki-chess-first-paragraph | fresh | 1190 | -4.26%† | -4.78% | -3.58% | -5.33% | -9.79% |
| vite-docs-philosophy | fresh | 3575 | -5.76%† | -4.87% | -5.28% | -6.07% | -12.11% |
| rust-book-ch17-00-async-await | fresh | 9734 | -4.78%† | -4.89% | -5.65% | -5.23% | -8.99% |
| comment-table | fresh | 310 | -4.42% | -4.94% | -4.42% | -3.92% | -7.35% |
| wiki-rainbow-plain-prose | fresh | 38800 | -5.42% | -5.03% | -4.76% | -5.52% | -8.48% |
| vue-docs-suspense | reuse | 8291 | -5.16%† | -5.18% | -5.80% | -5.64% | -9.53% |
| vite-docs-performance | reuse | 8184 | -6.63%† | -5.31% | -5.73% | -5.23% | -10.07% |
| legacy-docs-markdown-extensions | reuse | 3707 | -5.52%† | -5.32% | -5.83% | -5.68% | -12.10% |
| typescript-handbook-typescript-5-0 | fresh | 50714 | -4.07%† | -5.47% | -3.58% | -1.28% | -5.46% |
| wiki-rainbow-plain-prose | reuse | 38800 | -4.78% | -5.48% | -5.22% | -5.24% | -9.07% |
| legacy-docs-migration-0-2 | reuse | 1985 | -6.13%† | -5.49% | -6.28% | -6.23% | -10.63% |
| wiki-tea-first-paragraph | fresh | 1126 | -5.78%† | -5.49% | -5.63% | -6.92% | -9.03% |
| rust-book-ch17-00-async-await | reuse | 9734 | -4.74%† | -5.50% | -6.26% | -4.65% | -8.75% |
| vue-docs-suspense | fresh | 8291 | -5.52%† | -5.51% | -5.83% | -5.02% | -9.66% |
| typescript-handbook-the-handbook | reuse | 5337 | -6.29%† | -5.52% | -5.67% | -5.91% | -12.25% |
| legacy-docs-adr-readme-theme-composition | reuse | 1532 | -5.24%† | -5.63% | -6.57% | -5.35% | -11.42% |
| comment-links | reuse | 278 | -4.01% | -5.74% | -4.71% | -4.46% | -9.49% |
| comment-inline-code | reuse | 285 | -5.79% | -5.88% | -6.39% | -6.37% | -10.97% |
| legacy-contributing | reuse | 9323 | -5.31%† | -6.11% | -6.04% | -4.77% | -8.83% |
| wiki-tea-plain-prose | fresh | 40577 | -6.81% | -6.17% | -6.79% | -6.53% | -9.74% |
| comment-unicode | reuse | 327 | -6.68% | -6.26% | -6.83% | -7.14% | -12.35% |
| wiki-rainbow-lead | fresh | 1859 | -6.49%† | -6.39% | -6.01% | -6.47% | -11.62% |
| vue-docs-ways-of-using-vue | reuse | 5883 | -7.02%† | -6.44% | -6.13% | -6.37% | -12.86% |
| typescript-handbook-typescript-5-0 | reuse | 50714 | -9.22%† | -6.47% | -5.69% | -3.73% | -7.13% |
| typescript-handbook-the-handbook | fresh | 5337 | -6.78%† | -6.47% | -6.37% | -6.06% | -13.65% |
| legacy-contributing | fresh | 9323 | -6.32%† | -6.54% | -6.55% | -7.02% | -8.92% |
| legacy-docs-migration-0-3 | reuse | 2379 | -6.76%† | -6.57% | -6.22% | -5.36% | -10.99% |
| legacy-docs-markdown-extensions | fresh | 3707 | -6.95%† | -6.91% | -6.58% | -6.76% | -13.38% |
| legacy-docs-readme-theme | fresh | 2848 | -7.77%† | -7.04% | -6.64% | -7.40% | -13.57% |
| legacy-docs-migration-0-8 | reuse | 2374 | -7.21%† | -7.13% | -7.22% | -7.38% | -11.78% |
| vite-docs-performance | fresh | 8184 | -7.64%† | -7.26% | -6.71% | -7.08% | -11.08% |
| vite-docs-api-plugin | fresh | 31890 | -5.30%† | -7.38% | -4.76% | -2.76% | -6.52% |
| legacy-docs-releasing | reuse | 4141 | -8.16%† | -7.61% | -7.66% | -9.00% | -12.06% |
| vue-docs-ways-of-using-vue | fresh | 5883 | -8.25%† | -7.83% | -7.91% | -8.16% | -13.61% |
| vue-docs-slots | fresh | 24211 | -4.55%† | -7.85% | -5.19% | -4.68% | -7.19% |
| wiki-rainbow-first-paragraph | fresh | 859 | -8.58%† | -7.87% | -8.38% | -9.04% | -12.41% |
| wiki-chess-plain-prose | reuse | 80966 | -6.48% | -7.93% | -6.74% | -7.07% | -10.49% |
| wiki-chess-plain-prose | fresh | 80966 | -6.11% | -7.94% | -8.24% | -7.80% | -11.35% |
| legacy-docs-migration-0-2 | fresh | 1985 | -8.11%† | -8.05% | -8.81% | -7.96% | -13.65% |
| comment-reproduction | reuse | 298 | -9.94% | -8.23% | -9.11% | -10.32% | -13.28% |
| legacy-docs-migration-0-3 | fresh | 2379 | -8.89%† | -8.37% | -8.17% | -8.58% | -13.62% |
| legacy-docs-adr-readme-theme-composition | fresh | 1532 | -8.65%† | -8.47% | -8.66% | -9.39% | -14.59% |
| wiki-tea-plain-prose | reuse | 40577 | -8.18% | -8.60% | -7.71% | -6.84% | -11.29% |
| vue-docs-slots | reuse | 24211 | -4.54%† | -8.75% | -4.25% | -4.41% | -6.44% |
| legacy-docs-migration-0-8 | fresh | 2374 | -9.60%† | -8.82% | -8.28% | -8.70% | -13.22% |
| wiki-volcano-plain-prose | fresh | 47303 | -9.36% | -9.09% | -9.60% | -10.12% | -13.96% |
| wiki-volcano-plain-prose | reuse | 47303 | -9.66% | -9.22% | -9.06% | -8.93% | -12.72% |
| vite-docs-features | reuse | 39739 | -11.09%† | -9.65%† | -9.82% | -7.18% | -10.54% |
| legacy-docs-releasing | fresh | 4141 | -8.64%† | -9.75% | -8.78% | -8.00% | -13.51% |
| vue-docs-reactivity-in-depth | reuse | 24001 | -4.90%† | -9.80% | -5.63% | -4.53% | -8.21% |
| comment-review-long | reuse | 957 | -11.45% | -10.00% | -10.55% | -10.30% | -16.63% |
| vite-docs-features | fresh | 39739 | -6.78%† | -10.66%† | -6.88% | -4.07% | -8.26% |
| comment-quote | reuse | 290 | -11.18% | -10.68% | -10.18% | -9.73% | -15.07% |
| vue-docs-reactivity-in-depth | fresh | 24001 | -6.13%† | -11.80% | -6.31% | -6.24% | -8.67% |
| typescript-handbook-compiler-options | reuse | 54026 | -12.85%† | -11.87% | -11.93% | -12.09% | -12.88% |
| comment-review | reuse | 282 | -12.22% | -12.59% | -12.34% | -13.31% | -17.23% |
| comment-question | reuse | 160 | -11.80% | -12.73% | -12.00% | -12.24% | -17.59% |
| typescript-handbook-compiler-options | fresh | 54026 | -13.46%† | -12.85% | -13.15% | -13.64% | -13.33% |
| typescript-handbook-advanced-types | fresh | 36745 | -12.25%† | -13.25% | -12.61% | -9.61% | -14.19% |
| rust-book-ch03-04-comments | fresh | 393 | -13.99%† | -13.29% | -13.06% | -13.43% | -16.83% |
| comment-checklist | reuse | 287 | -14.40% | -13.67% | -14.09%† | -14.33% | -17.46%† |
| guard-angle-link | reuse | 41 | -14.34% | -13.81%† | -14.24% | -15.17% | -17.38% |
| typescript-handbook-advanced-types | reuse | 36745 | -13.54%† | -13.90% | -13.03% | -10.10% | -14.36% |
| comment-links | fresh | 278 | -14.98% | -13.95% | -15.08% | -14.99% | -20.11% |
| comment-review-long | fresh | 957 | -14.79% | -14.28% | -15.06% | -14.34% | -20.44% |
| comment-ack | reuse | 37 | -14.69% | -14.77% | -15.07% | -15.77% | -17.85% |
| comment-unicode | fresh | 327 | -13.79% | -14.96% | -14.03% | -13.87% | -20.18% |
| comment-inline-code | fresh | 285 | -19.56% | -18.86% | -19.35% | -20.43% | -23.84% |
| comment-checklist | fresh | 287 | -19.65% | -19.44% | -19.67%† | -20.40% | -23.52%† |
| comment-reproduction | fresh | 298 | -19.05% | -19.73% | -20.10% | -20.70% | -22.96% |
| comment-quote | fresh | 290 | -22.97% | -22.19% | -21.15% | -20.84% | -25.51% |
| rust-book-ch00-00-introduction | reuse | 10839 | -22.36%† | -22.39% | -22.91% | -22.12% | -25.38% |
| rust-book-ch00-00-introduction | fresh | 10839 | -22.64%† | -22.47% | -23.47% | -22.29% | -26.09% |
| guard-angle-link | fresh | 41 | -23.28% | -24.20%† | -23.92% | -25.10% | -27.16% |
| comment-review | fresh | 282 | -26.54% | -25.66% | -26.49% | -26.52% | -30.17% |
| comment-question | fresh | 160 | -31.98% | -31.83% | -32.25% | -31.87% | -35.56% |
| comment-ack | fresh | 37 | -34.89% | -35.84% | -35.15% | -37.40% | -37.71% |
| comment-incident | fresh | 1124 | -35.47%† | -36.31% | -36.47% | -35.99% | -39.94% |
| comment-incident | reuse | 1124 | -36.15%† | -36.49% | -36.60% | -35.78% | -40.72% |
