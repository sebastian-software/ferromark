//! Inline code spans (CommonMark "Code spans").
//!
//! The opener is a run of backticks; the closer is the next run of exactly
//! the same length. Everything in between is literal, so this is one of the
//! few inline constructs that can be skipped over in bulk rather than
//! walked byte by byte.

use crate::allocator::Vec;
use crate::ast::{Node, Span};
use memchr::{memchr2, memchr3};
use smallvec::SmallVec;

use super::super::Parser;
use super::super::line_scan::{is_line_ending_byte, line_terminator_end};

/// Stack-local tracker of backtick run lengths proven to have no closing run
/// ahead in the current inline slice, preventing superlinear rescanning on
/// unmatched backtick runs.
#[derive(Default)]
pub(in crate::parser) struct BacktickDeadEnds {
    /// Bit `len - 1` is set when no closing run of length `len` (`1..=64`)
    /// exists ahead.
    short_mask: u64,
    /// Unmatched run lengths `> 64`.
    long: SmallVec<[usize; 4]>,
    /// True once a scan reached the end without encountering any backtick at all.
    no_more_ticks: bool,
}

impl BacktickDeadEnds {
    #[inline]
    pub(in crate::parser) fn is_dead(&self, run_len: usize) -> bool {
        if self.no_more_ticks {
            return true;
        }
        if run_len <= 64 {
            (self.short_mask & (1u64 << (run_len - 1))) != 0
        } else {
            self.long.contains(&run_len)
        }
    }

    #[inline]
    pub(in crate::parser) fn mark_dead(&mut self, run_len: usize, saw_any_tick: bool) {
        if !saw_any_tick {
            self.no_more_ticks = true;
            return;
        }
        if run_len <= 64 {
            self.short_mask |= 1u64 << (run_len - 1);
        } else if !self.long.contains(&run_len) {
            self.long.push(run_len);
        }
    }
}

impl<'a> Parser<'a> {
    pub(super) fn parse_inline_code(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
        dead_backticks: &mut BacktickDeadEnds,
    ) {
        let bytes = content.as_bytes();
        let open_len = Self::marker_run_len(bytes, *pos, b'`');
        let code_start = *pos + open_len;

        if dead_backticks.is_dead(open_len) {
            Self::push_text(
                children,
                &content[*pos..code_start],
                offset + *pos,
                offset + code_start,
            );
            *pos = code_start;
            return;
        }

        // The closer is the next backtick run of exactly the opener's
        // length (CommonMark "Code spans"). memchr jumps between runs, and
        // searching for the line ending in the same pass answers the question
        // `normalize_code_span` would otherwise re-scan the whole span for.
        let mut cursor = code_start;
        let mut has_newline = false;
        let mut saw_any_tick = false;
        while cursor < bytes.len() {
            let Some(off) = memchr3(b'`', b'\n', b'\r', &bytes[cursor..]) else {
                break;
            };
            cursor += off;
            if is_line_ending_byte(bytes[cursor]) {
                has_newline = true;
                cursor = line_terminator_end(bytes, cursor);
                continue;
            }
            saw_any_tick = true;
            let run = Self::marker_run_len(bytes, cursor, b'`');
            if run == open_len {
                let span = Span::new((offset + *pos) as u32, (offset + cursor + run) as u32);
                children.push(Node::InlineCode(crate::ast::InlineCode {
                    value: self.normalize_code_span(&content[code_start..cursor], has_newline),
                    span,
                }));
                *pos = cursor + run;
                return;
            }
            cursor += run;
        }

        dead_backticks.mark_dead(open_len, saw_any_tick);

        // No closer: the opening run is literal text.
        Self::push_text(
            children,
            &content[*pos..code_start],
            offset + *pos,
            offset + code_start,
        );
        *pos = code_start;
    }

    /// Applies the code span content rules: line endings become spaces,
    /// and one leading plus one trailing space is dropped when the content
    /// starts and ends with a space without being all spaces.
    ///
    /// `has_newline` comes from the closer scan, which already walked these
    /// bytes; re-deriving it here cost 1-3% of a parse because almost every
    /// code span is single-line and paid a whole extra scan to prove it.
    fn normalize_code_span(&self, raw: &'a str, has_newline: bool) -> &'a str {
        let value: &'a str = if has_newline {
            let mut converted =
                crate::allocator::String::with_capacity_in(raw.len(), self.allocator.bump());
            let bytes = raw.as_bytes();
            let mut start = 0usize;
            let mut cursor = 0usize;
            while let Some(rel) = memchr2(b'\r', b'\n', &bytes[cursor..]) {
                let at = cursor + rel;
                converted.push_str(&raw[start..at]);
                converted.push(' ');
                cursor = line_terminator_end(bytes, at);
                start = cursor;
            }
            converted.push_str(&raw[start..]);
            converted.into_bump_str()
        } else {
            raw
        };

        let stripped = value.starts_with(' ')
            && value.ends_with(' ')
            && value.len() >= 2
            && value.bytes().any(|byte| byte != b' ');
        if stripped {
            &value[1..value.len() - 1]
        } else {
            value
        }
    }
}
