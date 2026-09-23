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
Both types share the three names `commonmark()`, `gfm_spec()` and `gfm()`, so a
pair is built from one name; `mdx()` is parser-only, because the renderer handles
MDX nodes without a profile of its own:

| Profile | Parser | Renderer |
| --- | --- | --- |
| `commonmark()` | Strict CommonMark | Strict CommonMark, no product conveniences |
| `gfm_spec()` | GFM without semantic footnotes | `commonmark()` plus the GFM tag filter |
| `gfm()` | GFM plus footnotes | `new()` plus the GFM tag filter |
| `mdx()` | MDX, GFM off | — |

Pair `ParserOptions::gfm_spec()` with `HtmlRendererOptions::gfm_spec()` for
specification-oriented GFM output, and `gfm()` with `gfm()` for the convenience
profile that keeps heading IDs, callouts, URL autolinking,
link targets, and fence metadata cleanup. `commonmark()` pairs the same way.
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
`Default::default()`, the `commonmark()`/`gfm()`/`gfm_spec()` profiles, and
cloning any of them perform no heap allocation. Building a renderer per document
from such a value therefore costs nothing for its configuration. Empty values keep their
meaning: an empty `base_url` is used as configured and prefixes nothing, and an empty
`autolink_patterns` list disables auto-linking rather than restoring the
defaults. See the
[decision record](decisions/2026-09-15-borrowed-renderer-options.md).
A root-absolute link stays root-absolute under an empty base, as recorded in the
[renderer review fixes](decisions/2026-09-21-renderer-fixes.md).

### Heading levels

`heading_level_offset` shifts the HTML heading level without changing the
Markdown AST. Positive values move toward `h6`, negative values move toward
`h1`, and results clamp to the valid `h1`–`h6` range. The default is `0`.
`map_heading_level` exposes this same mapping for consumers that produce
heading metadata; generated IDs and fragment links remain based on the original
heading text.

```rust
use ferromark::{HtmlRenderer, map_heading_level};

let _renderer = HtmlRenderer::new().with_heading_level_offset(1);
assert_eq!(map_heading_level(1, 1), 2);
```

This behavior is recorded in the
[heading offset decision](decisions/2026-09-23-heading-level-offset.md).

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

## Heading IDs

The renderer assigns heading IDs in document order. It keeps the first requested
ID and suffixes later collisions with `-1`, `-2`, and so on, skipping suffixes
already used by another heading. Explicit `{#id}` values follow the same rule.
`HeadingIdPlanner` exposes that assignment rule to Rust tools that produce
heading metadata or links alongside Ferromark output.

## Incremental fragments

A streaming caller renders one document in committed pieces. `HtmlRenderer`
carries the state that has to survive those pieces — generated heading IDs and
semantic footnote numbering and slugs — so a later fragment continues the
document instead of restarting at the first ID:

```rust
use ferromark::{Allocator, HtmlRenderer, Parser};

let mut renderer = HtmlRenderer::new();
let mut page = String::new();
for chunk in ["# Notes\n", "More text.\n"] {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, chunk).parse()?;
    page.push_str(&renderer.render_incremental_fragment(&document));
}
renderer.reset_incremental_state();
```

- `render_incremental_fragment` renders a committed fragment and keeps the
  cross-fragment heading-ID and footnote state, so repeated headings and
  colliding footnote labels keep getting unique anchors across the whole stream.
  For a single document the output matches `render` with the same options.
- `render_incremental_fragment_with_hooks` is the same entry point through
  `HtmlRenderHooks`.
- `render_provisional_fragment` renders an unstable fragment that the next
  streaming update may replace. It leaves the committed heading-ID and footnote
  state untouched, so a provisional render never claims an ID or a footnote
  number that the committed render then has to skip.
- `render_provisional_fragment_with_hooks` is the hooked provisional entry
  point.
- `reset_incremental_state` clears the carried state so the next document starts
  from a clean renderer. Caches derived from the immutable options, such as the
  autolink index, are kept.

Renderer options apply to each fragment as well. In particular,
`heading_level_offset` stays in effect across committed, provisional, hooked,
and post-reset fragment renders.

These entry points are deliberately separate from `render`, which resets that
state for every document and keeps its exact one-shot setup cost. Each fragment
is parsed on its own, in an arena that outlives the render call, and the
fragments are concatenated by the caller.

## The arena is single-threaded, and bumpalo is public

`Allocator` wraps and dereferences to `bumpalo::Bump`, which it also re-exports,
and `allocator::Box`, `allocator::Vec` and `allocator::String` are bumpalo
collections or thin wrappers around them. bumpalo is therefore part of this
crate's public API: a bumpalo major release is a ferromark major release. No
other dependency leaks into the public surface.

`Document` and `Node` are `!Send` and `!Sync`, and `Allocator` is `!Sync`. An
arena and the AST that lives in it stay on the thread that created them; parse
and render per thread, and move the rendered `String` — which owns nothing in
the arena — across threads instead. See the
[decision record](decisions/2026-09-17-bumpalo-public-api.md).

## What the 2.0.0 API freeze covers

From 2.0.0 the public Rust API follows semver, under the rules recorded in the
[API surface decision](decisions/2026-09-17-api-surface.md):

- `ParseErrorKind` is `#[non_exhaustive]`. Match it with a wildcard arm; new
  error categories arrive in minor releases.
- `Node` and its companion enums stay exhaustive, so a `match` over the AST is
  checked by the compiler. A new AST node kind is a major release.
- `ParserOptions` and `HtmlRendererOptions` stay exhaustive structs, so
  `..Default::default()` keeps working. A new option field is a major release.
- Build options with struct-update syntax rather than listing every field:

  ```rust
  use ferromark::HtmlRendererOptions;

  let options = HtmlRendererOptions {
      sanitize: true,
      ..HtmlRendererOptions::gfm_spec()
  };
  ```

  A field-by-field literal stops compiling when a field is added or removed;
  the struct-update form survives a removal and needs no edit for an addition.
