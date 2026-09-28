# Optional technical abbreviation markup

Use `HtmlRenderer::with_abbreviations` or
`to_html_with_options_and_abbreviations` in Rust, or set
`autoAbbreviations: true` in Node.js, to wrap eligible technical terms in HTML
`<abbr>` elements. The option is off by default in Node. Ferromark does not
infer expansions from surrounding documents or learn them from earlier renders.

```rust
use std::collections::BTreeMap;

use ferromark::{
    AbbreviationOptions, HtmlRendererOptions, ParserOptions,
    to_html_with_options_and_abbreviations,
};

let mut abbreviations = AbbreviationOptions::default();
abbreviations.overrides = BTreeMap::from([
    ("API".to_string(), Some("Custom API title".to_string())),
    ("GraphQL".to_string(), Some(String::new())),
]);
let html = to_html_with_options_and_abbreviations(
    "API and GraphQL use HTTPS.",
    ParserOptions::default(),
    HtmlRendererOptions::default(),
    abbreviations,
)
.unwrap();
assert!(html.contains("<abbr title=\"Custom API title\">API</abbr>"));
assert!(html.contains("<abbr>GraphQL</abbr>"));
assert!(html.contains("<abbr title=\"Hypertext Transfer Protocol Secure\">HTTPS</abbr>"));
```

```js
import { Renderer } from "ferromark";

const renderer = new Renderer({
  autoAbbreviations: true,
  abbreviations: {
    API: "Custom API title",
    GraphQL: "",
    XYZ: null,
  },
});

renderer.toHtml("API and GraphQL use HTTPS. XYZ is suppressed.");
```

`HtmlRenderer::with_options_and_abbreviations` and the
`HtmlRenderer::with_abbreviations` builder offer the same configuration for
AST-based rendering. In Rust, explicitly choosing either abbreviation API
enables matching, including with an empty override map. Create the non-exhaustive
options type with `AbbreviationOptions::default()` and set its public
`overrides` field. In Node, supplying
`abbreviations` alone does not enable matching; set `autoAbbreviations: true`.
The Node `Renderer` builds its dictionary once and reuses it across calls; each
document's matches remain independent.

## Matching rules

Matching is case-sensitive. The built-in dictionary is tried before the
uppercase heuristic, and the caller map takes precedence over both.

- A heuristic candidate contains ASCII uppercase letters and optional ASCII
  digits, and it must contain at least two uppercase letters. Thus `API`,
  `HTTP2`, and `UTF8` qualify; `A` and `42` do not.
- A candidate must have a token boundary on each side. Unicode letters and
  digits and `_` count as identifier characters. Ferromark does not extract
  `API` from `xAPI`, `APIclient`, `ÄAPI`, `APIÄ`, or `API_TOKEN`.
- A final lowercase `s` is treated as a plural suffix when the following
  character is a token boundary. Ferromark wraps the singular token and leaves
  the `s` outside: `APIs` becomes `<abbr title="Application Programming
  Interface">API</abbr>s`.
- Hyphens and slashes are boundaries. A complete built-in term takes
  precedence, so `HTTP/2` and `UTF-8` are each wrapped as one term. Other
  compounds are considered at their separator boundaries; for example,
  `API-like` wraps `API` and leaves `-like` unchanged.
- A caller map can register exact, case-sensitive terms outside the heuristic,
  including mixed-case names such as `GraphQL`. These entries use the same
  token-boundary rule.

Each occurrence is processed, including text inside emphasis and Markdown link
labels. URL text remains unchanged, link destinations stay in attributes,
image alt text stays plain text, and generated markup never enters an HTML
attribute.

## Dictionary overrides

| Map value | Result |
| --- | --- |
| Key absent | Use a built-in title, or wrap a qualifying unknown candidate without a title |
| Nonempty string | Wrap with that title, overriding a built-in title |
| Empty string | Wrap without a `title`, overriding a built-in title |
| `null` | Leave the exact term as plain text, even if the heuristic or built-in dictionary recognizes it |

Rust uses the non-exhaustive `AbbreviationOptions` type with an
`overrides: BTreeMap<String, Option<String>>` field; create it with
`AbbreviationOptions::default()` and set that field. `Some("")` means a bare
wrapper and `None` suppresses the term. Node uses
`Record<string, string | null>` with the same meaning. Matching is deterministic;
render order and other documents do not modify the dictionary.

The maintained initial dictionary is `AI`, `API`, `ASCII`, `CLI`, `CPU`, `CSS`,
`DNS`, `DOM`, `GPU`, `HTML`, `HTML5`, `HTTP`, `HTTP/2`, `HTTPS`, `IDE`, `IP`,
`JSON`, `JWT`, `MIME`, `RAM`, `REST`, `RPC`, `SQL`, `SSH`, `SSL`, `TCP`, `TLS`,
`URI`, `URL`, `UTF-8`, `UTF8`, `XML`, and `YAML`. These titles are authored
project data; the implementation uses no network lookup or imported
abbreviation corpus.

## Protected content and limitations

Matching happens while HTML is rendered. The parsed AST is unchanged, so source
spans, heading IDs, `transform()` heading text, and other plain-text metadata
continue to use the authored Markdown. No abbreviation annotation is exposed
as an AST node.

Code spans and blocks, math, raw HTML (including when raw HTML is escaped),
authored `<abbr>` elements, MDX `<abbr>` children, MDX attributes and
expressions, front matter, image metadata, URL text, and link destinations do
not receive generated wrappers. Existing markup is not nested or repeated.
When `autoAbbreviations` is on, link labels are still eligible prose, but a
visible URL inside a link is kept literal to prevent nested anchors.

This heuristic can mistake all-uppercase words for abbreviations. In the
representative fixture
[`benches/fixtures/abbreviations.md`](../benches/fixtures/abbreviations.md),
`MUST` is deliberately recognized even though it is a normative word, and
`README` also qualifies as an unknown uppercase token. The benchmark suppresses
`README` with a `null` override and gives `ID` an explicit expansion. Use a
`null` entry for known false positives in your own content. This fixture is an
evaluation sample, not a recognition-rate or accuracy claim.

The matcher and URL boundary index are prepared when an enabled renderer is
constructed. Disabled renderers build neither and do no abbreviation scan.
`benches/abbreviations.rs` separates disabled rendering, enabled documents with
matches, enabled documents without candidates, and repeated renders through
one renderer.

See the [Rust renderer options](rust-api.md#technical-abbreviations),
[Node.js options](../node/ferromark/index.d.mts), and the
[accepted decision](decisions/2026-09-27-optional-abbreviation-markup.md).
