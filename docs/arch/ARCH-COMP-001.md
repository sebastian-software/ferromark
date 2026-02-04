# ARCH-COMP-001: md4c vs pulldown-cmark vs md-fast

**Scope**: Architectural and performance-relevant comparison focused on CommonMark behavior, with emphasis on link reference definitions, pipeline, and DoS guards.

**Score format**: `5/5` = standout for performance + CommonMark correctness in this area. Scores are a snapshot and should be revisited after profiling or behavior changes.

| Area | md4c | pulldown-cmark | md-fast | Notes |
|---|---|---|---|---|
| Parsing pipeline | Two-phase: line/block analysis, build ref-def hashtable, then process blocks and inlines. `4/5` | Tree-based two-pass parser; first pass builds tree + refdefs, second pass resolves inlines. `4/5` | Streaming block parse; inline parsing happens per paragraph with refdefs extracted at paragraph close. `4/5` | Tradeoff is streaming latency vs having all refdefs available. |
| Ref-def detection point | Consumes refdefs only at start of paragraph or setext header block on block end. `4/5` | First pass scans for refdefs before paragraph parsing; refdefs act like paragraph for lazy continuation. `4/5` | Extracts refdefs from the start of paragraph at `close_paragraph`. `4/5` | All three align with CommonMark rule: refdefs only at paragraph start. |
| Ref-def parsing | Dedicated label, destination, and optional title parsing with multiline support. `4/5` | Explicit label scan with interrupt rules; destination + optional title parsing. `4/5` | `parse_link_ref_def` parses label, dest, title over paragraph text. `4/5` | Implementation details differ, but semantics are similar. |
| Label normalization | Unicode whitespace collapse + Unicode case-folding in hash/compare. `5/5` | Collapses ASCII whitespace in label; uses `UniCase` for case-insensitive labels; table-specific escape handling. `4/5` | Decodes entities, processes backslash escapes for `[]\\`, collapses whitespace, lowercases (special-cases ß). `4/5` | md4c has the most explicit Unicode folding logic. |
| Ref-def storage/lookup | Hashtable with bucket lists and sorting; built after parsing. `4/5` | `HashMap<LinkLabel, LinkDef>` via `RefDefs`. `4/5` | `Vec<LinkRefDef>` + `HashMap<label, index>`. `4/5` | All are O(1) average lookup. |
| Duplicate label handling | Duplicates ignored; first wins. `4/5` | `entry().or_insert` keeps first. `4/5` | `insert` checks existing and keeps first. `4/5` | CommonMark: first definition wins. |
| Inline parsing timing | Inline processing occurs after refdefs are known (no streaming). `3/5` | Inline passes run in second pass when needed. `4/5` | Inline parsing is streaming per paragraph after refdef extraction. `4/5` | md-fast has best streaming potential. |
| Link destination paren limit | 32-level nesting limit. `5/5` | `LINK_MAX_NESTED_PARENS = 32`. `5/5` | `MAX_LINK_PAREN_DEPTH = 32`. `5/5` | All align with CommonMark safety allowance. |
| Ref-def expansion limit | Limits total refdef expansion output size. `4/5` | `link_ref_expansion_limit` default `max(text.len(), 100_000)`. `4/5` | `MAX_LINK_REF_EXPANSIONS = 100 * 1024`. `4/5` | Prevents recursive expansion DoS. |
| Streaming output latency | Lower (needs post-pass). `3/5` | Medium (two-pass). `3/5` | Higher streaming potential (single pass + buffered refdef extraction). `4/5` | md-fast is best aligned with streaming output. |

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

**TODO**
- Add rows for HTML block parsing differences and list tightness handling if we want a broader spec coverage comparison.
- Re-score after the next profiling cycle and any refdef or streaming refactor.
