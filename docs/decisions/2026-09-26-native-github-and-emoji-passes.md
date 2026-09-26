# Native GitHub references and emoji shortcodes

- Status: Accepted
- Date: 2026-09-26
- Related: #399, #400, #404, #405, [native transform pipeline](../arch/ADR-0022-native-transform-pipeline.md)

## Context

The upstream fixture suites for `remark-github` and `remark-gemoji` record
reference behavior. They do not implement those features in Ferromark. This
decision records the native subset and its boundaries; the opt-in passes live
in `ferromark-transforms` and are also available from the Node binding.

The default parser and renderer continue to process the document as before.
Callers who need these transformations add the passes after parsing and before
derived heading metadata or rendering. Node's `passes` array keeps caller order.
The existing top-level `typography` property remains a compatibility shortcut,
but cannot be combined with `passes` because that would make pass order
ambiguous.

## GitHub references

`GitHubReferencesPass` requires an explicit `owner/repository` configuration.
The two ASCII path segments are validated before any document is changed; the
pass does not infer a repository from package metadata or Git remotes and does
not access the network, filesystem, or a callback.

The supported forms are:

- Local `#N` and case-insensitive `GH-N` issue or pull-request references,
  where `N` is a positive decimal number.
- Explicit cross-repository `owner/repository#N` and
  `owner/repository@commit` references.
- User mentions `@user` and team-shaped mentions `@org/team`.
- Bare 7–40 character hexadecimal commit IDs and `start...end` commit ranges.

The authored label remains plain text inside an ordinary `Link` node. The pass
does not adopt upstream strong mention labels, inline-code commit labels,
shortened GitHub URLs, URL callbacks, or repository inference. It skips
existing links, code, math, raw HTML content, MDX expressions, image metadata,
link destinations and titles. Bare URLs recognized by the configured renderer
matcher stay intact, including their fragments. New destinations pass through
normal HTML escaping and URL policy. A short denylist avoids linking common
English words that happen to contain 7–40 hexadecimal characters.

The reference is `remark-github@12.0.0` from the pinned registry artifact and
source revision recorded in
[`benchmarks/remark-github-oracle`](../../benchmarks/remark-github-oracle/README.md).
The oracle's normalized trees are checked against the pinned upstream package;
native tests independently check URLs, labels, boundaries and protected
content.

## Emoji shortcodes

`EmojiShortcodesPass` uses the exact `gemoji@8.1.0` `nameToEmoji` map: 1,913
case-sensitive aliases, including `:+1:`. Unknown aliases remain unchanged.
The pass does not add emoticons, spacing, HTML wrappers, or accessibility
markup. If an unknown shortcode overlaps a known one, scanning can still match
the later known shortcode, as in `:other:smile:` → `:other😄`.

The source is the integrity-pinned MIT npm artifact recorded in
[`benchmarks/native-transform-oracles`](../../benchmarks/native-transform-oracles/README.md).
`scripts/generate-gemoji-data.mjs` deterministically writes the sorted Rust
table; CI checks that output against the installed, exact package version. The
full notice is shipped in `transforms/data/LICENSE.gemoji`. The runtime uses
only the compiled map; it has no JavaScript dependency, download, or lookup
outside the opt-in pass.

Code, math, raw HTML content, MDX expressions, image metadata, link
destinations, titles, and renderer-recognized bare URLs remain unchanged.
Ordinary non-URL link labels are prose and can be transformed. Emoji in a
heading changes its rendered text before the renderer derives the heading ID.

## Shared protected-content behavior

The prose traversal collects source spans for text between paired inline raw
HTML tags before it edits the AST. Typography, GitHub references, and emoji all
leave those spans unchanged. The same passes skip block HTML nodes, code, math,
MDX expressions and image metadata; URL-sensitive passes use the matcher's
same `HtmlRendererOptions` as the renderer. Replacements retain the covered
source span. Applying a pass twice does not re-link generated links or remap
already replaced shortcodes.

This discovery corrected an earlier test gap: skipping the raw HTML tag nodes
alone did not protect the text between them. The shared traversal and the
regression cases now enforce the actual boundary for all three built-ins.

## Disabled-path cost

No pass is configured by default. The Node renderer bypasses the pipeline when
its pass list is empty, so it does not traverse prose or scan URLs on the
default path. URL protection runs only after a caller enables a URL-sensitive
pass.

The repository's N-API boundary benchmark compared the exact parent commit
`8f0549d` with the GitHub/emoji changes on Node 24.21.0, Rust 1.95.0, and an
Apple M1 Ultra in one A/B run. Both builds had PGO disabled; each run measured
all 57 broad corpus documents with 15 paired rounds, 2 ms batches, and 20 ms
warmup. In the no-options lanes, the corpus-summed median time was 1.12% lower
for `toHtml` and 0.46% lower for `Renderer.toHtml`; the isolated Rust core was
0.86% and 0.83% higher, respectively. Per-document medians were -0.90% and
+0.22%. The empty-Markdown `{}` options lane increased from 250.0 ns to 265.1
ns. That 15.1 ns fixed difference is about 0.03% of the corpus's 53.1 μs mean
render time; the full corpus was not separately timed with an options object.

## Validation

Rust integration tests cover the supported forms, parser-fragmented text runs,
URL and raw-HTML protection, link nesting, source spans, repeated application,
and heading IDs. Node tests exercise the same output through every public
rendering entry point and verify `passes` getter order and configuration
errors. Pinned Remark outputs remain reference evidence; differences in native
labels and scope are described above rather than treated as parity failures.
