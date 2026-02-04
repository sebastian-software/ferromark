# ARCH-EXP-005: Inline: Unstable Sort for Emit Points

**Hypothesis**: `sort_unstable_by_key` reduces sort overhead for emit points.

**Change**: Switch to `sort_unstable_by_key` in emit ordering.

**Result**: CommonMark: OK. Simple bench: +0.5% to +1.2% throughput (borderline).

**Decision**: Kept.

**Notes**: Tiny improvement; near noise.
