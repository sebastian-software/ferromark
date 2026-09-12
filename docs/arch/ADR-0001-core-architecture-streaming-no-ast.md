# ADR-0001: core architecture streaming no ast

**Status**: Accepted

**Context**: ferromark targets high-throughput CommonMark parsing with minimal allocations and predictable linear-time behavior.

**Decision**: Use a streaming, event-based architecture with no AST. Block parsing emits block events; inline parsing operates on ranges into the input; rendering consumes events directly.

**Consequences**:
- Avoids retaining a complete AST. Memory use and allocation counts still
  depend on document structure, buffer sizing and the alternative parser's
  representation; neither advantage is guaranteed for every input. The
  [native heap audit](../reports/2026-09-12-ox-memory/REPORT.md) measures both
  advantages and counterexamples against an arena-based AST parser.
- Easier to optimize hot paths (cursor scanning, range slicing).
- Some cross-block features (e.g., link ref definitions) require careful handling without an AST.
