# Measured Suspense results

Generated from archived observations; negative paired changes mean less elapsed time. Repeated ranges are not confidence intervals.

## Measured PR candidate

Three fresh process pairs, 103 inputs, seven alternating 50 ms windows after 60 ms warmup. Absolute times are medians of the three process medians.

| Input | Before µs | After µs | Changes |
| --- | ---: | ---: | --- |
| ox-large-cm | 237.051 | 237.154 | -0.25%, +0.68%, -0.39% |
| ox-large-tables | 247.580 | 248.857 | -0.00%, +0.65%, -0.68% |
| ox-huge-tables | 5919.633 | 5889.820 | +0.45%, -1.32%, -0.34% |
| short | 0.489 | 0.485 | -0.80%, -2.92%, -3.04% |
| short-reuse | 0.193 | 0.193 | -0.22%, -0.33%, -0.66% |
| plain | 6.584 | 6.589 | +0.57%, -0.10%, +0.09% |
| multiline | 12.921 | 12.851 | +0.20%, -0.64%, -0.38% |
| inline | 38.950 | 38.571 | -1.04%, -0.97%, -0.81% |
| lists | 44.346 | 44.286 | +0.30%, -0.14%, -0.29% |
| code | 15.050 | 15.088 | -0.45%, +0.95%, +0.36% |
| tables | 40.503 | 40.432 | -0.62%, +0.05%, -0.14% |
| headings | 37.206 | 37.401 | -0.39%, +0.03%, +4.46% |
| unique-headings | 33.560 | 33.249 | -1.13%, -0.95%, -0.54% |
| long-prose | 7.693 | 7.691 | -0.03%, -0.34%, -0.25% |
| late-marker | 10.391 | 10.370 | -0.20%, +0.48%, -0.46% |
| long-code | 12.535 | 12.516 | -0.15%, -0.31%, -0.24% |
| escape-dense | 11.412 | 11.373 | +0.38%, -0.67%, -0.50% |
| commonmark/tiny | 0.467 | 0.457 | -1.26%, -1.90%, -2.07% |
| commonmark/short-100b | 0.858 | 0.846 | +1.47%, -1.35%, -2.63% |
| gfm_overlap/tiny | 0.466 | 0.457 | -1.10%, -1.94%, -1.79% |
| gfm_overlap/short-100b | 0.859 | 0.848 | +2.26%, -1.24%, -2.81% |
| tables/tables-plain | 24.962 | 24.939 | +0.99%, +0.27%, -0.09% |
| tables/tables-commonmark-inline | 31.771 | 31.634 | +0.36%, -0.43%, -0.20% |
| tables/tables-links | 36.899 | 36.769 | -0.74%, -0.03%, -0.23% |
| commonmark/publication-5k | 23.514 | 23.487 | +0.42%, -0.22%, -0.14% |
| guard/plain | 24.971 | 24.943 | -0.26%, +0.20%, -0.37% |
| guard/mixed | 31.694 | 31.560 | -0.27%, -0.23%, -0.42% |
| guard/links | 36.966 | 36.691 | -0.22%, +0.08%, -0.89% |
| guard/mixed-document | 17.757 | 17.656 | -0.01%, -0.11%, -0.78% |
| guard/one-table | 16.919 | 16.925 | -0.19%, +0.04%, +0.26% |
| guard/long-cells | 23.360 | 23.343 | +0.52%, +0.39%, -0.07% |
| guard/wide9 | 36.386 | 36.359 | -0.85%, -0.55%, -0.07% |
| guard/wide16 | 55.480 | 55.502 | +0.61%, -0.27%, -0.20% |
| guard/escapes-code | 66.203 | 65.270 | -0.51%, -1.56%, -0.53% |
| guard/prose | 9.491 | 9.547 | +0.70%, -0.35%, -0.50% |
| guard/links-prose | 35.719 | 35.580 | -0.44%, -0.39%, -0.52% |
| guard/edge-cases | 1330.223 | 1327.807 | -0.17%, -0.18%, -0.13% |
| guard/references | 10.395 | 10.244 | -1.47%, -1.62%, -1.45% |
| guard/empty | 0.142 | 0.138 | -1.13%, -3.37%, -3.91% |
| guard/gfm-document | 23.359 | 23.272 | -0.50%, -0.40%, -0.38% |
| guard/long-delimiters | 13.549 | 13.560 | -0.60%, +0.16%, +0.48% |
| guard/deep-emphasis | 38.498 | 38.430 | -0.22%, -0.37%, -0.18% |
| mdx/short | 1.191 | 1.172 | -2.14%, -0.97%, -1.57% |
| mdx/segments | 163.559 | 162.749 | -0.12%, -0.50%, -2.21% |
| mdx/tables-code | 146.418 | 145.287 | -0.42%, -0.59%, -1.72% |
| corpus/vue-docs/src/api/application.md | 54.790 | 53.860 | -0.34%, -0.86%, -1.70% |
| corpus/vue-docs/src/api/built-in-directives.md | 57.967 | 57.306 | -0.77%, -1.10%, -1.14% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 16.458 | 16.432 | -0.52%, -0.27%, -0.16% |
| corpus/vue-docs/src/guide/extras/render-function.md | 38.361 | 38.117 | -0.64%, -2.11%, -1.64% |
| corpus/vite-docs/docs/config/shared-options.md | 72.657 | 71.835 | -0.39%, -1.13%, -2.16% |
| corpus/vite-docs/docs/guide/api-plugin.md | 71.215 | 71.719 | +1.56%, +0.71%, -0.01% |
| corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | 100.112 | 98.760 | -2.86%, -0.33%, -1.35% |
| corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | 63.998 | 64.037 | -0.59%, +1.49%, +0.06% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 73.789 | 74.709 | +1.29%, -1.38%, +2.00% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 16.097 | 15.907 | -0.28%, -2.80%, -1.41% |
| corpus/typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 27.200 | 27.151 | +0.55%, -0.10%, -0.37% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 95.444 | 95.056 | -1.02%, -0.41%, +0.83% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 79.034 | 82.034 | +2.05%, +3.95%, +2.12% |
| corpus/concat/vue-docs/matched | 2633.910 | 2601.091 | -0.29%, -1.47%, +0.10% |
| corpus/concat/vue-docs/upstream | 2936.109 | 2914.637 | -0.64%, -0.73%, -0.58% |
| corpus/concat/vite-docs/matched | 1422.123 | 1416.545 | +0.05%, +0.13%, -0.39% |
| corpus/concat/vite-docs/upstream | 1692.047 | 1707.234 | +0.90%, -0.33%, -0.82% |
| corpus/concat/rust-book/matched | 4176.919 | 4153.698 | -0.17%, -0.34%, -1.63% |
| corpus/concat/rust-book/upstream | 4624.490 | 4612.570 | -0.57%, -0.32%, +0.20% |
| corpus/concat/typescript-handbook/matched | 5041.037 | 5057.560 | +0.33%, -0.00%, -0.80% |
| corpus/concat/typescript-handbook/upstream | 5687.857 | 5673.760 | -0.69%, -0.25%, +0.20% |
| corpus/ox-parser/SIMPLE_MD/1 | 1.519 | 1.511 | -0.40%, -0.03%, -0.55% |
| corpus/ox-parser/LARGE_MD/1 | 8.077 | 8.029 | +0.13%, -0.32%, -0.59% |
| corpus/ox-parser/LARGE_MD/100 | 735.202 | 735.097 | -0.01%, -0.12%, -0.06% |
| html/root-short-lines | 35.970 | 35.797 | +0.19%, -1.34%, -0.22% |
| html/root-long-lines | 2.808 | 2.805 | +0.42%, +7.93%, -1.70% |
| html/separate-blocks | 156.961 | 156.821 | +0.09%, -0.34%, -0.88% |
| html/comment-long-lines | 3.652 | 3.607 | -1.24%, -0.39%, +0.60% |
| html/script-long-lines | 3.586 | 3.570 | +0.12%, -0.71%, -1.63% |
| html/script-candidates | 57.299 | 57.149 | +0.16%, +0.03%, -0.26% |
| html/comment-short-lines | 59.134 | 59.515 | +0.17%, -0.13%, +1.10% |
| html/processing-long-lines | 3.589 | 3.573 | +0.46%, -0.45%, -0.73% |
| html/cdata-long-lines | 3.595 | 3.596 | +0.01%, +1.42%, -0.26% |
| html/declaration-long-lines | 3.546 | 3.548 | -0.15%, -0.26%, +0.73% |
| html/container-html | 132.479 | 132.738 | -0.36%, +0.36%, -0.24% |
| html/custom-tags | 190.544 | 190.189 | +0.41%, +0.03%, -0.32% |
| fence/root-short | 196.179 | 196.694 | -0.40%, +0.41%, -0.40% |
| fence/root-long | 5.646 | 5.611 | -0.62%, -0.69%, -0.57% |
| fence/root-indented | 193.716 | 193.511 | -0.36%, +0.00%, -0.14% |
| fence/root-tabs | 183.251 | 182.454 | +0.60%, +0.09%, -0.48% |
| fence/root-container | 224.821 | 223.846 | -0.66%, -0.43%, +0.22% |
| fence/root-crlf | 184.072 | 184.709 | +0.10%, +0.46%, -0.45% |
| utf8/paragraph-0 | 9.353 | 9.402 | +0.41%, +0.55%, -0.13% |
| utf8/paragraph-32768 | 8.586 | 8.567 | +0.02%, +0.21%, -0.81% |
| utf8/paragraph-65536 | 7.750 | 7.733 | +0.98%, -0.79%, -0.22% |
| utf8/html-0 | 5.095 | 5.070 | +0.50%, -0.58%, -1.05% |
| utf8/html-32768 | 4.365 | 4.320 | -0.40%, -0.72%, -1.38% |
| utf8/html-65536 | 3.532 | 3.532 | +1.09%, -0.19%, -1.10% |
| target/tag-code-prose | 64.931 | 28.779 | -55.41%, -55.69%, -55.73% |
| target/plain-code-prose | 18.816 | 18.633 | -1.01%, -0.94%, -1.54% |
| target/html-prose | 12.769 | 12.742 | +0.17%, -0.34%, -0.41% |
| target/uri-autolinks | 29.768 | 29.668 | -0.11%, -0.55%, -0.34% |
| target/email-autolinks | 34.673 | 34.727 | -0.36%, +0.16%, +0.08% |
| target/code-and-autolinks | 70.602 | 40.449 | -42.99%, -42.76%, -42.67% |
| target/code-with-escapes | 30.394 | 30.331 | +0.07%, -0.95%, -0.38% |
| target/code-with-entities | 37.207 | 37.038 | -0.15%, -0.70%, -1.20% |
| target/code-html-attributes | 145.875 | 60.946 | -58.28%, -58.02%, -58.13% |
| target/dense-code | 57.139 | 56.661 | -0.21%, -0.35%, -1.38% |

### Repeated small costs

Every input at least 1% slower in all three final pairs is listed here. The complete table also retains isolated losses.

| Input | Changes |
| --- | --- |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | +2.05%, +3.95%, +2.12% |

### Focused follow-up

Three fresh pairs, seven alternating 100 ms windows. These include every input over 2% slower in any final pair, the heading control, Suspense and HTML prose. Earlier observations remain above.

| Input | Before µs | After µs | Changes |
| --- | ---: | ---: | --- |
| headings | 37.688 | 39.635 | +0.48%, +5.17%, +8.12% |
| gfm_overlap/short-100b | 0.876 | 0.872 | -0.75%, +1.05%, -0.59% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | 16.495 | 16.504 | -0.72%, -0.31%, +0.05% |
| corpus/rust-book/src/ch21-02-multithreaded.md | 75.564 | 74.646 | -0.53%, +1.39%, -2.32% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 79.321 | 81.015 | +2.43%, +2.62%, +0.62% |
| html/root-long-lines | 2.812 | 2.822 | -0.18%, +0.74%, +0.36% |
| target/html-prose | 12.792 | 12.805 | +0.03%, -0.28%, +0.10% |

Inputs over 2% slower in all three final pairs: 1.

## Fresh native Ox comparison

Growing arena only. Three fresh process groups, nine alternating 60 ms windows after 35 ms warmup. These use a separate driver from the before/after measurements; do not combine their absolute timings.

| Document | Ferromark µs | Ox growing µs | Ferromark extra time, three groups |
| --- | ---: | ---: | --- |
| vue-docs/src/api/application.md | 53.738 | 63.206 | -14.98%, -15.67%, -15.20% |
| vue-docs/src/api/built-in-directives.md | 57.130 | 66.564 | -14.15%, -14.16%, -15.20% |
| vue-docs/src/guide/built-ins/suspense.md | 16.333 | 10.365 | +57.53%, +57.58%, +59.45% |
| vue-docs/src/guide/extras/render-function.md | 37.369 | 31.189 | +19.81%, +19.20%, +19.03% |
| vite-docs/docs/config/shared-options.md | 70.571 | 65.036 | +8.77%, +8.51%, +8.58% |
| vite-docs/docs/guide/api-plugin.md | 70.826 | 74.146 | -5.39%, -4.30%, -6.58% |
| rust-book/src/ch02-00-guessing-game-tutorial.md | 99.253 | 69.481 | +42.85%, +42.24%, +41.27% |
| rust-book/src/ch09-02-recoverable-errors-with-result.md | 61.931 | 39.497 | +54.76%, +58.72%, +57.37% |
| rust-book/src/ch21-02-multithreaded.md | 72.883 | 51.423 | +42.99%, +40.34%, +42.82% |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 15.949 | 14.376 | +10.65%, +11.21%, +11.57% |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 27.282 | 26.575 | +2.66%, +2.33%, +2.31% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 94.092 | 72.895 | +29.08%, +25.90%, +28.92% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 78.312 | 59.689 | +31.20%, +33.37%, +33.06% |
| ox-parser/SIMPLE_MD/1 | 1.513 | 1.268 | +19.28%, +18.92%, +20.53% |
| ox-parser/LARGE_MD/1 | 8.001 | 7.098 | +12.71%, +11.34%, +13.79% |
| ox-parser/LARGE_MD/100 | 727.666 | 688.836 | +5.35%, +5.55%, +5.64% |

649 corpus outputs checked; 594 normalize equally. The other cases remain explicit diagnostics and do not support a ranking.

## Growth controls

Two fresh pairs, seven alternating 50 ms windows after 35 ms warmup. Each control has exact before/after HTML equality. Repetition counts refer to one paragraph, not separate renders.

| Input | Before µs | After µs | Changes |
| --- | ---: | ---: | --- |
| scaling/tag-code/40 | 9.979 | 8.044 | -19.39%, -19.38% |
| scaling/mixed/40 | 20.140 | 16.848 | -15.97%, -16.72% |
| scaling/tag-code/80 | 22.983 | 14.912 | -35.15%, -35.09% |
| scaling/mixed/80 | 47.506 | 32.674 | -31.44%, -31.00% |
| scaling/tag-code/160 | 64.375 | 28.726 | -55.33%, -55.42% |
| scaling/mixed/160 | 124.327 | 65.336 | -47.42%, -47.47% |
| scaling/tag-code/320 | 193.316 | 56.371 | -70.68%, -70.99% |
| scaling/mixed/320 | 357.707 | 130.987 | -63.41%, -63.36% |
| scaling/tag-code/640 | 647.235 | 110.854 | -82.82%, -82.92% |
| scaling/mixed/640 | 991.910 | 246.907 | -75.16%, -75.06% |
| scaling/tag-code/1280 | 1711.806 | 191.194 | -88.85%, -88.81% |
| scaling/mixed/1280 | 2016.797 | 418.045 | -79.34%, -79.20% |

## Initial independent screens

Five alternating 30 ms windows. These exploratory results are not adoption evidence by themselves. Full control results and source patches accompany each screen.

| Variant | Inputs | Suspense change | Inputs over 2% slower |
| --- | ---: | ---: | ---: |
| autolink-code | 93 | -4.12% | 3 |
| autolink-fused | 103 | -1.48% | 2 |
| code-emit | 103 | -3.92% | 4 |
| code-prose | 103 | -9.94% | 6 |
| code-prose-outlined | 103 | -11.41% | 5 |
| linear-autolink | 103 | -5.72% | 6 |
| linear-prose | 103 | -8.55% | 5 |
| linear-ranges | 103 | +1.15% | 1 |
| linear-tag-prose | 103 | -9.59% | 4 |

## Identical-binary controls

Three fresh pairs compare byte-identical baseline binaries using the same seven-window protocol. These observations diagnose measurement variability; they do not subtract noise from candidate results.

| Input | Changes |
| --- | --- |
| headings | +0.14%, -0.03%, -0.08% |
| corpus/vue-docs/src/guide/built-ins/suspense.md | +0.38%, +0.22%, +0.02% |
| html/script-long-lines | +0.30%, -0.63%, +0.09% |
| target/html-prose | -0.05%, +0.07%, -0.36% |
| corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | -1.65%, -0.20%, -0.96% |

## Exact-output checks for the PR candidate

| Guard | Comparisons | Failures |
| --- | ---: | ---: |
| block-verification.json | 20,740 | 0 |
| extended-verification.json | 60,000 | 0 |
| extra-verification.json | 16,374 | 0 |
| fence-verification.json | 12,000 | 0 |
| precedence-verification.json | 20,000 | 0 |
| timing-verification.json | 103 | 0 |
| verification.json | 2,695 | 0 |

Total: 131,912 logical comparisons. The whole-render work test additionally proves that membership searches no longer exhibit the recorded quadratic growth.
