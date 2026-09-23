# Heading level offset

## Decision

The HTML renderer supports a signed `heading_level_offset`. It shifts every
Markdown heading level by the configured amount and clamps the result to the
valid HTML range, `h1` through `h6`. The default offset is zero, preserving
existing output.

The public `map_heading_level` function is the single mapping used by ordinary
HTML rendering, render hooks, Node heading metadata, and Rust incremental
fragments. Heading IDs continue to derive from the original text, so existing
fragment targets remain stable. The offset changes the output level and the
metadata level, not the parsed AST.

Node exposes this setting as `headingOffset`. A `Renderer` stores it with its
fixed options, so arena resets and later render calls do not discard it.
Node accepts only finite signed 32-bit integer values; the core mapping accepts
that full range and clamps the resulting heading level.

## Rationale

Documentation systems often embed one Markdown document beneath another
heading hierarchy. Shifting levels at render time lets the caller preserve a
single source document without rewriting Markdown. Keeping the mapping in the
renderer avoids changing parser semantics and ensures all output and metadata
paths agree.

Clamping prevents invalid `h0` or `h7` elements and gives extreme values a
predictable result. Leaving IDs untouched keeps inbound fragment links stable
when the same document is rendered at a different depth.

## Validation

Focused Rust and Node tests cover positive and negative offsets, both clamp
boundaries, i32 overflow edges, nested headings, invalid Node configuration,
hook rendering, Node metadata, reusable renderers, and incremental fragments.
The Rust formatting, full workspace tests, and Clippy checks pass. The Node
addon builds, all 46 package tests and panic-unwind verification pass, and the
typecheck, formatter, linter, dependency audit, pack check, and clean-install
smoke check pass.
