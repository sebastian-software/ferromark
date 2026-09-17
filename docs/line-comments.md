# Source-only line comments

`ParserOptions::line_comments` adds Ferromark v1's `//` comment convention to
Markdown. It is off by default and in every preset, including CommonMark, GFM,
and MDX. It requires no renderer option or Cargo feature.
The [v1 regression cases](https://github.com/sebastian-software/ferromark/blob/143ec2ce151d87d2a3d804a048014afc97733ae0/tests/line_comment_tests.rs)
provide the original behavior reference.

```rust
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

let source = "First line.\n// Editorial note, omitted from HTML.\nSecond line.\n";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::with_options(
    &allocator,
    source,
    ParserOptions { line_comments: true, ..ParserOptions::gfm() },
).parse().unwrap();
let html = HtmlRenderer::with_options(HtmlRendererOptions::gfm_spec()).render(&document);
assert_eq!(html, "<p>First line.\nSecond line.</p>\n");
```

## Recognition and structure

- `//` must start a physical source line, optionally after zero to three ASCII
  spaces. A space after the slashes is optional; `//note` and a bare `//` count.
- The entire comment line is omitted, including its line ending. It introduces
  no blank line, soft break, or hard break of its own. Existing blank lines and
  the line breaks between surviving text lines retain their meaning.
- Tabs and four or more leading spaces do not introduce comments. URLs,
  `Text // trailing text`, and escaped `\//` stay ordinary Markdown.
  A protocol-relative URL on its own line starts with `//` and therefore needs
  escaping or Markdown link syntax when this option is enabled.
- Eligibility is determined before container prefixes are stripped. Thus
  `> // visible`, `- // visible`, and nested explicitly prefixed forms remain
  visible. Unprefixed comment lines between quote or list lines do not split
  those containers or make a tight list loose.
- Comments are transparent to setext heading underlines, table delimiters and
  body rows, and reference definitions. Their text cannot define links or
  footnotes. Table attributes and the other table options can be combined with
  line comments.
- Fenced and indented code blocks and raw HTML blocks preserve their content.
  Block-level captured MDX expressions, ESM, and JSX syntax are not rewritten
  as Markdown comments. This is a physical-line Markdown extension, not a JavaScript
  comment stripper. Like v1, recognition precedes inline parsing, so an eligible
  comment line inside a multiline inline construct is still omitted.
- LF, CRLF, and lone CR line endings are supported. Leading BOM and NUL
  normalization continue to map node spans back to the caller's source.

For example, a comment can sit between a heading and its underline:

```markdown
Release notes
// Confirm this title before publishing.
-------------
```

It becomes a level-two heading. A literal code example stays intact:

````markdown
```js
// This comment belongs to the code example.
run();
```
````

## AST and compatibility

Comments are omitted from the v2 AST and HTML. V1's `BlockEvent::Comment` event
has no v2 counterpart. Surviving nodes retain source spans in the original
document; a node spanning multiple retained lines can enclose omitted comment
lines in its source range. Its text value contains only the retained content.

The span checks also exposed an existing mapping error at the start of a
dedented list continuation: an inline node could incorrectly include the
list's indentation. Inline spans now start at their content; block spans still
include structural indentation. This changes affected source spans without
changing default HTML output.

Implemented at the project owner's request as an optional extension beyond
CommonMark/GFM. Default parsing and existing specification/snapshot expectations
remain unchanged. Adding a public `ParserOptions` field changes exhaustive
struct literals in this unpublished API. Historical benchmark reports retain
their original configurations and results; no performance claim is made here.

The [integration tests](../tests/line_comments.rs)
cover comment syntax, paragraph/container structure, opaque content, references,
tables, line endings, source spans, and renderer entry-point parity.

The [direct v1 comparison](../benchmarks/line-comments-oracle/README.md) records
39 agreeing cases out of 40. The remaining case exposes an existing limitation:
link reference definitions directly inside list items are not collected
globally by v2. The mismatch persists with comments disabled and the comment
line physically removed. Definitions at the document root and inside block
quotes resolve in the covered cases. This separate reference-collection issue
is not fixed by the line-comment extension.

## Validation

- `cargo fmt --all --check`
- `cargo test --workspace --all-features --locked`: 747 tests in 53 suites,
  including 20 line-comment regression tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo bench --workspace --no-run --locked`
- Separate offline compilation of the three active SIMD/optimization workers.
- The Rust example above runs successfully; the rendered README matrix and
  local documentation links were checked.
