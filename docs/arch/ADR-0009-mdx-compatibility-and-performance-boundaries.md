# ADR-0009: MDX Compatibility and Performance Boundaries

**Status:** Accepted
**Date:** 2026-07-24

## Context

Official MDX permits Markdown blocks inside JSX when the JSX and the Markdown
are on separate lines. This includes common documentation patterns such as a
heading or a list nested in a component. The official compiler also treats JSX
as part of a larger grammar: it changes some Markdown syntax rules, recognises
inline MDX, and parses JavaScript expressions/ESM.

Ferromark's current MDX renderer intentionally uses a lightweight, line-based
segmenter. It correctly renders Markdown inside root-level component tags, but
does not keep block-container state while detecting flow JSX/ESM. The opt-in
`parse_events` API can promote a tag-only paragraph inside a container, but it
is not a complete rendering-compatible MDX parser.

There was no measured record explaining the missing container-flow behaviour.
ARCH-EXP-016 now supplies a first bound: an intentionally incomplete,
zero-allocation, universal recognition pass added 9.8–12.2% to plain Markdown,
2.0–4.2% to root-level MDX, and 13.1–15.0% to container-heavy MDX. A correct
implementation requires more state and must be measured separately.

## Decision

Ferromark will provide one correct MDX parsing model rather than separate
"fast" and "compatible" modes. The existing `mdx` Cargo feature and `mdx::*`
API are the opt-in boundary: callers that use the normal Markdown API do not
pay for MDX recognition, while callers that select MDX get container-aware MDX
semantics without another public mode choice.

The MDX implementation must not add the universal second pass measured by
ARCH-EXP-016. Container recognition will instead be fused into the opt-in MDX
path or selectively invoked from proven candidates. The implementation must be
validated against reference MDX fixtures and benchmarked end to end before its
rendering semantics change.

"MDX compatibility" here means parsing MDX syntax and preserving its semantic
structure within Ferromark's renderer and event APIs. Ferromark does not execute
components or replace the JavaScript compiler provided by `@mdx-js/mdx`.

## Consequences

- Normal Markdown parsing remains independent of MDX recognition and retains
  its existing hot path.
- MDX callers do not need to understand or choose between two subtly different
  MDX dialects.
- The current line-based segmenter must evolve or be replaced inside the MDX
  module to support container-local flow constructs correctly.
- MDX rendering may become measurably more expensive as compatibility improves;
  that cost is accepted only in the already opt-in MDX path and must be
  published with correctness evidence.
- Full `@mdx-js/mdx` compiler parity remains out of scope unless a later
  decision expands Ferromark from parsing/rendering into JavaScript compilation.

## References

- MDX: What is MDX?
  https://mdxjs.com/docs/what-is-mdx/
- `@mdx-js/mdx` package architecture
  https://mdxjs.com/packages/mdx/
- ARCH-EXP-016: MDX Container-Flow Recognition Cost
