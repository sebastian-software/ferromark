# ADR-0005: Front Matter — Pre-parse Extraction

**Status:** Accepted
**Date:** 2026-02-09

## Context

Front matter (YAML/TOML metadata between `---` or `+++` delimiters) must be extracted before Markdown parsing begins. If passed to the block parser, the opening `---` would be interpreted as a thematic break or setext heading underline.

## Decision

Front matter is handled entirely **before** block parsing:

1. `extract_front_matter()` scans for exactly `---` or `+++` at byte 0, finds the matching closing delimiter, and returns `(content, rest_offset)`.
2. `parse()` / `parse_with_options()` return `ParseResult { html, front_matter: Option<&str> }` — zero-copy slice into the input.
3. `to_html_*_with_options()` silently strip front matter when `options.front_matter` is `true`.
4. The `BlockParser` is never modified — it receives the markdown starting after the closing delimiter.

### Detection rules

- Opening delimiter: exactly 3 chars (`---` or `+++`) at byte offset 0, optional trailing whitespace, then newline.
- `----` (4+) is NOT a delimiter.
- Closing delimiter must use the same character as the opener.
- No closing delimiter = no front matter (entire document is parsed as markdown).

## Consequences

- Zero performance overhead when `front_matter: false` (default).
- Zero changes to `BlockParser` — all complexity is in `extract_front_matter()`.
- Front matter content is a `&str` borrow, no allocation needed.
- `ParseResult` carries both HTML and front matter, enabling YAML/TOML parsing by the caller.
