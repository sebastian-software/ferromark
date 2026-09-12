# HTML line scanning after PR #310

The preceding [HTML parser experiment](../2026-09-12-html-block-parser/REPORT.md)
removed repeated general block dispatch. This round investigates the remaining
cost of short root HTML continuations and HTML opener lookahead. The retained
change uses a bounded constant byte search on NEON-enabled AArch64, specializes
single-byte comparisons in the shared scanner, and delays line lookahead when
only a tag prefix is needed. Parser state and public event granularity remain
unchanged.

[RESULTS.md](RESULTS.md) is generated from the archived observations. It contains
the complete exploratory screens, repeated production comparisons, allocator
measurements, fresh Ox comparison and exact-output checks. Negative paired changes
mean less elapsed time. The source identity and executable hashes are in
[metadata.json](metadata.json); the production diff is in
[implementation.patch](implementation.patch).

## Diagnosis and experiments

The frozen baseline is merged PR #310, commit
`41bf2701ffaeef9f2a247a89f1889be118d0a501`. Its source files and output oracles were
rebuilt before testing any candidate. An attribution probe adds only
`#[inline(never)]` to the HTML continuation loop: without it, the optimized debug
information attributes most loop samples to the calling HTML-start function.
The probe is diagnostic and is not the production implementation or timing
baseline. Native baseline and production profiles are also archived separately. The final captures still place most samples in block parsing. Inlining can shift individual symbol attribution; elapsed-time changes come from the paired runs.

The ranked initial hypotheses concern indentation, repeated block-kind dispatch,
parser-state updates, and premature full-line lookahead. The isolated probe then
identifies newline search as another material cost. Candidate changes are tested
independently before combinations:

- Local indentation, deferred column updates, separate block-kind loops and a
  locally held block kind do not provide a convincing enough benefit. The existing
  state machinery stays intact.
- A word-at-a-time prefix check helps some synthetic short lines, but provides
  less useful improvement on the real documents. Bounded memchr calls also offer
  less benefit.
- Calling `ByteSet::new` inside the loop adds work. The retained searcher is a
  compile-time constant, as the existing inline and escape searchers are.
- A single NEON byte equality avoids the nibble-table lookups used for larger
  sets. This optimization is confined to the existing checked SIMD helper.
- Scanning the whole remainder with the byte-set loop penalizes long lines. The
  retained search covers at most the first 128 bytes, then uses memchr. The final
  short remainder also keeps the original memchr path. Other architectures retain
  the original root HTML newline search.
- HTML types 1-6 only require an opening prefix and tag-name boundary. Type 7 still
  checks a complete physical line, including its attributes and trailing space.

The Ox source inspection supports examining short-line setup costs: its line
pre-pass uses a specialized scanner while other paths retain memchr. This is not
a port of that scanner; Ferromark reuses its existing byte-search implementation
and retains its existing LF/CR behavior. See the earlier
[Ox source investigation](../2026-09-12-ox-corpus-profiling/REPORT.md) for the pinned
source and corpus provenance.

## Validation and measurement boundary

The standalone Rust driver uses fresh render state and owned HTML, with explicit
reuse cases where the archived dataset requests them. It builds outside repository
Cargo configuration with locked dependencies, opt-level 3, fat LTO, one codegen
unit, panic abort and the system allocator. Inherited Rust flags are removed.
The machine is an Apple M1 Pro; compiler and target details are recorded in the
metadata. Builds, tests, profiles and other timing suites do not run concurrently
with timed windows.

Each exploratory screen uses five alternating 30 ms windows per implementation
and input, after 60 ms warmup. The first four state/indentation/recognition screens
run the original and public block/MDX guards; the scan screens run the original
and every timing-input guard. The formatted production source runs all guard
sets before three fresh process pairs with nine alternating 75 ms windows per
implementation and input. All per-input medians and raw elapsed/count windows
are retained. A cross-input median is descriptive, not a representative weighted
application workload.

The guards compare exact HTML, public block and inline events, complete MDX event
streams, source segments, headings, resource-limit metadata and reused state with
the frozen implementation. Existing differences from Ox remain in the guards.
New repository tests exercise physical-line boundaries, Unicode, NUL, raw CR,
unterminated tails, deep indentation and subsequent code-block columns. The shared
byte-search tests compare every possible single-byte needle with a scalar oracle
at vector and tail boundaries. These tests pass on the frozen source before the
optimization as well as on production.

Memory is measured separately: 51 inputs, three fresh processes per source and
ten observations per process/input, including ordinary and HTML documents up to
8 MiB. Counters cover requested allocations during rendering and retain the owned
HTML at the observation boundary. Input storage is excluded. These are allocator
measurements, not RSS; the optimization introduces no allocation or reservation.

The fresh Ox corpus comparison is a separate native driver with heading IDs
enabled, three process groups, nine rotating 60 ms windows per engine/input and
35 ms warmup. Ox is pinned to `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`.
The growing arena is primary; the presized arena remains visible separately.
Only equal normalized outputs support comparisons. Unequal cases remain
explicit diagnostics. Do not subtract its absolute times from those of the
optimization driver, or generalize selected cases into an engine ranking.

## Reproduction and published snapshots

[REPRODUCE.md](REPRODUCE.md) explains source reconstruction, output replay, timing,
memory and profile commands. `check-evidence.py` validates archive checksums and
recomputes paired timings, Ox corpus medians and allocation summaries from the raw observations. `replay.py --verify`
independently rebuilds the frozen sources and replays the output oracles.
Required local Rust checks and the relevant benchmark/profiling contracts are
recorded under `validation/`. Archived logs and patches retain their original
bytes, including tool-generated whitespace.

README and homepage tables remain the independently dated full-comparison
snapshots produced by PR #308. This focused optimization experiment does not
replace individual cells in those tables with results from a different driver.
The architecture decision is recorded in
[ARCH-EXP-024](../../arch/ARCH-EXP-024-html-line-scanning.md).
