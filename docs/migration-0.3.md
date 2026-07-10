# Migrating to ferromark 0.3

ferromark 0.3 adds new integration APIs and removes Cargo features that did not enable supported behavior.

## Remove unused Cargo feature names

The `std`, `neon`, and `trace` Cargo features have been removed:

- `std` did not provide a `no_std` alternative.
- `neon` did not enable Node bindings. The 0.3 Node package uses napi-rs and is distributed through npm instead.
- `trace` did not enable tracing.

If your dependency declaration explicitly enables any of these names, remove them:

```toml
# Before
ferromark = { version = "0.2", features = ["std"] }

# After
ferromark = "0.3"
```

`mdx` remains the only opt-in Cargo feature:

```toml
ferromark = { version = "0.3", features = ["mdx"] }
```

## Render fenced code with an integration hook

Use `to_html_with_renderer` or `to_html_into_with_renderer` when a downstream syntax highlighter should replace fenced-code output:

```rust
use ferromark::{FencedCodeBlock, FencedCodeRenderer, TrustedHtml};

struct Highlighter;

impl FencedCodeRenderer for Highlighter {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        // Return None for unsupported languages to keep ferromark's escaped fallback.
        let _ = block;
        None
    }
}
```

The renderer receives the decoded language word and raw code only for fenced blocks. `TrustedHtml` is emitted verbatim even under `RenderPolicy::Untrusted`; the renderer must escape every untrusted value it embeds. Indented code and a renderer that returns `None` keep the existing safe output.

`BlockEvent::CodeBlockStart` now contains a `CodeBlockKind`, allowing event consumers to distinguish fenced blocks without languages from indented code blocks.
