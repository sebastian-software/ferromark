# ARCH-EXP-007: Inline Emit: Simple Streaming Fast-Path

**Hypothesis**: If no complex inline features exist, emit text/breaks directly to avoid emit_points build.

**Change**: Added fast-path in `emit_events` for only escapes and breaks.

**Result**: CommonMark: OK. Simple bench: no meaningful improvement.

**Decision**: Reverted.

**Notes**: Overhead of checks and mark scanning outweighed benefits.
