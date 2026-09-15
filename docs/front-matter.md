# Frontmatter extraction

Enable `ParserOptions::front_matter` to extract source-only document metadata.
The option is off by default and in every CommonMark, GFM, and MDX preset.
No additional Cargo feature, renderer setting, or YAML/TOML dependency is needed.

```markdown
---
title: Release notes
draft: true
tags:
  - rust
  - markdown
---

# Release notes
```

`---` indicates YAML-style metadata; `+++` indicates TOML-style metadata:

```markdown
+++
title = "Release notes"
draft = true
+++

# Release notes
```

Both blocks are removed from the rendered Markdown. The application receives
their exact contents, without the delimiter lines:

```rust
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use ferromark::ast::FrontMatterKind;

let source = "---\ntitle: Hello\n---\n# Content\n";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::with_options(
    &allocator,
    source,
    ParserOptions { front_matter: true, ..ParserOptions::gfm() },
).parse().unwrap();

let metadata = document.front_matter.as_ref().unwrap();
assert_eq!(metadata.kind, FrontMatterKind::Yaml);
assert_eq!(metadata.value, "title: Hello\n");
assert_eq!(metadata.content_span.source_text(source), metadata.value);
let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm()).render(&document);
assert_eq!(html, "<h1>Content</h1>\n");
```

## Syntax and boundaries

- Only one block at the original document start is recognized, optionally after
  one UTF-8 BOM. Leading spaces, blank lines, comments, ESM, or other content
  prevent recognition. Enabling line comments does not change that rule.
- Both delimiter lines must start in column one and contain exactly three
  matching `-` or `+` characters. Trailing ASCII spaces and tabs are allowed.
  The opening delimiter requires a line ending; the closing delimiter may end
  at EOF. LF, CRLF, lone CR, and mixed line endings work.
- The first matching delimiter line closes the block. Four-character markers,
  mixed delimiters, indented delimiters, and `...` are not closing delimiters.
  Empty metadata blocks are allowed.
- An unclosed or malformed block remains ordinary Markdown; no source is
  silently discarded. Frontmatter-looking text later in the document or inside
  a list, block quote, code block, or JSX element remains ordinary content.
- Metadata is opaque: Markdown syntax, `//` lines, HTML, MDX-looking strings,
  reference definitions, and footnotes inside it do not affect the body.
- `value` borrows the original source without copying or decoding. Whitespace,
  line endings, and NUL bytes remain unchanged. This differs from Markdown text,
  where CommonMark NUL normalization still applies. A BOM at the start of the
  body after frontmatter remains body content; it is not stripped a second time.

The delimiters identify the intended format, not whether the content is valid
YAML or TOML. Use an application-selected deserializer to interpret values.
JSON frontmatter and automatic metadata injection into HTML are not included.

## AST and compatibility

`Document::front_matter` is an optional arena-owned `FrontMatter` containing
`kind`, `value`, `span`, and `content_span`. `span` includes both delimiter lines
and their line endings, excluding the initial BOM; `content_span` exactly covers
`value`. The document and Markdown body spans refer to the complete original
source, including the skipped prefix when locating body nodes.

`Visit::visit_front_matter` runs before visits to Markdown children. Metadata
is separate from `Document::children`, so standard, borrowed, and hook-based
HTML rendering all omit it. No Markdown `Node` variant is added; the 32-byte
node-size and arena-only ownership guards remain enforced.

The delimiter rules follow the
[pinned v1 tests](https://github.com/sebastian-software/ferromark/blob/143ec2ce151d87d2a3d804a048014afc97733ae0/tests/front_matter_tests.rs).
V2 additionally supports lone CR and consistent leading-BOM recognition through
the regular parser API. Metadata extraction runs before normalization, reference
prepasses, and MDX parsing, so metadata cannot influence those passes.

Implemented at the project owner's request. The public `ParserOptions` and
`Document` structs gain fields in this unpublished development API, changing
exhaustive struct literals. Document `Debug` output includes metadata when present;
existing default HTML, debug, and pretty snapshots remain unchanged. Historical
benchmark reports retain their original API/configuration and results; this
change makes no performance claim.

The [integration tests](../crates/ferromark/tests/front_matter.rs) cover
delimiters, fallbacks, raw values, source spans, metadata isolation, visitors,
and renderer entry points.

## Validation

The workspace passes 763 tests across 54 suites, including 16 frontmatter tests.
Original snapshots and conformance fixtures remain unchanged. Formatting and
Clippy pass with warnings denied, and every workspace benchmark target builds.
The Rust example above also compiles and runs through the public `ferromark`
facade; all local documentation links resolve, and the README feature matrix
renders with seven cells in every row.
