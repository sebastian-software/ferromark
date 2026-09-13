# ARCH-EXP-005: Inline: Unstable Sort for Emit Points

**Hypothesis**: `sort_unstable_by_key` reduces sort overhead for emit points.

**Change**: Switch to `sort_unstable_by_key` in emit ordering.

**Result**: CommonMark: OK. Simple bench: +0.5% to +1.2% throughput (borderline).

**Decision**: Kept.

**Notes**: Tiny improvement; near noise.

## ARM recheck, 2026-09-13

Retained after comparing packed integer keys, stable sorting, and their
combination on Apple M1 Pro. The selected inputs did not show broad benefits;
focused repetitions preserved runtime counterexamples, and stable sorting
required additional temporary allocations on larger controls. See the
[measured comparison](../reports/2026-09-13-emit-sort-arm/REPORT.md), including
the frozen sources, raw windows and allocation observations. The x86 Callgrind
suggestions that motivated the recheck did not establish an ARM throughput gain.
