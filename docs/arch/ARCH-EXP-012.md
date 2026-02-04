# ARCH-EXP-012: Inline: Skip Code Span Resolution Without Backticks

**Hypothesis**: Avoid code-span resolution if no backtick marks exist.

**Change**: Check for backtick marks before `resolve_code_spans`.

**Result**: CommonMark: OK. Simple bench: no change.

**Decision**: Reverted.

**Notes**: Extra checks outweighed savings.
