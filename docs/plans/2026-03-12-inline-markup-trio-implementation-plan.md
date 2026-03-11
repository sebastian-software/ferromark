# Inline Markup Trio Implementation Plan

**Date:** 2026-03-12

## Scope

One PR covering:

- `superscript`
- `subscript`
- strikethrough tightening to `~~...~~`
- documentation and benchmarks

## Steps

1. Extend `Options`

- Add `superscript: bool`
- Add `subscript: bool`
- Update defaults and examples

2. Update inline events and renderer

- Add `SuperscriptStart` / `SuperscriptEnd`
- Add `SubscriptStart` / `SubscriptEnd`
- Render to `<sup>` and `<sub>`

3. Add resolvers

- Add a superscript resolver for `^...^`
- Add a subscript resolver for `~...~`
- Tighten strikethrough resolver to `~~...~~` only

4. Preserve fast paths

- Keep specialized mark collection/scanner paths for the default configuration
- Avoid extra runtime conditionals in the default hot path

5. Add tests

- new superscript tests
- new subscript tests
- updated strikethrough tests
- precedence and disabled-mode tests

6. Update documentation

- README
- CHANGELOG
- ADR references in PR description

7. Benchmark

- compare disabled-path throughput against `main`
- run at least one tilde/caret-heavy synthetic workload

## Risk focus

- precedence interactions with emphasis, code spans, math, and links
- accidental regressions in existing strikethrough behavior
- hidden overhead in the disabled default path
