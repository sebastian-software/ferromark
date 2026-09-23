use super::Parser;
use super::cursor::LineIndent;
use super::whitespace;

pub(super) struct ParsedListItem<'a> {
    pub(super) ordered: bool,
    /// Marker byte identifying the list flavor: the bullet (`-`, `*`,
    /// `+`) for unordered items, the delimiter (`.`, `)`) for ordered
    /// ones. Lists only continue across items with the same marker.
    pub(super) marker: u8,
    pub(super) start: Option<u32>,
    pub(super) content: &'a str,
    /// Source offset of `content[synthetic_indent..]`.
    pub(super) content_offset: usize,
    /// Leading spaces of `content` that stand for tab columns and have no
    /// source bytes of their own; zero unless a tab after the marker was
    /// expanded into spaces.
    pub(super) synthetic_indent: usize,
    pub(super) content_source_end: usize,
    /// Column (relative to the marker line's start) where continuation
    /// lines must be indented to belong to this item: marker indent +
    /// marker width + the columns of whitespace that follow, tabs
    /// expanded to a tab stop of four (one column when the item is empty
    /// or starts with indented code).
    pub(super) content_indent: usize,
    pub(super) checked: Option<bool>,
}

impl<'a> Parser<'a> {
    /// Checks whether a trimmed line opens a list item. Callers that
    /// already produced the trimmed line via `parse_block` /
    /// `line_starts_block` reuse that slice instead of re-scanning the
    /// source for a newline and re-running `trim_start`.
    pub(super) fn try_parse_list_line(trimmed: &str) -> bool {
        let bytes = trimmed.as_bytes();

        // Unordered: bullet followed by space/tab or end of line.
        if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
            return matches!(bytes.get(1), None | Some(b' ' | b'\t'));
        }

        // Ordered: up to nine digits, `.` or `)`, then space/tab or EOL.
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        (1..=9).contains(&i)
            && matches!(bytes.get(i), Some(b'.' | b')'))
            && matches!(bytes.get(i + 1), None | Some(b' ' | b'\t'))
    }

    /// Variant for paragraph interruption: only non-empty items can
    /// interrupt a paragraph, and ordered ones only when numbered 1.
    pub(super) fn try_parse_list_interrupt(trimmed: &str) -> bool {
        let bytes = trimmed.as_bytes();
        if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
            return matches!(bytes.get(1), Some(b' ' | b'\t'))
                && !whitespace::is_blank(&trimmed[1..]);
        }
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        i > 0
            && trimmed[..i] == *"1"
            && matches!(bytes.get(i), Some(b'.' | b')'))
            && matches!(bytes.get(i + 1), Some(b' ' | b'\t'))
            && !whitespace::is_blank(&trimmed[i + 1..])
    }

    /// Calculates the indentation level (number of spaces) of the current line.
    ///
    /// A tab counts as a flat four columns here, wherever it sits — see
    /// [`LineIndent::flat_columns`](super::cursor::LineIndent), which
    /// produces the same count for callers that walk the line anyway.
    pub(super) fn calc_indentation(&self, start: usize) -> usize {
        let mut indent = 0;
        let bytes = self.source.as_bytes();
        for byte in bytes.iter().skip(start) {
            match byte {
                b' ' => indent += 1,
                b'\t' => indent += 4, // Assume tab is 4 spaces
                _ => break,
            }
        }
        indent
    }

    pub(super) fn parse_task_list_prefix(&self, content: &'a str) -> Option<(bool, usize)> {
        if !self.options.task_lists || content.len() < 3 {
            return None;
        }

        if (content.starts_with("[x]") || content.starts_with("[X]"))
            && (content.len() == 3 || content.starts_with("[x] ") || content.starts_with("[X] "))
        {
            return Some((true, usize::from(content.len() > 3) + 3));
        }

        if content.starts_with("[ ]") && (content.len() == 3 || content.starts_with("[ ] ")) {
            return Some((false, usize::from(content.len() > 3) + 3));
        }

        None
    }

    /// The list item a whole `line` opens, trimmed the way the list walk
    /// used to trim it before its line facts carried the trimmed slice.
    /// Only the per-line reference walk the tests compare against still
    /// calls it.
    #[cfg(test)]
    pub(super) fn parse_list_item_line_from_line(
        &self,
        line_start: usize,
        line: &'a str,
    ) -> Option<ParsedListItem<'a>> {
        let trimmed = whitespace::trim_start(line);
        self.parse_list_item_line_from_trimmed(line_start, line, trimmed)
    }

    /// `line` must come from the parser's own line scanner: `str::lines()`
    /// recognizes LF and CRLF but not a lone CR, so a sibling after a CR
    /// blank line has to stay limited to its own source line rather than
    /// swallowing the rest of the list. `trimmed` is the line's tail from
    /// its first content byte, which each caller has already located.
    pub(super) fn parse_list_item_line_from_trimmed(
        &self,
        line_start: usize,
        line: &'a str,
        trimmed: &'a str,
    ) -> Option<ParsedListItem<'a>> {
        let trimmed_offset = line_start + (line.len() - trimmed.len());
        let bytes = trimmed.as_bytes();

        let (ordered, marker, marker_width, start) =
            if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
                (false, bytes[0], 1, None)
            } else {
                let mut digits = 0;
                while digits < bytes.len() && bytes[digits].is_ascii_digit() {
                    digits += 1;
                }
                if !(1..=9).contains(&digits) || !matches!(bytes.get(digits), Some(b'.' | b')')) {
                    return None;
                }
                (
                    true,
                    bytes[digits],
                    digits + 1,
                    trimmed[..digits].parse().ok(),
                )
            };

        // Content begins after 1–4 spaces (or a tab). More than four
        // spaces means the item starts with indented code: content is
        // taken to start one column after the marker, keeping the extra
        // spaces. A bare marker at end of line is an empty item.
        let after_marker = &bytes[marker_width..];
        let spaces = after_marker
            .iter()
            .take_while(|&&byte| byte == b' ')
            .count();
        let content_skip = match after_marker.first() {
            None => 0,
            Some(b'\t') => 1,
            Some(b' ') if spaces <= 4 && spaces < after_marker.len() => spaces,
            Some(b' ') if spaces >= after_marker.len() => spaces, // marker + trailing blanks
            Some(b' ') => 1,
            Some(_) => return None,
        };

        let marker_indent = line.len() - trimmed.len();
        let ws_run = after_marker
            .iter()
            .take_while(|&&byte| matches!(byte, b' ' | b'\t'))
            .count();
        if after_marker[..ws_run].contains(&b'\t') {
            // Tabs after the marker expand to a tab stop of four columns
            // from the marker's original column (CommonMark 0.31.2
            // section 2.2).
            let marker_end_col = marker_indent + marker_width;
            let mut end_col = marker_end_col;
            for &byte in &after_marker[..ws_run] {
                end_col = if byte == b'\t' {
                    (end_col / 4 + 1) * 4
                } else {
                    end_col + 1
                };
            }
            let rest = &trimmed[marker_width + ws_run..];
            // Section 5.2: one to four columns of separation start the
            // content at the column the whitespace run ends on, so a tab
            // after `-` in column 0 puts the content at column 4 and
            // continuation lines need that much indentation.
            //
            // Only the document's own parser can measure that column. A
            // container re-parses its content with the prefix stripped, so
            // column 0 of `line` is not column 0 of the source line and the
            // width of a tab is no longer recoverable: the tab in `> -\tfoo`
            // is one column wide, the one in `-\tfoo` three. A sub-source
            // therefore keeps the narrowest reading a tab can have, one
            // separating column, which never splits an item that the real
            // column would hold together. Whether the run is wide enough for
            // indented code does not depend on the starting column — one tab
            // always lands one to four columns on, two always five or more —
            // so the branch below is the same either way.
            if self.nesting_depth == 0
                && end_col - marker_end_col <= 4
                && !whitespace::is_blank(rest)
            {
                return Some(ParsedListItem {
                    ordered,
                    marker,
                    start,
                    content: rest,
                    content_offset: trimmed_offset + marker_width + ws_run,
                    synthetic_indent: 0,
                    content_source_end: line_start + line.len(),
                    content_indent: end_col,
                    checked: None,
                });
            }
            // Five or more columns (or no content at all) mean the item
            // starts with indented code: the content starts one column
            // after the marker and everything beyond it becomes content
            // spaces so alignment survives the item re-parse (`-\t\tfoo`
            // is an item holding two-space-indented code).
            let extra_columns = end_col.saturating_sub(marker_end_col + 1);
            let mut expanded = self.allocator.new_string();
            for _ in 0..extra_columns {
                expanded.push(' ');
            }
            expanded.push_str(rest);
            let content: &'a str = expanded.into_bump_str();
            return Some(ParsedListItem {
                ordered,
                marker,
                start,
                content,
                content_offset: trimmed_offset + marker_width + 1,
                // The whitespace after the first separating byte stands for
                // `extra_columns` spaces. Each of its `ws_run - 1` bytes is
                // at least one column wide, so the rest of the spaces are
                // made up.
                synthetic_indent: extra_columns.saturating_sub(ws_run - 1),
                content_source_end: line_start + line.len(),
                content_indent: marker_end_col + 1,
                checked: None,
            });
        }

        let mut content = &trimmed[marker_width + content_skip..];
        let mut content_offset = trimmed_offset + marker_width + content_skip;
        // Continuation indent counts the marker's own indent plus the
        // marker and its separating spaces; empty items count one column.
        let content_indent = marker_indent
            + marker_width
            + if whitespace::is_blank(content) {
                1
            } else {
                content_skip.max(1)
            };
        let mut checked = None;

        if let Some((done, consumed)) = self.parse_task_list_prefix(content) {
            checked = Some(done);
            content = &content[consumed..];
            content_offset += consumed;
        }

        Some(ParsedListItem {
            ordered,
            marker,
            start,
            content,
            content_offset,
            synthetic_indent: 0,
            content_source_end: line_start + line.len(),
            content_indent,
            checked,
        })
    }

    /// [`Self::push_line_without_indent`] for a line whose leading run the
    /// caller's line walk has already measured as `indent`.
    ///
    /// A run without tabs needs no second walk: its columns are its bytes,
    /// so the dedent is a slice and the padding a count. A run with a tab
    /// takes the column walk.
    pub(super) fn push_measured_line_without_indent(
        out: &mut crate::allocator::String<'a>,
        line: &str,
        indent: &LineIndent,
        columns: usize,
    ) -> usize {
        if indent.has_tab {
            return Self::push_line_without_indent(out, line, columns);
        }
        for _ in columns..indent.bytes {
            out.push(' ');
        }
        out.push_str(&line[indent.bytes..]);
        indent.bytes.min(columns)
    }

    /// Pushes `line` minus its first `columns` columns onto `out`,
    /// expanding the whole leading whitespace run to spaces so tab stops
    /// keep their original alignment through item/quote re-parsing.
    pub(super) fn push_line_without_indent(
        out: &mut crate::allocator::String<'a>,
        line: &str,
        columns: usize,
    ) -> usize {
        let bytes = line.as_bytes();
        let mut col = 0usize;
        let mut i = 0usize;
        let mut source_start = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b' ' => {
                    col += 1;
                    i += 1;
                    if col <= columns {
                        source_start = i;
                    }
                }
                b'\t' => {
                    let next_col = (col / 4 + 1) * 4;
                    if next_col <= columns {
                        source_start = i + 1;
                    } else if col < columns {
                        source_start = i;
                    }
                    col = next_col;
                    i += 1;
                }
                _ => break,
            }
        }
        for _ in columns..col {
            out.push(' ');
        }
        out.push_str(&line[i..]);
        source_start
    }

    pub(super) fn strip_indent_columns(line: &str, columns: usize) -> &str {
        let mut consumed = 0;
        let mut byte_index = 0;

        for ch in line.chars() {
            let width = if ch == ' ' {
                1
            } else if ch == '\t' {
                4
            } else {
                break;
            };

            if consumed + width > columns {
                break;
            }

            consumed += width;
            byte_index += ch.len_utf8();
        }

        &line[byte_index..]
    }
}
