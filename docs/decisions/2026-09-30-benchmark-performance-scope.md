# Score performance on every frozen input

- Status: Accepted
- Date: 2026-09-30
- Scope: New manual comparison campaigns and ecosystem pairs

## Context

The homepage compares the speed of public Markdown-to-HTML APIs. Filtering each
factor by HTML agreement changes its denominator and hides performance on inputs
where libraries implement different syntax or renderer behavior. A syntax subset
such as TanStack Markdown is still a relevant performance candidate. Spec
conformance is a separate concern and is not this campaign's goal.

## Decision

Every new factor includes all 57 frozen documents. Keep equal-document geometric
means of the per-engine median of process-round medians, with fresh/reuse calls
and Native/Node.js contracts retained separately. Outputs, agreement categories,
option guards, and rotating-document controls remain in the raw evidence.
Agreement counts describe this corpus; they are not conformance rates and never
filter candidates, documents, or timing samples.

Use the public APIs with the documented options. Do not rewrite their output or
invent custom parser functionality to force agreement. Disclose supported syntax,
fixed extensions, markup differences, runtime, allocation, and cache behavior.
The comparison measures those calls on the same inputs; it does not promise equal
feature sets. Keep guards for deterministic output, configured options, state
isolation, complete samples, and consumed work. No generic conformance pass is
required to participate.

New homepage factors get `*` when any document differs from Ferromark. Their
linked reports retain agreement counts and explain API differences. Separate
performance measurements from feature/spec comparisons.

Pair schema 3 and manual suite schema 5 record `scoring_scope: all-documents` and
the committed scoring-policy hash. Publication recomputes every aggregate,
requires all inputs, and rejects downgrading the policy. Historical pair/suite
schemas remain readable with their original equivalent-output factors. Never
silently recalculate or relabel historical published numbers.

## Consequences

A smaller feature set can be faster on an input that another library parses more
fully. This is a documented property of the chosen public API workload, not a
reason to discard its measurement. Candidate selection still considers adoption,
innovation, supported runtimes, and an executable full-document HTML API.

Adding an adapter does not publish a speed claim. New factors require a complete,
reviewed campaign under its recorded sampling policy. This decision changes the
scoring denominator, not the existing sampling windows or paid-run dispatch.
