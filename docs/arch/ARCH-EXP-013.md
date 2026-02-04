# ARCH-EXP-013: Link-Ref Pre-Scan via Full Block Parse

**Hypothesis**: Pre-scanning link reference definitions with a no-op sink enables streaming or avoids buffering, without significant overhead.

**Change**: Added `BlockEventSink` + `parse_into` and ran a full block parse into a null sink to collect link refs before the real parse/render pass.

**Result**: CommonMark: OK. Simple bench: ~63.5 MiB/s (regressed vs ~86.6 MiB/s baseline).

**Decision**: Reverted.

**Notes**: Two full block passes are too expensive. Need a lighter pre-scan to avoid full block parsing.
