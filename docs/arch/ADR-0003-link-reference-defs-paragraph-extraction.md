# ADR-0003: link reference defs paragraph extraction

**Status**: Accepted

**Context**: CommonMark link reference definitions are parsed from paragraphs and must be available for resolving reference links.

**Decision**: Extract link reference definitions when closing paragraphs (block parser), normalize labels, and store in LinkRefStore.

**Consequences**:
- Keeps parsing single-pass without AST.
- Reference definitions are resolved before inline parsing of subsequent blocks.
- Streaming render without pre-scan requires either buffering lists or pre-scan of refs.
