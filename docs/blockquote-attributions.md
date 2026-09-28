# Block quote attributions

`blockquote_attributions` (Rust) and `blockquoteAttributions` (Node.js) attach
one visible source line to a block quote. The option is off by default and in
every preset except `ParserOptions::ffm()`; it does not require image captions
or bracketed spans.

```markdown
> The parser keeps the whole passage inside the quote.
: Jane Doe, **author** [profile](https://example.org) {#source .byline}
```

When enabled, Ferromark represents this as a `Figure` whose content is the
original `BlockQuote` and whose caption is parsed inline Markdown. HTML uses a
`<figure>` containing the `<blockquote>` followed by a `<figcaption>`. It does
not infer a URL or emit `<cite>` or a `cite` attribute. Inline and reference
links use the ordinary link parser and the renderer's URL policy.

## Boundaries

The source must be a nonempty line beginning with `:` and whitespace. It must
sit outside the quote, immediately after it or after one blank line. At most
three leading spaces are accepted. When enabled, an eligible immediate source
line ends a lazy quote paragraph; when disabled, CommonMark lazy continuation
is unchanged. The source and quote must share the same parent container, such
as a list item; nested quotes attach to the nearest eligible quote. In a list,
indent the source to keep it inside the item:

```markdown
- > Excerpt
  : Jane
```

An outdented `: Jane` lazily continues the quote in its list item and does not
attach. This keeps the list intact when another item follows. The source is
one line. A larger gap,
escaped colon, empty source, attribute-only source, or malformed trailing
attribute block does not create a figure. A recognized
`[!NOTE]`-style marker in the parsed quote paragraph keeps the quote out of
attribution parsing, including when nested in a list or another quote and when
the HTML renderer has `callouts` disabled. Entity-encoded opening brackets are
recognized too. Code, headings, link definitions, and resolved links are not
callout markers. Unrecognized markers remain
ordinary quote text and may receive an attribution. A following definition
list keeps its normal parse.
An empty quote cannot receive an attribution.
Fenced and indented code, raw HTML, math, and MDX inside the quote retain their
normal parsing boundaries.

A trailing `{#id .class}` block applies to the generated `<figure>` without
enabling image captions. Key/value attributes use the shared grammar and
require `extended_attributes` (Rust) or `extendedAttributes` (Node.js); plain
source lines and ID/class suffixes work without that flag. IDs share the
renderer’s document-wide collision planning with heading IDs. Caption text uses
the same HTML escaping, link handling, and URL policy as other inline Markdown.
Visitors, transforms, and render hooks see both the quote content and caption.
Figure spans cover the quote, optional blank line, and source line in the
original document.

```rust
use ferromark::{HtmlRenderer, Parser, ParserOptions};

let source = "> A short passage.\n: Jane Doe";
let allocator = ferromark::Allocator::new();
let document = Parser::with_options(
    &allocator,
    source,
    ParserOptions {
        blockquote_attributions: true,
        ..ParserOptions::gfm()
    },
).parse()?;
let html = HtmlRenderer::new().render(&document);
assert!(html.contains("<figcaption>Jane Doe</figcaption>"));
```

```js
import { toHtml } from "ferromark";

const html = toHtml("> A short passage.\n: Jane Doe", {
  blockquoteAttributions: true,
});
```

The parser checks candidate source lines only when the option is enabled. It
checks the prefix and optional ID/class suffix before parsing the inline
caption; ordinary documents without eligible quotes do not pay for inline
caption parsing.
