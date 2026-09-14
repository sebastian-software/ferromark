# Method and experiment history

## Sources and scope

The reference is Ferromark v2 commit
`ae963d425a4c15841ced5f7359a4b9adc47ca931`. The candidate is that source plus
[`candidate.patch`](candidate.patch). The production change is confined to
`Parser::unescape_link_component`; inline destinations/titles and reference
definitions share this helper. There is no dependency update, new library
`unsafe`, option change, or snapshot update.

Both workers use Rust 1.95.0, generic AArch64 code generation, optimization level
3, fat LTO, one codegen unit, panic abort, and the system allocator. Source tree,
lockfile, worker, and binary hashes are recorded in `build.json`. Each isolated
worker retains the source lock's registry versions and checksums. `memchr`
2.8.0 is already a parser dependency. On this target, generic code generation
enables NEON, and memchr selects its AArch64 NEON implementation with short-input
handling. Its portable implementations remain available on other targets.

Measurements were made on an Apple M1 Pro running macOS 26.6.2 on AC power.
This was an active desktop: see [`preflight.txt`](preflight.txt). No compiler or
other benchmark was visible in that sample. Background services were active;
CPU affinity and thermal state were not controlled. The thermal probe failed,
which is recorded in `run.json`. The CPU name was verified separately with
`sysctl` because that probe is restricted when launched inside the Python runner.
No x86-64 measurement or cross-platform speed claim follows from this run.

## Candidate selection

The diagnostic corpus was fixed before timing. Four small implementations of
the same idea were screened with one round of three paired 15 ms windows:

1. **Repeated search:** call `memchr2` on every loop iteration. Long clean URLs
   improved, but dense backslash escapes suffered a large regression.
2. **Search after scalar handling:** process the current special byte directly,
   then search when an ordinary byte is reached. This removed the dense-escape
   penalty but left overhead between closely spaced malformed entities.
3. **Eight-byte probe:** scan a short local prefix before each bulk search. This
   did not resolve the remaining tradeoff and added overhead on short links.
4. **Single probe:** locate the first backslash or ampersand once. Return the
   original borrowed slice if neither exists; otherwise use the original scalar
   loop from that first candidate onward. This is the final measured candidate.

Each screen's source patch, verification, raw samples, build identity, and
summary are archived under `screens/`. These short screens guided selection;
the final candidate then received the complete three-round measurement. No
threshold was tuned against individual real-document timings.

## Inputs and options

The matrix keeps all 57 inputs from the frozen
[broad comparison](../2026-09-14-broad-markdown/README.md), from 37 to 113,609
UTF-8 bytes, including authored comments, project documentation, reference text,
and Wikipedia-derived views. Fifteen separately labeled synthetic diagnostics
cover empty input, short links/titles/references, long URLs, Unicode, escapes,
valid/invalid entities, MDX, and optional extensions. They are excluded from
broad-corpus aggregates. Article excerpts and full views overlap; their count
does not represent independent articles or a population sample.

Source attribution and licenses remain in
[`benchmarks/broad-comparison`](../../../benchmarks/broad-comparison/README.md).
`make_corpus.py` reproduces the exact 72-case input set without downloads.

CommonMark and GFM options match the earlier comparison: GFM footnotes are off;
XHTML output is enabled; the hard break is `<br />\n`; renderer URL autolinking
and target-blank options are off; raw HTML is disallowed for GFM. MDX uses
`ParserOptions::mdx()`. The extension diagnostic enables GFM, footnotes,
superscript/subscript, smart punctuation, math, definition lists, heading
attributes, wiki links, and CJK emphasis. Exact options are in `worker.rs`.

## Timed stages and checks

| Stage | Timed work |
| --- | --- |
| Fresh | Source-sized arena, parser, new renderer, owned HTML, and destruction |
| Reuse | Parse with retained arena, retained renderer, borrowed HTML, arena reset |
| Parse | Parse with retained arena, AST consumption, destruction and arena reset |
| Render | Render an AST parsed once before timing; retain renderer/output capacity |

Input loading, HTML/AST formatting for verification, hashes, and IPC are outside
the timer. The worker consumes the full AST or HTML through `black_box` and
returns a checksum derived from verified child counts or UTF-8 HTML byte counts.
Every warmup and timing checksum is checked with 64-bit wrapping arithmetic.

All selected stages must produce byte-identical HTML and AST `Debug` output,
both across versions and across stages. Unlike the v1/v2 comparison, there is
no canonicalization or mismatch exclusion. Each live timing worker is verified
before and after a batch of paired windows. Arena capacities are compared
between versions at matching lifecycle points: resetting an arena can retain
only its largest chunk, so warm and initial capacities may legitimately differ.

The final matrix has 72 cases × 4 stages × 3 rounds × 3 pairs: **2,592 paired
measurements**. Each window is at least 50 ms, preceded by a 50 ms warmup per
worker/job. Batches contain 32 document iterations. Jobs are shuffled with a
fixed seed and engine order alternates between pairs and rounds.

Ratios are baseline ns/document divided by candidate ns/document. Per-case
results use the median paired ratio, with all pairs and round medians retained.
Group figures are geometric means of those case ratios. Round ranges describe
observed variation; they are not confidence intervals. Render-only measurements
are controls: the renderer source did not change, so small differences there
cannot be credited to this parser scan and can reflect noise or compiled layout.

## Memory and correctness

The allocation worker is a separate instrumented executable and supplies no
performance numbers. It warms each case once, then counts allocation,
zero-allocation, deallocation, and reallocation calls plus requested layout
sizes around the complete stage. Reallocation old/new sizes are separate;
`alloc_bytes` and `requested_bytes` include zeroed requests, and `zeroed_bytes`
is their diagnostic subset. These are aggregate counters,
not an ordered trace, live-memory measurement, or resident-set size.

All **216 case/stage comparisons** (72 × fresh/reuse/parse) have identical
counters, output checksums, and arena capacity. The archive is
`allocation-allocations.json.gz`. This complements the exact AST/HTML checks in
the uninstrumented workers.

The candidate library passes 628 tests, including the unchanged existing snapshots
and conformance baselines. Two added tests compare against the original scalar
walk across more than 15,000 boundary, tail, ASCII escape, malformed entity,
dense-candidate, and deterministic mixed-Unicode inputs. Unchanged components
must retain the original borrowed pointer. Formatting, strict Clippy, and all
seven Criterion benchmark builds also pass. Harness admission/checksum guards
cover empty HTML, UTF-8 byte lengths, AST changes, arena lifecycle differences,
and corrupt/wrapped checksums.
