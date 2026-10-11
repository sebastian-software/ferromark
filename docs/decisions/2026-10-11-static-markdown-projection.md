# Static Markdown projection from prepared source

Issue [#523](https://github.com/sebastian-software/ferromark/issues/523) needs static
documentation exports without another MDX scanner or an HTML-to-Markdown round
trip. The first implementation applies omissions to original source using the
existing parsed tree and attribute spans. It preserves ordinary Markdown and
code instead of introducing a general serializer.

`Document::project_markdown` owns the core policy. Node's immutable prepared
handle exposes `projectMarkdown`; it reads its owned source and tree and does
not rerun preparation, native passes, rendering, or highlighting. Both prepared
entry points use the same method. JSX title matching shares the renderer's
static heading collector to prevent separate omission rules.

Paired wrappers and fragments retain static source children, while every tag
and attribute is discarded. Child expressions, ESM, self-closing elements,
intrinsic raw-text elements, and raw HTML are omitted with source diagnostics.
Wrapper diagnostics disclose the loss of component-specific semantics. Only
boundaries inside already parsed JSX nodes are verified, using attribute spans
to skip quotes, expressions, and spreads; uncertain bounds omit the complete
element. Existing public AST struct literals remain compatible.

Raw HTML body omissions are planned from parsed HTML spans sorted by original
source position. This keeps closing tags aligned with authored text when table
captions or lowered inline footnotes have a different AST display order. The
existing tag scanner reads only these original HTML slices; source gaps and
code are never scanned for tags.

Every diagnostic uses an original half-open UTF-8 byte span. Removal preserves
line endings and container prefixes and separates adjacent content where
necessary. Authored indentation is retained, including code; this can differ
from JSX's normalized child indentation. Projection does not serialize native
transform results or promise runtime component output. Front matter and the
first top-level matching title are omitted only by explicit request.

The [projection guide](../markdown-projection.md) owns the detailed behavior and
limits. Callers continue to own site navigation, document metadata, joining
documents, and `llms.txt` formatting. Existing rendering and conformance output
are unchanged.
