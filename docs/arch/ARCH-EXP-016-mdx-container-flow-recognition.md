# ARCH-EXP-016: MDX Container-Flow Recognition Cost

**Status:** Complete
**Date:** 2026-07-24

## Question

Can Ferromark recognise flow JSX below Markdown block-container prefixes
(blockquotes and lists) without materially slowing the existing MDX renderer?

This is a narrower question than complete `@mdx-js/mdx` compatibility. The
reference compiler also changes Markdown grammar, parses inline MDX, validates
JavaScript, and compiles to JavaScript. This experiment measures only the
additional *recognition* work required before a container-aware implementation
could reuse Ferromark's existing JSX tag parser.

## Prototype

`benches/mdx.rs` is a bench-only, zero-allocation probe. For every physical
line it:

1. skips horizontal indentation plus repeated `>` and list-marker prefixes;
2. checks whether the remaining logical line starts with `<`;
3. invokes the existing `parse_jsx_tag`; and
4. accepts it only when the tag owns the rest of the line.

It deliberately does **not** track list continuation indentation or container
state across lines, recognise multiline tags/expressions/ESM, alter output, or
claim compatibility. It is therefore an inexpensive screening probe, not a
candidate implementation. A correct implementation has additional state and
semantic work, though a fused or selectively invoked design could avoid part of
the separate-pass cost measured here.

The benchmark compares `mdx::render` with the same renderer after that probe
has run. `parse_events` is reported separately because it returns a semantic
event stream, not rendered HTML.

## Method

- `cargo bench --bench mdx --features mdx`
- 80 samples, 3-second warm-up, 5-second measurement window
- Three alternating render/probe rounds; all figures below are medians of each
  round's medians
- `plain_markdown`: existing 20 KiB CommonMark fixture
- `root_flow`: repeated root-level `<Callout>` blocks containing headings and
  lists
- `container_flow`: the equivalent `<Callout>` blocks under `>` prefixes

Environment: macOS 26.5.2 on `aarch64-apple-darwin`, Rust
`1.95.0-nightly (842bd5be2, 2026-01-29)`, bench profile (fat LTO,
one codegen unit), commit `b2bf788`. This is a development-machine screening
result, not a published cross-machine performance claim.

## Results

| Corpus | `render` median | `render` + probe median | Added time, three rounds |
|---|---:|---:|---:|
| Plain Markdown | 107.6–112.1 µs | 119.6–123.1 µs | **+9.8–12.2%** (median +11.1%) |
| Root-level MDX | 709.6–732.9 µs | 724.0–755.8 µs | **+2.0–4.2%** (median +3.1%) |
| Container-flow MDX | 210.5–214.2 µs | 239.9–242.3 µs | **+13.1–15.0%** (median +13.7%) |

For context, the already opt-in `parse_events` path took approximately 122 µs
for `plain_markdown` and 257–258 µs for `container_flow`. Those figures are
not an HTML-rendering comparison, but reinforce that richer MDX semantics should
be an explicit costed path rather than silently replacing `render`.

## Result

The hypothesis that a universal extra scan would be negligible is rejected.
Even this incomplete, line-local probe costs about 10–14% for the two documents
where callers most expect a cheap Markdown/MDX render. A fused or selective
implementation could do better, but its cost cannot be inferred from this probe
alone.

The data does **not** say that container-flow compatibility is infeasible. It
says that it should not be added as an unconditional second pass. ADR-0009
selects a single correct MDX parsing model behind the existing opt-in `mdx`
feature/API boundary, so a production implementation needs to be fused into a
proven block-aware MDX path or selectively invoked from strong candidates, then
benchmarked against this baseline with official-MDX conformance fixtures.

## Follow-up gate

Before changing public MDX rendering semantics:

1. build a correct prototype that handles nested blockquotes, ordered and
   unordered list continuation lines, multiline tags/expressions, and recovery;
2. validate those fixtures against `@mdx-js/mdx`;
3. compare its end-to-end output and throughput with the fast renderer; and
4. confirm that normal Markdown benchmarks remain unchanged and publish the
   measured cost within the opt-in MDX path.
