# Measured optimization ports

The [first SIMD round](reports/2026-09-14-simd-round/README.md) measured a small
link-unescape prototype. It remains on local branch `codex/simd-link-probe`:
parsing/full processing improved, but render-only controls need explanation
before promotion. `main` retains the original core implementation.

OX-Content already contains SIMD nibble classifiers,
SWAR line scans, optimized code-span scanning, arena sizing, and an autolink
first-byte index; importing those same ideas again would not be a new port.

## Measured priority after the broad comparison

The [broad Markdown comparison](reports/2026-09-14-broad-markdown/README.md)
supersedes synthetic-only prioritization. Initial v2 wins 35 of 36 comparable
cases with reuse; the 310-byte table comment remains a counterexample. That
identifies a workload, not a proven scanner bottleneck. Profile before importing
v1's table or nested-list techniques.

The SIMD prototype's broad geometric-mean ratios are 1.049× for parsing and
1.034× for full processing, with unchanged allocation counters. Its repeated
render-only losses on two large cases prevent a claim of improvement across all
stages. Investigate code generation/layout and memory placement before another
acceptance run. All candidate variants, raw measurements, and controls are in
the report.

## 1. Combine optional inline marker searches

Donor: [Ferroni's RegSet candidate scanner](../../ferroni/src/regset.rs), especially
`first_byte_candidates` and `SkipNeedle`. Background:
[Ferroni's search ADR](../../ferroni/docs/app/routes/adr/007-simd-accelerated-search.mdx).

Target: [inline/marker_scan.rs](../crates/ferromark_parser/src/parser/inline/marker_scan.rs).
The current parser maintains separate forward scans for the core markers and
optional `{`, `^`, and `$` syntax. Explore combining active marker candidates,
using specialized short-set searches where that reduces repeated traversal.
Retain the current path as the baseline; the donor's regex workload does not
establish a Markdown speedup.

Check every enabled-marker combination, cursor offset, UTF-8 input, short tail,
and no-match case against a scalar oracle. Measure plain text, MDX, math, and
superscript separately before any end-to-end conclusion.

## 2. Link unescape: measured single-probe prototype

Donor: [Ferrocat's structural scanner](../../ferrocat/crates/ferrocat-po/src/scan.rs)
(`find_escapable_byte`) and [text routines](../../ferrocat/crates/ferrocat-po/src/text.rs)
(`unescape_string_known`, `escape_string_from`). Reuse the scan/copy structure,
not the PO format's escaping rules.

Target: `unescape_link_component` in
[inline/link_target.rs](../crates/ferromark_parser/src/parser/inline/link_target.rs),
then the related [reference path](../crates/ferromark_parser/src/parser/reference.rs).
The measured candidate probes once with `memchr2`, returns unchanged components
immediately, and keeps the original scalar tail after the first candidate byte.
Repeated SIMD searches regressed on dense escapes; an eight-byte local probe
did not resolve the tradeoff. Preserve Markdown's backslash/entity semantics and
borrowed-versus-arena-owned output behavior when revisiting this work.

Use URL/title-heavy documents, many short links, escaped punctuation, malformed
entities, numeric entities, and Unicode. Require equal AST/output, allocation
counts, and full conformance results. See the report for the measured gain and
the render-only acceptance blocker. The
[expanded inventory](reports/2026-09-14-simd-round/INVENTORY.md) also identifies
table scan fusion and bare-URL scanning as future candidates.

## Port discipline

1. Freeze representative inputs and the current accepted commit as the reference. Measure
   parse-only, render-only, and parse-plus-render with identical options and
   allocation/reuse policies.
2. Introduce one bounded change per commit. Keep default CommonMark/GFM and optional
   MDX/extension modes in the correctness and performance matrix.
3. For SIMD, retain a scalar fallback and test all byte values, alignments, tails,
   and high-bit bytes. Validate AArch64 and x86-64 separately; do not generalize
   from this fork's initial AArch64-only build/test run.
4. Run existing tests and snapshots unchanged, then randomized/adversarial guards
   relevant to the change. Verify byte-identical output before timing.
5. Record raw repetitions, hardware, toolchain, options, and confidence/variance.
   Keep only changes whose end-to-end gain survives noise without a material
   regression on short/plain input or increased retained memory.

Avoid wholesale arena or AST redesign as the first port. The current no-drop
arena and compact-node invariants are valuable baseline properties; allocation
savings alone are insufficient evidence for a throughput change.
