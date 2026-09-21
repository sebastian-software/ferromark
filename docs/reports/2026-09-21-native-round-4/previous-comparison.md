# Position against the preceding native comparison

Identical library pins, options, and agreeing input sets. Positive change means
v2 improved its relative throughput against that engine; negative means it lost
ground. These are ratios from separate runs, not isolated causal measurements
of one optimization. Inspect process-round variation before interpreting small changes.

The preceding run is [2026-09-21-native-release-fixed](../2026-09-21-native-release-fixed/README.md)
with v2 at `60602a5a`; this run measures v2 at `39b1f0b7`. The other five engine
pins, the flags, the timed loops and the 57 frozen inputs are unchanged.

| Group | N | Lifecycle | Against | Previous v2 throughput | Current | Position change |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| all-six | 14 | fresh | OX-Content original | 1.149× | 1.192× | +3.76% |
| all-six | 14 | fresh | Ferromark v1 | 1.619× | 1.673× | +3.33% |
| all-six | 14 | fresh | md4c | 4.910× | 4.508× | -8.19% |
| all-six | 14 | fresh | pulldown-cmark | 2.469× | 2.556× | +3.54% |
| all-six | 14 | fresh | Bun native bun_md | 6.900× | 7.148× | +3.60% |
| all-six | 14 | reuse | OX-Content original | 1.040× | 1.081× | +3.96% |
| all-six | 14 | reuse | Ferromark v1 | 1.437× | 1.485× | +3.34% |
| all-six | 14 | reuse | md4c | 5.638× | 5.163× | -8.42% |
| all-six | 14 | reuse | pulldown-cmark | 2.691× | 2.789× | +3.64% |
| all-six | 14 | reuse | Bun native bun_md | 8.275× | 8.597× | +3.90% |
| configurable-five | 50 | fresh | Ferromark v1 | 1.981× | 2.074× | +4.73% |
| configurable-five | 50 | fresh | md4c | 3.543× | 3.508× | -1.00% |
| configurable-five | 50 | fresh | pulldown-cmark | 2.529× | 2.645× | +4.60% |
| configurable-five | 50 | fresh | Bun native bun_md | 5.740× | 6.026× | +4.98% |
| configurable-five | 50 | reuse | Ferromark v1 | 1.847× | 1.929× | +4.46% |
| configurable-five | 50 | reuse | md4c | 3.686× | 3.638× | -1.29% |
| configurable-five | 50 | reuse | pulldown-cmark | 2.580× | 2.695× | +4.45% |
| configurable-five | 50 | reuse | Bun native bun_md | 6.148× | 6.462× | +5.11% |
| comments | 11 | fresh | Ferromark v1 | 1.607× | 1.660× | +3.31% |
| comments | 11 | fresh | md4c | 5.033× | 4.448× | -11.63% |
| comments | 11 | fresh | pulldown-cmark | 2.152× | 2.228× | +3.52% |
| comments | 11 | fresh | Bun native bun_md | 4.803× | 4.977× | +3.63% |
| comments | 11 | reuse | Ferromark v1 | 1.425× | 1.473× | +3.33% |
| comments | 11 | reuse | md4c | 6.081× | 5.349× | -12.04% |
| comments | 11 | reuse | pulldown-cmark | 2.439× | 2.529× | +3.69% |
| comments | 11 | reuse | Bun native bun_md | 6.077× | 6.305× | +3.74% |
| encyclopedia | 8 | fresh | Ferromark v1 | 3.165× | 3.284× | +3.75% |
| encyclopedia | 8 | fresh | md4c | 4.110× | 4.079× | -0.75% |
| encyclopedia | 8 | fresh | pulldown-cmark | 3.239× | 3.350× | +3.43% |
| encyclopedia | 8 | fresh | Bun native bun_md | 6.075× | 6.399× | +5.33% |
| encyclopedia | 8 | reuse | Ferromark v1 | 2.993× | 3.097× | +3.49% |
| encyclopedia | 8 | reuse | md4c | 4.160× | 4.129× | -0.73% |
| encyclopedia | 8 | reuse | pulldown-cmark | 3.262× | 3.366× | +3.16% |
| encyclopedia | 8 | reuse | Bun native bun_md | 6.288× | 6.631× | +5.45% |
| plain-prose | 4 | fresh | Ferromark v1 | 1.589× | 1.636× | +2.92% |
| plain-prose | 4 | fresh | md4c | 3.976× | 4.088× | +2.83% |
| plain-prose | 4 | fresh | pulldown-cmark | 3.414× | 3.519× | +3.06% |
| plain-prose | 4 | fresh | Bun native bun_md | 17.126× | 17.652× | +3.07% |
| plain-prose | 4 | reuse | Ferromark v1 | 1.420× | 1.464× | +3.08% |
| plain-prose | 4 | reuse | md4c | 3.840× | 3.961× | +3.14% |
| plain-prose | 4 | reuse | pulldown-cmark | 3.267× | 3.374× | +3.28% |
| plain-prose | 4 | reuse | Bun native bun_md | 17.175× | 17.850× | +3.93% |
| readme | 2 | fresh | Ferromark v1 | 1.651× | 1.756× | +6.36% |
| readme | 2 | fresh | md4c | 2.541× | 2.624× | +3.25% |
| readme | 2 | fresh | pulldown-cmark | 1.996× | 2.116× | +6.01% |
| readme | 2 | fresh | Bun native bun_md | 3.922× | 4.168× | +6.26% |
| readme | 2 | reuse | Ferromark v1 | 1.549× | 1.658× | +7.04% |
| readme | 2 | reuse | md4c | 2.529× | 2.630× | +4.01% |
| readme | 2 | reuse | pulldown-cmark | 1.960× | 2.097× | +6.97% |
| readme | 2 | reuse | Bun native bun_md | 3.956× | 4.249× | +7.40% |
| reference | 4 | fresh | Ferromark v1 | 1.743× | 1.834× | +5.27% |
| reference | 4 | fresh | md4c | 2.529× | 2.637× | +4.26% |
| reference | 4 | fresh | pulldown-cmark | 1.793× | 1.896× | +5.77% |
| reference | 4 | fresh | Bun native bun_md | 3.724× | 3.913× | +5.08% |
| reference | 4 | reuse | Ferromark v1 | 1.704× | 1.770× | +3.84% |
| reference | 4 | reuse | md4c | 2.513× | 2.580× | +2.67% |
| reference | 4 | reuse | pulldown-cmark | 1.777× | 1.857× | +4.48% |
| reference | 4 | reuse | Bun native bun_md | 3.755× | 3.906× | +4.00% |
| technical-docs | 21 | fresh | Ferromark v1 | 2.010× | 2.129× | +5.94% |
| technical-docs | 21 | fresh | md4c | 3.000× | 3.083× | +2.78% |
| technical-docs | 21 | fresh | pulldown-cmark | 2.583× | 2.727× | +5.56% |
| technical-docs | 21 | fresh | Bun native bun_md | 5.638× | 5.964× | +5.79% |
| technical-docs | 21 | reuse | Ferromark v1 | 1.911× | 2.018× | +5.57% |
| technical-docs | 21 | reuse | md4c | 2.996× | 3.070× | +2.47% |
| technical-docs | 21 | reuse | pulldown-cmark | 2.561× | 2.697× | +5.33% |
| technical-docs | 21 | reuse | Bun native bun_md | 5.777× | 6.119× | +5.92% |
