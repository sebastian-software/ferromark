# Ferromark Flavored Markdown preset

- Status: Accepted
- Date: 2026-09-28
- Related: #479

## Context

Version 3.0 groups Ferromark's opt-in authoring syntax as Ferromark Flavored
Markdown (FFM). Enabling it meant setting up to fifteen parser options, and the
Node.js API had no named profiles at all. A single switch makes FFM a usable
dialect rather than a list of flags, but each enabled option can cost parse
time even when a document does not use its syntax.

## Decision

Add `ParserOptions::ffm()` in Rust and `preset: "ffm"` in Node.js.

The profile is `ParserOptions::gfm()` plus marked text, inserted text, inline
notes, superscript, definition lists, heading and table attributes, merged
table cells, image attributes and captions, block quote attributions, shared
key/value attributes, bracketed spans, source-only line comments, and
guillemet digraphs.

It excludes subscript (it would turn GFM `~text~` strikethrough into
subscript), math and wiki links (they need downstream typesetting or routing),
front matter, CJK emphasis, and MDX. Renderer policies stay separate: the
preset does not change sanitization, table colgroups, heading IDs, or
typography. Guillemet digraphs therefore stay literal text until a typography
pass with an explicit language converts them.

In Node.js the preset also enables technical abbreviation markup, because the
Node options object configures parsing and rendering together. In Rust,
abbreviations are a renderer feature: pair the profile with
`HtmlRenderer::with_abbreviations`. Node.js applies the preset before every
individual option, so explicit values, including `false`, still override it.
The Node.js package keeps its own defaults for everything outside the FFM
syntax, for example bare-URL autolinks stay off. An unknown preset name is an
error.

## Performance

Measured on the 3.0 core (Rust 1.95, fat LTO, generic CPU, Apple M1 Ultra),
parse only, with each option enabled on six real documents that do not use
its syntax, compared to the GFM options:

| Option | Geometric mean | Worst document |
| --- | ---: | ---: |
| `definition_lists` | +20.9% | +48.1% |
| `inline_footnotes` | +5.6% | +10.8% |
| `insertions` | +5.5% | +12.0% |
| `superscript` | +4.9% | +9.7% |
| `highlight` | +4.2% | +11.4% |
| `line_comments` | +3.6% | +15.2% |
| every other FFM option | at most +0.8% | at most +4.0% |

With the whole profile enabled, the same documents parse 10–55% slower and
render end to end 7–29% slower. Technical abbreviation markup adds render
time in proportion to the prose: +15% to +114% on these documents. The preset
is a convenience for richer documents, not a free default; applications that
need only one or two features should keep enabling them individually. The
largest remaining cost, definition-list scanning in documents without
definition lists, is the first optimization candidate.

## Consequences

`ParserOptions::ffm()` adds a named constructor; `ParserOptions` fields are
unchanged. Node.js gains an optional `preset` field on the object-taking
native entry, so the packed option layout and its bits stay unchanged. Adding
options to FFM later changes the output of existing preset users and needs
its own decision.
