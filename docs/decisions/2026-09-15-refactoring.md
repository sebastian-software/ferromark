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
