# Optional technical abbreviation markup

- Status: Accepted
- Date: 2026-09-27
- Related: #463

## Context

Technical prose often repeats uppercase terms whose expansions are known to the
site or author but are not written in each Markdown file. Requiring raw HTML
for every occurrence makes the source harder to maintain. Automatic detection
must remain predictable and must not change the AST or default output.

## Decision

Add an explicit abbreviation configuration in Rust and Node.js. It wraps each
eligible complete uppercase token during HTML rendering, using one maintained
built-in dictionary and a caller-provided exact-term override map. Matching
stays off by default. The Rust core owns the matcher, so Node calls use the same
behavior.

Candidates are case-sensitive ASCII uppercase letters plus optional ASCII
digits, with at least two uppercase letters. A Unicode letter, Unicode digit,
or underscore is an identifier character for the surrounding token boundary.
The final lowercase `s` is a plural suffix when followed by a boundary; it is
left outside the wrapper. Hyphens and slashes are boundaries, except that a
complete dictionary term such as `HTTP/2` or `UTF-8` is matched as one term.
Exact caller entries can add mixed-case terms such as `GraphQL`.

The caller map has three distinct value states: a nonempty string supplies a
title, an empty string forces a wrapper without a title, and `null` suppresses
the exact term. Missing keys leave the built-in dictionary and heuristic
available. The map alone does not enable processing.

Processing runs during rendering. It does not mutate or annotate the AST.
Therefore source spans, heading IDs, and `transform()` heading metadata keep
their authored text. Text inside emphasis and Markdown link labels remains
eligible. Code, math, raw HTML, authored abbreviation markup, MDX expressions
and attributes, front matter, image metadata, URL text, and link destinations
remain outside generated markup. Generated text and titles use the renderer's
escaping path.

## Consequences

Rust consumers pass a separate typed `AbbreviationOptions` to the additive
`HtmlRenderer::with_abbreviations` builder,
`HtmlRenderer::with_options_and_abbreviations` constructor, or
`to_html_with_options_and_abbreviations` convenience function. The new options
type is non-exhaustive so it can gain fields later; the existing
`HtmlRendererOptions` struct remains frozen and source-compatible. Node
consumers use `autoAbbreviations` and `abbreviations` on the additive options
object, with `abbreviations` typed as `Record<string, string | null>`. The
renderer prepares dictionary and URL boundary lookup data once when the option
is enabled, then reuses it. Disabled renderers do not build the matcher or scan
text. The heuristic can wrap
all-uppercase words such as `MUST` or `README`; callers can suppress these with
`null` overrides.

The initial dictionary is authored in the Rust implementation and has no
external data license or network dependency. The public behavior is opt-in,
does not claim recognition accuracy, and can grow its dictionary independently
of the token heuristic. A configuration file, document-local learning, and
AST annotations remain outside this decision.

## Validation

Rust and Node tests cover the option defaults, built-in and unknown terms,
overrides, boundaries, plural and compound handling, protected contexts,
renderer reuse, and all Node rendering entry points. The sample
[`abbreviations.md`](../abbreviations.md) records the observed false
positives. A Criterion benchmark compares disabled, enabled, no-candidate, and
reused-renderer cases; allocation tests check disabled construction and warmed
renderer reuse.
