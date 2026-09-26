# Optional locale-aware typography

Automatic punctuation changes stay out of Ferromark's parser and renderer.
Applications can opt in after parsing through the `ferromark-transforms`
`TypographyPass`, or use the Node.js `typography` option. With no pass or
option, authored punctuation and HTML output are unchanged.
This behavior is specified by the
[accepted typography decision](decisions/2026-09-26-optional-typography-pass.md).

## Reviewed locale rules

The pass requires exactly one explicit language: `en`, `es`, `fr`, `pt`, `de`,
`it`, `nl`, `pl`, `ru`, or `uk`. It does not detect a language, choose one by
default, or switch languages within a document. `en` uses the US English rule
set; regional variants such as `en-GB` are not separate supported codes. The
`pt` rules use a Portugal Portuguese quote convention and are independently
reviewed because the pinned Typograf reference has no Portuguese locale.

| Code | Primary quotation marks | Nested quotation marks | Apostrophes | Measurement spacing | ASCII dash sequences |
| --- | --- | --- | --- | --- | --- |
| `en` | “…” | ‘…’ | Curly | Non-breaking space | `--` and `---` become em dashes |
| `es` | «…» | “…” | Curly | Non-breaking space | Preserved |
| `fr` | « … » | “…” | Curly | Non-breaking space | Preserved |
| `pt` | «…» | “…” | Curly | Non-breaking space | Preserved |
| `de` | „…“ | ‚…‘ | Curly | Non-breaking space | Preserved |
| `it` | «…» | “…” | Curly | Non-breaking space | Preserved |
| `nl` | ‘…’ | “…” | Curly | Non-breaking space | Preserved |
| `pl` | „…” | ‚…‘ | Curly | Non-breaking space | Preserved |
| `ru` | «…» | „…“ | Straight | Ordinary space | `--` and `---` become em dashes |
| `uk` | «…» | „…“ | Straight | Ordinary space | Preserved |

Three periods become an ellipsis in each language. A spaced dash sequence in
English and Russian gets a non-breaking space before the em dash. In French,
the pass uses narrow no-break spaces inside guillemets and non-breaking spaces
before `;`, `:`, `!`, and `?`. For supported non-Russian languages, a space
between a number and one of `km/h`, `°C`, `km`, `cm`, `mm`, `kg`, `mg`, `m`,
`g`, or `%` becomes non-breaking. A double hyphen between letters is left
alone in every locale. Existing Unicode quotes, dashes, and ellipses are not
normalized.

The English, Spanish, French, German, Italian, Dutch, Polish, Russian, and
Ukrainian examples were checked against the pinned
[SmartyPants and Typograf outputs](../benchmarks/native-transform-oracles/README.md).
The native pass implements only the listed subset; it does not claim full
compatibility with either plugin. Portuguese has an independently reviewed
fixture and is not labeled Typograf-compatible.

## Protected content and order

Quote context can span ordinary inline markup such as emphasis and link labels.
The pass leaves code, math, raw HTML, MDX expressions and module payloads,
image metadata, link destinations and titles unchanged. It protects bare URLs
using the same renderer URL matcher supplied to `TransformContext`; URL
recognition is work done only when this optional pass runs. Escaped and
entity-authored straight quotes stay straight. Protected regions and block
transitions end quote pairing.

Run typography after other AST edits that need to inspect or replace source
punctuation. Run it before heading metadata, outlines, or rendering so those
derived values observe the transformed text. Reapplying the pass is
idempotent.

## Rust configuration

```rust
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser};
use ferromark_transforms::{
    TransformContext, TransformPipeline, TypographyLanguage, TypographyOptions,
    TypographyPass,
};

let source = "She said \"Hello\" -- it's 12 km...";
let allocator = Allocator::new();
let mut document = Parser::new(&allocator, source).parse()?;
let renderer_options = HtmlRendererOptions::new();
let context = TransformContext::new(&allocator, source, &renderer_options);
let mut pipeline = TransformPipeline::new();
pipeline.add(TypographyPass::new(TypographyOptions::new(
    TypographyLanguage::English,
)));
pipeline.run(&mut document, &context)?;

let mut renderer = HtmlRenderer::with_options(renderer_options);
let html = renderer.render(&document);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use `with_dashes(false)` or `with_ellipses(false)` to disable either class of
replacement. The rule table keeps the language required in Rust's type-level
configuration.

## Node.js configuration

```js
import { toHtml } from "ferromark";

const html = toHtml('She said "Hello" -- it\'s 12 km...', {
  typography: { language: "en" },
});
```

The same option works with `Renderer`, buffer output, `transform()`, and the
highlighter helpers. Omitting `typography` leaves punctuation unchanged. If
the object is present, `language` is required; an unsupported code fails with
an error. `dashes` and `ellipses` default to `true` and can be disabled
independently. See the
[Node options reference](../node/ferromark/index.d.mts).

The parser's former `smart_punctuation` option and inline English-only pass
were removed so the core has no implicit locale assumption or typography
work. The original removal decision and benchmark history are retained in the
[project report](reports/2026-09-14-ox-regression/ATTEMPTS.md).
