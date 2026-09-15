# First compatibility correction batch

The user authorized fixing the compatibility issues found in the
[audit](../reports/2026-09-14-compatibility-audit/README.md), including regressions
relative to Ferromark v1. These are deliberate semantic corrections, separate
from the earlier output-preserving SIMD work.

## Decisions

- Recognize LF, CRLF, and CR consistently in list-item boundaries, inline HTML,
  and angle link destinations. Inline HTML values normalize line endings to LF;
  node spans continue to address the original source. This removes duplicated
  list content and incorrect HTML/link recognition.
- Replace literal U+0000 with U+FFFD before reference collection and parsing,
  as required by CommonMark. Clean source remains borrowed. Only NUL-bearing
  input allocates replacement text and a compact offset map. Existing recursive
  span traversal applies the map once after nested source remapping, including
  MDX attributes and table/list spans.
- Percent-encode backslash, brackets, and backtick in URL attributes while
  retaining actual URL delimiters and valid IPv6 host brackets. The IPv6
  exception validates the host and never bypasses escaping for arbitrary
  bracketed text. Update scalar and SIMD classifiers together and check them
  against the independent byte-at-a-time path.
- Implement single-tilde GFM strikethrough, keep runs of three or more tildes
  literal, and respect escaped/code-span delimiters. Add the current GFM
  `mailto:` and `xmpp:` forms with contiguous address and resource boundaries.
  Explicit subscript configuration remains a separate product extension.
- Preserve meaningful code/raw-text whitespace and reserved URL escapes in the
  spec test normalizer. Parser errors and panics fail the suites directly;
  they cannot be accepted as baseline differences. Give the three intentional
  CommonMark-to-GFM autolink exceptions explicit expected outputs.

## Review and acceptance

Each corrected area has an initial failing reproduction and focused regression
tests. Root integration review additionally checks original source spans,
IPv6/HTML-attribute boundaries, invalid extended-autolink prefixes, and delimiter
contexts. The continuous suite now includes all 1,304 CRLF/CR transformations
of CommonMark's 652 examples and all 28 extension examples from the frozen
2026-09-14 GFM page. The full audit remains reproducible separately.

The old specification fixtures, expected-failure lists, and existing snapshots
are retained. New tests make the previous coverage gaps explicit. Generated
heading IDs, callouts, TOC, footnote defaults, fence metadata policy, and the
bounded MDX feature are outside this first batch; their documented product
policies are not silently changed.

Ferromark v1 is a useful comparison, not the conformance oracle. The pinned v1
main used in the native report handles several of these cases correctly, but
also differs from the spec on some CR and NUL inputs. Correctness is judged
against the frozen specification examples and their stated line/character rules.
