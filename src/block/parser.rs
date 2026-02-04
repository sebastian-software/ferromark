//! Block parser implementation.

use crate::cursor::Cursor;
use crate::limits;
use crate::Range;
use smallvec::SmallVec;

use super::event::{BlockEvent, ListKind, TaskState};
use crate::link_ref::{LinkRefStore, normalize_label_into, LinkRefDef};

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

/// HTML block kinds (CommonMark types 1-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HtmlBlockKind {
    Type1,
    Type2,
    Type3,
    Type4,
    Type5,
    Type6,
    Type7,
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
    /// For list items, this is used to implement the two-blank-line rule.
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
    /// Extra spaces to prepend to each line of indented code (from tab expansion).
    indented_code_extra_spaces: usize,
    /// Pending blank lines in indented code (only emit if code continues).
    /// Stores (extra_spaces_beyond_4, newline_range) for each blank line.
    pending_code_blanks: Vec<(u8, Range)>,
    /// Current HTML block kind, if inside an HTML block.
    html_block: Option<HtmlBlockKind>,
    /// Number of bytes skipped by the last leading-indent scan for this line.
    line_indent_bytes: usize,
    /// Optional indent start override for the first line of an HTML block.
    pending_html_indent_start: Option<usize>,
    /// Collected link reference definitions.
    link_refs: LinkRefStore,
    /// Stack of open containers (blockquotes, list items).
    container_stack: SmallVec<[Container; 8]>,
    /// Whether we're in a tight list context.
    #[allow(dead_code)]
    tight_list: bool,
    /// Currently open lists (for tracking across item closes).
    open_lists: SmallVec<[OpenList; 4]>,
    /// Remaining columns from a partially-consumed tab.
    /// When a tab expands beyond what's needed for container indent, the excess
    /// columns are stored here and added to the next indent measurement.
    partial_tab_cols: usize,
    /// Current absolute column position within the line (for tab expansion).
    current_col: usize,
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
            indented_code_extra_spaces: 0,
            pending_code_blanks: Vec::new(),
            html_block: None,
            line_indent_bytes: 0,
            pending_html_indent_start: None,
            link_refs: LinkRefStore::new(),
            container_stack: SmallVec::new(),
            tight_list: false,
            open_lists: SmallVec::new(),
            partial_tab_cols: 0,
            current_col: 0,
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

        // Close any unclosed indented code block (discard trailing blanks)
        if self.in_indented_code {
            self.pending_code_blanks.clear();
            self.in_indented_code = false;
            events.push(BlockEvent::CodeBlockEnd);
        }

        // Close any unclosed HTML block
        if self.html_block.is_some() {
            self.html_block = None;
            events.push(BlockEvent::HtmlBlockEnd);
        }

        // Close all open containers
        self.close_all_containers(events);
    }

    /// Take the collected link reference definitions.
    pub fn take_link_refs(&mut self) -> LinkRefStore {
        std::mem::take(&mut self.link_refs)
    }

    /// Parse a single line.
    fn parse_line(&mut self, events: &mut Vec<BlockEvent>) {
        let line_start = self.cursor.offset();

        // Reset column tracking at the start of each line
        self.partial_tab_cols = 0;
        self.current_col = 0;

        // Check for blank line first (before any space skipping), unless we're in an HTML block
        if self.html_block.is_none() && self.is_blank_line() {
            // If we're inside a fenced code block, ensure containers still match.
            if self.fence_state.is_some() {
                self.cursor = Cursor::new_at(self.input, line_start);
                self.partial_tab_cols = 0;
                self.current_col = 0;
                let matched_containers = self.match_containers(events);
                if matched_containers < self.container_stack.len() {
                    self.fence_state = None;
                    events.push(BlockEvent::CodeBlockEnd);
                    let (indent, _) = self.skip_indent();
                    self.close_containers_from(matched_containers, indent, events);
                    self.close_paragraph(events);
                    self.handle_blank_line_containers(events, true);
                    return;
                }
                self.parse_fence_line_in_container(events);
                return;
            }
            // Track columns while skipping whitespace (for code block blank lines)
            let mut cols = 0usize;
            while let Some(b) = self.cursor.peek() {
                if b == b' ' {
                    cols += 1;
                    self.cursor.bump();
                } else if b == b'\t' {
                    cols = (cols + 4) & !3;
                    self.cursor.bump();
                } else {
                    break;
                }
            }
            let newline_start = self.cursor.offset();
            if !self.cursor.is_eof() && self.cursor.at(b'\n') {
                self.cursor.bump();
            }
            let ws_end = self.cursor.offset();

            // Blank lines inside indented code are preserved (but buffered)
            // Check this BEFORE closing blockquotes, as closing will end the code block
            if self.in_indented_code {
                // Check if we're inside a blockquote - if so, the blank line closes it
                // which also closes the indented code block
                let has_blockquote = self.container_stack.iter()
                    .any(|c| c.typ == ContainerType::BlockQuote);
                if !has_blockquote {
                    // Buffer the blank line with any extra whitespace beyond 4 columns
                    let extra_spaces = cols.saturating_sub(4) as u8;
                    self.pending_code_blanks.push((extra_spaces, Range::new(newline_start as u32, ws_end as u32)));
                    return;
                }
                // Fall through to close blockquotes (which will close the code block too)
            }

            self.close_paragraph(events);
            // This is a truly blank line (no container markers) - close blockquotes
            self.handle_blank_line_containers(events, true);
            return;
        }

        // Reset to line start for container matching
        self.cursor = Cursor::new_at(self.input, line_start);

        // Try to match and continue existing containers
        // This handles the indent requirements per container type
        let matched_containers = self.match_containers(events);

        // If we're inside an HTML block, handle it after container matching.
        if self.html_block.is_some() {
            if matched_containers < self.container_stack.len() {
                // Containers didn't match, close the HTML block
                self.html_block = None;
                events.push(BlockEvent::HtmlBlockEnd);
                let (indent, _) = self.skip_indent();
                self.close_containers_from(matched_containers, indent, events);
                // Fall through to continue parsing the line normally
            } else {
                self.parse_html_block_line(events);
                return;
            }
        }

        // If we're inside a fenced code block, handle it after container matching
        // but BEFORE skip_spaces (since spaces may be part of code content)
        if self.fence_state.is_some() {
            // If containers didn't match, close the fenced code block
            if matched_containers < self.container_stack.len() {
                self.fence_state = None;
                events.push(BlockEvent::CodeBlockEnd);
                // Close unmatched containers and continue with normal parsing
                let (indent, _) = self.skip_indent();
                self.close_containers_from(matched_containers, indent, events);
                // Fall through to continue parsing the line normally
            } else {
                // Containers matched, handle as fenced code line
                self.parse_fence_line_in_container(events);
                return;
            }
        }

        // Get current indent (in columns) after container matching
        let (indent, indent_bytes) = self.skip_indent();
        self.line_indent_bytes = indent_bytes;

        // Check for blank line AFTER container matching (e.g., ">>" followed by newline)
        if self.cursor.is_eof() || self.cursor.at(b'\n') {
            if !self.cursor.is_eof() {
                self.cursor.bump();
            }
            self.close_paragraph(events);
            // Container markers were present, so don't close blockquotes
            self.handle_blank_line_containers(events, false);
            return;
        }

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
            // Note: setext underlines do NOT work via lazy continuation - they must be at
            // the same container level as the paragraph content. So we don't check for them here.
            if self.can_lazy_continue(matched_containers, indent) {
                // Normal lazy continuation - add this line to the paragraph
                let line_start = self.cursor.offset();
                self.parse_paragraph_line(line_start, events);
                return;
            }

            // close_containers_from is smart about keeping lists open when starting new items
            // Pass indent so it knows if a new item is actually possible (only at indent < 4)
            self.close_containers_from(matched_containers, indent, events);
        }

        // If we're in an indented code block and containers matched, handle continuation
        if self.in_indented_code {
            if indent >= 4 {
                // Continue the code block - first emit any pending blank lines
                for (extra_spaces, blank_range) in self.pending_code_blanks.drain(..) {
                    if extra_spaces > 0 {
                        events.push(BlockEvent::VirtualSpaces(extra_spaces));
                    }
                    events.push(BlockEvent::Code(blank_range));
                }

                // Calculate extra spaces for this line (columns beyond 4)
                let extra_spaces = indent.saturating_sub(4);

                // Cursor is past all whitespace. Content starts at current position.
                let text_start = self.cursor.offset();
                let line_end = self.find_line_end();
                let content_end = if !self.cursor.is_eof() && self.cursor.at(b'\n') {
                    self.cursor.bump();
                    line_end + 1
                } else {
                    line_end
                };

                // Emit virtual spaces if there are extra columns
                if extra_spaces > 0 {
                    events.push(BlockEvent::VirtualSpaces(extra_spaces as u8));
                }
                events.push(BlockEvent::Code(Range::new(text_start as u32, content_end as u32)));
                return;
            } else {
                // Close the code block - discard pending blank lines (trailing blanks)
                self.pending_code_blanks.clear();
                self.in_indented_code = false;
                self.indented_code_extra_spaces = 0;
                events.push(BlockEvent::CodeBlockEnd);
            }
        }

        // Check for setext heading underline (when in a paragraph)
        // Must check BEFORE thematic break since `---` can be either
        // Note: indent must be < 4 for a valid setext underline
        if indent < 4 && self.in_paragraph {
            if let Some(level) = self.is_setext_underline_after_indent() {
                // Strip link reference definitions before deciding on setext conversion.
                let consumed = self.extract_link_ref_defs();
                if consumed > 0 {
                    let drain_count = consumed.min(self.paragraph_lines.len());
                    self.paragraph_lines.drain(0..drain_count);
                }
                if self.paragraph_lines.is_empty() {
                    // No paragraph content left after stripping definitions; not a setext heading.
                    // Treat this line as normal paragraph content.
                    let line_start = self.cursor.offset();
                    self.parse_paragraph_line(line_start, events);
                    return;
                } else {
                    // Skip to end of line
                    while !self.cursor.is_eof() && !self.cursor.at(b'\n') {
                        self.cursor.bump();
                    }
                    if !self.cursor.is_eof() {
                        self.cursor.bump();
                    }
                    // Convert paragraph to heading
                    self.close_paragraph_as_setext_heading(level, events);
                    return;
                }
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
            // Pass the pre-marker indent so content_indent can be calculated correctly
            if self.try_list_item(indent, events) {
                self.parse_line_content(events);
                return;
            }
        }

        // Check for indented code block (4+ spaces, not in paragraph)
        if indent >= 4 && !self.in_paragraph {
            self.start_indented_code(indent, events);
            return;
        }

        // Parse regular block content (pass known indent to avoid re-measuring)
        self.parse_line_content_with_indent(indent, events);
    }

    /// Check if line is blank after consuming whitespace.
    fn is_blank_line(&self) -> bool {
        let slice = self.cursor.remaining_slice();
        for &b in slice {
            if b == b' ' || b == b'\t' {
                continue;
            }
            return b == b'\n';
        }
        true // EOF is treated as blank
    }

    /// Calculate the column that a tab at the given column would expand to.
    #[inline]
    fn tab_column(col: usize) -> usize {
        (col + 4) & !3
    }

    /// Skip whitespace (spaces and tabs) returning (columns, bytes).
    /// Includes any remaining columns from a partially-consumed tab.
    /// Uses self.current_col for correct tab expansion and updates it.
    fn skip_indent(&mut self) -> (usize, usize) {
        // Add any remaining columns from a partially-consumed tab
        let partial = self.partial_tab_cols;
        self.partial_tab_cols = 0;

        let start_col = self.current_col;
        let mut bytes = 0;
        while let Some(b) = self.cursor.peek() {
            if b == b' ' {
                self.current_col += 1;
                bytes += 1;
                self.cursor.bump();
            } else if b == b'\t' {
                self.current_col = Self::tab_column(self.current_col);
                bytes += 1;
                self.cursor.bump();
            } else {
                break;
            }
        }
        // Return the number of columns measured PLUS any partial tab carryover
        (self.current_col - start_col + partial, bytes)
    }

    /// Skip up to `max_cols` columns of whitespace, returning (columns_skipped, bytes_skipped).
    /// If a tab would exceed max_cols, consumes the tab and stores excess in partial_tab_cols.
    /// Uses self.current_col for correct tab expansion and updates it.
    fn skip_indent_max(&mut self, max_cols: usize) -> (usize, usize) {
        // Include any carryover from previous partial tab
        let partial = self.partial_tab_cols;
        self.partial_tab_cols = 0;

        // If we already have enough from partial tab, just return
        if partial >= max_cols {
            self.partial_tab_cols = partial - max_cols;
            return (max_cols, 0);
        }

        let mut bytes = 0;
        let mut cols_counted = partial;

        while cols_counted < max_cols {
            match self.cursor.peek() {
                Some(b' ') => {
                    cols_counted += 1;
                    self.current_col += 1;
                    bytes += 1;
                    self.cursor.bump();
                }
                Some(b'\t') => {
                    let next_col = Self::tab_column(self.current_col);
                    let tab_width = next_col - self.current_col;
                    let cols_needed = max_cols - cols_counted;
                    if tab_width <= cols_needed {
                        cols_counted += tab_width;
                        self.current_col = next_col;
                        bytes += 1;
                        self.cursor.bump();
                    } else {
                        // Tab would exceed max_cols - consume it but save excess
                        self.partial_tab_cols = tab_width - cols_needed;
                        self.current_col = next_col;
                        bytes += 1;
                        self.cursor.bump();
                        return (max_cols, bytes);
                    }
                }
                _ => break,
            }
        }
        (cols_counted, bytes)
    }

    /// Parse line content after container markers have been handled.
    /// Measures indent from current position.
    fn parse_line_content(&mut self, events: &mut Vec<BlockEvent>) {
        let (indent, indent_bytes) = self.skip_indent();
        self.line_indent_bytes = indent_bytes;
        self.parse_line_content_with_indent(indent, events);
    }

    /// Parse line content with a known indent value.
    /// Cursor should already be past the leading whitespace.
    fn parse_line_content_with_indent(&mut self, indent: usize, events: &mut Vec<BlockEvent>) {
        let first = self.cursor.peek_or_zero();

        // Check for blank line (can happen after container markers)
        if first == 0 || first == b'\n' {
            if first == b'\n' {
                self.cursor.bump();
            }
            self.close_paragraph(events);
            return;
        }

        // Try to parse block-level constructs (only if indent < 4)
        if indent < 4 {
            if is_simple_line_start(first) {
                let line_start = self.cursor.offset();
                self.parse_paragraph_line(line_start, events);
                return;
            }

            // Check for setext heading underline (when in a paragraph)
            // Must check BEFORE thematic break since `---` can be either
            if self.in_paragraph {
                if let Some(level) = self.is_setext_underline_after_indent() {
                    // Skip to end of line
                    while !self.cursor.is_eof() && !self.cursor.at(b'\n') {
                        self.cursor.bump();
                    }
                    if !self.cursor.is_eof() {
                        self.cursor.bump();
                    }
                    // Convert paragraph to heading
                    self.close_paragraph_as_setext_heading(level, events);
                    return;
                }
            }

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

            // Check for list item (pass indent for absolute content_indent calculation)
            if self.try_list_item(indent, events) {
                self.parse_line_content(events);
                return;
            }
        }

        // Check for HTML block
        if self.try_html_block_start(indent, events) {
            return;
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
        // Track the deepest list that matched with non-blank content
        // (this is the level where a blank line would make the list loose)
        let mut deepest_list_match: Option<usize> = None;

        for i in 0..self.container_stack.len() {
            let container = &self.container_stack[i];
            match container.typ {
                ContainerType::BlockQuote => {
                    // Try to match `>` marker with up to 3 leading spaces/tabs
                    let save_pos = self.cursor.offset();
                    let save_partial = self.partial_tab_cols;
                    let save_col = self.current_col;
                    let (cols, _bytes) = self.skip_indent();
                    if cols <= 3 && self.cursor.at(b'>') {
                        self.cursor.bump();
                        self.current_col += 1;
                        // Optional space after > - use skip_indent_max(1) for proper tab handling
                        self.skip_indent_max(1);
                        matched += 1;
                    } else {
                        // Can't continue blockquote, reset cursor and break
                        self.cursor = Cursor::new_at(self.input, save_pos);
                        self.partial_tab_cols = save_partial;
                        self.current_col = save_col;
                        break;
                    }
                }
                ContainerType::ListItem { content_indent, kind, marker } => {
                    // Check if line is blank (after any spaces we've consumed so far)
                    let remaining = self.cursor.remaining_slice();
                    let is_blank = remaining.is_empty() || remaining[0] == b'\n' ||
                        remaining.iter().take_while(|&&b| b == b' ' || b == b'\t')
                            .count() == remaining.len().min(remaining.iter().position(|&b| b == b'\n').unwrap_or(remaining.len()));

                    if is_blank {
                        // Blank lines always match list items
                        matched += 1;
                    } else {
                        // Save position, partial tab state, and column
                        let save_pos = self.cursor.offset();
                        let save_partial = self.partial_tab_cols;
                        let save_col = self.current_col;
                        let (cols, _bytes) = self.skip_indent();

                        if cols >= content_indent {
                            // Enough indent to continue the list item
                            // We need to position the cursor so that only content_indent
                            // columns have been consumed. This is tricky with tabs.
                            // Rewind and skip exactly content_indent columns.
                            self.cursor = Cursor::new_at(self.input, save_pos);
                            self.partial_tab_cols = save_partial;
                            self.current_col = save_col;
                            let (_skipped_cols, _skipped_bytes) = self.skip_indent_max(content_indent);
                            // Now cursor is past content_indent columns worth of whitespace
                            // Any excess from partial tab consumption is in partial_tab_cols

                            // Track this list as a potential loose candidate
                            let list_index = self.container_stack[..=i].iter()
                                .filter(|c| matches!(c.typ, ContainerType::ListItem { .. }))
                                .count();
                            if list_index > 0 {
                                deepest_list_match = Some(list_index - 1);
                            }
                            matched += 1;
                        } else {
                            // Not enough indent - check if it's a new list item of same type
                            // (cursor is already past the whitespace)
                            let is_same_list = self.peek_list_marker(kind, marker);

                            // Reset cursor, partial tab state, and column
                            self.cursor = Cursor::new_at(self.input, save_pos);
                            self.partial_tab_cols = save_partial;
                            self.current_col = save_col;

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

        // After matching, mark the deepest matched list as loose if it had a blank line.
        // This ensures the blank only affects the innermost level that actually continues.
        if let Some(list_idx) = deepest_list_match {
            if let Some(open_list) = self.open_lists.get_mut(list_idx) {
                if open_list.blank_in_item {
                    open_list.tight = false;
                }
            }
            // Clear blank_in_item for all outer lists - the blank was "consumed" by the deeper level
            for outer_list in self.open_lists[..list_idx].iter_mut() {
                outer_list.blank_in_item = false;
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
                // Followed by space, tab, or newline (blank list item)
                if b != marker {
                    return false;
                }
                let after = self.cursor.peek_ahead(1);
                after == Some(b' ') || after == Some(b'\t') || after == Some(b'\n') || after.is_none()
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
                if self.cursor.peek_ahead(offset) != Some(delimiter) {
                    return false;
                }
                // Must be followed by space, tab, or newline
                let after = self.cursor.peek_ahead(offset + 1);
                after == Some(b' ') || after == Some(b'\t') || after == Some(b'\n') || after.is_none()
            }
        }
    }

    /// Check if we can do lazy continuation for a container (blockquote or list item).
    /// Returns true if:
    /// 1. We're in a paragraph
    /// 2. The first unmatched container is a blockquote or list item
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

        // The first unmatched container must be a blockquote or list item
        // (CommonMark allows lazy continuation for paragraphs in both)
        let container = &self.container_stack[matched];
        let is_lazy_container = matches!(
            container.typ,
            ContainerType::BlockQuote | ContainerType::ListItem { .. }
        );
        if !is_lazy_container {
            return false;
        }

        // Note: we don't check indent >= 4 for indented code here because
        // we're in a paragraph. Indented code blocks can only start when
        // NOT in a paragraph.

        // Check if the current line would start a new block
        // Pass indent to would_start_block so it can ignore block starts
        // at 4+ indent (which become continuation or indented code)
        !self.would_start_block(indent)
    }

    /// Check if the current position would start a new block.
    /// Used for lazy continuation checks.
    /// `indent` is the number of spaces at the start of the line (before current position).
    fn would_start_block(&self, indent: usize) -> bool {
        let b = self.cursor.peek_or_zero();
        if b == 0 {
            return false;
        }

        match b {
            // ATX heading - only at indent < 4
            b'#' => indent < 4,
            // Fenced code block - only at indent < 4
            b'`' | b'~' => indent < 4 && self.cursor.remaining_slice().iter().take_while(|&&c| c == b).count() >= 3,
            // Blockquote - only at indent < 4
            b'>' => indent < 4,
            // Unordered list marker or thematic break - only at indent < 4
            b'-' | b'*' | b'+' => {
                if indent >= 4 {
                    // At 4+ indent, thematic breaks are still possible
                    return self.peek_thematic_break();
                }
                // Check if followed by space/tab/newline (list item) or if it's a thematic break
                let after = self.cursor.peek_ahead(1);
                after == Some(b' ') || after == Some(b'\t') || after == Some(b'\n')
                    || after.is_none() || self.peek_thematic_break()
            }
            // Ordered list marker - only at indent < 4
            b'0'..=b'9' => {
                if indent >= 4 {
                    return false;
                }
                // Check if digit(s) followed by . or ) then space/tab/newline
                let mut offset = 1;
                while self.cursor.peek_ahead(offset).map_or(false, |c| c.is_ascii_digit()) {
                    offset += 1;
                }
                let delim = self.cursor.peek_ahead(offset);
                if delim != Some(b'.') && delim != Some(b')') {
                    return false;
                }
                let after = self.cursor.peek_ahead(offset + 1);
                after == Some(b' ') || after == Some(b'\t') || after == Some(b'\n') || after.is_none()
            }
            // HTML block (only types that can interrupt paragraphs) - only at indent < 4
            b'<' => indent < 4 && self.peek_html_block_start(true).is_some(),
            // Note: We don't check for setext underlines (= or plain line of -) here because
            // setext underlines can't interrupt lazy continuation. They only work when the
            // paragraph is at the same container level as the underline.
            // Blank or other content - not a block start
            _ => false,
        }
    }

    /// Close containers starting from index, being smart about lists.
    /// `indent` is the current line's indent - used to determine if a new list item is possible.
    fn close_containers_from(&mut self, from: usize, indent: usize, events: &mut Vec<BlockEvent>) {
        // Check if we're about to close a list item but might start a new one
        while self.container_stack.len() > from {
            let top = self.container_stack.last().unwrap();

            // Only consider "same list" continuation if:
            // 1. This is the LAST container we need to close
            // 2. The indent is < 4 (otherwise we can't start a new item)
            let is_last_to_close = self.container_stack.len() == from + 1;
            let can_start_new_item = indent < 4;

            if is_last_to_close && can_start_new_item {
                if let ContainerType::ListItem { kind, marker, .. } = top.typ {
                    // Check if the current position has a same-type list marker
                    let save_pos = self.cursor.offset();
                    let save_partial = self.partial_tab_cols;
                    let save_col = self.current_col;
                    self.skip_indent();
                    let is_same_list = self.peek_list_marker(kind, marker);
                    self.cursor = Cursor::new_at(self.input, save_pos);
                    self.partial_tab_cols = save_partial;
                    self.current_col = save_col;

                    if is_same_list {
                        // Just close the item, not the list
                        self.container_stack.pop();
                        self.close_paragraph(events);
                        events.push(BlockEvent::ListItemEnd);
                        // Don't pop from open_lists
                        continue;
                    }
                }
            }

            self.close_top_container(events);
        }
    }

    /// Handle blank line for container continuation.
    /// `close_blockquotes`: true if this is a truly blank line (no `>` markers),
    /// false if the line had container markers but blank content.
    fn handle_blank_line_containers(&mut self, events: &mut Vec<BlockEvent>, close_blockquotes: bool) {
        // A blank line (without > marker) closes blockquotes
        if close_blockquotes {
            // Close all blockquote containers from the top
            while let Some(container) = self.container_stack.last() {
                if container.typ == ContainerType::BlockQuote {
                    self.close_top_container(events);
                } else {
                    break;
                }
            }
        }

        // Two-blank-line rule: a list item can begin with at most one blank line.
        // If the innermost list item has no content and we see a blank line, close it.
        // But keep the list open - only close the item.
        if let Some(container) = self.container_stack.last() {
            if let ContainerType::ListItem { .. } = container.typ {
                if !container.has_content {
                    // This is the second blank line (first was the blank item itself)
                    // Close just the list item, not the list
                    self.container_stack.pop();
                    self.close_paragraph(events);
                    events.push(BlockEvent::ListItemEnd);
                    // Mark blank_in_item for the list - the blank line is between items,
                    // which will make the list loose when the next item starts.
                    if let Some(open_list) = self.open_lists.last_mut() {
                        open_list.blank_in_item = true;
                    }
                    return; // Already handled blank marking for this case
                }
            }
        }

        // Mark all lists with an active item as having seen a blank line.
        // We don't know yet which level the blank is at - that's determined
        // when we see the continuation line.
        //
        // BUT: If close_blockquotes is false, the line had container markers (like `>`).
        // If there's a blockquote on the stack, the blank is inside the blockquote,
        // not directly in the list item, so don't mark the list.
        if !close_blockquotes {
            if let Some(container) = self.container_stack.last() {
                if container.typ == ContainerType::BlockQuote {
                    // The blank is inside a blockquote, not between list item blocks
                    return;
                }
            }
        }

        let active_list_count = self.container_stack.iter()
            .filter(|c| matches!(c.typ, ContainerType::ListItem { .. }))
            .count();
        let start_idx = self.open_lists.len().saturating_sub(active_list_count);
        for open_list in self.open_lists[start_idx..].iter_mut() {
            open_list.blank_in_item = true;
        }
    }

    /// Try to start a blockquote.
    fn try_blockquote(&mut self, events: &mut Vec<BlockEvent>) -> bool {
        if !self.cursor.at(b'>') {
            return false;
        }

        self.cursor.bump(); // consume >
        self.current_col += 1;

        // Optional space after > (consumes 1 column of whitespace)
        // Use skip_indent_max(1) to properly handle partial tab consumption.
        // If it's a tab, the excess columns are stored in partial_tab_cols.
        self.skip_indent_max(1);

        // Close paragraph if any
        self.close_paragraph(events);

        // Mark the current container as having content (before pushing new container)
        self.mark_container_has_content();

        // Push blockquote container
        self.container_stack.push(Container {
            typ: ContainerType::BlockQuote,
            has_content: false,
        });

        events.push(BlockEvent::BlockQuoteStart);
        true
    }

    /// Try to start a list item.
    /// `pre_marker_indent` is the number of spaces before the list marker (absolute column position).
    fn try_list_item(&mut self, pre_marker_indent: usize, events: &mut Vec<BlockEvent>) -> bool {
        let start_offset = self.cursor.offset();

        // Check for unordered list marker (-, *, +)
        if let Some((marker, relative_content_indent)) = self.try_unordered_marker() {
            // CommonMark: a blank list item cannot interrupt a paragraph
            // A blank item has relative_content_indent == 2 (marker + 1 implicit space)
            let is_blank_item = relative_content_indent == 2 && self.cursor.at(b'\n');
            if self.in_paragraph && is_blank_item {
                // Reset cursor and don't start list
                self.cursor = Cursor::new_at(self.input, start_offset);
                return false;
            }

            // Absolute content_indent = spaces before marker + marker width + spaces after marker
            let absolute_content_indent = pre_marker_indent + relative_content_indent;
            self.start_list_item(ListKind::Unordered, marker, absolute_content_indent, events);
            return true;
        }

        // Check for ordered list marker (1. 2. etc)
        if let Some((start_num, relative_content_indent, delimiter)) = self.try_ordered_marker() {
            // CommonMark: an ordered list can only interrupt a paragraph if it starts with 1
            // Also, a blank list item cannot interrupt a paragraph
            let is_blank_item = self.cursor.at(b'\n');
            if self.in_paragraph && (start_num != 1 || is_blank_item) {
                // Reset cursor and don't start list
                self.cursor = Cursor::new_at(self.input, start_offset);
                return false;
            }
            // Absolute content_indent = spaces before marker + marker width + spaces after marker
            let absolute_content_indent = pre_marker_indent + relative_content_indent;
            self.start_list_item(
                ListKind::Ordered { start: start_num, delimiter },
                delimiter,
                absolute_content_indent,
                events,
            );
            return true;
        }

        false
    }

    /// Try to parse an unordered list marker (-, *, +).
    /// Returns (marker_char, relative_content_indent) where relative_content_indent is
    /// the column offset from marker to where content starts.
    fn try_unordered_marker(&mut self) -> Option<(u8, usize)> {
        let marker = self.cursor.peek()?;
        if marker != b'-' && marker != b'*' && marker != b'+' {
            return None;
        }

        // Must be followed by space or tab or newline
        let after_marker = self.cursor.peek_ahead(1);
        if after_marker != Some(b' ') && after_marker != Some(b'\t') && after_marker != Some(b'\n') {
            // Could be thematic break for - and *
            return None;
        }

        self.cursor.bump(); // consume marker (1 column)
        self.current_col += 1;

        // Handle blank list item (marker followed by newline)
        if self.cursor.at(b'\n') {
            return Some((marker, 2)); // marker + 1 implicit space
        }

        // Count columns of whitespace between marker and content
        let pos_after_marker = self.cursor.offset();
        let col_after_marker = self.current_col;
        let partial_after_marker = self.partial_tab_cols;
        let (cols_after_marker, _bytes_after_marker) = self.skip_indent();

        // Check if this is a blank list item (only whitespace after marker)
        if self.cursor.at(b'\n') || self.cursor.is_eof() {
            // Blank list item: content_indent = marker(1) + 1 implicit space
            // Reset cursor to after just 1 space/tab column
            self.cursor = Cursor::new_at(self.input, pos_after_marker);
            self.current_col = col_after_marker;
            self.partial_tab_cols = partial_after_marker;
            self.skip_indent_max(1);
            return Some((marker, 2));
        }

        // CommonMark rule: 1-4 columns after marker is normal
        // 5+ columns means blank item with indented content (only 1 counts)
        if cols_after_marker >= 5 {
            // Put cursor back to just after 1 column of whitespace
            self.cursor = Cursor::new_at(self.input, pos_after_marker);
            self.current_col = col_after_marker;
            self.partial_tab_cols = partial_after_marker;
            self.skip_indent_max(1);
            // content_indent = marker(1) + 1 column
            return Some((marker, 2));
        }

        // content_indent = marker(1) + cols_after_marker
        Some((marker, 1 + cols_after_marker))
    }

    /// Try to parse an ordered list marker (1. 2. etc).
    /// Returns (number, relative_content_indent, delimiter) where delimiter is '.' or ')'.
    /// relative_content_indent is in columns.
    fn try_ordered_marker(&mut self) -> Option<(u32, usize, u8)> {
        let start = self.cursor.offset();
        let start_col = self.current_col;
        let mut num: u32 = 0;
        let mut digits = 0;

        // Parse digits
        while let Some(b) = self.cursor.peek() {
            if b.is_ascii_digit() {
                if digits >= limits::MAX_LIST_MARKER_DIGITS {
                    // Too many digits, reset and return
                    self.cursor = Cursor::new_at(self.input, start);
                    self.current_col = start_col;
                    return None;
                }
                num = num * 10 + (b - b'0') as u32;
                digits += 1;
                self.cursor.bump();
                self.current_col += 1;
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
                self.current_col = start_col;
                return None;
            }
        };
        self.cursor.bump(); // consume . or )
        self.current_col += 1;

        // Must be followed by space, tab, or newline
        if !self.cursor.at(b' ') && !self.cursor.at(b'\t') && !self.cursor.at(b'\n') {
            self.cursor = Cursor::new_at(self.input, start);
            self.current_col = start_col;
            return None;
        }

        // Handle blank list item (marker followed by newline)
        if self.cursor.at(b'\n') {
            // relative_content_indent = digits + delimiter + 1 implicit space
            let relative_content_indent = digits + 2;
            return Some((num, relative_content_indent, delimiter));
        }

        // Count columns of whitespace between marker and content
        let pos_after_delim = self.cursor.offset();
        let col_after_delim = self.current_col;
        let partial_after_delim = self.partial_tab_cols;
        let (cols_after_marker, _bytes) = self.skip_indent();

        // Check if this is a blank list item (only whitespace after marker)
        if self.cursor.at(b'\n') || self.cursor.is_eof() {
            // Blank list item: relative_content_indent = digits + delimiter + 1 column
            // Reset cursor to after just 1 column of whitespace
            self.cursor = Cursor::new_at(self.input, pos_after_delim);
            self.current_col = col_after_delim;
            self.partial_tab_cols = partial_after_delim;
            self.skip_indent_max(1);
            let relative_content_indent = digits + 2;
            return Some((num, relative_content_indent, delimiter));
        }

        // CommonMark rule: 1-4 columns after marker is normal
        // 5+ columns means blank item with indented content (only 1 counts)
        if cols_after_marker >= 5 {
            // Put cursor back to just after 1 column of whitespace
            self.cursor = Cursor::new_at(self.input, pos_after_delim);
            self.current_col = col_after_delim;
            self.partial_tab_cols = partial_after_delim;
            self.skip_indent_max(1);
            // relative_content_indent = digits + delimiter(1) + 1 column
            let relative_content_indent = digits + 2;
            return Some((num, relative_content_indent, delimiter));
        }

        // relative_content_indent = digits + delimiter(1) + cols_after_marker
        let relative_content_indent = digits + 1 + cols_after_marker;
        Some((num, relative_content_indent, delimiter))
    }

    /// Start a new list item.
    fn start_list_item(
        &mut self,
        kind: ListKind,
        marker: u8,
        content_indent: usize,
        events: &mut Vec<BlockEvent>,
    ) {
        // Close paragraph if any
        self.close_paragraph(events);

        // Check if we have an open list waiting for more items.
        // We're continuing a list if:
        // 1. There's a compatible open list, AND
        // 2. Either we're not inside a list item, OR there are more open lists
        //    than list items in container_stack (meaning there's a nested list waiting)
        let list_item_count = self.container_stack.iter()
            .filter(|c| matches!(c.typ, ContainerType::ListItem { .. }))
            .count();
        // If open_lists.len() > list_item_count, there's a "free" open list
        // that was started but whose item was closed - we should continue it
        let has_waiting_list = self.open_lists.len() > list_item_count;
        let continuing_list = has_waiting_list && self.is_compatible_list(kind, marker);

        // Check if we're inside a list item (for the nesting case)
        let inside_list_item = list_item_count > 0;

        if !continuing_list {
            // Close any existing list items from incompatible lists
            // (but not if we're nesting inside a matched item)
            if !inside_list_item {
                while let Some(container) = self.container_stack.last() {
                    if matches!(container.typ, ContainerType::ListItem { .. }) {
                        self.close_top_container(events);
                    } else {
                        break;
                    }
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

        // Mark the current container as having content (before pushing new container)
        self.mark_container_has_content();

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

            // Close any open indented code block inside this container
            if self.in_indented_code {
                self.pending_code_blanks.clear();
                self.in_indented_code = false;
                events.push(BlockEvent::CodeBlockEnd);
            }

            match container.typ {
                ContainerType::BlockQuote => {
                    events.push(BlockEvent::BlockQuoteEnd);
                }
                ContainerType::ListItem { kind, .. } => {
                    events.push(BlockEvent::ListItemEnd);

                    // Count remaining ListItem containers
                    let remaining_items = self.container_stack.iter()
                        .filter(|c| matches!(c.typ, ContainerType::ListItem { .. }))
                        .count();

                    // Close lists until open_lists count matches remaining items
                    // This properly handles nested lists: each nesting level has one
                    // ListItem container and one open list
                    while self.open_lists.len() > remaining_items {
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

        // Mark the current container as having content
        self.mark_container_has_content();

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

        // Mark the current container as having content
        self.mark_container_has_content();

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
    /// Check if the current position (after indent has been skipped) is a setext underline.
    /// Returns Some(level) where level is 1 for '=' and 2 for '-', or None.
    /// Unlike peek_setext_underline, this assumes indent has already been consumed.
    fn is_setext_underline_after_indent(&self) -> Option<u8> {
        let slice = self.cursor.remaining_slice();
        if slice.is_empty() {
            return None;
        }

        // Must start with = or -
        let underline_char = slice[0];
        if underline_char != b'=' && underline_char != b'-' {
            return None;
        }

        // Count the underline characters (at least 1)
        let mut pos = 0;
        while pos < slice.len() && slice[pos] == underline_char {
            pos += 1;
        }

        // Skip trailing spaces/tabs
        while pos < slice.len() && (slice[pos] == b' ' || slice[pos] == b'\t') {
            pos += 1;
        }

        // Must end at newline or EOF
        if pos < slice.len() && slice[pos] != b'\n' {
            return None;
        }

        Some(if underline_char == b'=' { 1 } else { 2 })
    }

    /// Close the paragraph as a setext heading with the given level.
    fn close_paragraph_as_setext_heading(&mut self, level: u8, events: &mut Vec<BlockEvent>) {
        if !self.in_paragraph || self.paragraph_lines.is_empty() {
            return;
        }

        self.in_paragraph = false;

        // Mark the current container as having content
        self.mark_container_has_content();

        events.push(BlockEvent::HeadingStart { level });

        // Emit text ranges for each line with soft breaks between
        // Trim trailing spaces/tabs from the last line
        let line_count = self.paragraph_lines.len();
        for (i, mut range) in self.paragraph_lines.drain(..).enumerate() {
            if i > 0 {
                events.push(BlockEvent::SoftBreak);
            }
            // Trim trailing whitespace from the last line
            if i == line_count - 1 {
                while range.end > range.start {
                    let b = self.input[(range.end - 1) as usize];
                    if b == b' ' || b == b'\t' {
                        range.end -= 1;
                    } else {
                        break;
                    }
                }
            }
            events.push(BlockEvent::Text(range));
        }

        events.push(BlockEvent::HeadingEnd { level });
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

        // Mark the current container as having content
        self.mark_container_has_content();

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
    /// `indent_cols` is the number of columns of indentation measured.
    fn start_indented_code(&mut self, indent_cols: usize, events: &mut Vec<BlockEvent>) {
        // Close any open paragraph first
        self.close_paragraph(events);

        // Mark the current container as having content
        self.mark_container_has_content();

        // Start the code block
        self.in_indented_code = true;
        // Store the excess columns (indent_cols - 4) to prepend as spaces
        self.indented_code_extra_spaces = indent_cols.saturating_sub(4);
        events.push(BlockEvent::CodeBlockStart { info: None });

        // The cursor is past the whitespace bytes.
        // The cursor is past the whitespace bytes.
        let text_start = self.cursor.offset();

        // Find end of line (including newline for code blocks)
        let line_end = self.find_line_end();
        let content_end = if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
            line_end + 1 // Include the newline
        } else {
            line_end
        };

        // Emit virtual spaces if there are extra columns beyond 4
        if self.indented_code_extra_spaces > 0 {
            events.push(BlockEvent::VirtualSpaces(self.indented_code_extra_spaces as u8));
        }
        // Emit the content (including newline) - use Code event to skip inline parsing
        events.push(BlockEvent::Code(Range::new(text_start as u32, content_end as u32)));
    }

    /// Try to start an HTML block.
    /// Returns true if an HTML block was started and the line was consumed.
    fn try_html_block_start(&mut self, indent: usize, events: &mut Vec<BlockEvent>) -> bool {
        let kind = match self.peek_html_block_start(self.in_paragraph) {
            Some(kind) => kind,
            None => return false,
        };

        if indent >= 4 {
            return false;
        }

        // Close any open paragraph first
        self.close_paragraph(events);
        // Close any orphaned lists before starting a block
        self.close_orphaned_lists(events);
        // Mark the current container as having content
        self.mark_container_has_content();

        self.html_block = Some(kind);
        self.pending_html_indent_start = Some(
            self.cursor.offset().saturating_sub(self.line_indent_bytes),
        );
        events.push(BlockEvent::HtmlBlockStart);

        // Consume the current line as HTML block content
        self.parse_html_block_line(events);
        true
    }

    /// Parse a single HTML block line after container matching.
    /// Called when we're inside an HTML block and containers matched.
    fn parse_html_block_line(&mut self, events: &mut Vec<BlockEvent>) {
        let kind = self.html_block.unwrap();

        let indent_start = self.pending_html_indent_start.take().unwrap_or_else(|| self.cursor.offset());
        let (_indent, _) = self.skip_indent();
        let content_start = self.cursor.offset();

        // Blank line handling for types 6/7 (end on blank line)
        if self.cursor.is_eof() || self.cursor.at(b'\n') {
            if matches!(kind, HtmlBlockKind::Type6 | HtmlBlockKind::Type7) {
                self.html_block = None;
                events.push(BlockEvent::HtmlBlockEnd);

                if !self.cursor.is_eof() {
                    self.cursor.bump();
                }

                self.close_paragraph(events);
                let close_blockquotes = self.container_stack.is_empty();
                self.handle_blank_line_containers(events, close_blockquotes);
                return;
            }
        }

        // Find end of line (including newline)
        let line_end = self.find_line_end();
        let content_end = if !self.cursor.is_eof() && self.cursor.at(b'\n') {
            self.cursor.bump();
            line_end + 1
        } else {
            line_end
        };

        // Emit the raw HTML line (including any indentation after container markers)
        events.push(BlockEvent::HtmlBlockText(Range::from_usize(indent_start, content_end)));

        // Check for HTML block end markers (types 1-5)
        let line = &self.input[content_start..line_end];
        if self.html_block_ends(kind, line) {
            self.html_block = None;
            events.push(BlockEvent::HtmlBlockEnd);
        }
    }

    /// Check if the current line starts an HTML block.
    /// `in_paragraph` controls whether type 7 is allowed to start (it can't interrupt).
    fn peek_html_block_start(&self, in_paragraph: bool) -> Option<HtmlBlockKind> {
        let line = self.current_line_slice();
        if line.is_empty() {
            return None;
        }

        // Type 1: <script|pre|style|textarea
        if self.starts_with_tag(line, b"script")
            || self.starts_with_tag(line, b"pre")
            || self.starts_with_tag(line, b"style")
            || self.starts_with_tag(line, b"textarea")
        {
            return Some(HtmlBlockKind::Type1);
        }

        // Type 2: <!--
        if line.starts_with(b"<!--") {
            return Some(HtmlBlockKind::Type2);
        }

        // Type 3: <?
        if line.starts_with(b"<?") {
            return Some(HtmlBlockKind::Type3);
        }

        // Type 4: <![CDATA[
        if line.starts_with(b"<![CDATA[") {
            return Some(HtmlBlockKind::Type4);
        }

        // Type 5: <! + letter
        if line.starts_with(b"<!") && line.len() > 2 && line[2].is_ascii_alphabetic() {
            return Some(HtmlBlockKind::Type5);
        }

        // Type 6: block tags
        if let Some((name, tag_end)) = self.parse_html_tag_name(line) {
            if self.is_block_tag(name) && self.tag_boundary(line, tag_end) {
                return Some(HtmlBlockKind::Type6);
            }
        }

        // Type 7: any other HTML tag (cannot interrupt a paragraph)
        if in_paragraph {
            return None;
        }
        if let Some((_name, tag_end)) = self.parse_html_tag(line) {
            if line[tag_end..].iter().all(|&b| Self::is_html_whitespace(b)) {
                return Some(HtmlBlockKind::Type7);
            }
        }

        None
    }

    /// Check if an HTML block ends on this line (types 1-5).
    fn html_block_ends(&self, kind: HtmlBlockKind, line: &[u8]) -> bool {
        match kind {
            HtmlBlockKind::Type1 => {
                self.contains_ci(line, b"</script")
                    || self.contains_ci(line, b"</pre")
                    || self.contains_ci(line, b"</style")
                    || self.contains_ci(line, b"</textarea")
            }
            HtmlBlockKind::Type2 => self.contains_bytes(line, b"-->"),
            HtmlBlockKind::Type3 => self.contains_bytes(line, b"?>"),
            HtmlBlockKind::Type4 => self.contains_bytes(line, b"]]>"),
            HtmlBlockKind::Type5 => self.contains_bytes(line, b">"),
            HtmlBlockKind::Type6 | HtmlBlockKind::Type7 => false,
        }
    }

    /// Get the current line slice (from cursor to before newline).
    fn current_line_slice(&self) -> &[u8] {
        let offset = self.cursor.offset();
        let slice = &self.input[offset..];
        let end = slice.iter().position(|&b| b == b'\n').unwrap_or(slice.len());
        &slice[..end]
    }

    /// Check if line starts with "<tag" (case-insensitive) and valid boundary.
    fn starts_with_tag(&self, line: &[u8], tag: &[u8]) -> bool {
        if line.len() < tag.len() + 1 || line[0] != b'<' {
            return false;
        }
        let name_start = 1;
        let name_end = name_start + tag.len();
        if name_end > line.len() {
            return false;
        }
        if !self.eq_ignore_ascii_case(&line[name_start..name_end], tag) {
            return false;
        }
        self.tag_boundary(line, name_end)
    }


    /// Parse a valid HTML tag on a single line and return (tag_name, end_index_after_tag).
    fn parse_html_tag<'b>(&self, line: &'b [u8]) -> Option<(&'b [u8], usize)> {
        if line.first() != Some(&b'<') {
            return None;
        }
        let mut i = 1;
        let mut is_closing = false;
        if i < line.len() && line[i] == b'/' {
            is_closing = true;
            i += 1;
        }
        if i >= line.len() || !line[i].is_ascii_alphabetic() {
            return None;
        }
        let start = i;
        i += 1;
        while i < line.len() && (line[i].is_ascii_alphanumeric() || line[i] == b'-') {
            i += 1;
        }
        let name = &line[start..i];

        if is_closing {
            while i < line.len() && Self::is_html_whitespace(line[i]) {
                i += 1;
            }
            if i < line.len() && line[i] == b'>' {
                return Some((name, i + 1));
            }
            return None;
        }

        loop {
            if i >= line.len() {
                return None;
            }
            if line[i] == b'>' {
                return Some((name, i + 1));
            }
            if line[i] == b'/' {
                i += 1;
                return if i < line.len() && line[i] == b'>' { Some((name, i + 1)) } else { None };
            }
            if !Self::is_html_whitespace(line[i]) {
                return None;
            }
            while i < line.len() && Self::is_html_whitespace(line[i]) {
                i += 1;
            }
            if i >= line.len() {
                return None;
            }
            if line[i] == b'>' {
                return Some((name, i + 1));
            }
            if line[i] == b'/' {
                i += 1;
                return if i < line.len() && line[i] == b'>' { Some((name, i + 1)) } else { None };
            }
            if !Self::is_attr_name_start(line[i]) {
                return None;
            }
            i += 1;
            while i < line.len() && Self::is_attr_name_char(line[i]) {
                i += 1;
            }
            let ws_start = i;
            while i < line.len() && Self::is_html_whitespace(line[i]) {
                i += 1;
            }
            if i < line.len() && line[i] == b'=' {
                i += 1;
                while i < line.len() && Self::is_html_whitespace(line[i]) {
                    i += 1;
                }
                if i >= line.len() {
                    return None;
                }
                let quote = line[i];
                if quote == b'"' || quote == b'\'' {
                    i += 1;
                    let value_start = i;
                    while i < line.len() && line[i] != quote {
                        i += 1;
                    }
                    if i >= line.len() {
                        return None;
                    }
                    if i > value_start && line[i - 1] == b'\\' {
                        return None;
                    }
                    i += 1;
                } else {
                    let mut had = false;
                    while i < line.len() && !Self::is_html_whitespace(line[i]) {
                        let b = line[i];
                        if b == b'"' || b == b'\'' || b == b'=' || b == b'<' || b == b'>' || b == b'`' {
                            break;
                        }
                        had = true;
                        i += 1;
                    }
                    if !had {
                        return None;
                    }
                }
            } else {
                i = ws_start;
            }
        }
    }

    /// Parse tag name from a line starting with "<" or "</".
    /// Returns (tag_name, end_index_after_name).
    fn parse_html_tag_name<'b>(&self, line: &'b [u8]) -> Option<(&'b [u8], usize)> {
        if line.first() != Some(&b'<') {
            return None;
        }
        let mut i = 1;
        if i < line.len() && line[i] == b'/' {
            i += 1;
        }
        if i >= line.len() || !line[i].is_ascii_alphabetic() {
            return None;
        }
        let start = i;
        i += 1;
        while i < line.len() && (line[i].is_ascii_alphanumeric() || line[i] == b'-') {
            i += 1;
        }
        Some((&line[start..i], i))
    }

    /// Check if the character after tag name is a valid boundary.
    fn tag_boundary(&self, line: &[u8], idx: usize) -> bool {
        if idx >= line.len() {
            return true;
        }
        matches!(line[idx], b' ' | b'\t' | b'\n' | b'>' | b'/')
    }

    #[inline]
    fn is_html_whitespace(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
    }

    #[inline]
    fn is_attr_name_start(b: u8) -> bool {
        b.is_ascii_alphabetic() || b == b'_' || b == b':'
    }

    #[inline]
    fn is_attr_name_char(b: u8) -> bool {
        Self::is_attr_name_start(b) || b.is_ascii_digit() || b == b'.' || b == b'-'
    }


    /// Check if the tag name is in the CommonMark block tag list.
    fn is_block_tag(&self, name: &[u8]) -> bool {
        if name.len() == 2
            && (name[0] | 0x20) == b'h'
            && (b'1'..=b'6').contains(&name[1])
        {
            return true;
        }

        const BLOCK_TAGS: [&[u8]; 56] = [
            b"address", b"article", b"aside", b"base", b"basefont", b"blockquote", b"body",
            b"caption", b"center", b"col", b"colgroup", b"dd", b"details", b"dialog", b"dir",
            b"div", b"dl", b"dt", b"fieldset", b"figcaption", b"figure", b"footer", b"form",
            b"frame", b"frameset", b"head", b"header", b"hr", b"html", b"iframe", b"legend",
            b"li", b"link", b"main", b"menu", b"menuitem", b"nav", b"noframes", b"ol",
            b"optgroup", b"option", b"p", b"param", b"section", b"source", b"summary",
            b"table", b"tbody", b"td", b"tfoot", b"th", b"thead", b"title", b"tr", b"track",
            b"ul",
        ];

        BLOCK_TAGS.iter().any(|&t| self.eq_ignore_ascii_case(name, t))
    }

    #[inline]
    fn eq_ignore_ascii_case(&self, a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter()
            .zip(b.iter())
            .all(|(&x, &y)| x.to_ascii_lowercase() == y.to_ascii_lowercase())
    }

    #[inline]
    fn contains_bytes(&self, haystack: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() || haystack.len() < needle.len() {
            return false;
        }
        haystack
            .windows(needle.len())
            .any(|w| w == needle)
    }

    #[inline]
    fn contains_ci(&self, haystack: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() || haystack.len() < needle.len() {
            return false;
        }
        haystack
            .windows(needle.len())
            .any(|w| self.eq_ignore_ascii_case(w, needle))
    }
    /// Find end of current line (position of \n or EOF).
    fn find_line_end(&mut self) -> usize {
        while !self.cursor.is_eof() && !self.cursor.at(b'\n') {
            self.cursor.bump();
        }
        self.cursor.offset()
    }

    /// Parse a fenced code line after container matching.
    /// Called when we're inside a fenced code block and containers matched.
    /// The cursor is at the content position (past container indent).
    fn parse_fence_line_in_container(&mut self, events: &mut Vec<BlockEvent>) {
        let fence = self.fence_state.as_ref().unwrap();
        let fence_char = fence.fence_char;
        let fence_len = fence.fence_len;
        let fence_indent = fence.indent;

        let content_pos = self.cursor.offset();

        // Check for closing fence (allow up to 3 spaces of indent)
        let mut temp_cursor = self.cursor;
        let mut cols = 0usize;
        while cols < 3 {
            match temp_cursor.peek() {
                Some(b' ') => {
                    cols += 1;
                    temp_cursor.bump();
                }
                Some(b'\t') => {
                    let next_col = Self::tab_column(cols);
                    cols = next_col;
                    temp_cursor.bump();
                }
                _ => break,
            }
        }
        if temp_cursor.at(fence_char) {
            let mut closing_len = 0;

            while temp_cursor.at(fence_char) {
                closing_len += 1;
                temp_cursor.bump();
            }

            // Closing fence must be at least as long as opening
            if closing_len >= fence_len {
                // Check that rest of line is only spaces/tabs
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
        // Reset to content_pos and skip up to fence_indent columns of whitespace
        self.cursor = Cursor::new_at(self.input, content_pos);
        let (_skipped_cols, _skipped_bytes) = self.skip_indent_max(fence_indent);

        let code_start = self.cursor.offset();

        // Find end of line
        let line_end = match self.cursor.find_newline() {
            Some(pos) => code_start + pos,
            None => code_start + self.cursor.remaining(),
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
        events.push(BlockEvent::Code(Range::from_usize(code_start, content_end)));
    }

    /// Parse a paragraph line.
    fn parse_paragraph_line(&mut self, _line_start: usize, events: &mut Vec<BlockEvent>) {
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
            // Before starting a paragraph, close any orphaned lists (lists with no active item)
            self.close_orphaned_lists(events);
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

        // Extract link reference definitions from the start of this paragraph.
        let consumed_lines = self.extract_link_ref_defs();
        if consumed_lines > 0 {
            let drain_count = consumed_lines.min(self.paragraph_lines.len());
            self.paragraph_lines.drain(0..drain_count);
        }

        if self.paragraph_lines.is_empty() {
            return;
        }

        // Mark the current container as having content
        self.mark_container_has_content();

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

    /// Extract link reference definitions from the start of the current paragraph.
    /// Returns the number of paragraph lines consumed by definitions.
    fn extract_link_ref_defs(&mut self) -> usize {
        if self.paragraph_lines.is_empty() {
            return 0;
        }

        // Fast path: only attempt parsing if any line starts with up to 3 spaces then '['.
        let mut has_candidate = false;
        'lines: for range in &self.paragraph_lines {
            let line = range.slice(self.input);
            let mut i = 0usize;
            let mut spaces = 0u8;
            while i < line.len() {
                match line[i] {
                    b' ' => {
                        spaces += 1;
                        if spaces > 3 {
                            break;
                        }
                        i += 1;
                    }
                    b'\t' => {
                        // A leading tab exceeds the allowed 3-space indent for link ref defs.
                        break;
                    }
                    b'[' => {
                        has_candidate = true;
                        break 'lines;
                    }
                    _ => break,
                }
            }
        }
        if !has_candidate {
            return 0;
        }

        let mut total_len = self.paragraph_lines.len().saturating_sub(1); // newlines
        for range in &self.paragraph_lines {
            total_len += range.len() as usize;
        }
        let mut para = Vec::with_capacity(total_len);
        for (i, range) in self.paragraph_lines.iter().enumerate() {
            if i > 0 {
                para.push(b'\n');
            }
            para.extend_from_slice(range.slice(self.input));
        }

        let mut pos = 0usize;
        let mut consumed_lines = 0usize;
        let mut label_buf = String::new();

        loop {
            // Only parse at start of a line
            if pos > 0 && para[pos - 1] != b'\n' {
                break;
            }
            let Some((def, end_pos)) = parse_link_ref_def(&para, pos) else {
                break;
            };

            normalize_label_into(def.label.as_slice(), &mut label_buf);
            if label_buf.is_empty() {
                break;
            }
            let link_def = LinkRefDef {
                url: def.url,
                title: def.title,
            };
            self.link_refs.insert(label_buf.clone(), link_def);

            let newline_count = para[pos..end_pos].iter().filter(|&&b| b == b'\n').count();
            let ends_with_newline = end_pos > 0 && para.get(end_pos - 1) == Some(&b'\n');
            consumed_lines += if ends_with_newline {
                newline_count
            } else {
                newline_count + 1
            };
            pos = end_pos;

            if pos >= para.len() {
                break;
            }
        }

        consumed_lines
    }

    /// Mark the innermost container as having content.
    /// Used for the two-blank-line rule: list items that have had content
    /// are not closed by a blank line.
    fn mark_container_has_content(&mut self) {
        if let Some(container) = self.container_stack.last_mut() {
            container.has_content = true;
        }
    }

    /// Close any open lists that have no active list item.
    /// This can happen after the two-blank-line rule closes an item.
    fn close_orphaned_lists(&mut self, events: &mut Vec<BlockEvent>) {
        // Count active list items in container stack
        let active_items = self.container_stack.iter()
            .filter(|c| matches!(c.typ, ContainerType::ListItem { .. }))
            .count();

        // Close lists that have no corresponding item
        while self.open_lists.len() > active_items {
            if let Some(open_list) = self.open_lists.pop() {
                events.push(BlockEvent::ListEnd {
                    kind: open_list.kind,
                    tight: open_list.tight,
                });
            }
        }
    }
}

struct ParsedLinkRefDef {
    label: Vec<u8>,
    url: Vec<u8>,
    title: Option<Vec<u8>>,
}

fn parse_link_ref_def(input: &[u8], start: usize) -> Option<(ParsedLinkRefDef, usize)> {
    let len = input.len();
    let mut i = start;

    // Up to 3 leading spaces
    let mut spaces = 0usize;
    while i < len && input[i] == b' ' && spaces < 3 {
        i += 1;
        spaces += 1;
    }

    if i >= len || input[i] != b'[' {
        return None;
    }
    i += 1;

    // Parse label
    let label_start = i;
    while i < len {
        match input[i] {
            b'\\' => {
                if i + 1 < len {
                    i += 2;
                } else {
                    return None;
                }
            }
            b'[' => return None,
            b']' => break,
            _ => i += 1,
        }
    }
    if i >= len || input[i] != b']' {
        return None;
    }
    let label_end = i;
    i += 1;

    if i >= len || input[i] != b':' {
        return None;
    }
    i += 1;

    // Skip whitespace (allow a single line break)
    let mut saw_newline = false;
    while i < len {
        match input[i] {
            b' ' | b'\t' => i += 1,
            b'\n' => {
                if saw_newline {
                    return None;
                }
                saw_newline = true;
                i += 1;
            }
            _ => break,
        }
    }
    if i >= len {
        return None;
    }

    // Parse destination
    let (url_bytes, mut i) = if input[i] == b'<' {
        i += 1;
        let url_start = i;
        while i < len && input[i] != b'>' && input[i] != b'\n' {
            i += 1;
        }
        if i >= len || input[i] != b'>' {
            return None;
        }
        let url_end = i;
        i += 1;
        // After angle destination, must be whitespace or end of line
        if i < len && !matches!(input[i], b' ' | b'\t' | b'\n') {
            return None;
        }
        (input[url_start..url_end].to_vec(), i)
    } else {
        let url_start = i;
        let mut parens = 0i32;
        while i < len {
            let b = input[i];
            if b == b'\\' && i + 1 < len {
                i += 2;
                continue;
            }
            if b == b'(' {
                parens += 1;
                i += 1;
                continue;
            }
            if b == b')' {
                if parens == 0 {
                    break;
                }
                parens -= 1;
                i += 1;
                continue;
            }
            if is_whitespace(b) {
                break;
            }
            i += 1;
        }
        if url_start == i {
            return None;
        }
        (input[url_start..i].to_vec(), i)
    };

    let mut line_end = i;
    while line_end < len && input[line_end] != b'\n' {
        line_end += 1;
    }

    // Skip whitespace before title
    let mut j = i;
    let mut had_title_sep = false;
    let mut title_on_newline = false;
    while j < len && (input[j] == b' ' || input[j] == b'\t') {
        j += 1;
        had_title_sep = true;
    }
    if j < len && input[j] == b'\n' {
        j += 1;
        had_title_sep = true;
        title_on_newline = true;
        while j < len && (input[j] == b' ' || input[j] == b'\t') {
            j += 1;
        }
    }

    let mut title_bytes = None;
    if had_title_sep && j < len {
        let opener = input[j];
        let closer = match opener {
            b'"' => b'"',
            b'\'' => b'\'',
            b'(' => b')',
            _ => 0,
        };

        if closer != 0 {
            j += 1;
            let title_start = j;
            while j < len {
                let b = input[j];
                if b == b'\\' && j + 1 < len {
                    j += 2;
                    continue;
                }
                if b == b'\n' && j + 1 < len && input[j + 1] == b'\n' {
                    // Blank line not allowed in title
                    if title_on_newline {
                        return Some((
                            ParsedLinkRefDef {
                                label: input[label_start..label_end].to_vec(),
                                url: url_bytes,
                                title: None,
                            },
                            if line_end < len { line_end + 1 } else { line_end },
                        ));
                    }
                    return None;
                }
                if b == closer {
                    break;
                }
                j += 1;
            }
            if j >= len || input[j] != closer {
                // Not a valid title.
                if title_on_newline {
                    return Some((
                        ParsedLinkRefDef {
                            label: input[label_start..label_end].to_vec(),
                            url: url_bytes,
                            title: None,
                        },
                        if line_end < len { line_end + 1 } else { line_end },
                    ));
                }
                return None;
            }
            let title_end = j;
            j += 1;
            title_bytes = Some(input[title_start..title_end].to_vec());

            while j < len && (input[j] == b' ' || input[j] == b'\t') {
                j += 1;
            }
            if j < len && input[j] != b'\n' {
                // Invalid title (extra text).
                if title_on_newline {
                    return Some((
                        ParsedLinkRefDef {
                            label: input[label_start..label_end].to_vec(),
                            url: url_bytes,
                            title: None,
                        },
                        if line_end < len { line_end + 1 } else { line_end },
                    ));
                }
                return None;
            }
            i = j;
        }
    }

    // If no title, ensure remaining is only whitespace
    if title_bytes.is_none() {
        // Definition ends at end of destination line.
        i = line_end;
    }

    // Consume end of line
    if i < len && input[i] == b'\n' {
        i += 1;
    }

    Some((
        ParsedLinkRefDef {
            label: input[label_start..label_end].to_vec(),
            url: url_bytes,
            title: title_bytes,
        },
        i,
    ))
}

#[inline]
fn is_simple_line_start(b: u8) -> bool {
    !matches!(
        b,
        b'#'
            | b'>'
            | b'-'
            | b'*'
            | b'+'
            | b'`'
            | b'~'
            | b'<'
            | b'='
            | b'\n'
            | b'\r'
            | b'0'..=b'9'
    )
}

#[inline]
fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n')
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
        // `---` after paragraph text is a setext heading (h2), not a thematic break
        // For thematic break after paragraph, need a blank line
        let input = "Text\n\n---";
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
