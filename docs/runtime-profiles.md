# Runtime profiles

Feature cost depends on the document and the integration lifecycle. An enabled
option can add a cheap local branch, a scan even when its syntax is absent, or
substantial work only when a matching construct occurs. A useful profile therefore
defines both accepted syntax and generated HTML before selecting options.

The first study uses the core at `c57ef45`, one runtime-configured native binary,
37 individual toggles, three synthetic size targets, and 13 mixed documents from
37 B to 113,609 B. Parser, renderer, fresh complete processing, and retained
complete processing are separate measurements. The
[source cost map](runtime-feature-costs.md) explains every public option and its
dependencies; the [harness](../benchmarks/runtime-profiles/README.md) records the
method and exact configurations.

## Measured results

The tables below preserve the original `c57ef45` measurements. Definition-list
and line-comment implementation costs have since been reduced; the
[follow-up investigation](reports/2026-09-14-feature-scan-optimization/README.md)
compares unchanged feature settings before and after those optimizations and
remeasures their remaining off/on overhead. No syntax or defaults changed.
The subsequent [line-comment dispatch study](reports/2026-09-14-line-comment-dispatch/README.md)
reduces that option's 4,125-byte plain-prose overhead further: about +1.0% in
parsing and +0.5% in complete reused processing. Those are unused-syntax costs
on one input shape, not the cost of removing a comment.

The [full report](reports/2026-09-14-runtime-profiles/README.md) includes all
37 [feature rows](reports/2026-09-14-runtime-profiles/FEATURES.md),
[mixed-document profile results](reports/2026-09-14-runtime-profiles/PROFILES.md),
and [longer confirmation runs](reports/2026-09-14-runtime-profiles/CONFIRMATION.md).
The main sweep used six paired windows per case/stage; selected findings were
remeasured with nine longer pairs across three process rounds.

On the 4,125-byte plain-prose probe, these options added work despite their syntax
being absent. HTML and AST remained identical:

| Enabled option | Isolated stage time change | Complete processing with reuse |
| --- | ---: | ---: |
| Definition lists | Parser +217% | +151% |
| Parser GFM autolinks | Parser +43% | +31% |
| Line comments | Parser +23% | +14% |
| Tables | Parser +5% | +3% |
| Renderer URL autolinks | Renderer +55% | +16% |

Definition lists, parser autolinks, and tables use the longer confirmation run;
line comments and renderer autolinks use the main sweep. These are costs on this
specific paragraph shape, not fixed taxes on all documents. Most other options
were close to the controls when their triggers were absent. In particular,
math, superscript, and MDX use the fused inline marker classifier without adding
separate full-text scans for each enabled extension.

Trigger-heavy probes tell a different story. At roughly 4 KiB, heading IDs added
54% to complete retained processing of repeated short headings; one TOC plus
many headings added 88% relative to IDs already enabled. Column names added 48%
on repeated small tables with colgroups already enabled. Code annotations added
321% on repeated short annotated fences. Each creates additional HTML or performs
additional semantic work. Those percentages should not be applied to an article
with two headings, a single table, or one unannotated code block.

Turning features on can also shorten parsing by recognizing a whole construct
instead of handling it as ordinary Markdown. The negative active-probe ratios
for task lists, wiki links, or math do not establish a same-output optimization.
Repeated synthetic footnote labels also affect output deduplication. Original
input/output bytes and ASTs are archived to make these differences inspectable.

The four use-case ablations keep HTML **and AST exactly equal**, including
frontmatter and source spans. Only unused parser extensions are removed:

| Tailored recipe | Input bytes | Fresh processing time saved | Reused processing time saved |
| --- | ---: | ---: | ---: |
| Comments | 345 | 15% | 18% |
| Article | 4,308 | 40% | 41% |
| Docs | 65,758 | 10% | 9% |
| MDX docs | 4,163 | 13% | 14% |

These confirmed results establish that selecting options can help. Each row is
one authored use-case probe, not an average gain for every document of that kind.
The mixed-document matrix separately shows what additional syntax/output costs
on the existing comments, project documentation, and Wikipedia corpus.

There is also a useful cross-stage interaction: after parser GFM autolinks have
created link nodes, enabling renderer URL detection added 13% to retained
processing of the 4 KiB plain probe, and 4% to the URL-rich probe, with identical
HTML and AST. All six sizes/shapes passed equality checks. The candidate recipes
therefore leave renderer `autolink_urls` off. Applications with custom renderer
URL patterns or different link policies must evaluate those requirements.

The active probes' time per byte was broadly stable between 4 KiB and 64 KiB.
That supports a constant-factor explanation on these shapes; it does not prove
worst-case linear behavior for malformed delimiters or deeply nested documents.
Unclosed frontmatter is explicitly checked: it adds a full unsuccessful scan
before normal Markdown parsing, about 26% complete retained time at 4 KiB.

## Candidate use-case recipes

These recipes are measured configurations, not new public constructors or changed
defaults. They start from the explicit CommonMark parser and renderer profiles.
The current `ParserOptions::gfm()` convenience preset includes footnotes;
`gfm_spec()` excludes them. Renderer `Default` separately enables heading IDs,
callouts, inline TOC, fence metadata, and renderer URL autolinking.

| Recipe | Parser additions | Renderer additions | Intended use |
| --- | --- | --- | --- |
| CommonMark | None | None | Basic Markdown with explicitly authored links; a narrow formatting contract |
| GFM spec | Tables, tasks, strikethrough, GFM autolinks | GFM tagfilter | Specification-oriented GFM serialization |
| Comments | Same GFM syntax | Escape raw HTML and validate Markdown link/image URLs | GitHub-style application comments without document metadata or navigation |
| Article | Footnotes, frontmatter, heading attributes | Heading IDs, semantic footnotes | Prose-led publication with metadata and citations |
| Docs | GFM syntax, footnotes, frontmatter, line comments, heading attributes | Tagfilter, heading IDs, inline TOC, callouts, fence metadata, semantic footnotes | Documentation with navigation and code examples |
| MDX docs | Docs plus MDX | Same as Docs; MDX nodes render automatically | Authored Markdown with component/expression syntax |

The article recipe deliberately assumes that tables and bare-URL recognition are
not part of the authoring contract. Add them when they are needed. A plain-comment
recipe could similarly use CommonMark syntax plus the same HTML policy; removing
GFM features from an application promising GFM would change its behavior.

Definition lists, math, superscript/subscript, wiki links, annotated code,
permalinks, source-span attributes, and table layout classes remain explicit
additions. Table column names require colgroups; TOCs require heading IDs;
annotations require fence metadata and the appropriate annotation syntax.
Enabling wiki links consumes `[[toc]]` as a link before the renderer can use it
as a TOC marker. Profiles must resolve such syntax conflicts deliberately.

An article configuration can be written with the existing public API:

```rust
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};

let parser_options = ParserOptions {
    footnotes: true,
    front_matter: true,
    heading_attributes: true,
    ..ParserOptions::commonmark()
};
let renderer_options = HtmlRendererOptions {
    heading_ids: true,
    semantic_footnotes: true,
    ..HtmlRendererOptions::commonmark()
};
let source = "---\ntitle: Article\n---\n# Article\n\nA claim[^n].\n\n[^n]: Evidence.\n";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::with_options(&allocator, source, parser_options).parse().unwrap();
let html = HtmlRenderer::with_options(renderer_options).render(&document);
assert!(document.front_matter.is_some());
assert!(html.contains("id=\"article\""));
assert!(html.contains("class=\"footnotes\""));
```

## What profile tuning can and cannot remove

Switching off an unused feature preserves output only when that input does not
depend on it. The study checks identical HTML **and AST** for its four controlled
profile ablations. Other active-feature and mixed-profile comparisons often
produce different output; they describe a feature budget rather than equivalent
implementations getting faster.

Retained processing resets the arena and renders into a retained borrowed output
buffer. Fresh processing includes owned option construction/cloning, allocation,
and teardown. This matters on short inputs. At the time of this study
`HtmlRenderer::new()` additionally had a static default-options path; the
configurable fresh benchmark intentionally used owned options on both sides, so
it does not measure that constructor shortcut. Renderer option strings have
since become `Cow<'static, str>`, which removes the asymmetry rather than
changing any number recorded here — see the
[decision record](decisions/2026-09-15-borrowed-renderer-options.md).

Profiles do not remove baseline CommonMark reference discovery or source
normalization. At the time of measurement they did not remove the renderer's
structural AST scan either: it still ran when both heading IDs and inline TOC
were off, which this study named as a separate implementation optimization
opportunity. A one-shot render now runs that scan only while `heading_ids` is
enabled, so the strict profiles measured here no longer pay it. Nesting limits
stay enabled. HTML policy and syntax support remain product decisions, not
settings to remove solely for a score.

At the time of measurement two inherited public renderer fields, `highlight` and
`soft_break`, had no rendering effect; they were measured as controls rather than
as functional profile dimensions. The 2.0.0 API freeze then removed
`HtmlRendererOptions::highlight` and wired `soft_break` through, so the
configured value is now emitted for every line ending in rendered inline text.
See the [API surface decision](decisions/2026-09-17-api-surface.md).

## Next implementation targets

The results suggest three separate follow-ups:

1. Definition-list and line-comment rejection has now been optimized in the
   [feature-scan follow-up](reports/2026-09-14-feature-scan-optimization/README.md).
   Actual definition bodies and comment-bearing paragraphs still perform
   necessary parsing, text joining, and source mapping; a late definition marker
   can retain speculative term probes on earlier paragraphs.
2. Avoid renderer preparation work when the selected output profile needs neither
   heading IDs nor TOC discovery. This has since been implemented: a one-shot
   render derives the scan's facts only while `heading_ids` is on, and otherwise
   skips the walk. The tables above predate that change and were not remeasured.
3. Once each use-case contract is settled, expose a profile that pairs parser
   and renderer settings. A named profile should preserve its documented syntax
   and output policy while implementation optimizations evolve underneath it.

No library defaults, parsing semantics, or public profile API changed in this
study. The current configurations can already be used through existing options.
