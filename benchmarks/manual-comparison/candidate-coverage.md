# Benchmark candidate coverage

Reviewed on **2026-09-30**. The original review found seven direct gaps in the
14-row executable matrix. The follow-up implemented those projects as eight new
lanes, then added TanStack Markdown and made three variants optional. The default
campaign now contains **nine Native and eleven Node.js execution variants**.
The inventory retains 23 executable rows; the homepage and default campaign show 20. MD4X has distinct explicit NAPI and WASM entries; OX-Content
has separate native and Node binding entries. Their pinned public API contracts
and local checks are in the [ecosystem runbook](../markdown-ecosystem/README.md#expanded-public-apis).
No new performance values are published by adding adapters.

Campaign pin update on **2026-10-01**: TanStack Markdown is now 1.0.0. The
2026-09-30 release/download observations below remain historical; the public
HTML workload and candidate selection are unchanged. See
[dependency readiness](dependency-readiness.md) for the release and checks.

The additional subset, language/runtime, and streaming candidates below remain
an explicit backlog. This matrix is not a claim to cover every Markdown engine.

The [metadata snapshot](candidate-review-2026-09-30.json) retains 57 GitHub
repository records and 25 npm package records, including sources and the npm
window **2026-08-30 through 2026-09-28**. Discovery used registry metadata, the
[CommonMark implementation directory](https://github.com/commonmark/commonmark-spec/wiki/list-of-commonmark-implementations),
framework dependencies, and candidate projects' own comparisons. No upstream
speed claim or benchmark result is imported into Ferromark's evidence.

## Selection criteria

A candidate warrants review if it has significant package usage, a documented
integration in an established product, or a distinct technical approach relevant
to parsing and rendering. Examples include pull parsing, AST/CST processing,
native bindings, WASM boundaries, small bundles, and incremental processing.
Novel approaches can qualify with little adoption; popularity is not a gate that
excludes competitors with promising designs.

Downloads include CI, transitive installations, and repeat requests; they are
not user counts. Stars indicate visibility, not quality or usage. A runtime's
stars, such as Bun's, do not measure adoption of its Markdown component. Avoid
comparing these numbers across registries as if they used the same definition.
Maintenance, usable releases, reproducible toolchains, output behavior, and
supported syntax are documented separately from popularity. HTML agreement or
conformance is not an admission criterion or a performance-scoring filter.
Every selected implementation processes the same 57 inputs through its public
HTML API; document feature differences rather than excluding inputs.

Keep **Native** and **Node.js** as the existing top-level groups. Within Node.js,
record JavaScript, native addon, and WASM execution explicitly. A native core
measurement does not measure its Node binding. A compiled Go adapter belongs in
Native, with its Go runtime and GC disclosed. JVM, .NET, Python, PHP, and Ruby
need their own runtime contracts if added; do not silently label them Native or
put their warmup behavior under Node's contract.

## Original executable coverage

These are measured project names, not 14 independent parser families. In
particular, remark includes micromark through its AST pipeline, while the
micromark row measures direct HTML output. Both operations are useful comparisons.

| Group | Projects already in the harness | Coverage |
| --- | --- | --- |
| Native | [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark), [md4c](https://github.com/mity/md4c), [Bun MD](https://github.com/oven-sh/bun), [OX-Content](https://github.com/ubugeeei-prod/ox-content) | Pull parser, callback renderer, optimized runtime component, arena/AST pipeline |
| Native | [markdown-rs](https://github.com/wooorm/markdown-rs), [Comrak](https://github.com/kivikakk/comrak), [cmark](https://github.com/commonmark/cmark), [cmark-gfm](https://github.com/github/cmark-gfm) | Rust AST implementations and C CommonMark/GFM reference implementations |
| Node.js | [marked](https://github.com/markedjs/marked), [markdown-it](https://github.com/markdown-it/markdown-it), [remark / unified](https://github.com/remarkjs/remark), [micromark](https://github.com/micromark/micromark), [Showdown](https://github.com/showdownjs/showdown), [commonmark.js](https://github.com/commonmark/commonmark.js) | Major JavaScript converters, tokens, AST/plugin pipelines, and reference parser |

All six existing Node packages have substantial usage in the retained snapshot.
Keep them; an old stable release alone is not a reason to drop Showdown, or to
exclude a missing mature implementation such as Remarkable. Ferromark v1 stays
an internal historical control rather than a public competitor row.
[Dependency readiness](dependency-readiness.md) records exact current pins and
[local readiness](../../docs/reports/2026-09-30-comparison-readiness/README.md)
records execution checks. Those checks apply to these 14 competitors only.

## Implemented direct additions

These Native/Node.js gaps now have executable adapters and exact package pins.
Versions were rechecked against their release registries; npm counts remain the
dated review window above. Read the public API contracts before running them.

| Project | Observed release | Reason to include | Execution |
| --- | --- | --- | --- |
| [Remarkable](https://github.com/jonschlinkert/remarkable) | npm 2.0.1 | 5,264,333 npm downloads; direct HTML converter with configurable rules and a CommonMark preset. It is different from remark. | Node.js JavaScript |
| [Goldmark](https://github.com/yuin/goldmark) | v2.1.6 | [Hugo's default parser](https://gohugo.io/configuration/markup/); independent Go AST/CST implementation and extensions. | Native Go, persistent worker |
| [markdown-exit](https://github.com/serkodev/markdown-exit) | npm 1.3.0 | 740,273 downloads; maintained TypeScript rewrite of markdown-it with changes to the parser/rendering implementation. | Node.js JavaScript |
| [markdown-it-ts](https://github.com/Simon-He95/markdown-it-ts) | npm 1.1.2 | 224,342 downloads; another distinct rewrite, with modular parsing/rendering and experimental incremental APIs. | Node.js JavaScript; ordinary full-document API first |
| [Sätteri](https://github.com/bruits/satteri) | npm satteri 0.10.5 | 13,649,320 downloads; Rust processing and a JavaScript plugin interface. Its pulldown-derived parser does not make its Node pipeline equivalent to the existing native pulldown row. | Node.js native addon; HTML pipeline without MDX/plugins |
| [MD4X](https://github.com/unjs/md4x) | npm md4x 0.0.30 | 16,147 downloads; MD4C-derived Zig implementation with explicit NAPI and WASM HTML APIs. Relevant native-to-Node and WASM alternatives despite lower adoption. | Two explicit Node.js variants: NAPI and WASM |
| [OX-Content Node binding](https://github.com/ubugeeei-prod/ox-content) | npm @ox-content/napi 3.2.13 | 61,844 downloads; direct competing Node API. Existing native OX-Content evidence does not cover JS string conversion or the binding's defaults. | Node.js native addon; measure public parseAndRender API |

Do not collapse markdown-exit and markdown-it-ts into markdown-it, or discard
Node/WASM implementations simply because a related native core is already
listed. Their public execution paths and costs differ. Conversely, label shared
parser lineage so that variants are not presented as independent algorithms.

## Default campaign and optional adapters

The main campaign keeps the original 14 rows plus Goldmark, Remarkable, Sätteri,
MD4X NAPI, OX-Content NAPI, and TanStack Markdown: **20 variants**. The TanStack
name and ecosystem make its public HTML API relevant, even though upstream
explicitly supports a syntax subset. Its fixed parser features, code markup,
and actual equivalent-output coverage must remain visible; no CommonMark/GFM
conformance or speed result is inferred.

markdown-exit, markdown-it-ts, and MD4X WASM are **optional**. The two TypeScript
rewrites are secondary to markdown-it for this full-document workload; their
async/incremental capabilities are not represented by its throughput factor.
The WASM path is useful for portable/browser workloads but is secondary to the
NAPI path in the main Node comparison. Keep their exact pins, adapters, tests,
and explicit `--scope extended` support. Binding rows for OX-Content and MD4X
NAPI stay in main: they measure the API Node consumers can choose against our
own binding, not an inferred native-core factor.

### OX-Content cross-check, 2026-09-30

I checked the official [performance page](https://ubugeeei-prod.github.io/ox-content/performance/index.html),
its [source](https://github.com/ubugeeei-prod/ox-content/blob/main/docs/content/performance.md),
and [benchmark runner](https://github.com/ubugeeei-prod/ox-content/blob/main/tools/benchmarks/bundle-size/parse-benchmark.mjs).
The additional names are accounted for as follows; no upstream measurements are
imported into our reports or homepage.

| Candidate in OX-Content | Main-campaign decision |
| --- | --- |
| TanStack Markdown | Included as @tanstack/markdown 0.0.16, direct HTML API with its subset contract. |
| @mizchi/markdown, JS/WASM/native | Existing innovation backlog. MoonBit/CST and target availability need separate adapter/build validation; three variants are not required for the initial shorter campaign. |
| md4w | Existing secondary WASM backlog. Its MD4C lineage is already represented; a Node/WASM bridge still needs an explicit runtime contract if added. |
| markdown-it-ts and MD4X WASM | Retained as optional, not overlooked or deleted. |
| xai-grok-markdown-core (Grok Build) | The [native adapter](https://github.com/ubugeeei-prod/ox-content/blob/main/tools/benchmarks/native-competitors/src/main.rs) measures `offset_events` only in parse-only rows. The [dependency](https://github.com/ubugeeei-prod/ox-content/blob/main/tools/benchmarks/native-competitors/Cargo.toml) is a pinned pulldown-cmark wrapper, not a separate full HTML-rendering implementation. Defer a facade/event-stream workload; pulldown-cmark remains in our HTML campaign. |

OX-Content distinguishes parse-only from parse+render workloads. Our campaign
measures full-document HTML API performance. Their host, defaults, corpus, and
denominators differ; their figures cannot establish our ranking. We retain output
differences as descriptive metadata rather than importing a conformance column.

## Additional adoption and innovation candidates

These should remain visible in the selection review. Some fit full-document HTML
throughput, while others need a narrower syntax profile or another workload.
Decisions about this campaign must identify which are included and why the
others are deferred; this document does not certify their adapters as ready.

| Project | Evidence or distinctive approach | Measurement decision |
| --- | --- | --- |
| [TanStack Markdown](https://github.com/TanStack/markdown) | npm @tanstack/markdown 0.0.16; 258,243 downloads; small parser and serializable AST with direct HTML API. Upstream explicitly targets controlled blog/docs syntax rather than complete CommonMark/GFM. | Implemented in the main Node campaign; disclose its syntax subset and actual agreement coverage. Its AI profile reparses accumulated text, rather than being a stateful incremental parser. |
| [@mizchi/markdown](https://github.com/mizchi/markdown.mbt) | npm 0.8.3; 780 downloads; MoonBit CST/incremental design with JS, WASM-GC, and native targets. WASM string interop is technically distinct from linear-memory UTF-8 bridges. | Innovation candidate despite small adoption. Validate published backend availability and Node 24 support; separate full-document HTML and edit workloads. |
| [Snarkdown](https://github.com/developit/snarkdown) | npm 2.0.0; 596,132 downloads; deliberately tiny regex-based parser. Upstream explicitly omits tables. | Relevant small-bundle/subset candidate. Disclose unsupported features; include every input in performance scoring. |
| [markdown-wasm](https://github.com/rsms/markdown-wasm) / [md4w](https://github.com/ije/md4w) | npm 1.2.0 / 0.2.7; 26,502 / 52,795 downloads; established MD4C-based WASM bridges. | Secondary Node/WASM candidates alongside MD4X. Verify their bundled MD4C revisions; a latest wrapper release can still contain an older core. |
| [Blackfriday](https://github.com/russross/blackfriday) / [gomarkdown](https://github.com/gomarkdown/markdown) | Established Go parser family: 5,602 / 1,737 GitHub stars. | Relevant next Native candidates, with exact extension profiles. Goldmark does not automatically cover their behavior or performance. |
| [Lute](https://github.com/88250/lute) | 1,676 stars; structured Go/JavaScript engine; upstream lists SiYuan and Vditor integrations and CJK/editor features. | Native candidate after Goldmark; disable highlighting and additional formatting in the HTML comparison. Editor-specific features need their own workload. |
| [Pandoc](https://github.com/jgm/pandoc) | 46,458 stars; major Haskell document converter with CommonMark/GFM readers and HTML output. | Important expanded Native candidate. Compare a pinned reader/writer in a persistent process; separately report CLI startup if relevant. Do not compare a fresh pandoc process against an already loaded library. |
| [Discount](https://github.com/Orc/discount) | Maintained C implementation of the original Markdown dialect with extensions. | Consider for an expanded legacy Native profile; disclose dialect differences against CommonMark. |
| [Sundown](https://github.com/vmg/sundown) / [Hoedown](https://github.com/hoedown/hoedown) | Established older C parser family; last pushed in 2018 / 2020 in this snapshot. | Legacy backlog rather than another current CommonMark reference. Reassess for concrete downstream use; do not infer current GitHub rendering from Sundown's historical README. |
| [MultiMarkdown](https://github.com/fletcher/MultiMarkdown-7) | Additional authoring syntax and multiple output formats; v6 upstream is deprecated, while v7 is currently a beta. | Relevant extended-document profile. Do not silently replace a stable release with v7's prerelease in the ordinary campaign. |
| [markdown-js](https://github.com/evilstreak/markdown-js) | npm markdown 0.5.0; 477,465 downloads, last published in 2013. | Explicit legacy candidate. Deferred from the initial extension, not assumed unused; revisit if supporting its old dialect is a product requirement. |

## Prominent ecosystems outside the current runtime scope

These are relevant projects, not absent because they were overlooked. Broad
claims about all Markdown implementations would require extending the runtime
scope and measuring them. Star counts below are visibility indicators from the
snapshot; documented downstream integrations provide stronger adoption evidence.

| Runtime | Candidates and adoption evidence | Proposed scope |
| --- | --- | --- |
| .NET | [Markdig](https://github.com/xoofx/markdig), 5,336 stars; CommonMark-oriented AST and extensions | Own .NET track with recorded runtime, tiered JIT/warmup or explicit AOT build contract |
| JVM | [commonmark-java](https://github.com/commonmark/commonmark-java), 2,695 stars, upstream lists OpenJDK/Gerrit/Atlassian; [flexmark-java](https://github.com/vsch/flexmark-java), 2,641 stars, richer dialect/extension support; [JetBrains Markdown](https://github.com/JetBrains/markdown), Kotlin/editor integration | Own JVM track; stabilize JIT and disclose heap/GC, then align full-document HTML options |
| Python | [Mistune](https://github.com/lepture/mistune), used by [Jupyter nbconvert](https://github.com/jupyter/nbconvert/blob/main/nbconvert/filters/markdown_mistune.py); [Python-Markdown](https://github.com/Python-Markdown/markdown), used by [MkDocs](https://www.mkdocs.org/user-guide/writing-your-docs/); [markdown-it-py](https://github.com/executablebooks/markdown-it-py), used by [Rich](https://github.com/Textualize/rich/blob/main/rich/markdown.py); [markdown2](https://github.com/trentm/python-markdown2) | Own Python track; distinguish a real port from a native core binding |
| PHP | [league/commonmark](https://github.com/thephpleague/commonmark), used by [Laravel Str::markdown](https://laravel.com/framework/docs/12.x/strings#method-str-markdown); [Parsedown](https://github.com/erusev/parsedown), 15,057 stars | Own PHP track, including opcache/JIT configuration and persistent process contract |
| Ruby | [kramdown](https://github.com/gettalong/kramdown), [Jekyll's default](https://jekyllrb.com/docs/configuration/markdown/); [Redcarpet](https://github.com/vmg/redcarpet), Ruby API with a C core | Own Ruby track; disclose native binding overhead for Redcarpet. Check RubyGems rather than kramdown's stale GitHub releases/latest entry. |

These runtime extensions are a backlog, not requirements secretly added to the
existing manual CLI. Keep the homepage compact: project links, measured version,
runtime/backend, and observations belong in tables. Candidate decisions and
workload details belong in the guide/repository; unmeasured candidates must not
acquire inferred factors or copied measurements.

## Different workloads and related implementations

| Project | Why it matters | Coverage decision |
| --- | --- | --- |
| [react-markdown](https://github.com/remarkjs/react-markdown) | 136,497,275 npm downloads; established React renderer on the unified ecosystem | Parser lineage is represented, but React rendering is not measured. Needs React/SSR output, not an HTML-string parser substitute. |
| [Streamdown](https://github.com/vercel/streamdown) | 24,853,884 downloads; streaming React rendering with remark/rehype plugins and incomplete-Markdown handling | Own chunked-rendering workload. Include repair, React work, and updates if comparing application behavior. |
| [streaming-markdown](https://github.com/thetarnav/streaming-markdown) | 68,338 downloads; independent chunk parser with rendering callbacks | Own streaming workload with fixed chunks and equivalent output sink; also test final completed-document correctness. |
| [Brookmd](https://github.com/siinghd/brookmd) | Rust/WASM streaming design; only 425 npm downloads | Innovation watchlist, not established adoption. Verify the published API and chunk-processing behavior before a streaming comparison. |
| [Lezer Markdown](https://lezer.codemirror.net/docs/ref/#markdown) | 19,687,864 npm downloads; incremental syntax trees for editors | Own edit/reparse workload; it does not supply the current HTML rendering contract. [The old GitHub repository](https://github.com/lezer-parser/markdown) moved to code.haverbeke.berlin; archived does not mean the package was abandoned. |
| [MDX](https://github.com/mdx-js/mdx) | 44,115,526 npm downloads; Markdown/JSX compilation | Own compiler workload with equivalent JavaScript output and syntax; not another Markdown-to-HTML speed row. |
| [Swift Markdown](https://github.com/swiftlang/swift-markdown) | Apple ecosystem AST/editing facade backed by cmark-gfm | Native core already represented; Swift-facing costs remain unmeasured. Add a facade/AST workload only if claiming Swift API performance. |
| [mdast-util-from-markdown](https://github.com/syntax-tree/mdast-util-from-markdown) | micromark-based Markdown-to-mdast conversion | Existing remark pipeline includes this family. Separate parsing-only/AST costs if useful; do not count it as an independent HTML engine. |
| [Mmark](https://github.com/mmarkdown/mmark) | IETF-oriented document processing | Separate extended-document/XML profile. Use the current repository, not the obsolete miekg/mmark pointer. |
| [Djot](https://github.com/jgm/djot) | Distinct lightweight markup language | Out of Markdown syntax scope; useful only for an explicitly different language comparison. |

For streaming, distinguish accumulated-text reparsing from stateful chunk
processing. Measure total CPU over the same chunk sequence, update latency,
retained memory, and final output equality. For editors, use the same edit trace
and validate the final tree/rendered document. Do not reuse a full-document
throughput factor to make streaming or incremental claims.

## Campaign acceptance

1. Resolve this candidate list into an explicit included/deferred matrix. The
   seven direct-addition projects above are implemented; verify them before describing
   the selection as representative of current Native/Node alternatives. Review
   the subset and innovation candidates explicitly as well.
2. Pin actual released packages/source revisions and supporting toolchains.
   Read the pinned API, not just documentation from main. Record parser lineage,
   backend selection, syntax options, and the complete public call being timed.
   MD4X's explicit NAPI/WASM imports must not silently fall back to each other.
3. Build and verify locally before allocating managed time. Exercise every
   existing frozen corpus input and option guard; retain disagreements without
   rewriting output to disguise differences. Guards check reproducibility, options,
   state isolation, and actual work; they are not a general spec conformance gate.
   All 57 inputs contribute to performance. Keep source fixtures immutable.
4. Inspect caches and repeated-input behavior. In particular, markdown-it-ts
   documents last-output caching in some paths. Add a rotating-document control
   for new Node adapters before claiming parser throughput; report memoized
   repeated-input behavior separately from actual parsing. Incremental APIs must
   never leak into a supposedly independent full-document call.
5. Keep process startup, imports, WASM initialization, and tool downloads outside
   steady-state library timing. Record runtime/compiler versions, GC, allocation,
   and warmup contracts. New runtime tracks need reviewed sampling parameters;
   do not silently inherit Node's short warmup for JVM or .NET.
6. Update the manual CLI, required coverage/publication checks, retained metadata,
   generated tables, and workflow matrix together when adapters are added. A
   candidate entry in this document is not an executed benchmark. Existing
   reports and the current 14-project readiness record remain immutable.

The original review supplied a sourced selection and implementation queue. The
seven direct projects and TanStack now have adapters; their homepage cells remain unmeasured
until a complete campaign is reviewed and imported. There is no claim that every implementation in every language has been
cataloged or measured.
