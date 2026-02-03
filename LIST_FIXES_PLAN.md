# CommonMark Compliance - Remaining Work

## Current Status (2026-02-03)

**Overall: 95.1% (346/364 in-scope tests)**

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
| Lists | 70.6% (12/17) | 5 remaining (HTML-related) |
| Entity refs | 84.6% (11/13) | 2 remaining |
| Links | 81.6% (40/49) | 9 remaining |

## Recently Fixed

### Fenced Code in List Items (Examples 263, 278) - FIXED

The core issue was that fenced code blocks inside list items were being handled before container matching. The fix:
1. Move `fence_state` check AFTER `match_containers()` in `parse_line()`
2. Add `parse_fence_line_in_container()` to handle fenced code within containers
3. Close fenced code when containers don't match
4. Add newlines before block elements in tight list items for proper rendering

## Remaining Failures (18 tests)

### List Items (2 failures)

**Example 292, 293**: Nested blockquote + list lazy continuation
```markdown
> 1. > Blockquote
continued here.
```
Issue: Lazy continuation through nested blockquote + list needs paragraph tags around content in the inner blockquote.

### Lists (5 failures - mostly HTML related)

**Example 308, 309**: HTML comments `<!-- -->` not rendered correctly.
These require HTML block parsing which is currently out of scope.

**Example 300**: Uses setext heading (out of scope but being counted as failure).

**Example 315**: Minor whitespace issue - requires investigation.

### Links (9 failures)

**Example 491**: Link with newline in angle-bracket destination
```markdown
[link](<foo
bar>)
```

**Example 494**: Angle bracket edge cases
```markdown
[a](<b)c
[a](<b)c>
[a](<b>c)
```

**Example 510**: Link with whitespace/newline before title
```markdown
[link](   /uri
  "title"  )
```

**Example 512**: Nested brackets in link text
```markdown
[link [foo [bar]]](/uri)
```

**Example 518**: Link inside link text
```markdown
[foo [bar](/uri)](/uri)
```

### Entity References (2 failures)

Need investigation - likely edge cases with numeric character references.

## Priority Order

### High Priority (In Scope, Achievable)

1. **Link edge cases** (5 of 9 are likely fixable)
   - Multiline link destinations in angle brackets
   - Nested bracket handling
   - Whitespace handling in link syntax

2. **Entity reference edge cases**
   - Need investigation

### Medium Priority

3. **Lazy continuation in nested containers** (Examples 292, 293)
   - Complex interaction between blockquote and list item paragraphs
   - Requires ensuring proper paragraph wrapping in nested blockquotes

### Out of Scope

- HTML block handling (Examples 308, 309)
- Reference link definitions
- Setext headings (Example 300)

## Success Criteria

Target: 96%+ compliance (350+/364)
- Current: 346/364 (95.1%)
- Need: 4 more tests to reach 96%
