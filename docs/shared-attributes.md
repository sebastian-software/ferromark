# Shared attributes and bracketed spans

Enable `extended_attributes` to put key/value metadata on supported Markdown
elements, and `bracketed_spans` to wrap inline Markdown in an attributed span.
Both are opt-in and independent of image captions.

```markdown
Das französische [C’est la vie]{lang=fr} trifft es gut.

[Product **offer**]{.product lang=en sku="A-17" tracking-category=offer}

[Documentation](https://example.org){.external hreflang=en}

![Process diagram](process.svg){.diagram width=800 loading=lazy}

## Introduction {#intro .compact lang=en}

| Item |
| --- |
| Book |

: Current prices {.price-list currency=EUR}

![Alt](figure.svg)

: Figure caption {.wide sku=7}
```

The product span emits `class="product" lang="en" data-sku="A-17"
data-tracking-category="offer"`. The link's suffix targets `<a>`; the image
suffix targets `<img>`; heading metadata targets the heading; table caption
metadata targets `<table>`; and image caption metadata targets `<figure>`.
An attributed span always remains a `<span>`; `.small` does not make `<small>`.
Reference-style links and images accept the same suffix after the closing
reference label. A span can contain emphasis, code, and links.

```rust
use ferromark::ParserOptions;

let options = ParserOptions {
    extended_attributes: true,
    bracketed_spans: true,
    table_attributes: true,
    image_captions: true,
    ..ParserOptions::gfm()
};
```

```js
import { toHtml } from "ferromark";

const html = toHtml("[Bonjour]{lang=fr}", {
  bracketedSpans: true,
});
```

The grammar accepts `#id`, `.class`, and `name=value`. Values may be unquoted
without spaces or single/double quoted; `hidden=""` expresses an empty value.
Use `class="one two"` to add multiple classes. Attribute names are normalized
to lowercase; values, IDs, and classes keep their case. The last repeated ID
or key wins, while classes accumulate. Invalid syntax remains visible
Markdown. A bare class name is never accepted.

Known HTML names such as `lang`, `loading`, `width`, and `hreflang` stay as
written. `aria-*` and `data-*` also stay as written. Other names become
`data-*` in HTML, while the AST keeps their authored names. If `sku` and
`data-sku` both appear, `data-sku` wins. Markdown owns link destinations,
image sources and alt text, and quoted titles; metadata cannot override them.
The renderer escapes values and applies its normal output policy.

With `extended_attributes` off, existing heading/image/table ID and class
options retain their earlier behavior. When it is on, headings use the strict
shared grammar, so malformed suffixes remain literal. See [image captions](image-captions.md)
and [table layout](table-layout.md) for those feature gates and attachment rules.
