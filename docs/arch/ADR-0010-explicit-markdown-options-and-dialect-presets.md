# ADR-0010: Explicit Markdown Options and Dialect Presets

**Status:** Accepted
**Date:** 2026-07-24

## Context

Ferromark introduced `Profile::Essentials`, `Profile::Extended`, and
`Profile::Full` as convenience layers over the individual `Options` fields.
The names looked like performance tiers even though they selected syntax, and
their feature groupings were project-specific rather than recognised Markdown
dialects. Because the API is still alpha and has no known users, retaining this
extra abstraction has more migration and explanation cost than removing it.

Individual options serve more than performance. Some skip measurable work, but
others select document semantics, output contracts, or security boundaries.
For example, front matter changes how an opening `---` is interpreted even when
its disabled fast path is nearly free.

## Decision

`Profile` is removed. Individual `Options` fields remain the source of truth
when a feature changes syntax, output, trust handling, or material parser work.

Ferromark provides three syntax constructors:

- `Options::minimal()` selects the smallest supported Markdown surface;
- `Options::commonmark()` selects CommonMark syntax; and
- `Options::gfm()` selects CommonMark plus the five GFM extensions.

These are dialect or syntax presets, not performance promises.
`Options::default()` retains its existing feature mix for compatibility and is
not an alias for one of the constructors.

HTML remains explicit and orthogonal:

- `allow_html` controls inline and block HTML parsing;
- `RenderPolicy` controls whether raw HTML and arbitrary URL schemes are
  trusted during rendering; and
- `disallowed_raw_html` enables the GFM tag filter in trusted mode.

Every constructor selects `RenderPolicy::Untrusted`. Trusted rendering always
requires an explicit caller choice.

## Consequences

- Callers can begin with a recognised syntax contract or a minimal surface,
  then override only the options relevant to their document.
- Adding a future option requires assigning it deliberately in all three
  constructors, which makes dialect drift visible during compilation and
  review.
- Features with negligible disabled-path cost remain configurable when
  enabling them changes document meaning or output.
- Alpha callers using `Profile` must migrate directly to a constructor or an
  explicit `Options` value; no deprecation layer is kept.
- Benchmarks name exact option configurations instead of implying stable speed
  from `Essentials`, `Extended`, or `Full`.

## References

- ADR-0004: GFM Extensions — Opt-in Architecture
- Historical profile design:
  `docs/plans/2026-07-11-markdown-profiles-parity-design.md`
