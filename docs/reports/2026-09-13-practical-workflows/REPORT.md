# Practical Markdown workflows: time and memory

Generated from the archived raw observations by `benchmarks/workflows/publish.py`.

Measured on Apple M1 Pro with 32 GiB of installed RAM, macOS 26.6.2,
Rust 1.97.1, `-C target-cpu=generic`, and the system allocator. Installed
RAM describes the host; the tables measure requested heap, not process RAM.

## Questions and scope

The corpus was frozen before timing. Preview comments are authored examples;
guide pages and documentation are verbatim snapshots of this project's real
files. They cover concrete integration choices without claiming to represent
the distribution of all Markdown content or customer traffic.

The secure preview lane compares fresh owned output with a retained renderer.
The metadata lane includes front matter and heading collection. Only the
trusted documentation HTML lane compares engines, with identical syntax flags
and reviewed complete output. It also measures immediate release versus
retaining every output until the end of the workload.

## Corpus and integration decisions

| Workload | Documents | Total input | Smallest–largest file |
| --- | ---: | ---: | ---: |
| previews | 12 | 2,081 bytes | 57–277 bytes |
| guides | 3 | 6,763 bytes | 1,426–3,721 bytes |
| documentation | 12 | 53,656 bytes | 1,532–9,323 bytes |

Reusing `Renderer` reduced time and allocation traffic for these previews.
Its peak heap was higher: retained scratch remains part of the worker's memory budget.

For the documentation collection, compare each engine's two output lifetimes
before using a memory figure to size your pipeline. Keeping completed HTML
in memory is application work included in the retained-output rows.

The metadata row measures a complete HTML-and-metadata operation; it does
not isolate the incremental cost of collecting headings or represent an
end-to-end site build.

## Results

| API / output lifetime | Time / complete workload | Peak live heap¹ | Allocated per workload² |
| --- | ---: | ---: | ---: |
| Fresh `to_html` calls | 11.5 µs | 6.9 KiB | 47.2 KiB |
| Retained `Renderer` | 7.2 µs | 8.4 KiB | 5.6 KiB |
| `parse`: HTML + front matter + headings | 20.1 µs | 20.6 KiB | 57.2 KiB |
| Ferromark · release each page | 139.8 µs | 26.0 KiB | 216.9 KiB |
| pulldown-cmark · release each page | 173.8 µs | 53.4 KiB | 585.6 KiB |
| Comrak · release each page | 533.3 µs | 144.7 KiB | 1428.9 KiB |
| Ferromark · keep all pages | 140.6 µs | 73.8 KiB | 217.6 KiB |
| pulldown-cmark · keep all pages | 174.2 µs | 108.8 KiB | 586.3 KiB |
| Comrak · keep all pages | 532.3 µs | 157.1 KiB | 1429.6 KiB |

Peak heap includes the session and all retained scratch above a baseline that
excludes loaded input and benchmark bookkeeping. Each memory observation
creates and warms a session, resets the peak while retaining its bytes in the
count, measures one complete workload, then drops the session and verifies a
return to baseline. Thirty observations per variant agree exactly. These
are requested allocation sizes, not allocator physical consumption or RSS.

The cumulative allocated column counts allocation/reallocation requests
during the workload; it is allocation traffic, not a memory budget.

## Variation and cold-session memory

The average-per-document column divides the workload time by its file count.
It is not a measured per-request percentile or a tail-latency prediction.

| Variant | Round medians (µs/workload) | Round spread | Average / document | Cold peak heap | Retained session heap | Allocation calls / workload |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Fresh `to_html` calls | 11.5, 11.5, 11.6 | 1.2% | 1.0 µs | 6.9 KiB | 0.0 KiB | 142 |
| Retained `Renderer` | 7.2, 7.2, 7.2 | 0.2% | 0.6 µs | 8.4 KiB | 7.7 KiB | 23 |
| `parse`: HTML + front matter + headings | 20.1, 20.0, 20.1 | 0.5% | 6.7 µs | 20.6 KiB | 0.0 KiB | 125 |
| Ferromark · release each page | 139.7, 139.8, 139.9 | 0.2% | 11.7 µs | 26.0 KiB | 0.0 KiB | 259 |
| pulldown-cmark · release each page | 173.8, 173.8, 174.2 | 0.2% | 14.5 µs | 53.4 KiB | 0.0 KiB | 232 |
| Comrak · release each page | 531.4, 533.3, 536.9 | 1.0% | 44.4 µs | 144.7 KiB | 0.0 KiB | 4520 |
| Ferromark · keep all pages | 139.9, 140.6, 140.7 | 0.6% | 11.7 µs | 73.8 KiB | 0.0 KiB | 262 |
| pulldown-cmark · keep all pages | 174.0, 174.2, 174.8 | 0.5% | 14.5 µs | 108.8 KiB | 0.0 KiB | 235 |
| Comrak · keep all pages | 532.1, 532.3, 533.8 | 0.3% | 44.4 µs | 157.1 KiB | 0.0 KiB | 4523 |

Cold peak covers session creation and its first complete workload after
process-wide initialization. Warm peak still includes retained session
allocations; it does not pretend reusable parser scratch is free.

## Reproduce and inspect

See [the harness](../../../benchmarks/workflows/README.md) and
[the measurement contract](../../arch/ARCH-COMP-003-practical-workflows.md).

- `corpus.json`: all exact input text, source paths/revisions, licenses, byte counts, and hashes.
- `build.json`, `Cargo.lock`, and build logs: compiler, CPU flags, dependencies, source/binary hashes, and host.
- `outputs.json.gz` and `admission.json`: original HTML, guide metadata, effective options, and output review.
- `windows.json.gz`: all 2,160 timed windows with elapsed nanoseconds, completed-workload counts, and output sizes.
- `warmups.json`: all 27 warmups; `memory.json.gz`: all 270 memory observations.
- `observations.json`: power, thermal, load, and process-CPU observations; `run.json`: protocol and completion times.
- `checksums.json`: hashes of every evidence input; the publisher verifies them before generating tables.

Measured source commit: `021dc1e4f66ba4f5c98352ccea99cb914be19b44`.

Timing uses an uninstrumented binary; heap accounting uses a separate binary
with the same source, flags, and dependency lock. The allocator counts
successful allocations, zeroed allocations, reallocations, and frees.
A realloc replaces its old logical size; hidden simultaneous backing storage,
allocator metadata/rounding/caches, stack, and loaded corpus storage are not
counted. Self-tests exercise growth, shrinkage, retained bytes, and balance.

Each of three process rounds warms every variant for three seconds, then
rotates its order through 80 windows of at least 63 ms. A time check occurs
after four complete workloads. Output construction and destruction are timed;
input loading, configuration, verification, IPC, and statistics are outside
the timer. This is warmed single-threaded native API work, not CLI startup,
Node.js latency, full-site generation, or a concurrent service benchmark.

## Limits on interpretation

Use the lifetime comparison to budget your own pipeline: sequentially
releasing pages and retaining every rendered page have different memory
requirements. Do not extrapolate a per-page peak by multiplication, treat
allocation volume as peak memory, or compare rows that deliver different
outputs or trust policies as a speed ranking.

The documents come from one project; short comments are hand-authored.
This was an interactive workstation: desktop, indexing, and backup activity
are visible in the host observations. Round variation is reported above;
the process was not CPU-pinned and the host was not an isolated benchmark machine.
All measurements use one Apple Silicon host on AC power, system allocation,
and no PGO. They do not establish x86-64 performance or production p95/p99.
Historical shared-mimalloc comparisons remain separate evidence.
