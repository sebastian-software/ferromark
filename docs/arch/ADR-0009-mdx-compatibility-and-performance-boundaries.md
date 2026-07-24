# ADR-0009: MDX Compatibility and Performance Boundaries

**Status:** Proposed
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

Ferromark will retain the current lightweight `mdx::segment` and `mdx::render`
path as the performance-oriented default. We will **not** add an unconditional
container-aware pre-scan to that path.

Full `@mdx-js/mdx`-style compatibility, if offered, will be a distinct,
explicitly selected compatibility path rather than an undocumented behaviour
change to the fast renderer. The exact public API is intentionally deferred
until a correct prototype exists; plausible forms include an MDX rendering
options type or a separately named compatibility entry point. The existing
semantic-event API remains opt-in and is not represented as full MDX compiler
compatibility.

No compatibility API is accepted by this ADR yet. It must first meet the
follow-up gate in ARCH-EXP-016, including reference-implementation fixtures and
end-to-end benchmarks. Its API and default must then be reviewed in a follow-up
ADR update or successor.

## Consequences

- Current renderer throughput and its simple, zero-copy segmentation contract
  remain stable for existing callers.
- Ferromark documents the deliberate gap instead of implying complete MDX
  support.
- Users needing container-aware or complete MDX behaviour have a clear path to
  a future opt-in mode instead of an accidental performance regression.
- A future compatibility implementation must preserve the default renderer's
  benchmark baseline and publish both correctness and performance evidence.
- Complete MDX parity is treated as a compiler/product direction, not as a
  small extension to the Markdown hot path.

## References

- MDX: What is MDX?
  https://mdxjs.com/docs/what-is-mdx/
- `@mdx-js/mdx` package architecture
  https://mdxjs.com/packages/mdx/
- ARCH-EXP-016: MDX Container-Flow Recognition Cost
