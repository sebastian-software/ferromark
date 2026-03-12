# ADR-0008: Inline Markup Syntax Alignment for Highlight, Superscript, and Subscript

**Status:** Accepted
**Date:** 2026-03-12

## Context

ferromark already supports opt-in `==text==` highlight syntax and currently treats both `~text~` and `~~text~~` as strikethrough. We want to add opt-in superscript and subscript without carrying an ambiguous tilde grammar long-term.

The main conflict is subscript: the most common cross-parser syntax is `~text~`, but that collides directly with single-tilde strikethrough. GitHub Flavored Markdown permits both one-tilde and two-tilde strikethrough, while Pandoc and several extension ecosystems use a more internally consistent trio:

- `~~text~~` for strikethrough
- `~text~` for subscript
- `^text^` for superscript

## Decision

ferromark will align these inline syntaxes as follows:

| Syntax | Meaning | Option | Default |
|---|---|---|---|
| `~~text~~` | Strikethrough | `strikethrough` | `true` |
| `~text~` | Subscript | `subscript` | `false` |
| `^text^` | Superscript | `superscript` | `false` |
| `==text==` | Highlight | `highlight` | `false` |

### Compatibility choice

ferromark deliberately drops single-tilde strikethrough support. This is a conscious divergence from the GFM specification in favor of a cleaner combined inline grammar and better interoperability with parsers and extension stacks that support all three constructs together.

### Options model

Each feature remains independently controllable in `Options`. No aggregate "extended CommonMark" flag is introduced in this change.

### Performance model

Optional inline extensions should remain effectively free when disabled. The parser should preserve specialized fast paths for the default configuration rather than threading additional runtime checks through the hottest scan loops.

## Consequences

- `~text~` is no longer strikethrough in ferromark.
- Existing users who relied on single-tilde strikethrough must switch to `~~text~~`.
- Subscript can be added without syntax ambiguity.
- Superscript and subscript stay opt-in and benchmarkable independently.
- Documentation must explicitly call out the GFM divergence so the behavior is transparent.

## Documentation requirements

- Update README feature lists, option examples, and syntax examples.
- Add changelog notes describing the single-tilde strikethrough change.
- Add tests that show the new literal behavior for `~text~` when `subscript` is disabled.
- Note the rationale and external references in the PR description.

## References

- GFM Spec
  https://github.github.io/gfm/
- Pandoc Manual
  https://pandoc.org/MANUAL.html
- markdown-it ecosystem plugins
  https://mdit-plugins.github.io/sub.html
  https://github.com/markdown-it/markdown-it-sup
- PyMdown Extensions
  https://facelessuser.github.io/pymdown-extensions/extensions/tilde/
  https://facelessuser.github.io/pymdown-extensions/extensions/caret/
