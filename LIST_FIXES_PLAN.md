# CommonMark Compliance - Remaining Work

## Current Status (2026-02-03)

**Overall: 94.5% (344/364 in-scope tests)**

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
| Lists | 88.2% (15/17) | 2 HTML-related |
| List items | 85.7% (24/28) | 4 remaining |
| Entity refs | 84.6% (11/13) | 2 remaining |
| Links | 81.6% (40/49) | 9 remaining |

## Remaining Failures (20 tests)

### List Items (4 failures)

**Example 263**: Fenced code inside list item
```markdown
1.  foo

    ```
    bar
    ```

    baz

    > bam
```
Issue: Fenced code fence ```` ``` ```` at 4-space indent inside list item is treated as indented code content instead of fenced code start.

**Example 278**: Fenced code in blank list items
```markdown
-
  foo
-
  ```
  bar
  ```
-
      baz
```
Issue: Same as 263 - fenced code fence not recognized at proper indent inside list.

**Example 292**: Nested blockquote + list lazy continuation
```markdown
> 1. > Blockquote
continued here.
```
Issue: Lazy continuation through nested blockquote + list needs paragraph tags.

### Lists (2 failures - HTML related, out of scope)

**Example 308, 309**: HTML comments `<!-- -->` not rendered correctly.
These require HTML block parsing which is currently out of scope.

**Example 315**: Minor whitespace issue (`<li>\n</li>` vs `<li></li>`)

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

1. **Fenced code inside list items** (Examples 263, 278)
   - Location: `parse_line_content()` and `try_code_fence()`
   - Issue: When inside a list item with indented content, fenced code fence at content_indent level should start a fenced code block, not be treated as indented code
   - Fix: Check for fenced code BEFORE checking for indented code when inside list item

2. **Link edge cases** (5 of 9 are likely fixable)
   - Multiline link destinations in angle brackets
   - Nested bracket handling
   - Whitespace handling in link syntax

3. **Empty list item whitespace** (Example 315)
   - Just a rendering fix for empty items

### Medium Priority

4. **Lazy continuation in nested containers** (Example 292)
   - Complex interaction between blockquote and list item paragraphs

5. **Entity reference edge cases**
   - Need investigation

### Out of Scope

- HTML block handling (Examples 308, 309)
- Reference link definitions

## Implementation Notes

### Fenced Code in List Items

The core issue: when we're inside a list item and see a line with 4+ spaces, we currently:
1. Check for indented code (indent >= 4)
2. Start indented code block

But we should:
1. Check for fenced code fence FIRST (even with indent)
2. Only then check for indented code

The fenced code detection should work at any indent level inside a list item, as long as it's at or after the content_indent position.

### Link Parsing

The link parser needs updates for:
1. Multiline destinations in angle brackets
2. Proper bracket nesting/balancing
3. Whitespace handling between components

## Success Criteria

Target: 96%+ compliance (350+/364)
- Lists: 17/17 (100%) - requires HTML blocks
- List items: 27/28 (96%+)
- Links: 45/49 (92%+)
- Entity refs: 13/13 (100%)
