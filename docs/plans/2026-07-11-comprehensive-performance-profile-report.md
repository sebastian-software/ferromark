# Comprehensive performance profile report

Status: Issue #63 portable baseline accepted; first evidence pass retained
Date: 2026-07-11
Design: [`2026-07-11-comprehensive-performance-profiling-design.md`](./2026-07-11-comprehensive-performance-profiling-design.md)

## Scope and evidence quality

This first pass validates the profiling infrastructure and updates the
opportunity ranking. It is not a replacement for published Criterion numbers.

Environment:

- Apple Silicon Mac Studio (`ARM64_T6000`);
- macOS 26.5.2;
- pinned `rustc 1.93.0`, LLVM 21.1.8;
- portable runs use `-C target-cpu=generic`;
- reusable output buffers;
- non-PGO builds;
- feature-off throughput/allocation runs;
- separate feature-on pipeline-counter run;
- three 9-11 second macOS `sample` profiles with 6,632-8,087 samples.

The throughput matrix used one-second diagnostic runs to validate coverage and
find strong signals. Treat every throughput delta as directional until the same
lane passes three alternating-order Criterion runs with the documented sample
and measurement settings. The working tree was dirty with the profiling
implementation, so these results are development evidence only.

## Issue #63 publication baseline

The publication baseline was collected from clean commit `9102db4` on an Apple
Silicon Mac Studio (`ARM64_T6000`) with macOS 26.5.2, pinned `rustc 1.93.0`
(LLVM 21.1.8), `-C target-cpu=generic`, and non-PGO release builds. The runner
used three repetitions, 80 Criterion samples, a five-second measurement window,
and a three-second warmup. It ran Ferromark first in repetitions one and three
and pulldown-cmark first in repetition two. The checked-in machine-readable
summary is [`2026-07-11-issue-63-publication-baseline.json`](../reports/2026-07-11-issue-63-publication-baseline.json).

Trusted CommonMark parity remains a deliberately separate comparison lane. It
uses trusted raw HTML in Ferromark because pulldown-cmark does not expose the
same untrusted-rendering boundary. Secure-default measurements are product-lane
numbers, not direct pulldown-cmark comparisons.

| Trusted CommonMark corpus | Ferromark MiB/s (three medians) | pulldown-cmark MiB/s (three medians) | Ferromark lead |
| --- | ---: | ---: | ---: |
| 5 KB | 272.02 / 271.67 / 271.59 | 268.37 / 268.01 / 267.31 | 1.36% / 1.37% / 1.60% |
| 20 KB | 276.20 / 277.16 / 276.80 | 265.70 / 264.05 / 264.97 | 3.95% / 4.97% / 4.46% |
| 50 KB | 290.41 / 289.96 / 289.36 | 277.35 / 276.84 / 276.83 | 4.71% / 4.74% / 4.53% |
| 250 KB | 292.86 / 293.41 / 293.49 | 275.13 / 273.85 / 276.59 | 6.44% / 7.14% / 6.11% |

The ordering is stable in all twelve parity comparisons. The 20 KB lead spans
1.01 percentage points, effectively the one-point screening boundary, so it is
evidence of a stable ordering rather than a precise marketing claim. The
publication protocol therefore unblocks source experiments but does not change
the README benchmark headline.

The secure-default Extended lane recorded 219.40 / 218.84 / 218.63 MiB/s at
5 KB, 234.86 / 235.19 / 234.72 MiB/s at 20 KB, and 220.85 / 225.41 / 224.98
MiB/s at 50 KB. At 50 KB, the Essentials, Extended, and Full profile medians
were 283.71–284.03, 220.85–225.41, and 211.51–212.03 MiB/s respectively. These
are controls for later experiments, not cross-parser claims.

The complete CommonMark report run as part of this gate currently reports 577
passing examples and 75 failures out of 652 under the default safety policy.
That result corrects the former README claim; it does not turn this performance
baseline into a claim of full CommonMark or raw-HTML parity.

## Primary parity smoke

Trusted CommonMark parity, portable code generation:

| Corpus | Ferromark | pulldown-cmark | Directional lead |
| --- | ---: | ---: | ---: |
| 5 KB | 256.16 MiB/s | 235.28 MiB/s | Ferromark +8.9% |
| 20 KB | 262.67 MiB/s | 234.13 MiB/s | Ferromark +12.2% |
| 50 KB | 274.67 MiB/s | 243.80 MiB/s | Ferromark +12.7% |
| 250 KB | 278.42 MiB/s | 242.49 MiB/s | Ferromark +14.8% |

This is materially wider than the previous near-parity observation, but the
toolchain, explicit portable target, and exact semantics differ from the README
baseline. It is a reason to run the publication protocol, not a new claim.

## Allocation shape

Per rendered document in trusted CommonMark parity:

| Corpus | Parser | Allocations | Reallocations | Requested bytes |
| --- | --- | ---: | ---: | ---: |
| 5 KB | Ferromark | 71 | 4 | 29,895 |
| 5 KB | pulldown-cmark | 13 | 5 | 71,247 |
| 20 KB | Ferromark | 79 | 5 | 75,959 |
| 20 KB | pulldown-cmark | 27 | 7 | 228,667 |
| 50 KB | Ferromark | 96 | 7 | 173,741 |
| 50 KB | pulldown-cmark | 61 | 8 | 565,407 |
| 250 KB | Ferromark | 213 | 10 | 831,809 |
| 250 KB | pulldown-cmark | 273 | 18 | 2,934,171 |

The unexpected result is that Ferromark performs more small allocations at
5/20/50 KB while requesting far fewer total bytes. At 250 KB it is lower on both
counts. Therefore “reduce allocation count” alone is the wrong target. The next
allocation work should identify fixed small buffers and preserve Ferromark's
better byte-volume scaling.

On the 50 KB product lane, `Essentials secure` used about 82 allocations and
172,887 requested bytes per document. `Extended secure` used about 347 and
207,421. The input contains references, HTML, tables, headings, and other syntax,
so this is real feature-path work rather than unused-option overhead.

## Pipeline work shape

One instrumented `Extended secure` 50 KB run reported these approximate values
per document:

| Counter | Per document |
| --- | ---: |
| Block events | 4,614 |
| Inline parses | 1,018 |
| Inline fast paths | 853 |
| Inline events | 2,081 |
| Inline marks | 719 |
| Emit points | 587 |
| Inline input bytes | 41,956 |
| Paragraph bytes copied | 36,080 |

Maximum retained capacities in that run were 6,542 block events and 64 each for
inline events, marks, and emit points. The counters show that the fast paths are
frequent and useful. They also show enough paragraph copying and event volume to
justify structural experiments, but not enough evidence to remove the
transformable event path.

## CPU profiles

### Mixed 50 KB product lane

Largest exclusive sample buckets included:

| Area | Samples |
| --- | ---: |
| Block-event rendering | 854 |
| Inline parser | 843 |
| Text escaping | 554 |
| Block parser | 548 |
| `memmove` | 502 |
| Inline rendering | 410 |
| Mark collection | 401 |
| Heading ID generation | 367 |
| Container matching | 245 |
| Paragraph-line parsing | 235 |

Heading ID generation is the most surprising newly visible feature cost. Its
profile includes hash-map insertion and rehashing. It needs a focused heading
corpus before changing its state model.

### Delimiter-heavy lane

The dominant buckets were mark collection (1,162), the inline parser (1,038),
inline rendering (623), emphasis resolution (500), text escaping (496),
`memmove` (426), and emit-point insertion sorting (405). This is strong evidence
for reducing repeated inline-stage work or emitted intermediate work. It is not
evidence that transformations require removal of events.

### Safe-URL-heavy lane

The dominant buckets were the inline parser (951), raw URL destination escaping
(848), HTML entity decoding (620), link resolution (482), URL safety
classification (272), URL encode/HTML escape (220), UTF-8 conversion (206), and
string pattern searching (172).

This is the clearest evidence of repeated work. Entity-free safe ASCII URLs
still pass through several classification, conversion, encoding, and escaping
steps. Consolidating scans has a measurable upper bound without weakening the
secure rendering contract.

## CPU-specific ceiling

Short `Extended secure` 50 KB runs produced:

| Code generation | Throughput |
| --- | ---: |
| portable/generic | 213.82 MiB/s |
| `apple-m1` + NEON | 218.42 MiB/s |
| `target-cpu=native` | 216.46 MiB/s |

The M1 mode was about 2.2% above the portable smoke; `native` did not beat the
explicit M1 mode. This suggests that CPU-specific code generation matters but is
not a large hidden reserve on this machine. Existing M1/NEON tuning captures
most of the visible ceiling. PGO remains unmeasured because no representative
`.profdata` artifact exists yet.

## Ranked opportunity map

### 1. Consolidate the secure URL pipeline

- **Evidence:** several large, separately visible URL decode/classify/encode
  buckets; 126.78 MiB/s on the focused safe-URL smoke.
- **Smallest experiment:** classify entity-free ASCII URLs and share one scan
  result between safety and destination rendering; keep the current fallback.
- **Expected return:** 1-3% on mixed secure input, substantially more on
  URL-heavy input.
- **Risk:** security-sensitive; differential/property tests are mandatory.
- **Extensibility:** neutral. The semantic link event and trust boundary remain.
- **Stop:** no repeated mixed-lane gain or any security/output difference.

### 2. Reduce delimiter-stage repeated work

- **Evidence:** mark collection, inline parsing, emphasis resolution, and
  emit-point sorting dominate the delimiter profile.
- **Smallest experiment:** have mark collection return a compact feature summary
  and skip only downstream stages proven absent.
- **Expected return:** 2-5% delimiter-heavy, potentially 1-2% mixed.
- **Risk:** high semantic surface around precedence and nesting.
- **Extensibility:** low impact if resolved events remain the boundary.
- **Stop:** no gain across simple, mixed, and delimiter-heavy corpora or any
  CommonMark regression.

### 3. Focus heading-ID state and hashing

- **Evidence:** 367 exclusive mixed-lane samples plus visible hash insertion and
  rehash work.
- **Smallest experiment:** add repeated/unique heading corpora and measure ID
  creation, collision tracking, allocations, and disabled-path cost.
- **Expected return:** unknown until isolated; potentially feature-specific
  0.5-2% on heading-heavy documents.
- **Risk:** duplicate-ID semantics and output compatibility.
- **Extensibility:** neutral; heading events and transformability remain.
- **Stop:** heading generation is not repeatably material after isolation.

### 4. Borrow contiguous paragraphs with a buffered fallback

- **Evidence:** about 36 KB copied per 52 KB document plus 502 `memmove` samples
  in the mixed profile.
- **Smallest experiment:** count source-contiguous paragraphs, then borrow only
  the simple case while preserving the existing buffer for indentation and
  container normalization.
- **Expected return:** 1-3% on 20/50 KB and lower copied bytes.
- **Risk:** range and newline normalization correctness.
- **Extensibility:** favorable if the borrowed slice feeds the same semantic
  event path.
- **Stop:** fewer copied bytes without a repeated throughput gain, or complexity
  leaks into consumers.

### 5. Audit fixed small allocations, not allocation count alone

- **Evidence:** Ferromark has higher small-document allocation counts but much
  lower requested-byte volume; `InlineParser::new` is visible but not dominant.
- **Smallest experiment:** attribute allocation stacks for Essentials and
  Extended 5 KB, then lazily initialize one rare buffer group at a time.
- **Expected return:** 1-3% at 5 KB; likely less at 50 KB.
- **Risk:** replacing useful preallocation can regress larger inputs.
- **Extensibility:** neutral.
- **Stop:** total bytes or 20/50 KB throughput regress by more than 1%.

### 6. Keep direct rendering optional

- **Evidence:** event volume, rendering, and copying are material, but the current
  profiles do not isolate event construction as the dominant universal cost.
- **Smallest experiment:** define an internal sink over already resolved inline
  semantics and compare event and HTML sinks without deleting either path.
- **Expected return:** potentially 2-5%, still unbounded.
- **Risk:** duplicated behavior and divergence between sinks.
- **Extensibility:** high impact; the event/transform path must remain canonical
  or share resolution logic.
- **Stop:** requires duplicated parsing rules or cannot preserve an explicit
  transformable path.

### 7. Defer more hand-written SIMD

- **Evidence:** existing M1 mode is only about 2.2% above portable and `native`
  adds nothing in the smoke; earlier SIMD variants were neutral.
- **Next gate:** instructions/cycles or a hot scalar scan must show a bounded
  opportunity not already covered by `memchr` or NEON.
- **Extensibility:** neutral, but portability risk is high.

## Recommended execution order

1. Run publication-quality CommonMark parity repetitions with the pinned
   toolchain.
2. Run focused heading-ID and allocation-stack diagnostics.
3. Implement the secure URL shared-scan experiment.
4. Implement the inline feature-summary experiment.
5. Measure source-contiguous paragraphs and prototype the borrowed fast path.
6. Re-profile before considering an optional direct-render sink.
7. Build representative PGO training data only after portable source changes
   stabilize.
