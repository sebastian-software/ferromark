# Optional writing syntax and reference links

`ParserOptions::highlight`, `ParserOptions::insertions`, and
`ParserOptions::inline_footnotes` default to `false` in every preset.
`ParserOptions::allow_link_refs` defaults to `true`. Node exposes these
switches as `highlight`, `insertions`, `inlineFootnotes`, and `allowLinkRefs`
on all rendering entry points, including `Renderer` and hooks.

## Configure the Rust parser

```rust
use ferromark::{Allocator, HtmlRenderer, Parser, ParserOptions};

let source = "This is ==important==, ++new++^[An explanatory *note*.].";
let allocator = Allocator::for_source_len(source.len());
let options = ParserOptions {
    highlight: true,
    insertions: true,
    inline_footnotes: true,
    ..ParserOptions::default()
};
let document = Parser::with_options(&allocator, source, options).parse()?;
let html = HtmlRenderer::new().render(&document);
```

Use `..ParserOptions::gfm_spec()` to retain GFM parsing options instead. Set
`allow_link_refs: false` separately when reference syntax should remain visible.
The source and allocator must outlive the parsed document.

## Inserted text

```js
import { toHtml } from 'ferromark'

toHtml('The timeout is ~~30 seconds~~ ++60 seconds++.', { insertions: true })
// <p>The timeout is <del>30 seconds</del> <ins>60 seconds</ins>.</p>\n
```

The option is off by default, including in CommonMark/GFM profiles and Node's
defaults. When enabled, `++...++` parses its body as ordinary inline Markdown
and renders a semantic `<ins>` element. Emphasis, links, code, and other inline
features can appear inside it. A single plus and an unmatched delimiter remain
literal. Delimiters use CommonMark-style whitespace and punctuation flanking;
an odd run leaves one literal plus outside the remaining pairs. On a
close-only odd run, that plus follows the closing insertion tags, matching the
[markdown-it-ins reference](https://github.com/markdown-it/markdown-it-ins).
Soft line breaks may occur inside an insertion, but it cannot cross a paragraph
or block boundary. Escapes, code, math payloads, HTML tags/attributes, URLs,
link destinations, and MDX payloads keep their own parsing rules.

With GFM autolinks enabled, `http://`, `https://`, and `ftp://` URL candidates
keep `++` inside their link text. A pair exactly at a scheme URL's end can close
an insertion opened before it. Fuzzy email and `www.` autolinks are created
after inline parsing, so their plus delimiters remain active insertion syntax.

The `++...++` notation is reserved for inserted text: `++ctrl+c++` is an
insertion regardless of its contents, not keyboard markup. This extension needs
no transforms package.

The public AST adds `Node::Insertion` and `Insertion`, with inline children and
a delimiter-inclusive source span. `Visit::visit_insertion` and
`walk_insertion` visit those children. Heading IDs and `transform()` metadata
use the inserted words without the delimiters.

## Marked text

```js
import { toHtml } from 'ferromark'

toHtml('This is ==**important**==.', { highlight: true })
// <p>This is <mark><strong>important</strong></mark>.</p>\n
```

Exactly two unescaped `=` characters open and close a mark. Runs of one or
three or more remain literal. Delimiters use the same whitespace/punctuation
flanking as `*` emphasis; whitespace immediately inside an opener or closer
prevents that boundary from matching. Intraword marks are allowed.

Marks may contain emphasis, links, code, and soft line breaks. A delimiter
inside code, an HTML tag, or a link destination cannot close an outer mark.
Marks cannot cross block boundaries. Setext heading recognition retains its
normal block-level precedence. Raw HTML policy is unchanged: `<mark>` is
renderer-generated markup, so it also works with raw HTML disabled in Node.

The AST exposes `Node::Highlight`, `Highlight`, `Visit::visit_highlight`, and
`walk_highlight`. The node retains delimiter-inclusive source spans. Heading
text, image alt text, autolinking, source maps, and HTML hooks traverse its
children. This parser option is separate from code syntax highlighting.

## Inline notes

```js
toHtml('A statement^[An explanatory *note*.]', { inlineFootnotes: true })
```

Inline notes work independently of `footnotes` and `allowLinkRefs`. Their
single-paragraph bodies accept inline formatting, links, code, escaped closing
brackets, and balanced brackets. Empty or unclosed notes remain literal.
`^[...]` takes precedence over superscript when both are enabled. Inline notes
inside another inline note remain literal rather than creating nested notes.
Image alt text does not create document footnotes.

The root parser lowers each note into existing `FootnoteReference` and
`FootnoteDefinition` nodes, retaining original source spans and setting `label`
to `None`. Generated numeric identifiers skip explicitly defined footnote
labels. Definitions are appended at document end. Lowering happens after
speculative link parsing and source remapping, so rejected links cannot leave
orphan definitions and container notes share one identifier sequence.

The HTML renderer's existing policy applies. `semantic_footnotes: true` uses
one ordered section and numbers reference and inline notes together in encounter
order. The legacy renderer displays identifiers and uses independent footnote
containers; mixed notes may therefore display nonsequential identifiers.
No new renderer policy is silently enabled.

## Disable reference links

```js
toHtml('[name]\n\n[name]: /destination', { allowLinkRefs: false })
```

Disabling reference links preserves full, collapsed, and shortcut reference
syntax as ordinary Markdown, including image references. Definitions remain
visible Markdown rather than being consumed. Inline links and images, wiki
links, and reference/inline footnotes keep their independently selected behavior.

The parser skips link-definition collection and resolution. When reference
footnotes are also disabled, it bypasses the fused definition prepass completely.
This is a dialect choice: on documents containing references, faster rendering
also reflects changed AST and HTML output, not an equivalent-output optimization.

## Performance evidence

The [measured results](reports/2026-09-15-optional-writing/README.md) separate
disabled, enabled-but-unused, and active syntax. The
[measurement harness](../benchmarks/optional-writing/README.md) compares a
frozen pre-change core, the candidate with both extensions off, and candidate
options on/off. It separates unused options from active syntax and records
HTML equality, AST equality, source bytes, raw paired timings, and build identity.

The retained implementation meets the disabled-option target on representative
workloads. Enabled-but-unused options have a modest cost on ordinary documents
and higher costs on literal marker decoys. Actual marks and notes also require
additional AST and rendering work; an inline note triggers document-wide
identifier collection and lowering. Keep extensions disabled when their syntax
is not part of the application's Markdown dialect.

Reference-link disabling was effectively neutral on ordinary text in the GFM
run. The CommonMark profile showed modest savings when reference footnotes were
also off and the whole definition prepass could be skipped. Reference-heavy
inputs change output, so those larger savings are not equivalent-output wins.
The report preserves row ranges and repeated measurements on one ARM64 host.
