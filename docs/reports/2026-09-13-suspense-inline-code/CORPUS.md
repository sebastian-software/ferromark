# Distribution across 638 individual documents

Three fresh process pairs per document, five alternating 20 ms windows after 20 ms warmup. Same native before/after operation and frozen inputs as the focused comparisons. This includes every individual document in the corpus, regardless of cross-engine output equality; concatenations and synthetic Ox controls are excluded. Before/after output equality is checked separately.

Changes are the median of the three paired percentage changes. The ±1% band is descriptive, not a confidence interval or significance test. Small changes must not be presented as reliably faster rendering.

| Project | Documents | Lower median | >1% faster | Within ±1% | >1% slower | >2% slower |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rust-book | 375 | 242 | 47 | 310 | 18 | 1 |
| typescript-handbook | 133 | 70 | 10 | 102 | 21 | 5 |
| vite-docs | 33 | 20 | 6 | 23 | 4 | 0 |
| vue-docs | 97 | 83 | 33 | 62 | 2 | 0 |
| all | 638 | 415 | 96 | 497 | 45 | 6 |

| Project | Lower in all three pairs | >1% slower in all three | Geometric mean change | Sum of before medians ms | Sum of after medians ms | Change in sum |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rust-book | 72 | 1 | -0.22% | 3.016 | 3.016 | -0.03% |
| typescript-handbook | 26 | 5 | +0.09% | 3.255 | 3.265 | +0.29% |
| vite-docs | 11 | 0 | -0.21% | 1.026 | 1.027 | +0.07% |
| vue-docs | 47 | 0 | -0.69% | 1.826 | 1.816 | -0.59% |
| all | 156 | 6 | -0.23% | 9.124 | 9.123 | -0.02% |

The geometric mean weights documents equally. The sum describes one render of each document using measured per-document medians; it is not a separately timed end-to-end build and excludes loading and setup. Neither assumes that this corpus matches a user’s workload.

## Longer checks of repeated costs

Every document over 1% slower in all three broad pairs receives three fresh seven-window comparisons at 100 ms per window, after 60 ms warmup. These retain the initial observations and do not replace them.

| Document | Broad changes | Longer changes |
| --- | --- | --- |
| rust-book/src/ch20-00-advanced-features.md | +1.62%, +1.15%, +1.04% | +0.78%, +1.15%, +0.65% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Introduction.md | +3.23%, +1.04%, +1.73% | +0.16%, +0.30%, +0.74% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/More on Functions.md | +3.01%, +1.13%, +1.03% | +0.82%, -1.52%, +0.67% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/appendices/ESM-CJS-Interop.md | +3.15%, +3.66%, +1.69% | +1.14%, +0.99%, +1.85% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.3.md | +5.68%, +1.94%, +1.73% | +1.00%, +1.67%, +1.48% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.4.md | +1.72%, +2.32%, +3.27% | +1.15%, +0.93%, +2.07% |

## All individual results

| Document | Before µs | After µs | Three paired changes |
| --- | ---: | ---: | --- |
| vue-docs/src/api/application.md | 55.955 | 55.112 | -2.16%, -1.51%, -0.92% |
| vue-docs/src/api/built-in-components.md | 30.101 | 29.634 | -2.72%, -1.28%, -1.20% |
| vue-docs/src/api/built-in-directives.md | 59.672 | 58.759 | -0.22%, -2.12%, +0.17% |
| vue-docs/src/api/built-in-special-attributes.md | 12.364 | 12.304 | -1.97%, +0.14%, -0.97% |
| vue-docs/src/api/built-in-special-elements.md | 16.838 | 16.677 | -1.60%, -1.27%, -0.30% |
| vue-docs/src/api/compile-time-flags.md | 7.213 | 7.184 | -0.88%, -0.69%, +1.87% |
| vue-docs/src/api/component-instance.md | 30.179 | 29.800 | -1.96%, -2.24%, -0.46% |
| vue-docs/src/api/composition-api-dependency-injection.md | 10.896 | 10.781 | -1.49%, -0.43%, -0.83% |
| vue-docs/src/api/composition-api-helpers.md | 13.226 | 13.039 | -1.66%, -1.41%, -1.59% |
| vue-docs/src/api/composition-api-lifecycle.md | 28.457 | 28.352 | -1.71%, +0.25%, -1.96% |
| vue-docs/src/api/composition-api-setup.md | 9.982 | 9.905 | +0.45%, -0.77%, -0.18% |
| vue-docs/src/api/custom-elements.md | 8.952 | 8.930 | -0.69%, -0.25%, -0.16% |
| vue-docs/src/api/custom-renderer.md | 5.204 | 5.194 | -1.28%, +0.59%, +1.21% |
| vue-docs/src/api/general.md | 15.438 | 15.071 | -1.09%, -2.37%, -2.29% |
| vue-docs/src/api/index.md | 0.992 | 0.995 | -0.71%, +2.29%, -0.63% |
| vue-docs/src/api/options-composition.md | 19.173 | 19.032 | -2.93%, -0.73%, -0.64% |
| vue-docs/src/api/options-lifecycle.md | 30.882 | 30.587 | -0.64%, -0.95%, -1.71% |
| vue-docs/src/api/options-misc.md | 14.558 | 14.395 | -2.08%, -1.91%, +0.39% |
| vue-docs/src/api/options-rendering.md | 10.692 | 10.492 | -1.57%, -1.87%, -1.06% |
| vue-docs/src/api/options-state.md | 37.805 | 37.443 | -1.15%, -1.17%, -0.71% |
| vue-docs/src/api/reactivity-advanced.md | 29.881 | 29.656 | -0.64%, -0.75%, -1.52% |
| vue-docs/src/api/reactivity-core.md | 44.226 | 44.150 | -1.11%, -0.17%, -0.37% |
| vue-docs/src/api/reactivity-utilities.md | 20.721 | 20.617 | -1.85%, -0.50%, -1.07% |
| vue-docs/src/api/render-function.md | 22.108 | 21.835 | -1.90%, -1.23%, -2.40% |
| vue-docs/src/api/sfc-css-features.md | 12.114 | 11.952 | -0.58%, -1.34%, -0.97% |
| vue-docs/src/api/sfc-script-setup.md | 39.154 | 38.756 | -0.84%, +0.35%, -3.24% |
| vue-docs/src/api/sfc-spec.md | 16.039 | 16.048 | -0.99%, +0.05%, +0.63% |
| vue-docs/src/api/ssr.md | 20.027 | 19.827 | -1.39%, -1.00%, -1.02% |
| vue-docs/src/api/utility-types.md | 17.083 | 17.141 | +0.42%, +1.05%, -1.55% |
| vue-docs/src/guide/best-practices/accessibility.md | 49.542 | 49.961 | -0.61%, +1.31%, -0.49% |
| vue-docs/src/guide/best-practices/performance.md | 19.876 | 20.089 | +0.65%, +1.07%, +0.68% |
| vue-docs/src/guide/best-practices/production-deployment.md | 6.617 | 6.620 | +1.59%, +0.05%, +0.30% |
| vue-docs/src/guide/best-practices/security.md | 13.442 | 13.402 | -0.35%, -0.29%, -0.92% |
| vue-docs/src/guide/built-ins/keep-alive.md | 14.149 | 13.847 | -2.41%, -2.14%, -2.33% |
| vue-docs/src/guide/built-ins/suspense.md | 16.829 | 16.624 | -0.62%, -1.21%, -1.55% |
| vue-docs/src/guide/built-ins/teleport.md | 13.708 | 13.633 | -0.02%, -1.09%, +0.20% |
| vue-docs/src/guide/built-ins/transition-group.md | 12.120 | 11.990 | -0.51%, -1.07%, -1.25% |
| vue-docs/src/guide/built-ins/transition.md | 53.767 | 54.200 | +0.54%, +1.09%, -0.27% |
| vue-docs/src/guide/components/async.md | 9.883 | 9.849 | -1.20%, +0.63%, -0.36% |
| vue-docs/src/guide/components/attrs.md | 17.308 | 17.184 | -0.72%, -0.46%, -0.83% |
| vue-docs/src/guide/components/events.md | 14.525 | 14.391 | -1.18%, -1.96%, +34.55% |
| vue-docs/src/guide/components/props.md | 39.149 | 38.765 | +0.54%, -0.98%, -0.23% |
| vue-docs/src/guide/components/provide-inject.md | 20.185 | 20.254 | -1.09%, +0.73%, -0.49% |
| vue-docs/src/guide/components/registration.md | 7.867 | 7.844 | -0.07%, -0.30%, +0.82% |
| vue-docs/src/guide/components/slots.md | 44.688 | 45.074 | -1.38%, -0.40%, +1.51% |
| vue-docs/src/guide/components/v-model.md | 35.837 | 35.451 | -1.22%, -1.08%, -2.51% |
| vue-docs/src/guide/essentials/application.md | 6.490 | 6.397 | -1.43%, -1.89%, -1.28% |
| vue-docs/src/guide/essentials/class-and-style.md | 17.792 | 17.772 | -0.37%, -0.94%, +0.63% |
| vue-docs/src/guide/essentials/component-basics.md | 47.038 | 46.525 | -0.88%, +0.93%, -1.47% |
| vue-docs/src/guide/essentials/computed.md | 16.895 | 16.842 | +0.41%, -0.53%, -1.08% |
| vue-docs/src/guide/essentials/conditional.md | 10.811 | 10.787 | -0.19%, +0.11%, -1.09% |
| vue-docs/src/guide/essentials/event-handling.md | 27.368 | 27.234 | -0.65%, +0.24%, -1.15% |
| vue-docs/src/guide/essentials/forms.md | 37.113 | 36.903 | -0.57%, -0.04%, -0.47% |
| vue-docs/src/guide/essentials/lifecycle.md | 7.149 | 7.041 | -0.54%, -0.76%, -1.51% |
| vue-docs/src/guide/essentials/list.md | 29.868 | 29.812 | -1.07%, -0.27%, -0.18% |
| vue-docs/src/guide/essentials/reactivity-fundamentals.md | 34.561 | 34.587 | -1.00%, -0.10%, +1.64% |
| vue-docs/src/guide/essentials/template-refs.md | 17.218 | 16.982 | -1.58%, -1.46%, -1.12% |
| vue-docs/src/guide/essentials/template-syntax.md | 22.183 | 22.184 | +0.19%, +0.64%, -0.99% |
| vue-docs/src/guide/essentials/watchers.md | 29.428 | 29.250 | -0.67%, -0.44%, -0.61% |
| vue-docs/src/guide/extras/animation.md | 8.982 | 8.896 | -0.38%, -0.68%, -1.57% |
| vue-docs/src/guide/extras/composition-api-faq.md | 16.849 | 16.889 | -0.00%, -0.30%, +0.88% |
| vue-docs/src/guide/extras/reactivity-in-depth.md | 36.564 | 36.138 | +0.03%, -1.16%, -1.19% |
| vue-docs/src/guide/extras/reactivity-transform.md | 19.516 | 19.366 | +0.40%, -0.44%, -0.85% |
| vue-docs/src/guide/extras/render-function.md | 39.470 | 39.181 | -2.28%, -0.73%, -0.63% |
| vue-docs/src/guide/extras/rendering-mechanism.md | 14.411 | 14.316 | +0.05%, +0.02%, -0.66% |
| vue-docs/src/guide/extras/ways-of-using-vue.md | 7.728 | 7.821 | -0.78%, +0.63%, +1.20% |
| vue-docs/src/guide/extras/web-components.md | 27.015 | 26.882 | -1.65%, -0.49%, +0.64% |
| vue-docs/src/guide/introduction.md | 16.693 | 16.539 | -0.81%, -1.29%, -1.03% |
| vue-docs/src/guide/quick-start.md | 27.136 | 27.195 | -0.62%, +0.77%, -0.45% |
| vue-docs/src/guide/reusability/composables.md | 24.490 | 24.536 | +0.63%, -0.40%, -0.20% |
| vue-docs/src/guide/reusability/custom-directives.md | 14.949 | 14.772 | -1.31%, -0.54%, -0.14% |
| vue-docs/src/guide/reusability/plugins.md | 8.302 | 8.261 | +0.28%, +0.75%, -0.57% |
| vue-docs/src/guide/scaling-up/routing.md | 7.737 | 7.682 | -0.70%, -0.29%, -1.43% |
| vue-docs/src/guide/scaling-up/sfc.md | 8.383 | 8.430 | -0.22%, +1.12%, +1.12% |
| vue-docs/src/guide/scaling-up/ssr.md | 29.752 | 29.445 | -0.51%, -0.96%, -1.52% |
| vue-docs/src/guide/scaling-up/state-management.md | 13.259 | 13.223 | +0.21%, -0.23%, -0.60% |
| vue-docs/src/guide/scaling-up/testing.md | 26.230 | 25.960 | -1.28%, -1.03%, +1.38% |
| vue-docs/src/guide/scaling-up/tooling.md | 24.976 | 25.107 | -0.69%, +0.68%, +0.85% |
| vue-docs/src/guide/typescript/composition-api.md | 24.300 | 24.125 | +0.27%, -0.80%, -2.23% |
| vue-docs/src/guide/typescript/options-api.md | 13.032 | 13.103 | -0.17%, +0.70%, +0.99% |
| vue-docs/src/guide/typescript/overview.md | 21.966 | 21.984 | -0.22%, +0.17%, -0.37% |
| vue-docs/src/tutorial/index.md | 1.234 | 1.252 | +2.23%, +1.74%, -0.94% |
| vue-docs/src/tutorial/src/step-1/description.md | 4.676 | 4.683 | +0.50%, -0.66%, -0.33% |
| vue-docs/src/tutorial/src/step-10/description.md | 2.371 | 2.353 | -1.18%, -0.74%, -1.49% |
| vue-docs/src/tutorial/src/step-11/description.md | 3.011 | 3.011 | +0.01%, -1.25%, -1.37% |
| vue-docs/src/tutorial/src/step-12/description.md | 3.661 | 3.595 | -2.42%, -0.85%, -1.94% |
| vue-docs/src/tutorial/src/step-13/description.md | 3.145 | 3.122 | -1.18%, +0.25%, -0.98% |
| vue-docs/src/tutorial/src/step-14/description.md | 2.976 | 2.966 | -1.83%, -0.01%, -1.61% |
| vue-docs/src/tutorial/src/step-15/description.md | 1.726 | 1.735 | -0.73%, -0.56%, +1.64% |
| vue-docs/src/tutorial/src/step-2/description.md | 7.659 | 7.554 | -1.34%, -1.41%, -1.37% |
| vue-docs/src/tutorial/src/step-3/description.md | 2.941 | 2.868 | -2.48%, -1.81%, -4.19% |
| vue-docs/src/tutorial/src/step-4/description.md | 5.129 | 5.032 | -1.88%, -1.93%, -1.41% |
| vue-docs/src/tutorial/src/step-5/description.md | 3.132 | 3.100 | -1.66%, -1.37%, -0.89% |
| vue-docs/src/tutorial/src/step-6/description.md | 3.061 | 3.038 | -0.34%, -0.13%, -1.40% |
| vue-docs/src/tutorial/src/step-7/description.md | 5.025 | 4.951 | -1.02%, -0.62%, -1.95% |
| vue-docs/src/tutorial/src/step-8/description.md | 4.344 | 4.300 | -0.64%, -0.40%, -1.09% |
| vue-docs/src/tutorial/src/step-9/description.md | 7.294 | 7.188 | -1.44%, -1.41%, -2.09% |
| vite-docs/docs/config/build-options.md | 48.422 | 48.423 | +0.48%, +0.00%, +0.29% |
| vite-docs/docs/config/dep-optimization-options.md | 16.985 | 16.698 | -1.70%, -1.48%, -1.69% |
| vite-docs/docs/config/index.md | 12.503 | 12.452 | -0.20%, +0.14%, -1.01% |
| vite-docs/docs/config/preview-options.md | 13.093 | 12.914 | -0.18%, -1.36%, -0.49% |
| vite-docs/docs/config/server-options.md | 46.610 | 47.242 | +0.13%, +2.01%, +1.22% |
| vite-docs/docs/config/shared-options.md | 73.588 | 73.541 | -0.68%, -1.07%, +0.60% |
| vite-docs/docs/config/ssr-options.md | 9.544 | 9.436 | -0.83%, -1.24%, -0.09% |
| vite-docs/docs/config/worker-options.md | 4.588 | 4.504 | -1.49%, -2.16%, -1.76% |
| vite-docs/docs/guide/api-environment-frameworks.md | 17.507 | 17.534 | +0.32%, +0.74%, -1.17% |
| vite-docs/docs/guide/api-environment-instances.md | 11.660 | 11.644 | -0.13%, -1.01%, +0.36% |
| vite-docs/docs/guide/api-environment-plugins.md | 23.497 | 23.250 | -0.38%, -1.84%, -1.32% |
| vite-docs/docs/guide/api-environment-runtimes.md | 15.763 | 15.729 | -0.22%, -1.46%, +0.01% |
| vite-docs/docs/guide/api-environment.md | 12.354 | 12.277 | -0.91%, +0.51%, -1.21% |
| vite-docs/docs/guide/api-hmr.md | 13.439 | 13.453 | +0.21%, +0.54%, +0.10% |
| vite-docs/docs/guide/api-javascript.md | 24.546 | 24.204 | -0.92%, -2.42%, -1.23% |
| vite-docs/docs/guide/api-plugin.md | 74.233 | 74.451 | +3.32%, +0.29%, -0.10% |
| vite-docs/docs/guide/assets.md | 11.943 | 11.865 | -1.19%, -0.93%, -0.45% |
| vite-docs/docs/guide/backend-integration.md | 22.830 | 22.809 | +0.27%, -1.15%, -0.42% |
| vite-docs/docs/guide/build.md | 22.751 | 22.842 | +0.56%, +0.05%, -0.51% |
| vite-docs/docs/guide/cli.md | 36.513 | 36.245 | -0.73%, -0.25%, +0.46% |
| vite-docs/docs/guide/dep-pre-bundling.md | 8.992 | 8.901 | -1.21%, -0.17%, -1.11% |
| vite-docs/docs/guide/env-and-mode.md | 25.187 | 25.401 | +1.78%, +0.73%, +0.75% |
| vite-docs/docs/guide/features.md | 94.498 | 95.640 | +2.24%, +1.69%, -0.93% |
| vite-docs/docs/guide/index.md | 25.834 | 25.802 | -0.12%, -0.23%, -0.02% |
| vite-docs/docs/guide/migration.md | 57.805 | 57.710 | -1.56%, -1.70%, +2.26% |
| vite-docs/docs/guide/performance.md | 12.256 | 12.246 | -0.36%, +0.52%, -0.13% |
| vite-docs/docs/guide/philosophy.md | 5.342 | 5.343 | -0.31%, +0.95%, +0.02% |
| vite-docs/docs/guide/ssr.md | 22.467 | 22.424 | -0.17%, -0.28%, -0.19% |
| vite-docs/docs/guide/static-deploy.md | 33.672 | 34.037 | +1.61%, -1.06%, +1.32% |
| vite-docs/docs/guide/troubleshooting.md | 29.733 | 30.009 | +2.30%, +0.55%, +0.71% |
| vite-docs/docs/guide/using-plugins.md | 5.839 | 5.871 | +0.25%, -1.69%, +0.69% |
| vite-docs/docs/guide/why.md | 9.777 | 9.686 | -0.91%, -0.55%, -1.22% |
| vite-docs/packages/vite/LICENSE.md | 182.463 | 182.317 | -0.68%, +1.03%, +1.74% |
| rust-book/2018-edition/src/SUMMARY.md | 34.343 | 33.940 | -1.53%, -0.87%, +0.35% |
| rust-book/2018-edition/src/appendix-00.md | 1.354 | 1.358 | -1.09%, +0.68%, -0.60% |
| rust-book/2018-edition/src/appendix-01-keywords.md | 1.413 | 1.383 | -1.64%, -0.24%, -3.38% |
| rust-book/2018-edition/src/appendix-02-operators.md | 1.406 | 1.421 | +1.07%, +1.36%, +0.05% |
| rust-book/2018-edition/src/appendix-03-derivable-traits.md | 1.461 | 1.443 | -1.21%, -0.52%, -0.58% |
| rust-book/2018-edition/src/appendix-04-useful-development-tools.md | 1.493 | 1.482 | -0.63%, +0.67%, -0.78% |
| rust-book/2018-edition/src/appendix-05-editions.md | 1.401 | 1.390 | -0.73%, -0.69%, -0.80% |
| rust-book/2018-edition/src/appendix-06-translation.md | 1.434 | 1.430 | +0.78%, -0.09%, -1.57% |
| rust-book/2018-edition/src/appendix-07-nightly-rust.md | 1.499 | 1.506 | +0.82%, +0.15%, -0.86% |
| rust-book/2018-edition/src/ch00-00-introduction.md | 1.392 | 1.398 | +0.10%, +0.98%, -0.84% |
| rust-book/2018-edition/src/ch01-00-getting-started.md | 1.412 | 1.411 | -0.20%, -0.05%, -0.59% |
| rust-book/2018-edition/src/ch01-01-installation.md | 1.394 | 1.393 | +0.46%, -0.71%, -0.69% |
| rust-book/2018-edition/src/ch01-02-hello-world.md | 1.390 | 1.386 | +0.48%, +0.33%, -1.44% |
| rust-book/2018-edition/src/ch01-03-hello-cargo.md | 1.392 | 1.377 | -1.11%, +1.49%, -1.51% |
| rust-book/2018-edition/src/ch02-00-guessing-game-tutorial.md | 1.467 | 1.461 | +1.15%, +0.18%, -0.97% |
| rust-book/2018-edition/src/ch03-00-common-programming-concepts.md | 1.478 | 1.441 | +0.39%, +0.13%, -2.50% |
| rust-book/2018-edition/src/ch03-01-variables-and-mutability.md | 1.449 | 1.447 | -0.13%, +0.03%, -1.79% |
| rust-book/2018-edition/src/ch03-02-data-types.md | 1.380 | 1.379 | -0.21%, +0.74%, -0.04% |
| rust-book/2018-edition/src/ch03-03-how-functions-work.md | 1.399 | 1.395 | -0.25%, +0.11%, -0.61% |
| rust-book/2018-edition/src/ch03-04-comments.md | 1.372 | 1.365 | -0.54%, +0.17%, -0.46% |
| rust-book/2018-edition/src/ch03-05-control-flow.md | 1.403 | 1.393 | -0.74%, +0.28%, -0.55% |
| rust-book/2018-edition/src/ch04-00-understanding-ownership.md | 1.439 | 1.451 | +1.08%, +0.93%, -1.18% |
| rust-book/2018-edition/src/ch04-01-what-is-ownership.md | 1.415 | 1.406 | -0.72%, +0.05%, -1.15% |
| rust-book/2018-edition/src/ch04-02-references-and-borrowing.md | 1.446 | 1.448 | +0.55%, +0.17%, -1.84% |
| rust-book/2018-edition/src/ch04-03-slices.md | 1.382 | 1.382 | +0.70%, -0.05%, -1.64% |
| rust-book/2018-edition/src/ch05-00-structs.md | 1.421 | 1.411 | -1.12%, +0.37%, -0.69% |
| rust-book/2018-edition/src/ch05-01-defining-structs.md | 1.420 | 1.418 | -0.61%, +0.47%, -0.84% |
| rust-book/2018-edition/src/ch05-02-example-structs.md | 1.418 | 1.418 | +0.01%, +0.71%, -3.32% |
| rust-book/2018-edition/src/ch05-03-method-syntax.md | 1.395 | 1.394 | +0.48%, +0.24%, -1.21% |
| rust-book/2018-edition/src/ch06-00-enums.md | 1.392 | 1.373 | -0.87%, +0.12%, -2.48% |
| rust-book/2018-edition/src/ch06-01-defining-an-enum.md | 1.404 | 1.402 | +0.41%, +0.52%, -1.91% |
| rust-book/2018-edition/src/ch06-02-match.md | 1.557 | 1.559 | +1.69%, +0.11%, -0.82% |
| rust-book/2018-edition/src/ch06-03-if-let.md | 1.560 | 1.543 | -0.83%, -1.14%, -0.58% |
| rust-book/2018-edition/src/ch07-00-packages-crates-and-modules.md | 1.416 | 1.416 | +0.04%, +0.26%, -1.66% |
| rust-book/2018-edition/src/ch07-01-packages-and-crates-for-making-libraries-and-executables.md | 1.524 | 1.521 | -0.08%, -0.42%, -1.74% |
| rust-book/2018-edition/src/ch07-02-modules-and-use-to-control-scope-and-privacy.md | 1.502 | 1.498 | +1.85%, -0.26%, -1.16% |
| rust-book/2018-edition/src/ch08-00-common-collections.md | 1.407 | 1.404 | -0.80%, -0.17%, -0.10% |
| rust-book/2018-edition/src/ch08-01-vectors.md | 1.415 | 1.411 | +1.00%, -0.96%, -0.90% |
| rust-book/2018-edition/src/ch08-02-strings.md | 1.416 | 1.407 | +0.60%, -0.63%, -0.88% |
| rust-book/2018-edition/src/ch08-03-hash-maps.md | 1.421 | 1.422 | -0.35%, +0.00%, -0.29% |
| rust-book/2018-edition/src/ch09-00-error-handling.md | 1.409 | 1.419 | +0.70%, +0.70%, +0.21% |
| rust-book/2018-edition/src/ch09-01-unrecoverable-errors-with-panic.md | 1.627 | 1.610 | -1.06%, -0.55%, -2.14% |
| rust-book/2018-edition/src/ch09-02-recoverable-errors-with-result.md | 1.637 | 1.629 | -0.46%, +0.21%, -0.53% |
| rust-book/2018-edition/src/ch09-03-to-panic-or-not-to-panic.md | 1.649 | 1.640 | +1.21%, -0.50%, -3.02% |
| rust-book/2018-edition/src/ch10-00-generics.md | 1.405 | 1.415 | -0.91%, +0.89%, +0.69% |
| rust-book/2018-edition/src/ch10-01-syntax.md | 1.376 | 1.366 | +0.20%, +0.39%, -2.68% |
| rust-book/2018-edition/src/ch10-02-traits.md | 1.394 | 1.389 | -0.45%, +0.12%, -2.70% |
| rust-book/2018-edition/src/ch10-03-lifetime-syntax.md | 1.435 | 1.433 | +0.18%, +0.25%, -1.69% |
| rust-book/2018-edition/src/ch11-00-testing.md | 1.381 | 1.385 | +1.00%, -0.20%, -1.70% |
| rust-book/2018-edition/src/ch11-01-writing-tests.md | 1.401 | 1.391 | -0.68%, -0.69%, -1.81% |
| rust-book/2018-edition/src/ch11-02-running-tests.md | 1.431 | 1.416 | -0.96%, -0.20%, -2.47% |
| rust-book/2018-edition/src/ch11-03-test-organization.md | 1.405 | 1.399 | -0.15%, -0.43%, -1.23% |
| rust-book/2018-edition/src/ch12-00-an-io-project.md | 1.456 | 1.450 | +0.54%, -1.22%, -0.40% |
| rust-book/2018-edition/src/ch12-01-accepting-command-line-arguments.md | 1.478 | 1.479 | +1.39%, +0.06%, -2.18% |
| rust-book/2018-edition/src/ch12-02-reading-a-file.md | 1.419 | 1.414 | +0.66%, -0.01%, -1.70% |
| rust-book/2018-edition/src/ch12-03-improving-error-handling-and-modularity.md | 1.566 | 1.558 | -0.52%, -0.46%, -0.09% |
| rust-book/2018-edition/src/ch12-04-testing-the-librarys-functionality.md | 1.619 | 1.647 | +1.03%, +2.72%, -0.80% |
| rust-book/2018-edition/src/ch12-05-working-with-environment-variables.md | 1.492 | 1.508 | +1.54%, +1.19%, -1.21% |
| rust-book/2018-edition/src/ch12-06-writing-to-stderr-instead-of-stdout.md | 1.597 | 1.607 | +0.48%, +0.91%, -2.60% |
| rust-book/2018-edition/src/ch13-00-functional-features.md | 1.465 | 1.462 | +0.53%, -0.06%, -0.72% |
| rust-book/2018-edition/src/ch13-01-closures.md | 1.519 | 1.517 | -0.13%, +0.55%, -2.65% |
| rust-book/2018-edition/src/ch13-02-iterators.md | 1.439 | 1.437 | -1.77%, +0.94%, -0.10% |
| rust-book/2018-edition/src/ch13-03-improving-our-io-project.md | 1.437 | 1.448 | +1.75%, -0.52%, -1.35% |
| rust-book/2018-edition/src/ch13-04-performance.md | 1.413 | 1.415 | +0.46%, +0.45%, +0.17% |
| rust-book/2018-edition/src/ch14-00-more-about-cargo.md | 1.455 | 1.427 | -1.10%, -0.52%, -1.94% |
| rust-book/2018-edition/src/ch14-01-release-profiles.md | 1.437 | 1.438 | +1.05%, +0.56%, -2.11% |
| rust-book/2018-edition/src/ch14-02-publishing-to-crates-io.md | 1.470 | 1.460 | -2.06%, -0.11%, -0.69% |
| rust-book/2018-edition/src/ch14-03-cargo-workspaces.md | 1.393 | 1.389 | -0.32%, +0.23%, -2.07% |
| rust-book/2018-edition/src/ch14-04-installing-binaries.md | 1.713 | 1.676 | -0.37%, -0.35%, -2.43% |
| rust-book/2018-edition/src/ch14-05-extending-cargo.md | 1.441 | 1.437 | -0.16%, +0.29%, -2.04% |
| rust-book/2018-edition/src/ch15-00-smart-pointers.md | 1.411 | 1.409 | -0.64%, +1.02%, -1.05% |
| rust-book/2018-edition/src/ch15-01-box.md | 1.720 | 1.719 | +0.77%, +0.06%, -2.83% |
| rust-book/2018-edition/src/ch15-02-deref.md | 1.713 | 1.722 | +0.09%, +1.14%, -0.20% |
| rust-book/2018-edition/src/ch15-03-drop.md | 1.635 | 1.622 | -0.81%, +0.31%, -0.58% |
| rust-book/2018-edition/src/ch15-04-rc.md | 1.709 | 1.689 | -1.66%, -0.41%, -0.81% |
| rust-book/2018-edition/src/ch15-05-interior-mutability.md | 1.765 | 1.765 | +0.35%, +0.24%, +0.02% |
| rust-book/2018-edition/src/ch15-06-reference-cycles.md | 1.429 | 1.431 | -0.23%, -0.04%, +0.35% |
| rust-book/2018-edition/src/ch16-00-concurrency.md | 1.388 | 1.385 | -0.35%, +0.97%, -0.17% |
| rust-book/2018-edition/src/ch16-01-threads.md | 1.408 | 1.411 | -0.19%, +0.52%, -1.93% |
| rust-book/2018-edition/src/ch16-02-message-passing.md | 1.465 | 1.462 | +0.30%, -0.30%, +1.15% |
| rust-book/2018-edition/src/ch16-03-shared-state.md | 1.410 | 1.395 | -1.12%, -0.01%, -1.66% |
| rust-book/2018-edition/src/ch16-04-extensible-concurrency-sync-and-send.md | 1.830 | 1.803 | +0.49%, +0.93%, -3.75% |
| rust-book/2018-edition/src/ch17-00-oop.md | 1.419 | 1.403 | -1.20%, -0.19%, -1.13% |
| rust-book/2018-edition/src/ch17-01-what-is-oo.md | 1.440 | 1.440 | +0.05%, +1.77%, -1.72% |
| rust-book/2018-edition/src/ch17-02-trait-objects.md | 1.465 | 1.474 | -1.45%, +0.78%, +0.57% |
| rust-book/2018-edition/src/ch17-03-oo-design-patterns.md | 1.472 | 1.462 | -0.98%, +0.04%, -1.16% |
| rust-book/2018-edition/src/ch18-00-patterns.md | 1.400 | 1.397 | +1.88%, +0.79%, -2.14% |
| rust-book/2018-edition/src/ch18-01-all-the-places-for-patterns.md | 1.493 | 1.477 | +0.69%, -0.45%, -1.22% |
| rust-book/2018-edition/src/ch18-02-refutability.md | 1.444 | 1.451 | -0.49%, +0.39%, +0.65% |
| rust-book/2018-edition/src/ch18-03-pattern-syntax.md | 1.409 | 1.412 | +0.26%, +1.09%, -0.67% |
| rust-book/2018-edition/src/ch19-00-advanced-features.md | 1.408 | 1.392 | -1.14%, -0.73%, -1.15% |
| rust-book/2018-edition/src/ch19-01-unsafe-rust.md | 1.393 | 1.406 | -1.43%, +1.47%, +0.92% |
| rust-book/2018-edition/src/ch19-02-advanced-lifetimes.md | 1.374 | 1.371 | -0.67%, +1.30%, -1.13% |
| rust-book/2018-edition/src/ch19-03-advanced-traits.md | 1.425 | 1.407 | -1.22%, +0.22%, -2.66% |
| rust-book/2018-edition/src/ch19-04-advanced-types.md | 1.420 | 1.404 | -1.47%, -0.71%, -2.48% |
| rust-book/2018-edition/src/ch19-05-advanced-functions-and-closures.md | 1.495 | 1.493 | -0.09%, -0.15%, -0.50% |
| rust-book/2018-edition/src/ch19-06-macros.md | 1.366 | 1.358 | -1.13%, +0.18%, -2.47% |
| rust-book/2018-edition/src/ch20-00-final-project-a-web-server.md | 1.476 | 1.486 | +1.09%, +0.70%, -2.64% |
| rust-book/2018-edition/src/ch20-01-single-threaded.md | 1.440 | 1.422 | +0.24%, +0.05%, -1.34% |
| rust-book/2018-edition/src/ch20-02-multithreaded.md | 1.493 | 1.473 | -1.48%, -1.39%, -0.31% |
| rust-book/2018-edition/src/ch20-03-graceful-shutdown-and-cleanup.md | 1.494 | 1.484 | -0.68%, +0.42%, -1.38% |
| rust-book/2018-edition/src/foreword.md | 1.335 | 1.341 | -0.15%, +0.50%, -1.13% |
| rust-book/first-edition/src/README.md | 1.383 | 1.383 | -0.01%, -0.73%, +0.40% |
| rust-book/first-edition/src/SUMMARY.md | 15.565 | 15.556 | -0.55%, -0.04%, -0.01% |
| rust-book/first-edition/src/associated-types.md | 1.524 | 1.511 | +0.42%, -0.87%, -0.72% |
| rust-book/first-edition/src/attributes.md | 1.348 | 1.342 | -0.46%, -0.21%, -1.42% |
| rust-book/first-edition/src/bibliography.md | 1.376 | 1.357 | -1.09%, +0.35%, -2.07% |
| rust-book/first-edition/src/borrow-and-asref.md | 1.396 | 1.388 | -0.61%, +1.21%, -1.51% |
| rust-book/first-edition/src/casting-between-types.md | 1.383 | 1.384 | +0.57%, +0.22%, -1.91% |
| rust-book/first-edition/src/choosing-your-guarantees.md | 1.413 | 1.422 | +0.61%, +0.90%, -0.71% |
| rust-book/first-edition/src/closures.md | 1.347 | 1.355 | -0.07%, +1.11%, -1.75% |
| rust-book/first-edition/src/comments.md | 1.356 | 1.337 | -1.47%, +0.86%, -1.86% |
| rust-book/first-edition/src/concurrency.md | 1.378 | 1.361 | -0.05%, -0.18%, -1.38% |
| rust-book/first-edition/src/conditional-compilation.md | 1.386 | 1.379 | -0.21%, -0.50%, -1.92% |
| rust-book/first-edition/src/const-and-static.md | 1.464 | 1.461 | -0.65%, +0.06%, +0.55% |
| rust-book/first-edition/src/crates-and-modules.md | 1.459 | 1.456 | -0.67%, +0.46%, +0.40% |
| rust-book/first-edition/src/deref-coercions.md | 1.616 | 1.598 | -0.15%, -1.10%, -1.42% |
| rust-book/first-edition/src/documentation.md | 1.479 | 1.454 | +0.95%, -0.64%, -1.89% |
| rust-book/first-edition/src/drop.md | 1.373 | 1.349 | +0.74%, -1.48%, -1.76% |
| rust-book/first-edition/src/effective-rust.md | 1.381 | 1.376 | -0.65%, +0.31%, -2.95% |
| rust-book/first-edition/src/enums.md | 1.372 | 1.378 | -0.48%, +1.82%, -0.86% |
| rust-book/first-edition/src/error-handling.md | 1.397 | 1.389 | -1.05%, -0.27%, -0.59% |
| rust-book/first-edition/src/ffi.md | 1.461 | 1.453 | +0.03%, -0.41%, -1.00% |
| rust-book/first-edition/src/functions.md | 1.384 | 1.383 | -0.07%, -0.06%, +0.47% |
| rust-book/first-edition/src/generics.md | 1.354 | 1.354 | +0.01%, +0.92%, -1.36% |
| rust-book/first-edition/src/getting-started.md | 1.402 | 1.391 | -1.96%, -0.76%, +0.39% |
| rust-book/first-edition/src/glossary.md | 1.350 | 1.344 | -1.27%, +0.13%, -1.07% |
| rust-book/first-edition/src/guessing-game.md | 1.419 | 1.410 | -0.27%, -0.67%, -4.65% |
| rust-book/first-edition/src/if-let.md | 1.358 | 1.363 | +0.62%, -0.10%, -0.29% |
| rust-book/first-edition/src/if.md | 1.364 | 1.368 | +0.28%, +2.02%, -1.82% |
| rust-book/first-edition/src/iterators.md | 1.350 | 1.357 | -0.08%, +0.65%, -1.16% |
| rust-book/first-edition/src/lifetimes.md | 1.371 | 1.376 | -0.17%, +0.37%, -1.38% |
| rust-book/first-edition/src/loops.md | 1.397 | 1.398 | +0.54%, +0.59%, -1.13% |
| rust-book/first-edition/src/macros.md | 1.387 | 1.358 | -2.07%, +0.68%, -4.71% |
| rust-book/first-edition/src/match.md | 1.346 | 1.360 | +0.23%, +1.89%, -0.96% |
| rust-book/first-edition/src/method-syntax.md | 1.383 | 1.381 | -0.89%, +0.69%, -0.76% |
| rust-book/first-edition/src/mutability.md | 1.379 | 1.376 | -0.47%, +0.07%, -1.40% |
| rust-book/first-edition/src/operators-and-overloading.md | 1.527 | 1.508 | -1.30%, -1.30%, -0.71% |
| rust-book/first-edition/src/ownership.md | 1.370 | 1.375 | +1.08%, +1.51%, -0.45% |
| rust-book/first-edition/src/patterns.md | 1.368 | 1.385 | +1.05%, +1.52%, -0.09% |
| rust-book/first-edition/src/primitive-types.md | 1.424 | 1.386 | -0.62%, -1.27%, -2.86% |
| rust-book/first-edition/src/procedural-macros.md | 1.536 | 1.540 | +1.07%, +0.25%, -1.94% |
| rust-book/first-edition/src/raw-pointers.md | 1.420 | 1.427 | +0.51%, -0.35%, -1.03% |
| rust-book/first-edition/src/references-and-borrowing.md | 1.412 | 1.428 | +1.21%, +1.27%, -1.41% |
| rust-book/first-edition/src/release-channels.md | 1.359 | 1.356 | +0.31%, -0.24%, -1.69% |
| rust-book/first-edition/src/strings.md | 1.350 | 1.352 | -0.43%, +1.27%, -0.69% |
| rust-book/first-edition/src/structs.md | 1.341 | 1.357 | +1.99%, +1.03%, -1.07% |
| rust-book/first-edition/src/syntax-and-semantics.md | 1.428 | 1.419 | -0.66%, +0.91%, -1.30% |
| rust-book/first-edition/src/syntax-index.md | 1.357 | 1.366 | +0.63%, +0.71%, -1.56% |
| rust-book/first-edition/src/testing.md | 1.355 | 1.357 | +0.56%, +0.64%, -2.19% |
| rust-book/first-edition/src/the-stack-and-the-heap.md | 1.460 | 1.455 | +0.75%, -0.22%, -1.55% |
| rust-book/first-edition/src/trait-objects.md | 1.397 | 1.399 | -0.94%, +0.36%, +0.93% |
| rust-book/first-edition/src/traits.md | 1.363 | 1.343 | -1.34%, +0.08%, -1.49% |
| rust-book/first-edition/src/type-aliases.md | 1.443 | 1.463 | -0.59%, +1.77%, -0.71% |
| rust-book/first-edition/src/ufcs.md | 1.379 | 1.380 | -0.96%, +0.02%, -1.40% |
| rust-book/first-edition/src/unsafe.md | 1.356 | 1.371 | +1.06%, -0.37%, -0.92% |
| rust-book/first-edition/src/unsized-types.md | 1.452 | 1.453 | +0.05%, +0.44%, -1.21% |
| rust-book/first-edition/src/using-rust-without-the-standard-library.md | 1.424 | 1.428 | +0.29%, +0.32%, -1.09% |
| rust-book/first-edition/src/variable-bindings.md | 1.349 | 1.347 | -0.13%, +1.17%, -1.54% |
| rust-book/first-edition/src/vectors.md | 1.348 | 1.351 | -0.37%, +1.65%, -0.87% |
| rust-book/packages/mdbook-trpl/src/bin/README - mdbook-trpl-note.md | 5.517 | 5.545 | +1.03%, -0.63%, +0.27% |
| rust-book/second-edition/src/SUMMARY.md | 33.690 | 33.426 | -0.78%, -0.17%, -0.18% |
| rust-book/second-edition/src/appendix-00.md | 1.339 | 1.347 | +0.83%, +0.73%, -0.96% |
| rust-book/second-edition/src/appendix-01-keywords.md | 1.385 | 1.382 | -0.22%, -0.64%, -1.03% |
| rust-book/second-edition/src/appendix-02-operators.md | 1.416 | 1.402 | -1.03%, -0.11%, -1.14% |
| rust-book/second-edition/src/appendix-03-derivable-traits.md | 1.437 | 1.424 | -0.90%, -0.33%, -1.11% |
| rust-book/second-edition/src/appendix-04-macros.md | 1.379 | 1.373 | -0.43%, -0.77%, -1.09% |
| rust-book/second-edition/src/appendix-05-translation.md | 1.436 | 1.428 | -0.11%, +0.11%, -1.60% |
| rust-book/second-edition/src/appendix-06-newest-features.md | 1.406 | 1.390 | -0.10%, -0.24%, -1.33% |
| rust-book/second-edition/src/appendix-07-nightly-rust.md | 1.510 | 1.492 | -1.81%, -0.71%, -0.67% |
| rust-book/second-edition/src/ch00-00-introduction.md | 1.388 | 1.379 | -0.71%, +1.66%, -0.88% |
| rust-book/second-edition/src/ch01-00-getting-started.md | 1.411 | 1.410 | +0.15%, +0.46%, -0.67% |
| rust-book/second-edition/src/ch01-01-installation.md | 1.390 | 1.387 | -0.49%, +0.94%, -1.29% |
| rust-book/second-edition/src/ch01-02-hello-world.md | 1.390 | 1.389 | +0.76%, -0.50%, -1.24% |
| rust-book/second-edition/src/ch01-03-hello-cargo.md | 1.393 | 1.389 | -0.26%, +1.49%, -0.92% |
| rust-book/second-edition/src/ch02-00-guessing-game-tutorial.md | 1.451 | 1.442 | -0.67%, +0.13%, -1.54% |
| rust-book/second-edition/src/ch03-00-common-programming-concepts.md | 1.469 | 1.458 | -0.51%, -0.28%, -1.17% |
| rust-book/second-edition/src/ch03-01-variables-and-mutability.md | 1.438 | 1.439 | -0.03%, -0.39%, +0.04% |
| rust-book/second-edition/src/ch03-02-data-types.md | 1.373 | 1.373 | +0.75%, -0.92%, -0.79% |
| rust-book/second-edition/src/ch03-03-how-functions-work.md | 1.400 | 1.393 | +1.09%, -0.07%, -1.43% |
| rust-book/second-edition/src/ch03-04-comments.md | 1.367 | 1.358 | -1.11%, +1.35%, -0.64% |
| rust-book/second-edition/src/ch03-05-control-flow.md | 1.392 | 1.388 | +0.24%, -0.52%, -0.26% |
| rust-book/second-edition/src/ch04-00-understanding-ownership.md | 1.457 | 1.445 | +0.11%, +0.13%, -1.54% |
| rust-book/second-edition/src/ch04-01-what-is-ownership.md | 1.402 | 1.388 | -0.66%, +0.26%, -1.02% |
| rust-book/second-edition/src/ch04-02-references-and-borrowing.md | 1.446 | 1.427 | -0.38%, +0.32%, -2.25% |
| rust-book/second-edition/src/ch04-03-slices.md | 1.385 | 1.379 | -0.75%, +0.27%, -0.38% |
| rust-book/second-edition/src/ch05-00-structs.md | 1.423 | 1.400 | -1.07%, +0.58%, -2.26% |
| rust-book/second-edition/src/ch05-01-defining-structs.md | 1.429 | 1.417 | +0.69%, +0.66%, -1.39% |
| rust-book/second-edition/src/ch05-02-example-structs.md | 1.415 | 1.416 | +0.09%, -0.12%, -0.59% |
| rust-book/second-edition/src/ch05-03-method-syntax.md | 1.399 | 1.404 | +0.69%, +0.70%, -1.10% |
| rust-book/second-edition/src/ch06-00-enums.md | 1.392 | 1.373 | -0.81%, -0.26%, -2.43% |
| rust-book/second-edition/src/ch06-01-defining-an-enum.md | 1.398 | 1.402 | +1.26%, -0.37%, -1.52% |
| rust-book/second-edition/src/ch06-02-match.md | 1.559 | 1.548 | -1.38%, -0.57%, -0.41% |
| rust-book/second-edition/src/ch06-03-if-let.md | 1.554 | 1.541 | -0.58%, -0.56%, -2.41% |
| rust-book/second-edition/src/ch07-00-modules.md | 1.479 | 1.466 | -0.05%, +0.06%, -1.80% |
| rust-book/second-edition/src/ch07-01-mod-and-the-filesystem.md | 1.620 | 1.615 | -0.28%, +0.70%, -0.94% |
| rust-book/second-edition/src/ch07-02-controlling-visibility-with-pub.md | 1.656 | 1.657 | -0.37%, +1.78%, -0.63% |
| rust-book/second-edition/src/ch07-03-importing-names-with-use.md | 1.519 | 1.512 | +1.51%, -0.74%, -0.90% |
| rust-book/second-edition/src/ch08-00-common-collections.md | 1.396 | 1.412 | +1.10%, +0.31%, -0.94% |
| rust-book/second-edition/src/ch08-01-vectors.md | 1.406 | 1.398 | -0.73%, -0.12%, -0.60% |
| rust-book/second-edition/src/ch08-02-strings.md | 1.415 | 1.410 | +0.27%, -0.29%, -0.50% |
| rust-book/second-edition/src/ch08-03-hash-maps.md | 1.431 | 1.420 | -0.64%, -0.83%, -1.10% |
| rust-book/second-edition/src/ch09-00-error-handling.md | 1.414 | 1.396 | -1.33%, -0.56%, -1.03% |
| rust-book/second-edition/src/ch09-01-unrecoverable-errors-with-panic.md | 1.646 | 1.650 | +0.88%, +0.52%, -1.51% |
| rust-book/second-edition/src/ch09-02-recoverable-errors-with-result.md | 1.638 | 1.631 | +0.65%, -1.02%, -1.49% |
| rust-book/second-edition/src/ch09-03-to-panic-or-not-to-panic.md | 1.643 | 1.664 | +1.39%, +1.57%, -1.20% |
| rust-book/second-edition/src/ch10-00-generics.md | 1.389 | 1.400 | +0.96%, +1.42%, -0.76% |
| rust-book/second-edition/src/ch10-01-syntax.md | 1.381 | 1.365 | -2.10%, +1.22%, -0.26% |
| rust-book/second-edition/src/ch10-02-traits.md | 1.406 | 1.386 | -1.43%, +0.28%, -1.25% |
| rust-book/second-edition/src/ch10-03-lifetime-syntax.md | 1.424 | 1.432 | +0.69%, -0.30%, -0.22% |
| rust-book/second-edition/src/ch11-00-testing.md | 1.395 | 1.397 | +0.56%, +1.28%, -1.75% |
| rust-book/second-edition/src/ch11-01-writing-tests.md | 1.396 | 1.395 | +0.77%, -0.12%, -0.75% |
| rust-book/second-edition/src/ch11-02-running-tests.md | 1.431 | 1.415 | -1.26%, -1.21%, -0.55% |
| rust-book/second-edition/src/ch11-03-test-organization.md | 1.416 | 1.396 | -0.81%, -0.21%, -2.29% |
| rust-book/second-edition/src/ch12-00-an-io-project.md | 1.443 | 1.444 | -1.87%, +0.68%, +0.24% |
| rust-book/second-edition/src/ch12-01-accepting-command-line-arguments.md | 1.482 | 1.481 | -0.20%, +1.10%, -1.16% |
| rust-book/second-edition/src/ch12-02-reading-a-file.md | 1.406 | 1.404 | +0.12%, +0.42%, -1.91% |
| rust-book/second-edition/src/ch12-03-improving-error-handling-and-modularity.md | 1.571 | 1.553 | -1.48%, -1.59%, -1.13% |
| rust-book/second-edition/src/ch12-04-testing-the-librarys-functionality.md | 1.613 | 1.630 | +2.41%, +1.53%, -0.70% |
| rust-book/second-edition/src/ch12-05-working-with-environment-variables.md | 1.490 | 1.522 | +2.33%, +2.63%, -1.69% |
| rust-book/second-edition/src/ch12-06-writing-to-stderr-instead-of-stdout.md | 1.618 | 1.636 | +0.57%, +2.86%, -1.14% |
| rust-book/second-edition/src/ch13-00-functional-features.md | 1.467 | 1.458 | -0.64%, -1.13%, -0.45% |
| rust-book/second-edition/src/ch13-01-closures.md | 1.518 | 1.517 | -0.03%, -0.59%, -0.35% |
| rust-book/second-edition/src/ch13-02-iterators.md | 1.422 | 1.429 | +2.25%, +0.50%, -0.96% |
| rust-book/second-edition/src/ch13-03-improving-our-io-project.md | 1.442 | 1.438 | +1.65%, -0.27%, -0.71% |
| rust-book/second-edition/src/ch13-04-performance.md | 1.417 | 1.421 | +0.28%, +0.72%, -0.75% |
| rust-book/second-edition/src/ch14-00-more-about-cargo.md | 1.454 | 1.435 | -1.52%, +0.46%, -2.47% |
| rust-book/second-edition/src/ch14-01-release-profiles.md | 1.434 | 1.434 | +0.28%, -1.33%, -0.56% |
| rust-book/second-edition/src/ch14-02-publishing-to-crates-io.md | 1.459 | 1.463 | +1.42%, +0.27%, -2.36% |
| rust-book/second-edition/src/ch14-03-cargo-workspaces.md | 1.395 | 1.396 | +1.18%, -0.17%, -1.17% |
| rust-book/second-edition/src/ch14-04-installing-binaries.md | 1.634 | 1.618 | -1.01%, +0.40%, -1.70% |
| rust-book/second-edition/src/ch14-05-extending-cargo.md | 1.428 | 1.435 | +0.49%, +0.51%, -0.72% |
| rust-book/second-edition/src/ch15-00-smart-pointers.md | 1.406 | 1.396 | +0.44%, -0.68%, -1.03% |
| rust-book/second-edition/src/ch15-01-box.md | 1.700 | 1.691 | -0.53%, -1.37%, -0.65% |
| rust-book/second-edition/src/ch15-02-deref.md | 1.701 | 1.697 | +0.36%, -0.66%, -0.22% |
| rust-book/second-edition/src/ch15-03-drop.md | 1.626 | 1.621 | +0.67%, +0.43%, -0.85% |
| rust-book/second-edition/src/ch15-04-rc.md | 1.686 | 1.666 | +0.04%, -0.49%, -1.26% |
| rust-book/second-edition/src/ch15-05-interior-mutability.md | 1.692 | 1.701 | +1.98%, +0.51%, -1.21% |
| rust-book/second-edition/src/ch15-06-reference-cycles.md | 1.419 | 1.414 | +0.67%, +0.78%, -0.53% |
| rust-book/second-edition/src/ch16-00-concurrency.md | 1.400 | 1.382 | +0.56%, +0.30%, -1.74% |
| rust-book/second-edition/src/ch16-01-threads.md | 1.404 | 1.399 | -0.41%, +1.12%, -0.56% |
| rust-book/second-edition/src/ch16-02-message-passing.md | 1.456 | 1.454 | +0.12%, -0.82%, -0.18% |
| rust-book/second-edition/src/ch16-03-shared-state.md | 1.402 | 1.382 | -0.30%, +0.17%, -1.68% |
| rust-book/second-edition/src/ch16-04-extensible-concurrency-sync-and-send.md | 1.816 | 1.800 | +0.51%, -0.81%, -1.13% |
| rust-book/second-edition/src/ch17-00-oop.md | 1.392 | 1.393 | +0.50%, +0.19%, -1.01% |
| rust-book/second-edition/src/ch17-01-what-is-oo.md | 1.446 | 1.431 | -0.93%, -1.00%, -0.25% |
| rust-book/second-edition/src/ch17-02-trait-objects.md | 1.471 | 1.464 | +0.54%, -2.59%, -0.86% |
| rust-book/second-edition/src/ch17-03-oo-design-patterns.md | 1.454 | 1.440 | -0.10%, +0.21%, -1.76% |
| rust-book/second-edition/src/ch18-00-patterns.md | 1.386 | 1.372 | -0.00%, +0.36%, -1.39% |
| rust-book/second-edition/src/ch18-01-all-the-places-for-patterns.md | 1.467 | 1.465 | +0.38%, +1.34%, -1.25% |
| rust-book/second-edition/src/ch18-02-refutability.md | 1.446 | 1.436 | +0.19%, -0.73%, -0.99% |
| rust-book/second-edition/src/ch18-03-pattern-syntax.md | 1.408 | 1.390 | -0.39%, +0.38%, -1.76% |
| rust-book/second-edition/src/ch19-00-advanced-features.md | 1.402 | 1.390 | -0.29%, +0.81%, -1.87% |
| rust-book/second-edition/src/ch19-01-unsafe-rust.md | 1.379 | 1.376 | +0.24%, +0.44%, -1.77% |
| rust-book/second-edition/src/ch19-02-advanced-lifetimes.md | 1.366 | 1.363 | +0.29%, +0.83%, -1.54% |
| rust-book/second-edition/src/ch19-03-advanced-traits.md | 1.411 | 1.406 | +0.10%, +0.12%, -1.23% |
| rust-book/second-edition/src/ch19-04-advanced-types.md | 1.412 | 1.396 | -0.34%, -0.46%, -1.15% |
| rust-book/second-edition/src/ch19-05-advanced-functions-and-closures.md | 1.498 | 1.490 | +1.14%, +0.60%, -1.06% |
| rust-book/second-edition/src/ch20-00-final-project-a-web-server.md | 1.467 | 1.467 | +0.29%, +1.48%, -1.95% |
| rust-book/second-edition/src/ch20-01-single-threaded.md | 1.438 | 1.419 | +0.38%, -1.38%, -1.33% |
| rust-book/second-edition/src/ch20-02-multithreaded.md | 1.471 | 1.453 | -1.25%, -1.21%, -1.24% |
| rust-book/second-edition/src/ch20-03-graceful-shutdown-and-cleanup.md | 1.490 | 1.472 | +0.16%, +0.25%, -1.93% |
| rust-book/second-edition/src/foreword.md | 1.336 | 1.326 | -0.01%, -0.24%, -0.76% |
| rust-book/src/SUMMARY.md | 35.686 | 35.863 | +0.53%, -1.26%, +0.31% |
| rust-book/src/appendix-00.md | 0.665 | 0.663 | +0.28%, +1.50%, -3.00% |
| rust-book/src/appendix-01-keywords.md | 21.188 | 21.037 | -0.85%, -0.56%, -0.71% |
| rust-book/src/appendix-02-operators.md | 58.461 | 60.081 | +13.98%, -1.07%, -0.79% |
| rust-book/src/appendix-03-derivable-traits.md | 24.905 | 24.778 | -1.22%, -0.14%, -0.51% |
| rust-book/src/appendix-04-useful-development-tools.md | 12.868 | 12.818 | +0.09%, -0.63%, -0.29% |
| rust-book/src/appendix-05-editions.md | 5.460 | 5.435 | -0.05%, +0.81%, -0.47% |
| rust-book/src/appendix-06-translation.md | 10.015 | 9.935 | -1.30%, +0.22%, -1.31% |
| rust-book/src/appendix-07-nightly-rust.md | 12.967 | 12.893 | -0.99%, +0.46%, -0.56% |
| rust-book/src/ch00-00-introduction.md | 17.780 | 17.641 | -0.55%, -0.96%, -1.05% |
| rust-book/src/ch01-00-getting-started.md | 1.304 | 1.288 | -1.19%, -1.88%, -0.68% |
| rust-book/src/ch01-01-installation.md | 14.070 | 14.171 | -0.63%, +0.54%, +0.72% |
| rust-book/src/ch01-02-hello-world.md | 16.977 | 16.937 | +0.06%, +0.54%, -1.26% |
| rust-book/src/ch01-03-hello-cargo.md | 21.905 | 21.813 | -1.48%, -1.38%, +0.17% |
| rust-book/src/ch02-00-guessing-game-tutorial.md | 101.691 | 100.900 | -1.25%, -0.30%, +0.66% |
| rust-book/src/ch03-00-common-programming-concepts.md | 3.080 | 3.093 | +0.05%, -0.90%, +0.81% |
| rust-book/src/ch03-01-variables-and-mutability.md | 17.940 | 18.025 | -0.72%, -0.46%, +0.67% |
| rust-book/src/ch03-02-data-types.md | 40.701 | 40.634 | -0.59%, +0.67%, +0.61% |
| rust-book/src/ch03-03-how-functions-work.md | 19.991 | 19.895 | +0.10%, -0.96%, -0.29% |
| rust-book/src/ch03-04-comments.md | 3.572 | 3.571 | -0.19%, +0.02%, +0.61% |
| rust-book/src/ch03-05-control-flow.md | 34.732 | 34.770 | -0.84%, +0.27%, +0.11% |
| rust-book/src/ch04-00-understanding-ownership.md | 0.845 | 0.843 | -0.55%, +1.26%, -1.03% |
| rust-book/src/ch04-01-what-is-ownership.md | 48.098 | 48.102 | +1.68%, +1.96%, -0.37% |
| rust-book/src/ch04-02-references-and-borrowing.md | 18.122 | 18.016 | +0.34%, -0.87%, -0.18% |
| rust-book/src/ch04-03-slices.md | 27.181 | 27.026 | +0.92%, -0.57%, -1.12% |
| rust-book/src/ch05-00-structs.md | 1.521 | 1.526 | +0.89%, +0.58%, +0.08% |
| rust-book/src/ch05-01-defining-structs.md | 27.494 | 27.447 | -0.17%, +2.06%, -1.63% |
| rust-book/src/ch05-02-example-structs.md | 24.039 | 23.760 | -1.69%, -1.03%, +0.08% |
| rust-book/src/ch05-03-method-syntax.md | 27.094 | 27.157 | +0.52%, -0.63%, -0.40% |
| rust-book/src/ch06-00-enums.md | 1.549 | 1.547 | -0.14%, +2.07%, -0.34% |
| rust-book/src/ch06-01-defining-an-enum.md | 28.876 | 29.120 | +0.19%, +1.35%, +1.86% |
| rust-book/src/ch06-02-match.md | 23.553 | 23.514 | -0.19%, +0.38%, -0.99% |
| rust-book/src/ch06-03-if-let.md | 13.172 | 13.032 | +0.36%, -0.91%, -1.18% |
| rust-book/src/ch07-00-managing-growing-projects-with-packages-crates-and-modules.md | 5.331 | 5.322 | -0.54%, +0.20%, -0.18% |
| rust-book/src/ch07-01-packages-and-crates.md | 8.557 | 8.532 | -0.07%, +0.32%, -0.43% |
| rust-book/src/ch07-02-defining-modules-to-control-scope-and-privacy.md | 16.609 | 16.561 | +0.52%, -0.01%, -0.85% |
| rust-book/src/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.md | 31.986 | 32.345 | +2.12%, +0.36%, -0.58% |
| rust-book/src/ch07-04-bringing-paths-into-scope-with-the-use-keyword.md | 28.288 | 28.480 | +0.82%, -0.20%, -0.09% |
| rust-book/src/ch07-05-separating-modules-into-different-files.md | 13.443 | 13.316 | -0.10%, -0.79%, -1.41% |
| rust-book/src/ch08-00-common-collections.md | 3.536 | 3.509 | -0.80%, +0.58%, -1.18% |
| rust-book/src/ch08-01-vectors.md | 20.313 | 20.288 | -0.28%, +1.04%, -0.12% |
| rust-book/src/ch08-02-strings.md | 35.054 | 35.029 | -0.07%, -0.64%, +0.29% |
| rust-book/src/ch08-03-hash-maps.md | 24.284 | 24.401 | +0.48%, +1.08%, +0.82% |
| rust-book/src/ch09-00-error-handling.md | 2.513 | 2.511 | -0.79%, +0.96%, -0.28% |
| rust-book/src/ch09-01-unrecoverable-errors-with-panic.md | 13.324 | 13.351 | -0.31%, -0.09%, +0.70% |
| rust-book/src/ch09-02-recoverable-errors-with-result.md | 65.199 | 65.069 | -1.01%, +3.43%, -0.20% |
| rust-book/src/ch09-03-to-panic-or-not-to-panic.md | 24.035 | 24.254 | +1.06%, -0.74%, +0.08% |
| rust-book/src/ch10-00-generics.md | 8.480 | 8.445 | -0.88%, -0.85%, -0.23% |
| rust-book/src/ch10-01-syntax.md | 29.629 | 29.324 | -0.99%, -0.03%, -1.28% |
| rust-book/src/ch10-02-traits.md | 39.125 | 38.578 | -1.64%, -0.36%, +1.62% |
| rust-book/src/ch10-03-lifetime-syntax.md | 53.637 | 54.448 | +2.37%, -1.21%, +0.72% |
| rust-book/src/ch11-00-testing.md | 3.038 | 3.044 | +0.25%, +1.15%, +0.15% |
| rust-book/src/ch11-01-writing-tests.md | 56.566 | 57.258 | +2.61%, -3.84%, +0.19% |
| rust-book/src/ch11-02-running-tests.md | 14.646 | 14.436 | -1.16%, -0.25%, -1.68% |
| rust-book/src/ch11-03-test-organization.md | 24.500 | 24.916 | +1.70%, -0.35%, +0.09% |
| rust-book/src/ch12-00-an-io-project.md | 7.749 | 7.767 | +0.16%, +1.61%, +0.14% |
| rust-book/src/ch12-01-accepting-command-line-arguments.md | 12.823 | 12.753 | +0.74%, +0.28%, -0.55% |
| rust-book/src/ch12-02-reading-a-file.md | 4.565 | 4.563 | +0.25%, -0.25%, -0.26% |
| rust-book/src/ch12-03-improving-error-handling-and-modularity.md | 61.690 | 62.517 | +1.34%, +0.90%, +1.84% |
| rust-book/src/ch12-04-testing-the-librarys-functionality.md | 20.258 | 20.246 | -0.60%, +0.33%, +0.57% |
| rust-book/src/ch12-05-working-with-environment-variables.md | 18.745 | 18.462 | -1.51%, -1.72%, -0.80% |
| rust-book/src/ch12-06-writing-to-stderr-instead-of-stdout.md | 7.929 | 7.814 | -0.50%, -0.46%, -1.44% |
| rust-book/src/ch13-00-functional-features.md | 2.448 | 2.430 | +0.09%, -0.85%, -0.92% |
| rust-book/src/ch13-01-closures.md | 40.116 | 40.333 | +0.81%, -0.89%, -0.63% |
| rust-book/src/ch13-02-iterators.md | 17.645 | 17.564 | +0.32%, +0.44%, -0.52% |
| rust-book/src/ch13-03-improving-our-io-project.md | 19.131 | 19.174 | +0.22%, +0.66%, -1.60% |
| rust-book/src/ch13-04-performance.md | 5.076 | 5.065 | -0.04%, +0.05%, -0.32% |
| rust-book/src/ch14-00-more-about-cargo.md | 2.263 | 2.230 | -1.46%, -0.73%, -1.46% |
| rust-book/src/ch14-01-release-profiles.md | 6.853 | 6.853 | +0.13%, +0.69%, -0.01% |
| rust-book/src/ch14-02-publishing-to-crates-io.md | 40.469 | 40.720 | -1.13%, +2.36%, +0.52% |
| rust-book/src/ch14-03-cargo-workspaces.md | 25.821 | 25.723 | +0.63%, -0.88%, -0.38% |
| rust-book/src/ch14-04-installing-binaries.md | 4.508 | 4.425 | -1.49%, -0.17%, -1.86% |
| rust-book/src/ch14-05-extending-cargo.md | 2.476 | 2.458 | +0.23%, -0.50%, -0.74% |
| rust-book/src/ch15-00-smart-pointers.md | 4.743 | 4.703 | -0.82%, +0.38%, -1.09% |
| rust-book/src/ch15-01-box.md | 24.894 | 24.823 | -0.43%, -0.36%, +0.51% |
| rust-book/src/ch15-02-deref.md | 33.522 | 32.946 | -0.33%, -1.35%, -2.73% |
| rust-book/src/ch15-03-drop.md | 13.053 | 13.114 | +0.57%, +0.31%, +0.10% |
| rust-book/src/ch15-04-rc.md | 20.224 | 20.282 | +0.88%, -0.93%, +0.29% |
| rust-book/src/ch15-05-interior-mutability.md | 41.054 | 41.736 | +1.86%, -2.92%, -1.61% |
| rust-book/src/ch15-06-reference-cycles.md | 41.466 | 41.902 | +1.05%, +1.88%, -0.55% |
| rust-book/src/ch16-00-concurrency.md | 4.930 | 4.965 | -0.65%, +2.02%, +0.75% |
| rust-book/src/ch16-01-threads.md | 19.594 | 19.552 | -0.21%, -1.19%, -0.98% |
| rust-book/src/ch16-02-message-passing.md | 18.702 | 18.979 | +1.48%, +0.62%, +0.63% |
| rust-book/src/ch16-03-shared-state.md | 24.687 | 24.398 | -1.46%, -1.29%, +0.74% |
| rust-book/src/ch16-04-extensible-concurrency-sync-and-send.md | 12.565 | 12.387 | -1.04%, -1.78%, -1.41% |
| rust-book/src/ch17-00-async-await.md | 12.248 | 12.231 | +1.09%, -0.16%, +0.65% |
| rust-book/src/ch17-01-futures-and-syntax.md | 42.674 | 42.178 | +1.41%, -1.32%, -1.16% |
| rust-book/src/ch17-02-concurrency-with-async.md | 38.979 | 39.072 | +1.08%, +0.83%, -0.30% |
| rust-book/src/ch17-03-more-futures.md | 19.944 | 20.155 | +1.54%, -1.02%, +0.37% |
| rust-book/src/ch17-04-streams.md | 9.788 | 9.708 | -0.82%, +0.35%, -0.62% |
| rust-book/src/ch17-05-traits-for-async.md | 61.405 | 61.280 | +0.35%, -1.22%, -0.37% |
| rust-book/src/ch17-06-futures-tasks-threads.md | 9.716 | 9.727 | +0.76%, -0.18%, -0.27% |
| rust-book/src/ch18-00-oop.md | 1.939 | 1.929 | -0.85%, +1.99%, -1.92% |
| rust-book/src/ch18-01-what-is-oo.md | 14.116 | 14.134 | -0.73%, +0.05%, +0.37% |
| rust-book/src/ch18-02-trait-objects.md | 26.072 | 26.169 | +0.54%, +0.81%, +0.37% |
| rust-book/src/ch18-03-oo-design-patterns.md | 63.596 | 64.139 | +0.73%, -0.18%, +0.85% |
| rust-book/src/ch19-00-patterns.md | 2.655 | 2.646 | +0.23%, -0.82%, -0.34% |
| rust-book/src/ch19-01-all-the-places-for-patterns.md | 22.094 | 22.143 | +0.79%, -1.74%, +1.05% |
| rust-book/src/ch19-02-refutability.md | 7.590 | 7.618 | +0.31%, +0.31%, +0.36% |
| rust-book/src/ch19-03-pattern-syntax.md | 62.862 | 62.142 | +1.03%, -1.66%, -2.15% |
| rust-book/src/ch20-00-advanced-features.md | 2.173 | 2.197 | +1.62%, +1.15%, +1.04% |
| rust-book/src/ch20-01-unsafe-rust.md | 53.760 | 53.498 | -3.28%, +1.99%, +1.05% |
| rust-book/src/ch20-02-advanced-traits.md | 47.830 | 48.096 | +2.19%, +1.14%, -0.54% |
| rust-book/src/ch20-03-advanced-types.md | 31.462 | 31.836 | +1.36%, +0.32%, -0.24% |
| rust-book/src/ch20-04-advanced-functions-and-closures.md | 17.487 | 17.273 | -0.90%, -1.24%, -1.47% |
| rust-book/src/ch20-05-macros.md | 54.646 | 54.162 | -0.89%, -0.97%, +1.55% |
| rust-book/src/ch21-00-final-project-a-web-server.md | 3.786 | 3.808 | +0.70%, +0.14%, +0.27% |
| rust-book/src/ch21-01-single-threaded.md | 39.405 | 39.183 | -0.61%, +0.43%, +0.60% |
| rust-book/src/ch21-02-multithreaded.md | 76.627 | 76.461 | -0.22%, -1.48%, +1.56% |
| rust-book/src/ch21-03-graceful-shutdown-and-cleanup.md | 20.969 | 21.148 | +0.25%, -1.12%, +1.79% |
| rust-book/src/foreword.md | 3.419 | 3.450 | +1.24%, +1.48%, +0.49% |
| rust-book/src/title-page.md | 5.500 | 5.437 | -0.28%, -2.34%, -1.01% |
| typescript-handbook/packages/documentation/copy/en/Nightly Builds.md | 7.749 | 7.820 | -0.10%, +1.40%, +1.41% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/By Example.md | 13.675 | 13.560 | -1.33%, -1.03%, +0.02% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Consumption.md | 4.049 | 4.094 | +0.06%, +1.11%, -1.14% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Deep Dive.md | 16.838 | 16.784 | -0.17%, -0.97%, -0.36% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Do's and Don'ts.md | 14.048 | 13.853 | -1.38%, -1.54%, -0.27% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Introduction.md | 6.654 | 6.723 | +3.23%, +1.04%, +1.73% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Library Structures.md | 25.200 | 25.091 | -1.12%, +1.01%, +0.98% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Publishing.md | 16.712 | 16.663 | -0.29%, -0.38%, +0.34% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/Templates.md | 3.312 | 3.321 | +0.38%, -0.62%, +0.28% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/global-modifying-module.d.ts.md | 2.899 | 2.890 | -0.31%, +1.31%, -0.83% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/global-plugin.d.ts.md | 18.928 | 18.959 | +0.16%, +0.11%, -0.06% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/global.d.ts.md | 6.551 | 6.609 | +0.89%, -0.14%, +1.14% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/module-class.d.ts.md | 2.012 | 2.002 | -0.51%, +0.69%, +0.88% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/module-function.d.ts.md | 2.035 | 2.054 | -0.32%, +1.06%, -0.03% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/module-plugin.d.ts.md | 2.647 | 2.652 | -0.20%, +0.19%, -0.88% |
| typescript-handbook/packages/documentation/copy/en/declaration-files/templates/module.d.ts.md | 12.915 | 12.788 | -0.98%, -0.78%, -0.50% |
| typescript-handbook/packages/documentation/copy/en/get-started/TS for Functional Programmers.md | 43.613 | 43.243 | +0.07%, -1.62%, +0.49% |
| typescript-handbook/packages/documentation/copy/en/get-started/TS for JS Programmers.md | 14.198 | 14.142 | -0.58%, +1.01%, +0.58% |
| typescript-handbook/packages/documentation/copy/en/get-started/TS for OOPers.md | 13.551 | 13.430 | -1.31%, -0.53%, -0.94% |
| typescript-handbook/packages/documentation/copy/en/get-started/TS for the New Programmer.md | 13.761 | 13.669 | -1.45%, +0.15%, +0.79% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Basic Types.md | 23.004 | 22.648 | -0.81%, -1.72%, -0.51% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Classes.md | 27.796 | 27.832 | +0.13%, +0.69%, -1.03% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Functions.md | 24.944 | 24.870 | -0.36%, +0.80%, -1.38% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Generics.md | 17.238 | 17.088 | -2.73%, -0.73%, +0.28% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Interfaces.md | 29.074 | 29.168 | -0.23%, +0.36%, +0.04% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Literal Types.md | 4.943 | 4.956 | +0.27%, +1.40%, -0.96% |
| typescript-handbook/packages/documentation/copy/en/handbook-v1/Unions and Intersections.md | 14.747 | 14.566 | -0.71%, -2.85%, -0.30% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Basics.md | 29.991 | 30.416 | +0.65%, +1.68%, +1.18% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Classes.md | 66.088 | 68.310 | +0.58%, +3.36%, -0.04% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Everyday Types.md | 52.837 | 52.986 | +2.89%, +0.28%, +1.30% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Modules.md | 20.943 | 20.683 | -0.78%, -1.64%, -0.01% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/More on Functions.md | 46.698 | 48.105 | +3.01%, +1.13%, +1.03% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Narrowing.md | 41.888 | 41.659 | -3.45%, +1.66%, -0.86% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Object Types.md | 48.855 | 50.042 | +2.43%, +1.25%, +0.29% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/The Handbook.md | 6.591 | 6.644 | +0.03%, +1.71%, +0.62% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Declarations.md | 10.499 | 10.429 | -0.31%, -0.66%, +0.27% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Conditional Types.md | 10.945 | 10.926 | -0.76%, +0.87%, +1.24% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Generics.md | 29.630 | 30.049 | +1.55%, +1.41%, -1.37% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Indexed Access Types.md | 3.273 | 3.257 | -1.19%, +0.78%, +0.14% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Keyof Type Operator.md | 2.678 | 2.657 | -0.55%, -0.02%, -1.56% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Mapped Types.md | 6.397 | 6.362 | -0.59%, -1.49%, -0.55% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Template Literal Types.md | 14.389 | 14.283 | -0.10%, -0.55%, -1.26% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/Typeof Type Operator.md | 4.703 | 4.685 | -0.38%, -0.91%, -0.45% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Type Manipulation/_Creating Types from Types.md | 4.301 | 4.287 | -0.32%, -1.30%, -0.34% |
| typescript-handbook/packages/documentation/copy/en/handbook-v2/Understanding Errors.md | 6.725 | 6.690 | +0.14%, -0.57%, -0.99% |
| typescript-handbook/packages/documentation/copy/en/javascript/Creating DTS files From JS.md | 7.471 | 7.488 | -0.35%, -1.20%, +0.25% |
| typescript-handbook/packages/documentation/copy/en/javascript/Intro to JS with TS.md | 5.644 | 5.665 | +0.38%, +1.67%, -0.52% |
| typescript-handbook/packages/documentation/copy/en/javascript/JSDoc Reference.md | 45.592 | 46.048 | +1.00%, +0.62%, +1.86% |
| typescript-handbook/packages/documentation/copy/en/javascript/Type Checking JavaScript Files.md | 12.125 | 12.154 | +1.15%, -0.47%, -0.55% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/Introduction.md | 3.213 | 3.223 | +0.32%, +0.65%, +0.68% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/Reference.md | 180.269 | 179.641 | -0.35%, -0.94%, -0.50% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/Theory.md | 71.847 | 72.931 | +0.53%, +0.27%, +3.78% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/appendices/ESM-CJS-Interop.md | 29.777 | 30.716 | +3.15%, +3.66%, +1.69% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/diagrams/esm-cjs-interop.md | 0.680 | 0.679 | -0.92%, -0.10%, -0.60% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/diagrams/theory.md | 1.090 | 1.089 | -0.83%, +1.10%, -0.94% |
| typescript-handbook/packages/documentation/copy/en/modules-reference/guides/Choosing Compiler Options.md | 20.978 | 21.119 | +0.07%, -1.92%, +0.87% |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options in MSBuild.md | 16.112 | 16.042 | +0.08%, -0.51%, -1.41% |
| typescript-handbook/packages/documentation/copy/en/project-config/Compiler Options.md | 27.457 | 27.335 | -0.35%, -1.42%, -0.80% |
| typescript-handbook/packages/documentation/copy/en/project-config/Configuring Watch.md | 9.496 | 9.479 | -0.18%, +0.31%, +0.23% |
| typescript-handbook/packages/documentation/copy/en/project-config/Integrating with Build Tools.md | 16.429 | 16.378 | +0.63%, -1.02%, -0.31% |
| typescript-handbook/packages/documentation/copy/en/project-config/Project References.md | 20.603 | 20.488 | -0.56%, -0.19%, +0.23% |
| typescript-handbook/packages/documentation/copy/en/project-config/tsconfig.json.md | 8.984 | 8.892 | -1.69%, -1.02%, +0.40% |
| typescript-handbook/packages/documentation/copy/en/reference/Advanced Types.md | 66.021 | 66.365 | +0.52%, -4.14%, -0.96% |
| typescript-handbook/packages/documentation/copy/en/reference/Declaration Merging.md | 13.421 | 13.399 | -0.38%, -0.11%, -0.04% |
| typescript-handbook/packages/documentation/copy/en/reference/Decorators.md | 33.197 | 32.537 | -1.99%, +2.33%, -0.42% |
| typescript-handbook/packages/documentation/copy/en/reference/Enums.md | 25.312 | 25.293 | +0.44%, +0.44%, -1.75% |
| typescript-handbook/packages/documentation/copy/en/reference/Iterators and Generators.md | 7.497 | 7.588 | +1.21%, +0.50%, +0.18% |
| typescript-handbook/packages/documentation/copy/en/reference/JSX.md | 33.597 | 34.272 | +2.01%, +1.73%, +0.73% |
| typescript-handbook/packages/documentation/copy/en/reference/Mixins.md | 7.653 | 7.595 | -1.26%, +3.76%, -0.15% |
| typescript-handbook/packages/documentation/copy/en/reference/Namespaces and Modules.md | 11.618 | 11.529 | -0.76%, +0.73%, +0.64% |
| typescript-handbook/packages/documentation/copy/en/reference/Namespaces.md | 12.598 | 12.482 | -1.38%, -1.78%, +0.12% |
| typescript-handbook/packages/documentation/copy/en/reference/Symbols.md | 8.999 | 8.940 | -0.65%, -3.49%, -0.76% |
| typescript-handbook/packages/documentation/copy/en/reference/Triple-Slash Directives.md | 15.254 | 15.299 | +0.30%, -0.79%, -0.87% |
| typescript-handbook/packages/documentation/copy/en/reference/Type Compatibility.md | 20.199 | 19.992 | -0.14%, -1.09%, +0.54% |
| typescript-handbook/packages/documentation/copy/en/reference/Type Inference.md | 8.044 | 8.038 | -0.89%, -0.08%, -0.05% |
| typescript-handbook/packages/documentation/copy/en/reference/Utility Types.md | 34.271 | 34.548 | +1.25%, +0.85%, +0.07% |
| typescript-handbook/packages/documentation/copy/en/reference/Variable Declarations.md | 50.407 | 50.361 | -2.93%, -0.09%, +0.65% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.1.md | 2.403 | 2.424 | +1.34%, +1.34%, -0.43% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.3.md | 2.580 | 2.546 | +0.85%, -1.33%, -2.30% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.4.md | 11.479 | 11.455 | -0.09%, -1.04%, +0.85% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.5.md | 23.632 | 23.752 | +0.51%, +0.27%, -0.30% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.6.md | 19.557 | 19.489 | -0.49%, +0.71%, -0.79% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.7.md | 10.553 | 10.573 | -0.77%, +0.34%, +0.51% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 1.8.md | 39.246 | 38.952 | +1.37%, -0.75%, -1.18% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.0.md | 69.674 | 69.970 | +4.74%, +0.42%, -0.19% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.1.md | 32.141 | 32.183 | +2.37%, -1.49%, +1.71% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.2.md | 15.431 | 15.327 | -0.11%, -0.79%, +0.64% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.3.md | 19.389 | 19.634 | +0.40%, +1.33%, +1.24% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.4.md | 9.042 | 8.990 | -0.99%, +0.79%, -0.69% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.5.md | 6.627 | 6.573 | -1.38%, -0.80%, -0.71% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.6.md | 17.992 | 18.004 | +0.07%, -0.08%, -0.16% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.7.md | 22.059 | 21.840 | -0.08%, -1.36%, -0.04% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.8.md | 29.304 | 28.835 | -1.60%, -0.74%, +0.12% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 2.9.md | 19.882 | 20.144 | +0.03%, +1.32%, +1.23% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.0.md | 18.026 | 18.011 | -0.11%, -0.30%, +0.52% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.1.md | 6.923 | 6.815 | -1.00%, -1.56%, -1.57% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.2.md | 14.558 | 14.574 | -0.58%, +0.54%, -0.26% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.3.md | 8.085 | 8.110 | -1.11%, -0.15%, +0.40% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.4.md | 27.633 | 27.469 | -0.87%, -0.26%, -0.79% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.5.md | 13.333 | 13.199 | -0.84%, -1.40%, -0.33% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.6.md | 21.192 | 20.899 | -1.39%, -0.66%, +0.36% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.7.md | 49.453 | 49.656 | +1.09%, +0.41%, +1.27% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.8.md | 30.419 | 30.451 | +0.11%, +0.95%, +0.39% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 3.9.md | 34.310 | 34.091 | -0.64%, -0.28%, -1.03% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.0.md | 47.612 | 47.446 | +0.41%, -0.35%, -1.32% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.1.md | 36.171 | 35.492 | -1.54%, -1.88%, -0.43% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.2.md | 37.533 | 36.719 | +0.94%, -3.18%, -1.53% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.3.md | 46.419 | 47.321 | +5.68%, +1.94%, +1.73% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.4.md | 44.913 | 45.237 | +0.72%, +2.02%, +2.68% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.5.md | 34.254 | 34.150 | -1.94%, +1.26%, +0.50% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.6.md | 27.487 | 27.210 | -1.01%, -3.07%, -0.77% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.7.md | 73.698 | 74.013 | -0.49%, +0.43%, +4.02% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.8.md | 32.814 | 33.395 | +3.33%, +1.77%, -0.99% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 4.9.md | 34.320 | 34.725 | +0.55%, +1.18%, +0.30% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | 99.080 | 99.127 | -1.36%, +1.68%, +0.77% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.1.md | 26.745 | 26.851 | +0.39%, +2.50%, -1.92% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.2.md | 46.921 | 48.236 | -1.06%, +4.22%, +0.65% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.3.md | 29.774 | 29.685 | -0.49%, +0.17%, -0.21% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.4.md | 36.847 | 37.703 | +1.72%, +2.32%, +3.27% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.5.md | 85.736 | 86.143 | +2.51%, -0.13%, +3.11% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.6.md | 52.823 | 53.533 | -2.61%, +2.57%, +3.91% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.7.md | 32.308 | 32.401 | +0.29%, +0.51%, +1.56% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.8.md | 21.216 | 21.415 | -0.56%, +1.46%, +0.09% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.9.md | 21.147 | 21.210 | +1.50%, +0.01%, -0.29% |
| typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | 83.241 | 84.478 | +1.43%, +1.49%, -0.03% |
| typescript-handbook/packages/documentation/copy/en/tutorials/ASP.NET Core.md | 18.048 | 18.078 | -1.22%, +0.41%, -0.92% |
| typescript-handbook/packages/documentation/copy/en/tutorials/Angular.md | 1.849 | 1.843 | -0.43%, +0.78%, -1.16% |
| typescript-handbook/packages/documentation/copy/en/tutorials/Babel with TypeScript.md | 5.884 | 5.910 | -0.35%, -0.19%, +2.34% |
| typescript-handbook/packages/documentation/copy/en/tutorials/DOM Manipulation.md | 19.760 | 19.570 | +0.71%, -0.96%, -0.60% |
| typescript-handbook/packages/documentation/copy/en/tutorials/Gulp.md | 23.567 | 23.606 | +0.50%, -0.04%, +0.29% |
| typescript-handbook/packages/documentation/copy/en/tutorials/Migrating from JavaScript.md | 30.262 | 29.960 | -1.75%, -1.37%, +2.76% |
| typescript-handbook/packages/documentation/copy/en/tutorials/React.md | 4.704 | 4.764 | +2.03%, +0.52%, +0.24% |
| typescript-handbook/packages/documentation/copy/en/tutorials/TypeScript Tooling in 5 minutes.md | 8.861 | 8.871 | +0.94%, +0.71%, -1.43% |
