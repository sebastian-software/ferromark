# Native JSX renderer

`JsxRenderer` converts Ferromark's Markdown/MDX AST directly into a JSX body
expression. It does not emit HTML first, add a React import, compile JSX, or
wrap the result in a page or module function. The caller owns those steps.

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
