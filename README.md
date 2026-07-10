# ferromark

[![Powered by Sebastian Software](https://img.shields.io/badge/Powered%20by-Sebastian%20Software-00718d?style=flat-square)](https://oss.sebastian-software.com)
[![CI](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ferromark.svg)](https://crates.io/crates/ferromark)
[![docs.rs](https://docs.rs/ferromark/badge.svg)](https://docs.rs/ferromark)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![clippy](https://img.shields.io/badge/clippy--strict-passing-brightgreen.svg)](https://doc.rust-lang.org/clippy/)

Markdown to HTML at 280 MiB/s. Faster than pulldown-cmark, md4c (C), and comrak. Passes all 652 CommonMark spec tests. Every GFM extension included.

## Quick start

```rust
let html = ferromark::to_html("# Hello\n\n**World**");
```

One function call, no setup. When allocation pressure matters:

```rust
let mut buffer = Vec::new();
ferromark::to_html_into("# Reuse me", &mut buffer);
// buffer survives across calls — zero repeated allocation
```

## Benchmarks

Numbers, not adjectives. Apple Silicon (M-series), July 2026. All parsers run with GFM tables, strikethrough, and task lists enabled; ferromark's non-GFM extras (heading IDs, callouts) are disabled so every parser performs the same work. Output buffers reused where APIs allow. Non-PGO binaries for a fair comparison.

**CommonMark 5 KB** (wiki-style, mixed content with tables)
| Parser | Throughput | vs ferromark |
|--------|----------:|------------:|
| **ferromark** | **259.6 MiB/s** | **baseline** |
| pulldown-cmark | 254.5 MiB/s | 0.98x |
| md4c (C) | 243.1 MiB/s | 0.94x |
| comrak | 67.9 MiB/s | 0.26x |

**CommonMark 50 KB** (same style, scaled)
| Parser | Throughput | vs ferromark |
|--------|----------:|------------:|
| **ferromark** | **280.5 MiB/s** | **baseline** |
| pulldown-cmark | 275.2 MiB/s | 0.98x |
| md4c (C) | 253.3 MiB/s | 0.90x |
| comrak | 71.8 MiB/s | 0.26x |

2% faster than pulldown-cmark. 11% faster than md4c. 4x faster than comrak. Competitor versions: pulldown-cmark 0.13.4, comrak 0.53, md4c @ 65c6c9d.

The fixtures are synthetic wiki-style documents with paragraphs, lists, code blocks, and tables. Nothing cherry-picked. Run them yourself: `cargo bench --bench comparison`

## What you get

**Full CommonMark**: 652/652 spec tests pass. No filtering, no exceptions.

**All five GFM extensions**: Tables, strikethrough, task lists, autolink literals, disallowed raw HTML.

**Beyond GFM**: Footnotes, front matter extraction (`---`/`+++`), heading IDs (GitHub-compatible slugs), math spans (`$`/`$$`), highlight/mark syntax (`==text==`), superscript (`^text^`), subscript (`~text~`), and callouts (`> [!NOTE]`, `> [!WARNING]`, ...).

**MDX support** (opt-in via `mdx` feature): Segment and render `.mdx` files without a JavaScript toolchain. Covers 90%+ of real-world MDX patterns in Next.js, Docusaurus, and Astro.

15 feature flags to turn on exactly what you need:

```text
allow_html · allow_link_refs · tables · strikethrough · highlight · superscript · subscript · task_lists
autolink_literals · disallowed_raw_html · footnotes · front_matter
heading_ids · math · callouts
```

Syntax note: ferromark uses `~~text~~` for strikethrough, `~text~` for subscript, and `^text^` for superscript. Single-tilde strikethrough is intentionally not supported.

## Trade-offs

ferromark is built for one job: turning Markdown into HTML as fast as possible. That focus means some things it deliberately skips:

- **No AST access.** You can't walk a syntax tree or write custom renderers against parsed nodes. If you need that, pulldown-cmark's iterator model or comrak's AST are better fits.
- **No source maps.** No byte-offset tracking for mapping HTML back to Markdown positions.
- **HTML only.** No XML, no CommonMark round-tripping, no alternative output formats.

These aren't planned. They'd compromise the streaming architecture that makes ferromark fast.

## Rendering untrusted Markdown

The default `RenderPolicy::Untrusted` is the browser-facing safety boundary. It escapes all raw HTML and allows relative URLs plus a small set of non-script schemes (`http`, `https`, `mailto`, `tel`, and similar). URL schemes are checked after entity and control-character normalization, so spellings such as `javas&#99;ript:` are blocked too.

```rust
let html = ferromark::to_html(user_supplied_markdown);
```

Trusted documents and MDX can opt into passthrough explicitly:

```rust
use ferromark::{Options, RenderPolicy};

let options = Options {
    render_policy: RenderPolicy::Trusted,
    ..Options::default()
};
let html = ferromark::to_html_with_options(trusted_markdown, &options);
```

`disallowed_raw_html` implements the narrower GFM tag filter in trusted mode. It is not a general-purpose HTML sanitizer and does not make arbitrary raw HTML safe by itself.

## MDX support

MDX is the standard for component-driven docs in Next.js, Docusaurus, and Astro. Processing it usually requires a full JavaScript toolchain — Node.js, acorn, babel, the works.

ferromark takes a different approach: segment `.mdx` files into typed blocks and render them at native speed. No JS runtime. No AST.

```toml
ferromark = { version = "0.1", features = ["mdx"] }
```

### Render — one call, full output

`render()` assembles the final output automatically: Markdown segments become HTML, JSX and expressions pass through unchanged, ESM and front matter are extracted separately.

```rust
use ferromark::mdx::render;

let input = r#"import { Card } from './card'

---
title: Hello
---

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

### Component — ready-to-use JSX module

`to_component()` wraps the output as a complete JSX/TSX module with a named export. Works with React 19, Preact, Solid, and any JSX framework.

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

### Segment — low-level control

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

The segmenter handles JSX attribute parsing (strings, expressions, spreads), brace-depth tracking (with string/comment/template-literal awareness), fragment syntax, member expressions (`<Foo.Bar>`), and multiline tags. Invalid constructs fall back to Markdown — no panics, always valid output.

Full example: `cargo run --features mdx --example mdx_segment`

<details>
<summary><strong>Scope and coverage</strong></summary>

<br>

The segmenter covers the block-level MDX patterns that make up 90%+ of real-world `.mdx` files: imports at the top, components wrapping content, expressions between paragraphs. This is what a typical Docusaurus, Next.js, or Astro page looks like — and it works out of the box.

What the segmenter deliberately skips — and why that's fine for most use cases:

| What | Our approach | When it matters |
|---|---|---|
| **Inline JSX** (`text <em>here</em>`) | Stays inside Markdown segments | Only if you mix JSX and prose on the same line inside a paragraph — rare in practice |
| **JS validation** | Heuristic detection (keyword + brace counting) instead of acorn/swc | Only if you need to report syntax errors in user-authored MDX at parse time |
| **Markdown grammar** | Standard CommonMark/GFM rules | Official mdxjs disables indented code and HTML syntax — relevant if your content relies on `<div>` being JSX, not HTML |
| **Container nesting** | `> <Component>` stays Markdown | Only if you put JSX inside blockquotes or list items — uncommon |
| **TypeScript generics** | `<Component<T>>` not parsed | Only relevant for TSX-heavy content pages — very rare in docs |
| **Error reporting** | Silent fallback to Markdown | Means broken JSX renders as text instead of failing — arguably safer for content pipelines |

The full `@mdx-js/mdx` compiler exists to produce a React component tree from MDX. It needs a JavaScript parser because it compiles to JSX. ferromark's segmenter exists to answer a simpler question: *where does the Markdown stop and the JSX start?* That question doesn't need a JS runtime.

For the detailed technical spec, see `src/mdx/mod.rs`.

</details>

## How it works

No AST. Block events stream from the scanner to the HTML writer with nothing in between.

```
Input bytes (&[u8])
       │
       ▼
   Block parser (line-oriented, memchr-driven)
       │ emits BlockEvent stream
       ▼
   Inline parser (mark collection → resolution → emit)
       │ emits InlineEvent stream
       ▼
   HTML writer (direct buffer writes)
       │
       ▼
   Output (Vec<u8>)
```

What makes this fast in practice:

- **Block scanning** runs on `memchr` for line boundaries. Container state is a compact stack, not a tree.
- **Inline parsing** has three phases: collect delimiter marks, resolve precedence (code spans, math, links, emphasis, strikethrough, subscript, superscript, highlight), emit. No backtracking.
- **Emphasis resolution** uses the CommonMark modulo-3 rule with a delimiter stack instead of expensive rescans.
- **SIMD scanning** (NEON on ARM) detects special characters in inline content.
- **Zero-copy references**: events carry `Range` pointers into the input, not copied strings.
- **Compact events**: 24 bytes each, cache-line friendly.
- **Hot/cold annotation**: `#[inline]` on tight loops, `#[cold]` on error paths, table-driven byte classification.

### Design principles

- **Linear time.** No regex, no backtracking, no quadratic blowup on adversarial input.
- **Low allocation pressure.** Compact events, range references, reusable output buffers.
- **Operational safety.** Enforced limits cap block nesting (32), inline marks (4,096), code-span backtick runs (32), link-destination parenthesis depth (32), ordered-list marker digits (9), and table columns (128). Footnote numbering has no arbitrary count cap; its definition-index lookup stays O(1) per reference.
- **Small dependency surface.** Minimal crates, straightforward integration.

<details>
<summary><strong>Detailed parser comparison</strong></summary>

<br>

How ferromark compares to the other three top-tier parsers across architecture, features, and output. Ratings use a 4-level heatmap focused on end-to-end Markdown-to-HTML throughput. Scoring is relative per row, so each row has at least one top mark.

Legend: 🟩 strongest &nbsp; 🟨 close behind &nbsp; 🟧 notable tradeoffs &nbsp; 🟥 weakest

Ferromark optimization backlog: [docs/arch/ARCH-PLAN-001-performance-opportunities.md](docs/arch/ARCH-PLAN-001-performance-opportunities.md)

<table>
  <thead>
    <tr>
      <th>Feature</th>
      <th>ferromark</th>
      <th>md4c</th>
      <th>pulldown-cmark</th>
      <th>comrak</th>
    </tr>
  </thead>
  <tbody>
    <tr><td colspan="5"><b>Performance-critical architecture and memory</b></td></tr>
    <tr>
      <td><b>Parser model (streaming, no AST)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Streaming parsers emit output as they scan, avoiding intermediate trees. ferromark and md4c stream directly; pulldown-cmark uses a pull iterator; comrak builds an AST.</small></td></tr>
    <tr>
      <td><b>API overhead profile</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Measures overhead on straight Markdown-to-HTML throughput. md4c callbacks and ferromark streaming events are lean; pulldown-cmark pull iterators are close; comrak's AST model adds more overhead for this workload.</small></td></tr>
    <tr>
      <td><b>Parse/render separation</b></td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
    </tr>
    <tr><td colspan="5"><small>Clear separation lets renderers be swapped or tuned. md4c and pulldown-cmark separate parse and render clearly; ferromark is mostly separated; comrak leans on AST-based renderers.</small></td></tr>
    <tr>
      <td><b>Inline parsing pipeline</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Multi-phase inline parsing (collect, resolve, emit) keeps the hot path linear. ferromark uses this approach; md4c and pulldown-cmark are optimized byte scanners; comrak does more AST bookkeeping.</small></td></tr>
    <tr>
      <td><b>Emphasis matching efficiency</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Stack-based algorithms reduce rescans on text-heavy documents. ferromark uses modulo-3 stacks; md4c and pulldown-cmark are optimized; comrak pays AST overhead.</small></td></tr>
    <tr>
      <td><b>Link reference processing cost</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>Link labels need normalization. ferromark, md4c, and pulldown-cmark minimize allocations; comrak handles more feature paths.</small></td></tr>
    <tr>
      <td><b>Zero-copy text handling</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Text slices that point directly into input reduce allocation and copy costs. ferromark uses ranges; md4c and pulldown-cmark borrow slices; comrak allocates AST nodes.</small></td></tr>
    <tr>
      <td><b>Allocation pressure (hot path)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Fewer allocations in tight loops means better CPU utilization. Streaming parsers allocate less during parse/render; AST parsers allocate many nodes.</small></td></tr>
    <tr>
      <td><b>Output buffer reuse</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Reusing buffers avoids repeated allocations across runs. ferromark, md4c, and pulldown-cmark allow reuse; comrak allocates internally.</small></td></tr>
    <tr>
      <td><b>Memory locality</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>A small working set fits in cache. Streaming parsers keep it small; AST-based parsing expands it.</small></td></tr>
    <tr>
      <td><b>Cache friendliness</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Linear scans and contiguous buffers work well for CPU caches. ferromark and md4c favor linear scans; pulldown-cmark is close; comrak traverses AST allocations.</small></td></tr>
    <tr>
      <td><b>SIMD availability</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>SIMD accelerates scanning for special characters. ferromark and pulldown-cmark have SIMD paths; md4c relies on C compiler optimizations; comrak is not SIMD-focused.</small></td></tr>
    <tr>
      <td><b>Hot-path control</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Performance headroom from low-level control in inner loops. md4c (C) and ferromark use tighter tuning; pulldown-cmark is mostly safe-Rust hot loops; comrak prioritizes flexibility.</small></td></tr>
    <tr>
      <td><b>Dependency footprint</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Fewer dependencies simplify builds. md4c and ferromark are minimal; pulldown-cmark is moderate; comrak is heavier.</small></td></tr>
    <tr>
      <td><b>Throughput ceiling (architectural)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Streaming architectures with fewer allocations generally allow higher throughput ceilings. ferromark and md4c lead; pulldown-cmark is close; comrak trades throughput for flexibility.</small></td></tr>
    <tr><td colspan="5">&nbsp;</td></tr>
    <tr><td colspan="5"><b>Feature coverage and extensibility</b></td></tr>
    <tr>
      <td><b>Extension breadth</b></td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>comrak has the broadest catalog; ferromark implements all 5 GFM extensions plus footnotes, front matter, heading IDs, math, highlight, subscript, superscript, and callouts; pulldown-cmark supports common GFM features; md4c supports common GFM features.</small></td></tr>
    <tr>
      <td><b>Spec compliance (CommonMark)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>All four target CommonMark. Beyond CommonMark and GFM, ferromark, pulldown-cmark, and comrak also support footnotes, heading IDs, math spans, and callouts.</small></td></tr>
    <tr>
      <td><b>Extension configuration surface</b></td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>Fine-grained flags let you disable features to reduce work. md4c has many flags; ferromark has 15 options; pulldown-cmark and comrak use option structs.</small></td></tr>
    <tr>
      <td><b>Raw HTML control</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>md4c and comrak expose explicit switches; ferromark provides <code>allow_html</code> and <code>disallowed_raw_html</code>; pulldown-cmark is more fixed.</small></td></tr>
    <tr>
      <td><b>GFM tables</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>All four support GFM tables.</small></td></tr>
    <tr>
      <td><b>Task lists, strikethrough</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>All four support both.</small></td></tr>
    <tr>
      <td><b>Footnotes</b></td>
      <td align="center">🟩</td>
      <td align="center">🟥</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>ferromark, pulldown-cmark, and comrak support footnotes; md4c does not.</small></td></tr>
    <tr>
      <td><b>Permissive autolinks</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>ferromark and md4c support GFM autolink literals (URL, www, email); comrak has relaxed autolinks; pulldown-cmark focuses on spec defaults.</small></td></tr>
    <tr>
      <td><b>Output safety toggles</b></td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>md4c and comrak provide explicit unsafe/escape switches; ferromark provides <code>allow_html</code> and <code>disallowed_raw_html</code>; pulldown-cmark is more fixed.</small></td></tr>
    <tr><td colspan="5">&nbsp;</td></tr>
    <tr><td colspan="5"><b>Rendering and output</b></td></tr>
    <tr>
      <td><b>Output streaming</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Incremental output lowers peak memory and removes extra passes. ferromark and md4c stream to buffers; pulldown-cmark streams events; comrak renders after AST work.</small></td></tr>
    <tr>
      <td><b>Output customization hooks</b></td>
      <td align="center">🟧</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>Callbacks and ASTs are great for custom rendering but add indirection. md4c callbacks and comrak AST are very flexible; pulldown-cmark iterators are easy to transform; ferromark is lower level.</small></td></tr>
    <tr>
      <td><b>Output formats</b></td>
      <td align="center">🟥</td>
      <td align="center">🟧</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>comrak emits HTML, XML, and CommonMark; pulldown-cmark provides HTML plus event streams; md4c has HTML and callbacks; ferromark targets HTML only.</small></td></tr>
    <tr>
      <td><b>Source position support</b></td>
      <td align="center">🟥</td>
      <td align="center">🟥</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>pulldown-cmark has strong source map support; comrak can emit source positions; ferromark and md4c skip this for speed.</small></td></tr>
    <tr>
      <td><b>Source map tooling</b></td>
      <td align="center">🟥</td>
      <td align="center">🟥</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>pulldown-cmark exposes event ranges; comrak can emit source position attributes; ferromark and md4c keep this minimal.</small></td></tr>
    <tr>
      <td><b>IO friendliness</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>md4c and ferromark stream into buffers; pulldown-cmark recommends buffered output; comrak often builds strings after AST work.</small></td></tr>
  </tbody>
</table>

</details>

## Building

```bash
cargo build            # development
cargo build --release  # optimized (recommended for benchmarks)
cargo test             # run tests
cargo test --test commonmark_spec -- --nocapture  # CommonMark spec
cargo bench            # benchmarks
```

## Project structure

```
src/
├── lib.rs          # Public API (to_html, to_html_into, parse, Options)
├── main.rs         # CLI binary
├── block/          # Block-level parser
│   ├── parser.rs   # Line-oriented block parsing
│   └── event.rs    # BlockEvent types
├── inline/         # Inline-level parser
│   ├── mod.rs      # Three-phase inline parsing
│   ├── marks.rs    # Mark collection + SIMD integration
│   ├── simd.rs     # NEON SIMD character scanning
│   ├── event.rs    # InlineEvent types
│   ├── code_span.rs
│   ├── emphasis.rs      # Modulo-3 stack optimization
│   ├── strikethrough.rs # GFM strikethrough resolution
│   ├── subscript.rs     # Subscript resolution (~text~)
│   ├── superscript.rs   # Superscript resolution (^text^)
│   ├── math.rs          # Math span resolution ($/$$ delimiters)
│   └── links.rs         # Link/image/autolink parsing
├── mdx/            # MDX segmenter + renderer (feature = "mdx")
│   ├── mod.rs      # Public API — Segment enum, segment(), render()
│   ├── render.rs   # Assembly layer: segments → HTML body + ESM + front matter
│   ├── splitter.rs # Line-based state machine
│   ├── jsx_tag.rs  # JSX tag boundary parser
│   └── expr.rs     # Expression boundary parser (brace/string/comment tracking)
├── footnote.rs     # Footnote store and rendering
├── link_ref.rs     # Link reference definitions
├── cursor.rs       # Pointer-based byte cursor
├── range.rs        # Compact u32 range type
├── render.rs       # HTML writer
├── escape.rs       # HTML escaping (memchr-optimized)
└── limits.rs       # DoS prevention constants
```

## License

MIT

---

<!-- ferramenta-family:start -->
## The Ferramenta family

This project is part of [Ferramenta](https://ferramenta.dev) — the family of Rust-native developer tools by [Sebastian Software](https://oss.sebastian-software.com) that keep the APIs the ecosystem already knows:

| Tool | Job |
| --- | --- |
| [ferroni](https://github.com/sebastian-software/ferroni) | Oniguruma-compatible regex engine |
| [ferriki](https://github.com/sebastian-software/ferriki) | Shiki-compatible syntax highlighting |
| **[ferromark](https://github.com/sebastian-software/ferromark)** | CommonMark/GFM Markdown to HTML |
| [ferrovia](https://github.com/sebastian-software/ferrovia) | SVGO-compatible SVG optimizer |
| [ferrocat](https://github.com/sebastian-software/ferrocat) | Translation catalog engine |
| [ferrolex](https://github.com/sebastian-software/ferrolex) | Spell, dictionary, and brand validation |
| [ferrugo](https://github.com/sebastian-software/ferrugo) | Rust-native PDF previews |
<!-- ferramenta-family:end -->

<!-- sebastian-software-branding:start -->
<p align="center">
  <a href="https://oss.sebastian-software.com">
    <img src="https://sebastian-brand.vercel.app/sebastian-software/logo-software.svg" alt="Sebastian Software" width="240" />
  </a>
</p>

<p align="center">
  <a href="https://oss.sebastian-software.com">Open Source at Sebastian Software</a><br />
  Copyright &copy; 2026 Sebastian Software GmbH
</p>
<!-- sebastian-software-branding:end -->
