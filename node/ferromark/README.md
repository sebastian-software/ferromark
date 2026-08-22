# ferromark for Node.js

Native Node.js bindings for the [ferromark](https://github.com/sebastian-software/ferromark) Markdown-to-HTML compiler.

```js
import { toHtml } from 'ferromark'

const html = toHtml('# Hello')
```

## Input size limit

ferromark source ranges use `u32` byte offsets, so input is limited to
4,294,967,295 bytes. Calls above that limit throw an `InvalidArg` native error
instead of parsing with truncated source offsets.

## Syntax highlighting with Ferriki

An initialized [Ferriki](https://github.com/sebastian-software/ferriki) highlighter plugs into the fenced-code renderer without coupling the two native cores:

```js
import { createHighlighter } from 'ferriki'
import { toHtmlWithHighlighter } from 'ferromark'

const highlighter = await createHighlighter({
  langs: ['rust'],
  themes: ['github-dark'],
})

const html = toHtmlWithHighlighter(
  '```rust\nfn main() {}\n```',
  highlighter,
  { theme: 'github-dark' },
)
```

Unsupported languages and highlighter errors fall back to ferromark's escaped `<pre><code>` output.
Highlighter HTML is otherwise written verbatim, so only pass an implementation that escapes untrusted code and metadata.
Fence meta text after the language (e.g. ` ```ts {1-3} title="…" `) reaches the highlighter as Shiki-style `meta.__raw`, so meta-driven transformers (line highlighting, titles) work unchanged.

## Document metadata for docs pipelines

`transform()` returns HTML together with the data documentation tooling needs — headings for a table of contents and the raw front matter block:

```js
import { transform } from 'ferromark'

const { html, headings, frontMatter } = transform(source, { frontMatter: true })
// headings: [{ level: 2, id: 'getting-started', text: 'Getting Started' }, …]
// frontMatter: raw text between the --- delimiters (parse with your YAML library)
```

`transformWithHighlighter()` combines this with fenced-code highlighting in the same native pass.

For sites deployed under a subpath (e.g. GitHub Pages), `linkBasePath` prefixes internal absolute link destinations natively:

```js
toHtml('[guide](/guide)', { linkBasePath: '/docs' })
// <p><a href="/docs/guide">guide</a></p>
```

Image sources and autolinks are not rewritten.

The package supports Node.js 22 or newer on glibc Linux, macOS, and Windows for x64 and arm64. It does not include a WASM fallback.

## Native build profile

The native addon is built with Cargo's workspace-level `release-node` profile. It
keeps the optimized release settings while enabling panic unwinding, allowing
N-API to translate a Rust panic into a JavaScript exception instead of aborting
the Node.js process.
