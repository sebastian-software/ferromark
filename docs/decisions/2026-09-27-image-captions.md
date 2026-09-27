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

The native AST stores image ID/classes on `Image` and a captioned image as
`Node::Figure` with `content`, inline `caption`, figure ID/classes, and a source
span covering the image through the caption. Visitors and rendering hooks
traverse both content and caption. The figure shape can later hold a quoted
block as content for #468. The parser does not infer a caption from alt text or
title, and an empty alt remains empty.

Image and table suffixes use one grammar: at most one `#id`, any number of
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
contain ordinary inline Markdown. No configuration file or site-level caption
inference is added.

## Boundaries and consequences

Only a standalone image paragraph is eligible. A colon line in ordinary prose
keeps paragraph continuation. With definition lists also enabled, an eligible
image caption takes precedence over a definition body; all other definition
lists keep their prior behavior. Container subparsers keep figure and caption
spans in the original source. Code, raw HTML, MDX, and link destinations are
unaffected. The normal renderer applies URL policy and HTML escaping to image
and caption content, including untrusted output and XHTML images.

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

The ordinary-prose differences are within the run's noise. Enabling captions
adds a visible parsing cost for image-heavy input because each candidate
paragraph is parsed again to decide whether the colon line attaches. The
benchmark exercises parsing only and makes no rendering throughput claim.
