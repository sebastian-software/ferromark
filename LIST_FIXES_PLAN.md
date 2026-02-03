# CommonMark Compliance - Remaining Work

## Current Status (2026-02-03)

**Overall: 96.2% (350/364 in-scope tests)**

| Section | Status | Notes |
|---------|--------|-------|
| ATX headings | 100% (16/16) | Complete |
| Autolinks | 100% (19/19) | Complete |
| Backslash escapes | 100% (8/8) | Complete |
| Blank lines | 100% (1/1) | Complete |
| Block quotes | 100% (20/20) | Complete |
| Code spans | 100% (20/20) | Complete |
| Emphasis | 100% (129/129) | Complete |
| Fenced code blocks | 100% (3/3) | Complete |
| Hard line breaks | 100% (11/11) | Complete |
| Images | 100% (1/1) | Complete |
| Inlines | 100% (1/1) | Complete |
| Paragraphs | 100% (6/6) | Complete |
| Precedence | 100% (1/1) | Complete |
| Soft line breaks | 100% (2/2) | Complete |
| Textual content | 100% (3/3) | Complete |
| Thematic breaks | 100% (16/16) | Complete |
| List items | 92.9% (26/28) | 2 remaining |
| Links | 89.8% (44/49) | 5 remaining |
| Lists | 70.6% (12/17) | 5 remaining (HTML-related) |
| Entity refs | 84.6% (11/13) | 2 remaining |

## Recently Fixed

### Link Parsing Improvements (Examples 510, 512, 518)

1. **Bracket balancing** (Example 512): `[link [foo [bar]]](/uri)` now works
   - Added `find_matching_close()` to properly balance nested brackets

2. **Link precedence** (Example 518): Inner links take precedence
   - When a link forms, outer open brackets are deactivated
   - `[foo [bar](/uri)](/uri)` → `[foo <a>bar</a>](/uri)`

3. **Newlines in link destinations** (Example 510)
   - Newlines in whitespace between URL and title are not line breaks
   - Extended `in_link_dest` check to cover entire `(...)` area

### Fenced Code in List Items (Examples 263, 278) - FIXED EARLIER

The core issue was that fenced code blocks inside list items were being handled before container matching.

## Remaining Failures (14 tests)

### Links (5 failures)

**Example 491**: Newline in angle-bracket destination
```markdown
[link](<foo
bar>)
```
Issue: Angle brackets should not be escaped when link fails to parse.

**Example 494**: Unclosed angle bracket in destination
Issue: Similar angle bracket handling.

**Example 520**: Complex nested image/link
```markdown
![[[foo](uri1)](uri2)](uri3)
```
Issue: Image alt text should include literal brackets when inner links form.

**Example 524**: Raw HTML in link text (out of scope)
**Example 526**: Autolink inside link destination (edge case)

### List Items (2 failures)

**Example 292, 293**: Lazy continuation in nested blockquotes
Issue: Paragraph wrapping in nested blockquote + list.

### Lists (5 failures - mostly HTML related)

**Example 308, 309**: HTML comments (out of scope)
**Example 300**: Setext heading (out of scope)
**Example 315**: Minor whitespace issue

### Entity References (2 failures)

Need investigation - likely edge cases with numeric character references.

## Priority Order

### In Scope, Potentially Fixable

1. **Entity reference edge cases** (2 tests)
2. **Lazy continuation in nested containers** (2 tests)

### Out of Scope / Low Priority

- HTML block handling (Examples 308, 309)
- Raw HTML in inline content (Example 524)
- Setext headings (Example 300)

## Success Criteria

Target: 96%+ compliance (350+/364) - **ACHIEVED!**
- Current: 350/364 (96.2%)
