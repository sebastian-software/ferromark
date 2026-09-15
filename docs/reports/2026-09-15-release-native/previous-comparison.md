# Position against the preceding native comparison

Identical library pins, options, and agreeing input sets. Positive change means
v2 improved its relative throughput against that engine; negative means it lost
ground. These are ratios from separate runs, not isolated causal measurements
of one optimization. Inspect process-round variation before interpreting small changes.

| Group | N | Lifecycle | Against | Previous v2 throughput | Current | Position change |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| all-six | 14 | fresh | OX-Content original | 1.000× | 0.986× | -1.40% |
| all-six | 14 | fresh | Ferromark v1 | 1.412× | 1.388× | -1.68% |
| all-six | 14 | fresh | md4c | 3.809× | 3.746× | -1.65% |
| all-six | 14 | fresh | pulldown-cmark | 2.154× | 2.122× | -1.52% |
| all-six | 14 | fresh | Bun native bun_md | 6.009× | 5.893× | -1.94% |
| all-six | 14 | reuse | OX-Content original | 1.018× | 0.991× | -2.60% |
| all-six | 14 | reuse | Ferromark v1 | 1.401× | 1.365× | -2.55% |
| all-six | 14 | reuse | md4c | 4.874× | 4.742× | -2.71% |
| all-six | 14 | reuse | pulldown-cmark | 2.635× | 2.564× | -2.67% |
| all-six | 14 | reuse | Bun native bun_md | 8.077× | 7.840× | -2.93% |
| configurable-five | 50 | fresh | Ferromark v1 | 1.896× | 1.855× | -2.16% |
| configurable-five | 50 | fresh | md4c | 3.229× | 3.143× | -2.65% |
| configurable-five | 50 | fresh | pulldown-cmark | 2.406× | 2.375× | -1.28% |
| configurable-five | 50 | fresh | Bun native bun_md | 5.531× | 5.369× | -2.93% |
| configurable-five | 50 | reuse | Ferromark v1 | 1.854× | 1.802× | -2.83% |
| configurable-five | 50 | reuse | md4c | 3.513× | 3.394× | -3.39% |
| configurable-five | 50 | reuse | pulldown-cmark | 2.573× | 2.523× | -1.97% |
| configurable-five | 50 | reuse | Bun native bun_md | 6.202× | 5.989× | -3.44% |
| comments | 11 | fresh | Ferromark v1 | 1.348× | 1.291× | -4.26% |
| comments | 11 | fresh | md4c | 3.619× | 3.464× | -4.27% |
| comments | 11 | fresh | pulldown-cmark | 1.811× | 1.737× | -4.06% |
| comments | 11 | fresh | Bun native bun_md | 4.024× | 3.841× | -4.54% |
| comments | 11 | reuse | Ferromark v1 | 1.387× | 1.311× | -5.53% |
| comments | 11 | reuse | md4c | 5.044× | 4.745× | -5.92% |
| comments | 11 | reuse | pulldown-cmark | 2.387× | 2.245× | -5.94% |
| comments | 11 | reuse | Bun native bun_md | 5.904× | 5.550× | -6.00% |
| encyclopedia | 8 | fresh | Ferromark v1 | 3.343× | 3.220× | -3.67% |
| encyclopedia | 8 | fresh | md4c | 4.224× | 4.041× | -4.32% |
| encyclopedia | 8 | fresh | pulldown-cmark | 3.401× | 3.286× | -3.39% |
| encyclopedia | 8 | fresh | Bun native bun_md | 6.554× | 6.179× | -5.72% |
| encyclopedia | 8 | reuse | Ferromark v1 | 3.287× | 3.144× | -4.34% |
| encyclopedia | 8 | reuse | md4c | 4.428× | 4.214× | -4.82% |
| encyclopedia | 8 | reuse | pulldown-cmark | 3.543× | 3.421× | -3.46% |
| encyclopedia | 8 | reuse | Bun native bun_md | 7.029× | 6.600× | -6.10% |
| plain-prose | 4 | fresh | Ferromark v1 | 1.550× | 1.523× | -1.75% |
| plain-prose | 4 | fresh | md4c | 3.862× | 3.800× | -1.61% |
| plain-prose | 4 | fresh | pulldown-cmark | 3.314× | 3.263× | -1.53% |
| plain-prose | 4 | fresh | Bun native bun_md | 16.690× | 16.391× | -1.79% |
| plain-prose | 4 | reuse | Ferromark v1 | 1.380× | 1.350× | -2.15% |
| plain-prose | 4 | reuse | md4c | 3.729× | 3.664× | -1.72% |
| plain-prose | 4 | reuse | pulldown-cmark | 3.178× | 3.131× | -1.47% |
| plain-prose | 4 | reuse | Bun native bun_md | 16.800× | 16.443× | -2.12% |
| readme | 2 | fresh | Ferromark v1 | 1.689× | 1.671× | -1.03% |
| readme | 2 | fresh | md4c | 2.554× | 2.493× | -2.38% |
| readme | 2 | fresh | pulldown-cmark | 2.010× | 2.028× | +0.89% |
| readme | 2 | fresh | Bun native bun_md | 4.049× | 3.981× | -1.69% |
| readme | 2 | reuse | Ferromark v1 | 1.603× | 1.587× | -0.99% |
| readme | 2 | reuse | md4c | 2.573× | 2.492× | -3.12% |
| readme | 2 | reuse | pulldown-cmark | 2.000× | 2.006× | +0.30% |
| readme | 2 | reuse | Bun native bun_md | 4.137× | 4.070× | -1.60% |
| reference | 4 | fresh | Ferromark v1 | 1.644× | 1.668× | +1.46% |
| reference | 4 | fresh | md4c | 2.392× | 2.400× | +0.34% |
| reference | 4 | fresh | pulldown-cmark | 1.684× | 1.745× | +3.67% |
| reference | 4 | fresh | Bun native bun_md | 3.556× | 3.572× | +0.46% |
| reference | 4 | reuse | Ferromark v1 | 1.623× | 1.629× | +0.38% |
| reference | 4 | reuse | md4c | 2.389× | 2.372× | -0.73% |
| reference | 4 | reuse | pulldown-cmark | 1.676× | 1.719× | +2.61% |
| reference | 4 | reuse | Bun native bun_md | 3.588× | 3.592× | +0.10% |
| technical-docs | 21 | fresh | Ferromark v1 | 1.971× | 1.945× | -1.33% |
| technical-docs | 21 | fresh | md4c | 2.873× | 2.817× | -1.94% |
| technical-docs | 21 | fresh | pulldown-cmark | 2.507× | 2.505× | -0.07% |
| technical-docs | 21 | fresh | Bun native bun_md | 5.563× | 5.453× | -1.97% |
| technical-docs | 21 | reuse | Ferromark v1 | 1.910× | 1.877× | -1.72% |
| technical-docs | 21 | reuse | md4c | 2.917× | 2.849× | -2.32% |
| technical-docs | 21 | reuse | pulldown-cmark | 2.530× | 2.520× | -0.42% |
| technical-docs | 21 | reuse | Bun native bun_md | 5.790× | 5.667× | -2.12% |
