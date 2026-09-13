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

All three lanes compare Ferromark, pulldown-cmark, and Comrak. Preview
and metadata adapters include escaping, an explicit URL allowlist, heading
IDs, callout rendering, and (for guides) borrowed raw front matter and
owned heading metadata. Each engine parses Markdown once. Adapter work
is inside both time and heap scopes; HTML and metadata are reviewed before
timing. The documentation lane also compares both output lifetimes.

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

The metadata rows measure complete HTML-and-metadata operations; they do
not isolate the incremental cost of collecting headings or represent an
end-to-end site build.

Preview and metadata comparisons include the application adapters needed
by pulldown-cmark and Comrak. Their escaping, URL checks, heading IDs,
and metadata collection are timed and counted in heap usage. These are
complete integration costs, not rankings of body-only parser calls.

## Results

| API / output lifetime | Time / complete workload | Peak live heap¹ | Allocated per workload² |
| --- | ---: | ---: | ---: |
| Ferromark · Fresh `to_html` calls | 11.7 µs | 6.9 KiB | 47.2 KiB |
| Ferromark · Retained `Renderer` | 7.3 µs | 8.4 KiB | 5.6 KiB |
| pulldown-cmark + preview adapter | 15.1 µs | 17.2 KiB | 202.5 KiB |
| Comrak + preview adapter | 38.8 µs | 7.9 KiB | 115.4 KiB |
| Ferromark · `parse`: HTML + front matter + headings | 20.3 µs | 20.6 KiB | 57.2 KiB |
| pulldown-cmark + metadata adapter | 31.1 µs | 44.2 KiB | 118.0 KiB |
| Comrak + metadata adapter | 80.4 µs | 71.6 KiB | 223.2 KiB |
| Ferromark · release each page | 143.4 µs | 26.0 KiB | 216.9 KiB |
| pulldown-cmark · release each page | 178.6 µs | 53.4 KiB | 585.6 KiB |
| Comrak · release each page | 538.9 µs | 144.7 KiB | 1428.9 KiB |
| Ferromark · keep all pages | 143.0 µs | 73.8 KiB | 217.6 KiB |
| pulldown-cmark · keep all pages | 178.3 µs | 108.8 KiB | 586.3 KiB |
| Comrak · keep all pages | 538.2 µs | 157.1 KiB | 1429.6 KiB |

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
| Ferromark · Fresh `to_html` calls | 11.7, 11.5, 11.7 | 2.3% | 1.0 µs | 6.9 KiB | 0.0 KiB | 142 |
| Ferromark · Retained `Renderer` | 7.3, 7.2, 7.3 | 1.0% | 0.6 µs | 8.4 KiB | 7.7 KiB | 23 |
| pulldown-cmark + preview adapter | 15.1, 15.0, 15.1 | 0.7% | 1.3 µs | 17.2 KiB | 0.0 KiB | 139 |
| Comrak + preview adapter | 39.0, 38.7, 38.8 | 0.9% | 3.2 µs | 7.9 KiB | 0.0 KiB | 525 |
| Ferromark · `parse`: HTML + front matter + headings | 20.5, 20.2, 20.3 | 1.4% | 6.8 µs | 20.6 KiB | 0.0 KiB | 125 |
| pulldown-cmark + metadata adapter | 31.2, 30.9, 31.1 | 1.0% | 10.4 µs | 44.2 KiB | 0.0 KiB | 204 |
| Comrak + metadata adapter | 80.6, 79.2, 80.4 | 1.7% | 26.8 µs | 71.6 KiB | 0.0 KiB | 916 |
| Ferromark · release each page | 144.0, 141.3, 143.4 | 1.9% | 11.9 µs | 26.0 KiB | 0.0 KiB | 259 |
| pulldown-cmark · release each page | 179.3, 178.0, 178.6 | 0.7% | 14.9 µs | 53.4 KiB | 0.0 KiB | 232 |
| Comrak · release each page | 543.4, 535.5, 538.9 | 1.5% | 44.9 µs | 144.7 KiB | 0.0 KiB | 4520 |
| Ferromark · keep all pages | 143.7, 141.6, 143.0 | 1.5% | 11.9 µs | 73.8 KiB | 0.0 KiB | 262 |
| pulldown-cmark · keep all pages | 179.6, 177.4, 178.3 | 1.2% | 14.9 µs | 108.8 KiB | 0.0 KiB | 235 |
| Comrak · keep all pages | 543.1, 536.1, 538.2 | 1.3% | 44.8 µs | 157.1 KiB | 0.0 KiB | 4523 |

Cold peak covers session creation and its first complete workload after
process-wide initialization. Warm peak still includes retained session
allocations; it does not pretend reusable parser scratch is free.

## Reproduce and inspect

See [the harness](../../../benchmarks/workflows/README.md) and
[the measurement contract](../../arch/ARCH-COMP-003-practical-workflows.md).

- `corpus.json`: all exact input text, source paths/revisions, licenses, byte counts, and hashes.
- `build.json`, `Cargo.lock`, and build logs: compiler, CPU flags, dependencies, source/binary hashes, and host.
- `outputs.json.gz` and `admission.json`: original HTML, guide metadata, effective options, and output review.
- `windows.json.gz`: all 3,120 timed windows with elapsed nanoseconds, completed-workload counts, and output sizes.
- `warmups.json`: all 39 warmups; `memory.json.gz`: all 390 memory observations.
- `observations.json`: power, thermal, load, and process-CPU observations; `run.json`: protocol and completion times.
- `checksums.json`: hashes of every evidence input; the publisher verifies them before generating tables.

Measured source commit: `9ad64811e8152ed88d20967f9222f48e64642966`.

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
