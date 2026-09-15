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
