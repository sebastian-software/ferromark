# Compatibility audit

This reporting tool compares the current root crates with CommonMark 0.31.2
and a frozen copy of the official GFM website. It also checks all 652
CommonMark inputs with CRLF and lone CR, and runs 30 targeted probes.
It does not modify the production crates or their inherited conformance baselines.

The separate [cmark oracle harness](CMARK.md) compares 106 complex inputs with
pinned official cmark/cmark-gfm builds. An additional 256 generated tilde/inline
combinations cover the delimiter failure families. Both sets also run as Rust
regressions against frozen reference output, independently of the spec gate.

From the repository root, with the pinned Rust toolchain, Python 3.11+, and
the dependencies already downloaded by the regular workspace build:

```sh
python3 benchmarks/compatibility-audit/build.py /tmp/ferromark-audit-build
python3 benchmarks/compatibility-audit/run.py \
  /tmp/ferromark-audit-build/target/release/compatibility-audit-worker \
  benchmarks/compatibility-audit/fixtures/gfm-0.29-2026-09-14.html \
  /tmp/ferromark-audit-results --fail-on-differences
python3 -m unittest discover -s benchmarks/compatibility-audit -p 'test_*.py'
```

Both output directories must be new. The offline build reuses root lockfile
versions and verifies that no new registry versions were resolved. It creates
a separate Cargo workspace and release binary; it does not change root dependencies.
The worker allocates a fresh arena, parses, and renders each JSONL request.

**The corrected implementation passes `--fail-on-differences`.** The flag
checks configured CommonMark, the current GFM extension sections, line endings,
and normative targeted probes. It also rejects parser errors/panics in any lane
and verifies that the deliberately incorrect normalizer counterexamples are
rejected. Deliberate default-profile differences are diagnostic. Without the flag the
command generates a report and exits successfully; that is not a conformance pass.

The optional `--upstream-worker PATH` uses the frozen six-engine worker from
the [native benchmark](../native-comparison/README.md) to compare selected
reproductions with original OX-Content and the previous V2 build. Its GFM lane
only enables tables, strikethrough, and tasks. It cannot establish inheritance
for the extended `mailto:`/`xmpp:` autolink failures.

## Profiles and interpretation

`commonmark` pairs `ParserOptions::commonmark()` with
`HtmlRendererOptions::commonmark()`: bare-URL autolinking, link target attributes,
heading IDs, callouts, inline TOC, and fence metadata cleanup are disabled.
`gfm` pairs `ParserOptions::gfm_spec()` with `HtmlRendererOptions::gfm()`, adding
the formal GFM extensions and tagfilter without footnotes.
`gfm-no-tagfilter` omits that last policy. `default` and `gfm-preset` retain
the public renderer defaults; the latter also retains GFM's footnotes setting.
`mdx` enables MDX syntax on the configured CommonMark profile. These explicit
profiles were added in the second correction batch; historical reports retain
their original configuration and output.

Every example keeps its input, expected HTML, actual HTML, errors, and both
the current spec normalizer and conservative comparison result. Fields named
`inherited_*` in the historical audit became `spec_*` after the normalizer
was strengthened. Heading-ID differences are
diagnostic, not silently admitted as exact output. Neither comparator is a
browser DOM implementation or a proof for arbitrary raw HTML/CSS. In particular,
the conservative helper still has limitations with duplicate attributes and
foreign-content whitespace. Read the
[original findings and classified exceptions](../../docs/reports/2026-09-14-compatibility-audit/README.md)
and [the fixes](../../docs/reports/2026-09-14-correctness-fixes/README.md)
before treating a mismatch count as a parser failure count.

## Fixture provenance

`fixtures/gfm-0.29-2026-09-14.html` is the unmodified response downloaded from
<https://github.github.com/gfm/> on 2026-09-14. Its SHA-256 is
`b153d814fdfc8624bb6da7449162c1cd707a637f7d1c1b636eb44b9cf63fa220`.
The page labels itself **0.29-gfm (2019-04-06)**, but differs from our older
24-example subset. `fixtures/gfm-examples.json` is mechanically extracted:
677 examples, 28 from extension sections, decorative space spans removed and
tab arrows converted to tabs. The report records the 22 unchanged examples
shared with the old subset; two changed and four additional examples remain visible.

The GFM specification is based on John MacFarlane's CommonMark specification
and licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
The HTML snapshot, extracted fixtures, and reproduced specification examples in
the result files retain that license and attribution; they are not relicensed
under this repository's MIT license. CommonMark test input is loaded directly
from the [separately attributed existing fixture](../../crates/ferromark_renderer/tests/spec_fixtures/README.md).
