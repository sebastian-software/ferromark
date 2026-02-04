# ARCH-EXP-011: Inline: Autolink Filter via Index

**Hypothesis**: Use linear index to filter autolinks in code spans faster than `any()`.

**Change**: Index-based scan of code spans during autolink retain.

**Result**: CommonMark: OK. Simple bench: no improvement.

**Decision**: Reverted.

**Notes**: No measurable win.
