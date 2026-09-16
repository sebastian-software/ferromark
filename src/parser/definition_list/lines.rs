use super::super::line_scan::line_end;
use super::Parser;

pub(super) struct DefinitionBodyLine {
    pub(super) body_start: usize,
    pub(super) body_end: usize,
}

impl Parser<'_> {
    pub(super) fn line_end_at(&self, line_start: usize) -> usize {
        line_end(self.source.as_bytes(), line_start)
    }

    pub(super) fn is_blank_line_at(&self, line_start: usize) -> bool {
        self.line_at(line_start).trim().is_empty()
    }

    pub(super) fn skip_blank_lines_from(&self, mut cursor: usize) -> usize {
        while cursor < self.source.len() {
            // One scan answers "is this line blank" and "where does the next
            // one start", instead of the line being searched twice per step.
            let (line, next) = self.line_and_next(cursor);
            if !line.trim().is_empty() && !self.is_line_comment_at(cursor) {
                break;
            }
            cursor = next;
        }
        cursor
    }
}

pub(super) fn trim_line_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut trimmed_start = start;
    while trimmed_start < end && matches!(bytes[trimmed_start], b' ' | b'\t') {
        trimmed_start += 1;
    }
    let mut trimmed_end = end;
    while trimmed_end > trimmed_start && matches!(bytes[trimmed_end - 1], b' ' | b'\t') {
        trimmed_end -= 1;
    }
    (trimmed_start, trimmed_end)
}

pub(super) fn has_unclosed_inline_code(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let Some(relative) = memchr::memchr(b'`', &bytes[cursor..]) else {
            return false;
        };
        let start = cursor + relative;
        let ticks = bytes[start..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        let mut search = start + ticks;
        let mut closed = false;
        while search < bytes.len() {
            if bytes[search] != b'`' {
                search += 1;
                continue;
            }
            let close = bytes[search..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if close >= ticks {
                cursor = search + close;
                closed = true;
                break;
            }
            search += close;
        }
        if !closed {
            return true;
        }
    }
    false
}
