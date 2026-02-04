# ADR-0002: inline parsing three phase

**Status**: Accepted

**Context**: Inline parsing is a major hotspot; we need deterministic O(n) behavior with CommonMark correctness.

**Decision**: Use a three-phase inline parser: (1) mark collection, (2) mark resolution by precedence (code spans, links, emphasis), (3) event emission based on resolved marks.

**Consequences**:
- Enables strict CommonMark precedence handling.
- Keeps inline scanning linear and localized to paragraphs/heads.
- Event emission can be optimized independently (e.g., allocation reuse).
