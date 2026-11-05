# ADR-0027: Inject Ferriki and Ferrigura the same way in Node.js

- Status: Accepted
- Date: 2026-11-05
- Refines [ADR-0024](ADR-0024-native-code-plugins.md) and
  [ADR-0026](ADR-0026-ferrigura-crate-feature.md) for Node.js applications.

## Context

Ferriki and Ferrigura reach Rust applications the same way: as optional
features of the `ferromark` crate that implement the HTML and JSX render hooks.
In Node.js they did not. The standard addon contains neither engine
([ADR-0024](ADR-0024-native-code-plugins.md)), and engine-enabled addons exist
only as custom bundles. Applications inject engines through callbacks instead,
but the callback API covered only whole-block replacement. A Ferriki
highlighter connected that way replaced Ferromark's code block markup, losing
line annotations, titles, labels and the JSX `codeBlockComponent`, and its HTML
was not valid JSX source. Ferrigura's SVG had the same JSX problem. Neither
callback knew where its block was, so applications could not report a failed
diagram with a file and line.

Publishing an engine-enabled addon would keep rendering in Rust, but Ferromark
would then build, version and distribute a much larger binary: the local
plugin-bundle measurement found a 9.4 MB tarball with both engines against
1.2 MB without. Markdown rendering in general should not carry a diagram
engine, while documentation tools such as Ardo can install the engines they use.

## Decision

In Node.js, applications install and initialize their engines and inject them
as code hooks. Ferromark neither imports nor pins `@ferriki/core` or
`@ferrigura/core`; the standard addon stays engine-free. Custom bundles remain
available for applications that need the native plugins and the static SVG
policy.

The hooks mirror the two Rust stages, so highlighting and diagrams follow one
contract:

- `renderCodeBlock(code, language, meta, block)` replaces a whole block. It
  returns HTML or JSX source for the entry point's format, or `{ html }` for
  either. JSX entry points convert that HTML, including inline SVG, with the
  existing rules for authored raw HTML, now case-preserving for SVG attributes
  (`markerWidth`) and camel-casing hyphenated and namespaced ones
  (`strokeWidth`, `xlinkHref`). Rust exposes the conversion as
  `ferromark::html_to_jsx`.
- `highlightCodeBlock(code, language, meta, block)` returns one fragment per
  line plus optional light and dark block colors. Ferromark keeps its wrapper,
  annotations and component, as with `highlight_code_block` in Rust.
- `block` is the complete fence's original UTF-8 byte range in both stages. The
  Rust `CodeHighlightInput` gains the same `span` that whole-block inputs
  already had.

Whole-block replacements precede highlighting, and within each stage the first
result wins. The binding passes one dispatcher to native rendering and calls it
once per stage, so every entry point keeps a single callback parameter. Hooks
passed to `CodeCompiler` run before its native plugins within each stage.

Ferromark ships engine-neutral adapters: `createDiagramCodeRenderer` for any
synchronous `render(text, options)` renderer, and `createLineHighlighter` for
any Shiki-compatible highlighter that exposes tokens and `<pre>` colors to
transformers. The line highlighter writes the same fragments as the native
Ferriki JSX integration, including `--shiki-light`/`--shiki-dark` variables.
The diagram adapter gives each fence the native integration's namespace,
`<idPrefix>-<start offset>`. Both adapters fall back to ordinary code and report
failures with the block range through `onRenderError` and `onHighlightError`.
`composeCodeHooks` combines functions and hook objects in priority order.

## Consequences

Ardo and other documentation tools can render Mermaid and highlight code from
the npm packages without a custom Ferromark build, and both engines keep
Ferromark's code block metadata. Applications that do not install an engine pay
nothing for it.

Each code block costs one JavaScript call per stage it reaches. No measurement
backs a performance claim for this path; highlighting and diagram rendering
dominate the work. Hook output is trusted application HTML: the static SVG
policy of [ADR-0025](ADR-0025-static-mermaid-output.md) applies only to the
native integration, and diagnostics go to the adapters' callbacks rather than
`takeDiagnostics()`. The application loads highlighting languages before
rendering, because hooks are synchronous.

`CodeHighlightInput` has a new public field, a breaking change for Rust code that
constructs it. Whole-block callbacks now receive a fourth argument, and
`composeCodeRenderers` accepts `{ html }` results.

## Validation

Node tests cover both stages in HTML, JSX and prepared documents, stage order,
block ranges, `{ html }` conversion, result validation, and both adapters with
single and dual themes. Rust tests cover `html_to_jsx` and the highlighting
span. A local run with `@ferriki/core` 1.3.0 and `@ferrigura/core` 0.4.0
rendered highlighting, a diagram and both fallbacks, and the JSX output parsed
without TypeScript diagnostics.
