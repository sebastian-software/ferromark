# Round 2 correctness review

## Material finding: sparse table-cell map drops the old out-of-range clamp

`structure-compact-table-map.patch` changes
`table_cell_boundary_offset` from dense lookup with
`offsets.get(index).or_else(last)` to:

```text
index + count(removed_position < index)
```

Those functions agree for every generated boundary in `0..=content.len()`;
the strict `<` rule also correctly maps a boundary exactly at a removed
backslash. They diverge for an out-of-range generated index, however. For
source `a\|` the generated content is `a|`, removed positions are `[1]`, and
the old function maps index `3` to its final source boundary `3`. The new
function maps index `3` to `4`, beyond the source content. Any malformed or
future inline node that carries a span beyond the generated cell length will
therefore receive an invalid absolute span instead of the baseline's clamped
span. A downstream `Span::source_text` can panic on that result.

The candidate's tests cover valid boundaries and sparse/dense agreement but do
not lock down this old defensive behavior. Preserve the clamp (for example,
pass generated length into the helper and clamp `index` before counting), or
document and prove the stronger invariant that every remapped span is always
within generated cell bounds. This is a correctness review finding; no timing
was run.

## Renderer linear trim

No material semantic mismatch was found by inspection. The lazy per-bracket
counts preserve the original condition: for a trailing closer, the old scan
counts bytes before that closer and removes it when `closes >= opens`; the new
scan includes the closer and removes it when `closes > opens`, then decrements
the cached closer count. Punctuation is still removed first, mixed bracket
types retain independent counts, and Unicode bytes are ignored exactly as in
the baseline. The archived exhaustive mixed-tail tests cover the normal
progression. No source-span behavior is involved in this renderer-only patch.

## Archived failure reproductions

- [`markers-variant-b-short-tail-failure.diff`](markers-variant-b-short-tail-failure.diff)
  records the one-byte `{` SIMD-tail failure, test name, input, and corrective
  guards.
- [`url-simd-mixed-utf8-failure.diff`](url-simd-mixed-utf8-failure.diff)
  records the CJK-plus-backtick failure, test name, input, and corrective ASCII
  continuation check.
