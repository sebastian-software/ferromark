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
| typescript-handbook-typescript-5-0 | reuse | 50714 | +15.22%† | +14.53% | +10.74% | +11.49% | +14.60% |
| typescript-handbook-typescript-5-0 | fresh | 50714 | +15.26%† | +13.27% | +10.00% | +9.59% | +13.07% |
| vue-docs-reactivity-in-depth | fresh | 24001 | +4.65%† | +11.85% | +4.09% | +4.96% | +8.00% |
| guard-angle-link | reuse | 41 | +10.41% | +11.18%† | -10.20% | +13.48% | +16.20% |
| vue-docs-reactivity-in-depth | reuse | 24001 | +4.41%† | +10.29% | +3.68% | +4.01% | +7.64% |
| vite-docs-features | fresh | 39739 | +8.07%† | +10.22%† | +5.09% | +5.28% | +8.08% |
| vite-docs-features | reuse | 39739 | +9.63%† | +10.16%† | +7.47% | +7.31% | +9.56% |
| comment-links | reuse | 278 | +9.16% | +9.57% | -6.20% | +9.52% | +14.51% |
| vite-docs-api-plugin | fresh | 31890 | +9.01%† | +9.46% | +5.04% | +5.84% | +8.21% |
| typescript-handbook-advanced-types | fresh | 36745 | +8.86%† | +9.38% | +7.39% | +5.69% | +10.42% |
| typescript-handbook-advanced-types | reuse | 36745 | +11.13%† | +9.23% | +5.76% | +5.65% | +9.48% |
| vue-docs-ways-of-using-vue | reuse | 5883 | +7.38%† | +8.66% | +4.77% | +7.80% | +14.49% |
| rust-book-ch03-04-comments | fresh | 393 | +9.29%† | +8.51% | -2.35% | +9.91% | +11.87% |
| rust-book-ch03-04-comments | reuse | 393 | +8.97%† | +8.25% | -3.01% | +9.34% | +12.84% |
| vue-docs-ways-of-using-vue | fresh | 5883 | +6.91%† | +8.25% | +4.64% | +7.71% | +13.22% |
| guard-angle-link | fresh | 41 | +4.11% | +8.20%† | -12.16% | +10.04% | +12.65% |
| wiki-rainbow-first-paragraph | reuse | 859 | +7.44%† | +8.03% | +1.33% | +7.74% | +12.08% |
| wiki-rainbow-lead | reuse | 1859 | +6.78%† | +7.76% | +3.21% | +6.86% | +13.72% |
| vite-docs-api-plugin | reuse | 31890 | +9.23%† | +7.75% | +4.62% | +4.87% | +7.46% |
| comment-links | fresh | 278 | +9.14% | +7.35% | -5.82% | +8.73% | +14.53% |
| wiki-chess-first-paragraph | reuse | 1190 | +7.21%† | +7.34% | +0.90% | +7.23% | +12.56% |
| comment-review-long | reuse | 957 | +7.59% | +7.33% | -3.50% | +7.09% | +14.76% |
| wiki-rainbow-lead | fresh | 1859 | +6.44%† | +7.32% | +2.85% | +7.12% | +13.62% |
| wiki-rainbow-first-paragraph | fresh | 859 | +6.80%† | +6.96% | +1.14% | +7.29% | +11.97% |
| vite-docs-performance | fresh | 8184 | +6.14%† | +6.89% | +3.26% | +7.03% | +11.97% |
| comment-inline-code | reuse | 285 | +2.39% | +6.76% | -14.05% | +2.79% | +8.19% |
| wiki-chess-lead | fresh | 4125 | +5.51%† | +6.61% | +3.63% | +6.95% | +11.43% |
| wiki-tea-lead | reuse | 6363 | +5.83%† | +6.58% | +3.92% | +6.08% | +10.63% |
| vite-docs-philosophy | reuse | 3575 | +6.18%† | +6.54% | +3.07% | +6.67% | +13.27% |
| vite-docs-philosophy | fresh | 3575 | +5.88%† | +6.50% | +2.85% | +7.27% | +13.02% |
| wiki-tea-article-body | fresh | 58814 | +9.10%† | +6.34% | +5.44%† | +6.14% | +9.23% |
| wiki-rainbow-article-body | reuse | 48422 | +6.69%† | +6.32%† | +3.76%† | +4.71% | +7.89% |
| wiki-volcano-lead | reuse | 4232 | +5.69%† | +6.24% | +3.58% | +6.33% | +13.34% |
| wiki-chess-lead | reuse | 4125 | +5.67%† | +6.19% | +3.82% | +6.79% | +12.05% |
| wiki-tea-lead | fresh | 6363 | +5.33%† | +6.04% | +3.61% | +5.78% | +9.53% |
| wiki-tea-first-paragraph | reuse | 1126 | +5.85%† | +5.99% | +1.51% | +6.85% | +10.01% |
| vue-docs-slots | reuse | 24211 | +2.76%† | +5.99% | +2.24% | +2.85% | +6.51% |
| wiki-volcano-first-paragraph | fresh | 2644 | +4.57%† | +5.96% | +2.50% | +6.26% | +12.04% |
| legacy-docs-readme | reuse | 1825 | +4.84%† | +5.72% | +1.18% | +5.37% | +8.41% |
| legacy-docs-readme | fresh | 1825 | +4.34%† | +5.68% | +2.13% | +5.21% | +9.40% |
| wiki-volcano-first-paragraph | reuse | 2644 | +4.58%† | +5.59% | +2.22% | +5.62% | +11.46% |
| legacy-docs-readme-theme | fresh | 2848 | +5.41%† | +5.56% | +1.21% | +4.82% | +11.51% |
| wiki-volcano-lead | fresh | 4232 | +4.01%† | +5.56% | +1.94% | +4.78% | +11.09% |
| comment-review-long | fresh | 957 | +6.08% | +5.51% | -3.59% | +6.24% | +13.12% |
| vue-docs-slots | fresh | 24211 | +2.60%† | +5.49% | +2.78% | +3.70% | +7.06% |
| vite-docs-performance | reuse | 8184 | +5.53%† | +5.47% | +2.86% | +5.03% | +10.59% |
| comment-unicode | reuse | 327 | +2.99% | +5.25% | -10.72% | +5.27% | +9.41% |
| legacy-docs-markdown-extensions | fresh | 3707 | +4.48%† | +5.12% | +2.33% | +5.32% | +13.10% |
| legacy-docs-adr-readme-theme-composition | reuse | 1532 | +5.17%† | +5.12% | -0.20% | +5.52% | +11.33% |
| legacy-docs-adr-readme-theme-composition | fresh | 1532 | +4.95%† | +4.73% | -0.94% | +6.51% | +11.34% |
| wiki-volcano-article-body | reuse | 69241 | +7.12%† | +4.48%† | +1.90%† | +3.55% | +8.28% |
| legacy-contributing | reuse | 9323 | +3.86%† | +4.44% | +1.70% | +4.32% | +10.30% |
| typescript-handbook-the-handbook | reuse | 5337 | +4.07%† | +4.43% | +1.76% | +4.56% | +11.53% |
| comment-checklist | fresh | 287 | +3.61% | +4.41% | -9.46%† | +4.85% | +8.80%† |
| wiki-tea-plain-prose | reuse | 40577 | +6.19% | +4.40% | +2.72% | +2.89% | +7.52% |
| wiki-tea-first-paragraph | fresh | 1126 | +4.66%† | +4.36% | +0.26% | +5.42% | +8.81% |
| legacy-node-ferromark-readme | fresh | 9075 | +3.55%† | +4.34% | +1.77% | +4.32% | +10.28% |
| legacy-contributing | fresh | 9323 | +3.93%† | +4.20% | +2.29% | +4.19% | +8.98% |
| comment-incident | reuse | 1124 | +3.68%† | +4.17% | -3.77% | +3.40% | +11.14% |
| comment-question | fresh | 160 | +3.76% | +4.17% | -16.85% | +3.57% | +9.23% |
| typescript-handbook-the-handbook | fresh | 5337 | +4.05%† | +4.11% | +1.63% | +3.71% | +11.43% |
| legacy-docs-mdx | fresh | 7422 | +3.14%† | +4.10% | +2.43% | +5.00% | +9.91% |
| comment-ack | reuse | 37 | +4.45% | +4.08% | -19.24% | +5.59% | +8.13% |
| comment-review | fresh | 282 | +5.92% | +4.08% | -11.93% | +6.33% | +11.33% |
| legacy-docs-markdown-extensions | reuse | 3707 | +4.22%† | +4.03% | +1.57% | +4.86% | +11.67% |
| comment-unicode | fresh | 327 | +1.75% | +3.96% | -11.52% | +3.06% | +8.82% |
| legacy-docs-readme-theme | reuse | 2848 | +4.69%† | +3.91% | -0.21% | +4.46% | +11.54% |
| legacy-docs-migration-0-3 | fresh | 2379 | +3.75%† | +3.91% | -0.75% | +2.39% | +9.05% |
| legacy-docs-releasing | fresh | 4141 | +2.89%† | +3.88% | +0.53% | +3.45% | +9.71% |
| wiki-chess-first-paragraph | fresh | 1190 | +4.95%† | +3.80% | -1.19% | +5.25% | +10.32% |
| wiki-volcano-plain-prose | fresh | 47303 | +4.02% | +3.79% | +3.58% | +4.07% | +8.26% |
| legacy-node-ferromark-readme | reuse | 9075 | +3.30%† | +3.74% | +0.24% | +2.95% | +10.43% |
| legacy-docs-releasing | reuse | 4141 | +3.29%† | +3.67% | +0.76% | +4.57% | +8.86% |
| legacy-docs-migration-0-3 | reuse | 2379 | +3.84%† | +3.57% | -1.12% | +2.90% | +8.14% |
| legacy-docs-migration-0-2 | fresh | 1985 | +2.62%† | +3.52% | -0.92% | +3.51% | +8.72% |
| rust-book-ch00-00-introduction | reuse | 10839 | +3.18%† | +3.49% | +2.43% | +3.94% | +9.45% |
| comment-inline-code | fresh | 285 | +2.44% | +3.49% | -12.62% | +4.47% | +8.95% |
| comment-question | reuse | 160 | +3.22% | +3.43% | -18.28% | +3.54% | +9.72% |
| wiki-rainbow-article-body | fresh | 48422 | +6.72%† | +3.42%† | +3.42%† | +4.57% | +8.80% |
| comment-checklist | reuse | 287 | +4.35% | +3.42% | -10.18%† | +4.39% | +8.08%† |
| rust-book-ch17-00-async-await | reuse | 9734 | +1.87%† | +3.39% | +1.90% | +2.74% | +7.14% |
| legacy-docs-migration-0-4 | fresh | 7045 | +2.37%† | +3.37% | +1.91% | +4.36% | +9.56% |
| wiki-volcano-plain-prose | reuse | 47303 | +4.96% | +3.36% | +3.21% | +3.87% | +8.17% |
| legacy-docs-mdx | reuse | 7422 | +3.36%† | +3.31% | +2.30% | +3.67% | +9.58% |
| wiki-volcano-article-body | fresh | 69241 | +5.05%† | +3.25%† | +2.70%† | +3.93% | +6.40% |
| wiki-chess-plain-prose | fresh | 80966 | +2.76% | +3.18% | +3.39% | +3.07% | +7.29% |
| legacy-docs-migration-0-2 | reuse | 1985 | +3.35%† | +3.10% | -0.51% | +4.88% | +8.78% |
| comment-review | reuse | 282 | +4.83% | +3.07% | -13.63% | +5.11% | +10.63% |
| wiki-chess-article-body | fresh | 113609 | +3.10%† | +3.03%† | +2.43%† | +3.09% | +6.23% |
| rust-book-ch00-00-introduction | fresh | 10839 | +3.64%† | +2.95% | +2.31% | +3.87% | +9.34% |
| wiki-rainbow-plain-prose | fresh | 38800 | +3.94% | +2.94% | +3.44% | +3.67% | +7.79% |
| wiki-tea-article-body | reuse | 58814 | +4.11%† | +2.89% | +1.17%† | +1.63% | +4.83% |
| vue-docs-suspense | reuse | 8291 | +2.05%† | +2.86% | +1.73% | +3.10% | +8.59% |
| wiki-rainbow-plain-prose | reuse | 38800 | +3.29% | +2.83% | +3.41% | +3.56% | +8.16% |
| rust-book-ch17-00-async-await | fresh | 9734 | +1.58%† | +2.81% | +1.04% | +2.36% | +7.10% |
| legacy-docs-migration-0-4 | reuse | 7045 | +2.75%† | +2.74% | +1.38% | +3.31% | +9.08% |
| vue-docs-suspense | fresh | 8291 | +2.60%† | +2.67% | +1.53% | +2.44% | +8.92% |
| wiki-tea-plain-prose | fresh | 40577 | +4.22% | +2.67% | +2.32% | +2.52% | +6.37% |
| comment-table | reuse | 310 | +1.76% | +2.49% | -7.15% | +2.35% | +4.76% |
| comment-incident | fresh | 1124 | +2.69%† | +2.37% | -4.54% | +3.04% | +8.77% |
| comment-reproduction | fresh | 298 | +2.38% | +2.21% | -10.87% | +3.48% | +5.56% |
| legacy-docs-migration-0-8 | reuse | 2374 | +2.02%† | +2.20% | -1.42% | +2.01% | +6.85% |
| comment-ack | fresh | 37 | +0.10% | +2.05% | -20.70% | +5.44% | +5.38% |
| legacy-docs-migration-0-8 | fresh | 2374 | +2.90%† | +1.87% | -1.43% | +2.32% | +7.13% |
| wiki-chess-plain-prose | reuse | 80966 | +3.10% | +1.86% | +1.95% | +2.08% | +6.00% |
| comment-table | fresh | 310 | +0.40% | +1.33% | -8.46% | +0.88% | +4.35% |
| comment-reproduction | reuse | 298 | +4.24% | +1.10% | -10.58% | +4.04% | +6.88% |
| wiki-chess-article-body | reuse | 113609 | +2.40%† | +1.02%† | +0.75%† | +1.55% | +4.71% |
| rust-book-appendix-02-operators | reuse | 22595 | +0.72%† | +0.87% | -0.63% | -0.43% | +3.51% |
| rust-book-appendix-02-operators | fresh | 22595 | +1.27%† | +0.40% | -0.35% | -1.21% | +3.34% |
| comment-quote | fresh | 290 | +1.91% | +0.04% | -16.24% | +1.22% | +4.86% |
| comment-quote | reuse | 290 | +0.29% | -0.09% | -17.09% | +1.07% | +4.43% |
| typescript-handbook-compiler-options | fresh | 54026 | -0.39%† | -0.52% | +0.68% | +2.61% | +0.73% |
| typescript-handbook-compiler-options | reuse | 54026 | -0.86%† | -2.12% | -0.73% | +0.20% | +0.18% |
