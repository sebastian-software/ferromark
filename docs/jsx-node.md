# Compile JSX and MDX in Node.js

Use Ferromark to turn trusted authored Markdown or MDX into JSX for your
application's build pipeline. It parses the document, applies selected native
passes, and emits JSX plus metadata. Your JSX transform compiles that output;
your application supplies components and evaluates the resulting program.

Install `ferromark` on Node.js 22.12 or newer. To try additions on `main` before
their release, [build the Node package from source](../node/ferromark/README.md#install).
The [TypeScript declarations](../node/ferromark/index.d.mts) own the full API
reference; the [Rust renderer contract](jsx-renderer.md) explains core output.

## Choose the operation

| Task                                         | API                                                    | Result                                                         |
| -------------------------------------------- | ------------------------------------------------------ | -------------------------------------------------------------- |
| Compile one document without highlighting    | `compileJsx(source, options?)`                         | JSX body and metadata                                          |
| Compile a complete MDX module                | `compileJsx(source, { output: "module", ...options })` | Module code and source map                                     |
| Highlight fences across documents            | `compiler.compile(source, options?)`                   | Body or module using cached native assets                      |
| Read metadata, then choose rendering options | `compiler.prepare(source, preparationOptions?)`        | Immutable document with metadata and repeatable render methods |
| Render code outside a Markdown document      | `compiler.renderCodeBlock({ code, language?, meta? })` | Intrinsic JSX and parsed fence metadata                        |

Create the compiler with `new JsxCompiler(compilerOptions?)`. Reusing it retains
loaded themes and grammars. Preparing a document also retains its parsed tree;
use that path when you need metadata before rendering or more than one output
from the same source. A single `compile` call remains the shorter path when all
options are already known. Preparation does not cache rendered JSX: highlighting
and code callbacks run on each render.

All these calls are synchronous. Use worker threads for sustained CPU work in a
server. Document inputs accept a string or UTF-8 `Uint8Array`, including a Node
`Buffer`. Standalone `code` is a string.

## Compile a first document

Save this as `example.mjs` and run `node example.mjs`:

```js
import { compileJsx } from "ferromark";

const result = compileJsx("# Hello\n\n<Badge>Native MDX</Badge>", {
  format: "mdx",
});

console.log(result.body);
console.log(result.headings.map(({ id }) => id)); // ["hello"]
console.log(result.components); // ["Badge"]
```

`body` is a JSX fragment expression, including for empty input. `esm` holds
authored imports and exports separately. `elements` lists the generated
Markdown tag names. `format` defaults to `"md"`; choose `"mdx"` for strict
JavaScript/JSX grammar validation. `componentPrefix: "_components"` qualifies
generated Markdown tags in a body, such as `<_components.p>`. It does not provide
that object or rewrite authored component names.

JSX compilation treats source as a program. It does not apply the HTML API's
`renderPolicy: "untrusted"`. Use the HTML APIs when you need escaped raw HTML
and URL filtering for untrusted Markdown.

## Prepare once and choose output later

The following example uses a small JSON theme so it runs without downloading
standard assets. Standard themes and native highlighting are covered below.

```js
import { JsxCompiler } from "ferromark";

const compiler = new JsxCompiler({
  theme: {
    name: "guide-theme",
    settings: [{ settings: { foreground: "#222222", background: "#ffffff" } }],
  },
  assets: { remote: false },
});
const source = "---\ntitle: Guide\n---\n\n# Guide\n\n## Usage\n\nReady :rocket:\n";
const prepared = compiler.prepare(source, {
  format: "mdx",
  frontMatter: true,
  passes: [{ kind: "emojiShortcodes" }],
});

console.log(prepared.metadata.frontMatter); // "title: Guide\n"
console.log(prepared.metadata.outline.map(({ id }) => id)); // ["guide", "usage"]

const body = prepared.render({ omitTitleHeading: "Guide", headingIdPrefix: "docs-" });
console.log(body.headings.map(({ id }) => id)); // ["docs-usage"]

const module = prepared.renderModule({ omitTitleHeading: "Guide", filename: "guide.mdx" });
console.log(module.map.sources); // ["guide.mdx"]
console.log(prepared.render().headings.length); // 2; the title was not removed from the tree
```

Preparation copies the source, parses once, runs ordered native passes once,
and computes a frozen metadata snapshot. Changing an input buffer afterward
does not change the document. Metadata reads do not render a body or load code
grammars. Compiler construction still loads its themes.

Front matter is raw YAML or TOML text, accompanied by `frontMatterKind` and
`frontMatterSpan` when present. Parse it with your application's chosen library
before deriving a title or other rendering choices. Ferromark does not infer a
title from front matter. `omitTitleHeading` removes only the first top-level H1
when its trimmed visible text matches the supplied title.

`metadata.esm` contains module statements with source ranges; `codeBlocks`
contains code, language, and raw metadata in source order. `metadata.outline`
uses preparation-time default heading settings. Read `headings` from the final
body or module when building navigation: those headings reflect title omission,
heading offsets, prefixes, and the final heading/footnote ID planner.

### Put options at the stage that owns them

| Stage                      | Options                                                                                                                                                          | When they take effect                               |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| Compiler construction      | `theme`, `languages`, `assets`, `lineNumbers`                                                                                                                    | Shared highlighting configuration; themes load here |
| `prepare`                  | `format`, parser features such as `frontMatter` or `footnotes`, `passes`, legacy `typography`                                                                    | Fixed grammar and native passes for this document   |
| `render` or `renderModule` | `componentPrefix`, `calloutComponents`, `codeComponents`, `codeBlockComponent`, `omitTitleHeading`, `headingIds`, `headingOffset`, `headingIdPrefix`, `callouts` | Fresh output and heading planning on each call      |
| `renderModule` only        | `providerImportSource`, `filename`, `defaultExport`, `reservedBindings`                                                                                          | Module composition and source map                   |

Do not combine `passes` with the legacy top-level `typography` option. Put
typography into the ordered passes when combining transformations. See the
[native transform guide](../transforms/README.md) for pass contracts.

The facade rejects keys from the wrong stage and unknown keys. To change parser
settings or passes, prepare a new document. Select module output by calling
`renderModule`; do not pass `output: "module"` to a prepared render. Modules
use `_components` for generated tags, so omit `componentPrefix` there or use
`"_components"`.

One-shot `compileJsx` and `compiler.compile` keep their combined-options shape.
For the same compiler, source, preparation options, render choices, and callback,
prepared output matches one-shot output. The prepared tree is immutable; each
render starts fresh. A prepared handle owns its source and tree and retains the
highlighter even after the compiler object becomes unreachable. Keeping handles
in an application cache also keeps those resources alive; dropping references
lets garbage collection reclaim them. There is no explicit disposal method.

## Compile an MDX module

```js
import { compileJsx } from "ferromark";

const source = 'export const section = "docs";\n\n# Guide\n\n<Badge />\n';
const result = compileJsx(source, {
  format: "mdx",
  output: "module",
  filename: "guide.mdx",
});

console.log(result.exports); // ["section"]
console.log(result.bindings); // ["section"]
console.log(result.map.sources); // ["guide.mdx"]
// Pass result.code to your JSX transform, then supply Badge through components.
```

`code` contains authored ESM, a content function, and `MDXContent` as the default
export. It still contains JSX and imports no framework runtime. Generated tags
use intrinsic defaults merged with a provider's `useMDXComponents()` and then
`props.components`; later entries win. Set `providerImportSource` to the module
that exports that provider function. An authored default export becomes the
layout and takes precedence over the `wrapper` component.

An authored ESM binding supplies a component when present; otherwise the module
looks it up in the components object. Missing components produce named errors
when the generated module runs. JSX inside authored JavaScript, such as
`{items.map((item) => <Card />)}`, remains source text: import or declare `Card`
yourself. Module blocks accept JavaScript, not TypeScript.

To append a custom default export, set `defaultExport: false`; the generated
`MDXContent` remains available as a local binding. List names declared by your
added code in `reservedBindings`. Check `exports` and `bindings` before adding
names that may already exist. See the [module contract](jsx-renderer.md#mdx-module-output)
for reserved names, layout forms, and source-map rules.

Source spans, including ESM and heading spans, count original UTF-8 bytes.
Body `mappings` and the module's version 3 `map` use zero-based lines and UTF-16
columns. These units differ for non-ASCII text; do not use byte offsets as
JavaScript string indexes.

## Highlight fences and standalone code

```js
import { JsxCompiler } from "ferromark";

const compiler = new JsxCompiler({
  theme: { light: "github-light-default", dark: "github-dark-default" },
  lineNumbers: true,
});
const code = "const ready = true;\n";
const meta = '[example] title="ready.ts" {1} :line-numbers=4';
const block = compiler.renderCodeBlock({ code, language: "typescript", meta });

console.log(block.title); // "ready.ts"
console.log(block.label); // "example"
console.log(block.lineNumbers); // true
console.log(block.jsx); // Intrinsic <pre><code> JSX, with highlighted spans
```

The compiler owns the native Ferriki highlighter. The default theme is the
GitHub light/dark pair. Themes load at construction; a recognized fence language
loads when code reaches highlighting, then stays cached in that compiler.
Prepared documents and standalone blocks share these assets.

Standard assets use Ferriki's verified release CDN and cache. `assets` can
select a local asset root, cache directory, or mirror. For offline operation,
prepopulate the required assets and set `assets.remote: false`, or use
`FERRIKI_CACHE_DIR` and `FERRIKI_ASSETS_REMOTE=0`. Missing required assets throw;
disabling downloads does not supply them. Named JSON themes and custom TextMate
grammars in `languages` support custom-only offline operation. Unknown languages
use escaped plain output.

Light styles work without theme-switching CSS. Dual-theme output also carries
`--shiki-light` / `--shiki-dark`, background, and font variables. Your stylesheet
selects the dark values when the application's theme changes; the compiler
does not install that stylesheet.

For document fences, precedence is whole-fence callback, language-specific
`codeComponents`, then native highlighting or plain output. `codeBlockComponent`
wraps native output with original code and parsed metadata props. The standalone
method always emits intrinsic tags and does not invoke these document overrides.

### Standalone fence metadata and output

The JSX markup is the same markup a fence produces with matching code,
language, and metadata when `componentPrefix`, `codeComponents`, and
`codeBlockComponent` are unset. The standalone result always uses plain
intrinsic `<pre>`, `<code>`, and `<span>` tags, has no document fragment, and
ends with the newline used after a fence in a document. The method shares the
compiler's highlighter, single or light/dark themes, and line-number default.
Unknown languages use escaped plain code. NUL characters become U+FFFD, and CR
and CRLF code input is normalized to LF, like code read from a fence. Empty
code, trailing empty lines, and a missing final newline are preserved.

The `language` value is trimmed and returned without a recognized metadata
suffix; its case is preserved. A blank or missing language is omitted. Metadata is split on
whitespace, except inside quoted values and `[...]` or `{...}` groups. The
supported tokens are:

- `title="..."` or `title='...'` sets the title; later title tokens replace
  earlier ones. Empty titles are ignored.
- `[label]` sets the first nonempty label. It also supplies the title when no
  title has been set; an explicit title token takes precedence.
- `{1,3-5}` selects one-based lines for the existing highlighted-line markup.
  Invalid and reversed ranges are ignored. A range that starts within the code
  and ends past its last line is clipped; individual line selections outside
  the code and ranges that start past it are ignored.
- `:line-numbers`, `showLineNumbers`, and `:line-numbers=N` enable line
  numbers. `N` must be a positive integer and sets the first displayed number.
  `:no-line-numbers` and `noLineNumbers` disable them. Tokens are applied from
  left to right, starting with the compiler's `lineNumbers` default.

Other metadata tokens are ignored. Recognized tokens may be appended directly
to the language, for example `typescript:line-numbers=4`.
`renderCodeBlock` does not invoke whole-fence callbacks or component mappings;
it always returns the reusable intrinsic markup. Rust callers can use
`JsxRenderer::render_code_block` for escaped plain code or
`render_code_block_with_hooks` to apply the same metadata and markup path with
a `highlight_code_block` hook.

## Recover from errors

| Symptom                                          | What to check                                                                                                               |
| ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| Strict MDX syntax error                          | Fix the authored JavaScript/JSX; `format: "md"` is a different grammar, not a repair for invalid MDX                        |
| Unknown option or option from the wrong stage    | Move parser/pass options to `prepare` and rendering options to the render call; use `renderModule` for module output        |
| Missing theme or grammar asset                   | Populate the offline assets, correct the local root/cache, or permit the configured remote source                           |
| Missing component when the generated module runs | Import the component or provide it through the module's component lookup; import components used inside authored JavaScript |
| Recursive compiler-use error                     | Finish the current synchronous code callback before starting another render that shares the compiler's highlighter          |

`render` and `renderModule` accept an optional synchronous code callback as their
second argument. Return a complete trusted JSX string to replace the fence;
return `undefined` or `null` to use normal native output. Callback exceptions
propagate. A Promise is not a supported return value. After a callback error,
the prepared document can be rendered again; the tree remains unchanged.

For example, with the `compiler` created above:

````js
const prepared = compiler.prepare("```text\nprivate example\n```\n");
const result = prepared.render({}, () => '<CodePlaceholder label="Example" />');
console.log(result.body); // Contains the replacement JSX
````

Return replacement JSX directly from the callback. Calling `renderCodeBlock`,
`compile`, or another prepared render on the same compiler while that callback
is running is recursive use and throws. Prepare or render additional work
outside the callback. Native-loading failures are covered in the
[Node installation troubleshooting](../node/ferromark/README.md#troubleshooting-native-loading).
