# ARCH-EXP-003: Inline Emit: Hot-Path Checks

**Hypothesis**: Avoid range checks for marks that cannot emit events to reduce branch cost.

**Change**: Reordered checks in emit loop; avoid link/autolink/html range checks unless needed.

**Result**: CommonMark: OK. Simple bench: +1.6% to +2.6% throughput.

**Decision**: Kept.

**Notes**: Small but consistent win.
