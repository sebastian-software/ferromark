# Export static Markdown from MDX

Use `prepared.projectMarkdown()` to export authored static documentation while
keeping Markdown links, reference definitions, headings, lists, tables, and code
examples. The result contains Markdown and diagnostics for every removed or
unwrapped construct. This capability is additive source on `main`; build the
[Node package from source](../node/ferromark/README.md#install) until it is released.

Prepare once with the grammar and parser features your source needs. The
asset-free `prepareDocument` entry and `JsxCompiler.prepare` return the same
immutable handle; projection works with either. It reuses the owned original
source and prepared tree without reparsing, rendering, executing JavaScript,
loading highlighting assets, or changing a later render.

```js
import { prepareDocument } from "ferromark";

const source = `---
title: Guide
---

import Box from "./box.js";

# Guide

<Box>**Static** [link](/guide) {dynamic}</Box>
`;
const prepared = prepareDocument(source, { format: "mdx", frontMatter: true });
const result = prepared.projectMarkdown({
  frontMatter: "omit",
  omitTitleHeading: "Guide",
});
console.log(result.markdown); // Includes **Static** [link](/guide), with blank lines retained.
console.log(result.diagnostics.map(({ code }) => code));
// front-matter-omitted, esm-omitted, title-heading-omitted,
// jsx-wrapper-unwrapped, expression-omitted
```

The full options and result types live in the
[TypeScript reference](../node/ferromark/index.d.mts). Navigation, document
titles, URLs, document concatenation, and `llms.txt` formatting belong to the caller.

## Source fidelity and limits

This is a narrow source projection, not a Markdown serializer or a component
renderer. Unchanged source slices retain their bytes, including reference
definitions that do not produce rendered elements. Backtick and tilde fences,
fence lengths, inline code, indented code, code metadata, line endings, and
MDX-looking text inside code remain intact. Native pass changes such as emoji
replacement, typography, and GitHub reference links are not serialized; the
projected text can differ from the prepared tree's rendered output.

Authored child indentation is retained, including indentation that the MDX
parser normalizes for component rendering. If a component's children rely on
that normalization, their interpretation as standalone Markdown can differ.
Projection retains blank lines and list/blockquote prefixes at removal
boundaries. It inserts a separating space where removing inline content would
join words or form new Markdown tokens; wrappers inside existing Markdown
formatting preserve that formatting's delimiters.

| Parsed construct | Static projection policy |
| --- | --- |
| MDX `import` / `export` | Omit the complete statement; never import or execute it. |
| Paired JSX components, intrinsic elements, and fragments | Discard both tags and every attribute; retain original static children in source order. |
| JSX `script`, `style`, `title`, `textarea` | Omit the entire element and body, including static-looking text. |
| MDX child expressions | Omit the complete expression, including literal strings and comments; never infer its value. |
| Self-closing JSX elements | Omit; they have no known static children. This includes images and line-break elements. |
| Raw HTML AST nodes | Omit; an HTML block can include its entire body. Inline raw-text elements and their bodies are omitted. Other inline HTML tags are omitted while separately parsed Markdown remains. |
| Unverifiable JSX source boundaries | Omit the whole element conservatively. |

Wrapper unwrapping is a structural policy: it does not understand tabs,
conditional visibility, slots, summaries, or any other component semantics.
Dynamic attributes and spreads are discarded with their wrapper. Diagnostics
make that loss explicit; the output does not promise parity with runtime React
output. Strict MDX parsing still rejects invalid JavaScript/JSX during
preparation. With `format: "md"`, ordinary text resembling MDX remains ordinary
Markdown unless the parser recognizes a raw HTML node.

## Metadata and diagnostics

Front matter and title headings are preserved by default. `frontMatter: "omit"`
removes only front matter recognized during parsing, including delimiter lines;
enable `frontMatter: true` when preparing it. Projection never parses YAML or
TOML values. `omitTitleHeading` considers only the first top-level H1, compares
its static display text with the trimmed nonempty title, and omits it only on a
match. Later matching headings and H1s inside components remain. The result's
`omittedTitleHeadingSpan` identifies the selected heading.

Each diagnostic has a stable `code`, explanatory `message`, and original
half-open UTF-8 byte range `[start, end)`. Offsets are neither JavaScript string
indices nor positions in projected Markdown. Use a UTF-8 buffer to read them:

For byte inputs, offsets refer to the owned UTF-8 source after the package's
standard decoding; invalid UTF-8 bytes are replaced before parsing.

```js
const bytes = Buffer.from(source, "utf8");
for (const { code, start, end } of result.diagnostics) {
  console.log(code, bytes.subarray(start, end).toString("utf8"));
}
```

Diagnostics appear in source order. A wrapper diagnostic covers the complete
element, so it can contain diagnostics for nested expressions or wrappers.
Requested front matter/title omissions also produce diagnostics. Output and
diagnostic objects are frozen owned copies; repeated calls return equal results.

## Native Rust

Call `Document::project_markdown` with the exact original source and
`MarkdownProjectionOptions`. The core returns owned Markdown and typed
diagnostics; no optional highlighting or JavaScript compiler feature is needed
for projection itself. Enable the `jsx` feature and `mdx_compatible` parser option
when you want the same strict grammar as Node's `format: "mdx"`.

Run the complete [Rust example](../examples/project_markdown.rs) with
`cargo run --example project_markdown --all-features --locked`, and the
[offline Node example](../node/examples/project-markdown.mjs) after building the
local package with `node node/examples/project-markdown.mjs`.

The core checks that visited spans are UTF-8 boundaries inside the supplied
source and returns `MarkdownProjectionError` for invalid ranges. Callers that
construct or transform their own AST must retain its original source spans and
pass the matching source. A tree does not authenticate a different source of
the same length.
