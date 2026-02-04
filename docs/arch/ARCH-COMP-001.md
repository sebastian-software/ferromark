# ARCH-COMP-001: md4c vs pulldown-cmark vs md-fast

**Scope**: Architectural and performance-relevant comparison focused on CommonMark behavior, with emphasis on link reference definitions, pipeline, and DoS guards.

**Score format**: `5/5` = standout for performance + CommonMark correctness in this area. Scores are a snapshot and should be revisited after profiling or behavior changes.

| Area | md4c | pulldown-cmark | md-fast | Notes |
|---|---|---|---|---|
| Parsing pipeline | Two-phase: line/block analysis, build ref-def hashtable, then process blocks and inlines. `4/5` | Tree-based two-pass parser; first pass builds tree + refdefs, second pass resolves inlines. `4/5` | Streaming block parse; inline parsing per paragraph with refdefs extracted at paragraph close. `4/5` | Tradeoff is streaming latency vs having all refdefs available. |
| Ref-def detection point | Consumes refdefs only at start of paragraph or setext header block on block end. `4/5` | First pass scans for refdefs before paragraph parsing; refdefs act like paragraph for lazy continuation. `4/5` | Extracts refdefs from the start of paragraph at `close_paragraph`. `4/5` | All three align with CommonMark rule: refdefs only at paragraph start. |
| Ref-def parsing | Dedicated label, destination, and optional title parsing with multiline support. `4/5` | Explicit label scan with interrupt rules; destination + optional title parsing. `4/5` | `parse_link_ref_def` parses label, dest, title over paragraph text. `4/5` | Implementation details differ, semantics similar. |
| Label normalization | Unicode whitespace collapse + Unicode case-folding in hash/compare. `5/5` | Collapses ASCII whitespace in label; uses `UniCase` for case-insensitive labels; table-specific escape handling. `4/5` | Decodes entities, processes backslash escapes for `[]\\`, collapses whitespace, lowercases (special-cases ß). `4/5` | md4c is strongest on Unicode folding. |
| Ref-def storage/lookup | Hashtable with bucket lists and sorting; built after parsing. `4/5` | `HashMap<LinkLabel, LinkDef>` via `RefDefs`. `4/5` | `Vec<LinkRefDef>` + `HashMap<label, index>`. `4/5` | All are O(1) average lookup. |
| Duplicate label handling | Duplicates ignored; first wins. `4/5` | `entry().or_insert` keeps first. `4/5` | `insert` checks existing and keeps first. `4/5` | CommonMark: first definition wins. |
| Inline parsing timing | Inline processing occurs after refdefs are known (no streaming). `3/5` | Inline passes run in second pass when needed. `4/5` | Inline parsing is streaming per paragraph after refdef extraction. `4/5` | md-fast has best streaming potential. |
| Inline emphasis algorithm | Delimiter stack with marks + backtracking. `4/5` | Delimiter stack with multi-pass inline resolution. `4/5` | Delimiter + mark stack; multi-phase inline emit. `4/5` | Comparable approaches; perf tuning is key. |
| Link destination paren limit | 32-level nesting limit. `5/5` | `LINK_MAX_NESTED_PARENS = 32`. `5/5` | `MAX_LINK_PAREN_DEPTH = 32`. `5/5` | All align with CommonMark safety allowance. |
| Ref-def expansion limit | Limits total refdef expansion output size. `4/5` | `link_ref_expansion_limit` default `max(text.len(), 100_000)`. `4/5` | `MAX_LINK_REF_EXPANSIONS = 100 * 1024`. `4/5` | Prevents recursive expansion DoS. |
| HTML block parsing | Dedicated HTML block types; early termination by tag/end conditions. `4/5` | HTML block types with scanner-driven detection. `4/5` | HTML block parsing exists but must match all CommonMark block types precisely. `3/5` | NEEDS WORK: verify all 7 HTML block types + termination rules. |
| Raw HTML passthrough | Preserves HTML block/inline by default. `4/5` | Preserves HTML block/inline by default. `4/5` | Preserves HTML block/inline by default. `4/5` | Rendering should align; ensure escaping rules for HTML blocks vs inline are spec-accurate. |
| List tightness | Computes tight/loose lists from blank line adjacency. `4/5` | Tightness computed in first pass with list metadata. `4/5` | Tightness computed during block parse with container state. `4/5` | Ensure behavior matches CommonMark edge cases (blank lines in list items). |
| Setext handling | Special-case underline line; supports refdef stripping before setext conversion. `4/5` | Setext handling in first pass with refdef interruption rules. `4/5` | Strips refdefs before setext conversion. `4/5` | All align with spec approach. |
| Autolinks | Dedicated scan for `<...>` and email autolinks. `4/5` | Scanner-based autolink parsing. `4/5` | Inline autolinks implemented. `4/5` | Verify edge cases (punct, unicode, angle constraints). |
| Entity handling | Decode entities at render time; parsing aware of `&`. `4/5` | Entity decoding where required. `4/5` | Decode entities in label normalization and renderer; fast-path when no `&`. `4/5` | md-fast already optimized; ensure behavior parity. |
| Backslash escapes | Full CommonMark escape table. `4/5` | Full escape set; table-specific behavior. `4/5` | Escape handling in inline + label normalization. `4/5` | Confirm escape set coverage for HTML blocks. |
| Tables / extensions | Supports some extensions; not CommonMark core. `3/5` | GFM tables, footnotes, other extensions. `4/5` | Extensions as optional. `3/5` | Not core CommonMark; measure impact on performance. |
| Footnotes / extensions | Not core. `2/5` | Footnotes supported with extra pass. `4/5` | Optional. `3/5` | Extensions should not hurt core performance. |
| Memory layout | Central block buffer; refdefs array; hashtable built after. `4/5` | Tree + pooled allocations for strings. `4/5` | Streaming buffers; event vectors preallocated. `4/5` | All efficient; md-fast benefits from event preallocation. |
| Streaming output latency | Lower (needs post-pass). `3/5` | Medium (two-pass). `3/5` | Higher streaming potential (single pass + buffered refdef extraction). `4/5` | md-fast best aligned with streaming output. |
| SIMD / CPU tuning | C baseline; easy SIMD spots in scanners. `3/5` | Rust baseline; some hotspots in scanners. `3/5` | Rust baseline; hotspots identified by profiling. `3/5` | NEEDS WORK: add SIMD or memchr-heavy paths where safe. |
| DoS limits | Hard limits for nesting, parsing guards. `4/5` | Several limits (link parens, etc). `4/5` | Central `limits.rs` with sane caps. `4/5` | All good; verify limits align with CommonMark allowance. |

**Primary references**
- md4c ref-def dictionary and Unicode folding: `/Users/sebastian/Workspace/md4c/src/md4c.c:1560`.
- md4c ref-def consumption at paragraph/setext start: `/Users/sebastian/Workspace/md4c/src/md4c.c:5060`.
- md4c two-phase flow: `/Users/sebastian/Workspace/md4c/src/md4c.c:6386`.
- pulldown-cmark two-pass parser note: `/Users/sebastian/Workspace/pulldown-cmark/pulldown-cmark/src/parse.rs:1`.
- pulldown-cmark refdef parsing loop: `/Users/sebastian/Workspace/pulldown-cmark/pulldown-cmark/src/firstpass.rs:456`.
- pulldown-cmark label normalization: `/Users/sebastian/Workspace/pulldown-cmark/pulldown-cmark/src/linklabel.rs:33`.
- pulldown-cmark link dest nesting limit: `/Users/sebastian/Workspace/pulldown-cmark/pulldown-cmark/src/parse.rs:55`.
- md-fast refdef extraction at paragraph close: `/Users/sebastian/Workspace/md-new/src/block/parser.rs:2228`.
- md-fast label normalization: `/Users/sebastian/Workspace/md-new/src/link_ref.rs:32`.
- md-fast DoS limits: `/Users/sebastian/Workspace/md-new/src/limits.rs:5`.

**NEEDS WORK summary (md-fast)**
- HTML block parsing: verify all 7 CommonMark HTML block types and termination rules.
- SIMD or scanner-level acceleration on hot paths.
- Re-evaluate label normalization vs full Unicode folding rules.
- Autolink and HTML edge cases in CommonMark conformance tests.

