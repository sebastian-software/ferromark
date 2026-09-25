# Read-only document outline

- Status: Accepted
- Date: 2026-09-25

## Context

Consumers need a structured view of headings for navigation and later table of
contents integrations. The renderer's heading text, effective levels, IDs and
source spans already define the behavior such a view must describe. A document
outline can expose that information without changing rendering or inserting
generated content into the AST.

An in-body table of contents is a separate concern: it transforms or renders
content at a marker and depends on the transform API being developed in #401.
That adapter is intentionally outside this core API.

## Decision

Add `Document::outline(&OutlineOptions) -> Vec<OutlineEntry>` as an explicit,
read-only operation on one parsed document. Options default to IDs on, legacy
footnotes, no heading prefix, no level offset and the effective level range
`1..=6`. Callers can set those ID modes, prefix, offset and inclusive level
filter. Each entry owns its display text and resolved optional heading ID, and
carries the heading's original `Span` as source provenance. The span refers to
the heading AST node's source range, including its Markdown syntax; it is not a
synthetic range around display text. The effective level uses the renderer's
`map_heading_level` rule. Display text uses the shared heading text collector.
The operation does not allocate AST nodes, rewrite nodes or add annotations.

Prefix configuration returns the existing `InvalidHeadingIdPrefix` error when
the prefix contains bytes outside ASCII letters, digits, `_` and `-`. An
invalid level range simply includes no levels. There are no fallible parse or
render steps in outline computation; with heading IDs disabled, each entry's
`id` is `None`.

Callers run any AST transformations first, then request the outline and render
that same resulting `Document`. This makes text, levels, IDs and spans describe
the tree that will be rendered, without making the outline operation itself a
transform or changing the tree.

Outline ID planning uses `HeadingIdPlanner` and the renderer's ID rules. Heading
IDs are claimed in AST order, before visiting inline heading children. Legacy
footnote target/reference claims and semantic footnote claims use the same
planner in the renderer's AST visitation order. The `semantic_footnotes`,
`heading_ids`, and heading-prefix options must match the renderer settings when
callers need resolved IDs to match HTML. Footnote IDs remain unprefixed. A
level filter applies after offset and clamping; filtered headings are omitted
from the result but still claim IDs, preserving later heading IDs.

The outline is one-shot for the exact `Document` passed to the call. It carries
no planner state across committed or provisional fragments and makes no
streaming guarantee. IDs, filtering, and source spans describe only that tree;
callers request another outline after obtaining a complete or updated
document. The API is not invoked by parsing or rendering, so existing calls
incur no additional traversal or allocation and the renderer's default output
is unchanged.

The outline core does not recognize a TOC marker or produce HTML. A later
in-body TOC adapter is a separate dependent change after #401 establishes the
transform contract.

## Consequences

Callers can build navigation from the same heading text, level, IDs and source
locations that the renderer uses. The returned vector and strings are owned by
the caller and are independent of the parser arena after construction.
Computing the outline is explicit work, and generating IDs requires walking
heading and footnote nodes to preserve the renderer's shared namespace.

Because the result is a snapshot of one document, callers that render a
transformed tree should request the outline from that same transformed tree.
The API deliberately leaves marker placement, HTML shape and renderer defaults
to the future adapter decision.

## Validation

Integration tests compare outline heading IDs and effective levels with the
HTML renderer in both footnote modes, including collisions, prefixes, explicit
duplicate IDs and filtered headings. Additional cases cover Unicode text,
container traversal, spans, level clamping and disabled heading IDs. Existing
parser and renderer tests continue to validate default output.
