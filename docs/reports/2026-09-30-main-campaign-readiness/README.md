# Main campaign readiness — macOS arm64

All **20 main execution variants (nine Native, eleven Node.js)** built and passed
full-input local verification on Apple M1 Ultra, 64 GB RAM, macOS 27.0, from clean commit
`df4d85ef45928dae3af87f435f0623072f5a63fe`. The source clone includes TanStack Markdown
0.0.16 and the new all-document scoring policy. This proves execution on this host;
Blacksmith and other platforms still need their own preflight.

The normal CLI prepared a fresh isolated clone, rebuilt both native harnesses and
the Node addon, replayed the pinned locks, audited sources, and ran real native,
Node adapter, and addon-loader tests. Verification ran from that clean source
clone after later presentation edits in the development worktree. The dirty
worktree guard correctly prevented verification from that development checkout.
Every selected library processed all 57 inputs in both lifecycles. Deterministic
output, options, state isolation, and rotating-input checks passed. TanStack's
31 equivalent outputs and 26 differences are descriptive counts, not a conformance
rate or an admission/scoring filter. New official factors include every input.

```sh
./scripts/benchmark-comparison prepare /private/tmp/ferromark-main-performance-preflight-20260930 --scope main
# In the resulting clean OUTPUT/source clone:
./scripts/benchmark-comparison verify /private/tmp/ferromark-main-performance-preflight-20260930
```

[Prepared provenance](prepared.json), [inventory](comparisons.json),
[scoring policy](scoring-policy.json), [verification summary](verification-summary.json),
[source audit](source-audit.json), build metadata, exact locks, compressed logs,
and raw outputs/guards under `verification/` record what was executed.

A separate real TanStack timing-path smoke used one round, one 1 ms sample, and
1 ms warmup on all 57 inputs. Its full-corpus aggregate was recomputed from the
raw samples, and the archive validator rejected the shortened run for publication.
[Diagnostic metadata](diagnostic-summary.json) and compressed raw files under
`diagnostics/` are explicitly not public speed evidence.

No official timed campaign, new homepage factor, or paid Blacksmith job is part
of this readiness check. Historical reports and values remain unchanged. The
shorter sampling profile and parallel Blacksmith VM allocation remain separate
work before a cloud campaign. [SHA256SUMS](SHA256SUMS) covers every retained file.
