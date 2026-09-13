# Measured results

Generated from the archived raw windows and profiles. Negative experiment changes mean less elapsed time.

## Current native comparison

Medians across three fresh process groups; each group has nine rotating 60 ms windows and a 35 ms warmup. All five inputs pass the limited normalized-output comparison.

| Document | Ferromark µs | Ox growing µs | Ox presized µs | Ferromark extra time vs growing, three groups |
| --- | ---: | ---: | ---: | --- |
| Compiler Options | 27.243 | 26.559 | 26.293 | +2.90%, +2.31%, +2.41% |
| MSBuild | 16.218 | 14.368 | 14.339 | +12.60%, +12.53%, +12.87% |
| Vue render function | 39.010 | 31.626 | 31.398 | +22.46%, +21.88%, +23.76% |
| Vue Suspense | 16.806 | 10.429 | 10.456 | +62.22%, +60.31%, +61.76% |
| Rust recoverable errors | 65.121 | 40.161 | 39.799 | +61.37%, +62.15%, +64.56% |

## Visible profile attribution

Each entry shows two captures. These are sampled stack shares, not exact phase durations. Inlining and the diagnostic call boundaries change attribution. Allocation, conversion and sort shares overlap other groups.

| Document | Ferromark visible block % | Ferromark visible inline % | Ferromark allocation stacks % | Ox allocation stacks % |
| --- | ---: | ---: | ---: | ---: |
| Compiler Options | 73.13, 74.98 | 4.73, 4.27 | 4.62, 3.90 | 3.72, 3.64 |
| MSBuild | 51.06, 50.56 | 15.23, 15.88 | 6.75, 6.37 | 6.36, 5.99 |
| Vue render function | 32.79, 34.41 | 23.26, 22.67 | 5.53, 5.16 | 3.91, 4.08 |
| Vue Suspense | 17.19, 16.59 | 41.35, 41.56 | 9.85, 9.68 | 6.46, 7.10 |
| Rust recoverable errors | 22.20, 21.54 | 43.69, 44.24 | 4.61, 4.24 | 3.61, 3.77 |

### Separate attribution probe

Only three functions are forced out of line. Its 649 outputs match the regular release build. These percentages describe the probe, not a performance improvement.

| Document | Mark collection % | Event construction and sorting % | Autolink search % |
| --- | ---: | ---: | ---: |
| Vue Suspense | 8.48, 8.27 | 9.06, 9.17 | 5.76, 5.88 |
| Rust recoverable errors | 16.55, 16.39 | 16.02, 16.48 | below top 25, below top 25 |

## Independent experiments

Each correct source passed 111,902 exact before/after output comparisons before its screen. Initial screening covers 93 inputs with five alternating 30 ms windows and 60 ms warmups. Incorrect sources are excluded from timing.

| Prototype | Output result | Compiler Options % | MSBuild % | Vue render function % | Vue Suspense % | Rust chapter % |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| html-blank-run | exact output preserved | -2.46 | -0.55 | -1.47 | -0.14 | +1.74 |
| softbreak-ranges | 339 in fence-verification.json; 6 in extra-verification.json; 493 in extended-verification.json | — | — | — | — | — |
| softbreak-no-code | 267 in fence-verification.json | — | — | — | — | — |
| softbreak-guarded | exact output preserved | -0.77 | +0.76 | +2.23 | +0.45 | +5.99 |
| escape-short-copy | exact output preserved | +0.30 | -0.33 | -0.19 | -0.32 | -3.45 |
| escape-single-scan | exact output preserved | +0.00 | -1.16 | -2.34 | -1.30 | -1.37 |
| autolink-prefilter | exact output preserved | +0.55 | -0.10 | +0.34 | -1.62 | +1.06 |

### Longer confirmation

Two additional fresh process pairs per correct prototype, seven alternating 50 ms windows per input and 60 ms warmup. The 37 selected inputs include the five target documents, baseline controls, and every input over 2% slower in any of the first four correct screens. These are repeat ranges, not confidence intervals.

| Prototype | Compiler Options % | MSBuild % | Vue render function % | Vue Suspense % | Rust chapter % |
| --- | ---: | ---: | ---: | ---: | ---: |
| html-blank-run | -2.59, -3.27 | -0.81, -0.29 | -0.46, -0.18 | -0.27, +0.14 | -0.91, -0.29 |
| softbreak-guarded | +0.41, +0.31 | +0.88, +0.67 | +1.82, +1.40 | +0.84, +1.03 | +4.40, +6.78 |
| escape-short-copy | -0.34, +0.14 | +0.16, -0.32 | -1.94, +0.41 | -1.13, -0.08 | -3.08, -0.67 |
| escape-single-scan | +0.39, -1.09 | -1.10, -1.73 | -1.12, -2.41 | -2.16, -2.41 | -2.63, -3.83 |
| autolink-prefilter | +0.09, -0.32 | +0.08, -0.48 | -0.43, -0.16 | -1.63, -2.00 | -1.93, -0.26 |

The later autolink screen also flagged the headings control outside the original selection. Two additional fresh pairs measure +1.79%, +2.21%.

### Repeated control regressions

Cases more than 2% slower in both longer pairs. This prevents target-document wins from hiding repeatable losses.

| Prototype | Input | Changes % |
| --- | --- | ---: |
| html-blank-run | guard/deep-emphasis | +2.73, +2.43 |
| softbreak-guarded | inline | +2.97, +2.90 |
| softbreak-guarded | code | +2.17, +2.68 |
| softbreak-guarded | late-marker | +13.51, +13.10 |
| softbreak-guarded | commonmark/publication-5k | +2.30, +2.12 |
| softbreak-guarded | guard/links-prose | +2.67, +3.01 |
| softbreak-guarded | guard/references | +3.87, +4.08 |
| softbreak-guarded | guard/long-delimiters | +18.87, +19.30 |
| softbreak-guarded | corpus/vite-docs/docs/config/shared-options.md | +3.71, +3.72 |
| softbreak-guarded | corpus/vite-docs/docs/guide/api-plugin.md | +3.82, +2.31 |
| softbreak-guarded | corpus/rust-book/src/ch02-00-guessing-game-tutorial.md | +3.19, +3.64 |
| softbreak-guarded | corpus/rust-book/src/ch09-02-recoverable-errors-with-result.md | +4.40, +6.78 |
| softbreak-guarded | corpus/rust-book/src/ch21-02-multithreaded.md | +4.95, +4.43 |
| softbreak-guarded | corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 5.0.md | +3.99, +2.25 |
| softbreak-guarded | corpus/typescript-handbook/packages/documentation/copy/en/release-notes/TypeScript 6.0.md | +5.09, +6.27 |
| escape-short-copy | fence/root-short | +2.18, +2.85 |
| escape-short-copy | fence/root-tabs | +3.09, +2.83 |
| escape-short-copy | fence/root-crlf | +2.21, +2.33 |
