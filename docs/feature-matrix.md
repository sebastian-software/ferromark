# Feature matrix: scope and sources

The [website matrix](https://ferromark.dev/guide/feature-comparison) inventories built-in
capabilities, including opt-in parser flags, renderer options, and Cargo
features. It does not claim that all options are enabled together, that output
is identical, or that every edge case conforms to a specification. A missing
built-in feature can often be implemented by a caller using events or hooks.

The comparison covers the six engines in the archived native benchmark plus
markdown-rs and micromark in the separate ecosystem benchmark. Ferromark v2
is the local core at `33c216b`, including the compatibility corrections,
[smart-punctuation removal](typography.md), [table extensions](table-layout.md),
and [line comments](line-comments.md), plus [frontmatter extraction](front-matter.md);
the current native benchmark also refreshes v1 to local `4e15141`. The remaining
four sources retain their original pins. Feature availability and timed
configuration remain separate: the benchmark enables only a shared subset of
Markdown extensions and disables optional renderer conveniences where possible.

| Engine | Reviewed source |
| --- | --- |
| Ferromark v1 | 0.9.0, local `4e15141`; original feature review at [published `143ec2ce`](https://github.com/sebastian-software/ferromark/tree/143ec2ce151d87d2a3d804a048014afc97733ae0), benchmark option contract rechecked on the local pin |
| Ferromark v2 | Local `33c216b`; [parser options](../src/parser/options.rs), [renderer options](../src/renderer/html/options.rs), and [table layout](table-layout.md) |
| OX-Content | [3.2.3, `a71a5893`](https://github.com/ubugeeei-prod/ox-content/tree/a71a58939ffe7f154117cea026f6d6e71a139393) — parser/renderer core |
| pulldown-cmark | [0.13.4 options](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/struct.Options.html) |
| md4c | [`65c6c9d7` flags and event types](https://github.com/mity/md4c/blob/65c6c9d72cebd9a731aaa5597414ce04d9ea5de3/src/md4c.h) |
| markdown-rs | [`markdown` 1.0.0 configuration](https://github.com/wooorm/markdown-rs/blob/1.0.0/src/configuration.rs) and [public APIs](https://docs.rs/markdown/1.0.0/markdown/) |
| micromark | [4.0.2 core](https://github.com/micromark/micromark/tree/4.0.2), plus the explicitly listed extensions below |
| Bun native core | [`76e9dcc6` options and render entry points](https://github.com/oven-sh/bun/blob/76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1/src/md/root.rs) |

The pinned local source exports and registry package were inspected directly.
Their revision and checksum provenance is in the
[current native benchmark record](reports/2026-09-14-native-matched/README.md).

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
  navigable HTML link. OX emits link nodes; pulldown-cmark exposes a wiki link
  event and HTML rendering. Ferromark removed its opt-in wiki links in 3.0;
  applications can resolve `[[Page]]` in a transform or link hook instead.
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
  IDs, callouts, and fence metadata cleanup.

## Source details

For v2/OX, the option types, AST node types, inline parser, and HTML renderer
establish the available features. Existing v2 regression tests
cover the individual extensions and profile behavior. A renderer option is not
evidence of a Markdown syntax: the parser must recognize that syntax before it
can be counted, which is why the inherited no-op `highlight` renderer option was
removed rather than counted as `==marked text==` support. The current v2 parser
now implements opt-in marked text and inline notes; see
[their syntax contract](optional-writing.md). Historical benchmark revisions
and other engine columns remain unchanged.

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
from the Ferromark core; heading data remains available through the AST and the
outline extension is tracked separately.

OX's [parser options](https://github.com/ubugeeei-prod/ox-content/blob/a71a58939ffe7f154117cea026f6d6e71a139393/crates/ox_content_parser/src/parser/options.rs),
[inline parser](https://github.com/ubugeeei-prod/ox-content/blob/a71a58939ffe7f154117cea026f6d6e71a139393/crates/ox_content_parser/src/parser/inline.rs),
and [HTML hooks](https://github.com/ubugeeei-prod/ox-content/blob/a71a58939ffe7f154117cea026f6d6e71a139393/crates/ox_content_renderer/src/html/renderer/hooks.rs)
establish its extension and customization surface. Its renderer handles callouts
and `[[toc]]` directly. Frontmatter processing in other OX packages does not count
as support in the parser/renderer core benchmarked here.

## markdown-rs and micromark

These sibling projects broaden the capability inventory beyond the archived
six-engine native benchmark. A capability entry does not imply that it was
measured or enabled in the matched benchmark profile.

- **markdown-rs 1.0.0** (`markdown` on crates.io) provides CommonMark constructs,
  configurable GFM including single-tilde strikethrough, footnotes and tag
  filtering, math, frontmatter, MDX syntax, HTML output, and mdast trees.
  [Configuration](https://github.com/wooorm/markdown-rs/blob/1.0.0/src/configuration.rs)
  and [APIs](https://docs.rs/markdown/1.0.0/markdown/) define the reviewed scope.
  ESM requires a parse callback; expression callbacks add language-aware
  validation. Math is syntax handling, frontmatter is metadata recognition,
  and custom HTML beyond compile options requires an application renderer.
- **micromark 4.0.2** provides CommonMark tokens and HTML compilation.
  [Official extensions](https://github.com/micromark/micromark/tree/4.0.2#list-of-extensions)
  supply GFM, footnotes, frontmatter, math and MDX. The matrix labels these
  separately installed packages **Ext.**, rather than built-in core features.
  [mdast-util-from-markdown](https://github.com/syntax-tree/mdast-util-from-markdown)
  supplies the companion tree API and needs matching mdast extensions for
  extension nodes. HTML extensions customize token compilation.
  Community extensions and remark plugins are outside this inventory.

The [ecosystem harness](../benchmarks/markdown-ecosystem/README.md) pins markdown
1.0.0, micromark 4.0.2, GFM strikethrough 2.1.0, table 2.1.1 and task-list-item
2.1.0. Its extension lane enables only those three GFM features. It does not
measure full GFM, math, frontmatter or MDX. The native pair and Node pair have
separate runtime/build contracts and results; they are never pooled with the
archived six-engine measurements.

## Node.js projects

The homepage includes the main HTML converters and parser ecosystems:
[marked](https://github.com/markedjs/marked),
[markdown-it](https://github.com/markdown-it/markdown-it),
[remark / unified](https://github.com/remarkjs/remark),
[micromark](https://github.com/micromark/micromark),
[Showdown](https://github.com/showdownjs/showdown), and
[commonmark.js](https://github.com/commonmark/commonmark.js).
The benchmark pins their versions and compares complete Markdown-to-HTML string
calls, including Ferromark's Node binding overhead. These projects cover both
direct converters and AST/plugin pipelines; their rows do not imply identical
feature sets or document agreement sets.

| Capability | marked | markdown-it | remark / unified | micromark | Showdown | commonmark.js |
| --- | --- | --- | --- | --- | --- | --- |
| HTML string output | ✓ | ✓ | HTML pipeline | ✓ | ✓ | HTML renderer |
| Tables | ✓ | ✓ | Ext. | Ext. | Opt-in | — |
| Strikethrough | ✓ | ✓ | Ext. | Ext. | Opt-in | — |
| Task lists | ✓ | Ext. | Ext. | Ext. | Opt-in | — |
| Public syntax representation | Tokens | Tokens | mdast / hast | Tokens; mdast companion | — | AST |

“HTML pipeline” means remark + remark-rehype + rehype-stringify; remark alone
serializes Markdown. “Ext.” means separately installed plugins/extensions.
The measured pipeline enables only table, strikethrough, and task-list syntax,
without remark-gfm's additional autolinks or footnotes. markdown-it retains
its `<s>` spelling and task-list plugin classes; Showdown retains its task-list
styles and extra fenced-code classes. These differences are not removed to
enlarge the agreement set.
commonmark.js enables CommonMark only in both engines on every corpus input.
[Exact versions, adapters, and measurement scope](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md).

[react-markdown](https://github.com/remarkjs/react-markdown) belongs to the
remark/unified ecosystem and returns React elements;
[MDX](https://github.com/mdx-js/mdx) compiles Markdown with JSX to JavaScript.
Those operations need their own workload rather than a second HTML-string
throughput row. Their project links are included here so readers can find the
framework integrations alongside the parser comparison.

The native overview also lists
[Comrak](https://github.com/kivikakk/comrak),
[cmark](https://github.com/commonmark/cmark), and
[cmark-gfm](https://github.com/github/cmark-gfm). Matched native measurements for all three are retained in the
[completed platform report](https://github.com/sebastian-software/ferromark/tree/main/docs/reports/2026-09-30-ecosystem-platforms). No factor is inferred
from another implementation or another runtime.
