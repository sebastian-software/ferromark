# Document ID uniqueness

## Decision

Heading IDs, footnote target IDs, and footnote reference IDs share one emitted
ID registry. Each claim follows AST document order. The first claim keeps its
requested ID; a later collision receives the first available `-N` suffix.
Repeated footnote references keep their established `-2`, `-3` names when
available and use the shared registry if one of those names is already taken.

The heading ID prefix is part of the heading's claim. Footnote IDs retain their
existing unprefixed `fn-*` and `fnref-*` forms. This also detects a collision
when a configured heading prefix makes a heading's emitted ID equal a footnote
ID.

Legacy footnote targets are claimed when the first reference or definition is
visited. A definition also reserves the ID for its backlink after rendering its
body, even if the corresponding reference appears later. Semantic footnote
targets are claimed when their reference or definition first appears, even
though the target markup is emitted at the end of rendering. Renderer output
and Node heading metadata walk the same AST order and use the same claims.
Committed fragments retain the registry; provisional fragments restore its
prior state.

## Rationale

Reserving every footnote ID before rendering headings would make the output of
a streaming renderer depend on content it has not received yet. Claiming IDs
as the AST is visited gives full-document rendering and incremental rendering
the same result. If a heading appears before a colliding footnote, the heading
keeps its ID and the footnote target or reference receives a suffix. If the
footnote appears first, the later heading receives the suffix.

## Compatibility

Documents without colliding emitted IDs keep their existing HTML. On a
collision, the generated suffix is reflected consistently in IDs, permalinks,
footnote references and backlinks. Authored fragment links are not rewritten.
