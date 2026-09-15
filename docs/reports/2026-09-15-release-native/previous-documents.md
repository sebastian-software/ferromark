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
| comment-incident | reuse | 1124 | +42.53%† | +42.10% | +42.73% | +42.47% | +41.94% |
| comment-incident | fresh | 1124 | +34.29%† | +36.61% | +36.53% | +35.36% | +36.24% |
| rust-book-ch00-00-introduction | reuse | 10839 | +24.24%† | +23.74% | +24.91% | +22.58% | +24.88% |
| rust-book-ch00-00-introduction | fresh | 10839 | +23.55%† | +23.13% | +24.88% | +21.90% | +22.94% |
| wiki-rainbow-article-body | reuse | 48422 | +0.73%† | +10.38%† | +1.53%† | +0.61% | +2.46% |
| comment-review | reuse | 282 | +5.67% | +5.85% | +5.37% | +6.16% | +5.49% |
| vue-docs-reactivity-in-depth | reuse | 24001 | +1.47%† | +5.78% | +1.51% | -0.61% | +3.28% |
| comment-quote | reuse | 290 | +5.24% | +5.12% | +4.74% | +6.11% | +4.53% |
| wiki-tea-first-paragraph | fresh | 1126 | +3.87%† | +5.06% | +5.76% | +5.10% | +7.31% |
| vue-docs-slots | fresh | 24211 | -0.35%† | +5.03% | +0.77% | -1.30% | +0.72% |
| wiki-chess-lead | reuse | 4125 | +3.07%† | +5.03% | +5.02% | +2.49% | +6.27% |
| wiki-rainbow-first-paragraph | fresh | 859 | +3.97%† | +5.00% | +5.95% | +4.54% | +7.20% |
| wiki-rainbow-first-paragraph | reuse | 859 | +3.50%† | +4.93% | +6.25% | +4.06% | +7.43% |
| wiki-volcano-lead | reuse | 4232 | +3.69%† | +4.88% | +5.28% | +4.30% | +6.28% |
| wiki-tea-lead | reuse | 6363 | +3.25%† | +4.75% | +5.08% | +2.99% | +5.80% |
| wiki-rainbow-lead | reuse | 1859 | +3.70%† | +4.70% | +5.46% | +4.15% | +6.84% |
| comment-quote | fresh | 290 | +3.51% | +4.67% | +3.46% | +3.45% | +2.54% |
| wiki-volcano-first-paragraph | reuse | 2644 | +3.66%† | +4.65% | +5.15% | +4.03% | +6.26% |
| wiki-rainbow-lead | fresh | 1859 | +3.51%† | +4.21% | +4.88% | +3.44% | +5.65% |
| vue-docs-slots | reuse | 24211 | +0.17%† | +3.96% | +0.83% | -0.11% | +0.83% |
| wiki-volcano-lead | fresh | 4232 | +3.51%† | +3.92% | +4.34% | +3.47% | +5.79% |
| guard-angle-link | reuse | 41 | +4.31% | +3.90%† | +4.56% | +2.87% | +5.54% |
| wiki-tea-first-paragraph | reuse | 1126 | +4.08%† | +3.87% | +4.99% | +3.85% | +6.62% |
| comment-question | reuse | 160 | +3.36% | +3.64% | +3.84% | +2.95% | +3.76% |
| wiki-chess-first-paragraph | reuse | 1190 | +2.32%† | +3.53% | +3.34% | +2.81% | +6.51% |
| wiki-chess-plain-prose | reuse | 80966 | +0.92% | +3.47% | +1.82% | +1.86% | +2.75% |
| comment-ack | reuse | 37 | +4.24% | +3.47% | +4.40% | +5.26% | +3.21% |
| wiki-tea-lead | fresh | 6363 | +3.13%† | +3.38% | +4.26% | +3.14% | +5.41% |
| comment-links | reuse | 278 | +1.91% | +3.33% | +2.85% | +1.79% | +4.02% |
| comment-review | fresh | 282 | +2.08% | +3.20% | +1.74% | +2.14% | +1.89% |
| wiki-chess-first-paragraph | fresh | 1190 | +1.79%† | +3.07% | +3.31% | +3.16% | +6.04% |
| comment-checklist | reuse | 287 | +3.71% | +3.02% | +3.70%† | +3.68% | +3.36%† |
| wiki-chess-lead | fresh | 4125 | +2.77%† | +2.97% | +3.78% | +2.55% | +5.87% |
| wiki-volcano-first-paragraph | fresh | 2644 | +3.50%† | +2.89% | +3.90% | +2.69% | +5.28% |
| vite-docs-philosophy | fresh | 3575 | +2.99%† | +2.83% | +3.60% | +1.49% | +3.16% |
| comment-checklist | fresh | 287 | +3.43% | +2.79% | +3.93%† | +3.86% | +3.59%† |
| guard-angle-link | fresh | 41 | +1.45% | +2.66%† | +2.91% | +2.24% | +4.03% |
| wiki-chess-plain-prose | fresh | 80966 | -0.13% | +2.63% | +1.27% | +1.24% | +1.69% |
| vite-docs-philosophy | reuse | 3575 | +3.38%† | +2.54% | +3.86% | +1.77% | +4.41% |
| legacy-docs-mdx | fresh | 7422 | +0.32%† | +2.40% | +4.26% | +0.95% | +2.04% |
| typescript-handbook-the-handbook | reuse | 5337 | +3.24%† | +2.36% | +2.54% | +1.85% | +3.15% |
| wiki-volcano-plain-prose | fresh | 47303 | +2.94% | +2.20% | +2.11% | +1.71% | +2.38% |
| vue-docs-suspense | reuse | 8291 | +2.97%† | +2.17% | +3.74% | +1.44% | +3.98% |
| legacy-docs-migration-0-8 | reuse | 2374 | +2.52%† | +2.16% | +2.82% | +1.57% | +3.19% |
| comment-ack | fresh | 37 | +0.96% | +1.94% | +1.57% | +2.19% | +0.84% |
| wiki-tea-plain-prose | reuse | 40577 | +1.18% | +1.93% | +2.31% | +1.52% | +2.34% |
| vite-docs-performance | fresh | 8184 | +2.36%† | +1.92% | +2.42% | +0.89% | +1.68% |
| vue-docs-suspense | fresh | 8291 | +1.72%† | +1.85% | +2.72% | +0.36% | +2.61% |
| vite-docs-api-plugin | reuse | 31890 | +1.59%† | +1.80% | +3.40% | +0.48% | +3.10% |
| wiki-volcano-plain-prose | reuse | 47303 | +1.75% | +1.77% | +1.68% | +1.16% | +1.77% |
| legacy-docs-migration-0-8 | fresh | 2374 | +2.35%† | +1.74% | +2.21% | +0.97% | +2.26% |
| comment-inline-code | reuse | 285 | +3.74% | +1.72% | +3.54% | +3.37% | +3.18% |
| legacy-docs-readme-theme | reuse | 2848 | +1.60%† | +1.69% | +1.87% | +0.98% | +2.64% |
| typescript-handbook-the-handbook | fresh | 5337 | +4.09%† | +1.65% | +2.19% | +1.36% | +3.96% |
| comment-table | fresh | 310 | +1.35% | +1.64% | +2.39% | +0.66% | +1.98% |
| wiki-rainbow-plain-prose | reuse | 38800 | +1.40% | +1.64% | +1.22% | +1.41% | +1.83% |
| legacy-docs-readme | fresh | 1825 | +1.50%† | +1.64% | +3.00% | -0.10% | +1.36% |
| legacy-docs-readme-theme | fresh | 2848 | +1.73%† | +1.56% | +2.46% | +0.30% | +4.16% |
| rust-book-ch03-04-comments | reuse | 393 | +0.53%† | +1.55% | +2.39% | +0.06% | +2.19% |
| comment-table | reuse | 310 | +1.55% | +1.54% | +2.53% | +1.10% | +2.82% |
| vue-docs-ways-of-using-vue | fresh | 5883 | +2.05%† | +1.52% | +3.14% | +1.74% | +2.44% |
| comment-review-long | fresh | 957 | +1.86% | +1.47% | +2.02% | +1.78% | +2.25% |
| wiki-rainbow-plain-prose | fresh | 38800 | +3.23% | +1.45% | +1.62% | +2.01% | +1.19% |
| comment-question | fresh | 160 | +1.29% | +1.45% | +1.73% | +2.19% | +1.35% |
| legacy-docs-readme | reuse | 1825 | +1.54%† | +1.43% | +4.39% | +0.83% | +2.34% |
| wiki-rainbow-article-body | fresh | 48422 | -6.25%† | +1.39%† | -3.18%† | -4.62% | -2.95% |
| legacy-contributing | reuse | 9323 | +1.16%† | +1.33% | +3.67% | -0.96% | +1.64% |
| comment-unicode | fresh | 327 | +0.17% | +1.32% | +0.69% | -1.23% | +1.51% |
| comment-reproduction | reuse | 298 | +2.26% | +1.27% | +0.58% | +2.85% | +2.81% |
| legacy-docs-adr-readme-theme-composition | reuse | 1532 | +0.58%† | +1.27% | +1.86% | +0.00% | +0.81% |
| comment-review-long | reuse | 957 | +2.83% | +1.22% | +2.37% | +2.51% | +2.19% |
| legacy-docs-markdown-extensions | reuse | 3707 | +1.10%† | +1.14% | +2.14% | +0.31% | +0.21% |
| legacy-contributing | fresh | 9323 | +0.78%† | +1.09% | +1.74% | -0.23% | +1.03% |
| legacy-docs-adr-readme-theme-composition | fresh | 1532 | +0.78%† | +1.00% | +1.04% | -0.40% | +0.50% |
| wiki-chess-article-body | reuse | 113609 | +1.05%† | +0.98%† | +2.07%† | +1.32% | +2.81% |
| vue-docs-ways-of-using-vue | reuse | 5883 | +1.54%† | +0.96% | +2.37% | +1.24% | +3.67% |
| comment-reproduction | fresh | 298 | +0.57% | +0.95% | +1.28% | +2.29% | +2.39% |
| rust-book-appendix-02-operators | reuse | 22595 | +0.96%† | +0.94% | +2.85% | -3.33% | +0.01% |
| vite-docs-performance | reuse | 8184 | +1.41%† | +0.94% | +1.99% | +0.78% | +2.78% |
| wiki-tea-plain-prose | fresh | 40577 | +1.25% | +0.85% | +1.54% | +1.26% | +2.05% |
| rust-book-ch17-00-async-await | reuse | 9734 | +0.58%† | +0.83% | +1.75% | -0.48% | +0.29% |
| wiki-chess-article-body | fresh | 113609 | +0.96%† | +0.81%† | +1.50%† | +0.58% | +2.46% |
| comment-unicode | reuse | 327 | +1.47% | +0.76% | +1.94% | +0.53% | +1.62% |
| legacy-docs-markdown-extensions | fresh | 3707 | +0.77%† | +0.70% | +1.22% | -0.01% | +2.94% |
| comment-links | fresh | 278 | +0.55% | +0.65% | +1.50% | -0.03% | +3.25% |
| legacy-node-ferromark-readme | reuse | 9075 | +0.70%† | +0.57% | +2.07% | -1.42% | +0.93% |
| legacy-docs-releasing | fresh | 4141 | +0.89%† | +0.54% | +1.71% | -1.12% | +0.88% |
| legacy-docs-migration-0-3 | reuse | 2379 | +0.65%† | +0.51% | +0.57% | -1.84% | +0.71% |
| legacy-docs-mdx | reuse | 7422 | +0.67%† | +0.49% | +1.82% | -0.12% | +0.67% |
| legacy-node-ferromark-readme | fresh | 9075 | +1.02%† | +0.45% | +1.89% | -1.66% | +2.08% |
| legacy-docs-migration-0-3 | fresh | 2379 | +1.65%† | +0.42% | +0.78% | -0.38% | +1.67% |
| rust-book-appendix-02-operators | fresh | 22595 | -0.09%† | +0.30% | +1.61% | -4.25% | -0.42% |
| rust-book-ch17-00-async-await | fresh | 9734 | -0.21%† | +0.27% | +0.96% | -0.31% | +0.91% |
| legacy-docs-releasing | reuse | 4141 | +1.02%† | +0.21% | +1.12% | -0.67% | -0.49% |
| rust-book-ch03-04-comments | fresh | 393 | -0.53%† | +0.04% | -0.44% | -2.06% | +0.86% |
| legacy-docs-migration-0-4 | fresh | 7045 | +1.45%† | +0.03% | +0.88% | -1.59% | +1.67% |
| legacy-docs-migration-0-4 | reuse | 7045 | +1.35%† | +0.01% | +1.68% | -0.58% | +0.93% |
| typescript-handbook-compiler-options | reuse | 54026 | -0.34%† | -0.01% | -1.14% | -0.74% | -0.46% |
| vite-docs-api-plugin | fresh | 31890 | -0.75%† | -0.06% | +1.09% | -1.35% | +1.62% |
| comment-inline-code | fresh | 285 | +0.30% | -0.37% | +0.63% | +2.01% | +2.36% |
| vite-docs-features | fresh | 39739 | -4.39%† | -0.75%† | -0.56% | -4.99% | -1.10% |
| legacy-docs-migration-0-2 | reuse | 1985 | -0.33%† | -0.75% | +0.23% | -1.49% | +0.08% |
| legacy-docs-migration-0-2 | fresh | 1985 | -0.06%† | -1.05% | +0.07% | -1.63% | +0.31% |
| typescript-handbook-compiler-options | fresh | 54026 | +0.43%† | -1.07% | -1.11% | -1.57% | -0.22% |
| vue-docs-reactivity-in-depth | fresh | 24001 | +0.40%† | -1.19% | +0.89% | -0.38% | +1.42% |
| wiki-tea-article-body | reuse | 58814 | -3.18%† | -2.09% | -1.57%† | -3.47% | -0.56% |
| wiki-volcano-article-body | reuse | 69241 | -2.92%† | -2.70%† | -2.04%† | -3.73% | -1.84% |
| typescript-handbook-advanced-types | reuse | 36745 | -6.95%† | -4.15% | -2.04% | -6.44% | -2.95% |
| typescript-handbook-advanced-types | fresh | 36745 | -2.41%† | -4.86% | -2.89% | -6.86% | -2.75% |
| wiki-volcano-article-body | fresh | 69241 | -5.56%† | -5.61%† | -4.52%† | -6.31% | -4.14% |
| wiki-tea-article-body | fresh | 58814 | -8.37%† | -8.35% | -8.31%† | -9.24% | -7.10% |
| vite-docs-features | reuse | 39739 | -11.01%† | -11.83%† | -9.27% | -13.29% | -9.47% |
| typescript-handbook-typescript-5-0 | reuse | 50714 | -14.00%† | -12.64% | -10.57% | -13.52% | -11.01% |
| typescript-handbook-typescript-5-0 | fresh | 50714 | -18.58%† | -13.70% | -12.50% | -15.53% | -12.72% |
