# ARCH-EXP-028: Ordered HTML/code range membership

**Date:** 2026-09-13

**Status:** Proposed; measured tradeoffs disclosed for review.

## Context

The pinned Vue Suspense guide contains 66 inline code spans and 36 less-than
signs outside fenced blocks, all inside inline code. Inspection of its inline
pipeline exposed quadratic HTML/code membership searches on larger paragraphs
with the same syntax.

## Proposal

Keep the precedence rules and resolver order. Advance through sorted,
non-overlapping HTML and code ranges instead of restarting membership searches
for every code opener, HTML candidate and autolink. This bounds these three
membership scans linearly. It does not make all inline parsing linear for
arbitrary inputs. Keep the autolink filter in an outlined helper: the combined
inline version exposed a repeatable heading-control cost that motivated a
separately measured implementation.

The public inline/block/MDX events, syntax options and render policies remain
unchanged. Release builds contain no work counters. A whole-render test counts
actual range probes and fails on the old quadratic implementation; boundary
cases check HTML attributes, code padding, entities and autolinks.

## Alternatives and limits

A validated text/code shortcut reduced Suspense time, but repeated
measurements exposed a cost on HTML prose. Moving the
shortcut, removing its scratch writes and skipping empty filters did not
eliminate that tradeoff. Those changes are not retained. Broader code-prose
shortcuts and autolink/event-emission changes also exposed control regressions.
The report preserves measured sources and unfavorable observations.

The proposal preserves the independently useful complexity fix. The final
TypeScript 6.0 comparison still has a small repeated cost. The revised tradeoff
policy accepts limited costs when the measured distribution supports them;
the report checks every individual corpus document and retains all outliers.
Performance evidence comes from Apple M1
Pro and native owned HTML production and destruction. Hosted x86 CI timing does
not establish ARM throughput. See the
[experiment report](../reports/2026-09-13-suspense-inline-code/REPORT.md) for the
final measurements and comparison with Ox's growing arena.
