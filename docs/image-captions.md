# Images, figures, and captions

The image extensions are opt-in. `image_attributes` enables ID and class
suffixes on images; `image_captions` enables a separate visible caption for a
standalone image. Both are off by default in Node and in every Rust preset
except `ParserOptions::ffm()`, which enables both.

```markdown
![Three components connected by arrows](pipeline.svg "Browser title"){.diagram}

: The **processing pipeline** {#pipeline .figure-wide}
```

With both options enabled, the output has a `<figure id="pipeline"
class="figure-wide">`, an `<img>` with `alt="Three components connected by
arrows"`, `title="Browser title"`, and `class="diagram"`, and a `<figcaption>`
containing the rendered emphasis. The `![...]` text is always alternative
text. Neither it nor the image title becomes the visible caption; an empty alt
text remains empty.

```rust
use ferromark::{to_html_with_options, HtmlRendererOptions, ParserOptions};

let parser = ParserOptions {
    image_attributes: true,
    image_captions: true,
    ..ParserOptions::gfm()
};
let html = to_html_with_options(
    "![Logo](logo.svg){.logo}\n\n: Company logo",
    parser,
    HtmlRendererOptions::gfm(),
)?;
```

```js
import { toHtml } from "ferromark";

const html = toHtml("![Logo](logo.svg){.logo}\n\n: Company logo", {
  imageAttributes: true,
  imageCaptions: true,
});
```

The image suffix works without a caption, including an inline image and a
reference-style image. A caption works without either suffix. A caption must
follow a paragraph containing only one image, immediately or after one blank
line, in the same list item, quote, or document container. Its source is one
line and may contain ordinary inline Markdown. A linked image or a paragraph
with surrounding text stays an ordinary paragraph. An unrelated colon line is
not consumed. An escaped colon remains literal. Multiple images and multiline
captions are not supported by this syntax.
If a caption-like line occurs before an unfinished multiline image closes,
the paragraph stays ordinary Markdown; no later caption line attaches.

| Suffix location | HTML target |
| --- | --- |
| After `![alt](url)` or `![alt][ref]` | `<img>` |
| At the end of the image caption line | `<figure>` |
| At the end of a table caption line | `<table>` |
| At the end of a heading | The heading |

Image and table blocks accept one `#id` and any number of `.class` tokens. Names must be
nonempty and cannot contain quotes, angle brackets, equals, braces, backslashes,
or control characters. A repeated ID or invalid token leaves the image or
table suffix as Markdown text. Heading attributes retain their older, more
permissive name parser. Explicit IDs share the renderer's document-wide collision
sequence with heading IDs; `heading_id_prefix` applies only to headings.
Attributes are escaped during HTML output. Arbitrary `key=value` attributes
are not part of this extension.
Authored IDs are emitted without a namespace under untrusted rendering; avoid
using them as trusted DOM property names.

With `table_attributes: true`, a table may now have a plain `: Caption` line
without an ID or class. The existing `: Caption {#id .class}` and
`: {#id .class}` forms remain available. `table_colgroup` and
`table_column_names` still generate classes on `<col>` elements, not on the
table or figure. See [table layout](table-layout.md) for those options.
