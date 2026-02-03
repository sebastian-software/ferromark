//! Block parser implementation.

use crate::cursor::Cursor;
use crate::limits;
use crate::Range;
use smallvec::SmallVec;

use super::event::{BlockEvent, ListKind, TaskState};

/// State for an open fenced code block.
#[derive(Debug, Clone)]
struct FenceState {
    /// The fence character (` or ~).
    fence_char: u8,
    /// Length of the opening fence.
    fence_len: usize,
    /// Indentation of the opening fence.
    indent: usize,
}

/// Type of container block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerType {
    /// Block quote (`>`)
    BlockQuote,
    /// List item
    ListItem {
        /// List type
        kind: ListKind,
        /// Marker character for unordered, or 0 for ordered
        marker: u8,
        /// Column where content starts (after marker + space)
        content_indent: usize,
    },
}

/// An open container on the stack.
#[derive(Debug, Clone)]
struct Container {
    /// Type of container.
    typ: ContainerType,
    /// Whether this container has had any content yet.
    #[allow(dead_code)]
    has_content: bool,
}

/// Tracks an open list that may have its items closed/reopened.
#[derive(Debug, Clone)]
struct OpenList {
    kind: ListKind,
    /// Marker character for unordered (-, *, +) or delimiter for ordered (., )).
    marker: u8,
    /// Whether the list is still tight (no blank lines in items).
    tight: bool,
    /// Whether we've seen a blank line since the last item started.
    /// Used to detect loose lists (blank line between items or inside items).
    blank_in_item: bool,
    /// Number of items so far.
    item_count: u32,
}

/// Block parser state.
pub struct BlockParser<'a> {
    /// Input bytes.
    input: &'a [u8],
    /// Current cursor position.
    cursor: Cursor<'a>,
    /// Whether we're currently in a paragraph.
    in_paragraph: bool,
    /// Accumulated paragraph text ranges.
    paragraph_lines: Vec<Range>,
    /// Current fenced code block state, if inside one.
    fence_state: Option<FenceState>,
    /// Whether we're in an indented code block.
    in_indented_code: bool,
    /// Stack of open containers (blockquotes, list items).
    container_stack: SmallVec<[Container; 8]>,
    /// Whether we're in a tight list context.
    #[allow(dead_code)]
    tight_list: bool,
    /// Currently open lists (for tracking across item closes).
    open_lists: SmallVec<[OpenList; 4]>,
}

impl<'a> BlockParser<'a> {
    /// Create a new block parser.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            cursor: Cursor::new(input),
            in_paragraph: false,
            paragraph_lines: Vec::new(),
            fence_state: None,
            in_indented_code: false,
            container_stack: SmallVec::new(),
            tight_list: false,
            open_lists: SmallVec::new(),
        }
    }

    /// Parse all blocks and collect events.
    pub fn parse(&mut self, events: &mut Vec<BlockEvent>) {
        while !self.cursor.is_eof() {
            self.parse_line(events);
        }

        // Close any open paragraph at end of input
        self.close_paragraph(events);

        // Close any unclosed fenced code block
        if self.fence_state.is_some() {
            self.fence_state = None;
            events.push(BlockEvent::CodeBlockEnd);
        }

        // Close any unclosed indented code block
        if self.in_indented_code {
            self.in_indented_code = false;
            events.push(BlockEvent::CodeBlockEnd);
        }

        // Close all open containers
        self.close_all_containers(events);
    }

    /// Parse a single line.
    fn parse_line(&mut self, events: &mut Vec<BlockEvent>) {
        let line_start = self.cursor.offset();

        // If we're inside a fenced code block, handle it specially
        if self.fence_state.is_some() {
            self.parse_fence_line(events);
            return;
        }

        // Check for blank line first (before any space skipping)
        let initial_spaces = self.count_leading_spaces();
        if self.is_blank_after(initial_spaces) {
            self.cursor = Cursor::new_at(self.input, line_start + initial_spaces);
            if !self.cursor.is_eof() && self.cursor.at(b'\n') {
                self.cursor.bump();
            }
            // Blank lines inside indented code are preserved
            if self.in_indented_code {
                // Emit just the newline
                events.push(BlockEvent::Text(Range::new((line_start + initial_spaces) as u32, (line_start + initial_spaces + 1) as u32)));
                return;
            }
            self.close_paragraph(events);
            self.handle_blank_line_containers(events);
            return;
        }

        // Reset to line start for container matching
        self.cursor = Cursor::new_at(self.input, line_start);

        // Try to match and continue existing containers
        // This handles the indent requirements per container type
        let matched_containers = self.match_containers(events);

        // Get current indent after container matching
        let indent = self.cursor.skip_spaces();

        // If we have unmatched containers, check for lazy continuation or close them
        if matched_containers < self.container_stack.len() {
            // If we're in an indented code block and containers don't match, close it
            if self.in_indented_code {
                self.in_indented_code = false;
                events.push(BlockEvent::CodeBlockEnd);
            }

            // Check if this is a thematic break - it should close all containers first
            if indent < 4 && self.peek_thematic_break() {
                self.close_all_containers(events);
                self.try_thematic_break(events);
                return;
            }

            // Check for lazy continuation (paragraph continues without > marker)
            if self.can_lazy_continue(matched_containers, indent) {
                // Don't close containers - just add this line to the paragraph
                let line_start = self.cursor.offset();
                self.parse_paragraph_line(line_start, events);
                return;
            }

            // close_containers_from is smart about keeping lists open when starting new items
            self.close_containers_from(matched_containers, events);
        }

        // If we're in an indented code block and containers matched, handle continuation
        if self.in_indented_code {
            if indent >= 4 {
                // Continue the code block
                let extra_spaces = indent.saturating_sub(4);
                let text_start = self.cursor.offset() - extra_spaces;
                let line_end = self.find_line_end();
                let content_end = if !self.cursor.is_eof() && self.cursor.at(b'\n') {
                    self.cursor.bump();
                    line_end + 1
                } else {
                    line_end
                };
                events.push(BlockEvent::Text(Range::new(text_start as u32, content_end as u32)));
                return;
            } else {
                // Close the code block - not enough indent
                self.in_indented_code = false;
                events.push(BlockEvent::CodeBlockEnd);
            }
        }

        // Check for thematic break (also when all containers matched, e.g. inside blockquote)
        if indent < 4 {
            if self.try_thematic_break(events) {
                return;
            }
        }

        // Check for new container starts (blockquote, list)
        if indent < 4 && self.container_stack.len() < limits::MAX_BLOCK_NESTING {
            // Check for blockquote
            if self.try_blockquote(events) {
                // Recursively parse the rest of the line
                self.parse_line_content(events);
                return;
            }

            // Check for list item - either continuing an existing list or starting new
            if self.try_list_item(events) {
                self.parse_line_content(events);
                return;
            }
        }

        // Check for indented code block (4+ spaces, not in paragraph)
        if indent >= 4 && !self.in_paragraph {
            self.start_indented_code(indent, events);
            return;
        }

        // Parse regular block content
        self.parse_line_content(events);
    }

    /// Count leading spaces without consuming them.
    fn count_leading_spaces(&self) -> usize {
        let slice = self.cursor.remaining_slice();
        let mut count = 0;
        for &b in slice {
            if b == b' ' {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Check if line is blank after given number of spaces.
    fn is_blank_after(&self, spaces: usize) -> bool {
        let slice = self.cursor.remaining_slice();
        if spaces >= slice.len() {
            return true;
        }
        slice[spaces] == b'\n'
    }

    /// Parse line content after container markers have been handled.
    fn parse_line_content(&mut self, events: &mut Vec<BlockEvent>) {
        let indent = self.cursor.skip_spaces();

        // Check for blank line (can happen after container markers)
        if self.cursor.is_eof() || self.cursor.at(b'\n') {
            if !self.cursor.is_eof() {
                self.cursor.bump();
            }
            self.close_paragraph(events);
            return;
        }

        // Try to parse block-level constructs (only if indent < 4)
        if indent < 4 {
            // Check for thematic break FIRST - `* * *` is a thematic break, not a list
            if self.try_thematic_break(events) {
                return;
            }

            // Check for nested containers (blockquote, list)
            if self.container_stack.len() < limits::MAX_BLOCK_NESTING {
                // Check for blockquote
                if self.try_blockquote(events) {
                    // Recursively parse the rest of the line
                    self.parse_line_content(events);
                    return;
                }

                // Check for list item
                if self.try_list_item(events) {
                    self.parse_line_content(events);
                    return;
                }
            }

            // Check for fenced code block
            if self.try_code_fence(indent, events) {
                return;
            }

            // Check for ATX heading
            if self.try_atx_heading(events) {
                return;
            }
        }

        // Check for indented code block (4+ spaces, not in paragraph)
        if indent >= 4 && !self.in_paragraph {
            self.start_indented_code(indent, events);
            return;
        }

        // Otherwise, it's paragraph content
        let line_start = self.cursor.offset();
        self.parse_paragraph_line(line_start, events);
    }

    /// Try to match existing containers at line start.
    /// Returns number of matched containers.
    fn match_containers(&mut self, _events: &mut Vec<BlockEvent>) -> usize {
        let mut matched = 0;

        for i in 0..self.container_stack.len() {
            let container = &self.container_stack[i];
            match container.typ {
                ContainerType::BlockQuote => {
                    // Try to match `>` marker with up to 3 leading spaces
                    let spaces = self.cursor.skip_spaces();
                    if spaces <= 3 && self.cursor.at(b'>') {
                        self.cursor.bump();
                        // Optional space after >
                        if self.cursor.at(b' ') {
                            self.cursor.bump();
                        }
                        matched += 1;
                    } else {
                        // Can't continue blockquote, break (don't close yet)
                        break;
                    }
                }
                ContainerType::ListItem { content_indent, kind, marker } => {
                    // Check if line is blank (after any spaces we've consumed so far)
                    let remaining = self.cursor.remaining_slice();
                    let is_blank = remaining.is_empty() || remaining[0] == b'\n';

                    if is_blank {
                        // Blank lines always match list items
                        matched += 1;
                    } else {
                        // Save position to check indent
                        let save_pos = self.cursor.offset();
                        let spaces = self.cursor.skip_spaces();

                        if spaces >= content_indent {
                            // Enough indent to continue the list item
                            // If we had a blank line, list becomes loose
                            if let Some(open_list) = self.open_lists.last_mut() {
                                if open_list.blank_in_item {
                                    open_list.tight = false;
                                }
                            }
                            matched += 1;
                        } else {
                            // Not enough indent - check if it's a new list item of same type
                            self.cursor = Cursor::new_at(self.input, save_pos + spaces);
                            let is_same_list = self.peek_list_marker(kind, marker);

                            // Reset cursor to saved position
                            self.cursor = Cursor::new_at(self.input, save_pos);

                            if is_same_list {
                                // This will start a new item in the same list
                                // Don't match this container
                                break;
                            } else {
                                // Different content, don't match
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Don't close containers here - let parse_line handle it
        matched
    }

    /// Peek ahead to see if there's a list marker of the same type.
    fn peek_list_marker(&self, kind: ListKind, marker: u8) -> bool {
        let b = match self.cursor.peek() {
            Some(b) => b,
            None => return false,
        };

        match kind {
            ListKind::Unordered => {
                // Must be the SAME marker character (-, *, or +)
                b == marker && self.cursor.peek_ahead(1) == Some(b' ')
            }
            ListKind::Ordered { delimiter, .. } => {
                // Must be digit(s) followed by the SAME delimiter (. or ))
                if !b.is_ascii_digit() {
                    return false;
                }
                // Find the delimiter after the digits
                let mut offset = 1;
                while self.cursor.peek_ahead(offset).map_or(false, |b| b.is_ascii_digit()) {
                    offset += 1;
                }
                // Check if delimiter matches
                self.cursor.peek_ahead(offset) == Some(delimiter)
            }
        }
    }

    /// Check if we can do lazy continuation for a blockquote.
    /// Returns true if:
    /// 1. We're in a paragraph
    /// 2. The first unmatched container is a blockquote
    /// 3. The current line doesn't start a new block
    fn can_lazy_continue(&self, matched: usize, indent: usize) -> bool {
        // Must be in a paragraph to do lazy continuation
        if !self.in_paragraph {
            return false;
        }

        // Must have unmatched containers
        if matched >= self.container_stack.len() {
            return false;
        }

        // The first unmatched container must be a blockquote
        // (lazy continuation doesn't apply to list items)
        if self.container_stack[matched].typ != ContainerType::BlockQuote {
            return false;
        }

        // If indent >= 4, this would be an indented code block
        if indent >= 4 {
            return false;
        }

        // Check if the current line would start a new block
        // If it would, we can't do lazy continuation
        !self.would_start_block()
    }

    /// Check if the current position would start a new block.
    /// Used for lazy continuation checks.
    fn would_start_block(&self) -> bool {
        let b = match self.cursor.peek() {
            Some(b) => b,
            None => return false,
        };

        match b {
            // ATX heading
            b'#' => true,
            // Fenced code block or thematic break
            b'`' | b'~' => self.cursor.remaining_slice().iter().take_while(|&&c| c == b).count() >= 3,
            // Blockquote
            b'>' => true,
            // Unordered list marker or thematic break
            b'-' | b'*' | b'+' => {
                // Check if followed by space (list item) or if it's a thematic break
                self.cursor.peek_ahead(1) == Some(b' ') || self.peek_thematic_break()
            }
            // Ordered list marker
            b'0'..=b'9' => {
                // Check if digit(s) followed by . or ) then space
                let mut offset = 1;
                while self.cursor.peek_ahead(offset).map_or(false, |c| c.is_ascii_digit()) {
                    offset += 1;
                }
                let delim = self.cursor.peek_ahead(offset);
                (delim == Some(b'.') || delim == Some(b')'))
                    && self.cursor.peek_ahead(offset + 1) == Some(b' ')
            }
            // HTML block (simplified check)
            b'<' => true,
            // Setext heading underline
            b'=' => self.cursor.remaining_slice().iter().all(|&c| c == b'=' || c == b' ' || c == b'\n'),
            // Blank or other content - not a block start
            _ => false,
        }
    }

    /// Close containers starting from index, being smart about lists.
    fn close_containers_from(&mut self, from: usize, events: &mut Vec<BlockEvent>) {
        // Check if we're about to close a list item but might start a new one
        while self.container_stack.len() > from {
            let top = self.container_stack.last().unwrap();

            if let ContainerType::ListItem { kind, marker, .. } = top.typ {
                // Check if the current position has a same-type list marker
                let save_pos = self.cursor.offset();
                self.cursor.skip_spaces();
                let is_same_list = self.peek_list_marker(kind, marker);
                self.cursor = Cursor::new_at(self.input, save_pos);

                if is_same_list {
                    // Just close the item, not the list
                    self.container_stack.pop();
                    self.close_paragraph(events);
                    events.push(BlockEvent::ListItemEnd);
                    // Don't pop from open_lists
                    continue;
                }
            }

            self.close_top_container(events);
        }
    }

    /// Handle blank line for container continuation.
    fn handle_blank_line_containers(&mut self, _events: &mut Vec<BlockEvent>) {
        // Mark any open lists as having seen a blank line in current item
        for open_list in self.open_lists.iter_mut() {
            open_list.blank_in_item = true;
        }
    }

    /// Try to start a blockquote.
    fn try_blockquote(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        if !self.cursor.at(b'>') {
            return false;
        }

        self.cursor.bump(); // consume >

        // Optional space after >
        if self.cursor.at(b' ') {
            self.cursor.bump();
        }

        // Close paragraph if any
        self.close_paragraph(events);

        // Push blockquote container
        self.container_stack.push(Container {
            typ: ContainerType::BlockQuote,
            has_content: false,
        });

        events.push(BlockEvent::BlockQuoteStart);
        true
    }

    /// Try to start a list item.
    fn try_list_item(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        let start_offset = self.cursor.offset();

        // Check for unordered list marker (-, *, +)
        if let Some(marker) = self.try_unordered_marker() {
            self.start_list_item(ListKind::Unordered, marker, start_offset, events);
            return true;
        }

        // Check for ordered list marker (1. 2. etc)
        if let Some((start_num, _marker_len, delimiter)) = self.try_ordered_marker() {
            // CommonMark: an ordered list can only interrupt a paragraph if it starts with 1
            if self.in_paragraph && start_num != 1 {
                // Reset cursor and don't start list
                self.cursor = Cursor::new_at(self.input, start_offset);
                return false;
            }
            self.start_list_item(
                ListKind::Ordered { start: start_num, delimiter },
                delimiter,
                start_offset,
                events,
            );
            return true;
        }

        false
    }

    /// Try to parse an unordered list marker (-, *, +).
    fn try_unordered_marker(&mut self) -> Option<u8> {
        let marker = self.cursor.peek()?;
        if marker != b'-' && marker != b'*' && marker != b'+' {
            return None;
        }

        // Must be followed by space or tab
        if self.cursor.peek_ahead(1) != Some(b' ') && self.cursor.peek_ahead(1) != Some(b'\t') {
            // Could be thematic break for - and *
            return None;
        }

        self.cursor.bump(); // consume marker
        self.cursor.bump(); // consume space

        Some(marker)
    }

    /// Try to parse an ordered list marker (1. 2. etc).
    /// Returns (number, marker_len, delimiter) where delimiter is '.' or ')'.
    fn try_ordered_marker(&mut self) -> Option<(u32, usize, u8)> {
        let start = self.cursor.offset();
        let mut num: u32 = 0;
        let mut digits = 0;

        // Parse digits
        while let Some(b) = self.cursor.peek() {
            if b.is_ascii_digit() {
                if digits >= limits::MAX_LIST_MARKER_DIGITS {
                    // Too many digits, reset and return
                    self.cursor = Cursor::new_at(self.input, start);
                    return None;
                }
                num = num * 10 + (b - b'0') as u32;
                digits += 1;
                self.cursor.bump();
            } else {
                break;
            }
        }

        if digits == 0 {
            return None;
        }

        // Must be followed by . or )
        let delimiter = match self.cursor.peek() {
            Some(b'.') => b'.',
            Some(b')') => b')',
            _ => {
                self.cursor = Cursor::new_at(self.input, start);
                return None;
            }
        };
        self.cursor.bump(); // consume . or )

        // Must be followed by space
        if !self.cursor.at(b' ') && !self.cursor.at(b'\t') {
            self.cursor = Cursor::new_at(self.input, start);
            return None;
        }
        self.cursor.bump(); // consume space

        Some((num, digits + 2, delimiter)) // digits + delimiter + space
    }

    /// Start a new list item.
    fn start_list_item(
        &mut self,
        kind: ListKind,
        marker: u8,
        start_offset: usize,
        events: &mut Vec<BlockEvent>,
    ) {
        // Close paragraph if any
        self.close_paragraph(events);

        // Check if we're continuing an existing list of the same type
        let continuing_list = self.is_compatible_list(kind, marker);

        if !continuing_list {
            // Close any existing list items from incompatible lists
            while let Some(container) = self.container_stack.last() {
                if matches!(container.typ, ContainerType::ListItem { .. }) {
                    self.close_top_container(events);
                } else {
                    break;
                }
            }

            // Start new list (tight will be determined later)
            events.push(BlockEvent::ListStart { kind, tight: true });
            self.open_lists.push(OpenList {
                kind,
                marker,
                tight: true,
                blank_in_item: false,
                item_count: 0,
            });
        }
        // Note: if continuing_list is true, the previous item was already
        // closed by close_containers_from, so we just add the new item

        // Calculate content indent
        let content_indent = self.cursor.offset() - start_offset;

        // Push list item container
        self.container_stack.push(Container {
            typ: ContainerType::ListItem {
                kind,
                marker,
                content_indent,
            },
            has_content: false,
        });

        // Track item count and blank line status for tight/loose detection
        if let Some(open_list) = self.open_lists.last_mut() {
            // If we've seen a blank line since the previous item, list becomes loose
            if open_list.item_count > 0 && open_list.blank_in_item {
                open_list.tight = false;
            }
            open_list.item_count += 1;
            open_list.blank_in_item = false;
        }

        // Check for task list checkbox
        let task = self.try_task_checkbox();

        events.push(BlockEvent::ListItemStart { task });
    }

    /// Check if we're continuing a compatible list.
    fn is_compatible_list(&self, kind: ListKind, marker: u8) -> bool {
        // Check open_lists for a compatible list
        if let Some(open_list) = self.open_lists.last() {
            return match (kind, open_list.kind) {
                // For ordered lists, delimiter (. vs )) must match
                (ListKind::Ordered { delimiter: d1, .. }, ListKind::Ordered { delimiter: d2, .. }) => d1 == d2,
                // For unordered lists, marker (-, *, +) must match
                (ListKind::Unordered, ListKind::Unordered) => open_list.marker == marker,
                _ => false,
            };
        }
        false
    }

    /// Try to parse a task list checkbox.
    fn try_task_checkbox(&mut self) -> TaskState {
        if !self.cursor.at(b'[') {
            return TaskState::None;
        }

        let checkbox_char = self.cursor.peek_ahead(1);
        if self.cursor.peek_ahead(2) != Some(b']') {
            return TaskState::None;
        }

        // Must be followed by space
        if self.cursor.peek_ahead(3) != Some(b' ') {
            return TaskState::None;
        }

        let state = match checkbox_char {
            Some(b' ') => TaskState::Unchecked,
            Some(b'x') | Some(b'X') => TaskState::Checked,
            _ => return TaskState::None,
        };

        // Consume checkbox
        self.cursor.advance(4);
        state
    }

    /// Close the topmost container.
    fn close_top_container(&mut self, events: &mut Vec<BlockEvent>) {
        if let Some(container) = self.container_stack.pop() {
            // Close paragraph first
            self.close_paragraph(events);

            match container.typ {
                ContainerType::BlockQuote => {
                    events.push(BlockEvent::BlockQuoteEnd);
                }
                ContainerType::ListItem { kind, .. } => {
                    events.push(BlockEvent::ListItemEnd);

                    // Check if this was the last item in the list
                    let has_more_items = self.container_stack.iter().any(|c| {
                        matches!(c.typ, ContainerType::ListItem { .. })
                    });

                    if !has_more_items {
                        // Close the list and remove from open_lists
                        let tight = self.open_lists.last().map_or(true, |l| l.tight);
                        events.push(BlockEvent::ListEnd { kind, tight });
                        self.open_lists.pop();
                    }
                }
            }
        }
    }

    /// Close all containers.
    fn close_all_containers(&mut self, events: &mut Vec<BlockEvent>) {
        while !self.container_stack.is_empty() {
            self.close_top_container(events);
        }
    }

    /// Check if the current line is a thematic break without consuming input.
    fn peek_thematic_break(&self) -> bool {
        // Must start with -, *, or _
        let marker = match self.cursor.peek() {
            Some(b'-') | Some(b'*') | Some(b'_') => self.cursor.peek().unwrap(),
            _ => return false,
        };

        // Count markers and spaces
        let mut count = 0;
        let mut temp_cursor = self.cursor;

        while let Some(b) = temp_cursor.peek() {
            if b == marker {
                count += 1;
                temp_cursor.bump();
            } else if b == b' ' || b == b'\t' {
                temp_cursor.bump();
            } else if b == b'\n' {
                break;
            } else {
                // Invalid character
                return false;
            }
        }

        // Need at least 3 markers
        count >= 3
    }

    /// Try to parse a thematic break.
    /// Returns true if successful.
    fn try_thematic_break(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        let _start_pos = self.cursor.offset();

        // Must start with -, *, or _
        let marker = match self.cursor.peek() {
            Some(b'-') | Some(b'*') | Some(b'_') => self.cursor.peek().unwrap(),
            _ => return false,
        };

        // Count markers and spaces
        let mut count = 0;
        let mut temp_cursor = self.cursor;

        while let Some(b) = temp_cursor.peek() {
            if b == marker {
                count += 1;
                temp_cursor.bump();
            } else if b == b' ' || b == b'\t' {
                temp_cursor.bump();
            } else if b == b'\n' {
                break;
            } else {
                // Invalid character
                return false;
            }
        }

        // Need at least 3 markers
        if count < 3 {
            return false;
        }

        // Consume the line
        self.cursor = temp_cursor;
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // Close any open paragraph
        self.close_paragraph(events);

        events.push(BlockEvent::ThematicBreak);
        true
    }

    /// Try to parse an ATX heading.
    /// Returns true if successful.
    fn try_atx_heading(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        // Must start with #
        if !self.cursor.at(b'#') {
            return false;
        }

        let _start_pos = self.cursor.offset();

        // Count # characters (1-6)
        let mut level = 0u8;
        let mut temp_cursor = self.cursor;

        while temp_cursor.at(b'#') && level < 7 {
            level += 1;
            temp_cursor.bump();
        }

        // Level must be 1-6
        if level == 0 || level > 6 {
            return false;
        }

        // Must be followed by space, tab, or end of line
        if !temp_cursor.is_eof()
            && !temp_cursor.at(b' ')
            && !temp_cursor.at(b'\t')
            && !temp_cursor.at(b'\n')
        {
            return false;
        }

        // Skip spaces after #
        temp_cursor.skip_whitespace();

        // Find content start and end
        let content_start = temp_cursor.offset();

        // Find end of line
        let line_end = match temp_cursor.find_newline() {
            Some(pos) => content_start + pos,
            None => content_start + temp_cursor.remaining(),
        };

        // Trim trailing # and spaces from content
        let content_end = self.trim_heading_end(content_start, line_end);

        // Update cursor to end of line
        self.cursor = Cursor::new_at(self.input, line_end);
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // Close any open paragraph
        self.close_paragraph(events);

        // Emit heading events
        events.push(BlockEvent::HeadingStart { level });

        if content_end > content_start {
            events.push(BlockEvent::Text(Range::from_usize(content_start, content_end)));
        }

        events.push(BlockEvent::HeadingEnd { level });

        true
    }

    /// Trim trailing # characters and spaces from heading content.
    fn trim_heading_end(&self, start: usize, end: usize) -> usize {
        if start >= end {
            return start;
        }

        let mut pos = end;

        // Trim trailing spaces
        while pos > start && (self.input[pos - 1] == b' ' || self.input[pos - 1] == b'\t') {
            pos -= 1;
        }

        // Check for closing # sequence
        let after_hashes = pos;
        while pos > start && self.input[pos - 1] == b'#' {
            pos -= 1;
        }

        // Closing hashes must be preceded by space (or be at start)
        if pos < after_hashes {
            if pos == start || self.input[pos - 1] == b' ' || self.input[pos - 1] == b'\t' {
                // Valid closing hashes, trim them and any preceding space
                while pos > start && (self.input[pos - 1] == b' ' || self.input[pos - 1] == b'\t') {
                    pos -= 1;
                }
            } else {
                // Hashes not preceded by space, keep them
                pos = after_hashes;
            }
        }

        pos
    }

    /// Try to parse a fenced code block opening.
    /// Returns true if successful.
    fn try_code_fence(&mut self, indent: usize, events: &mut Vec<BlockEvent>) -> bool {
        // Must start with ` or ~
        let fence_char = match self.cursor.peek() {
            Some(b'`') | Some(b'~') => self.cursor.peek().unwrap(),
            _ => return false,
        };

        // Count fence characters (need at least 3)
        let mut fence_len = 0;
        let mut temp_cursor = self.cursor;

        while temp_cursor.at(fence_char) {
            fence_len += 1;
            temp_cursor.bump();
        }

        if fence_len < 3 {
            return false;
        }

        // For backtick fences, info string cannot contain backticks
        let _info_start = temp_cursor.offset();

        // Skip optional spaces before info string
        temp_cursor.skip_whitespace();
        let info_content_start = temp_cursor.offset();

        // Find end of line
        let line_end = match temp_cursor.find_newline() {
            Some(pos) => info_content_start + pos,
            None => info_content_start + temp_cursor.remaining(),
        };

        // Check for backticks in info string (invalid for backtick fences)
        if fence_char == b'`' {
            let info_slice = &self.input[info_content_start..line_end];
            if info_slice.contains(&b'`') {
                return false;
            }
        }

        // Trim trailing whitespace from info string
        let mut info_end = line_end;
        while info_end > info_content_start
            && (self.input[info_end - 1] == b' ' || self.input[info_end - 1] == b'\t')
        {
            info_end -= 1;
        }

        // Move cursor past the line
        self.cursor = Cursor::new_at(self.input, line_end);
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // Close any open paragraph
        self.close_paragraph(events);

        // Store fence state
        self.fence_state = Some(FenceState {
            fence_char,
            fence_len,
            indent,
        });

        // Emit code block start with info string
        let info = if info_end > info_content_start {
            Some(Range::from_usize(info_content_start, info_end))
        } else {
            None
        };
        events.push(BlockEvent::CodeBlockStart { info });

        true
    }

    /// Start an indented code block.
    fn start_indented_code(&mut self, indent: usize, events: &mut Vec<BlockEvent>) {
        // Close any open paragraph first
        self.close_paragraph(events);

        // Start the code block
        self.in_indented_code = true;
        events.push(BlockEvent::CodeBlockStart { info: None });

        // The cursor is past `indent` spaces. We want to strip exactly 4.
        // The content starts at cursor position (we already skipped `indent` spaces).
        // We need to "give back" (indent - 4) spaces if indent > 4.
        let extra_spaces = indent.saturating_sub(4);
        let text_start = self.cursor.offset() - extra_spaces;

        // Find end of line (including newline for code blocks)
        let line_end = self.find_line_end();
        let content_end = if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
            line_end + 1 // Include the newline
        } else {
            line_end
        };

        // Emit the content (including newline)
        events.push(BlockEvent::Text(Range::new(text_start as u32, content_end as u32)));
    }

    /// Parse a line inside an indented code block.
    fn parse_indented_code_line(&mut self, events: &mut Vec<BlockEvent>) {
        let line_start = self.cursor.offset();

        // Count leading spaces
        let initial_spaces = self.count_leading_spaces();

        // Check for blank line
        if self.is_blank_after(initial_spaces) {
            // Blank lines in indented code are preserved - emit just the newline
            self.cursor = Cursor::new_at(self.input, line_start + initial_spaces);
            let newline_pos = self.cursor.offset();
            if !self.cursor.is_eof() && self.cursor.at(b'\n') {
                self.cursor.bump();
                // Emit just the newline
                events.push(BlockEvent::Text(Range::new(newline_pos as u32, (newline_pos + 1) as u32)));
            }
            return;
        }

        // If indent < 4, this ends the indented code block
        if initial_spaces < 4 {
            // Close the code block
            self.in_indented_code = false;
            events.push(BlockEvent::CodeBlockEnd);

            // Re-parse this line as a new block
            self.cursor = Cursor::new_at(self.input, line_start);
            self.parse_line(events);
            return;
        }

        // Continue the code block - strip exactly 4 spaces
        self.cursor = Cursor::new_at(self.input, line_start + 4);
        let content_start = self.cursor.offset();
        let line_end = self.find_line_end();

        // Include the newline in the content
        let content_end = if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
            line_end + 1
        } else {
            line_end
        };

        events.push(BlockEvent::Text(Range::new(content_start as u32, content_end as u32)));
    }

    /// Find end of current line (position of \n or EOF).
    fn find_line_end(&mut self) -> usize {
        while !self.cursor.is_eof() && !self.cursor.at(b'\n') {
            self.cursor.bump();
        }
        self.cursor.offset()
    }

    /// Parse a line inside a fenced code block.
    fn parse_fence_line(&mut self, events: &mut Vec<BlockEvent>) {
        let fence = self.fence_state.as_ref().unwrap();
        let fence_char = fence.fence_char;
        let fence_len = fence.fence_len;
        let fence_indent = fence.indent;

        let line_start = self.cursor.offset();

        // Skip up to fence_indent spaces
        let mut spaces = 0;
        while spaces < fence_indent && self.cursor.at(b' ') {
            self.cursor.bump();
            spaces += 1;
        }

        // Check for closing fence
        if self.cursor.at(fence_char) {
            let mut closing_len = 0;
            let mut temp_cursor = self.cursor;

            while temp_cursor.at(fence_char) {
                closing_len += 1;
                temp_cursor.bump();
            }

            // Closing fence must be at least as long as opening
            if closing_len >= fence_len {
                // Check that rest of line is only spaces
                temp_cursor.skip_whitespace();
                if temp_cursor.is_eof() || temp_cursor.at(b'\n') {
                    // Valid closing fence
                    self.cursor = temp_cursor;
                    if !self.cursor.is_eof() && self.cursor.at(b'\n') {
                        self.cursor.bump();
                    }

                    self.fence_state = None;
                    events.push(BlockEvent::CodeBlockEnd);
                    return;
                }
            }
        }

        // Not a closing fence, emit as code content
        // Reset to line start and capture the whole line
        self.cursor = Cursor::new_at(self.input, line_start);

        // Skip up to fence_indent spaces for content
        let mut spaces = 0;
        while spaces < fence_indent && self.cursor.at(b' ') {
            self.cursor.bump();
            spaces += 1;
        }

        let content_start = self.cursor.offset();

        // Find end of line
        let line_end = match self.cursor.find_newline() {
            Some(pos) => content_start + pos,
            None => content_start + self.cursor.remaining(),
        };

        // Include the newline in the code content range
        let content_end = if line_end < self.input.len() && self.input[line_end] == b'\n' {
            line_end + 1
        } else {
            line_end
        };

        // Move cursor past the newline
        self.cursor = Cursor::new_at(self.input, content_end);

        // Emit the code line (including newline)
        events.push(BlockEvent::Code(Range::from_usize(content_start, content_end)));
    }

    /// Parse a paragraph line.
    fn parse_paragraph_line(&mut self, _line_start: usize, _events: &mut Vec<BlockEvent>) {
        // Find end of line
        let content_start = self.cursor.offset();
        let line_end = match self.cursor.find_newline() {
            Some(pos) => content_start + pos,
            None => content_start + self.cursor.remaining(),
        };

        // Move cursor to next line
        self.cursor = Cursor::new_at(self.input, line_end);
        if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
        }

        // If we weren't in a paragraph, we are now
        if !self.in_paragraph {
            self.in_paragraph = true;
        }

        // Add this line to paragraph content
        // We include from original line_start to capture any leading spaces we skipped
        // Actually, use content_start which is after indent
        if line_end > content_start {
            self.paragraph_lines.push(Range::from_usize(content_start, line_end));
        }
    }

    /// Close an open paragraph.
    fn close_paragraph(&mut self, events: &mut Vec<BlockEvent>) {
        if !self.in_paragraph {
            return;
        }

        self.in_paragraph = false;

        if self.paragraph_lines.is_empty() {
            return;
        }

        events.push(BlockEvent::ParagraphStart);

        // Emit text ranges for each line with soft breaks between
        for (i, range) in self.paragraph_lines.drain(..).enumerate() {
            if i > 0 {
                // Add soft break between lines
                events.push(BlockEvent::SoftBreak);
            }
            events.push(BlockEvent::Text(range));
        }

        events.push(BlockEvent::ParagraphEnd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Vec<BlockEvent> {
        let mut parser = BlockParser::new(input.as_bytes());
        let mut events = Vec::new();
        parser.parse(&mut events);
        events
    }

    fn get_text<'a>(input: &'a str, event: &BlockEvent) -> &'a str {
        match event {
            BlockEvent::Text(range) => {
                std::str::from_utf8(range.slice(input.as_bytes())).unwrap()
            }
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_empty_input() {
        let events = parse("");
        assert!(events.is_empty());
    }

    #[test]
    fn test_blank_lines() {
        let events = parse("\n\n\n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello, world!";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Hello, world!");
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
    }

    #[test]
    fn test_multiline_paragraph() {
        let input = "Line 1\nLine 2\nLine 3";
        let events = parse(input);

        // ParagraphStart, Text, SoftBreak, Text, SoftBreak, Text, ParagraphEnd
        assert_eq!(events.len(), 7);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Line 1");
        assert_eq!(events[2], BlockEvent::SoftBreak);
        assert_eq!(get_text(input, &events[3]), "Line 2");
        assert_eq!(events[4], BlockEvent::SoftBreak);
        assert_eq!(get_text(input, &events[5]), "Line 3");
        assert_eq!(events[6], BlockEvent::ParagraphEnd);
    }

    #[test]
    fn test_paragraphs_separated_by_blank() {
        let input = "Para 1\n\nPara 2";
        let events = parse(input);

        assert_eq!(events.len(), 6);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Para 1");
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert_eq!(events[3], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[4]), "Para 2");
        assert_eq!(events[5], BlockEvent::ParagraphEnd);
    }

    #[test]
    fn test_thematic_break_dashes() {
        let events = parse("---");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_asterisks() {
        let events = parse("***");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_underscores() {
        let events = parse("___");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_with_spaces() {
        let events = parse("- - -");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_many() {
        let events = parse("----------");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    #[test]
    fn test_thematic_break_too_few() {
        let input = "--";
        let events = parse(input);
        // Should be a paragraph, not a thematic break
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_thematic_break_mixed_invalid() {
        let input = "-*-";
        let events = parse(input);
        // Mixed markers = paragraph
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_atx_heading_h1() {
        let input = "# Heading";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], BlockEvent::HeadingStart { level: 1 });
        assert_eq!(get_text(input, &events[1]), "Heading");
        assert_eq!(events[2], BlockEvent::HeadingEnd { level: 1 });
    }

    #[test]
    fn test_atx_heading_h2() {
        let input = "## Heading";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::HeadingStart { level: 2 });
    }

    #[test]
    fn test_atx_heading_h6() {
        let input = "###### Heading";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::HeadingStart { level: 6 });
    }

    #[test]
    fn test_atx_heading_h7_invalid() {
        let input = "####### Heading";
        let events = parse(input);

        // 7 # is not a valid heading
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_atx_heading_empty() {
        let input = "#";
        let events = parse(input);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], BlockEvent::HeadingStart { level: 1 });
        assert_eq!(events[1], BlockEvent::HeadingEnd { level: 1 });
    }

    #[test]
    fn test_atx_heading_closing_hashes() {
        let input = "# Heading #";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(get_text(input, &events[1]), "Heading");
    }

    #[test]
    fn test_atx_heading_closing_hashes_multiple() {
        let input = "## Heading ##";
        let events = parse(input);

        assert_eq!(get_text(input, &events[1]), "Heading");
    }

    #[test]
    fn test_atx_heading_closing_hashes_with_space() {
        let input = "# Heading #  ";
        let events = parse(input);

        assert_eq!(get_text(input, &events[1]), "Heading");
    }

    #[test]
    fn test_atx_heading_no_space_after_hashes() {
        let input = "#Heading";
        let events = parse(input);

        // No space after # = not a heading
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_thematic_break_closes_paragraph() {
        let input = "Text\n---";
        let events = parse(input);

        assert_eq!(events.len(), 4);
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert_eq!(events[3], BlockEvent::ThematicBreak);
    }

    #[test]
    fn test_heading_closes_paragraph() {
        let input = "Text\n# Heading";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert_eq!(events[3], BlockEvent::HeadingStart { level: 1 });
    }

    #[test]
    fn test_indented_content() {
        let input = "   Text with indent";
        let events = parse(input);

        // Up to 3 spaces is allowed for normal blocks
        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[1]), "Text with indent");
    }

    #[test]
    fn test_thematic_break_with_leading_spaces() {
        let events = parse("   ---");
        assert_eq!(events, vec![BlockEvent::ThematicBreak]);
    }

    // Fenced code block tests

    fn get_code<'a>(input: &'a str, event: &BlockEvent) -> &'a str {
        match event {
            BlockEvent::Code(range) => {
                std::str::from_utf8(range.slice(input.as_bytes())).unwrap()
            }
            _ => panic!("Expected Code event"),
        }
    }

    fn get_info<'a>(input: &'a str, event: &BlockEvent) -> Option<&'a str> {
        match event {
            BlockEvent::CodeBlockStart { info } => {
                info.as_ref().map(|r| std::str::from_utf8(r.slice(input.as_bytes())).unwrap())
            }
            _ => panic!("Expected CodeBlockStart event"),
        }
    }

    #[test]
    fn test_code_fence_backticks() {
        let input = "```\ncode\n```";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], BlockEvent::CodeBlockStart { .. }));
        assert_eq!(get_code(input, &events[1]), "code\n");
        assert_eq!(events[2], BlockEvent::CodeBlockEnd);
    }

    #[test]
    fn test_code_fence_tildes() {
        let input = "~~~\ncode\n~~~";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], BlockEvent::CodeBlockStart { .. }));
        assert_eq!(get_code(input, &events[1]), "code\n");
        assert_eq!(events[2], BlockEvent::CodeBlockEnd);
    }

    #[test]
    fn test_code_fence_with_info() {
        let input = "```rust\nfn main() {}\n```";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(get_info(input, &events[0]), Some("rust"));
        assert_eq!(get_code(input, &events[1]), "fn main() {}\n");
    }

    #[test]
    fn test_code_fence_info_with_spaces() {
        let input = "```rust cargo\ncode\n```";
        let events = parse(input);

        assert_eq!(get_info(input, &events[0]), Some("rust cargo"));
    }

    #[test]
    fn test_code_fence_longer_closing() {
        let input = "```\ncode\n`````";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(events[2], BlockEvent::CodeBlockEnd);
    }

    #[test]
    fn test_code_fence_shorter_closing_invalid() {
        let input = "````\ncode\n```";
        let events = parse(input);

        // Should not close, code continues and fence closes at EOF
        assert_eq!(events.len(), 4); // start, code, "```", end
    }

    #[test]
    fn test_code_fence_empty() {
        let input = "```\n```";
        let events = parse(input);

        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], BlockEvent::CodeBlockStart { .. }));
        assert_eq!(events[1], BlockEvent::CodeBlockEnd);
    }

    #[test]
    fn test_code_fence_multiline() {
        let input = "```\nline1\nline2\nline3\n```";
        let events = parse(input);

        assert_eq!(events.len(), 5);
        assert_eq!(get_code(input, &events[1]), "line1\n");
        assert_eq!(get_code(input, &events[2]), "line2\n");
        assert_eq!(get_code(input, &events[3]), "line3\n");
    }

    #[test]
    fn test_code_fence_no_closing() {
        let input = "```\ncode";
        let events = parse(input);

        // Code block should be closed at EOF (no trailing newline)
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], BlockEvent::CodeBlockStart { .. }));
        assert_eq!(get_code(input, &events[1]), "code");  // No newline at EOF
        assert_eq!(events[2], BlockEvent::CodeBlockEnd);
    }

    #[test]
    fn test_code_fence_with_blank_lines() {
        let input = "```\n\ncode\n\n```";
        let events = parse(input);

        assert_eq!(events.len(), 5);
        assert_eq!(get_code(input, &events[1]), "\n");
        assert_eq!(get_code(input, &events[2]), "code\n");
        assert_eq!(get_code(input, &events[3]), "\n");
    }

    #[test]
    fn test_code_fence_backticks_in_tilde_fence() {
        let input = "~~~\n```\n~~~";
        let events = parse(input);

        assert_eq!(events.len(), 3);
        assert_eq!(get_code(input, &events[1]), "```\n");
    }

    #[test]
    fn test_code_fence_closes_paragraph() {
        let input = "text\n```\ncode\n```";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::ParagraphStart);
        assert_eq!(events[2], BlockEvent::ParagraphEnd);
        assert!(matches!(events[3], BlockEvent::CodeBlockStart { .. }));
    }

    #[test]
    fn test_code_fence_two_backticks_invalid() {
        let input = "``\ncode\n``";
        let events = parse(input);

        // Two backticks is not a valid fence
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_code_fence_backtick_in_info_invalid() {
        let input = "```rust`extra\ncode\n```";
        let events = parse(input);

        // Backtick in info string makes it not a code fence
        assert_eq!(events[0], BlockEvent::ParagraphStart);
    }

    #[test]
    fn test_code_fence_preserves_content() {
        let input = "```\n  indented\n    more\n```";
        let events = parse(input);

        assert_eq!(get_code(input, &events[1]), "  indented\n");
        assert_eq!(get_code(input, &events[2]), "    more\n");
    }

    // Blockquote tests

    #[test]
    fn test_blockquote_simple() {
        let input = "> quote";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::BlockQuoteStart);
        assert_eq!(events[1], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[2]), "quote");
        assert_eq!(events[3], BlockEvent::ParagraphEnd);
        assert_eq!(events[4], BlockEvent::BlockQuoteEnd);
    }

    #[test]
    fn test_blockquote_multiline() {
        let input = "> line1\n> line2";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::BlockQuoteStart);
        assert_eq!(events[1], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[2]), "line1");
        assert_eq!(events[3], BlockEvent::SoftBreak);
        assert_eq!(get_text(input, &events[4]), "line2");
    }

    #[test]
    fn test_blockquote_no_space() {
        let input = ">quote";
        let events = parse(input);

        // > without space is still valid
        assert_eq!(events[0], BlockEvent::BlockQuoteStart);
    }

    #[test]
    fn test_blockquote_nested() {
        let input = "> > nested";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::BlockQuoteStart);
        assert_eq!(events[1], BlockEvent::BlockQuoteStart);
        assert!(matches!(events[2], BlockEvent::ParagraphStart));
    }

    #[test]
    fn test_blockquote_ends() {
        let input = "> quote\n\nparagraph";
        let events = parse(input);

        // Blockquote ends on blank line
        let mut found_quote_end = false;
        let mut found_para_after = false;
        for event in events.iter() {
            if *event == BlockEvent::BlockQuoteEnd {
                found_quote_end = true;
            }
            if found_quote_end && *event == BlockEvent::ParagraphStart {
                found_para_after = true;
            }
        }
        assert!(found_quote_end);
        assert!(found_para_after);
    }

    // List tests

    #[test]
    fn test_list_unordered_dash() {
        let input = "- item";
        let events = parse(input);

        assert!(matches!(events[0], BlockEvent::ListStart { kind: ListKind::Unordered, .. }));
        assert!(matches!(events[1], BlockEvent::ListItemStart { .. }));
        assert_eq!(events[2], BlockEvent::ParagraphStart);
        assert_eq!(get_text(input, &events[3]), "item");
    }

    #[test]
    fn test_list_unordered_asterisk() {
        let input = "* item";
        let events = parse(input);

        assert!(matches!(events[0], BlockEvent::ListStart { kind: ListKind::Unordered, .. }));
    }

    #[test]
    fn test_list_unordered_plus() {
        let input = "+ item";
        let events = parse(input);

        assert!(matches!(events[0], BlockEvent::ListStart { kind: ListKind::Unordered, .. }));
    }

    #[test]
    fn test_list_multiple_items() {
        let input = "- item1\n- item2\n- item3";
        let events = parse(input);

        // Count list item starts
        let item_count = events.iter()
            .filter(|e| matches!(e, BlockEvent::ListItemStart { .. }))
            .count();
        assert_eq!(item_count, 3);

        // Should have exactly one list
        let list_count = events.iter()
            .filter(|e| matches!(e, BlockEvent::ListStart { .. }))
            .count();
        assert_eq!(list_count, 1);
    }

    #[test]
    fn test_list_ordered() {
        let input = "1. first\n2. second";
        let events = parse(input);

        assert!(matches!(events[0], BlockEvent::ListStart { kind: ListKind::Ordered { start: 1, .. }, .. }));
    }

    #[test]
    fn test_list_ordered_start_number() {
        let input = "5. fifth";
        let events = parse(input);

        assert!(matches!(events[0], BlockEvent::ListStart { kind: ListKind::Ordered { start: 5, .. }, .. }));
    }

    #[test]
    fn test_list_ordered_paren() {
        let input = "1) item";
        let events = parse(input);

        assert!(matches!(events[0], BlockEvent::ListStart { kind: ListKind::Ordered { .. }, .. }));
    }

    #[test]
    fn test_list_task_unchecked() {
        let input = "- [ ] task";
        let events = parse(input);

        assert!(matches!(events[1], BlockEvent::ListItemStart { task: TaskState::Unchecked }));
    }

    #[test]
    fn test_list_task_checked() {
        let input = "- [x] done";
        let events = parse(input);

        assert!(matches!(events[1], BlockEvent::ListItemStart { task: TaskState::Checked }));
    }

    #[test]
    fn test_list_task_checked_uppercase() {
        let input = "- [X] done";
        let events = parse(input);

        assert!(matches!(events[1], BlockEvent::ListItemStart { task: TaskState::Checked }));
    }

    #[test]
    fn test_list_ends_on_blank() {
        let input = "- item\n\nparagraph";
        let events = parse(input);

        let has_list_end = events.iter().any(|e| matches!(e, BlockEvent::ListEnd { .. }));
        assert!(has_list_end);
    }

    #[test]
    fn test_blockquote_with_list() {
        let input = "> - item";
        let events = parse(input);

        assert_eq!(events[0], BlockEvent::BlockQuoteStart);
        assert!(matches!(events[1], BlockEvent::ListStart { .. }));
    }
}
