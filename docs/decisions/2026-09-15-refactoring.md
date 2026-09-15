# Incremental architecture cleanup

The owner authorized this order, with separate commits and retention only after
behavior and performance checks: source spans, definition collection, renderer
policy. Prefer bounded changes with exact output/source-position comparisons;
do not use cleanup as a reason to change syntax, published interfaces, or feature
costs silently.

## 1. Source spans — retained

Keep one full-tree traversal behind the existing static `SpanMap` contract.
Constant offsets and JSX byte mappings supply their own policies, alongside
existing source normalization and line mappings. Retain the specialized table
map and its restricted node domain. The shared traversal owns AST shape knowledge,
including attributes and captions, without introducing dynamic dispatch or new
source allocations. The [report](../reports/2026-09-15-refactor-spans/README.md)
records exact-output checks, tests and measured costs. No coverage exclusions
change.

## 2. Reference collection — retained

Use an internal parse phase chosen at construction, inherited by container
sub-parsers. Collection does not masquerade as disabled syntax options and cannot
start another document prepass. A necessary-shape filter keeps documents without
link-definition candidates out of collection. All candidate link definitions,
including root definitions, use the real block grammar. Remove the separate flat
scanner, quote stripping and joined paragraph bookkeeping. Footnote-only inputs
retain the independent raw-line label policy without a link-collection pass.

### Intended semantic correction

The removed flat scanner let an unclosed quoted fence hide a later root reference
definition. An old edge test explicitly required that incorrect result.
[CommonMark 0.31.2, section 4.5, example 128](https://spec.commonmark.org/0.31.2/#example-128)
requires a fenced block to end with its containing block quote. The real block
grammar already has that boundary. Update the incorrect edge expectation and add
exact renderer regressions for both fence characters and LF, CRLF and CR inputs.
This correction is within the owner's explicit authorization to fix official-spec
bugs. Specification fixtures and frozen conformance outputs are unchanged.

The [report](../reports/2026-09-15-refactor-references/README.md) records equivalence
on the measured corpora and performance. This does not redesign MDX footnote scope
or fix the separate label-only-inside-JSX gap.

## 3. Renderer link policy — retained

Keep one anchor-opening implementation for URL conversion, sanitization, escaping,
external-link attributes, and titles. Both normal and hook-enabled rendering call
it; each retains its child traversal and saves/restores the existing `in_link`
state. The default path does not run through hooks. This change adds no public API,
callback dispatch or allocations and leaves image policy unchanged.

The [report](../reports/2026-09-15-refactor-renderer/README.md) records default and
no-op-hook measurements separately, including configured links. Existing tests
cover specification examples, reused renderer state, hook replacement/skip/wrap,
and highlighter fallback. No-op measurements do not model user callback costs.
Heading writers already share lower-level helpers; this bounded step does not
introduce a generic renderer framework or change AST visitation.

## Final retention check

Retain all three bounded changes. A direct 57-document comparison against the
pre-refactor commit measures +0.56% fresh and +0.38% reused overall. Accept this
small cost for removing 529 net production lines and duplicated responsibilities;
this is not a claim of perfect cost neutrality. The targeted reference improvement
and quoted-fence correctness fix provide additional value. All 828 workspace tests,
Clippy, formatting, benchmark builds, Node checks and the unchanged coverage gate
pass. Coverage is 93.99% lines without new exclusions; denominator reduction is
part of the increase. Detailed costs and limitations remain in the linked reports.

## 4. Inline helper file boundaries — retained

Replace the mixed `inline_helpers.rs` module with owners named for their work:

- `inline/image.rs`: image syntax and alternative-text flattening, 153 lines.
- `delimiters.rs`: shared marker runs, closed-code skipping, cached closer
  discovery and balanced brackets, 153 lines; its existing scalar-oracle tests
  live in `delimiters/tests.rs` (170 lines).
- `inline.rs`: the existing inline-node capacity and text-node construction
  helpers stay next to their primary caller. Capacity is private; image parsing
  is visible only inside the inline module. No visibility is widened.

Delimiter discovery also serves math and MDX, so it remains a parser-level module
rather than being hidden under image or link parsing. These are organizational
boundaries within the existing parser, not independently stateful components.
Keep algorithms, signatures, parser layout, allocation policies, fixtures and test
expectations unchanged. No traits, new crates or runtime dispatch are introduced.
Do not split remaining files purely to meet a line-count limit. The architecture-
specific marker scanner and URL escaping remain separate prospective steps.

See the [measurement report](../reports/2026-09-15-refactor-inline-files/README.md)
for the retention decision and validation against `61eafed`.

The 57-document aggregate is −0.19% fresh / −0.31% reused, with exact HTML/AST
preserved. A mixed fresh-footnote outlier was investigated using longer windows
and did not repeat (−0.08% fresh / +0.10% reused). Retain the boundaries for clearer
ownership and narrower visibility. All 828 workspace tests, Clippy, formatting and
benchmark builds pass. The change is organizational; no speedup is claimed.
