# Image attributes and separate captions

- Status: Accepted
- Date: 2026-09-27
- Related: #466, #467, #468

## Context

Markdown image text is alternative text. Ferromark already has a separate
colon-prefixed table caption line and ID/class attribute blocks for headings
and tables. An image needs styling without requiring a caption, and a visible
caption must not replace either the alt text or the optional HTML title.

## Decision

Two independent `ParserOptions` flags are off in every preset:
`image_attributes` accepts an immediately adjacent `{#id .class}` suffix on
inline-destination and reference-style images; `image_captions` attaches one
nonempty `: Caption` line to a paragraph containing exactly one image. Node
uses `imageAttributes` and `imageCaptions`. A caption may follow immediately or
after one blank line, within the same block container. Linked images, mixed
prose, multiple images, and larger gaps do not become figures. An escaped
colon, indentation beyond three spaces, and an empty or attribute-only caption
remain ordinary Markdown. The image's own suffix targets `<img>`; a trailing
caption suffix targets `<figure>`. A caption without attributes is valid.

The native AST stores image ID/classes in an optional `ElementAttributes`
allocation on `Image` and a captioned image as
`Node::Figure` with `content`, inline `caption`, optional figure attributes, and a source
span covering the image through the caption. Visitors and rendering hooks
traverse both content and caption. The figure shape can later hold a quoted
block as content for #468. The parser does not infer a caption from alt text or
title, and an empty alt remains empty.

Image, table, and figure suffixes use one ID/class scanner and caption-line
grammar: at most one `#id`, any number of
`.class` tokens, nonempty names, and no quotes, angle brackets, equals, braces,
backslashes, or controls. Invalid blocks stay literal. Repeated IDs invalidate
the block. Heading suffixes retain their existing permissive parser, including
quoted names that the renderer escapes; narrowing that enabled legacy syntax
would be a separate compatibility change. The renderer escapes values and claims explicit IDs in
document order against heading IDs, appending `-1`, `-2`, and so on to collisions.
The configured heading ID prefix remains specific to headings. Generated
`<col>` classes retain their existing behavior. Arbitrary `key=value` metadata
belongs to #467.

With `table_attributes` enabled, a nonempty `: Caption` line now attaches to a
table without requiring an attribute block. `: {#id .class}` remains metadata
without a caption. This intentionally broadens the enabled table extension;
default CommonMark and GFM parsing remain unchanged. Figure and table captions
contain ordinary inline Markdown. A brace-free `: text | value |` within a table
body remains a data row; a plain caption with a pipe can follow after the
table. Malformed brace syntax remains literal at both attachment points. No configuration file or site-level caption
inference is added.

## Boundaries and consequences

Only a standalone image paragraph is eligible. A colon line in ordinary prose
keeps paragraph continuation. With definition lists also enabled, an eligible
image caption takes precedence over a definition body; all other definition
lists keep their prior behavior. Container subparsers keep figure and caption
spans in the original source. Code, raw HTML, MDX, and link destinations are
unaffected. The normal renderer applies URL policy and HTML escaping to image
and caption content, including untrusted output and XHTML images.
Caption lines also attach through existing lazy list/blockquote paragraph
continuation; for example `> ![Alt](a.png)\n: Caption` stays inside the quote.
Explicit authored IDs are not prefixed by `heading_id_prefix` or namespaced in
untrusted output, matching existing heading IDs. Applications accepting
untrusted authored IDs should avoid relying on them as safe DOM names.

Disabled image attributes return before suffix scanning, and disabled image
captions do not parse candidate lines. The `image_captions` benchmark measures
parse-only cost with 64 repeated blocks per workload. Criterion medians on an
arm64 Mac with rustc 1.95.0, 10 samples, 0.2-second warmup, and 0.5-second
measurement were:

| Workload | Both off | Attributes | Captions | Both on |
| --- | ---: | ---: | ---: | ---: |
| Ordinary prose (4,544 bytes) | 5.37 µs | 5.31 µs | 5.32 µs | 5.36 µs |
| Valid images and captions (3,968 bytes) | 11.66 µs | 13.72 µs | 20.92 µs | 28.21 µs |
| Repeated invalid IDs (3,520 bytes) | 7.95 µs | 9.64 µs | 11.99 µs | 13.81 µs |

These measurements describe the initial implementation before review fixes.
The follow-up avoids repeated inline parsing of caption-like lines once a
paragraph is ruled out, and keeps ordinary image nodes compact with an
optional attribute allocation. An external release harness then compared
`main` with the reviewed PR using GFM defaults (both options off), 96 repeated
blocks per input, seven 250 ms windows after a 200 ms warmup, and two
alternating process cycles. Medians on the same arm64 Mac were:

| Workload | Main parse | PR parse | Main parse + render | PR parse + render |
| --- | ---: | ---: | ---: | ---: |
| Ordinary prose | 10.84 µs | 10.89 µs (+0.5%) | 15.22 µs | 15.26 µs (+0.3%) |
| Ordinary images | 10.55 µs | 10.79 µs (+2.3%) | 16.19 µs | 16.51 µs (+2.0%) |
| Ordinary links | 12.93 µs | 12.97 µs (+0.4%) | 19.49 µs | 19.44 µs (−0.3%) |

These are descriptive local timings, not a cross-machine guarantee. The
within-build Criterion target exercises parsing only. The adversarial
caption-like paragraph has a scaling regression test.
