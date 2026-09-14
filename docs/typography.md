# Typography belongs in an optional document transform

Decision: 2026-09-14, requested by the project owner.

Remove `ParserOptions::smart_punctuation` and its inline post-pass from the core.
The implementation chose fixed English quotation marks, applied English-oriented
apostrophe heuristics, and replaced ASCII ellipses and dash sequences. It had no
locale or configurable quotation conventions. Typography is an editorial choice
that should be applied explicitly after Markdown parsing.

The parser now preserves authored punctuation in every profile. Markdown escapes,
entity decoding, and HTML output escaping retain their existing semantics. The
removed option was disabled in all presets, so default CommonMark, GFM, and MDX
behavior stays the same. Callers that explicitly set the removed field must
remove that setting; no deprecated no-op option is retained in this unpublished
development API.

A future typography plugin should transform text nodes between parsing and
rendering, with an explicit locale or configurable quote pairs, nested-quotation
rules, spacing, and apostrophe handling. It should respect code, math, URLs,
authored typography, and embedded languages. Ellipsis and dash replacement should
be independently configurable. No plugin API or replacement transform is added
by this removal; consumers can already inspect and transform the public AST.

## Validation and historical measurements

Regression coverage checks literal ASCII and Unicode punctuation across parser
profiles, link labels, GFM autolink boundaries, and HTML escaping. Existing
specification fixtures and snapshots are unchanged.

Active SIMD/optimization timing and allocation workers no longer enable smart
punctuation in the `extensions` profile, including when built against an older
core. This keeps both sides of a new comparison on the same feature set. The
archived reports, logs, source hashes, and frozen workers retain the historical
configuration. Reproduce those measurements using their original harness
revision; do not compare their extension-profile timings directly with runs from
the updated harness. No new speed claim accompanies this feature removal.

Validation passed: workspace formatting, all-feature locked workspace tests,
Clippy with warnings denied, and the locked benchmark build. The three active
timing/allocation worker sources also compile against the updated facade in a
separate offline Cargo check.
