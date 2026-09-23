# Heading ID prefixes

**Status:** Accepted for Ferromark v2

## Decision

`HtmlRenderer::try_with_heading_id_prefix` configures an optional prefix for
emitted heading IDs. It applies to generated and explicit heading IDs after
the shared heading ID planner assigns collision suffixes. Thus `a`, `a`, and
`a-1` still plan as `a`, `a-1`, and `a-1-1`; with `docs-` they emit as
`docs-a`, `docs-a-1`, and `docs-a-1-1`. Generated permalinks and Node heading
metadata use those same prefixed values. With the default empty prefix, output
is unchanged.

Prefixes accept only ASCII letters, digits, `_`, and `-`. Rust returns
`InvalidHeadingIdPrefix`; Node rejects the option with an invalid-argument
error. This keeps the value safe in both `id` and fragment attributes without
changing ID escaping for explicit IDs.

The option is configured on `HtmlRenderer` instead of adding a field to the
exhaustive `HtmlRendererOptions` struct. That preserves the v2 Rust API freeze.
Node exposes the same setting as `headingIdPrefix`.

## Boundaries

- Prefixes namespace heading IDs only. Footnote IDs keep their existing
  `fn-*` / `fnref-*` scheme; the separate collision between those IDs and
  headings remains a distinct decision.
- Authored same-document fragment links are left unchanged. A link such as
  `[install](#install)` will not be rewritten to `#docs-install`.
- Heading offsets change the emitted heading level and metadata level, not the
  ID. Applying an ID prefix is independent of the #402 level mapping.
- The #396 outline and optional TOC must read IDs from the same planner and
  apply this prefix when that API is implemented.
