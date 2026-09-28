# Block quote attributions

- Status: Accepted
- Date: 2026-09-28
- Related: #466, #467, #468

## Context

Images and block quotes are content; visible descriptions and sources are
separate publishing metadata. The AST's `Figure` node already wraps content and
an inline caption, and `BlockQuote` preserves its own nested Markdown structure.
A block quote needs an attribution that remains visible and is not conflated
with HTML's `cite` URL semantics.

## Decision

Add opt-in `blockquote_attributions` (`blockquoteAttributions` in Node), off in
every preset. A nonempty colon-prefixed source line immediately after a block
quote or after one blank line becomes the `Figure` caption. The figure's content
is the full `BlockQuote`; reference and inline links use normal inline parsing.
The renderer emits `<figure><blockquote>…</blockquote><figcaption>…</figcaption></figure>`.
It does not generate a `<cite>` element or a `cite` URL attribute.

The line must be outside the quote and in the same parsed parent container. The
parser attaches to the nearest nested quote and accepts no gap larger than one
blank line. An outdented source after a quote in a list item remains lazy
continuation inside the quote; an indented source can attach inside the item.
This also preserves a following list item in the same list. A trailing valid attribute block
targets the figure. Empty and attribute-only captions, escaped colons, and
malformed attribute blocks do not create a figure. The attribution option
leaves callout handling unchanged: the parser checks the parsed quote paragraph
and excludes recognized callout markers even when nested or the HTML renderer has
`callouts` disabled. An unrecognized `[!…]` marker remains an ordinary quote
and can receive an attribution. Definition lists continue to win when the next
block begins with a term. With the option off, CommonMark lazy continuation is
unchanged.
An empty quote does not receive a figure.

The existing `Figure` AST shape carries the quote, caption, figure ID/classes,
shared key/value attributes when `extended_attributes` is enabled, and an
original-source span through the source line. Generic visitors,
transforms, and rendering hooks traverse both content and caption. Inline
caption text is passed through ordinary HTML escaping and URL policy. Protected
blocks inside the quote remain ordinary quote content.

## Compatibility

The attribution option is disabled in every preset, preserving existing quote
parsing and rendering until explicitly enabled. Adding the public
`ParserOptions::blockquote_attributions` field changes an exhaustive Rust
options struct and requires a major Rust API release. The Node.js option is
additive. The Node facade uses its existing object-taking entry only when the
attribution option is enabled, leaving the default packed path and its bit
allocation unchanged. The generated figure uses the existing `Figure` AST
variant.

## Performance

`benches/blockquote_attributions.rs` compares enabled and disabled parsing for
ordinary prose, quotes without sources, valid attributions, and malformed
attribute suffixes. It repeats each shape 64 times. Criterion ran on an arm64
Mac with rustc 1.95.0, 10 samples, 0.2-second warmup, and 0.5-second
measurement after rebasing onto #467, removing duplicate attribution parsing,
and caching callout recognition per quote. The central estimates and
enabled-over-disabled differences were:

| Input | Bytes | Off | On | Change |
| --- | ---: | ---: | ---: | ---: |
| Ordinary prose | 4,096 | 4.815 µs | 4.795 µs | −0.4% |
| Quotes without sources | 3,520 | 14.441 µs | 15.979 µs | +10.7% |
| Valid attributions | 5,376 | 22.956 µs | 23.041 µs | +0.4% |
| Malformed suffixes | 2,112 | 11.874 µs | 16.949 µs | +42.7% |

The ordinary-prose difference is negligible at this sample size. The valid
attribution row compares different outputs: only the enabled parser builds a
figure and parses its inline caption, so its +0.4% includes that work. The
quotes-without-sources and malformed-suffix rows compare equal output and
isolate the enabled scan cost (+10.7% and +42.7%, respectively). Malformed
suffixes still make the enabled option noticeably more expensive. These are
parse-only measurements within this PR; they do not compare the disabled path
against the base branch or measure rendering throughput.

To isolate the disabled path, a separate local batch comparison used the same
57-document corpus against the #473 base at `85f99144`, with the attribution
option off on both sides. A common worker applied the same explicit CommonMark
and GFM profile flags to both builds. HTML and AST outputs matched exactly.
Three rounds used five alternating pairs of 40 ms windows:

| Combined CommonMark/GFM batch | Base median | PR median | Change |
| --- | ---: | ---: | ---: |
| Fresh render | 1.058 ms | 1.059 ms | +0.117% |
| Reused render | 1.049 ms | 1.053 ms | +0.388% |

Fresh per-round differences were −0.935%, −1.476%, and +0.117%; reused
differences were +0.388%, −1.176%, and +0.913%. These small changes are within
local run variation, so this comparison shows no measurable default-off
regression. The batch mode reports aggregate timings rather than per-document
outliers.
