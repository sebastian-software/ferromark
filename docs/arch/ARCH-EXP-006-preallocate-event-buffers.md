# ARCH-EXP-006: Render: Preallocate Event Buffers

**Hypothesis**: Preallocating block/inline event Vecs reduces allocator churn on simple docs.

**Change**: Preallocate `events` and `inline_events` in `render_to_writer`.

**Result**: CommonMark: OK. Simple bench: +13% to +14% throughput.

**Decision**: Kept.

**Notes**: Large win; reduces `RawVec` growth hotspots.
