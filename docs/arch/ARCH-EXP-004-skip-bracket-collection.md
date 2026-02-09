# ARCH-EXP-004: Inline: Skip Bracket Collection

**Hypothesis**: Avoid collecting brackets if no bracket marks exist to skip link resolution work.

**Change**: Check for `[`/`]` marks before `collect_brackets`.

**Result**: CommonMark: OK. Simple bench: +5.6% to +6.6% throughput.

**Decision**: Kept.

**Notes**: Good win for non-link-heavy docs.
