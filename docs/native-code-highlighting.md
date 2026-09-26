# Native code highlighting seam

Ferromark can accept highlighted code from a Rust render hook without changing
its parser or default renderer. This is the Ferromark side of [issue #393]. A
Ferriki adapter requires Ferriki's reusable, N-API-free Rust library API, which
is tracked in [Ferriki #125] and [Ferriki #126]. The existing `ferriki-core`
package is private and its errors still use N-API types, so this seam does not
add a Ferriki dependency or claim to complete that integration.

[issue #393]: https://github.com/sebastian-software/ferromark/issues/393
[Ferriki #125]: https://github.com/sebastian-software/ferriki/issues/125
[Ferriki #126]: https://github.com/sebastian-software/ferriki/issues/126

## Contract

Implement `HtmlRenderHooks::highlight_code_block` and render with
`HtmlRenderer::render_with_hooks`. The hook receives:

- `code`: the displayed code after Ferromark removes enabled VitePress inline
  annotation directives;
- `language`: the normalized fence language used for the HTML class;
- `raw_language` and `raw_meta`: the original fence fields, so metadata is not
  silently conflated with the language.

The hook returns `HighlightedCodeBlock { lines }`, with one independently
tag-balanced HTML fragment for each `code.split('\n')` line, including a final
empty line when the code ends in a newline. Ferriki's token lines are a natural
input for these fragments. Ferromark inserts them into its own `<pre><code>`
structure and keeps titles, line numbers, links, annotations, source spans and
classes. The hook may return `None` for an unknown language or recoverable
highlighter error. Ferromark then escapes and renders plain code. A result with
the wrong line count also falls back to plain code.

Fragments are **trusted HTML**. The adapter must escape every source token and
attribute it emits, and must not return `<pre>`, `<code>`, or tags spanning line
boundaries. Ferromark continues to escape its own metadata and attributes. A
future adapter should map Ferriki's typed errors to a documented fallback
policy, rather than silently treating all failures as unknown languages.

The older `render_node` hook still runs first. Its `Handled` result retains the
whole-block replacement contract used by the Node callback renderer. The new
method is called only when `render_node` returns `Default`; ordinary
`HtmlRenderer::render` never dispatches hooks or loads highlighting assets.

## Rust usage

```rust
use ferromark::{CodeHighlightInput, HighlightedCodeBlock, HtmlRenderHooks};

struct CodeHighlighter;

impl HtmlRenderHooks for CodeHighlighter {
    fn highlight_code_block(&mut self, input: CodeHighlightInput<'_>) -> Option<HighlightedCodeBlock> {
        // A real adapter asks its reusable highlighter for token lines and
        // escapes each token before constructing the per-line HTML.
        let _ = input;
        None
    }
}
```

This hook remains usable with the same highlighter instance across blocks and
documents. The adapter can own that instance or borrow it for a render. The
remaining Ferriki work includes a portable asset provider, a publishable Rust
API, and a compiled adapter example; see [Ferriki #124], [Ferriki #125], and
[Ferriki #126].

[Ferriki #124]: https://github.com/sebastian-software/ferriki/issues/124
