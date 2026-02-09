# ARCH-EXP-009: Block: Slice-Based Indent Scanning

**Hypothesis**: Replacing Cursor peek loops with slice iteration reduces overhead.

**Change**: Rewrite `skip_indent` / `skip_indent_max` using `remaining_slice()`.

**Result**: CommonMark: OK. Simple bench: no improvement (slight regression).

**Decision**: Reverted.

**Notes**: Cursor-based code was already competitive.
