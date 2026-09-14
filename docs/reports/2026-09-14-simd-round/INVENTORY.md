# SIMD round inventory

This inventory records baseline v2 paths and donor techniques. The measured
single-probe prototype and its acceptance decision are in the
[round report](README.md). Every candidate must preserve byte-identical parser
and renderer output before timing.

## Existing accelerated paths

- [Inline marker scan](../../../crates/ferromark_parser/src/parser/inline/scan.rs)
  already has a scalar flag-table path plus NEON, SSSE3, and AVX2 nibble
  classifiers. [Optional marker dispatch](../../../crates/ferromark_parser/src/parser/inline/marker_scan.rs)
  still calls separate cached `memchr` searches for `{`, `^`, and `$`.
- [Pre-pass line scanning](../../../crates/ferromark_parser/src/parser/line_scan.rs)
  already uses SWAR and an AArch64 NEON path. [Table splitting](../../../crates/ferromark_parser/src/parser/table.rs)
  and [table-pipe unescaping](../../../crates/ferromark_parser/src/parser/table_cell_source.rs)
  use `memchr`, but perform separate boundary and unescape scans.
- [Renderer escaping](../../../crates/ferromark_renderer/src/html/escape.rs)
  uses 64-bit SWAR masks, overlapping tails, and short-run copy logic.
  [Nibble classifiers](../../../crates/ferromark_renderer/src/html/escape/nibble.rs)
  provide AArch64 vector and runtime-detected x86-64 SSSE3 paths. The current
  short path is already SWAR; an input-length threshold should be treated as
  target-specific because its x86 break-even is unvalidated on the M1 host.
- [Bare-URL indexing](../../../crates/ferromark_renderer/src/html/autolink/index.rs)
  already gates default URLs with `memmem` for `://`, uses `memchr`/`memchr2`/
  `memchr3` for small candidate sets, and has second-byte and gate-tail filters.
  Custom-pattern overflow intentionally falls back to a table scan.

## Ranked candidates

### 1. Link and reference unescape: first bounded experiment

Target [inline/link_target.rs](../../../crates/ferromark_parser/src/parser/inline/link_target.rs),
`Parser::unescape_link_component`; reference definitions reuse it through
`unescape_reference_component` in [reference.rs](../../../crates/ferromark_parser/src/parser/reference.rs).
The baseline loop examines every byte, including unchanged URL and title text.
The first bounded experiment is an initial `memchr2(b'\\', b'&')` probe, then a
scan/copy loop that bulk-copies unchanged spans. Keep the existing arena string
allocation only after a valid backslash escape or entity is found.

The structural scan/copy shape is supported by Ferrocat's local donor
[scan.rs](../../../../ferrocat/crates/ferrocat-po/src/scan.rs) and
[text.rs](../../../../ferrocat/crates/ferrocat-po/src/text.rs). This is donor
evidence about a technique, not evidence of a v2 gain. Measure URL/title-heavy
inputs, many short links, valid and malformed entities, numeric entities,
escaped punctuation, and Unicode before deciding whether to keep it.

Preserve ASCII-punctuation-only backslash removal, literal malformed `&`, entity
expansion, byte-safe UTF-8 slicing, and the borrowed-versus-arena-owned result.
The related reference-definition path must produce the same labels, URLs, and
titles.

### 2. Fuse optional inline marker searches

Target [inline/marker_scan.rs](../../../crates/ferromark_parser/src/parser/inline/marker_scan.rs).
When MDX, superscript, or math is enabled, the current `special`, `brace`,
`caret`, and `dollar` forward scans can traverse the same suffix independently.
A merged active-marker classifier/cache is the bounded experiment; retain the
existing core path for the default option set.

The relevant donor definitions are [v1 inline/simd.rs](../../../../ferromark/src/inline/simd.rs)
and [v1 byte_search.rs](../../../../ferromark/src/byte_search.rs). The pinned
source `/private/tmp/ferromark-v2-comparison/main/src/byte_search.rs` additionally
shows an AArch64 nibble-table classifier and first-lane extraction. Neither
donor establishes a v2 end-to-end improvement.

Check every enabled-marker combination, cursor offset, no-match case, short
tail, and high-bit byte against a scalar oracle. Disabled options must never
become candidates, and the earliest-byte result must remain unchanged.

### 3. Fuse table boundary and escaped-pipe scans

Targets [table.rs](../../../crates/ferromark_parser/src/parser/table.rs),
`table_row_cells_with_offsets`, and
[table_cell_source.rs](../../../crates/ferromark_parser/src/parser/table_cell_source.rs),
`unescape_table_pipes`. The boundary pass searches for `|` and reverse-counts
backslashes; the cell pass searches for `|` again and builds source offsets.
Explore one forward `memchr2`/`memchr3` or byte-set pass carrying slash parity,
while retaining a cheap plain-cell fast path.

Ferromark's table donor [block/parser.rs](../../../../ferromark/src/block/parser.rs)
uses a three-byte search for pipe, backslash, and backtick with a stateful token
advance. It is a scan-shape
donor only: v2 must retain its own escaped-pipe and source-offset semantics, and
the donor's backtick-opaque behavior cannot be copied without differential tests.
Use the existing table-pipe and table benchmarks as the falsification screen;
the prior local comparisons identify table workloads as a remaining v2
counterexample, but do not isolate this scanner as the cause.

Correctness checks must cover leading/trailing pipes, odd and even backslash
runs, escaped pipes inside code spans, Unicode, truncation, and every remapped
inline span.

### 4. Renderer bare-URL scan: future measurement candidate

Targets [html/autolink.rs](../../../crates/ferromark_renderer/src/html/autolink.rs),
`scan_url_end` and `trim_trailing_punct`. `scan_url_end` currently tests nine
ASCII terminators per byte and falls back to scalar UTF-8 decoding plus the
CJK/fullwidth `ends_url` ranges. A vector ASCII scan may be explored only with
an exact scalar fallback at the first non-ASCII byte. Preserve all nine ASCII
terminators, UTF-8 boundaries, and CJK/fullwidth sentence punctuation.

`trim_trailing_punct` can recount bracket pairs when several closers are removed
in succession. The v1 donor [inline/links.rs](../../../../ferromark/src/inline/links.rs)
uses cached excess-close state for repeated `)`; that is an algorithmic donor,
not a drop-in SIMD port. v2 handles `()`, `[]`, and `{}`, so any cached balance
state must preserve each pair's current behavior.

The autolink index is already substantially gated, so measure this candidate
separately from index changes. No renderer speedup is claimed here.

## Lower-priority ideas

The short-input escape dispatch boundary needs x86-specific measurement before
changing its SSSE3/SWAR choice. The current M1 results cannot establish that
tradeoff. An ASCII-only fast path in
[heading.rs](../../../crates/ferromark_renderer/src/html/heading.rs) could skip
long lowercase alphanumeric runs, while keeping separator handling and Unicode
case expansion scalar. Heading scan cost has not been isolated as a v2
bottleneck, so this remains below the candidates above.
