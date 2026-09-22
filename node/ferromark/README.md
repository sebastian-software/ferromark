# ferromark v2 for Node.js

Native Node.js bindings for the [Ferromark](https://github.com/sebastian-software/ferromark) Markdown-to-HTML compiler.

[Documentation site](https://ferromark.dev/) ·
[Rust crate](https://crates.io/crates/ferromark)

## Install

```sh
npm install ferromark
```

To build from source, check out this repository and run from `node/`:

```sh
pnpm install --frozen-lockfile
pnpm build
pnpm test
```

See the [v2 migration guide](../../docs/migration-v2.md) for breaking changes.
The v2 engine uses an arena AST and retains upstream MIT attribution in `LICENSE`.

Removed options `tableColumnWidths` and `indentedCodeBlocks` throw an unknown-option error. `tableColgroup`,
`tableColumnNames`, `tableAttributes`,
`headingAttributes`, `wikiLinks`, `cjkEmphasis`, and `mdx` expose v2 features.
Optional `highlight` (`==text==`) and `inlineFootnotes` (`^[note]`) default off.
`allowLinkRefs` defaults on; disabling it keeps reference definitions visible.
See [optional writing syntax](../../docs/optional-writing.md).
Heading slugs and extension HTML follow v2. `linkBasePath` enables v2 site routing:
root-absolute links, images, and raw HTML URLs use the base, and Markdown links
become index.html routes. Highlighters receive fenced and indented code blocks.

The package requires Node.js 22.12 or newer. It installs one platform-specific
native package for glibc or musl Linux, macOS, and Windows on x64 and arm64;
musl support includes Alpine Linux. GNU Linux binaries target glibc 2.17 or
newer, although the installed Node.js runtime may impose a newer requirement.
There is no WASM fallback.

Consumers and contributors have different floors on purpose. The published
package supports Node.js 22.12.0 and newer, while the repository's `node/`
development workspace declares Node.js 22.13.0 for its pinned pnpm toolchain.
The `node-floor` CI job therefore builds the addon on the workspace version and
then runs the package tests on 22.12.0, so the published floor stays the one
that is actually verified.

```js
import { toHtml } from "ferromark";

const html = toHtml("# Hello");
```

Both ESM `import` and CommonJS `require()` work on supported Node.js versions:

```js
const { toHtml } = require("ferromark");
```

## Optional writing syntax

```js
import { toHtml } from "ferromark";

const html = toHtml("This is ==important==^[An explanatory *note*.].", {
  highlight: true,
  inlineFootnotes: true,
});

// Reference definitions stay visible; ordinary inline links still work.
const literalReferences = toHtml("[name]\n\n[name]: /target", {
  allowLinkRefs: false,
});
```

`highlight` and `inlineFootnotes` default to `false`; `allowLinkRefs` defaults to
`true`. The same options work with `Renderer`, buffer output, transforms, and
highlighter helpers. Inline notes do not require `footnotes: true`. Their output
uses v2's existing footnote markup. Marked text does not enable code highlighting.

Keep writing extensions off when their syntax is not needed. Disabled-option
measurements were close to the pre-feature core; enabling unused syntax still
costs several percent on ordinary documents and more on marker-heavy inputs.
Reference-link savings depend on the profile and input. See the
[syntax contract](../../docs/optional-writing.md) and
[measured costs](../../docs/reports/2026-09-15-optional-writing/README.md).

## Repeated rendering

Create a `Renderer` when processing many documents with the same options. It
retains native parser scratch allocations between calls while keeping each
document's headings, references, and footnotes isolated.

```js
import { Renderer } from "ferromark";

const renderer = new Renderer({ headingIds: true });
const first = renderer.toHtml("# First");
const second = renderer.toHtml("# Second");
```

Create one renderer per worker; its options are fixed at construction.

## Buffer output

Use `toHtmlBuffer()` when the rendered HTML goes directly to a byte-oriented
consumer such as an HTTP response. It returns a UTF-8 Node.js `Buffer` copied
from the rendered bytes, which skips the JavaScript string transcode.

```js
import { toHtmlBuffer } from "ferromark";

response.end(toHtmlBuffer("# Hello"));
```

Reusable renderers provide the same output path through
`renderer.toHtmlBuffer(markdown)`.

## Untrusted by default

`toHtml()` and `transform()` default to `renderPolicy: 'untrusted'`. Raw HTML
is escaped, and unsafe link and image URL schemes (such as `javascript:`) are
removed from the rendered attributes. Use this default for Markdown from users
or other untrusted sources.

```js
toHtml("<img src=x onerror=alert(1)>");
// '&lt;img src=x onerror=alert(1)&gt;'
```

Set `renderPolicy: 'trusted'` only when the Markdown source is trusted. Trusted
mode permits arbitrary URL schemes and ordinary raw HTML. The default
`disallowedRawHtml` filter still removes GFM-disallowed tags; set it to `false`
only when trusted content needs those tags. Trusted mode is not appropriate for
untrusted user content.

```js
toHtml('<span class="note">Internal note</span>', {
  renderPolicy: "trusted",
});
// '<p><span class="note">Internal note</span></p>\n'
```

See [`Options`](./index.d.mts) for the complete optional syntax and rendering
configuration. `transform()` also returns headings and optional front matter;
the highlighter helpers below accept trusted highlighter HTML.

## Options reference

Every `Options` property is optional; omitted values use the Node binding defaults. The TypeScript declaration is the complete, editor-linked reference. Defaults on: `allowHtml`, `tables`, `strikethrough`, `taskLists`, `disallowedRawHtml`, `headingIds`, and `callouts`. All other boolean syntax extensions default off; `renderPolicy` defaults to `'untrusted'` and `linkBasePath` is unset.

Unknown option names throw a `TypeError` that identifies the rejected key, so
misspellings such as `taskList` cannot silently change rendered output.

Set `headingIdPrefix` to namespace generated and explicit heading IDs, their
permalinks, and `transform()` heading metadata. For example,
`toHtml("# Intro", { headingIdPrefix: "docs-" })` emits `id="docs-intro"`.
Prefixes accept ASCII letters, digits, underscores, and hyphens. Authored
fragment links and footnote IDs are unchanged.

`mergedTableCells`, `tableColgroup`, and `tableColumnNames` require `tables`. `disallowedRawHtml` only filters a narrow GFM tag list in trusted mode and is not a sanitizer. `renderPolicy: 'trusted'` permits raw HTML and unrestricted URL schemes, so use it only for trusted Markdown. See [`Options`](./index.d.mts) for each field's semantics and examples above for `frontMatter` and `linkBasePath`.

## Input size limit

Ferromark stores source positions as compact `u32` offsets, so the parser
addresses just under 4 GB of Markdown. Node.js strings stop far below that
(`buffer.constants.MAX_STRING_LENGTH`), so no JavaScript input can reach the
limit and the binding does not check for it.

## Syntax highlighting with Ferriki

An initialized [Ferriki](https://github.com/sebastian-software/ferriki) highlighter plugs into the code-block renderer without coupling the two native cores:

````js
import { createHighlighter } from "ferriki";
import { toHtmlWithHighlighter } from "ferromark";

const highlighter = await createHighlighter({
  langs: ["rust"],
  themes: ["github-dark"],
});

const html = toHtmlWithHighlighter("```rust\nfn main() {}\n```", highlighter, {
  theme: "github-dark",
  onHighlightError(error, { lang }) {
    console.warn(`Could not highlight ${lang}`, error);
  },
});
````

Unsupported languages and highlighter exceptions fall back to Ferromark's escaped `<pre><code>` output. Use `onHighlightError` to observe exceptions; if that callback throws, the render call throws too. Invalid highlighter return values also surface as native callback errors.
Highlighter HTML is otherwise written verbatim, so only pass an implementation that escapes untrusted code and metadata.
Fence meta text after the language (e.g. ` ```ts {1-3} title="…" `) reaches the highlighter as Shiki-style `meta.__raw`, so meta-driven transformers (line highlighting, titles) work unchanged.

## Document metadata for docs pipelines

`transform()` returns HTML together with the data documentation tooling needs — headings for application-owned navigation and the raw front matter block:

```js
import { transform } from "ferromark";

const { html, headings, frontMatter } = transform(source, { frontMatter: true });
// headings: [{ level: 2, id: 'getting-started', text: 'Getting Started' }, …]
// frontMatter: raw text between the --- delimiters (parse with your YAML library)
```

`transformWithHighlighter()` combines this with code-block highlighting in the same native pass.

For sites deployed under a subpath (e.g. GitHub Pages), `linkBasePath` prefixes internal absolute link destinations natively. A trailing slash is optional:

```js
toHtml("[guide](/guide) ![logo](/logo.png)", { linkBasePath: "/docs" });
// <p><a href="/docs/guide">guide</a> <img src="/docs/logo.png" alt="logo"></p>
```

Image sources take the same base, as do root-absolute URLs in raw HTML under
`renderPolicy: 'trusted'`, where raw HTML is written instead of escaped.
Absolute URLs, including autolinks, keep their destination.

## Troubleshooting native loading

The native binding loads when constructing `Renderer` or on the first call to
`toHtml()`, `toHtmlBuffer()`, `transform()`, or a highlighter helper. If that
load fails:

| Message or environment                       | Resolution                                                                                                                                                                                                                                                                                                                                                       |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Node.js below 22.12                          | Upgrade to Node.js 22.12 or newer; this is the package's declared engine requirement and the first Node 22 release with unflagged `require(esm)` support.                                                                                                                                                                                                        |
| Unsupported platform or architecture         | Use macOS, Windows, or glibc/musl Linux on x64 or arm64.                                                                                                                                                                                                                                                                                                         |
| `could not load the optional native package` | Reinstall without `--omit=optional` and verify that your lockfile includes Ferromark's package for the current platform.                                                                                                                                                                                                                                         |
| `ERR_DLOPEN_FAILED`                          | Read the wrapped loader message for the exact binary and platform. On GNU Linux, verify glibc 2.17 or newer and required shared libraries; on Windows, install or repair the Microsoft Visual C++ Redistributable; on macOS, check architecture, OS compatibility, quarantine, and code-signing policy. The original loader error is available as `error.cause`. |

This package does not include a WASM fallback, so unsupported environments need
one of the supported native runtimes rather than a JavaScript fallback.

## Native build profile

The native addon is built with Cargo's workspace-level `release-node` profile. It
keeps the optimized release settings while enabling panic unwinding, allowing
N-API to translate a Rust panic into a JavaScript exception instead of aborting
the Node.js process.

<!-- ferramenta-family:start -->

**ferromark** is part of the [Ferramenta](https://ferramenta.dev) family — A family of Rust tools.

Siblings: [ferroni](https://sebastian-software.github.io/ferroni/) — Oniguruma-compatible regex engine · [ferriki](https://github.com/sebastian-software/ferriki) — Shiki-compatible syntax highlighting · [ferrolex](https://github.com/sebastian-software/ferrolex) — Spell checking for text and code · [ferrocat](https://ferrocat.dev) — Translation catalog engine · [palamedes](https://palamedes.dev) — Internationalization for TypeScript applications · [ferrovia](https://github.com/sebastian-software/ferrovia) — SVGO-compatible SVG optimizer · [ferralk](https://github.com/sebastian-software/ferralk) — Glob matching and parallel filesystem walking · [ferrugo](https://github.com/sebastian-software/ferrugo) — PDF previews for untrusted files.

<!-- ferramenta-family:end -->
