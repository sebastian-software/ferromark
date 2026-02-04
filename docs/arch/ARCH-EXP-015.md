# ARCH-EXP-015: Hybrid Paragraph Buffer (Bench-Only)

**Hypothesis**: Buffer only paragraphs with ref-candidates; stream others to reduce overhead vs full pre-scan.

**Change**: Bench-only prototype that splits input into paragraphs and calls `md_fast::to_html` per paragraph (placeholder for future streaming renderer).

**Result**:
- `simple`: ~45.0 MiB/s (far slower than baseline)
- `links`: ~53.2 MiB/s
- `refs`: ~50.8 MiB/s
- `mixed`: ~55.6 MiB/s

**Decision**: Keep as benchmark-only evidence; not representative of a true streaming hybrid.

**Notes**: This prototype is pessimistic because it re-parses per paragraph. A real streaming renderer would avoid per-paragraph parser setup and should be much cheaper.
