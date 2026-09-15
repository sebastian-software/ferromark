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

### String-valued renderer options

`soft_break`, `hard_break`, `base_url`, `source_path`, and
`code_annotation_meta_key` are `Cow<'static, str>`, and `autolink_patterns` is
`Cow<'static, [Cow<'static, str>]>`. Assign with `.into()`: a `&'static str`
borrows and a `String` moves in.

```rust
use ferromark::{to_html_with_options, HtmlRendererOptions, ParserOptions};

let base_from_config = String::from("/docs/");
let html = to_html_with_options(
    "[guide](/guide/setup.md) and https://example.com",
    ParserOptions::default(),
    HtmlRendererOptions {
        convert_md_links: true,
        base_url: base_from_config.into(),
        hard_break: "<br />\n".into(),
        autolink_patterns: vec!["https://".into(), "mailto:".into()].into(),
        ..HtmlRendererOptions::default()
    },
)?;
assert!(html.contains("/docs/guide/setup/index.html"));
```

Every documented default is static data, so `HtmlRendererOptions::new()`,
`Default::default()`, the `commonmark()`/`gfm()` profiles, and cloning any of
them perform no heap allocation. Building a renderer per document from such a
value therefore costs nothing for its configuration. Empty values keep their
meaning: an empty `base_url` is not the default `"/"`, and an empty
`autolink_patterns` list disables auto-linking rather than restoring the
defaults. See the
[decision record](decisions/2026-09-15-borrowed-renderer-options.md).

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
