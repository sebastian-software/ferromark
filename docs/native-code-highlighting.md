# Native code highlighting seam

Ferromark can accept highlighted code from a Rust render hook without changing
its parser or default renderer. This is the Ferromark side of [issue #393].
[Ferriki PR #128] supplies the reusable, N-API-free Rust highlighter and a
[reference adapter] using this hook. The default Ferromark crate has no Ferriki
dependency.

[issue #393]: https://github.com/sebastian-software/ferromark/issues/393
[Ferriki PR #128]: https://github.com/sebastian-software/ferriki/pull/128
[reference adapter]: https://github.com/sebastian-software/ferriki/blob/main/docs/rust-api.md#ferromark-adapter-contract

## Contract

Implement `HtmlRenderHooks::highlight_code_block` and render with
`HtmlRenderer::render_with_hooks`. The hook receives:

- `code`: the displayed code after Ferromark removes enabled VitePress inline
  annotation directives;
- `language`: the normalized fence language used for the HTML class;
- `raw_language` and `raw_meta`: the original fence fields, so metadata is not
  silently conflated with the language.

The hook returns `HighlightedCodeBlock::new(lines)`, with one independently
tag-balanced HTML fragment for each `code.split('\n')` line, including a final
empty line when the code ends in a newline. `with_colors(foreground,
background)` carries theme colors onto the outer `<pre>` style attribute.
Ferromark keeps titles, line numbers, links, annotations, source spans and
classes. The hook may return `None` for an unknown language or recoverable
highlighter error. Ferromark then escapes and renders plain code. A result with
the wrong line count or a carriage return or newline inside one fragment also
falls back to plain code.

Fragments are **trusted HTML**. The adapter must escape every source token and
attribute it emits, and must not return `<pre>`, `<code>`, or tags spanning line
boundaries. Ferromark escapes its own metadata, attributes, and supplied theme
colors. The [reference adapter] maps Ferriki failures to plain-code fallback;
applications can record non-language errors before returning `None`.

The older `render_node` hook still runs first. Its `Handled` result retains the
whole-block replacement contract used by the Node callback renderer. The new
method is called only when `render_node` returns `Default`; ordinary
`HtmlRenderer::render` never dispatches hooks or loads highlighting assets.

## Rust usage

The [reference adapter] uses a reusable Ferriki `Highlighter`, calls
`highlight_html_lines`, and passes its escaped line fragments and theme colors
to `HighlightedCodeBlock`. It lowercases the normalized fence language before
lookup. The adapter can own the highlighter or borrow it for a render.
