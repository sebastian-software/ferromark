# The public API frozen for 2.0.0

## Scope

The API-freeze review of 2026-09-17 went through every public item of the
`ferromark` crate before the stable v2 release. After 2.0.0 the surface follows
semver, so removing an item costs a major version and keeping a no-op item costs
the same to correct later. This record holds the four decisions that came out of
the review: which types stay exhaustive, what is removed, which fields become
real, and how the profile constructors are named. The bumpalo coupling is its
own record ([2026-09-17-bumpalo-public-api](2026-09-17-bumpalo-public-api.md)).

All of it is breaking for `2.0.0-rc` users and for nobody else; the
[migration guide](../migration-v2.md) lists the mechanical changes.

## Exhaustiveness

`#[non_exhaustive]` goes on `ParseErrorKind` and on nothing else.

Error categories are the one part of this API that is expected to grow from
real parsing work rather than from a feature decision. `ParseErrorKind` now has
a single variant, `NestingTooDeep`, and a future release should be able to
report a second one without a major version. Callers already have to handle an
error they cannot enumerate usefully, so the wildcard arm that
`#[non_exhaustive]` forces costs them nothing.

Everything else stays exhaustive, for the opposite reason.

`Node` has 33 variants, and a `match` over it is how a caller writes a visitor,
a transformer or a second renderer. Exhaustiveness is what tells such a caller
that a new node kind exists; behind `#[non_exhaustive]` the compiler would send
every new kind to their wildcard arm and the output would silently lose it. A
new AST node kind is a change of what this crate can represent, which is a major
version regardless of how the enum is annotated.

`ParserOptions` and `HtmlRendererOptions` stay exhaustive structs because
`#[non_exhaustive]` on a struct forbids the documented construction outside the
crate:

```rust
HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() }
```

That form — a struct literal over a profile constructor — is the whole
configuration story of this crate, and the alternative would be a builder for
roughly thirty fields. So a new option field is a major version, and the
documented recommendation is to construct options with struct-update syntax:
a field-by-field literal breaks on any field change, while `..Default::default()`
survives a removal and needs no edit for an addition.

`AlignKind`, `FrontMatterKind`, `CodeAnnotationSyntax`, `HtmlRenderControl` and
the MDX attribute enums are closed sets that a caller matches the same way
`Node` is matched, and they stay exhaustive with it.

`#![warn(missing_docs)]` is on in `src/lib.rs`, which found exactly one
undocumented public item (`CodeAnnotationSyntax`) and now keeps the next one
from reaching a release.

## Removed dead surface

Nothing below was ever produced or usefully implemented, and each item would
have been frozen by 2.0.0.

- **`Renderer` trait, `RenderError`, `RenderResult`.** `RenderError` was never
  constructed. The only `Renderer` impl returned `Ok(self.render(doc))`, and its
  `render` shadowed the inherent `HtmlRenderer::render` with a different return
  type, so a caller who imported the trait got a `Result` that could not be an
  `Err`. Rendering does not fail: it writes into a `String`. A caller who wants
  to abstract over renderers can declare their own trait; a caller who wants
  custom output uses `HtmlRenderHooks`.
- **`ParseErrorKind::UnexpectedToken` and `InvalidSyntax`.** Never constructed.
- **`ParseErrorKind::UnexpectedEof`.** Constructed in exactly one defensive
  branch in `parse_fenced_code`, reached only if block dispatch called the
  fenced-code parser without a fence character — which
  `try_parse_fenced_code_at` has already checked. The branch is now a
  `debug_assert!` plus "not a fenced code block", so the invariant is still
  stated and no public variant describes an error no input produces.
- **`Position`.** A line/column/offset triple with zero uses in the crate. The
  AST carries `Span` byte offsets, and a caller who needs lines computes them
  from the source they still own.

`#[non_exhaustive]` on `ParseErrorKind` is what makes this safe: a real error
path can reintroduce any of these variants in a minor release.

## `ParserOptions.gfm` and `HtmlRendererOptions.highlight` removed

Both fields were documented configuration that did nothing.

`ParserOptions.gfm` was metadata. The parser reads the individual extension
fields; `ParserOptions::gfm()` set `gfm: true` along with them, but setting
`gfm: true` by itself enabled no extension. A boolean that looks like a profile
switch and is not one is worse than no field.

`HtmlRendererOptions.highlight` came from OX-Content, where it switched on the
upstream highlighting crate that this fork removed. A bare boolean cannot
highlight anything without a highlighter. Syntax highlighting in v2 goes through
`HtmlRenderHooks` in Rust and `toHtmlWithHighlighter` in Node, which is also
where an external highlighter attaches — additively, within 2.x. The name also
collided with `ParserOptions.highlight`, which is the `==mark==` syntax switch
and is the field the Node `highlight` option maps to; that meaning is unchanged.

## `HtmlRendererOptions.soft_break` implemented

`soft_break` was stored on the public options and dropped in the conversion into
the internal renderer options, so the documented default `"\n"` was the only
value that ever reached the output. It is now transferred and emitted, mirroring
`hard_break`.

A soft break is the line ending that joins two lines of the same block. There is
no soft-break AST node to attach the value to: mdast has none either, and the
inline parser deliberately folds a soft line break into the surrounding text run
so that prose stays one node — it only becomes a one-character node where the
run had to stop anyway, as with a CRLF source. A hard break, by contrast, is a
`Break` node. Every line ending that reaches rendered inline text is therefore a
soft break, and the renderer substitutes the configured value for each of them.
A line ending written as a character reference such as `&#10;` is one too, and is
emitted the same way; that is documented on the field.

The value is written verbatim, exactly like `hard_break`, so `xhtml` does not
rewrite it: a caller who needs `<br />` supplies `<br />`. This keeps the two
break options consistent, and `xhtml` keeps meaning "self-close the tags the
renderer generates itself" — `<hr>`, `<img>` and `<col>`.

The conversion caches whether the configured value differs from `"\n"`, so the
default configuration pays one predictable boolean test on the text path and no
string comparison, and its output is byte-identical to before.

## Renderer profile names aligned with the parser

`ParserOptions::gfm()` was the convenience profile and `ParserOptions::gfm_spec()`
the strict one, while `HtmlRendererOptions::gfm()` was the *strict* profile with
no `gfm_spec()` at all. The same name meant opposite things on the two types, so
the correct pairing was `ParserOptions::gfm_spec()` with
`HtmlRendererOptions::gfm()` — which is what the Node binding did, and what
nobody would guess.

The renderer now follows the parser:

- `HtmlRendererOptions::gfm_spec()` is today's strict behavior — `commonmark()`
  plus `disallow_raw_html`.
- `HtmlRendererOptions::gfm()` becomes the convenience profile — `new()` plus
  `disallow_raw_html`, so it keeps heading IDs, callouts, TOC substitution, URL
  autolinking, link targets and fence metadata cleanup.

Parser names do not change: they are the ones that were already right, and both
types now answer to the same four names, so `commonmark()`, `gfm_spec()` and
`gfm()` pair with themselves.

This is a semantic change for anyone calling the renderer's `gfm()`, which is
why it lands before 2.0.0 rather than after. Every internal caller that relied
on the strict profile — the Node binding, the PGO training profiles, the
integration tests, the table-layout example and the documentation snippets —
moved to `gfm_spec()`, so no rendered output and no snapshot changes.

## Verification

Output equality is the binding constraint. Snapshots, the CommonMark 0.31.2 and
GFM conformance baselines and the Node package snapshots are unchanged, which is
what proves the profile rename and the soft-break wiring are inert at their
defaults. `tests/profiles.rs` pins the field values of `commonmark()`, `gfm()`
and `gfm_spec()` on both option structs, so a future edit cannot quietly move a
flag between profiles. Renderer tests cover `soft_break` values `" "` and
`"<br>"` with and without `xhtml`, the default leaving output untouched, and a
soft break inside emphasis, a heading and a table cell.
