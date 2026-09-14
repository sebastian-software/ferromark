# Explicit CommonMark/GFM profiles

## Intended contract

The existing `Default`/`new()` presets retain Ferromark product conveniences. New named profiles make dialect comparisons explicit:

- `ParserOptions::commonmark()` is the default parser configuration.
- `ParserOptions::gfm_spec()` enables GFM task lists, tables, strikethrough, and autolinks, while leaving Ferromark footnotes disabled. The existing `ParserOptions::gfm()` remains the convenience preset and keeps footnotes enabled.
- `HtmlRendererOptions::commonmark()` disables renderer URL autolinking/blank targets, heading IDs, callouts, inline TOC replacement, and VitePress fence metadata cleanup. It preserves raw HTML passthrough.
- `HtmlRendererOptions::gfm()` starts from that strict renderer profile and enables GFM tagfilter through `disallow_raw_html`.

The four renderer switches are also public fields so callers can compose a profile. Defaults are all enabled to preserve existing output. Heading permalinks are suppressed when heading IDs are off; TOC marker replacement is likewise suppressed when either TOC or IDs are disabled, avoiding links to missing anchors.

## Regression coverage

`crates/ferromark_renderer/tests/profiles.rs` checks strict heading/callout/TOC/fence behavior, GFM tagfilter and no footnotes, output equality across normal, hooks, and incremental rendering, and preservation of old defaults. The code-fence probe uses ```` ```js{1} ```` and confirms the strict class retains `js{1}` while the default metadata path remains covered by the existing renderer code tests.

## Feedback loop

The initial red run is preserved in `profiles-before.log`; it was blocked by a concurrent syntax error in `parser/source_normalization.rs`, before profile API/test execution. After that parser file was repaired by its owner, the focused profile test passed:

```
cargo test -p ferromark_renderer --test profiles --locked
# 4 passed
```

The focused existing code-render tests also passed:

```
cargo test -p ferromark_renderer --lib test_render_code_block --locked
# 8 passed
```

## Parity hardening

A focused heading-attribute probe found that the hook renderer emitted an explicit heading ID but dropped AST heading classes. The hook path now emits escaped classes in the same order as normal rendering. The test also compares normal, `render_borrowed`, hooks, and incremental output, then enables `heading_permalinks` and `inline_toc` while keeping `heading_ids` disabled; explicit IDs, generated permalinks, and TOC replacement are all absent while heading classes and the literal marker remain.

The updated focused test run is recorded in `profiles-after-attribute.log`:

```
cargo test -p ferromark_renderer --test profiles --locked
# 5 passed
```
