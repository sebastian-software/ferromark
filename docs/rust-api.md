# Rust API

## Direct HTML output

```rust
let html = ferromark::to_html("**Hello**")?;
assert_eq!(html, "<p><strong>Hello</strong></p>\n");
```

The convenience functions own a temporary arena and renderer for each document.
The returned `String` is independent of the source and arena. All functions
return `Result` and propagate parser errors, including the nesting limit.

| Function | Output | Options |
| --- | --- | --- |
| `to_html(source)` | New `String` | Rust defaults |
| `to_html_with_options(source, parser, renderer)` | New `String` | Explicit syntax and rendering |
| `to_html_into(source, output)` | Append to `&mut String` | Rust defaults |
| `to_html_into_with_options(source, output, parser, renderer)` | Append to `&mut String` | Explicit syntax and rendering |

Rust defaults match `ParserOptions::default()` and
`HtmlRendererOptions::default()`, including raw HTML passthrough. For untrusted
input, explicitly select `sanitize: true`. Node's default rendering policy
continues to escape raw HTML and filter unsafe URL schemes.

## Syntax and rendering options

```rust
use ferromark::{to_html_with_options, HtmlRendererOptions, ParserOptions};

let html = to_html_with_options(
    "==Important== <b>authored HTML</b>",
    ParserOptions { highlight: true, ..ParserOptions::default() },
    HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
)?;
```

Parser options select the Markdown dialect; renderer options control HTML.
For specification-oriented GFM output, pair `ParserOptions::gfm_spec()` with
`HtmlRendererOptions::gfm()`. CommonMark has a matching `commonmark()` pair.
See [optional writing syntax](optional-writing.md) for marks and inline notes.

## Append to an existing string

```rust
let mut output = String::from("<main>\n");
ferromark::to_html_into("First document.", &mut output)?;
ferromark::to_html_into("Second document.", &mut output)?;
output.push_str("</main>\n");
```

The helpers preserve existing contents and reuse the output string's capacity.
Parsing finishes before any output is appended, so a parse error leaves the
string unchanged. Each call has independent heading, reference, and footnote
state. Each call creates its own parser arena and renderer scratch storage.

## Inspect or transform the document

```rust
use ferromark::{Allocator, HtmlRenderer, Parser};

let source = "Hello, **world**!";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::new(&allocator, source).parse()?;
let html = HtmlRenderer::new().render(&document);
```

The source and allocator must outlive the document. Drop the document before
resetting its allocator. Use AST visitors and `HtmlRenderHooks` when a caller
needs document structure or custom output. Reuse `HtmlRenderer` and reset the
allocator between documents to retain their buffers explicitly.
