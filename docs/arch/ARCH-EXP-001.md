# ARCH-EXP-001: Render: Entity Decode Fast-Path

**Hypothesis**: Skipping HTML entity decode when no '&' in inline text reduces hotpath cost in simple docs.

**Change**: In `write_text_with_entities`, early-return to `escape_text_into` if no `&` present.

**Result**: CommonMark: OK. Simple bench: +16% to +21% throughput.

**Decision**: Kept.

**Notes**: Targets `html_escape::decode_html_entities` hotspot.
