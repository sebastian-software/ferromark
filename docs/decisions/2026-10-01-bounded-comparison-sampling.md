# Bound comparison campaigns with named sampling profiles

- Status: Accepted
- Date: 2026-10-01
- Scope: New manual comparisons and Blacksmith campaigns

## Context

The 20-row campaign repeats all 57 documents in both lifecycles, plus rotating
controls. Six 40 ms windows, 60 ms warmup and minute-long lane pauses make each
platform expensive to iterate. The purpose is a useful public API comparison,
not distinguishing tiny differences in the final multiplier.

## Decision

Default new manual campaigns to `balanced`: three independent process rounds,
three alternating 10 ms samples per round, 30 ms per-engine/input warmup and
5-second lane pauses. Keep `standard` with the existing 3/6/40/60 sampling and
60-second pauses. Direct lower-level runners retain their standard defaults.
The committed scoring policy records both exact contracts. Preparation freezes
the selection; every pair, rotating control and native lane must use it.

Keep every document, all output/option/state guards, checksums, fresh/reuse
lifecycles, randomized/alternating order, median-of-round-medians aggregation,
equal-document geometric means and raw observations. Output agreement remains
descriptive and never filters scoring. Arbitrary shortened diagnostics remain
unpublishable. Old campaigns retain their original contract and figures.

Allow two concurrent Blacksmith jobs on separate VMs. Timing stays sequential
inside each VM and starts after all builds and output checks. Keep all trials;
do not choose the fastest allocation or merge incompatible profiles/platforms.

## Consequences

Balanced has less timing evidence and shorter JIT/allocator warmup. Inspect raw
round spread and rotating controls before publishing. A close or noisy ranking
should be confirmed with standard sampling or more independent allocations;
neither profile alone guarantees statistical significance or physical isolation.

The main balanced matrix requests approximately 15 minutes of timing, warmup and
pauses per VM. Slow calls, process starts, verification, downloads and builds add
time. This is a calculated budget, not a measured runtime promise. New reports
and generated descriptions must state the actual profile and its parameters.
