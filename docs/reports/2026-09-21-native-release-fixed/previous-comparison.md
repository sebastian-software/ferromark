# Position against the preceding native comparison

Identical library pins, options, and agreeing input sets. Positive change means
v2 improved its relative throughput against that engine; negative means it lost
ground. These are ratios from separate runs, not isolated causal measurements
of one optimization. Inspect process-round variation before interpreting small changes.

The preceding run is [2026-09-16-native-segments](../2026-09-16-native-segments/README.md)
with v2 at `7c887a2b`; this run measures v2 at `60602a5a`. The other five engine
pins, the flags, the timed loops and the 57 frozen inputs are unchanged.

| Group | N | Lifecycle | Against | Previous v2 throughput | Current | Position change |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| all-six | 14 | fresh | OX-Content original | 1.189× | 1.149× | -3.35% |
| all-six | 14 | fresh | Ferromark v1 | 1.673× | 1.619× | -3.22% |
| all-six | 14 | fresh | md4c | 4.522× | 4.910× | +8.58% |
| all-six | 14 | fresh | pulldown-cmark | 2.568× | 2.469× | -3.88% |
| all-six | 14 | fresh | Bun native bun_md | 7.468× | 6.900× | -7.61% |
| all-six | 14 | reuse | OX-Content original | 1.084× | 1.040× | -3.99% |
| all-six | 14 | reuse | Ferromark v1 | 1.493× | 1.437× | -3.78% |
| all-six | 14 | reuse | md4c | 5.179× | 5.638× | +8.86% |
| all-six | 14 | reuse | pulldown-cmark | 2.803× | 2.691× | -4.01% |
| all-six | 14 | reuse | Bun native bun_md | 8.988× | 8.275× | -7.94% |
| configurable-five | 50 | fresh | Ferromark v1 | 2.074× | 1.981× | -4.51% |
| configurable-five | 50 | fresh | md4c | 3.507× | 3.543× | +1.04% |
| configurable-five | 50 | fresh | pulldown-cmark | 2.648× | 2.529× | -4.48% |
| configurable-five | 50 | fresh | Bun native bun_md | 6.277× | 5.740× | -8.56% |
| configurable-five | 50 | reuse | Ferromark v1 | 1.938× | 1.847× | -4.70% |
| configurable-five | 50 | reuse | md4c | 3.644× | 3.686× | +1.15% |
| configurable-five | 50 | reuse | pulldown-cmark | 2.702× | 2.580× | -4.49% |
| configurable-five | 50 | reuse | Bun native bun_md | 6.737× | 6.148× | -8.73% |
| comments | 11 | fresh | Ferromark v1 | 1.660× | 1.607× | -3.20% |
| comments | 11 | fresh | md4c | 4.462× | 5.033× | +12.80% |
| comments | 11 | fresh | pulldown-cmark | 2.243× | 2.152× | -4.03% |
| comments | 11 | fresh | Bun native bun_md | 5.215× | 4.803× | -7.90% |
| comments | 11 | reuse | Ferromark v1 | 1.486× | 1.425× | -4.08% |
| comments | 11 | reuse | md4c | 5.385× | 6.081× | +12.94% |
| comments | 11 | reuse | pulldown-cmark | 2.549× | 2.439× | -4.31% |
| comments | 11 | reuse | Bun native bun_md | 6.641× | 6.077× | -8.49% |
| encyclopedia | 8 | fresh | Ferromark v1 | 3.349× | 3.165× | -5.50% |
| encyclopedia | 8 | fresh | md4c | 4.185× | 4.110× | -1.80% |
| encyclopedia | 8 | fresh | pulldown-cmark | 3.436× | 3.239× | -5.75% |
| encyclopedia | 8 | fresh | Bun native bun_md | 6.749× | 6.075× | -9.99% |
| encyclopedia | 8 | reuse | Ferromark v1 | 3.194× | 2.993× | -6.29% |
| encyclopedia | 8 | reuse | md4c | 4.266× | 4.160× | -2.49% |
| encyclopedia | 8 | reuse | pulldown-cmark | 3.481× | 3.262× | -6.27% |
| encyclopedia | 8 | reuse | Bun native bun_md | 7.041× | 6.288× | -10.69% |
| plain-prose | 4 | fresh | Ferromark v1 | 1.639× | 1.589× | -3.05% |
| plain-prose | 4 | fresh | md4c | 4.102× | 3.976× | -3.08% |
| plain-prose | 4 | fresh | pulldown-cmark | 3.528× | 3.414× | -3.22% |
| plain-prose | 4 | fresh | Bun native bun_md | 18.397× | 17.126× | -6.91% |
| plain-prose | 4 | reuse | Ferromark v1 | 1.464× | 1.420× | -3.02% |
| plain-prose | 4 | reuse | md4c | 3.948× | 3.840× | -2.74% |
| plain-prose | 4 | reuse | pulldown-cmark | 3.368× | 3.267× | -3.00% |
| plain-prose | 4 | reuse | Bun native bun_md | 18.455× | 17.175× | -6.94% |
| readme | 2 | fresh | Ferromark v1 | 1.734× | 1.651× | -4.77% |
| readme | 2 | fresh | md4c | 2.591× | 2.541× | -1.91% |
| readme | 2 | fresh | pulldown-cmark | 2.092× | 1.996× | -4.55% |
| readme | 2 | fresh | Bun native bun_md | 4.308× | 3.922× | -8.96% |
| readme | 2 | reuse | Ferromark v1 | 1.622× | 1.549× | -4.51% |
| readme | 2 | reuse | md4c | 2.546× | 2.529× | -0.70% |
| readme | 2 | reuse | pulldown-cmark | 2.042× | 1.960× | -3.99% |
| readme | 2 | reuse | Bun native bun_md | 4.329× | 3.956× | -8.61% |
| reference | 4 | fresh | Ferromark v1 | 1.822× | 1.743× | -4.37% |
| reference | 4 | fresh | md4c | 2.609× | 2.529× | -3.05% |
| reference | 4 | fresh | pulldown-cmark | 1.850× | 1.793× | -3.10% |
| reference | 4 | fresh | Bun native bun_md | 3.933× | 3.724× | -5.31% |
| reference | 4 | reuse | Ferromark v1 | 1.769× | 1.704× | -3.68% |
| reference | 4 | reuse | md4c | 2.569× | 2.513× | -2.16% |
| reference | 4 | reuse | pulldown-cmark | 1.822× | 1.777× | -2.48% |
| reference | 4 | reuse | Bun native bun_md | 3.947× | 3.755× | -4.85% |
| technical-docs | 21 | fresh | Ferromark v1 | 2.118× | 2.010× | -5.10% |
| technical-docs | 21 | fresh | md4c | 3.054× | 3.000× | -1.77% |
| technical-docs | 21 | fresh | pulldown-cmark | 2.711× | 2.583× | -4.72% |
| technical-docs | 21 | fresh | Bun native bun_md | 6.211× | 5.638× | -9.23% |
| technical-docs | 21 | reuse | Ferromark v1 | 2.011× | 1.911× | -4.94% |
| technical-docs | 21 | reuse | md4c | 3.047× | 2.996× | -1.67% |
| technical-docs | 21 | reuse | pulldown-cmark | 2.684× | 2.561× | -4.61% |
| technical-docs | 21 | reuse | Bun native bun_md | 6.361× | 5.777× | -9.18% |
