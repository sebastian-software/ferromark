//! Physical source-line comments, independent of Markdown container prefixes.

use super::Parser;
use super::line_scan::next_line_start;
use super::spans::SourceMap;

pub(super) struct CommentDefinitionRegion<'a> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) text: &'a str,
    pub(super) map: Option<SourceMap>,
}

impl<'a> Parser<'a> {
    pub(super) fn is_line_comment_at(&self, start: usize) -> bool {
        if !self.options.line_comments {
            return false;
        }
        if let Some(lines) = &self.comment_lines {
            return lines.contains(&(start as u32));
        }
        let bytes = self.source.as_bytes();
        if start > 0 && !matches!(bytes.get(start - 1), Some(b'\n' | b'\r')) {
            return false;
        }
        let mut marker = start;
        while marker < bytes.len() && marker - start < 3 && bytes[marker] == b' ' {
            marker += 1;
        }
        bytes.get(marker..).is_some_and(|rest| rest.starts_with(b"//"))
    }

    /// Carry physical eligibility through dedenting without turning `> //`
    /// or `- //` into comments. The nested block parser decides whether an
    /// eligible line is Markdown or belongs to an opaque code/HTML block.
    pub(super) fn sub_parser_with_source_map(
        &self,
        source: &'a str,
        lazy_lines: rustc_hash::FxHashSet<u32>,
        map: &SourceMap,
    ) -> Parser<'a> {
        let mut parser = self.sub_parser_with_lazy_lines(source, lazy_lines);
        if self.options.line_comments {
            let lines: rustc_hash::FxHashSet<u32> = map
                .line_origins()
                .filter(|&(_, original)| {
                    let start = memchr::memrchr2(b'\n', b'\r', &self.source.as_bytes()[..original])
                        .map_or(0, |newline| newline + 1);
                    self.is_line_comment_at(start)
                })
                .map(|(generated, _)| generated as u32)
                .collect();
            parser.options.line_comments = !lines.is_empty();
            parser.comment_lines = (!lines.is_empty()).then(|| std::rc::Rc::new(lines));
        }
        parser
    }

    /// Call only in Markdown contexts; code/HTML consumers use physical lines.
    pub(super) fn skip_line_comments_from(&self, mut start: usize) -> usize {
        while self.is_line_comment_at(start) {
            start = next_line_start(self.source.as_bytes(), start);
        }
        start
    }

    /// Join a paragraph's surviving physical lines, preserving original spans.
    /// The common path borrows the source without allocating a string or map.
    pub(super) fn without_line_comments(
        &self,
        start: usize,
        end: usize,
    ) -> (&'a str, Option<SourceMap>) {
        if !self.options.line_comments {
            return (&self.source[start..end], None);
        }
        let bytes = self.source.as_bytes();
        let mut cursor = start;
        while cursor < end && !self.is_line_comment_at(cursor) {
            cursor = next_line_start(bytes, cursor);
        }
        if cursor >= end {
            return (&self.source[start..end], None);
        }
        let mut text = self.allocator.new_string();
        let mut map = SourceMap::default();
        cursor = start;
        while cursor < end {
            let next = next_line_start(bytes, cursor).min(end);
            if !self.is_line_comment_at(cursor) {
                map.push_line(text.len(), next - cursor, cursor, next - cursor);
                text.push_str(&self.source[cursor..next]);
            }
            cursor = next;
        }
        (text.into_bump_str(), Some(map))
    }
}
