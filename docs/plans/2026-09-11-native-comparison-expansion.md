# Native comparison expansion

**Date:** 2026-09-11
**Base:** main `223cff7495435c8fc44713fd4852ff91e0327bf2`, after PR #295.
**Status:** Native cmark/cmark-gfm adapters implemented and [measured](../reports/2026-09-11-native-cmark-comparison.md); Goldmark and Sätteri are subsequent work.

## Scope

Compare native parsing and rendering APIs only. Node.js wrappers, JS bindings,
WASM, and per-document CLI startup are excluded for every candidate, including
Ferromark. AST construction, allocation, output ownership, and runtime GC remain
part of an engine's native work. No assumptions about language alone substitute
for measurement.

The merged baseline already has Bun, pulldown-cmark, Comrak, and MD4C adapters,
reviewed workload admission, and realistic tables with plain cells, links, and
other CommonMark inline content. Reuse those fixtures and review rules before
adding more syntax or changing public comparison tables.

## Sequence

1. **cmark and cmark-gfm:** add the C CommonMark reference implementation and its
   GFM fork. The [native pair harness](../../benchmarks/cmark-comparison/README.md)
   links one C library with Ferromark per process to avoid conflicting exported
   symbols. Core cmark participates only in CommonMark; the fork also runs
   tables, double-tilde strikethrough, tasks, their shared overlap, and neutral
   controls. These are system-allocator results, separate from Bun's pinned
   nightly/mimalloc environment. Public figures require a deliberate later
   publication integration, not copying numbers between environments.
2. **Goldmark:** pin a released Go version and library revision; assess v1/v2
   independently instead of silently changing generations. Use a long-lived
   native Go driver, with parsing, rendering, fresh output, and normal GC inside
   its measurement windows. Exchange fixture/results outside timing. Interleave
   measurement windows with a native Ferromark process and record scheduler,
   GC, toolchain, and memory settings. Do not force Go allocations through the
   Bun allocator or include a cgo call on every render merely to share a driver.
3. **Sätteri Markdown:** invoke its Rust API with JS plugins disabled. Pin its
   actual pulldown-cmark dependency, feature settings, output stage, and native
   compiler dependencies. Measure the full native Markdown-to-HTML pipeline;
   measuring pulldown alone would duplicate an existing candidate.
4. **Sätteri MDX:** first define an output-stage and semantic compatibility
   corpus. Ferromark segments/preserves JSX, ESM, and expressions and can emit a
   JSX/TSX module. It does not validate all JavaScript or implement the complete
   MDX grammar. Sätteri's native compiler does additional JS/MDX work. Follow
   [ADR-0009](../arch/ADR-0009-mdx-compatibility-and-performance-boundaries.md):
   report partial processing separately from compilation; do not present their
   ratio as equal-work throughput. Do not add a Node measurement as a fallback.
5. **Optional screening:** Markdig, native MD4X, and other plausible native
   contenders once the first three additions have measured evidence. Lower
   prominence does not exclude a credible performance candidate.

## Capability contracts

Keep capabilities bound to tested commits and individual flags. Each addition
needs executable positive/negative fixtures before timing is admitted:

- CommonMark core, with raw HTML/URL policy explicitly matched.
- Tables, double-tilde strikethrough, tasks, and their common overlap.
- Bare autolinks and GFM tag filtering tested independently before any lane is
  called full GFM. Goldmark's documented GFM bundle omits tag filtering;
  pulldown's `ENABLE_GFM` is not an aggregate GFM switch.
- Footnotes, math, alerts, heading IDs/attributes, metadata, definition lists,
  and other syntax each get separate supporting-engine subsets. Preserve
  syntax and output differences; parsing math is not typesetting it.
- Native MDX segmentation, semantic events, JSX module generation, and JS
  compilation are different work contracts.

Do not let an unsupported feature disappear into literal text and then appear
as a speed win. Continue reporting HTML fidelity separately from admission under
[ARCH-COMP-002](../arch/ARCH-COMP-002-workload-comparability.md).

## Sources

- [cmark](https://github.com/commonmark/cmark): C CommonMark reference implementation.
- [cmark-gfm](https://github.com/github/cmark-gfm): GitHub's extension fork.
- [Goldmark](https://github.com/yuin/goldmark): native Go API, GFM bundle, v2 migration.
- [Sätteri](https://github.com/bruits/satteri): native Rust Markdown/MDX pipeline.
- [Astro 7](https://astro.build/blog/astro-7/): Sätteri integration and native compiler stages.

These sources motivate adapter selection; their project benchmarks are not
Ferromark measurements or evidence that a candidate matches our speed.
