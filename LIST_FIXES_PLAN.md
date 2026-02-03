# List Handling Fixes - Detailed Plan

Based on analysis of md4c and pulldown-cmark reference implementations.

## Current Status (Updated 2026-02-03)

- Lists: 15/17 (88.2%) - was 9/17 (52.9%)
- List items: 24/28 (85.7%) - was 17/28 (60.7%)
- Block quotes: 20/20 (100%) - was 18/20 (90%)
- Overall in-scope: 94.5% (344/364) - was 91.2% (332/364)

### Completed Phases
- **Phase 1**: Content indent calculation using absolute column positions ✓
- **Phase 3**: Nested list detection, closing, and tight/loose rendering ✓
- **Phase 4**: Two-blank-line rule enforcement ✓
- **Phase 5**: Indented code detection within list items ✓

### Additional Fixes
- Buffer blank lines in indented code blocks (no trailing blanks)
- Detect blank lines after container matching (e.g., ">>")
- Only apply same-list continuation when all parent containers matched
- Blank list items cannot interrupt paragraphs
- Blank lines without > markers close blockquotes
- Recognize blank list items in same-list continuation
- Enable lazy continuation for list item paragraphs
- Don't recognize block starts at 4+ indent in lazy continuation
- Close lists when indent >= 4 prevents new items
- Two-blank-line rule keeps list open for more items

### Remaining
- **Phase 2**: Container matching edge cases (fenced code inside list items)
- HTML block handling (currently out of scope)

## Original Status

- Lists: 9/17 (52.9%)
- List items: 17/28 (60.7%)
- Total: 26/45 (~58%)

## Key Insights from Reference Implementations

### md4c Approach
1. **Two-tier indentation**: `mark_indent` (marker position) + `contents_indent` (content threshold)
2. **Retroactive loose marking**: Stores block offset to mark lists loose after detection
3. **Two-blank-line rule**: List item can begin with at most one blank line
4. **Compatibility check**: New markers must use same character and proper indent

### pulldown-cmark Approach
1. **ListItem(indent)**: Stores required indentation for continuation
2. **Spine traversal**: Efficient parent/container lookups
3. **Lazy tight/loose**: Assumes tight, converts on blank lines between items
4. **Tab handling**: Tabs = 4-space tab stops

## Identified Issues in md-fast

### Issue 1: Incorrect content_indent calculation
**Current**: We store `content_indent` but may not calculate it correctly per CommonMark.

**CommonMark Rule**:
- Content indent = position after marker + 1-4 spaces
- If 4+ spaces after marker, only 1 counts (rest is content indentation)

**Fix**: Update `try_list_item` to correctly calculate content_indent.

### Issue 2: Nested list detection
**Current**: Nested lists may not trigger correctly.

**CommonMark Rule**:
- A new list item can interrupt a paragraph only if:
  - Same list type (ordered/unordered)
  - Same marker character (-, *, +) for unordered
  - Proper indentation relative to parent

**Fix**: Check indentation against parent's `contents_indent`.

### Issue 3: Tight/loose list detection incomplete
**Current**: We track `blank_in_item` but may not handle all cases.

**CommonMark Rules**:
- Tight if NO blank lines between items AND no blank lines in items
- Blank line between items → loose
- Blank line inside item (before nested content) → loose

**Fix**: Track blank lines more precisely, mark retroactively.

### Issue 4: Two-blank-line rule not enforced
**Current**: We don't enforce the "at most one blank line" rule.

**CommonMark Rule**:
- A list item can begin with at most one blank line
- Two consecutive blank lines end the list

**Fix**: Track consecutive blank lines in list items.

### Issue 5: Indented code inside list items
**Current**: May not handle indented code threshold correctly.

**CommonMark Rule**:
- Inside list item, code indent = `contents_indent + 4`
- Not just 4 spaces from line start

**Fix**: Pass `contents_indent` context to indented code detection.

## Implementation Plan

### Phase 1: Fix content_indent calculation
```
Location: src/block/parser.rs, try_list_item()
```

1. After detecting list marker, calculate content_indent:
   - For `- item`: marker_pos + 2 (marker + 1 space)
   - For `1. item`: marker_pos + digits + 2 (digits + marker + 1 space)
   - Add 0-3 additional spaces if present

2. Cap at 4 spaces after marker (rest becomes content indentation)

### Phase 2: Fix container matching for lists
```
Location: src/block/parser.rs, match_containers()
```

1. For ListItem containers, check:
   - Line indent >= content_indent → continues item
   - Line indent < content_indent but has same marker → new item
   - Line indent < content_indent, different content → closes item

2. Properly handle blank lines in list items

### Phase 3: Fix tight/loose detection
```
Location: src/block/parser.rs, OpenList struct
```

1. Track `had_blank_line` per list (not just per item)
2. When closing list, check if any blank lines occurred
3. Update ListEnd event to include correct `tight` flag

### Phase 4: Enforce two-blank-line rule
```
Location: src/block/parser.rs, handle_blank_line_containers()
```

1. Track consecutive blank line count in list items
2. On second blank line, close the list item
3. Don't allow continuation after two blank lines

### Phase 5: Fix indented code in list context
```
Location: src/block/parser.rs, parse_line_content()
```

1. Calculate effective indent based on container stack
2. For indented code: need `contents_indent + 4` spaces
3. Pass context to indented code detection

## Test Cases to Verify

### Basic nested list (Example 307)
```markdown
- foo
  - bar
    - baz


      bim
```
Expected: Properly nested `<ul>` structure with `bim` in innermost item.

### List with indented code (Example 254)
```markdown
1.  A paragraph
    with two lines.

        indented code

    > A block quote.
```
Expected: Code block inside list item (8 spaces = 4 for item + 4 for code).

### Tight vs loose (Example 308)
```markdown
- foo
- bar

<!-- -->

- baz
- bim
```
Expected: First list tight, HTML comment, second list tight.

### Two-blank-line rule (Example 257)
```markdown
 -    one

     two
```
Expected: List ends after "one", "two" becomes code block.

## Order of Implementation

1. **Phase 1**: content_indent calculation (foundational)
2. **Phase 2**: container matching (depends on Phase 1)
3. **Phase 3**: tight/loose detection (can be parallel)
4. **Phase 4**: two-blank-line rule (builds on Phase 2)
5. **Phase 5**: indented code in lists (depends on Phase 1)

## Success Criteria

- Lists: 17/17 (100%)
- List items: 28/28 (100%)
- Overall compliance: 95%+
