# MDX integration

Enable the `mdx` feature to segment Markdown, JSX, expressions, and ESM in
trusted content. Ferromark can render the Markdown and package the result as
JSX/TSX without running JavaScript. A downstream toolchain still compiles and
executes the resulting component code.

For the overview and trust boundary, see [MDX support](../README.md#mdx-support).

```bash
cargo add ferromark --features mdx
```

## Render — one call, full output

`render()` assembles the final output automatically: Markdown segments become HTML, JSX and expressions pass through unchanged, ESM and front matter are extracted separately.

```rust
use ferromark::mdx::render;

let input = r#"---
title: Hello
---

import { Card } from './card'

# Hello World

<Card title="Example">

Markdown **inside** a component.

</Card>

{new Date().getFullYear()}
"#;

let output = render(input);
// output.body        — HTML with JSX/expressions passed through
// output.esm         — vec!["import { Card } from './card'\n"]
// output.front_matter — Some("title: Hello\n")
```

Use `render_with_options()` for custom Markdown settings (heading IDs, math, footnotes, etc.).

## Component — ready-to-use JSX module

`to_component()` wraps the output as a complete JSX/TSX module with a named export. Compile that module with your JSX framework and toolchain.

```rust
let output = render(input);
let tsx = output.to_component("HelloWorld")?;
```

```tsx
import { Card } from './card'

export function HelloWorld() {
  return (
    <>
      <h1 id="hello-world">Hello World</h1>
      <Card title="Example">
        <p>Markdown <strong>inside</strong> a component.</p>
      </Card>
      {new Date().getFullYear()}
    </>
  );
}
```

## Segment — low-level control

When you need full control over each block, use `segment()` directly:

```rust
use ferromark::mdx::{segment, Segment};

for seg in segment(input) {
    match seg {
        Segment::Esm(s)              => { /* import/export — pass through */ }
        Segment::Markdown(s)         => { /* parse with ferromark::to_html(s) */ }
        Segment::JsxBlockOpen(s)     => { /* <Component> */ }
        Segment::JsxBlockClose(s)    => { /* </Component> */ }
        Segment::JsxBlockSelfClose(s)=> { /* <Component /> */ }
        Segment::Expression(s)       => { /* {expression} */ }
    }
}
```

The segmenter handles JSX attribute parsing (strings, expressions, spreads), brace-depth tracking (with string/comment/template-literal awareness), fragment syntax, member expressions (`<Foo.Bar>`), and multiline tags. ESM continuation only crosses lines when lexical structure or an import/export clause requires it, so a complete semicolonless declaration cannot consume following Markdown prose. Invalid or ambiguous constructs fall back to Markdown. Use the strict APIs below when malformed structure must produce diagnostics.

For source locations, `segment_spanned()` returns the same zero-copy segments
with contiguous [`Range`](https://docs.rs/ferromark/latest/ferromark/struct.Range.html)
values into the original UTF-8 input. A range covers the exact segment text,
including delimiters and a trailing newline when it belongs to that segment.

`segment()` remains deliberately permissive: malformed MDX falls back to a
Markdown segment. For content pipelines that must reject malformed structure,
use `segment_strict()`. It reports typed diagnostics with byte ranges; convert
an offset to a one-based line and Unicode column only when presenting it to a
user with `source_location()`.

```rust
use ferromark::mdx::{segment_strict, source_location};

let input = "<Card bad=>\n";
let diagnostics = segment_strict(input).unwrap_err();
let location = source_location(input, diagnostics[0].primary_range.start_usize());
assert_eq!((location.line, location.column), (1, 1));
```

Strict mode checks MDX structure (flow-expression delimiters, JSX tag shape and
nesting, ESM placement, and ambiguous/incomplete ESM continuations). It intentionally does not parse or type-check
JavaScript or TypeScript inside an otherwise well-delimited ESM block or
expression.

Compiler and localization consumers can opt into a flat semantic event buffer
without going through HTML. `parse_events()` composes the existing MDX
segmenter, block parser, and MDX-aware inline parser; ranges in the returned
events point into the original input.

```rust
use ferromark::InlineEvent;
use ferromark::mdx::{MdxEvent, parse_events};

let input = "# Hello {name}\n";
let stream = parse_events(input);
let prose = stream.events.iter().filter_map(|event| match event {
    MdxEvent::Inline(InlineEvent::Text(range)) => {
        Some(range.slice_str(input.as_bytes()).unwrap())
    }
    _ => None,
}).collect::<Vec<_>>();

assert_eq!(prose, vec!["Hello "]);
```

The event path is fully opt-in and does not alter the normal Markdown or MDX
HTML renderer. `parse_events_strict()` applies the same structural diagnostics
as `segment_strict()` before producing events. Its strict validation covers
flow constructs; malformed inline MDX retains the documented text fallback.
Inside blockquotes and list items, a paragraph containing only one JSX tag or
expression is promoted to the corresponding flow event. Mixed prose remains
inline MDX, and the surrounding Markdown container events stay balanced.

Full example: `cargo run --features mdx --example mdx_segment`

<details>
<summary><strong>Scope and coverage</strong></summary>

<br>

The segmenter covers the block-level MDX patterns a typical Docusaurus, Next.js, or Astro page is built from: imports at the top, components wrapping content, expressions between paragraphs. [`tests/mdx_segment_tests.rs`](../tests/mdx_segment_tests.rs) exercises that supported set.

Check these boundaries against your content before integrating:

| What | Our approach | When it matters |
|---|---|---|
| **Inline JSX** (`text <em>here</em>`) | Stays in `segment()` Markdown blocks; `parse_events()` and `InlineParser::parse_mdx()` expose typed MDX inline events | Use the opt-in event APIs when a downstream consumer must distinguish prose and components |
| **JS validation** | Heuristic detection (keyword + brace counting) instead of acorn/swc | Only if you need to report syntax errors in user-authored MDX at parse time |
| **Markdown grammar** | Standard CommonMark/GFM rules | Official mdxjs disables indented code and HTML syntax — relevant if your content relies on `<div>` being JSX, not HTML |
| **Container nesting** | `> <Component>` stays Markdown to the renderer; `parse_events()` promotes tag-only or expression-only container paragraphs to semantic flow events | Rendering-level container MDX, multiline constructs across prefixes, and container-local ESM remain out of scope |
| **TypeScript generics** | `<Component<T>>` not parsed | Only relevant for TSX-heavy content pages |
| **Error reporting** | Permissive fallback by default; opt-in structural diagnostics with `segment_strict()` | Use strict mode when broken MDX must fail a content pipeline |

The full `@mdx-js/mdx` compiler exists to produce a React component tree from MDX. It needs a JavaScript parser because it compiles to JSX. Ferromark's segmenter exists to answer a simpler question: *where does the Markdown stop and the JSX start?* That question doesn't need a JS runtime.

For the detailed technical spec, see [`src/mdx/mod.rs`](../src/mdx/mod.rs).

</details>

