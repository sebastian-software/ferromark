# ADR-0011: Event-Based Presentation Adapter

**Status:** Proposed
**Date:** 2026-07-25

## Context

An iA Presenter-like consumer needs to split one MDX document into slides and
separate audience content from moderator-only notes. Ferromark already exposes
an opt-in, source-ranged MDX event stream. Explicit flow components such as
`<Moderator>...</Moderator>` can identify notes without adding
presentation-specific punctuation to Markdown.

Two gaps prevent a clean downstream adapter:

- `BlockEvent::ThematicBreak` did not carry the source range needed to retain
  exact slide boundaries.
- The current renderer owns document-wide link references, footnote
  definitions, heading identifiers, and block state. Rendering arbitrary
  subsets of a flat event stream as if they were independent documents would
  expose an API whose balancing and identifier guarantees are unclear.

Reparsing source slices for every slide would avoid the second problem, but
would duplicate parsing work, lose the original container context, and make
slide splitting dependent on raw `---` scanning. A presentation-specific
parser mode would duplicate MDX grammar and add another compatibility surface.

## Decision

Ferromark will treat presentations as an opt-in adapter over the semantic MDX
event stream, not as a parser mode and not as new core Markdown syntax.

The reusable core event now records a thematic break as
`BlockEvent::ThematicBreak(Range)`. The range covers the marker run and
intervening/trailing horizontal whitespace, excluding container indentation
and the line ending. The MDX event-stream contract is bumped to version 2
because this changes a public event variant.

The proposed adapter API is:

```rust
pub struct PresentationOptions<'a> {
    pub moderator_components: &'a [&'a str],
}

pub struct Presentation {
    pub stream: MdxEventStream,
    pub shared_events: EventSelection,
    pub slides: Vec<Slide>,
}

pub struct Slide {
    pub source_range: Range,
    pub opening_boundary: Option<Range>,
    pub audience: EventSelection,
    pub moderator: EventSelection,
}

pub struct EventSelection {
    pub event_ranges: Vec<std::ops::Range<usize>>,
}

pub fn parse_presentation(
    input: &str,
    options: &PresentationOptions<'_>,
) -> Result<Presentation, Vec<PresentationDiagnostic>>;
```

Selections contain indices into the one owned `MdxEventStream`; they do not
clone source text or semantic events. Front matter and ESM are shared document
events rather than slide content. Component names are explicit configuration;
Ferromark does not assign special meaning to a generic `<Notes>` component
unless the caller requests it.

A generic public event-substream-to-HTML renderer is deferred. It should only
be added after its link, footnote, heading-ID, JSX-context, and balanced-event
contracts can be stated independently of presentations. The presentation
adapter should first expose structured selections; an eager HTML convenience
layer can then be built on the same renderer contract.

## Slide and channel rules

- Only a thematic-break event at Markdown container depth zero and JSX flow
  depth zero splits a slide.
- A boundary at the beginning or end creates an empty first or last slide.
  Consecutive boundaries preserve the empty slide between them.
- A Setext underline is a heading event, never a slide boundary.
- Thematic breaks in block quotes, lists, moderator content, or any other flow
  component remain content of the current slide.
- A configured moderator component must be a balanced root-level flow JSX
  container. Its wrapper events are structural and omitted from both channel
  selections; all nested Markdown and JSX content belongs to the moderator
  selection.
- Other JSX containers remain in their current channel. A moderator component
  nested inside a Markdown container is diagnosed rather than silently
  changing container balance.
- Malformed or mismatched flow JSX uses the strict MDX diagnostics and produces
  no partial presentation.

These rules deliberately do not use `//` comments as moderator notes. Line
comments are source annotations with no rendered output and a separate option.

## Footnotes

Footnotes are slide-local at rendering time. Each slide numbers references in
first-reference order, emits only definitions referenced by that slide, and
prefixes generated IDs with the slide ordinal. A definition may therefore be
rendered on more than one slide without cross-slide links.

Implementing this without reparsing requires `MdxEventStream` to retain the
footnote definition store just as it already retains link-reference
definitions. The adapter must not ship eager HTML until that prerequisite is
implemented and tested. Documents without footnotes are unaffected.

## Considered options

### Add presentation syntax to the block parser

Rejected. It would make a presentation concern part of every Markdown parse
and compete with CommonMark meanings for indentation and thematic breaks.

### Reparse each slide source range

Rejected as the primary design. It repeats parsing, complicates references
across boundaries, and cannot reliably recover JSX/container context from an
arbitrary slice.

### Render arbitrary cloned event vectors

Deferred. A vector alone does not carry the stores and state needed for
correct links, footnotes, identifiers, or balanced containers.

### Return event selections over one semantic stream

Chosen. It preserves one parse, exact source identity, and document-level
semantic stores while keeping presentation policy outside the hot Markdown
path.

## Consequences

- Existing `to_html*` and MDX rendering entry points keep their behavior and do
  not run presentation analysis.
- The ordinary block parser stores one range in thematic-break events but adds
  no pass, allocation, or source scan.
- Consumers can identify exact semantic slide boundaries immediately.
- Audience/moderator HTML remains follow-up work rather than being exposed with
  underspecified renderer semantics.
- Implementing slide-local footnotes requires the MDX semantic stream to retain
  footnote definitions.
- Accepting this proposal will require tests for all boundary and channel rules
  before the adapter becomes stable.

## Validation and review triggers

- On 2026-07-25, the `parsing/thematic_breaks` fixture (500 boundaries,
  approximately 20.5 KB) measured 103.25 µs on `main` and 103.67 µs with
  source-ranged events. Criterion reported no performance change (95%
  confidence interval: -0.04% to +0.50%).
- Benchmark normal Markdown rendering with a thematic-break-heavy fixture
  before accepting the event-shape change; no material regression is allowed.
- Benchmark the adapter separately from `mdx::parse_events`.
- Revisit the selection model if a renderer cannot preserve balanced
  containers without copying events.
- Revisit shared events if ESM or front matter gains slide-local semantics.

## References

- Issue #5: event-based presentation output
- ADR-0001: Core Architecture — Streaming, No AST
- ADR-0009: MDX Compatibility and Performance Boundaries
- iA Presenter Markdown Guide: <https://ia.net/presenter/support/basics/markdown>
