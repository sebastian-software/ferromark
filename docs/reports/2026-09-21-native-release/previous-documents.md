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
| comment-checklist | reuse | 287 | +53.05% | +53.57% | +52.02%† | +53.17% | +58.97%† |
| comment-quote | reuse | 290 | +49.95% | +50.13% | +47.46% | +48.16% | +57.54% |
| comment-checklist | fresh | 287 | +45.20% | +47.18% | +45.93%† | +47.93% | +53.32%† |
| comment-review-long | reuse | 957 | +41.72% | +40.01% | +39.91% | +39.12% | +49.99% |
| comment-quote | fresh | 290 | +40.13% | +37.82% | +36.67% | +39.68% | +47.09% |
| comment-review-long | fresh | 957 | +36.47% | +35.67% | +35.77% | +36.74% | +46.17% |
| typescript-handbook-the-handbook | fresh | 5337 | +34.38%† | +34.44% | +34.77% | +34.84% | +42.04% |
| typescript-handbook-the-handbook | reuse | 5337 | +36.74%† | +33.91% | +36.08% | +36.90% | +43.17% |
| comment-incident | reuse | 1124 | +30.71%† | +31.04% | +30.88% | +30.95% | +40.01% |
| comment-incident | fresh | 1124 | +28.73%† | +29.57% | +29.41% | +29.03% | +35.91% |
| vite-docs-api-plugin | fresh | 31890 | +26.64%† | +28.16% | +27.24% | +27.56% | +30.79% |
| vite-docs-api-plugin | reuse | 31890 | +27.33%† | +24.92% | +25.14% | +26.75% | +26.99% |
| comment-ack | reuse | 37 | +18.79% | +19.56% | +17.92% | +20.54% | +22.54% |
| legacy-docs-releasing | reuse | 4141 | +20.00%† | +19.08% | +19.58% | +21.05% | +24.94% |
| wiki-volcano-plain-prose | fresh | 47303 | +16.64% | +18.99% | +18.78% | +19.32% | +23.77% |
| legacy-docs-releasing | fresh | 4141 | +18.88%† | +18.59% | +20.23% | +19.10% | +24.37% |
| wiki-chess-plain-prose | fresh | 80966 | +17.29% | +18.49% | +18.28% | +17.31% | +22.01% |
| vue-docs-ways-of-using-vue | reuse | 5883 | +18.72%† | +18.18% | +17.53% | +18.21% | +26.21% |
| wiki-volcano-plain-prose | reuse | 47303 | +17.42% | +17.82% | +17.84% | +18.04% | +21.75% |
| wiki-chess-plain-prose | reuse | 80966 | +16.33% | +16.85% | +16.15% | +16.02% | +19.68% |
| vue-docs-ways-of-using-vue | fresh | 5883 | +17.05%† | +16.85% | +17.10% | +17.93% | +24.20% |
| vite-docs-features | reuse | 39739 | +14.48%† | +16.13%† | +14.91% | +15.18% | +15.08% |
| comment-ack | fresh | 37 | +12.91% | +15.10% | +13.12% | +18.50% | +17.61% |
| comment-question | reuse | 160 | +14.31% | +15.04% | +12.31% | +13.88% | +20.61% |
| vite-docs-performance | fresh | 8184 | +15.30%† | +14.50% | +14.95% | +15.11% | +21.09% |
| vite-docs-performance | reuse | 8184 | +15.60%† | +12.60% | +14.45% | +14.21% | +19.23% |
| vite-docs-features | fresh | 39739 | +12.31%† | +11.47%† | +9.77% | +11.01% | +11.92% |
| comment-review | reuse | 282 | +9.84% | +10.77% | +8.75% | +10.74% | +15.66% |
| vite-docs-philosophy | reuse | 3575 | +10.87%† | +10.46% | +11.00% | +11.43% | +17.04% |
| vite-docs-philosophy | fresh | 3575 | +9.64%† | +10.02% | +9.95% | +11.35% | +15.17% |
| legacy-contributing | fresh | 9323 | +10.77%† | +9.42% | +11.13% | +11.74% | +16.20% |
| legacy-contributing | reuse | 9323 | +10.45%† | +9.29% | +11.20% | +9.80% | +14.29% |
| rust-book-ch17-00-async-await | reuse | 9734 | +8.28%† | +9.05% | +10.09% | +8.79% | +13.46% |
| comment-question | fresh | 160 | +7.89% | +8.65% | +8.35% | +8.67% | +14.72% |
| vue-docs-reactivity-in-depth | fresh | 24001 | +6.64%† | +8.34% | +7.34% | +7.88% | +9.97% |
| vue-docs-reactivity-in-depth | reuse | 24001 | +5.81%† | +8.21% | +7.32% | +7.05% | +10.53% |
| rust-book-ch17-00-async-await | fresh | 9734 | +8.56%† | +7.92% | +9.05% | +8.74% | +13.98% |
| wiki-tea-plain-prose | reuse | 40577 | +7.89% | +7.70% | +7.03% | +6.60% | +10.91% |
| legacy-docs-migration-0-3 | reuse | 2379 | +8.30%† | +7.46% | +8.97% | +7.33% | +12.65% |
| legacy-docs-migration-0-3 | fresh | 2379 | +7.25%† | +7.06% | +7.64% | +6.73% | +12.14% |
| wiki-chess-article-body | fresh | 113609 | +7.88%† | +6.84%† | +6.67%† | +8.07% | +10.71% |
| comment-review | fresh | 282 | +7.48% | +6.84% | +7.47% | +9.00% | +14.26% |
| wiki-tea-plain-prose | fresh | 40577 | +6.56% | +6.74% | +7.14% | +6.72% | +10.25% |
| rust-book-ch00-00-introduction | reuse | 10839 | +6.52%† | +6.69% | +8.11% | +7.04% | +11.79% |
| wiki-chess-first-paragraph | reuse | 1190 | +6.03%† | +6.57% | +5.96% | +5.98% | +10.91% |
| comment-inline-code | reuse | 285 | +5.44% | +6.56% | +5.49% | +6.07% | +10.75% |
| wiki-tea-lead | reuse | 6363 | +5.66%† | +6.37% | +5.67% | +5.88% | +10.05% |
| comment-reproduction | reuse | 298 | +6.73% | +6.18% | +7.51% | +8.35% | +9.52% |
| wiki-tea-lead | fresh | 6363 | +5.69%† | +6.11% | +4.73% | +5.17% | +9.10% |
| wiki-volcano-article-body | reuse | 69241 | +8.80%† | +5.98%† | +5.19%† | +5.30% | +9.19% |
| comment-unicode | reuse | 327 | +5.80% | +5.78% | +5.35% | +7.96% | +11.49% |
| wiki-tea-first-paragraph | reuse | 1126 | +6.35%† | +5.69% | +4.50% | +5.82% | +9.35% |
| wiki-volcano-article-body | fresh | 69241 | +7.38%† | +5.60%† | +6.17%† | +6.81% | +8.71% |
| wiki-rainbow-first-paragraph | reuse | 859 | +6.32%† | +5.39% | +5.44% | +5.48% | +9.85% |
| wiki-volcano-first-paragraph | reuse | 2644 | +5.45%† | +5.29% | +5.83% | +5.87% | +11.48% |
| wiki-rainbow-article-body | fresh | 48422 | +6.92%† | +5.23%† | +3.97%† | +3.88% | +8.04% |
| guard-angle-link | reuse | 41 | +6.13% | +5.18%† | +4.66% | +8.44% | +11.37% |
| wiki-rainbow-lead | reuse | 1859 | +6.24%† | +5.01% | +5.41% | +5.51% | +11.61% |
| typescript-handbook-typescript-5-0 | fresh | 50714 | +4.36%† | +5.01% | +5.80% | +7.39% | +8.47% |
| wiki-chess-lead | fresh | 4125 | +4.52%† | +4.97% | +4.59% | +5.60% | +8.91% |
| wiki-tea-first-paragraph | fresh | 1126 | +5.05%† | +4.90% | +3.96% | +5.52% | +9.15% |
| comment-inline-code | fresh | 285 | +5.06% | +4.85% | +4.87% | +7.07% | +11.30% |
| wiki-volcano-lead | fresh | 4232 | +4.56%† | +4.84% | +4.37% | +4.55% | +10.20% |
| typescript-handbook-typescript-5-0 | reuse | 50714 | +5.87%† | +4.72% | +4.67% | +6.13% | +7.27% |
| wiki-chess-article-body | reuse | 113609 | +5.41%† | +4.72%† | +5.75%† | +5.52% | +9.23% |
| wiki-volcano-first-paragraph | fresh | 2644 | +4.39%† | +4.71% | +4.88% | +4.70% | +11.16% |
| comment-reproduction | fresh | 298 | +4.44% | +4.63% | +4.93% | +6.08% | +8.81% |
| comment-unicode | fresh | 327 | +3.17% | +4.59% | +3.39% | +5.32% | +10.08% |
| wiki-volcano-lead | reuse | 4232 | +5.14%† | +4.38% | +4.56% | +4.18% | +10.76% |
| rust-book-ch00-00-introduction | fresh | 10839 | +5.26%† | +4.33% | +6.13% | +5.13% | +10.28% |
| wiki-rainbow-article-body | reuse | 48422 | +5.78%† | +4.33%† | +3.98%† | +4.18% | +7.18% |
| wiki-chess-lead | reuse | 4125 | +5.08%† | +4.24% | +5.08% | +5.13% | +10.00% |
| wiki-rainbow-lead | fresh | 1859 | +4.70%† | +4.12% | +4.27% | +4.31% | +10.08% |
| wiki-tea-article-body | fresh | 58814 | +7.79%† | +3.89% | +6.08%† | +6.19% | +9.66% |
| wiki-rainbow-first-paragraph | fresh | 859 | +4.36%† | +3.67% | +3.68% | +5.33% | +9.14% |
| vue-docs-suspense | reuse | 8291 | +3.38%† | +3.51% | +3.36% | +4.33% | +9.22% |
| guard-angle-link | fresh | 41 | +2.37% | +3.09%† | +1.52% | +6.06% | +8.64% |
| comment-table | reuse | 310 | +2.62% | +3.02% | +3.58% | +3.61% | +6.59% |
| legacy-docs-readme-theme | fresh | 2848 | +3.34%† | +3.01% | +2.56% | +4.01% | +8.40% |
| vue-docs-suspense | fresh | 8291 | +3.62%† | +2.94% | +4.15% | +3.69% | +9.47% |
| rust-book-ch03-04-comments | fresh | 393 | +3.61%† | +2.93% | +2.60% | +5.25% | +7.47% |
| vue-docs-slots | fresh | 24211 | +2.26%† | +2.80% | +2.73% | +2.74% | +5.68% |
| legacy-docs-adr-readme-theme-composition | fresh | 1532 | +2.57%† | +2.74% | +2.38% | +4.93% | +8.84% |
| typescript-handbook-advanced-types | reuse | 36745 | +2.08%† | +2.49% | +3.44% | +3.59% | +5.55% |
| comment-links | fresh | 278 | +3.13% | +2.29% | +2.48% | +3.60% | +8.73% |
| wiki-chess-first-paragraph | fresh | 1190 | +3.73%† | +2.19% | +3.63% | +4.66% | +9.51% |
| legacy-docs-adr-readme-theme-composition | reuse | 1532 | +2.57%† | +2.16% | +3.02% | +3.61% | +8.35% |
| wiki-rainbow-plain-prose | reuse | 38800 | +1.32% | +2.04% | +2.59% | +2.05% | +6.35% |
| comment-table | fresh | 310 | +1.44% | +1.91% | +1.33% | +2.45% | +5.22% |
| legacy-docs-mdx | reuse | 7422 | +3.19%† | +1.90% | +3.53% | +3.20% | +7.53% |
| rust-book-ch03-04-comments | reuse | 393 | +3.39%† | +1.87% | +2.42% | +2.56% | +8.60% |
| legacy-docs-markdown-extensions | fresh | 3707 | +2.14%† | +1.82% | +2.28% | +3.29% | +8.54% |
| legacy-docs-migration-0-8 | reuse | 2374 | +2.51%† | +1.67% | +2.72% | +3.44% | +7.00% |
| legacy-docs-readme | reuse | 1825 | +2.37%† | +1.59% | +2.35% | +4.58% | +6.47% |
| legacy-docs-readme | fresh | 1825 | +2.05%† | +1.36% | +2.61% | +3.79% | +6.71% |
| wiki-rainbow-plain-prose | fresh | 38800 | +1.80% | +1.34% | +3.03% | +2.35% | +6.19% |
| legacy-docs-readme-theme | reuse | 2848 | +2.19%† | +1.28% | +1.28% | +2.56% | +8.83% |
| legacy-docs-markdown-extensions | reuse | 3707 | +1.59%† | +1.24% | +2.33% | +2.79% | +7.51% |
| comment-links | reuse | 278 | +2.41% | +1.23% | +1.50% | +3.28% | +7.80% |
| typescript-handbook-advanced-types | fresh | 36745 | +0.97%† | +1.23% | +1.19% | -0.39% | +2.80% |
| legacy-docs-migration-0-8 | fresh | 2374 | +2.41%† | +1.17% | +1.19% | +2.82% | +6.95% |
| legacy-docs-mdx | fresh | 7422 | +1.20%† | +1.08% | +2.39% | +2.89% | +7.27% |
| legacy-node-ferromark-readme | fresh | 9075 | +2.01%† | +0.98% | +3.03% | +3.30% | +8.12% |
| vue-docs-slots | reuse | 24211 | +1.68%† | +0.90% | +2.24% | +1.83% | +4.47% |
| wiki-tea-article-body | reuse | 58814 | +3.83%† | +0.90% | +1.41%† | +1.94% | +3.68% |
| legacy-docs-migration-0-2 | fresh | 1985 | +0.46%† | +0.42% | +0.79% | +0.93% | +5.58% |
| legacy-node-ferromark-readme | reuse | 9075 | +1.15%† | +0.32% | +1.05% | +2.68% | +6.86% |
| legacy-docs-migration-0-4 | fresh | 7045 | +0.10%† | +0.31% | +1.67% | +1.04% | +5.27% |
| legacy-docs-migration-0-4 | reuse | 7045 | +0.48%† | -0.08% | +0.66% | +2.09% | +5.66% |
| legacy-docs-migration-0-2 | reuse | 1985 | -0.75%† | -0.65% | +0.80% | +2.13% | +5.19% |
| typescript-handbook-compiler-options | fresh | 54026 | +0.34%† | -0.87% | +0.40% | +1.18% | +0.27% |
| typescript-handbook-compiler-options | reuse | 54026 | +0.09%† | -1.58% | +0.13% | -0.42% | +0.00% |
| rust-book-appendix-02-operators | reuse | 22595 | -0.52%† | -2.06% | -0.28% | +1.92% | +1.91% |
| rust-book-appendix-02-operators | fresh | 22595 | -0.77%† | -2.72% | -0.15% | -1.11% | +1.53% |
