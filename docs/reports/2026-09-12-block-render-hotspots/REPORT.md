# Block rendering and validation after PR #311

This round measures four remaining costs from the preceding native profiles:
list fixup, fenced-code continuation dispatch, per-line render events, and final
UTF-8 validation. The retained implementation skips unnecessary list fixup,
consumes root fences directly, compacts root ranges only for rendering, and uses
a checked ASCII prefix to accelerate larger string conversions. Event reservation
now depends on actual event density before applying the existing input-size hint.

[RESULTS.md](RESULTS.md) is generated from archived raw observations. It contains
all exploratory screens, the three production process pairs, requested allocation
counters, and a fresh native Ox corpus comparison. Negative paired changes mean
less elapsed time. The source and executable identities are in
[metadata.json](metadata.json); [implementation.patch](implementation.patch)
contains the production change relative to the frozen baseline.

<!-- measured-summary:start -->

## Confirmed outcomes

| Document | Baseline µs | Production µs | Relative changes across three pairs |
| --- | ---: | ---: | --- |
| Compiler Options | 30.408 | 27.164 | -10.64%, -10.62%, -10.67% |
| Compiler Options in MSBuild | 17.810 | 16.162 | -9.35%, -9.49%, -8.57% |
| Vue render function | 44.071 | 38.435 | -12.73%, -13.50%, -12.59% |
| Vue Suspense | 17.078 | 16.674 | -2.25%, -2.37%, -2.32% |

0 of 93 inputs are more than 2% slower in all three pairs. Individual changes and all raw observations remain visible in RESULTS.md.

Representative large-input requested heap peaks (owned HTML live, input excluded):

| Input | Bytes | Baseline peak bytes | Production peak bytes | Change |
| --- | ---: | ---: | ---: | ---: |
| large/mixed-8mib | 8388689 | 58547388 | 58547388 | +0.00% |
| large/plain-8mib | 8388608 | 44040224 | 35652640 | -19.05% |
| html/root-8388608 | 8388608 | 18875904 | 10488320 | -44.44% |
| html/comment-8388608 | 8388608 | 18875904 | 10488320 | -44.44% |
| html/script-8388608 | 8388608 | 27264512 | 10488320 | -61.53% |
| fence/root-8388608 | 8388601 | 37750222 | 20974062 | -44.44% |
| fence/indented-8388608 | 8388604 | 37750230 | 37750262 | +0.00% |
| fence/container-8388608 | 8388604 | 37750238 | 37750238 | +0.00% |

<!-- measured-summary:end -->

## Diagnosis and isolated experiments

The frozen baseline is merged PR #311, commit
`9b1965bd52b856b2675b23bcf6fe545e713d94cd`.
The original output oracles were regenerated from that source before changing it.
Baseline profiles are separate from the production profiles. An attribution-only
probe adds `#[inline(never)]` to `HtmlWriter::into_string`; it identifies the final
owned-string conversion as the dominant UTF-8 validation call on Compiler Options.
That probe is neither the timing baseline nor the production implementation.

The ranked hypotheses were tested separately before combinations:

- **List post-processing.** Detecting only the absence of lists helps pure HTML,
  but tight lists also need no correction. The retained flag records whether any
  list closes loose. Fixup then applies each closing value directly, using an
  inline stack that can spill for deeper nesting. Public raw parser events still
  start tight, and the public helper still tolerates unmatched starts and ends.
- **Fenced-code continuation.** A root loop entered from the recognized opener
  avoids repeated general block dispatch. Container matching, indentation and
  closing-fence rules remain shared with the ordinary path. A separate early
  branch for plain content bytes was screened but is not retained.
- **Earlier event compaction.** Merging every code/HTML emission adds overhead to
  indented and container cases. The retained version instead defers root HTML
  continuation emission until the block ends and borrows an unindented root
  fence's complete body. Public block/MDX streams retain their existing granularity. Filtered
  trusted HTML retains its original slices for tag-filter lookahead; prefix
  removal and virtual spaces continue through the existing renderer.
- **UTF-8 validation.** A whole-output `is_ascii` check followed by the standard
  conversion rescans large prefixes when Unicode occurs late. Dedicated early,
  middle and late Unicode controls expose that regression. Standard blockwise
  `is_ascii` checks also provide no useful gain here. The retained OR reduction
  proves complete ASCII blocks, then validates only the remaining suffix. The
  larger loop is outlined to protect short conversions from code expansion.

Compaction alone does not remove an input-sized reservation. Uniformly smaller
reservation ratios and a fixed cap were therefore measured independently. The
retained adaptive variant starts with 64 slots and parses until 64 events or EOF
before applying the previous `input.len() / 16` hint. Already-sized reused buffers
and short inputs skip that initial phase. If input remains after that phase, the parser applies the hint;
large compact streams avoid it. The threshold is checked between parser calls.
A single uncompacted block can consume the remaining input and grow its buffer
naturally before the threshold is checked, which explains additional allocations
in some large indented fences. Ordinary EOF cleanup runs through the same parser.

The candidate directories preserve source hashes, patches, guards and raw windows.
`variants/` contains the exact exploratory sources, including the attribution
probe. Combinations were screened before freezing and formatting production.
Single exploratory runs are not presented as confirmed application speedups.

## UTF-8 safety and compatibility

The public byte writer still accepts arbitrary bytes and validates them when
converting to a string. Each accepted 4096-byte prefix block has no high bit in its
OR reduction, proving that every byte is ASCII. ASCII ends at a character boundary.
The remaining suffix must pass the standard UTF-8 validator before the ownership
conversion can be unchecked. The owned bytes do not change between those checks
and conversion. This is a local proof over the actual output, not an assumption
about parser input or a claim based only on fuzzing.

If the suffix is invalid, the original full conversion constructs the error,
preserving its offset, error length and owned bytes. If no complete ASCII block is
proven, the original conversion handles the whole buffer. Short outputs and the
public `as_str` method retain their standard validation paths. No dependency or
public API is added.

## Validation and measurement boundary

The standalone driver uses native Rust, fresh render state and owned HTML, except
for explicitly named reuse cases. It builds outside repository Cargo configuration
with locked dependencies, opt-level 3, fat LTO, one codegen unit, panic abort and
the system allocator. Inherited Rust flags are removed. The machine is an Apple
M1 Pro; compiler and target details are in the metadata. Builds, tests, profiles
and other timing suites run sequentially rather than competing with timed windows.

Each exploratory screen uses five alternating 30 ms windows per implementation
and input after 60 ms warmup. The first four screens have 81 inputs. Subsequent
screens and production have 93: six additional fence shapes and six 64 KiB
Unicode-position controls. Production runs three fresh process pairs with nine
alternating 75 ms windows per implementation/input. Every raw elapsed/count window
and per-input median is retained. A cross-input median is descriptive rather than
a weighted application workload or engine ranking.

The exact-output checks compare HTML, public block and inline events, complete
MDX event streams and source segments, headings, resource-limit metadata and
reused state with the frozen implementation. There are 111,902 logical comparisons,
including 12,000 deterministic fence/container cases. The additional repository
tests cover physical fence ranges, callback content/metadata, list fixup and reuse,
and standard-library UTF-8 results including invalid sequences at scan boundaries.
The tests pass on the frozen source before the change and on production.

Memory uses 57 inputs, three fresh processes per source and ten observations per
process/input, including ordinary, HTML and fenced-code documents up to 8 MiB.
Counters cover requested heap allocations during rendering with owned HTML still
live; input storage is excluded. These are not RSS, stack usage or total process
memory. Compaction and adaptive reservation are evaluated by these counters rather
than inferred from event count. Allocation calls and requested bytes can differ
from peak live bytes; the complete summaries retain all of them. The adaptive
reservation adds allocation calls for some event-rich inputs, and two small
controls have higher peaks. This is not a claim that every allocation counter
improves on every document.

The eight inputs of at least 8,000,000 bytes are also checked for latency with
three additional process pairs. Each pair uses five alternating windows requested
at 30 ms after 60 ms warmup. The driver checks its timer after each 16-render batch,
so large-input windows can run substantially longer; every reported time uses the
actual elapsed time divided by its render count. This follow-up specifically
checks the possible latency cost of additional growth allocations.

The fresh Ox corpus comparison uses a separate native driver with heading IDs
enabled, three process groups, nine rotating 60 ms windows per engine/input and
35 ms warmup. Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`.
Its growing arena is primary and presizing remains visible separately. Only equal
normalized outputs support a comparison; unequal outputs remain explicit
diagnostics. Absolute times from that driver must not be subtracted from the
optimization driver's times. The pinned build and input provenance are documented
in the earlier [Ox investigation](../2026-09-12-ox-corpus-profiling/REPORT.md).

## CI compiler follow-up

CI's Clippy 1.98 flags `chunks_exact(4096)` in favor of `as_chunks::<4096>().0`.
The fixed-size array view retains the same ASCII proof and UTF-8 error contract,
but it changes several generated instructions. All final production measurements
and output checks were therefore repeated on the corrected source. The original
implementation's measurements, source identity and profiles remain archived under
`pre-ci-production`, `memory/pre-ci-production`, `corpus/pre-ci`, `large/pre-ci`,
and the `pre-ci-production` profile prefix. The current result tables use the
corrected production source.

## Remaining profile costs

The production captures still place most Compiler Options samples in block
parsing, including root HTML recognition and newline scanning. Vue render function
now spends a smaller share in visible block frames, leaving inline parsing,
escaping and copying as substantial costs. Inlining changes which symbol receives
samples: the newly visible HTML newline helper is not evidence that newline
scanning regressed. Paired elapsed times, rather than symbol percentages alone,
measure the improvement. Raw captures and `profile-summary.json` preserve the
complete attribution, including the separate UTF-8 diagnostic probe.

## Reproduction and published snapshots

[REPRODUCE.md](REPRODUCE.md) explains source reconstruction, exact-output replay,
timing, memory and profile commands. `check-evidence.py` checks every archived file
hash and recomputes paired timings, corpus medians and allocator summaries from
raw observations. `replay.py --verify` rebuilds both sources and checks the baseline
against the archived oracles before production. Required Rust checks and the
benchmark/profiling contracts are recorded under `validation/`. Logs and patches
retain their original bytes, including tool-generated whitespace.

The README and homepage remain the independently dated full-comparison snapshots
from PR #308. This focused experiment does not replace individual published cells
with measurements from a different driver. The decision is recorded in
[ARCH-EXP-025](../../arch/ARCH-EXP-025-block-render-hotspots.md).
