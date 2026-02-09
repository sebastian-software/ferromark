# ADR-0007: Math Spans — Dollar Sign Delimiters

**Status:** Accepted
**Date:** 2026-02-09

## Context

Math rendering is a high-demand feature for technical documentation. Both pulldown-cmark and comrak support `$`/`$$` delimiters. GitHub itself renders math using this syntax.

## Decision

Math spans follow the **code span pattern**:

- `$...$` for inline math.
- `$$...$$` for display math.
- Content inside is **not parsed** for inline markup (emphasis, links, etc.).

### Resolution precedence

Math is resolved after code spans (which have highest precedence) and before links/emphasis:

1. Code spans (`\`...\``) — highest, marks content as `IN_CODE`.
2. **Math spans** (`$...$`, `$$...$$`) — marks content as `IN_CODE`.
3. Autolinks, HTML spans.
4. Links, images, reference links.
5. Emphasis, strikethrough — lowest.

### Rendering (pulldown-cmark compatible)

- `$x^2$` renders as `<code class="language-math math-inline">x^2</code>`.
- `$$E=mc^2$$` renders as `<code class="language-math math-display">E=mc^2</code>`.

This format works with KaTeX, MathJax, and GitHub's math rendering.

### Content processing

- Leading/trailing single space stripped if both present and content is not all-space (same as code spans).
- Newlines converted to spaces in rendered output.
- HTML special characters (`<`, `>`, `&`, `"`) are escaped.
- Backslash before `$` escapes it (`\$` is a literal `$`).

### Files modified

- `src/inline/math.rs` — `resolve_math_spans()` (similar to `code_span.rs`).
- `src/inline/marks.rs` — `$` in `SPECIAL_CHARS`, `next_special`, `collect_marks`.
- `src/inline/simd.rs` — `$` in NEON char sets.
- `src/inline/mod.rs` — `$` in `has_inline_specials`; `MathInline`/`MathDisplay` in `EmitKind`.
- `src/inline/event.rs` — `MathInline(Range)`, `MathDisplay(Range)` variants.
- `src/lib.rs` — `math: bool` option (default `false`); rendering in `render_inline_event`.

### Option

- `math: bool` (default `false`).
- When `false`, `$` is treated as literal text and `resolve_math_spans` is not called.

## Consequences

- Zero overhead when `math: false` — `$` marks are collected but not resolved; the fast-path `!has_specials` check handles the common case.
- `$` is added to SIMD char sets (negligible cost, same as `~` for strikethrough).
- Math option defaults to `false` for backwards compatibility and CommonMark compliance.
