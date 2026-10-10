# Native JSX renderer

`JsxRenderer` converts Ferromark's Markdown/MDX AST directly into a JSX body
expression. It does not emit HTML first, add a React import, or compile JSX.
`render` returns the body and its parts. With the `jsx` feature,
[`render_module`](#mdx-module-output) wraps the body in a complete MDX module.

```rust
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{JsxRenderer, JsxRendererOptions};

let allocator = Allocator::new();
let source = "# Hello\n\nA paragraph.";
let document = Parser::with_options(&allocator, source, ParserOptions::mdx())
    .parse()
    .unwrap();
let options = JsxRendererOptions::default();
let output = JsxRenderer::with_options(options).render(&document, source);
assert!(output.body.starts_with("<>\n"));
```

The body is always a JSX fragment, including for an empty document. Markdown
text is serialized as JavaScript string children so braces, angle brackets,
entities, and whitespace remain data rather than becoming JSX expressions.
Authored MDX elements, expressions, literal attributes, spreads, and their
ordering are emitted from the AST/source. Top-level ESM is returned separately
in `JsxOutput::esm` in document order. These ESM strings and JSX expressions
are source, not sandboxed or evaluated code. A code hook may return trusted JSX
for a fenced block; the caller is responsible for its validity and safety.

`JsxOutput::components` lists authored component roots in first-reference order,
including configured callout/code components. `elements` lists intrinsic names
generated for Markdown constructs even when `component_prefix` qualifies those
elements, for example `p` for `<_components.p>`. Authored JSX names are never
prefixed. The prefix must be a valid JSX member-expression prefix. `headings`
contains the headings actually emitted, with IDs planned in the same order as
generated heading and footnote IDs; MDX element children contribute their
visible text. `omitted_title_heading` reports the source span of the omitted
first matching top-level H1 so a caller can keep source-backed metadata in
sync. It does not parse frontmatter or infer a title.

Source mappings use zero-based lines and UTF-16 columns, like JavaScript
tooling. They include generated node starts and each source line copied from
preserved expressions or literal JSX attributes. Fenced code descriptors retain
their original source span, code, language, and metadata.

Raw HTML nodes are normalized into JSX: HTML entity text is decoded before
string-child serialization, common HTML attribute names such as `class` are
mapped to JSX spellings, style declarations become object-valued `style`
props, comments become JSX comments, and void elements are self-closing.
Namespaced raw tags are retained as visible source text because their names
are not portable in JSX. Declarations such as doctypes are also preserved as
text children. This renderer does not promise browser-native handling for
script/style content or inline event-handler strings after a framework compiles
the JSX.

Generated Markdown defaults to native HTML-like intrinsic tags, with optional
component mappings for callouts and fenced code languages. The code hook takes
precedence over a language mapping. Set `callouts` to `false` to render a
callout marker as ordinary block-quote text. Heading level offsets, ID
prefixes, and exact title-heading omission are configured on
`JsxRendererOptions`.

## MDX module output

`JsxRenderer::render_module` needs the `jsx` feature. It returns a complete ES
module that still contains JSX, with a source map. The module imports no
framework and nothing evaluates it; the caller's JSX transform compiles it.
Parse the document with `mdx_compatible` so module blocks have exact source
ranges.

```rust
use ferromark::{Allocator, JsxModuleOptions, JsxRenderer, Parser, ParserOptions};

let source = "import { Chart } from './chart.js'\n\n# Sales\n\n<Chart />\n\n<Note />\n";
let allocator = Allocator::new();
let options = ParserOptions { mdx: true, mdx_compatible: true, ..ParserOptions::default() };
let document = Parser::with_options(&allocator, source, options).parse().unwrap();
let module = JsxRenderer::new()
    .render_module(&document, source, &JsxModuleOptions {
        provider_import_source: Some("docs/provider".to_owned()),
        filename: Some("sales.mdx".to_owned()),
        ..JsxModuleOptions::default()
    })
    .unwrap();
assert!(module.code.contains("export default function MDXContent(props = {})"));
assert_eq!(module.bindings, ["Chart"]);
```

The module follows the MDX module contract:

- **Authored ESM** comes first, in document order, as written.
- **`_createMdxContent(props)`** returns the body. Generated Markdown elements
  are members of `_components`. That object merges the intrinsic defaults, the
  provider's `useMDXComponents()`, and `props.components`; a later entry wins.
  `provider_import_source` names the module that exports `useMDXComponents`.
  Without it the module imports nothing.
- **`MDXContent(props)`** is the default export and renders the content inside
  the layout. Set `default_export` to `false` to keep it a local binding and
  append your own default export.
- **The layout** is an authored default export. It replaces the `wrapper`
  component from the provider or `props.components`. A declaration
  (`export default function Layout() {}`), an expression (`export default
  Layout`), a specifier (`export { Layout as default }`), and a re-export
  (`export { default } from './layout.js'`) are all accepted. A second default
  export is an error.
- **A component reference** whose root identifier the document's ESM binds
  uses that binding. Every other reference comes from `_components`. When it is
  undefined, the module throws an error that names the component instead of
  the framework's generic element-type error. A member reference such as
  `<kit.Input />` checks `kit` as an object and `kit.Input` as a component.
  `props` refers to the function parameter.

The generated names `_components`, `_createMdxContent`, `_missingMdxReference`,
`_provideComponents`, `MDXLayout`, and `MDXContent` are reserved. An authored
declaration of one of them is an error, so no pass over the document is needed
to find free names. `reserved_bindings` extends that set with names the caller
declares in code it adds: a component reference to such a name uses the
caller's binding, and an authored declaration of it is an error.

`exports` lists the names the authored ESM exports, and `bindings` lists every
name it binds at the top level, including imports. A caller reads them to
decide which exports it still has to add. Import declarations are hoisted, so
a caller can append its own imports and exports after the module code without
changing the source map.

The source map is a version 3 map with zero-based lines and UTF-16 columns.
Authored ESM maps per word, so a position inside a module block resolves to
its own column. A generated node maps to the start of its Markdown source, and
a rewritten default export maps to its statement.

Module output differs from `@mdx-js/mdx` in three places:

- JSX inside authored JavaScript, in an expression or a module block, is
  source text and not part of the tree. In `{items.map((item) => <Card />)}`,
  `Card` is a plain identifier: import it or declare it. The provider does not
  supply it.
- Module blocks are JavaScript. TypeScript syntax is a parse error that says
  so.
- The module carries no JSX runtime comment; the caller's JSX transform
  decides the runtime.

## Native Ferriki highlighting

Enable both `jsx` and `ferriki` and pass `FerrikiJsxHooks` to
`render_with_hooks`. The adapter borrows a public Ferriki `Highlighter`; use
`new(highlighter, theme)` for a single theme or
`with_light_dark_themes(highlighter, light, dark)` for a pair. The caller owns
asset preparation and can observe highlighting failures with
`with_error_handler`. Unknown languages select escaped plain output.

Highlight hooks return balanced inner fragments for each source line. The JSX
renderer converts them to JSX-safe children and owns `<pre><code>` markup,
title/label attributes, highlighted lines, and line numbers. Dual-theme tokens
carry light/dark CSS color and font variables; the light styles work without
theme-switching CSS. No framework runtime or HTML injection prop is emitted.

`code_block_component` wraps that output in a caller-owned component with
original code and metadata props. A whole-fence callback has priority, followed
by language-specific component mappings, native highlighting, and plain
rendering. Fence metadata accepts `title="..."`, `[label]`, `{1,3-5}`,
`showLineNumbers`, `:line-numbers`, and `:no-line-numbers`.

The Node `JsxCompiler` owns the native highlighter and loads standard assets
itself. Reuse the compiler across documents. Themes load at construction;
grammars load only for fences that reach highlighting. JSON theme/grammar
registrations support custom-only offline use. `assets` or Ferriki's environment
variables configure verified remote/cache loading. The simpler `compileJsx`
function leaves code unhighlighted and remains suitable for metadata and scope
analysis.

### Rendering a standalone code block

`JsxCompiler.renderCodeBlock` accepts the code and the same language and
metadata values used by a Markdown fence. It returns intrinsic JSX and the
parsed `language`, `title`, `label`, and effective `lineNumbers` value:

```ts
const block = compiler.renderCodeBlock({
  code: 'const answer = "日本語";\n',
  language: "typescript",
  meta: '[example] title="answer.ts" {1} :line-numbers=4',
});

block.jsx; // <pre><code>…</code></pre> JSX with the compiler's native highlighting
block.language; // "typescript"
block.title; // "answer.ts"
block.label; // "example"
block.lineNumbers; // true
```

The JSX markup is the same markup a fence produces with matching code,
language, and metadata when `componentPrefix`, `codeComponents`, and
`codeBlockComponent` are unset. The standalone result always uses plain
intrinsic `<pre>`, `<code>`, and `<span>` tags, has no document fragment, and
ends with the newline used after a fence in a document. The method shares the
compiler's highlighter, single or light/dark themes, and line-number default.
Unknown languages use escaped plain code. CR and CRLF code input is normalized
to LF, like code read from a fence. Empty code, trailing empty lines, and a
missing final newline are preserved.

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
