# Opt-in writing syntax and reference policy

## Scope

The user requested measured implementations of `==marked text==`, Pandoc-style
`^[inline notes]`, and a reference-link toggle. Keep the writing extensions off
by default and investigate material performance regressions before retaining the
change. Indented-code toggles, numeric table widths, MDX compilation, and old
event compatibility APIs are outside this change.

## Design

- Add `Highlight` to the arena AST, the existing delimiter stack, visitors,
  span remapping, text extraction, autolinking, and both HTML rendering paths.
  Exactly two equals signs form a delimiter; ordinary block parsing retains
  precedence. Extend the existing SIMD nibble tables and scalar oracle together.
- Add inline notes without a second public footnote representation. The parser
  first emits internal unlabeled definition placeholders, then lifts surviving
  notes into ordinary references and document-end definitions. Assign identifiers
  only after speculative link parsing and source remapping. Skip explicit
  footnote labels collected from the final AST, including nested definitions,
  and reuse existing legacy/semantic HTML policies.
- A root-owned arena `Cell<bool>` is shared with container sub-parsers only when
  inline notes are enabled. It records whether a note was parsed, avoiding a
  document-wide presence scan and unnecessary lowering walks. Speculative parses
  can set the flag, but only surviving AST nodes become definitions. Empty input
  and disabled notes allocate no flag. The flag does not outlive the arena.
- Preserve isolated equals signs and non-note carets in the current text run.
  They must not allocate extra AST nodes just because an extension is enabled.
  Skip whitespace-surrounded `==` runs that cannot form a delimiter. Consume
  unclosed `^[` literals without re-entering the link parser. Superscript retains
  its caret handling when explicitly selected.
- `allow_link_refs` defaults on. Disabling it skips definition collection and
  leaves definitions visible as ordinary Markdown. Inline links/images, wiki
  links, and footnotes remain independently configurable. With reference
  footnotes off, the complete fused definition prepass is skipped.
- Expose `highlight`, `inlineFootnotes`, and `allowLinkRefs` consistently in the
  Node wrapper, native binding, declarations, reusable renderer, and hook paths.

See the [syntax contract](../optional-writing.md) and
[performance report](../reports/2026-09-15-optional-writing/README.md).

## Verification and acceptance

Use the unchanged pre-feature integration commit as the baseline. Compare exact
HTML and debug AST with both writing extensions off; separately measure unused
options, active dense syntax, sparse syntax, and literal marker decoys. Preserve
per-workload results and repeat selected comparisons. Active syntax and disabled
reference resolution change output, so their timing ratios are not equivalent-
output optimization claims. Measurements on one ARM64 workstation do not establish
performance on other processors.

Tests cover delimiter boundaries, literal fallbacks, nested formatting, existing
footnote policies, identifier collisions, speculative links, images, nested
containers, source maps, hooks, renderer reuse, and the unchanged specification
corpora. Keep the existing 90% line-coverage gate and Node package checks.
