# Caller-placed table of contents

- Status: Accepted
- Date: 2026-09-26
- Related: [read-only outline decision](2026-09-25-read-only-document-outline.md), #396

## Context

Ferromark already exposes a read-only outline from one parsed document. The
removed renderer-owned `[[toc]]` behavior combined marker recognition, content
generation and placement inside Markdown rendering. Consumers still need an
optional in-body table of contents, but the parser and renderer should keep
their current syntax and default output.

The AST transform crate now provides the extension boundary. Its provenance
contract uses `Span::empty()` for inserted content and does not distinguish an
inserted node from a genuine empty source range.

## Decision

`ferromark-transforms` exposes `build_table_of_contents`, which builds a nested
unordered-list AST node from an outline and an arena allocator. It does not
find or replace a Markdown marker and does not mutate the document. The caller
inserts the returned node wherever it wants it in the AST.

The helper uses each outline entry's display text and resolved ID, preserves
source order, and nests an entry under the nearest preceding entry with a
shallower effective heading level. Missing heading levels do not create empty
list levels. Every generated AST span is `Span::empty()`. An empty outline
returns no node; an entry without an ID returns a typed error before AST nodes
are allocated.

Callers run transforms first, compute the outline from the document that will
be rendered, and use ID settings that match the renderer. The helper covers one
complete document only. It carries no heading-ID state between committed or
provisional fragments. Node heading metadata remains unchanged and continues
to use the shared `HeadingIdPlanner`.

## Consequences

Rust callers can replace marker-based placement with an explicit API call and
choose the insertion point. Default parsing and rendering do no outline or TOC
work. Generated links are ordinary AST links, so the renderer applies its
normal URL and HTML escaping policy.

The helper copies outline strings into the caller's arena, and its generated
spans cannot be distinguished from other empty spans. Callers must not build
the outline before transformations or use renderer ID options that differ
from the outline options.

## Validation

Integration tests render a caller-inserted TOC, verify nested links use the
outline IDs and check empty-outline and missing-ID behavior. The API guide
shows how to migrate from `[[toc]]` to caller-controlled placement.
