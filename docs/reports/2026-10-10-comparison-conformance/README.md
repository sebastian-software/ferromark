# Comparison conformance

Finite output agreement with 652 CommonMark 0.31.2 examples and 28 **GFM extension** examples. This is not a full GFM conformance score or a performance eligibility filter.

Measured 2026-10-10T14:37:09.251442+00:00 on macOS-27.0.1-arm64-arm-64bit; Ferromark source `ad50470e88601f1674c97b033c45a346d45c6919`.

[Method, reproduction command, and comparison rules](../../../benchmarks/conformance/README.md). [Build provenance](build.json). [Machine-readable summary](summary.json). Each library link below retains every input, expected/actual HTML, error, and classification.

| Variant | Version | CommonMark | GFM extensions |
| --- | --- | ---: | ---: |
| [Ferromark (Native)](ferromark-native.json) | 3.3.0 | 652/652 (100.00%) | 28/28 (100.00%) |
| [Ferromark (Node.js)](ferromark-node.json) | 3.3.0 | 652/652 (100.00%) | 28/28 (100.00%) |
| [pulldown-cmark (Native)](pulldown-cmark.json) | 0.13.4 | 652/652 (100.00%) | 12/28 (42.86%) |
| [md4c (Native)](md4c.json) | 0.6.0 | 652/652 (100.00%) | 16/28 (57.14%) |
| [Bun MD (Native)](bun.json) | 1.4.2 | 652/652 (100.00%) | 22/28 (78.57%) |
| [OX-Content (Native)](ox-content.json) | 3.2.13 | 605/652 (92.79%) | 23/28 (82.14%) |
| [markdown-rs (Native)](markdown-rs.json) | 1.0.0 | 652/652 (100.00%) | 26/28 (92.86%) |
| [Comrak (Native)](comrak.json) | 0.55.0 | 652/652 (100.00%) | 26/28 (92.86%) |
| [cmark (Native)](cmark.json) | 0.31.2 | 652/652 (100.00%) | Unsupported |
| [cmark-gfm (Native)](cmark-gfm.json) | 0.29.0.gfm.13 | 641/652 (98.31%) | 26/28 (92.86%) |
| [Goldmark (Native)](goldmark.json) | 2.1.6 | 652/652 (100.00%) | 24/28 (85.71%) |
| [marked (Node.js)](marked.json) | 18.0.14 | 640/652 (98.16%) | 27/28 (96.43%) |
| [markdown-it (Node.js)](markdown-it.json) | 15.0.2 | 652/652 (100.00%) | 17/28 (60.71%) |
| [remark / unified (Node.js)](remark.json) | 15.0.1 | 651/652 (99.85%) | 22/28 (78.57%) |
| [micromark (Node.js)](micromark.json) | 4.0.3 | 652/652 (100.00%) | 25/28 (89.29%) |
| [Showdown (Node.js)](showdown.json) | 2.1.0 | 357/652 (54.75%) | 11/28 (39.29%) |
| [commonmark.js (Node.js)](commonmark.json) | 0.31.2 | 652/652 (100.00%) | Unsupported |
| [Remarkable (Node.js)](remarkable.json) | 2.0.1 | 531/652 (81.44%) | Unsupported |
| [Sätteri (Node.js)](satteri.json) | 0.10.5 | 652/652 (100.00%) | 22/28 (78.57%) |
| [MD4X (Node.js)](md4x-napi.json) | 0.0.30 | 649/652 (99.54%) | 17/28 (60.71%) |
| [OX-Content (Node.js)](ox-content-napi.json) | 3.2.13 | 596/652 (91.41%) | 14/28 (50.00%) |
| [TanStack Markdown (Node.js)](tanstack-markdown.json) | 1.0.0 | 378/652 (57.98%) | 13/28 (46.43%) |

## Profiles and limitations

### Ferromark (Native)

CommonMark: `ParserOptions::commonmark + HtmlRendererOptions::commonmark`.

GFM extensions: `ParserOptions::gfm_spec + HtmlRendererOptions::gfm_spec`.

Full spec profiles; GFM adds autolink literals and tagfilter to the timed shared profile.

### Ferromark (Node.js)

CommonMark: `toHtml: trusted, allowHtml, headingIds=false, callouts=false; tables/strike/tasks/autolinkLiterals/disallowedRawHtml=false`.

GFM extensions: `Same; tables/strike/tasks/autolinkLiterals/disallowedRawHtml=true`.

The public Node HTML API retains fence metadata handling; other optional syntax uses addon defaults. GFM adds autolinks and tagfilter.

### pulldown-cmark (Native)

CommonMark: `Parser::new_ext: Options::empty; html::push_html`.

GFM extensions: `ENABLE_TABLES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS`.

No public literal-autolink or tagfilter switch in 0.13.4. ENABLE_GFM only enables blockquote alerts and is intentionally off.

### md4c (Native)

CommonMark: `md_html: parser_flags=0; renderer_flags=0`.

GFM extensions: `parser_flags=0x100|0x200|0x800|0x4|0x8|0x400; renderer_flags=0`.

Adds all permissive autolinks. No public tagfilter option; the broader GITHUB preset also enables footnotes and admonitions.

### Bun MD (Native)

CommonMark: `render_to_html_with_options: all BOOL_FIELD_SETTERS false`.

GFM extensions: `Only tables, strikethrough, tasklists, permissive_autolinks, tag_filter true`.

Direct native Bun MD implementation, not the JavaScript Bun.markdown boundary. Adds autolinks and tagfilter.

### OX-Content (Native)

CommonMark: `ParserOptions::default; HtmlRendererOptions::new with autolink_urls/autolink_target_blank/link_target_blank=false`.

GFM extensions: `Same; parser tables/strikethrough/task_lists/autolinks=true; renderer disallow_raw_html=true`.

Renderer retains built-in IDs, TOC, callouts, and fence handling: the public renderer has no disable switches. Adds parser autolinks and tagfilter.

### markdown-rs (Native)

CommonMark: `Options::default; allow_dangerous_html/allow_dangerous_protocol=true`.

GFM extensions: `Options::gfm; footnote constructs=false; dangerous HTML/protocol=true`.

Full formal GFM features including tagfilter; enables literal autolinks beyond timed profile.

### Comrak (Native)

CommonMark: `Options::default; render.unsafe=true`.

GFM extensions: `Same; extension table/strikethrough/tasklist/autolink/tagfilter=true`.

Uses the released 0.55.0 tagfilter option despite its upstream deprecation. Adds literal autolinks and tagfilter.

### cmark (Native)

CommonMark: `cmark_markdown_to_html: CMARK_OPT_UNSAFE`.

GFM extensions: unsupported.

No GFM extension API.

### cmark-gfm (Native)

CommonMark: `Parser: CMARK_OPT_UNSAFE, no extensions`.

GFM extensions: `Same; attach table, strikethrough, tasklist, autolink, tagfilter`.

Enables full formal extensions. CommonMark implementation targets an older spec; measured against 0.31.2.

### Goldmark (Native)

CommonMark: `parser.New; html.New(html.WithUnsafe())`.

GFM extensions: `Add NewTableParser/NewStrikethroughParser/NewTaskListItemParser/NewLinkifyParser and corresponding HTML renderers`.

No tagfilter option in pinned public v2 API. Adds linkify to the timed profile.

### marked (Node.js)

CommonMark: `new Marked({gfm:false,breaks:false,pedantic:false,async:false})`.

GFM extensions: `Same with gfm=true`.

Uses stock GFM URL tokenizer, unlike timing. Raw HTML passes through; no tagfilter option.

### markdown-it (Node.js)

CommonMark: `commonmark preset; html=true, linkify=false, typographer=false, validateLink accepts all protocols`.

GFM extensions: `Same; linkify=true, enable table/strikethrough/linkify, markdown-it-task-lists defaults; linkify.set({fuzzyLink:true})`.

Task plugin adds CSS classes. Linkify recognizes bare domains beyond formal GFM; fuzzyLink=true enables its public www recognizer, which linkify-it 6 disables by default. No tagfilter option. Adds linkify to timing profile.

### remark / unified (Node.js)

CommonMark: `remark + remark-rehype + rehype-stringify; allowDangerousHtml=true`.

GFM extensions: `Same; micromark table/strike/tasks/autolink extensions and corresponding mdast extensions`.

Raw HAST passes through; no tagfilter option in this pipeline. Adds literal autolink extension to timing profile.

### micromark (Node.js)

CommonMark: `micromark: allowDangerousHtml/allowDangerousProtocol=true`.

GFM extensions: `Same; table/strike/tasks/autolink syntax and HTML extensions plus tagfilter HTML extension`.

Enables the five formal GFM extensions without footnotes. Adds autolinks and tagfilter to timing profile.

### Showdown (Node.js)

CommonMark: `Converter: noHeaderId, ghCodeBlocks, literalMidWordUnderscores; ellipsis/tables/strikethrough/tasklists/simplifiedAutoLink/simpleLineBreaks/parseImgDimensions/encodeEmails/openLinksInNewWindow/metadata/completeHTMLDocument=false; headerLevelStart=1`.

GFM extensions: `Same; tables/strikethrough/tasklists/simplifiedAutoLink=true`.

Closest public rendering options; no CommonMark preset or GFM tagfilter. Literal underscores enabled beyond timing profile.

### commonmark.js (Node.js)

CommonMark: `Parser({smart:false}); HtmlRenderer({safe:false,softbreak:"\n"})`.

GFM extensions: unsupported.

CommonMark-only public API; same options as timing.

### Remarkable (Node.js)

CommonMark: `Remarkable(commonmark,{html:true,breaks:false,typographer:false})`.

GFM extensions: unsupported.

Supports partial extensions but no task-list API/plugin in this pinned workspace; full GFM suite unsupported.

### Sätteri (Node.js)

CommonMark: `markdownToHtml: all features=false`.

GFM extensions: `features.gfm={footnotes:false}; all other features=false`.

rawHtml=false preserves opaque raw nodes verbatim; true reparses HTML. Uses the public native addon, not a substituted Rust crate. Same as timing profile.

### MD4X (Node.js)

CommonMark: `md4x/napi init; renderToHtml({headingIds:false,full:false,heal:false})`.

GFM extensions: `Same fixed parser configuration`.

Public native addon cannot disable its parser extensions or select a formal spec profile. Both suites measured, not relabeled as CommonMark-only.

### OX-Content (Node.js)

CommonMark: `parseAndRender: gfm/mdx/footnotes/tables/strikethrough/taskLists/autolinks/superscript/subscript/smartPunctuation/math/definitionLists/headingAttributes/wikiLinks=false`.

GFM extensions: `Same; tables/strikethrough/taskLists/autolinks=true`.

Native binding retains renderer builtins and lacks renderer tagfilter options. Adds parser autolinks to timing profile.

### TanStack Markdown (Node.js)

CommonMark: `renderHtml: allowHtml=true, headingIds/headingAnchors/frontmatter/codeLineNumbers=false, extensions=[], urlTransform identity`.

GFM extensions: `Same fixed syntax subset`.

Cannot disable built-in tables/strike/tasks/footnotes. Both suites measured. Same options as timing profile.

## Specification attribution

CommonMark examples are from John MacFarlane’s CommonMark 0.31.2 specification. GFM extension examples are from the frozen official 0.29-gfm website response retrieved on September 14, 2026, based on CommonMark. The inputs and expected outputs reproduced in the JSON files retain the [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) license. They are not covered by the repository’s MIT license. [Fixture provenance](../../../benchmarks/compatibility-audit/README.md#fixture-provenance).
