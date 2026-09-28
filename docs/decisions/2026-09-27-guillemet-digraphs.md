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
`»…«` direction. Unmatched markers, empty pairs, and runs longer than two
remain literal. ASCII spaces and tabs directly inside a pair are trimmed in
full before locale spacing is added. Pairs can cross ordinary inline formatting
and inline raw HTML tags; the HTML markup itself stays unchanged. Protected
content and block boundaries end pairing. Escaped and entity-authored angle
brackets stay literal.

Balanced markers always mean guillemet quotes while the option is enabled, even
when the same characters could be read as shift operators. For example,
`a << b and c >> d` is quoted text in this opt-in syntax. A lone `a << b` or
`x >> 2` remains literal because it has no matching pair. Authors who need a
balanced shift expression can use a code span or leave the option off. This is
the deliberate choice that allows ordinary spaced French prose such as
`Il a dit << Bonjour >> et il part.` to use the feature.

Where a renderer auto-links bare URLs, a pair stays literal if a URL starts
immediately after its opener or ends immediately before its closer, including
ASCII padding or punctuation that would otherwise be trimmed. The bare URL can
still become a link, and the markers stay outside its destination. Authors can
use an inline Markdown link inside a pair when they need explicit link markup.

Code spans, code blocks, math, and raw HTML retain their existing parsing and
rendering. In MDX mode, an enabled doubled opener takes precedence over JSX at
that position, so `<<Foo />>` is treated as text for typography. Unmatched
single markers remain literal. The portable alternative is to type ordinary
quotes and select the desired typography language explicitly.

## Compatibility

With the option unset, CommonMark parsing, conformance output, snapshots, and
benchmark inputs remain unchanged. The disabled typography path skips guillemet
pairing and its URL-range bookkeeping. With the parser option enabled, only
doubled opening angle brackets bypass raw HTML and autolink recognition. The
post-parse typography pass performs no language detection and is still opt-in.

Adding `ParserOptions::guillemet_digraphs` changes the exhaustive Rust options
struct and therefore requires an API-breaking release under the project’s
semver policy. The Node.js option is additive.

## Performance with the option off

A local before/after comparison used the same 57-document corpus and the same
worker in the base and PR builds. The worker applied explicit CommonMark and
GFM profile flags on both revisions. An adapter supplied the same effective
profile on the older base API, whose aggregate aliases differ from the current
option fields. All compared HTML and AST outputs matched exactly. This corpus
measured parsing and rendering, not the optional typography pass. Three rounds
used five alternating pairs of 40 ms windows for fresh and reused renderers:

| Workload | Base median | PR median | Change |
| --- | ---: | ---: | ---: |
| Fresh render | 1.048 ms | 1.044 ms | −0.405% |
| Reused render | 1.005 ms | 1.007 ms | +0.267% |

The fresh per-round differences were +0.687%, −0.618%, and −0.405%; reused
differences were +0.319%, +0.003%, and +0.267%. These small changes are within
local run variation. A separate focused Criterion comparison measured the
typography pass with guillemet conversion off on the same host and toolchain:

| Workload | Base median | PR median | Change |
| --- | ---: | ---: | ---: |
| Prose with quotes | 111.70 µs | 111.31 µs | −0.56% |
| Prose with angle text | 80.29 µs | 79.49 µs | −0.67% |

These medians came from 100 Criterion samples on an Apple Silicon arm64 host
with Rust 1.95.0. I copied the same benchmark source into a clean snapshot of
base commit `0bd64f49` and added only the Criterion bench target metadata there.
I confirmed the base and PR outputs match byte-for-byte (8,798 bytes) before
timing. Criterion classified both changes as within its configured noise
threshold; the separate x86-64 report remains hardware-specific.

## Validation

Rust and Node tests cover the option default, destructive raw-HTML examples,
inline Markdown, unmatched and longer marker runs, escaped markers, protected
syntax, link bracket scanning, blockquote parsing, URL behavior, typography
composition in French, Danish, and English, and heading metadata.
