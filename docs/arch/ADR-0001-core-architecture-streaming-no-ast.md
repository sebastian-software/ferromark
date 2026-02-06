# ADR-0001: core architecture streaming no ast

**Status**: Accepted

**Context**: ferromark targets high-throughput CommonMark parsing with minimal allocations and predictable linear-time behavior.

**Decision**: Use a streaming, event-based architecture with no AST. Block parsing emits block events; inline parsing operates on ranges into the input; rendering consumes events directly.

**Consequences**:
- Lower memory use and fewer allocations than AST-based designs.
- Easier to optimize hot paths (cursor scanning, range slicing).
- Some cross-block features (e.g., link ref definitions) require careful handling without an AST.
