# ADR-0014: Increase the bounded reference-resolution allowance

**Status:** Accepted
**Date:** 2026-09-11

## Context

The native comparison exposed a false positive in ordinary long-document
rendering: the 1 MiB CommonMark fixture resolved only 404 of its 564 charter
references before hitting the 32,768-unit document-wide work budget. This was
safe fallback behavior, but insufficient headroom for a supported workload.

## Decision

Increase `MAX_REFERENCE_RESOLUTION_WORK` to **262,144 work units**, eight times
the former value. Decouple the constant from the per-paragraph inline-mark cap:
many small paragraphs should have a larger total allowance than one paragraph.
The counter still charges bracket records and scanned label bytes; it remains
shared across paragraphs and MDX segments and resets for a new document.

Keep the existing literal fallback and resource-limit reporting. The limit is
not removed, reset per paragraph, or disabled specially for benchmarks. A fixed
increase is a narrow production change; input-proportional budgets and configurable
limits need separate API and adversarial-work analysis.

## Consequences and validation

Maximum budgeted reference-resolution work increases eightfold. This is a real
tradeoff: adversarial documents may consume more work before fallback. The
algorithm and bounded-counter mechanism remain unchanged; the larger constant
does not itself preallocate memory. It is not a guarantee that every document
of a particular byte size can resolve every reference.

A regression against the actual 1 MiB fixture fails at the old limit and passes
for both CommonMark and GFM at the new limit, resolving every charter reference.
Resource tests also verify that deep bracket nesting still exhausts the budget,
that the allowance stays shared across paragraphs and MDX segments, and that
parser reuse resets it. The partial-candidate boundary test now accounts for
the budget's remainder instead of assuming one specific modulo-three value.

The [workload comparison contract](ARCH-COMP-002-workload-comparability.md)
continues to reject incomplete work in historical outputs. Raising the current
limit does not retroactively validate old truncated-output measurements.
