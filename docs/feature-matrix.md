# Feature matrix: scope and sources

The [README matrix](../README.md#features-at-a-glance) inventories built-in
capabilities, including opt-in parser flags, renderer options, and Cargo
features. It does not claim that all options are enabled together, that output
is identical, or that every edge case conforms to a specification. A missing
built-in feature can often be implemented by a caller using events or hooks.

The comparison uses the same six engines as the native benchmark. Ferromark v2
is the local core at `40047f7`, including the compatibility corrections,
[smart-punctuation removal](typography.md), [table extensions](table-layout.md),
and [line comments](line-comments.md), plus [frontmatter extraction](front-matter.md);
the other five sources are the exact
benchmark pins. This avoids mixing newer upstream features into an older timing
comparison. Feature availability and timed configuration remain separate: the
benchmark enabled only a shared subset of Markdown extensions.

| Engine | Reviewed source |
| --- | --- |
| Ferromark v1 | [0.9.0, `143ec2ce`](https://github.com/sebastian-software/ferromark/tree/143ec2ce151d87d2a3d804a048014afc97733ae0) |
| Ferromark v2 | Local `40047f7` plus [frontmatter extraction](front-matter.md); [parser options](../crates/ferromark_parser/src/parser/options.rs), [renderer options](../crates/ferromark_renderer/src/html/options.rs), and [table layout](table-layout.md) |
| OX-Content | [3.2.3, `a71a5893`](https://github.com/ubugeeei-prod/ox-content/tree/a71a58939ffe7f154117cea026f6d6e71a139393) — parser/renderer core |
| pulldown-cmark | [0.13.4 options](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/struct.Options.html) |
| md4c | [`65c6c9d7` flags and event types](https://github.com/mity/md4c/blob/65c6c9d72cebd9a731aaa5597414ce04d9ea5de3/src/md4c.h) |
| Bun native core | [`76e9dcc6` options and render entry points](https://github.com/oven-sh/bun/blob/76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1/src/md/root.rs) |

The pinned local source exports and registry package were inspected directly.
Their revision and checksum provenance is in the
[native benchmark record](reports/2026-09-14-native-engines/PROVENANCE.md).

## What the labels mean

[CommonMark](https://spec.commonmark.org/0.31.2/) specifies the everyday Markdown
syntax in the first group. [GFM](https://github.github.io/gfm/) means GitHub
Flavored Markdown and adds tables, strikethrough, task lists, expanded autolinks,
and a raw-HTML tag filter. Footnotes and alert boxes are useful on GitHub too,
but are not among those five extensions in the published GFM specification.

A check mark means that the described capability exists in the reviewed core;
it is not a conformance score. Use the separate
[correctness results](reports/2026-09-14-reference-compatibility/README.md) for
Ferromark v2's tested behavior. Published syntax names do not guarantee matching
rendering policies or identical edge cases across libraries.

## Important distinctions

- **Line comments:** v1 and v2 can omit physical source lines beginning with
  `//` after at most three ASCII spaces. This is an opt-in source notation,
  separate from HTML comments and JavaScript comments inside MDX. Code and raw
  HTML blocks stay opaque; explicit `>` or list prefixes are not stripped for
  comment recognition. V1 exposes comment events; v2 omits comments from the AST
  while preserving original source spans for the remaining nodes.

- **Table widths:** v2 can generate CSS-addressable `<col>` elements and table
  IDs/classes. Columns retain positional classes and can also receive names
  derived from header text. Width values remain in application CSS. This differs
  from v1's numeric width hints derived from delimiter dash counts; the matrix continues
  to distinguish the two capabilities. Its table-layout rows cover built-in
  Markdown table handling, excluding raw HTML authored directly and layouts
  implemented through application rendering. V2 captions require an ID/class list
  on the same metadata line; see the [syntax boundaries](table-layout.md#syntax-and-boundaries).

- **Single tildes:** v1 reserves `~text~` for subscript; original OX handles
  double-tilde strikethrough. V2, pulldown-cmark, and md4c support single-tilde
  strikethrough when their competing subscript option is off. Bun's pinned
  delimiter collector accepts one or two tildes.
- **GFM presets:** pulldown-cmark's `ENABLE_GFM` enables alert block quotes; it
  does not turn on tables, tasks, or strikethrough, and does not add bare URL
  autolinking or the tag filter. md4c's `MD_DIALECT_GITHUB` bundles its relevant
  available extensions, including footnotes and alerts, but has no dedicated
  GFM tag filter. Disabling all raw HTML is a different capability.
- **Math:** support means recognizing and preserving formula notation. It does
  not mean a TeX typesetter is included. V2 emits math wrappers; md4c and Bun
  emit `<x-equation>` elements; pulldown-cmark exposes math events. Applications
  supply the desired formula presentation.
- **Wiki links:** md4c and Bun produce `<x-wikilink>` elements. Their recognition
  of `[[Page]]` does not by itself supply page routing or turn it into a normal
  navigable HTML link. V2 and OX emit link nodes; pulldown-cmark exposes a wiki
  link event and HTML rendering.
- **Frontmatter:** recognizing a delimited metadata block is separate from
  parsing YAML/TOML into typed values. The table credits extraction/recognition,
  not semantic metadata deserialization. V2 extracts a single block only at the
  document start, optionally after a BOM, and preserves its original text and
  source spans. It never parses metadata as Markdown or includes it in HTML.
- **MDX:** v1's opt-in segment/event APIs and v2/OX's JSX, expression, and ESM
  capture are bounded syntax support. They are not full `@mdx-js/mdx` compilers
  or JavaScript execution engines. OX's separate framework/SSG packages and
  Bun's JavaScript/React runtime APIs are outside this core comparison.
- **Rendering policy:** raw HTML may require explicit opt-in (notably v1's
  trusted policy). A GFM tag filter handles a fixed tag list; it is not general
  HTML sanitization. V2's CommonMark/GFM renderer profiles disable convenience
  IDs, callouts, TOC substitution, and fence metadata cleanup.

## Source details

For v2/OX, the option types, AST node types, inline parser, HTML renderer, and
TOC/callout code establish the available features. Existing v2 regression tests
cover the individual extensions and profile behavior. Options such as `highlight`
on the HTML renderer are not evidence of `==marked text==` syntax: the parser
must recognize that syntax before it can be counted.

For pulldown-cmark, the versioned `Options` and `Tag` documentation, the bundled
HTML writer, and `Parser::into_offset_iter()` establish the parser/rendering
surface. Its events can be transformed by the caller without a stored AST.

For md4c, `src/md4c.h` defines all extension flags and block/span event kinds;
`src/md4c.c` implements delimiter recognition and `src/md4c-html.c` implements
its HTML output. This pin includes footnotes, highlights, alerts, super/subscript,
and spoilers; those should not be inferred from older md4c releases.

For Bun, `src/md/root.rs` defines the options, public boolean setters, and
`render_with_renderer()`. `types.rs` defines block/span kinds and callback data;
`inlines.rs` handles tilde runs; `html_renderer.rs` implements math/wiki output.
The [Bun Markdown documentation](https://bun.com/docs/runtime/markdown) explains
the public product API, but this matrix is bounded by the pinned native source.

Ferromark v1's [public options and result types](https://github.com/sebastian-software/ferromark/blob/143ec2ce151d87d2a3d804a048014afc97733ae0/src/lib.rs)
cover its footnotes, frontmatter, line comments, column spans/widths, math, marked text,
super/subscript, callouts, automatic heading IDs, and heading records for a
caller-built TOC. Its generic block/inline events are public, but the built-in
HTML renderer's callback is limited to fenced code (`FencedCodeRenderer`).
The optional [MDX module](https://github.com/sebastian-software/ferromark/tree/143ec2ce151d87d2a3d804a048014afc97733ae0/src/mdx)
provides segmentation, structural checks, and semantic events. Smart punctuation,
wiki links, explicit heading attributes, and a rendered inline TOC are absent
from this native snapshot.

OX's [parser options](https://github.com/ubugeeei-prod/ox-content/blob/a71a58939ffe7f154117cea026f6d6e71a139393/crates/ox_content_parser/src/parser/options.rs),
[inline parser](https://github.com/ubugeeei-prod/ox-content/blob/a71a58939ffe7f154117cea026f6d6e71a139393/crates/ox_content_parser/src/parser/inline.rs),
and [HTML hooks](https://github.com/ubugeeei-prod/ox-content/blob/a71a58939ffe7f154117cea026f6d6e71a139393/crates/ox_content_renderer/src/html/renderer/hooks.rs)
establish its extension and customization surface. Its renderer handles callouts
and `[[toc]]` directly. Frontmatter processing in other OX packages does not count
as support in the parser/renderer core benchmarked here.
