# Epic #71: Evidence-gated performance campaign

## Decision

Run the campaign as a sequence of isolated, evidence-gated experiments. The
baseline package (#63) is the prerequisite for every later package. At most one
experiment is active at a time. An accepted result merges before the next
experiment starts; a rejected result remains as a closed, unmerged draft pull
request with its exact measurements recorded in the issue.

This is the selected approach because it preserves comparable measurements and
makes negative results useful. A single long-lived optimization branch would
mix variables and invalidate rebaselining. A documentation-only campaign would
not test the production hypotheses.

## Boundaries

The public rendering contract remains intact throughout the campaign:

- secure-default rendering stays the product lane;
- closest-semantics parity results are separate and explicitly named;
- the semantic event and future transform/plugin path remains available;
- portable source wins are reported separately from PGO and CPU-specific
  builds;
- no benchmark claim changes until the publication baseline passes its protocol.

## Execution flow

1. #63 defines the pinned toolchain, machine metadata, fixtures, result
   artifacts, repetitions, and publication protocol.
2. #62, #65, and #67 run from that untouched baseline in dependency-safe order.
3. #64, #66, and #68 run after their stated prerequisites and updated profiles.
4. #69 may introduce only an internal optional sink that shares resolved
   semantics with the event path.
5. #70 evaluates PGO and target-specific lanes only after accepted portable
   source work is stable.
6. The epic closes only with a final scorecard listing secure-default,
   parity, profile, allocation, copied-byte, event-volume, PGO, and
   accepted/rejected results.

## Measurement and decision protocol

Each package uses an untouched current-`main` baseline worktree and an isolated
candidate worktree. Screening uses at least 80 Criterion samples, a five-second
measurement window, and a three-second warmup. Promising candidates run three
times, with baseline and candidate order alternated when practical. Results
under one percent are noise unless repeated medians and confidence intervals
agree. No primary guardrail may regress by more than one percent reproducibly.

Every issue records the baseline commit, `rustc -Vv`, CPU mode, machine and
power state, command lines, result artifacts, and final decision. The Epic #71
dashboard is updated when a package starts, produces a draft PR, or reaches its
final disposition.

## Correctness, safety, and Rust design constraints

Production changes use borrowing rather than needless cloning, keep hot paths
statically dispatched where concrete types are known, and express failures with
explicit `Result`/`Option` handling. No production `unwrap`, undocumented
`unsafe`, or HTML-only shortcut may bypass the semantic boundary. Public Rust
APIs get rustdoc that states behavior, errors, and executable examples where
useful.

Each kept parser or renderer change runs format, locked all-feature tests,
strict Clippy, the complete CommonMark report, and representative raw-HTML,
unsafe-URL, table, reference-link, nested-emphasis, and fenced-code checks.
Security-sensitive changes also receive focused differential or property tests.

## Documentation

Benchmark documentation leads with the reader-relevant claim, states the exact
lane and semantic boundary, and separates confirmed measurements from
directional evidence. It names the command and environment needed to reproduce
the result, avoids unverified comparative language, and explains a rejection as
a useful result rather than hiding it.

## Completion criteria

All nine work packages have a final accepted, rejected, or explicitly blocked
disposition; no package is silently skipped. The Epic #71 table and individual
issues contain accurate links and measurements, the final scorecard is present,
and all accepted code has passed the required validation and review/CI gates.
