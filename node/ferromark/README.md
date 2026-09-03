# ferromark for Node.js

Native Node.js bindings for the [ferromark](https://github.com/sebastian-software/ferromark) Markdown-to-HTML compiler.

## Install

```sh
npm install ferromark
# or: pnpm add ferromark
```

The package requires Node.js 22 or newer. It installs one platform-specific
native package for glibc or musl Linux, macOS, and Windows on x64 and arm64;
musl support includes Alpine Linux. GNU Linux binaries target glibc 2.17 or
newer, although the installed Node.js runtime may impose a newer requirement.
There is no WASM fallback.

```js
import { toHtml } from 'ferromark'

const html = toHtml('# Hello')
```

## Repeated rendering

Create a `Renderer` when processing many documents with the same options. It
retains native parser scratch allocations between calls while keeping each
document's headings, references, and footnotes isolated.

```js
import { Renderer } from 'ferromark'

const renderer = new Renderer({ headingIds: true })
const first = renderer.toHtml('# First')
const second = renderer.toHtml('# Second')
```

Create one renderer per worker; its options are fixed at construction.

## Untrusted by default

`toHtml()` and `transform()` default to `renderPolicy: 'untrusted'`. Raw HTML
is escaped, and unsafe link and image URL schemes (such as `javascript:`) are
removed from the rendered attributes. Use this default for Markdown from users
or other untrusted sources.

```js
toHtml('<img src=x onerror=alert(1)>')
// '&lt;img src=x onerror=alert(1)&gt;'
```

Set `renderPolicy: 'trusted'` only when the Markdown source is trusted. Trusted
mode permits arbitrary URL schemes and ordinary raw HTML. The default
`disallowedRawHtml` filter still removes GFM-disallowed tags; set it to `false`
only when trusted content needs those tags. Trusted mode is not appropriate for
untrusted user content.

```js
toHtml('<span class="note">Internal note</span>', {
  renderPolicy: 'trusted',
})
// '<p><span class="note">Internal note</span></p>\n'
```

See [`Options`](./index.d.mts) for the complete optional syntax and rendering
configuration. `transform()` also returns headings and optional front matter;
the highlighter helpers below accept trusted highlighter HTML.

## Options reference

Every `Options` property is optional; omitted values use the Rust `Options::default()` values. The TypeScript declaration is the complete, editor-linked reference. Defaults on: `allowHtml`, `allowLinkRefs`, `tables`, `strikethrough`, `taskLists`, `disallowedRawHtml`, `headingIds`, `callouts`, and `indentedCodeBlocks`. All other boolean syntax extensions default off; `renderPolicy` defaults to `'untrusted'` and `linkBasePath` is unset.

Unknown option names throw a `TypeError` that identifies the rejected key, so
misspellings such as `taskList` cannot silently change rendered output.

`mergedTableCells` and `tableColumnWidths` require `tables`. `disallowedRawHtml` only filters a narrow GFM tag list in trusted mode and is not a sanitizer. `renderPolicy: 'trusted'` permits raw HTML and unrestricted URL schemes, so use it only for trusted Markdown. See [`Options`](./index.d.mts) for each field's semantics and examples above for `frontMatter` and `linkBasePath`.

## Input size limit

ferromark source positions use compact `u32` values, so input is limited to
4,294,967,294 bytes. Calls above that limit throw an `InvalidArg` native error
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
  {
    theme: 'github-dark',
    onHighlightError(error, { lang }) {
      console.warn(`Could not highlight ${lang}`, error)
    },
  },
)
```

Unsupported languages and highlighter exceptions fall back to ferromark's escaped `<pre><code>` output. Use `onHighlightError` to observe exceptions; if that callback throws, the render call throws too. Invalid highlighter return values also surface as native callback errors.
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

## Troubleshooting native loading

The native binding loads when constructing `Renderer` or on the first call to
`toHtml()`, `transform()`, or a highlighter helper. If that load fails:

| Message or environment | Resolution |
| --- | --- |
| Node.js below 22 | Upgrade to Node.js 22 or newer; this is the package's declared engine requirement. |
| Unsupported platform or architecture | Use macOS, Windows, or glibc/musl Linux on x64 or arm64. |
| `could not load the optional native package` | Reinstall without `--omit=optional` and verify that your lockfile includes ferromark's package for the current platform. |
| `ERR_DLOPEN_FAILED` | Read the wrapped loader message for the exact binary and platform. On GNU Linux, verify glibc 2.17 or newer and required shared libraries; on Windows, install or repair the Microsoft Visual C++ Redistributable; on macOS, check architecture, OS compatibility, quarantine, and code-signing policy. The original loader error is available as `error.cause`. |

This package does not include a WASM fallback, so unsupported environments need
one of the supported native runtimes rather than a JavaScript fallback.

## Native build profile

The native addon is built with Cargo's workspace-level `release-node` profile. It
keeps the optimized release settings while enabling panic unwinding, allowing
N-API to translate a Rust panic into a JavaScript exception instead of aborting
the Node.js process.
