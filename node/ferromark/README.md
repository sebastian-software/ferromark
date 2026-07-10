# ferromark for Node.js

Native Node.js bindings for the [ferromark](https://github.com/sebastian-software/ferromark) Markdown-to-HTML compiler.

```js
import { toHtml } from 'ferromark'

const html = toHtml('# Hello')
```

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

The package supports Node.js 20 or newer on glibc Linux, macOS, and Windows for x64 and arm64. It does not include a WASM fallback.
