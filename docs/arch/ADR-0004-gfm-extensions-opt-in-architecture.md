# ADR-0004: GFM Extensions — Opt-in Architecture

**Status:** Accepted
**Date:** 2026-02-08

## Context

ferromark needed GFM (GitHub Flavored Markdown) compatibility: tables, strikethrough, task lists, autolink literals, and disallowed raw HTML. These features are non-trivial additions that should not penalise users who only need CommonMark.

## Decision

Each GFM extension is controlled by an independent boolean in `Options`:

| Extension | Option | Default |
|---|---|---|
| Tables | `tables` | `true` |
| Strikethrough | `strikethrough` | `true` |
| Task lists | `task_lists` | `true` |
| Autolink literals | `autolink_literals` | `false` |
| Disallowed raw HTML | `disallowed_raw_html` | `true` |

**Zero-overhead when disabled:**

- **Block parser** fast paths (`parse_simple_paragraph_run`, `parse_line_content_with_indent`) bail out early when the relevant option is off.
- **Inline parser** checks `strikethrough` and `autolink_literals` bools — `~` added to SIMD char sets (negligible cost), autolink pre-filter uses rare byte patterns (`://`, `@`, `www.`).
- **Tables** use a `could_be_delimiter_row()` pre-filter before expensive `split_table_cells()`.

## Consequences

- Autolink literals default to `false` because the 3 memchr passes per inline chunk add ~3-5% overhead.
- Total GFM overhead with all defaults: ~8-12% vs pre-GFM baseline.
- New extensions follow the same pattern: add Options flag, gate detection in block/inline parsers, update fast-path bail-outs.
