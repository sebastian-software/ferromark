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
| comment-ack | reuse | 37 | +0.20% | +0.76% | +29.40% | -1.57% | -0.59% |
| comment-reproduction | reuse | 298 | -1.85% | -0.01% | +16.02% | -0.80% | -0.67% |
| comment-question | reuse | 160 | -0.45% | -0.07% | +25.68% | -0.20% | -1.03% |
| typescript-handbook-compiler-options | reuse | 54026 | -0.59%† | -0.12% | -0.13% | -3.47% | -1.45% |
| comment-review | reuse | 282 | -1.98% | -0.16% | +19.10% | -1.77% | -2.38% |
| comment-checklist | reuse | 287 | -2.37% | -0.23% | +14.06%† | -2.17% | -1.82%† |
| comment-ack | fresh | 37 | +0.23% | -0.65% | +27.55% | -3.60% | -1.48% |
| comment-unicode | fresh | 327 | +1.03% | -0.87% | +16.64% | +0.08% | -0.72% |
| typescript-handbook-compiler-options | fresh | 54026 | -0.60%† | -1.01% | -1.86% | -3.84% | -1.48% |
| comment-reproduction | fresh | 298 | -1.87% | -1.05% | +15.14% | -0.71% | -0.37% |
| comment-table | fresh | 310 | -0.17% | -1.10% | +9.05% | -0.17% | -0.77% |
| comment-review | fresh | 282 | -2.05% | -1.10% | +18.25% | -2.66% | -2.22% |
| comment-incident | fresh | 1124 | -2.10%† | -1.44% | +5.36% | -1.41% | -1.81% |
| comment-table | reuse | 310 | -1.52% | -1.71% | +8.17% | -1.62% | -1.29% |
| comment-unicode | reuse | 327 | -0.23% | -1.73% | +14.78% | -1.59% | -1.62% |
| rust-book-ch17-00-async-await | fresh | 9734 | -1.08%† | -1.83% | -0.48% | -2.71% | -2.79% |
| legacy-docs-migration-0-8 | fresh | 2374 | -2.01%† | -1.84% | +1.42% | -0.99% | -2.00% |
| comment-quote | fresh | 290 | -2.00% | -1.88% | +16.92% | -1.30% | -1.01% |
| legacy-docs-migration-0-2 | reuse | 1985 | -2.73%† | -1.95% | +1.98% | -1.50% | -2.06% |
| legacy-docs-migration-0-3 | fresh | 2379 | -2.55%† | -1.96% | +2.47% | -1.02% | -1.67% |
| comment-quote | reuse | 290 | -2.01% | -1.97% | +18.67% | -2.01% | -0.91% |
| comment-checklist | fresh | 287 | -2.63% | -1.97% | +12.10%† | -3.06% | -2.41%† |
| legacy-docs-migration-0-8 | reuse | 2374 | -2.21%† | -2.04% | +1.72% | -1.29% | -2.55% |
| wiki-chess-plain-prose | fresh | 80966 | -3.09% | -2.15% | -2.34% | -2.30% | -2.42% |
| comment-incident | reuse | 1124 | -2.57%† | -2.17% | +5.73% | -2.58% | -2.31% |
| legacy-docs-migration-0-3 | reuse | 2379 | -2.37%† | -2.22% | +2.29% | -1.06% | -1.56% |
| wiki-tea-lead | reuse | 6363 | -1.19%† | -2.37% | +0.25% | -1.69% | -4.51% |
| rust-book-ch00-00-introduction | fresh | 10839 | -3.01%† | -2.38% | -1.58% | -3.23% | -3.06% |
| wiki-tea-first-paragraph | reuse | 1126 | -1.59%† | -2.44% | +1.58% | -2.89% | -5.09% |
| wiki-rainbow-plain-prose | reuse | 38800 | -3.79% | -2.48% | -3.49% | -2.98% | -3.63% |
| wiki-chess-plain-prose | reuse | 80966 | -3.64% | -2.53% | -2.33% | -2.56% | -3.36% |
| wiki-tea-first-paragraph | fresh | 1126 | -1.45%† | -2.58% | +2.42% | -2.43% | -4.68% |
| wiki-rainbow-plain-prose | fresh | 38800 | -3.93% | -2.59% | -3.31% | -3.20% | -2.66% |
| comment-question | fresh | 160 | -2.66% | -2.67% | +21.66% | -2.72% | -2.65% |
| rust-book-appendix-02-operators | fresh | 22595 | -1.89%† | -2.78% | -1.10% | -2.38% | -3.02% |
| legacy-docs-migration-0-2 | fresh | 1985 | -3.88%† | -2.78% | +1.94% | -1.89% | -2.14% |
| wiki-volcano-first-paragraph | fresh | 2644 | -1.26%† | -2.78% | +0.79% | -3.01% | -3.65% |
| rust-book-appendix-02-operators | reuse | 22595 | -1.67%† | -2.79% | -1.22% | -2.45% | -2.94% |
| wiki-tea-plain-prose | fresh | 40577 | -4.88% | -2.79% | -2.36% | -2.88% | -3.35% |
| rust-book-ch17-00-async-await | reuse | 9734 | -2.26%† | -2.85% | -1.47% | -3.02% | -3.45% |
| legacy-docs-migration-0-4 | reuse | 7045 | -3.82%† | -2.86% | -1.79% | -2.09% | -3.35% |
| wiki-tea-plain-prose | reuse | 40577 | -4.48% | -2.87% | -2.24% | -2.87% | -3.25% |
| rust-book-ch00-00-introduction | reuse | 10839 | -2.42%† | -2.93% | -1.80% | -3.44% | -3.46% |
| legacy-docs-releasing | reuse | 4141 | -3.08%† | -2.96% | -0.26% | -2.74% | -3.18% |
| legacy-docs-releasing | fresh | 4141 | -2.96%† | -3.04% | +0.21% | -3.20% | -3.08% |
| wiki-volcano-lead | fresh | 4232 | -1.84%† | -3.09% | +0.26% | -2.62% | -3.88% |
| wiki-rainbow-article-body | fresh | 48422 | -5.89%† | -3.15%† | -3.71%† | -4.64% | -5.72% |
| legacy-contributing | fresh | 9323 | -3.83%† | -3.18% | -2.53% | -3.40% | -3.34% |
| wiki-volcano-first-paragraph | reuse | 2644 | -1.21%† | -3.19% | +0.47% | -2.71% | -3.74% |
| typescript-handbook-advanced-types | reuse | 36745 | -7.71%† | -3.20% | -2.08% | -4.06% | -3.79% |
| wiki-chess-first-paragraph | reuse | 1190 | -2.29%† | -3.20% | +3.10% | -3.33% | -6.12% |
| comment-inline-code | fresh | 285 | -3.12% | -3.28% | +14.15% | -3.62% | -4.43% |
| legacy-docs-migration-0-4 | fresh | 7045 | -3.62%† | -3.36% | -2.54% | -4.01% | -4.35% |
| legacy-docs-mdx | reuse | 7422 | -3.50%† | -3.41% | -1.75% | -3.45% | -3.60% |
| wiki-tea-lead | fresh | 6363 | -1.83%† | -3.48% | -1.02% | -2.59% | -4.35% |
| legacy-contributing | reuse | 9323 | -3.84%† | -3.58% | -1.20% | -3.67% | -4.54% |
| wiki-rainbow-first-paragraph | fresh | 859 | -3.31%† | -3.59% | +2.75% | -3.34% | -5.47% |
| wiki-rainbow-first-paragraph | reuse | 859 | -3.61%† | -3.67% | +2.94% | -3.31% | -5.01% |
| wiki-volcano-plain-prose | fresh | 47303 | -4.82% | -3.81% | -3.01% | -3.48% | -3.49% |
| wiki-rainbow-lead | reuse | 1859 | -3.07%† | -3.86% | +0.64% | -3.26% | -5.22% |
| legacy-node-ferromark-readme | fresh | 9075 | -3.36%† | -3.87% | -1.74% | -4.05% | -3.10% |
| wiki-chess-lead | reuse | 4125 | -2.62%† | -3.88% | -0.86% | -4.10% | -6.08% |
| legacy-node-ferromark-readme | reuse | 9075 | -4.35%† | -3.90% | -2.05% | -4.35% | -4.33% |
| wiki-volcano-article-body | fresh | 69241 | -4.43%† | -3.92%† | -3.66%† | -3.83% | -5.14% |
| wiki-rainbow-lead | fresh | 1859 | -3.21%† | -3.97% | +0.40% | -3.91% | -5.85% |
| wiki-volcano-plain-prose | reuse | 47303 | -5.29% | -4.05% | -4.10% | -4.28% | -4.90% |
| legacy-docs-mdx | fresh | 7422 | -3.70%† | -4.10% | -3.09% | -4.47% | -4.80% |
| legacy-docs-markdown-extensions | reuse | 3707 | -4.75%† | -4.21% | -1.50% | -4.70% | -4.53% |
| vue-docs-suspense | fresh | 8291 | -3.90%† | -4.35% | -2.89% | -4.11% | -4.46% |
| vue-docs-suspense | reuse | 8291 | -4.41%† | -4.35% | -3.78% | -3.85% | -4.43% |
| wiki-volcano-lead | reuse | 4232 | -2.18%† | -4.36% | -2.11% | -3.20% | -5.57% |
| legacy-docs-markdown-extensions | fresh | 3707 | -4.25%† | -4.53% | -1.14% | -4.72% | -4.88% |
| wiki-chess-first-paragraph | fresh | 1190 | -3.05%† | -4.63% | +2.90% | -3.00% | -5.50% |
| wiki-tea-article-body | reuse | 58814 | -5.41%† | -4.64% | -4.16%† | -4.15% | -5.98% |
| vue-docs-slots | reuse | 24211 | -4.99%† | -4.76% | -5.13% | -5.58% | -6.25% |
| wiki-chess-lead | fresh | 4125 | -3.84%† | -4.78% | -2.33% | -5.59% | -7.05% |
| typescript-handbook-the-handbook | fresh | 5337 | -5.92%† | -4.89% | -2.90% | -4.60% | -6.03% |
| wiki-rainbow-article-body | reuse | 48422 | -5.00%† | -5.06%† | -3.62%† | -4.85% | -5.75% |
| comment-inline-code | reuse | 285 | -3.25% | -5.16% | +15.68% | -3.67% | -4.03% |
| wiki-tea-article-body | fresh | 58814 | -5.27%† | -5.24% | -3.88%† | -4.18% | -5.81% |
| typescript-handbook-the-handbook | reuse | 5337 | -5.47%† | -5.33% | -2.82% | -5.29% | -7.26% |
| legacy-docs-readme-theme | reuse | 2848 | -6.56%† | -5.68% | -2.36% | -6.23% | -5.85% |
| legacy-docs-adr-readme-theme-composition | reuse | 1532 | -5.69%† | -5.81% | -0.03% | -5.90% | -5.56% |
| legacy-docs-adr-readme-theme-composition | fresh | 1532 | -5.84%† | -5.82% | -0.12% | -5.51% | -5.28% |
| wiki-chess-article-body | reuse | 113609 | -5.46%† | -5.83%† | -5.07%† | -5.28% | -6.38% |
| wiki-chess-article-body | fresh | 113609 | -6.09%† | -5.99%† | -5.46%† | -5.88% | -7.19% |
| vue-docs-slots | fresh | 24211 | -4.87%† | -6.48% | -4.92% | -5.77% | -6.10% |
| vite-docs-api-plugin | fresh | 31890 | -8.44%† | -6.55% | -5.71% | -6.63% | -6.63% |
| wiki-volcano-article-body | reuse | 69241 | -6.24%† | -6.56%† | -5.17%† | -5.61% | -7.23% |
| vite-docs-performance | reuse | 8184 | -6.47%† | -6.56% | -5.11% | -6.22% | -7.96% |
| legacy-docs-readme-theme | fresh | 2848 | -7.68%† | -6.74% | -3.38% | -6.19% | -5.92% |
| vite-docs-features | reuse | 39739 | -10.17%† | -7.28%† | -6.59% | -6.44% | -8.11% |
| vite-docs-performance | fresh | 8184 | -6.59%† | -7.32% | -4.45% | -6.95% | -6.74% |
| vite-docs-philosophy | reuse | 3575 | -6.99%† | -7.53% | -4.13% | -6.91% | -7.48% |
| comment-review-long | fresh | 957 | -8.79% | -7.78% | +2.09% | -7.76% | -7.91% |
| vue-docs-reactivity-in-depth | reuse | 24001 | -6.58%† | -7.87% | -5.47% | -6.59% | -7.08% |
| vite-docs-philosophy | fresh | 3575 | -7.56%† | -7.89% | -4.54% | -7.39% | -7.17% |
| legacy-docs-readme | fresh | 1825 | -7.43%† | -8.04% | -4.54% | -7.26% | -8.60% |
| vite-docs-api-plugin | reuse | 31890 | -9.24%† | -8.50% | -6.83% | -7.10% | -7.12% |
| comment-review-long | reuse | 957 | -9.01% | -8.51% | +1.26% | -8.69% | -8.53% |
| vite-docs-features | fresh | 39739 | -9.85%† | -8.70%† | -7.35% | -7.63% | -8.68% |
| legacy-docs-readme | reuse | 1825 | -9.27%† | -9.17% | -5.62% | -8.64% | -9.39% |
| typescript-handbook-advanced-types | fresh | 36745 | -11.58%† | -9.45% | -7.52% | -8.83% | -8.05% |
| vue-docs-ways-of-using-vue | reuse | 5883 | -8.82%† | -9.70% | -7.11% | -9.18% | -9.90% |
| vue-docs-ways-of-using-vue | fresh | 5883 | -9.02%† | -9.73% | -7.36% | -9.07% | -8.73% |
| vue-docs-reactivity-in-depth | fresh | 24001 | -6.27%† | -9.77% | -6.00% | -6.70% | -7.35% |
| typescript-handbook-typescript-5-0 | reuse | 50714 | -11.33%† | -9.91% | -7.92% | -9.06% | -9.63% |
| rust-book-ch03-04-comments | fresh | 393 | -10.45%† | -10.51% | -0.84% | -10.74% | -10.71% |
| comment-links | fresh | 278 | -13.58% | -12.66% | +0.97% | -12.74% | -14.16% |
| rust-book-ch03-04-comments | reuse | 393 | -12.64%† | -13.26% | -2.13% | -13.53% | -12.78% |
| comment-links | reuse | 278 | -14.85% | -13.67% | +0.02% | -13.71% | -15.22% |
| typescript-handbook-typescript-5-0 | fresh | 50714 | -17.44%† | -14.10% | -12.77% | -12.93% | -13.35% |
| guard-angle-link | fresh | 41 | -14.80% | -15.83%† | +3.31% | -17.38% | -17.64% |
| guard-angle-link | reuse | 41 | -18.70% | -19.20%† | +0.18% | -19.57% | -21.13% |
