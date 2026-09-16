# Position against the preceding native comparison

Identical library pins, options, and agreeing input sets. Positive change means
v2 improved its relative throughput against that engine; negative means it lost
ground. These are ratios from separate runs, not isolated causal measurements
of one optimization. Inspect process-round variation before interpreting small changes.

The preceding run is [2026-09-15-release-native](../2026-09-15-release-native/README.md)
with v2 at `c232d97d`; this run measures v2 at `7c887a2b`. The other five engine
pins, the flags, the timed loops and the 57 frozen inputs are unchanged.

| Group | N | Lifecycle | Against | Previous v2 throughput | Current | Position change |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| all-six | 14 | fresh | OX-Content original | 0.986× | 1.189× | +20.55% |
| all-six | 14 | fresh | Ferromark v1 | 1.388× | 1.673× | +20.52% |
| all-six | 14 | fresh | md4c | 3.746× | 4.522× | +20.72% |
| all-six | 14 | fresh | pulldown-cmark | 2.122× | 2.568× | +21.04% |
| all-six | 14 | fresh | Bun native bun_md | 5.893× | 7.468× | +26.73% |
| all-six | 14 | reuse | OX-Content original | 0.991× | 1.084× | +9.28% |
| all-six | 14 | reuse | Ferromark v1 | 1.365× | 1.493× | +9.37% |
| all-six | 14 | reuse | md4c | 4.742× | 5.179× | +9.21% |
| all-six | 14 | reuse | pulldown-cmark | 2.564× | 2.803× | +9.30% |
| all-six | 14 | reuse | Bun native bun_md | 7.840× | 8.988× | +14.65% |
| configurable-five | 50 | fresh | Ferromark v1 | 1.855× | 2.074× | +11.85% |
| configurable-five | 50 | fresh | md4c | 3.143× | 3.507× | +11.57% |
| configurable-five | 50 | fresh | pulldown-cmark | 2.375× | 2.648× | +11.47% |
| configurable-five | 50 | fresh | Bun native bun_md | 5.369× | 6.277× | +16.91% |
| configurable-five | 50 | reuse | Ferromark v1 | 1.802× | 1.938× | +7.58% |
| configurable-five | 50 | reuse | md4c | 3.394× | 3.644× | +7.38% |
| configurable-five | 50 | reuse | pulldown-cmark | 2.523× | 2.702× | +7.09% |
| configurable-five | 50 | reuse | Bun native bun_md | 5.989× | 6.737× | +12.48% |
| comments | 11 | fresh | Ferromark v1 | 1.291× | 1.660× | +28.65% |
| comments | 11 | fresh | md4c | 3.464× | 4.462× | +28.80% |
| comments | 11 | fresh | pulldown-cmark | 1.737× | 2.243× | +29.07% |
| comments | 11 | fresh | Bun native bun_md | 3.841× | 5.215× | +35.78% |
| comments | 11 | reuse | Ferromark v1 | 1.311× | 1.486× | +13.39% |
| comments | 11 | reuse | md4c | 4.745× | 5.385× | +13.47% |
| comments | 11 | reuse | pulldown-cmark | 2.245× | 2.549× | +13.53% |
| comments | 11 | reuse | Bun native bun_md | 5.550× | 6.641× | +19.66% |
| encyclopedia | 8 | fresh | Ferromark v1 | 3.220× | 3.349× | +4.01% |
| encyclopedia | 8 | fresh | md4c | 4.041× | 4.185× | +3.56% |
| encyclopedia | 8 | fresh | pulldown-cmark | 3.286× | 3.436× | +4.57% |
| encyclopedia | 8 | fresh | Bun native bun_md | 6.179× | 6.749× | +9.23% |
| encyclopedia | 8 | reuse | Ferromark v1 | 3.144× | 3.194× | +1.58% |
| encyclopedia | 8 | reuse | md4c | 4.214× | 4.266× | +1.22% |
| encyclopedia | 8 | reuse | pulldown-cmark | 3.421× | 3.481× | +1.75% |
| encyclopedia | 8 | reuse | Bun native bun_md | 6.600× | 7.041× | +6.70% |
| plain-prose | 4 | fresh | Ferromark v1 | 1.523× | 1.639× | +7.61% |
| plain-prose | 4 | fresh | md4c | 3.800× | 4.102× | +7.95% |
| plain-prose | 4 | fresh | pulldown-cmark | 3.263× | 3.528× | +8.12% |
| plain-prose | 4 | fresh | Bun native bun_md | 16.391× | 18.397× | +12.24% |
| plain-prose | 4 | reuse | Ferromark v1 | 1.350× | 1.464× | +8.48% |
| plain-prose | 4 | reuse | md4c | 3.664× | 3.948× | +7.75% |
| plain-prose | 4 | reuse | pulldown-cmark | 3.131× | 3.368× | +7.56% |
| plain-prose | 4 | reuse | Bun native bun_md | 16.443× | 18.455× | +12.24% |
| readme | 2 | fresh | Ferromark v1 | 1.671× | 1.734× | +3.74% |
| readme | 2 | fresh | md4c | 2.493× | 2.591× | +3.90% |
| readme | 2 | fresh | pulldown-cmark | 2.028× | 2.092× | +3.15% |
| readme | 2 | fresh | Bun native bun_md | 3.981× | 4.308× | +8.23% |
| readme | 2 | reuse | Ferromark v1 | 1.587× | 1.622× | +2.21% |
| readme | 2 | reuse | md4c | 2.492× | 2.546× | +2.17% |
| readme | 2 | reuse | pulldown-cmark | 2.006× | 2.042× | +1.79% |
| readme | 2 | reuse | Bun native bun_md | 4.070× | 4.329× | +6.35% |
| reference | 4 | fresh | Ferromark v1 | 1.668× | 1.822× | +9.26% |
| reference | 4 | fresh | md4c | 2.400× | 2.609× | +8.68% |
| reference | 4 | fresh | pulldown-cmark | 1.745× | 1.850× | +5.98% |
| reference | 4 | fresh | Bun native bun_md | 3.572× | 3.933× | +10.08% |
| reference | 4 | reuse | Ferromark v1 | 1.629× | 1.769× | +8.62% |
| reference | 4 | reuse | md4c | 2.372× | 2.569× | +8.32% |
| reference | 4 | reuse | pulldown-cmark | 1.719× | 1.822× | +5.98% |
| reference | 4 | reuse | Bun native bun_md | 3.592× | 3.947× | +9.88% |
| technical-docs | 21 | fresh | Ferromark v1 | 1.945× | 2.118× | +8.91% |
| technical-docs | 21 | fresh | md4c | 2.817× | 3.054× | +8.40% |
| technical-docs | 21 | fresh | pulldown-cmark | 2.505× | 2.711× | +8.22% |
| technical-docs | 21 | fresh | Bun native bun_md | 5.453× | 6.211× | +13.91% |
| technical-docs | 21 | reuse | Ferromark v1 | 1.877× | 2.011× | +7.13% |
| technical-docs | 21 | reuse | md4c | 2.849× | 3.047× | +6.94% |
| technical-docs | 21 | reuse | pulldown-cmark | 2.520× | 2.684× | +6.54% |
| technical-docs | 21 | reuse | Bun native bun_md | 5.667× | 6.361× | +12.24% |
