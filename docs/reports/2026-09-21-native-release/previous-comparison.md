# Position against the preceding native comparison

Identical library pins, options, and agreeing input sets. Positive change means
v2 improved its relative throughput against that engine; negative means it lost
ground. These are ratios from separate runs, not isolated causal measurements
of one optimization. Inspect process-round variation before interpreting small changes.

The preceding run is [2026-09-16-native-segments](../2026-09-16-native-segments/README.md)
with v2 at `7c887a2b`; this run measures v2 at `bffc89f6`. The other five engine
pins, the flags, the timed loops and the 57 frozen inputs are unchanged.

| Group | N | Lifecycle | Against | Previous v2 throughput | Current | Position change |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| all-six | 14 | fresh | OX-Content original | 1.189× | 1.069× | -10.04% |
| all-six | 14 | fresh | Ferromark v1 | 1.673× | 1.501× | -10.27% |
| all-six | 14 | fresh | md4c | 4.522× | 4.063× | -10.16% |
| all-six | 14 | fresh | pulldown-cmark | 2.568× | 2.283× | -11.11% |
| all-six | 14 | fresh | Bun native bun_md | 7.468× | 6.387× | -14.48% |
| all-six | 14 | reuse | OX-Content original | 1.084× | 0.954× | -11.93% |
| all-six | 14 | reuse | Ferromark v1 | 1.493× | 1.313× | -12.06% |
| all-six | 14 | reuse | md4c | 5.179× | 4.578× | -11.59% |
| all-six | 14 | reuse | pulldown-cmark | 2.803× | 2.461× | -12.22% |
| all-six | 14 | reuse | Bun native bun_md | 8.988× | 7.584× | -15.62% |
| configurable-five | 50 | fresh | Ferromark v1 | 2.074× | 1.922× | -7.35% |
| configurable-five | 50 | fresh | md4c | 3.507× | 3.240× | -7.60% |
| configurable-five | 50 | fresh | pulldown-cmark | 2.648× | 2.432× | -8.16% |
| configurable-five | 50 | fresh | Bun native bun_md | 6.277× | 5.546× | -11.64% |
| configurable-five | 50 | reuse | Ferromark v1 | 1.938× | 1.784× | -7.95% |
| configurable-five | 50 | reuse | md4c | 3.644× | 3.345× | -8.22% |
| configurable-five | 50 | reuse | pulldown-cmark | 2.702× | 2.468× | -8.64% |
| configurable-five | 50 | reuse | Bun native bun_md | 6.737× | 5.920× | -12.12% |
| comments | 11 | fresh | Ferromark v1 | 1.660× | 1.468× | -11.58% |
| comments | 11 | fresh | md4c | 4.462× | 3.958× | -11.29% |
| comments | 11 | fresh | pulldown-cmark | 2.243× | 1.960× | -12.59% |
| comments | 11 | fresh | Bun native bun_md | 5.215× | 4.377× | -16.07% |
| comments | 11 | reuse | Ferromark v1 | 1.486× | 1.278× | -13.97% |
| comments | 11 | reuse | md4c | 5.385× | 4.661× | -13.43% |
| comments | 11 | reuse | pulldown-cmark | 2.549× | 2.185× | -14.28% |
| comments | 11 | reuse | Bun native bun_md | 6.641× | 5.450× | -17.93% |
| encyclopedia | 8 | fresh | Ferromark v1 | 3.349× | 3.207× | -4.24% |
| encyclopedia | 8 | fresh | md4c | 4.185× | 4.014× | -4.09% |
| encyclopedia | 8 | fresh | pulldown-cmark | 3.436× | 3.273× | -4.74% |
| encyclopedia | 8 | fresh | Bun native bun_md | 6.749× | 6.155× | -8.80% |
| encyclopedia | 8 | reuse | Ferromark v1 | 3.194× | 3.031× | -5.09% |
| encyclopedia | 8 | reuse | md4c | 4.266× | 4.051× | -5.04% |
| encyclopedia | 8 | reuse | pulldown-cmark | 3.481× | 3.300× | -5.20% |
| encyclopedia | 8 | reuse | Bun native bun_md | 7.041× | 6.372× | -9.50% |
| plain-prose | 4 | fresh | Ferromark v1 | 1.639× | 1.475× | -10.02% |
| plain-prose | 4 | fresh | md4c | 4.102× | 3.676× | -10.39% |
| plain-prose | 4 | fresh | pulldown-cmark | 3.528× | 3.173× | -10.07% |
| plain-prose | 4 | fresh | Bun native bun_md | 18.397× | 15.954× | -13.28% |
| plain-prose | 4 | reuse | Ferromark v1 | 1.464× | 1.320× | -9.83% |
| plain-prose | 4 | reuse | md4c | 3.948× | 3.566× | -9.68% |
| plain-prose | 4 | reuse | pulldown-cmark | 3.368× | 3.049× | -9.49% |
| plain-prose | 4 | reuse | Bun native bun_md | 18.455× | 16.119× | -12.66% |
| readme | 2 | fresh | Ferromark v1 | 1.734× | 1.714× | -1.16% |
| readme | 2 | fresh | md4c | 2.591× | 2.520× | -2.74% |
| readme | 2 | fresh | pulldown-cmark | 2.092× | 2.020× | -3.42% |
| readme | 2 | fresh | Bun native bun_md | 4.308× | 4.011× | -6.90% |
| readme | 2 | reuse | Ferromark v1 | 1.622× | 1.607× | -0.94% |
| readme | 2 | reuse | md4c | 2.546× | 2.504× | -1.67% |
| readme | 2 | reuse | pulldown-cmark | 2.042× | 1.970× | -3.50% |
| readme | 2 | reuse | Bun native bun_md | 4.329× | 4.058× | -6.25% |
| reference | 4 | fresh | Ferromark v1 | 1.822× | 1.723× | -5.45% |
| reference | 4 | fresh | md4c | 2.609× | 2.447× | -6.18% |
| reference | 4 | fresh | pulldown-cmark | 1.850× | 1.742× | -5.83% |
| reference | 4 | fresh | Bun native bun_md | 3.933× | 3.636× | -7.55% |
| reference | 4 | reuse | Ferromark v1 | 1.769× | 1.679× | -5.12% |
| reference | 4 | reuse | md4c | 2.569× | 2.409× | -6.21% |
| reference | 4 | reuse | pulldown-cmark | 1.822× | 1.696× | -6.93% |
| reference | 4 | reuse | Bun native bun_md | 3.947× | 3.651× | -7.50% |
| technical-docs | 21 | fresh | Ferromark v1 | 2.118× | 1.977× | -6.67% |
| technical-docs | 21 | fresh | md4c | 3.054× | 2.837× | -7.12% |
| technical-docs | 21 | fresh | pulldown-cmark | 2.711× | 2.506× | -7.57% |
| technical-docs | 21 | fresh | Bun native bun_md | 6.211× | 5.515× | -11.21% |
| technical-docs | 21 | reuse | Ferromark v1 | 2.011× | 1.879× | -6.56% |
| technical-docs | 21 | reuse | md4c | 3.047× | 2.824× | -7.30% |
| technical-docs | 21 | reuse | pulldown-cmark | 2.684× | 2.483× | -7.51% |
| technical-docs | 21 | reuse | Bun native bun_md | 6.361× | 5.645× | -11.25% |
