# ARCH-EXP-002: Inline Emit: Reduce Allocations

**Hypothesis**: Reusing buffers and reserving event capacity reduces Vec growth and allocator churn.

**Change**: Reuse suppress ranges buffer and reserve `events` capacity; linear suppress-range scan.

**Result**: CommonMark: OK. Simple bench: +2.7% to +7.1% throughput.

**Decision**: Kept.

**Notes**: Improves `InlineParser::emit_events` allocations.
