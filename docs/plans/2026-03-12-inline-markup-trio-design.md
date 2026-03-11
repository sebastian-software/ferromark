# Inline Markup Trio Design

**Date:** 2026-03-12
**Status:** Approved

## Summary

Implement `highlight`, `superscript`, and `subscript` as separate optional inline extensions in one PR, while tightening strikethrough to `~~text~~` only.

## Goals

- Add `^text^` superscript support behind its own option.
- Add `~text~` subscript support behind its own option.
- Keep `==text==` highlight as its own option.
- Eliminate the long-term syntax conflict between subscript and single-tilde strikethrough.
- Keep disabled-path performance effectively neutral.
- Make the behavioral change explicit in docs and release notes.

## Non-goals

- No aggregate preset flag in this change.
- No attempt to remain fully compatible with GFM single-tilde strikethrough.
- No iA Presenter import mode or syntax aliasing.

## Chosen syntax

- `~~text~~` -> `<del>`
- `~text~` -> `<sub>`
- `^text^` -> `<sup>`
- `==text==` -> `<mark>`

## Why this design

This matches the cleanest ecosystem-wide trio used by Pandoc and common Markdown extension stacks. It avoids a permanent ambiguity around `~...~` and keeps each extension independently measurable and debuggable.

The main tradeoff is a deliberate divergence from GFM's allowance for single-tilde strikethrough. We accept that tradeoff because ferromark is still early and the combined syntax system is stronger and easier to explain.

## API shape

Add two new booleans to `Options`:

- `superscript: bool`
- `subscript: bool`

Keep:

- `strikethrough: bool`
- `highlight: bool`

No preset helper is added in this change.

## Parsing model

- Keep the current three-phase inline architecture.
- Resolve code spans and math before these extensions.
- Resolve links before emphasis-like formatting where required by existing precedence.
- Treat `superscript`, `subscript`, `strikethrough`, and `highlight` as separate delimiter grammars.
- Keep default-path scanning specialized so disabled features do not add meaningful cost.

## Documentation scope

- ADR for the syntax choice and GFM divergence
- README updates
- changelog note
- PR description note
- feature examples and non-examples in tests

## Validation

- targeted parser tests for each syntax
- conflict tests around `~text~`, `~~text~~`, `^text^`, and nesting
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --check`
- branch vs `main` benchmark spot-checks for the disabled path
