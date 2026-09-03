# Migrating from ferromark 0.3 to 0.7

This guide covers the breaking release boundaries from 0.3.3 through 0.7.0:
0.4 removes `Profile`, 0.5 extends the configurable inline-parser API, 0.6
changes several Rust integration types, and 0.7 raises the Rust and Node.js
runtime floors. Apply the sections in order when upgrading across more than
one release.

If you are upgrading from 0.2 or earlier, read the [0.2 migration
guide](migration-0.2.md) and the [0.3 migration guide](migration-0.3.md)
first. If you also adopt the upcoming 0.8 API changes, continue with the [0.8
migration guide](migration-0.8.md).

## Before you start

Update the Rust toolchain and, for the npm package, Node.js before changing
source code. Then run `cargo check --all-features`; this also checks the
opt-in `mdx` feature. Keep `features = ["mdx"]` in a Cargo dependency
declaration when your application uses the Rust MDX APIs: this feature remains
opt-in throughout these releases.

## 0.4: replace `Profile`

`Profile` and `Options::from(Profile)` were added in 0.3.1 and removed in
0.4. Choose the syntax surface explicitly with `Options::minimal()`,
`Options::commonmark()`, `Options::gfm()`, or `Options::default()`, then
change the public fields your application needs. The render policy remains
separate from those syntax choices.

For the former `Profile::Extended`, `Options::default()` preserves the 0.3
profile's default option set:

```rust
// Before (0.3.1–0.3.3)
use ferromark::{Options, Profile};

let options = Options::from(Profile::Extended);
```

<!-- migration-example: profile-extended -->
```rust
// After (0.4+)
use ferromark::Options;

let options = Options::default();
assert!(options.allow_html);
assert!(options.tables);
```

Do not assume that another `Profile` has a one-to-one preset replacement.
Start from the closest preset and set the required fields explicitly. For
example, a GFM-oriented configuration starts with `Options::gfm()`; select
`RenderPolicy::Trusted` separately only for Markdown you trust.

## 0.5: update configurable inline parsing

`Options` gained `inline_footnotes`, which enables Pandoc-style `^[note]`
syntax. Code that constructed an `Options` literal for 0.4 must set the new
field (normally `false` to preserve the prior behavior).

The public `InlineParser::parse_with_options` method also gained the
`inline_footnotes: bool` positional argument between `math` and
`footnote_store`. Insert `false` when retaining the 0.4 behavior, or `true`
when handling inline-footnote syntax:

```rust,ignore
// Before (0.4): `false` is the `math` argument, then the footnote store.
parser.parse_with_options(
    text, refs, true, true, false, false, false, true, false, None, &mut events,
);
```

<!-- migration-example: inline-parser-argument -->
```rust
// After (0.5+): the new `false` follows the `math` argument.
use ferromark::{InlineEvent, InlineParser};

let mut parser = InlineParser::new();
let mut events = Vec::<InlineEvent>::new();
parser.parse_with_options(
    b"plain text",
    None,
    true,
    true,
    false,
    false,
    false,
    true,
    false, // math
    false, // inline_footnotes
    None,
    &mut events,
);
assert!(!events.is_empty());
```

`InlineEvent` also gained `InlineFootnote(Range)`. Add handling for that
variant to exhaustive `InlineEvent` matches. Its range covers the note content
without the surrounding `^[` and `]`.

## 0.6: clone options and make integration matches forward-compatible

`Options` no longer implements `Copy` because it now owns the optional
`link_base_path`. Clone an `Options` value at a former implicit-copy site when
you need to retain it. `link_base_path` itself prefixes internal absolute link
destinations; it does not rewrite image sources or autolinks.

```rust,ignore
// Before (0.5): this copied `options`.
let options_for_second_use = options;
```

<!-- migration-example: options-clone -->
```rust
// After (0.6+): clone when both values are needed.
use ferromark::Options;

let options = Options::default();
let options_for_second_use = options.clone();
assert_eq!(options, options_for_second_use);
```

`FencedCodeBlock` gained a decoded `meta` field and is now
`#[non_exhaustive]`. Custom fenced-code renderers must use `..` in patterns so
future fields do not break their build. Read `meta` when a highlighter supports
info-string metadata such as `{1-3}`.

```rust,ignore
// Before (0.5): this pattern named every field.
fn render(block: ferromark::FencedCodeBlock<'_>) {
    let ferromark::FencedCodeBlock { language, code } = block;
    let _ = (language, code);
}
```

<!-- migration-example: fenced-code-pattern -->
```rust
// After (0.6+)
use ferromark::{FencedCodeBlock, FencedCodeRenderer, TrustedHtml};

struct Highlighter;

impl FencedCodeRenderer for Highlighter {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        let FencedCodeBlock {
            language,
            meta,
            code,
            ..
        } = block;
        let _ = (language, meta, code);
        None
    }
}
```

`parse`, `parse_with_options`, and `parse_with_renderer` now return document
`headings` and a `resource_limits` report alongside HTML and front matter. If
your code constructs or destructures `ParseResult`, account for those fields:

```rust,ignore
// Before (0.5)
let ferromark::ParseResult { html, front_matter } = ferromark::parse(input);
```

<!-- migration-example: parse-result-headings -->
```rust
// After (0.6+)
use ferromark::{ParseResult, parse};

let ParseResult {
    html,
    front_matter,
    headings,
    resource_limits,
} = parse("# Guide");
assert!(front_matter.is_none());
assert!(html.contains("Guide"));
assert_eq!(headings[0].text, "Guide");
assert!(resource_limits.is_empty());
```

The Node package did not remove a callable API in 0.6. It added `transform()`
and `transformWithHighlighter()` for the same HTML, heading, and front-matter
metadata; use those functions when the extra metadata is needed.

## 0.7: raise runtime prerequisites

ferromark 0.7 requires Rust 1.88 or newer. Update the compiler used by local
builds and CI before updating the crate version:

```sh
rustup toolchain install 1.88
cargo +1.88 check --all-features
```

The published `ferromark` npm package requires Node.js 22.12.0 or newer. Update
the runtime before running `npm install ferromark` or loading the native
module. Repository contributors who run the `node/` workspace also need its
pinned pnpm 11.17.0 toolchain and Node.js 22.13.0 or newer.

## Validate the completed upgrade

Run the checks that match the surfaces your project uses:

```sh
cargo check --all-features
# For npm consumers, verify that `node --version` is 22.12.0 or newer.
```

For custom renderers, include a fenced block with an info string in a local
test and verify that unknown future fields are ignored by the `..` pattern.
For inline-parser users, exercise both the `inline_footnotes: false` path and,
when enabled, a `^[note]` input.
