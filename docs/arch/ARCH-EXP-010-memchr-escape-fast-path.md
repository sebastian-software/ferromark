# ARCH-EXP-010: Escape: memchr Fast-Path

**Hypothesis**: Fast-path escape functions when no escapable chars exist.

**Change**: memchr pre-scan in `escape_*` to return early.

**Result**: CommonMark: OK. Simple bench: no improvement.

**Decision**: Reverted.

**Notes**: Extra scan cost offset benefit.
