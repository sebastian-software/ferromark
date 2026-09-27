# Opt-in guillemet digraphs

- Status: Accepted
- Date: 2026-09-27
- Related: #400, #403, #464

## Context

Authors sometimes type French-style quotation marks as `<<Bonjour>>` because
`«` and `»` are inconvenient to enter. Without special handling, the second
`<` starts a raw HTML tag such as `<Bonjour>`, so Markdown parsing can discard
the quoted word before a post-parse transform can inspect it. The owner selected
the `remark-fix-guillemets` use case in #400 and asked that it remain an explicit
syntax opt-in.

## Decision

Add `ParserOptions::guillemet_digraphs` and the Node.js `guillemetDigraphs`
option. Both default to `false` and remain off in every Rust preset. When
enabled, the inline parser emits a run of at least two opening angle brackets as
text before attempting raw HTML or angle-autolink parsing. The enclosed content
continues through the ordinary inline parser, so emphasis and Markdown links
still work. Closing `>` characters remain ordinary text. Block parsing still
owns line-start blockquote markers.

The parser option only preserves source punctuation. It does not emit locale
quotes itself. `TypographyOptions::with_guillemet_digraphs(true)` explicitly
enables conversion in the optional typography pass; the Node binding forwards
`guillemetDigraphs` to that pass. When enabled, the pass recognizes balanced,
exact `<<` and `>>` marker pairs and maps them through the selected language's
primary quote rules, including French narrow no-break spaces and Danish
`»…«` direction. Unmatched markers and runs longer than two remain literal.
Pairs can cross ordinary inline formatting, but protected nodes, raw HTML, and
block boundaries end pairing. Escaped and entity-authored angle brackets stay
literal. Where a renderer auto-links bare URLs, a URL immediately adjacent to
a marker stays outside quote conversion so a generated quote cannot become
part of its destination. Authors can use an inline Markdown link inside a pair
when they need explicit link markup.

Code spans, code blocks, math, and raw HTML retain their existing parsing and
rendering. In MDX mode, an enabled doubled opener takes precedence over JSX at
that position, so `<<Foo />>` is treated as text for typography. Unmatched
single markers remain literal. Paired markers with ASCII padding on both sides
and alphanumeric operands outside the pair, such as `a << b and c >> d`, remain
literal as ambiguous shift expressions. Standalone French `<< Bonjour >>`
absorbs its padding before locale spacing is applied.
The portable alternative is to type ordinary quotes and select the desired
typography language explicitly.

## Compatibility

With the option unset, CommonMark parsing, conformance output, snapshots, and
benchmark inputs remain unchanged. With it enabled, only doubled opening angle
brackets bypass raw HTML and autolink recognition. The post-parse typography
pass performs no language detection and is still opt-in.

Adding `ParserOptions::guillemet_digraphs` changes the exhaustive Rust options
struct and therefore requires an API-breaking release under the project’s
semver policy. The Node.js option is additive.

## Performance with the option off

A local before/after comparison used the same 57-document corpus and the same
worker in the base and PR builds. The worker applied explicit CommonMark and
GFM profile flags on both revisions because its older preset aliases no longer
matched this branch's API. All compared HTML and AST outputs matched exactly.
Three rounds used five alternating pairs of 40 ms windows for fresh and reused
renderers:

| Workload | Base median | PR median | Change |
| --- | ---: | ---: | ---: |
| Fresh render | 1.048 ms | 1.044 ms | −0.405% |
| Reused render | 1.005 ms | 1.007 ms | +0.267% |

The fresh per-round differences were +0.687%, −0.618%, and −0.405%; reused
differences were +0.319%, +0.003%, and +0.267%. These small changes are within
local run variation. In two focused Criterion typography workloads with the
new gate off, quote text measured 111.32 µs (−10.18% against its saved base
sample) and angle text 78.912 µs (−17.35%). The focused results do not replace
the broader corpus comparison.

## Validation

Rust and Node tests cover the option default, destructive raw-HTML examples,
inline Markdown, unmatched and longer marker runs, escaped markers, protected
syntax, link bracket scanning, blockquote parsing, URL behavior, typography
composition in French, Danish, and English, and heading metadata.
