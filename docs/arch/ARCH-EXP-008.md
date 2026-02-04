# ARCH-EXP-008: Block Streaming: List-Buffered Render

**Hypothesis**: Streaming block events (avoid full Vec) reduces memory and speeds up render.

**Change**: BlockEventSink + streaming render; buffer list events to fix tight/loose.

**Result**: CommonMark: OK. Simple bench: -27% to -28% throughput.

**Decision**: Reverted.

**Notes**: Required two block passes for link refs; too costly.
