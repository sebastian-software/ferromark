# Native benchmark flag contract

This run compares Markdown to HTML using each original native public API.
Ferromark v2 is pinned to `33c216b` and v1 to `4e15141`; the other engines retain
exactly the original comparison pins. No parser implementation is modified.

| Behavior | CommonMark lane (17 documents) | Shared extension lane (40 documents) |
| --- | --- | --- |
| Core Markdown | On | On |
| Pipe tables, strikethrough, task lists | Off | On |
| Bare URL/email autolinks | Off | Off |
| Raw HTML | Pass through | Pass through |
| GFM tagfilter, sanitization, new-tab targets | Off | Off |
| Footnotes, frontmatter, line comments, definition lists | Off | Off |
| Math, superscript/subscript, wiki links, MDX | Off | Off |
| Smart punctuation, heading attributes, table layout extensions | Off | Off |
| Heading IDs, callouts, inline TOC, fence metadata | Off where available | Off where available |

The extension lane is **not full GFM**: pulldown-cmark has no native expanded
URL/email autolinking or tagfilter option. Enabling a vendor's `gfm` preset
would also enable different extras. The corpus's legacy profile label `gfm`
is retained in raw data and maps explicitly to `gfm-shared` in this worker.

## Exact adapter choices

- **v2:** `ParserOptions::commonmark()` with `tables`, `strikethrough`, and
  `task_lists` set from the lane; `HtmlRendererOptions::commonmark()`.
  The renderer profile disables automatic IDs, callouts, TOC substitution,
  metadata cleanup, URL autolinking, and new-tab targets. New optional syntax
  such as definitions and line comments is disabled, even on input containing
  that notation.
- **v1:** `Options::commonmark()`, `RenderPolicy::Trusted`, lane-controlled
  tables/strikethrough/task lists, and explicit `heading_ids=false`,
  `callouts=false`, `disallowed_raw_html=false`.
- **pulldown-cmark:** `Options::empty()` plus only `ENABLE_TABLES`,
  `ENABLE_STRIKETHROUGH`, and `ENABLE_TASKLISTS` in the extension lane.
- **md4c:** parser flags `0` or `0x0100 | 0x0200 | 0x0800`; renderer flags `0`.
- **Bun native:** initialize `Options::default()`, then set every entry in
  `BOOL_FIELD_SETTERS` to false except the lane-controlled `tables`,
  `strikethrough`, and `tasklists` fields.
- **OX original:** parser defaults plus the same three lane switches;
  renderer URL autolinking and link/new-tab targeting disabled. It has no
  public switches for automatic IDs, callouts, inline TOC, or fence language
  metadata normalization. The benchmark retains that original behavior.
  Renderer hooks would require custom replacement rendering, so they are not
  substituted for the engine's native renderer.

Resource checks stay native. V1's resource-limit fallback is explicitly
rejected during verification; no measured corpus input triggers it.

## Executable checks

`run.py:behavior_checks` verifies both lifecycles, both lanes, and repeated
mixed input cycles for all six engines before timing. Guards cover duplicate
headings, callouts, `[[toc]]`, `js{1}` fences, code whitespace, raw HTML tagfilter,
frontmatter, footnotes, definition lists, comments, sub/superscript, table extras,
new-tab targets, `.md` links, and reference state. It verifies the four original
OX exceptions positively rather than merely skipping assertions.

Single-tilde strikethrough remains dialect-dependent, even with these same
flags: a first guard incorrectly expected `H~2~O` to stay literal in the shared
lane. That assertion was corrected before timing; subscript remains forbidden,
and literal single tildes are asserted in the CommonMark lane. This was a guard
correction, not a parser or corpus modification.

Output equivalence is a separate gate: all six engines agree on 14 frozen
corpus cases; the five configurable renderers agree on 50. Extra IDs, task CSS
classes, code-language changes, link destinations, and content are not stripped
to manufacture agreement. No input is edited or removed based on timing.
