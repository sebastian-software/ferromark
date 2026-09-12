# Measured results

Generated from the archived measurement windows by `make-results.py`. Times are microseconds; lower is better. Ratios above 1 mean Ox is faster.

## Same upstream text, different versions and settings

| Input | Ferro 0.7 defaults | Ferro 0.9 defaults | Ox defaults | Ferro 0.9 tables | Ox tables | Matched ratio |
|---|---:|---:|---:|---:|---:|---:|
| upstream-large | 295.85 | 275.12 | 192.09 | 269.47 | 222.67 | 1.21x |
| upstream-huge | 6663.11 | 6264.00 | 4291.20 | 6134.16 | 4959.93 | 1.24x |

## Workload localization

Both use the matched CommonMark settings with heading IDs, except the table workload which enables tables in both. All these pairs passed Ox-normalized output equality.

| Input | Ferromark | Ox | Ferro / Ox |
|---|---:|---:|---:|
| short | 0.484 | 0.486 | 1.00x |
| plain | 7.442 | 5.049 | 1.47x |
| inline | 40.371 | 33.610 | 1.20x |
| lists | 46.387 | 51.928 | 0.89x |
| code | 18.622 | 12.204 | 1.53x |
| tables | 43.802 | 49.328 | 0.89x |
| headings | 42.529 | 29.618 | 1.44x |

## One-variable controls

| Input | Ox presized, fresh renderer | Ox growing arena | Ox reused renderer | Ferro heading IDs on | Ferro heading IDs off |
|---|---:|---:|---:|---:|---:|
| upstream-large | 178.447 | 178.200 | 177.788 | 258.092 | 233.351 |
| upstream-huge | 3940.156 | 3934.784 | 3932.755 | 5814.938 | 5296.747 |
| short | 0.486 | 0.528 | 0.247 | 0.484 | 0.484 |
| headings | 29.618 | 29.705 | 29.201 | 42.529 | 22.016 |

## Run stability

Median per process run, in microseconds.

| Mode, large input | Run 1 | Run 2 | Run 3 |
|---|---:|---:|---:|
| ferro-tables | 268.17 | 269.47 | 271.26 |
| ox-tables | 221.26 | 222.67 | 223.25 |
| ferro-cm | 257.33 | 258.09 | 260.88 |
| ox-cm | 177.60 | 178.45 | 178.63 |

## Isolated scanner comparison

Nanoseconds per search. Actual upstream scanner implementations, with the same 11-byte set for both; this is not Ferromark’s different Markdown marker set. All 256 byte values at every offset in lengths 1–64 were checked for equal results before each run. Three runs, eleven 20 ms windows per variant; fixed per-case engine order. These are diagnostic kernel timings, not end-to-end speedups.

| Bytes | Match | Ferromark ByteSet | Ox scanner | Ferro / Ox |
|---|---|---:|---:|---:|
| 8 | absent | 4.73 | 5.17 | 0.91x |
| 8 | last | 4.29 | 5.17 | 0.83x |
| 8 | first | 1.33 | 2.29 | 0.58x |
| 16 | absent | 3.17 | 2.61 | 1.22x |
| 16 | last | 7.09 | 1.64 | 4.31x |
| 16 | first | 3.17 | 1.65 | 1.92x |
| 32 | absent | 4.85 | 2.93 | 1.66x |
| 32 | last | 8.97 | 1.98 | 4.53x |
| 32 | first | 3.18 | 1.65 | 1.93x |
| 128 | absent | 15.48 | 6.73 | 2.30x |
| 128 | last | 20.40 | 4.90 | 4.17x |
| 128 | first | 3.19 | 1.65 | 1.94x |
| 1024 | absent | 114.01 | 36.76 | 3.10x |
| 1024 | last | 119.19 | 36.24 | 3.29x |
| 1024 | first | 3.19 | 1.65 | 1.94x |
| 65536 | absent | 7202.73 | 2315.68 | 3.11x |
| 65536 | last | 7216.62 | 2298.55 | 3.14x |
| 65536 | first | 3.18 | 1.64 | 1.94x |

## Allocation counts

Mean over ten calls after one warmup. Global allocator calls include allocations and reallocations. Requested bytes count every allocation request (including full replacement size on realloc), not peak live memory or RSS. Instrumented runs are excluded from timing. Renderer construction is inside each fresh call and outside each reused call.

| Case | Mode | Allocation calls | Requested bytes |
|---|---|---:|---:|
| upstream-large | ferro-cm | 146 | 603963 |
| upstream-large | ferro-tables | 146 | 603963 |
| upstream-large | ox-cm | 20 | 1331542 |
| upstream-large | ox-tables | 20 | 1331542 |
| upstream-large | ox-cm-grow | 22 | 741686 |
| upstream-large | ox-cm-reuse | 9 | 1314368 |
| short | ferro-cm | 10 | 3312 |
| short | ferro-tables | 10 | 3312 |
| short | ox-cm | 13 | 20836 |
| short | ox-tables | 13 | 20836 |
| short | ox-cm-grow | 14 | 1876 |
| short | ox-cm-reuse | 3 | 20566 |
| tables | ferro-cm | 12 | 34012 |
| tables | ferro-tables | 8 | 142013 |
| tables | ox-cm | 12 | 76694 |
| tables | ox-tables | 14 | 229558 |
| tables | ox-cm-grow | 13 | 39814 |
| tables | ox-cm-reuse | 2 | 76424 |

## Reuse on both sides

Supplementary three-run comparison, same protocol. Both still return a fresh owned HTML string; Ferromark retains its parser buffers and Ox retains renderer scratch, with a fresh Ox arena per document. Heading IDs are enabled on both.

| Input | Ferromark fresh | Ox fresh | Ferromark reused | Ox reused |
|---|---:|---:|---:|---:|
| short | 0.472 | 0.496 | 0.194 | 0.246 |
| upstream-large | 257.276 | 178.792 | 251.032 | 178.649 |
