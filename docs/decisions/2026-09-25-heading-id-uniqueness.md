# Unique heading IDs

## Context

The renderer and Node metadata visitor each assigned heading IDs separately.
Both counted repeats of a base slug, but neither registered suffixed IDs or
deduplicated explicit IDs. A sequence such as `a`, `a`, `a-1` therefore emitted
`a`, `a-1`, `a-1`. Heading IDs could also collide with the IDs used by
footnotes.

## Decisions

- Use one `HeadingIdPlanner` in the Rust renderer, inline TOC collector, and
  Node heading metadata visitor. A heading's requested ID is its explicit
  `{#id}` value when present, otherwise its slugified text.
- The first heading to request an available ID keeps it. Every later collision
  receives the next available `-N` suffix, starting at `-1`. The planner checks
  every emitted ID, so a suffix already claimed by another heading is skipped.
- Do not pre-scan explicit heading IDs. Headings are planned in render order:
  an earlier generated heading may claim a value that a later explicit heading
  requests, and the later heading receives a suffix. This keeps the order
  deterministic without a separate allocation-heavy ID prepass.
- Reserve the IDs a document's footnotes will emit before planning headings.
  A heading that requests one of those IDs receives a suffix. Legacy footnotes
  reserve `fn-*` targets and `fnref-*` reference IDs; semantic footnotes
  reserve their slug-based IDs and reference occurrences.
- A renderer keeps the planner across committed fragments, snapshots it around
  provisional fragments, and clears it on reset. Full rendering, hooked
  rendering, the inline TOC, and Node metadata use the same planning rules.
- The planner works on unprefixed heading IDs. If a future option adds an ID
  prefix, it must apply the prefix to the planned ID and every reference to it.

## Consequences

Non-colliding documents keep their existing output. Documents with duplicate
heading IDs, suffix collisions, explicit-ID repeats, or collisions with
footnote IDs intentionally receive unique heading IDs. Node metadata and TOC
links now report the same values as the rendered headings.
