# ADR-0006: Heading IDs — Deferred Tag Emission

**Status:** Accepted
**Date:** 2026-02-09

## Context

GitHub-compatible heading IDs (`<h1 id="hello-world">`) require knowing the heading's text content to generate the slug. However, the `HeadingStart` block event fires before any content is received. The `<hN>` tag was previously emitted in the `HeadingStart` handler.

## Decision

**Defer the heading open tag** from `HeadingStart` to `HeadingEnd`:

1. `HeadingStart` — stores the heading level and begins content accumulation (no HTML output).
2. Content arrives via `Text` and `SoftBreak` events, accumulated in `HeadingState`.
3. `HeadingEnd` — generates the slug from raw content, emits `<hN id="slug">`, then renders inline content, then emits `</hN>`.

### Slug algorithm (GitHub-compatible)

1. Strip inline markup delimiters (`*`, `~`, `` ` ``, `[`, `]`, `!`, `#`) from raw bytes.
2. Lowercase ASCII; preserve multibyte UTF-8.
3. Replace whitespace runs with `-`.
4. Remove non-alphanumeric, non-hyphen, non-underscore ASCII characters.
5. Strip leading/trailing `-`.
6. Deduplicate via `HeadingIdTracker`: append `-1`, `-2`, etc. on collision.
7. Empty slug after stripping falls back to `"heading"`.

### Option

- `heading_ids: bool` (default `true`).
- When `false`, plain `<hN>` tags are emitted (no `id` attribute).
- `HtmlWriter::heading_start_with_id(level, id)` emits `<hN id="...">`.

## Consequences

- Default `to_html()` output now includes heading IDs — CommonMark spec tests use `heading_ids: false`.
- Slug is generated from pre-inline-parse text, so markup delimiters are stripped but entity-decoded text is not processed (matching GitHub behaviour).
- `HeadingIdTracker` is per-document, reset for each parse call.
