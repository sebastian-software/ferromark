# ARCH-EXP-025: Block rendering and UTF-8 validation

**Status:** Accepted
**Date:** 2026-09-12

## Context

After ARCH-EXP-024, the native Ox corpus profiles still show list post-processing,
fenced-code continuation dispatch, per-line block events, and final UTF-8
validation. These are separate costs: reducing event count does not necessarily
reduce the capacity reserved for those events, and removing validation requires
a proof that covers the public byte-oriented writer.

## Decision

1. Track whether block parsing closes any loose list. List starts already default
   to tight, so other documents need no fixup scan. When fixup is needed, apply
   each closing value directly to its matching start. A small inline stack avoids
   the separate patch allocation and spills for deeper nesting. The public helper
   retains its handling of unmatched starts and ends.
2. Enter the root fence continuation loop from the recognized opener. Container
   prefixes still use the existing per-line path. Both paths share the closing
   delimiter check, including indentation, tabs, trailing whitespace and EOF.
3. Compact contiguous ranges only in the private render path. Root HTML defers
   continuation emission until the block ends; unindented root fences borrow one
   body range. Indented/container code and filtered trusted HTML retain their
   existing boundaries. Public block events, MDX metadata and MDX event streams
   retain their existing granularity. No public parser mode is added.
4. Start fresh rendering with a small event buffer. For larger inputs, parse until
   there are 64 events or EOF before applying the existing input-size reservation
   hint. Short inputs and already-sized reused buffers skip this initial phase.
   Large documents with compact event streams therefore avoid a reservation based
   only on source bytes. When input remains after that phase, the parser uses the
   reservation strategy established in ARCH-EXP-006. The threshold is checked
   between parser calls: a single uncompacted block can consume the remaining
   input and grow the buffer naturally before that check, adding allocations.
5. Keep UTF-8 validation local to `HtmlWriter::into_string`. For larger outputs,
   an OR reduction proves successive 4096-byte blocks are ASCII. Standard UTF-8
   validation checks the remaining suffix. Only those two successful proofs
   permit the unchecked ownership conversion; no parser-wide encoding invariant
   is assumed. Invalid input falls back to the original full conversion so error
   offsets and owned error bytes remain identical. The larger loop is outlined
   to keep it out of short-output conversion. `as_str` remains unchanged.

The ASCII prefix consists entirely of complete one-byte characters. Its endpoint
is therefore a UTF-8 boundary, and concatenating it with a validated suffix is
valid UTF-8. The owned bytes do not change between validation and conversion.
Tests against the standard library supplement this proof; they do not replace it.

## Rejected alternatives

- Detecting any list is too conservative: tight lists need no fixup either.
- A general merge check on every emitted code/HTML line adds bookkeeping to
  indented and container paths. Compaction stays at the recognized root block.
- Uniformly smaller reservation ratios and a fixed capacity cap produce less
  useful tradeoffs than waiting for evidence of event density.
- A whole-output ASCII test followed by full UTF-8 validation rescans long
  prefixes when Unicode occurs late. The exploratory corpus includes explicit
  counterexamples. Standard `is_ascii` prefix checks also do not provide a useful
  throughput gain in this measured environment.
- Smaller/inlined reduction variants can penalize tiny documents. The retained
  implementation isolates the larger conversion path and is measured as part of
  complete rendering, not just a validation microbenchmark.

## Evidence and limits

The [report](../reports/2026-09-12-block-render-hotspots/REPORT.md) contains the
isolated screens, three fresh production process pairs, exact-output oracles,
allocator observations up to 8 MiB, profiles, source identities and replay tools.
The baseline is merged PR #311. Measurements use native Rust, fresh owned HTML
except explicitly named reuse cases, the system allocator, and an Apple M1 Pro.
Other architectures require their own performance measurements.

The new tests cover public list fixup, renderer reuse, physical fence ranges,
custom fence callbacks, and valid/invalid UTF-8 at scan boundaries. Differential
guards also cover complete public block/inline and MDX output against the frozen
baseline, including 12,000 additional fence/container cases.

The native Ox comparison remains a separate driver with heading IDs enabled.
Its growing arena is primary, presizing is separate, and only equal normalized
outputs support a comparison. Allocation counters measure requested heap memory
with owned HTML live, excluding input storage; they are not RSS or total memory.
Published README/homepage tables remain independently dated full-comparison
snapshots rather than mixing these focused timings into their individual cells.
