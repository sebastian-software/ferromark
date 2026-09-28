# Upgrading to Ferromark v3

Version 3.0 introduces Ferromark Flavored Markdown (FFM): opt-in syntax for
figures, quote attributions, shared attributes, bracketed spans, inserted text,
and guillemet digraphs, plus opt-in technical abbreviation markup. Every new
option is off by default. Documents render as before unless you enable one;
the only removal is the opt-in wiki link syntax.

The major version comes from the Rust API. The Node.js options gain new fields
and lose `wikiLinks`.

## Rust

**New `ParserOptions` fields.** `image_attributes`, `image_captions`,
`blockquote_attributions`, `extended_attributes`, `bracketed_spans`,
`insertions`, and `guillemet_digraphs` default to `false`. A struct literal
without `..ParserOptions::default()` (or another preset) no longer compiles:

```rust
use ferromark::ParserOptions;

let options = ParserOptions {
    footnotes: true,
    ..ParserOptions::gfm()
};
```

**New AST variants.** `Node::Figure`, `Node::Span`, and `Node::Insertion`
appear only when their options are enabled. An exhaustive `match` on `Node`
needs arms for them or a wildcard. `Figure` wraps an image or block quote in
`content` with inline `caption` children.

**Heading metadata moved.** `Heading::id` and `Heading::classes` are replaced
by one optional `attributes: Option<Box<ElementAttributes>>`. Read authored
values through the accessors:

```rust
use ferromark::ast::Heading;

fn describe(heading: &Heading<'_>) {
    let id = heading.explicit_id();
    let classes = heading.classes();
    println!("{id:?} {classes:?}");
}
```

**New optional fields.** `Link`, `Image`, and `Figure` carry the same
`attributes` field; it stays `None` unless an attribute option matched.
`TableAttributes` gains `attributes` for key/value entries. Code that builds
these nodes by hand must set the new fields.

**Wiki links removed.** `ParserOptions::wiki_links` is gone; `[[Page]]` is
plain text in every profile. Resolve wiki-style links in your own text
transform if you need them.

**Abbreviations.** `HtmlRendererOptions` is unchanged. Automatic `<abbr>`
markup uses the separate, non-exhaustive `AbbreviationOptions` with
`HtmlRenderer::with_abbreviations`, `HtmlRenderer::with_options_and_abbreviations`,
or `to_html_with_options_and_abbreviations`.

## Node.js

Existing option objects keep working. The new fields are `imageAttributes`,
`imageCaptions`, `blockquoteAttributions`, `extendedAttributes`,
`bracketedSpans`, `insertions`, `guillemetDigraphs`, `autoAbbreviations`, and
`abbreviations`. The `wikiLinks` option is removed; passing it throws an
unknown-option `TypeError`. The macOS package now ships for Apple Silicon only; Intel Macs
are no longer supported.

## Transforms

`ferromark-transforms` visits the new nodes, so typography, GitHub references,
and emoji shortcodes also apply inside spans, insertions, and figure captions.
The typography pass converts guillemet digraphs only when both
`ParserOptions::guillemet_digraphs` and
`TypographyOptions::with_guillemet_digraphs(true)` are set.

See the [changelog](../CHANGELOG.md) for the complete list and the
[FFM guide](https://ferromark.dev/guide/ffm) for the syntax.
