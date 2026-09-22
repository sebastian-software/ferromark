# Remove renderer-owned inline TOC

Status: accepted
Date: 2026-09-22
Related: [#395](https://github.com/sebastian-software/ferromark/issues/395), [#396](https://github.com/sebastian-software/ferromark/issues/396)

## Context

Ferromark v2 inherited a renderer convenience that replaced a standalone
`[[toc]]` paragraph with an HTML navigation list. The behavior is not part of
CommonMark or GFM, consumes source syntax that is otherwise ordinary Markdown,
and couples document analysis to HTML rendering. It also duplicates work that
an application or an AST extension can perform with more control over the
resulting structure.

## Decision

Remove inline TOC substitution and the renderer options that controlled it:
`inline_toc` and `toc_max_depth`. The renderer treats `[[toc]]` as ordinary
Markdown text unless a future extension explicitly assigns it another meaning.
Heading IDs remain an independent renderer feature.

The replacement direction is an extension operating on the native Markdown
AST. That extension can expose a document outline and, when desired, provide
an application-owned TOC projection. It is tracked separately in #396.

## Consequences

- Existing v2 callers that rendered `[[toc]]` receive literal paragraph text
  and must opt into an extension or build navigation from heading data.
- The public renderer options are smaller and no longer require a
  document-wide TOC scan before rendering.
- The change is delivered directly in the v2.1 follow-up. No deprecation shim
  is warranted because v2.0 has no meaningful installed base yet.
- Migration guidance, profile documentation, benchmarks, and regression tests
  record the new behavior.
