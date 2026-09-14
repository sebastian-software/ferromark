# CommonMark, GFM, and MDX compatibility audit

**V2 has confirmed compatibility gaps despite its passing inherited tests.**
The highest-priority findings are line-ending handling, literal NUL handling,
and URL serialization. The current GFM website also exposes missing syntax
coverage. This commit records the audit and reproductions; it does not change
production semantics or the historical conformance baselines.

Audited source: `060595be27b96130c82ecee8e1542cd26194538d`, with the same core
as `adf891ad3755f632b44a910b5e6063040400ac17`. The audit worker uses the root
dependency versions and Rust 1.95.0 on aarch64 macOS. It is separate from the
native benchmark's Rust/Bun environment. Source, fixture, binary, and lockfile
hashes are in [metadata.json](raw/metadata.json).

## What the test counts mean

The existing tests pass **652 CommonMark 0.31.2 examples** in core mode and
**24 historical GFM extension examples** under their inherited HTML normalizer.
The CommonMark suite additionally runs GFM mode with three expected autolink
exceptions. These are useful checks, but neither the fixture coverage nor the
normalizer justifies an unrestricted “100% CommonMark/GFM” claim.

The stricter audit preserves code whitespace and reserved URL escapes, and
reports heading IDs separately. It produces these results:

| Profile / fixture | Examples | Exact HTML | Serialization equivalent | Heading-ID difference only | Other differences |
| --- | ---: | ---: | ---: | ---: | ---: |
| Configured CommonMark 0.31.2 | 652 | 507 | 98 | 40 | 7 |
| Public defaults / CommonMark | 652 | 498 | 98 | 40 | 16 |
| Configured GFM / current extension sections | 28 | 20 | 3 | 0 | 5 |
| Configured GFM / full website | 677 | 514 | 101 | 40 | 22 |
| GFM without tagfilter / full website | 677 | 518 | 101 | 40 | 18 |
| Public GFM preset / full website | 677 | 503 | 101 | 40 | 33 |

“Other” is a review queue, **not a parser failure count**. Renderer choices,
older spec rules, and examples showing core behavior inside the GFM document
need separate interpretation. Full input/output and token differences are in
[results.json](raw/results.json). No parser error or panic occurred in these sweeps.

The [current CommonMark specification](https://spec.commonmark.org/0.31.2/)
uses HTML to show document structure; it does not mandate every renderer
choice. Added heading IDs and fence language-class policy therefore should not
automatically be labeled parsing violations.

## Confirmed issues and follow-up order

### 1. Line endings can alter or duplicate document content

Every CommonMark example was rerun after replacing LF with CRLF and with lone
CR: **1,304 variants, 12 differences**. CRLF fails on examples **491, 615,
616**; lone CR fails on **306, 309, 311, 313, 326, 491, 615, 616, 621**.
Only output CR/LF spelling is normalized before comparing with the original
LF output. [All twelve reproductions](raw/line-endings.json) retain exact input
and output strings.

Minimized examples, shown with escaped control characters:

| Input | Observed problem |
| --- | --- |
| `- a\r\r- b\r\r- c\r` | `- c` appears inside `b`'s paragraph and again as its own list item |
| `- a\r\r- b\r  - c\r` | The nested list containing `c` is emitted twice |
| `<a  /><b2\r\ndata="foo" >\r\n` | The second valid inline HTML tag becomes escaped text |
| `[link](<foo\rbar>)\r` | An invalid multiline angle destination becomes an anchor with CR in its URL |

[CommonMark §2.1](https://spec.commonmark.org/0.31.2/#characters-and-lines)
recognizes LF, CRLF, and CR as line endings. Source inspection found missing
CR checks in `parser/inline_html.rs` and the angle branch of
`parser/inline/link_target.rs`. The CRLF form of example 491 already rejects
the link but then fails the inline-HTML fallback; its cause is not acceptance
of a CRLF link destination. The five list cases need closer tracing of
continuation ownership and synthetic-source ranges; `list_item.rs` uses
`str::lines()`, which does not split lone CR. That is a lead, not a proven full
explanation of the duplication.

Acceptance for a fix: all 1,304 variants, plus the minimized sibling and nested
list probes, agree with their LF controls without losing source-span accuracy.

### 2. Literal NUL is not replaced

Six contexts were checked: plain text, inline code, fenced code, a link
destination, a raw HTML attribute, and a comment. All six retain U+0000 or
parse differently from the required replacement. For example `a\u0000b`
renders with the NUL still present; `[x](/a\u0000b)` stops being a link.

[CommonMark §2.3](https://spec.commonmark.org/0.31.2/#insecure-characters)
requires U+0000 to become U+FFFD. The control `a&#0;b` already becomes `a�b`,
so numeric-entity decoding works while literal-source handling does not.
The [targeted probes](raw/probes.json) preserve both forms. Fix this at a
consistent source/AST boundary, with explicit handling of original source spans.

### 3. URL output can change the destination

All seven stricter CommonMark differences concern URL spelling: examples
**20, 202, 346, 502, 526, 538, 603**. The inherited normalizer accepts all seven
because it percent-decodes their attributes.

At least the backslash-in-path cases are more than cosmetic. Example 502,
`[link](foo\bar)`, emits `href="foo\bar"` instead of `href="foo%5Cbar"`.
Resolving against `https://host.example/base/` with Node's URL implementation
produced respectively `/base/foo/bar` and `/base/foo%5Cbar`.
[Recorded resolution output](raw/url-resolution.jsonl) and the
[URL Standard](https://url.spec.whatwg.org/#relative-slash-state) support keeping
these distinct. Other cases involve query or hostname punctuation and still
need context-specific review; the count of seven is not seven independently
proven navigation bugs.

Acceptance for a fix: preserve the intended destination through HTML and URL
parsing, and stop the test normalizer from equating reserved escaped delimiters
with literal delimiters. Existing `%23` input itself is preserved correctly;
the normalizer counterexample is a test weakness, not evidence that the parser
currently corrupts every encoded URL.

### 4. The current GFM extension corpus has five failures

The [official GFM page](https://github.github.com/gfm/) still labels itself
**0.29-gfm (2019-04-06)**. Our frozen response from 2026-09-14 contains **677
examples, including 28 extension examples**. The old subset has 24: 22 match
byte-for-byte, two have changed, and four examples are additional. The initial
assumption that the old subset exactly matched the live page was rejected by
the extraction cross-check. [The mapping](raw/extension-mapping.json) records it.

| Official example | Missing or incorrect behavior |
| --- | --- |
| [491](https://github.github.com/gfm/#example-491) | Single `~word~` stays literal instead of becoming strikethrough |
| [493](https://github.github.com/gfm/#example-493) | Triple tildes incorrectly produce partial strikethrough |
| [633](https://github.github.com/gfm/#example-633) | Extended `mailto:` / `xmpp:` forms remain plain text |
| [634](https://github.github.com/gfm/#example-634) | XMPP resource forms are not linked |
| [635](https://github.github.com/gfm/#example-635) | XMPP resource-boundary behavior is missing |

Tables, task items, the other checked autolinks, and the configured tagfilter
match the remaining **23/28** extension examples after serialization comparison.
This is a finite example result, not proof over all inputs.

The **22** differences in the full configured GFM sweep decompose into:
**7 URL cases + 5 extension gaps + 5 tagfilter/core-example interactions +
3 extended-autolink/core-example interactions + 2 older HTML-comment rules**.
The raw-HTML core examples show unfiltered script/style tags, whereas a global
GFM tagfilter escapes them. Three core autolink examples show plain text that
the extension links. Examples 649–650 use older comment rules; V2 follows the
newer CommonMark behavior there. These interactions must not be converted into
22 undifferentiated parser defects or “fixed” by weakening CommonMark behavior.

### 5. Public presets mix syntax and product rendering policies

| Area | Current behavior | Compatibility implication |
| --- | --- | --- |
| `ParserOptions::gfm()` | Enables footnotes as well as the formal extensions | This is GFM plus an additional syntax extension |
| Renderer tagfilter | `disallow_raw_html` defaults off | Selecting only the parser's GFM preset does not produce the filtered GFM profile |
| Bare URLs | Renderer autolinking defaults on | Default CommonMark parsing still gains extra links during rendering |
| External links / headings | New-tab attributes default on; heading IDs unconditional | HTML differs from spec examples; IDs are not switchable today |
| Callouts / `[[toc]]` | Interpreted unconditionally | Literal CommonMark/GFM content can be replaced even in configured audit profiles |
| Fence info | VitePress-like metadata is stripped even with annotations off | Output policy differs from the usual first-info-word serialization |
| Sanitization | Opt-in; distinct from tagfilter | A syntax profile is not GitHub's full post-processing policy |

The concrete defaults are captured in [probes.json](raw/probes.json). A useful
next design step is explicit CommonMark, formal GFM, and product presets with
renderer switches for TOC, callouts, and heading IDs. Keep intentional existing
behavior available and document any default changes as semantic decisions.

### 6. MDX is a bounded syntax and static-output feature

V2 recognizes component JSX/fragments, attributes, balanced expression source,
and top-level import/export source into dedicated AST nodes. It does not parse
JavaScript into an expression AST, validate arbitrary JS, resolve imports,
compile components, or provide the removed framework runtime.

The static renderer emits component island placeholders; expressions and ESM
are silent. For example `Hello {name}.` renders as `<p>Hello .</p>`. Lowercase
tags remain raw HTML. The ESM scanner itself documents incomplete regex,
automatic-semicolon-insertion, and template-expression handling. This is useful
MDX syntax support, but it is not equivalent to the
[MDX compiler and runtime model](https://mdxjs.com/docs/what-is-mdx/).
Three reproducible boundary probes are included; this audit is not an exhaustive
MDX grammar comparison.

## Test-harness weaknesses

The inherited normalizer only protects `<pre>` whitespace. A verified
counterexample compares `<p><code>  </code></p>` equal to an empty code span.
It also equates `href="/a%23b"` with `href="/a#b"`. Both deliberately wrong
expected outputs are rejected by the conservative helper and accepted by the
inherited one. The actual ordinary inline-code and encoded-URL control probes
pass; a weak test is not itself proof of a production bug in those controls.

The known-failure ratchet stores only mode/example IDs. A listed failure can
worsen or become a panic while remaining on the accepted list. Keep expected
GFM profile exceptions separate from true failures, and require a stable
failure classification/output when documenting actual open defects.

The conservative comparator also has limits: it is an HTML tokenizer, not a
browser parser. Duplicate attributes currently collapse through a dictionary,
and foreign-content/CSS whitespace is not modeled fully. Neither comparator
should certify arbitrary raw HTML. Nine audit-helper tests cover extraction,
code/URL/raw-text preservation, and the distinction between heading-ID-only
and strict equality. Production normalizers and old baselines remain untouched.

## Inheritance, verification, and reproduction

**22 selected cases are byte-identical across the current V2 worker, the frozen
previous V2 worker, and original OX-Content v3.2.3.** These are the twelve
line-ending cases, six literal-NUL cases, their numeric-entity control, the
backslash-path case, and two strikethrough probes. This establishes inheritance
for those reproduced behaviors, not the absence of every possible SIMD
regression. [Comparison data](raw/upstream-comparison.json) records all outputs.
The original OX revision is `a71a58939ffe7f154117cea026f6d6e71a139393`; the
external worker provenance is in the [native benchmark report](../2026-09-14-native-engines/README.md).

Follow the [audit commands](../../../benchmarks/compatibility-audit/README.md).
The complete report is generated offline from the frozen fixtures. Its
`--fail-on-differences` mode currently exits **1**, exposing the open issues;
the existing workspace remains green: **638 Rust tests**, formatting, Clippy,
and benchmark compilation pass. The nine new helper tests also pass. No
performance measurements were rerun because no production implementation changed.

Raw logs, inputs, expected/actual HTML, and metadata are committed here. The
GFM HTML snapshot and extracted examples are in the harness's `fixtures/`
directory with CC BY-SA attribution. Investigation stopped at reproducible
findings and source-level leads because this task requested an audit; the
production fixes are the explicit follow-up work above.
