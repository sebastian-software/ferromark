# Renderer fixes from the final v2 review

The final pre-release review of Ferromark v2 found four renderer defects: two
unbounded computations reachable from ordinary fence metadata, one arithmetic
overflow, and two link-conversion paths that disagreed about what an empty
`base_url` means. This record covers the renderer work package. Only the
`base_url` item changes rendered output for input that previously rendered at
all; the other three either could not finish, panicked, or wrapped.

## Decisions

- **Bound code-annotation line ranges by the block.** `parse_line_numbers`
  expanded `start-end` one line at a time and checked `SmallVec::contains` on
  every push, so `annotate="highlight:1-40000"` on a one-line block cost 167 ms
  in release and quadrupled per doubling, while the VitePress form
  `{1-999999999999}` never returned. A line number past the last line annotates
  nothing — the appliers look it up with `get_mut` and skip a miss — so the
  parser now takes the block's line count, drops entries that start past the
  end, clamps the remaining ends to it, and expands sorted, merged ranges once.
  Both the emitted list and the expansion work are bounded by the block's line
  count, and the rest is linear in the metadata. A bound too large for `usize`
  saturates instead of failing to parse, so such a range is clamped like any
  other rather than dropped.

  The result is still ascending and de-duplicated, which is what the previous
  push-then-sort loop produced, so every valid selection renders byte for byte
  as before. Clamping deliberately does not move a range's *start*: `9-12` on a
  three-line block continues to annotate nothing rather than sliding onto the
  final line.

- **Saturate the code-block line-number start.** `:line-numbers=<n>` accepts any
  `usize`, and rendering added the line offset to it. At
  `:line-numbers=18446744073709551615` that panicked under overflow checks and
  wrapped to `data-line-number="0"` in release. The written line number now
  saturates, so the last representable number repeats instead of wrapping, and
  the `id`/`data-line-anchor` values of line links use the same saturated
  number. Starts that can be represented count up exactly as before.

- **Keep root-absolute links root-absolute under an empty base.** With
  `convert_md_links: true` and `base_url: ""` (`linkBasePath: ""` in the Node
  binding), `join_base_route` dropped the leading slash, so `[a](/b.md)` became
  `b/index.html` — resolved against whatever directory the page is served from —
  while `apply_base_to_root_absolute_url` left `[c](/d)` as `/d` under the same
  configuration. The two paths now agree: an empty base adds no prefix, and a
  root-absolute source link stays root-absolute, so `/b.md` becomes
  `/b/index.html`. A base without a leading slash is still taken verbatim, which
  keeps a deliberately relative base working, and relative links never see the
  base at all.

  `2026-09-15-borrowed-renderer-options.md` records that "an empty `base_url` is
  not the same as `/`". That statement is about the option conversion: an empty
  value is carried into `RendererOptions` verbatim instead of being replaced by
  a default, and it still is. What changes here is only the routing semantics of
  that value, which now match the root-absolute path that was already correct:
  an empty base and `"/"` both prefix nothing, and a configured base such as
  `"/docs"` remains distinguishable from both.

- **Keep the code-block index out of provisional fragments.**
  `render_provisional_fragment` and `render_provisional_fragment_with_hooks`
  promise to render "without mutating committed state", and
  `reset_incremental_state` to clear "renderer state that spans incremental
  fragments", but `code_block_index` was neither snapshotted nor cleared; only
  the one-shot `prepare_render` reset it. A provisional fragment therefore
  renamed the next committed fragment's generated line-link prefixes
  (`code-js-1` became `code-js-2`), and a reset did not restart the numbering.
  The field is now saved and restored around both provisional entry points and
  cleared by the reset, next to the other cross-fragment state. Committed
  fragments keep advancing it, which is the documented behavior they share with
  heading-ID de-duplication.

## Intended output change

Exactly one rendered-output change is intended, for `convert_md_links: true`
together with an empty `base_url`: a root-absolute Markdown link (in Markdown
syntax or in a raw HTML `href`/`src`) now keeps its leading slash.

| Source | Base | Before | After |
| --- | --- | --- | --- |
| `[a](/b.md)` | `""` | `b/index.html` | `/b/index.html` |
| `[c](/d)` | `""` | `/d` | `/d` |
| `[a](/b.md)` | `"/"` | `/b/index.html` | `/b/index.html` |
| `[a](/b.md)` | `"/docs"` or `"/docs/"` | `/docs/b/index.html` | `/docs/b/index.html` |
| `[e](./f.md)` | any | `../f/index.html` | `../f/index.html` |

The default `base_url` is `"/"`, so this is reachable only by configuring an
empty base explicitly. `tests/renderer_option_strings.rs` pinned the previous
output; its `base_url` case now pins the corrected rule and keeps checking that
a configured base is neither dropped nor substituted.

The overflow and range fixes change output only for input that previously
panicked, wrapped, or did not terminate.

## Verification

Each defect was reproduced before it was fixed, in a release build: the
annotation range measured 2.8 ms, 10.4 ms, 42.4 ms and 167.1 ms at 5,000,
10,000, 20,000 and 40,000 lines (fourfold per doubling); the line-number start
rendered `data-line-number="0"` on the second line; and the empty base rendered
`href="b/index.html"` next to `href="/d"`.

`tests/renderer_review_fixes.rs` covers all four: bounded ranges for both
syntaxes with a one second guard budget, range ends inside, at, and past the
block, a range starting past the block, list de-duplication, saturating and
ordinary line-number starts, line-link anchors, the base matrix (`""`, `"/"`,
`"/docs"`, `"/docs/"` with `.md` and non-`.md` root-absolute links plus a
relative `.md` link, and the raw-HTML path), and the incremental code-block
index across provisional, committed, hook-driven and reset renders. The guard
tests run in release as well as debug. Every other snapshot suite and the
CommonMark/GFM conformance suites are unchanged.

In release the guarded inputs now render in 2.3 µs (`highlight:1-40000` on a
one-line block, previously 167 ms), 95 µs (`highlight:1-10000000`), and 4.5 µs
(`{1-999999999999}`, previously non-terminating).

The Node binding maps `linkBasePath` onto `base_url` and turns link conversion
on, so `linkBasePath: ""` inherits the corrected rule; `node/ferromark/test`
checks it next to `"/"` with a built native addon.
