# Ferromark v2

A lean, arena-allocated Markdown parser and HTML renderer.

Ferromark v2 brings together our work on
[Ferromark v1](https://github.com/sebastian-software/ferromark) and the
arena-based AST architecture of [OX-Content](https://github.com/ubugeeei-prod/ox-content).
In v1, we encountered performance limits that called for a deeper architectural
change. OX-Content provided the foundation for that rebuild, and we are grateful
to its authors. V2 combines that foundation with selected v1 features and
optimizations, alongside new development, to shape a Markdown-to-HTML library
with its own scope and direction.

This is a local development baseline with a fresh Git history and unpublished packages.
The API is not compatible with Ferromark v1 and is not a stable v2 release.

```rust
use ferromark::{Allocator, HtmlRenderer, Parser};

let source = "Hello, **world**!";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::new(&allocator, source).parse().unwrap();
let html = HtmlRenderer::new().render(&document);
assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>\n");
```

`ParserOptions::gfm()` enables the implemented GFM syntax plus footnotes.
For specification-oriented output, pair `ParserOptions::gfm_spec()` with
`HtmlRendererOptions::gfm()` (tagfilter enabled, footnotes disabled). The analogous
CommonMark pair is `ParserOptions::commonmark()` and
`HtmlRendererOptions::commonmark()`. These renderer profiles disable automatic
heading IDs, callouts, TOC, and fence metadata cleanup; the existing defaults
retain those conveniences. `ParserOptions { mdx: true, ..ParserOptions::gfm() }` adds MDX
syntax recognition and static component-island output; it does not provide an
MDX compiler or runtime. Math, definition lists, and other extensions remain configurable.
The allocator and source must outlive the document. Reuse/reset an allocator only after its documents
have been dropped. HTML options and renderer hooks remain available directly.

The workspace contains four core crates and a small re-export facade:

| Crate | Responsibility |
| --- | --- |
| `ferromark` | Public entry point |
| `ferromark_allocator` | Arena allocation and buffer helpers |
| `ferromark_ast` | Nodes, source spans, visitors |
| `ferromark_parser` | Markdown to AST |
| `ferromark_renderer` | AST to HTML |

The core retains CommonMark/GFM support, configurable syntax extensions, HTML
rendering options, and regression tests. Site generation, JavaScript frameworks,
bindings, editor services, and the upstream profiler are removed. See the precise
[cleanup boundary](docs/fork.md) and [source provenance](UPSTREAM.md).

Build and verify with the pinned Rust toolchain:

```sh
cargo fmt --all --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo bench --workspace --no-run --locked
```

Try the retained stdin rendering example:

```sh
printf 'Hello, **world**!\n' | cargo run --quiet --locked -p ferromark_renderer --example render_stdin
```

Seven self-contained Criterion suites cover parsing, reference prepasses, tables,
pipe scans, rendering, headings, and sanitized URLs. Run individual suites with
`cargo bench -p ferromark_parser --bench table_pipes --locked` or
`cargo bench -p ferromark_renderer --bench renderer --locked`.
The [optimization roadmap](docs/optimization-roadmap.md) records candidates for
measured ports from the Ferramenta projects.

The [optimization rounds](docs/reports/2026-09-14-optimization-rounds/README.md)
record measured SIMD and algorithm changes, including rejected variants and raw
results. The core now skips clean link unescaping, fuses enabled inline markers,
uses compact constant-time table span maps, trims URL brackets in linear time,
and scans ASCII URL spans with NEON. The
[first SIMD study](docs/reports/2026-09-14-simd-round/README.md) remains the historical
record of the link prototype and its render-only build sensitivity.

The [runtime-profile study](docs/runtime-profiles.md) measures 37 individual
options, unused-feature overhead, and candidate recipes for comments, articles,
documentation, and MDX. It separates parser/rendering costs and fresh/reused
lifecycles, with exact-output checks for tailored-profile comparisons.
The [definition-list and line-comment follow-up](docs/reports/2026-09-14-feature-scan-optimization/README.md)
investigates and reduces their scan/allocation overhead while preserving HTML,
ASTs, and source positions. It records accepted and rejected attempts.

## Features at a glance

**CommonMark** defines everyday Markdown: headings, lists, links, emphasis, and
code. **GFM** (GitHub Flavored Markdown) adds features such as tables and
checklists. The groups below spell out those names and show useful extras.
Footnotes and alert boxes are listed separately from the published GFM extensions.

**✓** = built in, possibly requiring an option or Cargo feature. **—** = no
built-in support. Qualifiers describe narrower support. This is a capability
inventory, not a claim of identical output or complete specification conformance.
It covers the native cores from the benchmark, with **v2 updated to the current
local core**; these are not the options enabled during timing.
[Exact versions, source references, and scope](docs/feature-matrix.md).

| Feature / what it does | Ferromark v1 | Ferromark v2 | OX-Content | pulldown-cmark | md4c | Bun native |
| --- | :---: | :---: | :---: | :---: | :---: | :---: |
| **CommonMark — everyday Markdown** | | | | | | |
| Headings — `# Title` through `###### Title` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Paragraphs and line breaks | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Bold and italic — `**bold**`, `*italic*` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Numbered, bulleted, and nested lists | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Block quotes — `> quoted text` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Horizontal separators — `---` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Links, images, and reusable reference links | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Inline code, fenced code blocks, and indented code | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Raw HTML inside Markdown — `<details>…</details>` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| **GFM — GitHub-style extensions** | | | | | | |
| Tables with left/center/right column alignment | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Checklists — `- [x] done`, `- [ ] pending` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Crossed-out text — `~~removed~~` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Single-tilde crossed-out text — `~removed~` | — | ✓ | — | ✓ | ✓ | ✓ |
| Turn bare URLs/emails into links — `www.example.com` | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| GFM HTML tag filter — filter its specified tag list | ✓ | ✓ | ✓ | — | — | ✓ |
| **Writing extras — beyond CommonMark/GFM** | | | | | | |
| Reference footnotes — `[^note]` plus a definition | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Inline footnotes — `^[note written here]` | ✓ | — | — | — | — | — |
| Definition lists — a term followed by `: explanation` | ✓ | ✓ | ✓ | ✓ | — | — |
| Math notation — `$x^2$`, `$$…$$` | Syntax | Syntax | Syntax | Syntax | Syntax | Syntax |
| Superscript — `x^2^` | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Subscript — `H~2~O` | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Highlighted text — `==important==` | ✓ | — | — | — | ✓ | — |
| Wiki links — `[[Page]]` | — | ✓ | ✓ | ✓ | Custom tag | Custom tag |
| Smart punctuation — curly quotes and ellipses | — | — | ✓ | ✓ | — | — |
| Extract frontmatter — metadata between `---` or `+++` | ✓ | ✓ | — | ✓ | — | — |
| Source-only line comments — hide `// note` lines from HTML | ✓ | ✓ | — | — | — | — |
| **Table layout — beyond GFM** | | | | | | |
| Merged table cells — one cell spans several columns | ✓ | ✓ | — | — | — | — |
| Numeric column-width hints — proportions from delimiter dashes | ✓ | — | — | — | — | — |
| Table IDs/classes — style a whole table with CSS | — | ✓ | — | — | — | — |
| Table captions — a label with inline Markdown formatting | — | With ID/class | — | — | — | — |
| Column classes for CSS widths — `col-1`, `col-2`, etc. | — | ✓ | — | — | — | — |
| Column names from headers — “Netto Preis” → `col-name-netto-preis` | — | ✓ | — | — | — | — |
| **Documentation and navigation** | | | | | | |
| Automatic heading IDs — link directly to a section | ✓ | ✓ | ✓ | — | — | ✓ |
| Explicit heading IDs/classes — `# Title {#id .class}` | — | ✓ | ✓ | ✓ | — | — |
| Alert boxes — `> [!NOTE]`, `> [!WARNING]` | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Table of contents from document headings | Heading data | Rendered TOC | Rendered TOC | — | — | — |
| **MDX — components and expressions mixed with Markdown** | | | | | | |
| Recognize JSX, `{expressions}`, and imports/exports | Limited | Limited | Limited | — | — | — |
| **Application integration** | | | | | | |
| Inspect/transform the parsed document | Events | Tree (AST) | Tree (AST) | Events | Callbacks | Callbacks |
| Customize generated output | Code hook / events | HTML hooks | HTML hooks | Event transforms | Renderer callbacks | Renderer callbacks |

Math support preserves formula notation; an application supplies mathematical
typesetting. “Custom tag” wiki links also need application-side routing/rendering.
Frontmatter support extracts metadata text, rather than parsing YAML/TOML values.
“Heading data” lets an application build a TOC; “Rendered TOC” includes HTML for
an inline `[[toc]]` marker. Trees expose nested document nodes; events/callbacks
expose elements in sequence. The three MDX entries cover bounded syntax handling,
not a full MDX compiler or JavaScript runtime. Single-tilde strikethrough and
subscript compete for the same syntax; enabling subscript gives it priority.

V2 preserves authored punctuation. Automatic typography belongs in an optional,
locale-aware document transform; the former English-oriented parser option has
been removed. See the [typography decision](docs/typography.md).

Enable `ParserOptions::front_matter` to extract a leading `---` (YAML) or `+++`
(TOML) metadata block into `document.front_matter`. It exposes the format, raw
content, and original source spans. The block is omitted from HTML and cannot
define Markdown links or footnotes. The option is off in every preset.
[Frontmatter syntax and Rust example](docs/front-matter.md).

Enable `ParserOptions::line_comments` to omit `// note` source lines from HTML.
Comments may have up to three leading spaces and do not separate paragraphs.
Code blocks, raw HTML blocks, and explicitly prefixed lines such as `> // text`
remain literal. The option is off in every preset.
[Syntax, examples, and source-span behavior](docs/line-comments.md).

V1 derives numeric column-width hints from delimiter dash counts. V2 instead
supports table IDs/classes, inline captions, and generated `colgroup` columns
whose widths are set in external CSS. Optional header-derived classes such
as `col-name-netto-preis` use the heading slug rules and retain positional classes.
Both versions use adjacent pipes for horizontal spans (`||` spans two columns).
All V2 table extras are opt-in and off in every preset. The table-layout rows
describe built-in handling of Markdown tables; raw HTML and custom rendering can
provide additional layouts in any engine.
[Syntax, options, and a runnable CSS example](docs/table-layout.md).

## Correctness and compatibility

The [second correction batch](docs/reports/2026-09-14-reference-compatibility/README.md)
closes the seven cmark findings left after the
[first fixes](docs/reports/2026-09-14-correctness-fixes/README.md). Strikethrough now
uses the inline delimiter stack, Unicode URL bytes receive percent encoding,
and one leading BOM is treated as an encoding marker with original spans preserved.

| Check | Result |
| --- | --- |
| Explicit CommonMark 0.31.2 profile | 652/652 agree, without heading-ID exceptions |
| Current GFM extension examples | 28/28 agree |
| CRLF / CR variants | 1,304/1,304 agree with their LF controls |
| Original cmark / cmark-gfm corpus | 106/106 agree |
| Additional tilde/inline combinations | 254/256 agree with cmark-gfm; two pinned-oracle nested-link defects follow the specification instead |
| Workspace regression tests | 770 pass, including both oracle corpora, renderer-profile/span checks, table layout/column names, line comments, frontmatter, and definition-list scan regressions |

Agreement permits conservative HTML serialization equivalence; raw output and
all mismatches remain in the report. The two reference exceptions have exact
spec-correct HTML assertions, and the generic strict oracle still rejects them.
The complete GFM website retains ten classified differences from global
extension policies and older HTML-comment rules. MDX remains bounded syntax
capture and static output. These finite suites do not prove correctness for
arbitrary CommonMark, GFM, or MDX input.

A subsequent [line-comment comparison](benchmarks/line-comments-oracle/README.md)
also exposes a pre-existing gap: reference definitions directly inside list
items are not yet collected globally. Root and block-quote definitions work in
the covered cases.

The [repeatable spec audit](benchmarks/compatibility-audit/README.md) and the
[106-case live oracle](benchmarks/compatibility-audit/CMARK.md) pass their failure
gates. The correction report records the short performance check; the full
six-engine comparison below remains a historical pre-correction measurement.

## Native engine comparison

On an Apple M1 Pro, the v2 core **before the correctness fixes** led the geometric
mean of the 57-document mix in this native Markdown→HTML comparison. This is a
historical measurement, not a rerun of the corrected core. The corpus spans **37–113,609 UTF-8
bytes**: comments, project documentation, and Wikipedia-derived prose.
**Speed relative to v2: higher is faster; v2 = 1.00×.**

| Engine | Fresh, all 57 | Reuse, all 57 | Fresh, 14 agreeing outputs |
| --- | ---: | ---: | ---: |
| Ferromark v2 | 1.00× | 1.00× | 1.00× |
| OX-Content original | 0.97× | 0.97× | 1.00× |
| Ferromark v1 | 0.72× | 0.75× | 0.81× |
| md4c | 0.38× | 0.36× | 0.26× |
| pulldown-cmark | 0.51× | 0.48× | 0.46× |
| Bun native core | 0.23× | 0.20× | 0.17× |

All engines call their native parser and HTML renderer directly; no JavaScript,
WASM, process startup, or file I/O is timed. Bun means its original native
`bun_md` engine, not the full Bun runtime. Reuse retains state where the public
API permits; Bun still uses its owned-output API. Three process rounds with six
rotating measurement windows per round use the same Rust compiler, mimalloc,
and pinned dependency environment. These are configured engine measurements,
not stock release builds or secure product defaults.

**Output differences matter:** the measured OX and v2 builds are byte-identical on all 57 inputs.
Only 14 cases agree across all six engines: ten comments and four plain-prose
views. The other 43 remain in the all-workload diagnostics: 34 differ only in
heading IDs, and nine have additional differences. The GFM lane uses the shared
subset of tables, strikethrough, and task lists; MDX and bare URL autolinking are off.

Selected size/content groups, fresh lifecycle, with the same speed scale:

| Group | N | OX original | V1 | md4c | pulldown | Bun native |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| <512 B | 12 | 0.99× | 0.79× | 0.28× | 0.59× | 0.29× |
| 32–128 KiB | 12 | 0.98× | 0.77× | 0.36× | 0.43× | 0.14× |
| comments | 12 | 1.00× | 0.83× | 0.29× | 0.57× | 0.26× |
| technical-docs | 22 | 0.98× | 0.71× | 0.44× | 0.50× | 0.22× |
| plain-prose | 4 | 1.00× | 0.80× | 0.25× | 0.30× | 0.06× |

This is not a win on every document: V1 is about **11% faster fresh / 16% faster
with reuse on the 310-byte table comment**. V1 and v2 are approximately tied on
reference documents with reuse. The roughly 3% aggregate gap between v2 and OX
is small and almost disappears in the strict-agreement subset.

[Full results and per-document timings](docs/reports/2026-09-14-native-engines/README.md),
[versions and build conditions](docs/reports/2026-09-14-native-engines/PROVENANCE.md),
[HTML differences](docs/reports/2026-09-14-native-engines/OUTPUT-REVIEW.md), and the
[reproducible harness](benchmarks/native-comparison/README.md) include raw windows,
output, source/lock hashes, and all size groups. Small differences and the
overlapping Wikipedia views should not be treated as independent statistical evidence.

The historical [two-engine broad Markdown comparison](docs/reports/2026-09-14-broad-markdown/INTERPRETATION.md)
measures 57 cases from 37 bytes to 114 KB: short comments, real documentation,
and Wikipedia-derived prose. It separates input size, content, output agreement,
and fresh/reused lifecycles. [Full tables and raw data](docs/reports/2026-09-14-broad-markdown/README.md)
and the [earlier synthetic diagnostic comparison](docs/reports/2026-09-13-current-ferromark/README.md)
use the same pinned parser binaries.

The source is MIT licensed; the original copyright notice is preserved in
[LICENSE](LICENSE). CommonMark and GFM specification fixtures carry their own
[CC-BY-SA attribution](crates/ferromark_renderer/tests/spec_fixtures/README.md).
The [benchmark corpus sources](benchmarks/broad-comparison/README.md) retain their
separate MIT, Apache, CC BY, or CC BY-SA licenses and attribution.
