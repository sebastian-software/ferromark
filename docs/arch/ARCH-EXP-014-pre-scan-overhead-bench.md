# ARCH-EXP-014: Pre-Scan Overhead Bench (Candidates vs Full)

**Hypothesis**: A lightweight pre-scan for link ref candidates might be cheap enough to enable streaming without large regressions.

**Change**: Added bench-only experiments that run `prescan_candidates` or `prescan_full` before `ferromark::to_html` (no functional change).

**Result**:
- `simple`: baseline ~86.6 MiB/s; candidates ~83.4 MiB/s; full ~82.3 MiB/s
- `links`: baseline ~88.2 MiB/s; candidates ~86.8 MiB/s; full ~84.0 MiB/s
- `refs`: baseline ~38.2 MiB/s (high variance); candidates ~40.4 MiB/s; full ~37.4 MiB/s
- `mixed`: baseline ~86.8 MiB/s; candidates ~84.1 MiB/s; full ~80.8 MiB/s

**Decision**: Kept as bench-only experiments.

**Notes**: Even candidate-only scans impose ~2–4% overhead on simple/mixed docs. A correct pre-scan must be substantially cheaper or highly selective.
