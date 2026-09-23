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

The planner covers heading IDs. Footnote IDs keep their existing `fn-*` and
`fnref-*` rules; collisions between a heading and a footnote are a separate
namespace concern. The companion heading-prefix issue (#407) will decide how
opt-in prefixes interact with footnote IDs and authored fragment links.

## Compatibility

Only documents with colliding heading IDs change. Generated suffixes already
in use are skipped when needed, and repeated explicit IDs are suffixed after
their first occurrence. Non-colliding output remains byte-identical.
