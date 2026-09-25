# Heading ID planning

## Decision

Heading IDs are assigned in document order by one `HeadingIdPlanner` in the
core crate. The first heading to request an ID keeps it. Later headings request
the base plus `-1`, `-2`, and so on, skipping every ID already assigned. This
rule applies equally to generated slugs and explicit `{#id}` values. Renderer
permalinks and Node heading metadata use the exact planned value.

There is no pre-scan for later explicit IDs. If a generated heading claims an
ID before a later heading requests it explicitly, the earlier heading keeps
the ID and the explicit heading is suffixed. This keeps planning single-pass
and gives every heading a unique ID without changing the IDs in documents that
have no collisions.

The planner is public so Rust tooling can derive IDs with the same rule as the
renderer. One-shot rendering clears it for each document. Incremental rendering
retains it across committed fragments and restores it after provisional
fragments.

## Scope

The follow-up decision in
[`2026-09-25-document-id-uniqueness.md`](2026-09-25-document-id-uniqueness.md)
extends the same registry to footnote targets and references. Footnotes keep
their existing `fn-*` and `fnref-*` IDs when there is no collision; in a
collision, the ID encountered later in document order is suffixed. Heading
prefixes are applied before a heading claims its ID, while footnote IDs remain
unprefixed.

## Compatibility

Only documents with colliding heading IDs change. Generated suffixes already
in use are skipped when needed, and repeated explicit IDs are suffixed after
their first occurrence. Non-colliding output remains byte-identical.
