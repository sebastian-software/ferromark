# Spec fixtures

- `commonmark-0.31.2-spec.txt` is the unmodified CommonMark specification
  `spec.txt` (version 0.31.2) by John MacFarlane, vendored from
  <https://github.com/commonmark/commonmark-spec/blob/0.31.2/spec.txt> and
  licensed under [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
  It is used here only as test data for the conformance suite in
  `tests/spec_commonmark.rs`.
- `gfm-extensions-spec.txt` holds a historical 24-example subset of the
  GitHub Flavored Markdown spec (0.29-gfm, also CC-BY-SA 4.0) —
  tables, task list items, strikethrough, autolinks, and disallowed raw
  HTML — driven by `tests/spec_gfm.rs`. The disallowed-raw-HTML section
  needs the opt-in `disallow_raw_html` renderer option, which the suite
  enables for that section only.
  This fixture is preserved unchanged. The
  [2026-09-14 audit](../../../../docs/reports/2026-09-14-compatibility-audit/README.md)
  found 28 extension examples on the official page with the same version label:
  22 unchanged, two changed, and four additional examples. A frozen complete
  snapshot is tested separately by the audit harness.
- `commonmark-known-failures.txt` and `gfm-known-failures.txt` track the
  examples that do not render per their spec. The conformance tests fail
  when an entry starts passing (remove the line) or when a conforming
  example regresses. Regenerate with:

  ```sh
  UPDATE_SPEC_BASELINE=1 cargo test -p ferromark_renderer --test spec_commonmark
  UPDATE_SPEC_BASELINE=1 cargo test -p ferromark_renderer --test spec_gfm
  ```

  The remaining `gfm <n> Autolinks` entries in the CommonMark baseline
  are not defects: with the GFM profile enabled, the autolink extension
  deliberately linkifies bare URLs/emails that plain CommonMark keeps as
  text (spec examples 608/611/612). Core mode matches all 652 examples under
  the inherited normalizer, which can hide significant URL and inline-code
  whitespace differences. This is not an unrestricted conformance claim.
