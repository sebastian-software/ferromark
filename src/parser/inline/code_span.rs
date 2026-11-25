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
/// unmatched backtick runs (including inputs with many distinct run lengths).
#[derive(Default)]
pub(in crate::parser) struct BacktickDeadEnds {
    /// Bit `len - 1` is set when no closing run of length `len` (`1..=64`)
    /// can exist for a future opener.
    short_mask: u64,
    /// Once a failed scan reached the end of the slice, sorted deduplicated
    /// run lengths `> 64` that appeared at least twice ahead. Any `> 64` width
    /// absent from this list cannot both open and close a code span later.
    long_alive: Option<SmallVec<[usize; 4]>>,
    /// True once a scan reached the end without encountering any backtick at all.
    no_more_ticks: bool,
    /// Counts full suffix scans to EOF (used by unit tests to assert O(1) scans).
    #[cfg(test)]
    eof_scans: usize,
}

impl BacktickDeadEnds {
    #[inline]
    pub(in crate::parser) fn is_dead(&self, run_len: usize) -> bool {
        if self.no_more_ticks {
            return true;
        }
        if run_len <= 64 {
            (self.short_mask & (1u64 << (run_len - 1))) != 0
        } else if let Some(alive) = &self.long_alive {
            alive.binary_search(&run_len).is_err()
        } else {
            false
        }
    }

    #[inline]
    pub(in crate::parser) fn note_run(
        run: usize,
        preceded_by_backslash: bool,
        seen_short_once: &mut u64,
        seen_short_twice: &mut u64,
        seen_long: &mut SmallVec<[usize; 8]>,
    ) {
        if run <= 64 {
            let bit = 1u64 << (run - 1);
            *seen_short_twice |= *seen_short_once & bit;
            *seen_short_once |= bit;
        } else {
            seen_long.push(run);
        }
        // A backslash immediately before a run of `run >= 2` backticks can
        // escape the first backtick in outer inline/delimiter scanning,
        // leaving an opener of length `run - 1`. Record `run - 1` so any
        // single actual closer of width `run - 1` ahead stays alive.
        if preceded_by_backslash && run >= 2 {
            let split_len = run - 1;
            if split_len <= 64 {
                let bit = 1u64 << (split_len - 1);
                *seen_short_twice |= *seen_short_once & bit;
                *seen_short_once |= bit;
            } else {
                seen_long.push(split_len);
            }
        }
    }

    #[inline]
    pub(in crate::parser) fn record_eof_scan(
        &mut self,
        open_len: usize,
        saw_any_tick: bool,
        seen_short_twice: u64,
        mut seen_long: SmallVec<[usize; 8]>,
    ) {
        #[cfg(test)]
        {
            self.eof_scans += 1;
        }
        if !saw_any_tick {
            self.no_more_ticks = true;
            return;
        }
        // Any width 1..=64 seen fewer than twice ahead cannot both open and
        // close a later code span (and `open_len` itself was seen 0 times).
        self.short_mask |= !seen_short_twice;
        if open_len <= 64 {
            self.short_mask |= 1u64 << (open_len - 1);
        }
        // Keep only `> 64` widths that appeared at least twice ahead,
        // excluding `open_len` itself and intersecting with any prior
        // `long_alive` set so previously excluded widths can never be revived
        // by backslash-split runs in a later scan.
        let mut long_twice: SmallVec<[usize; 4]> = SmallVec::new();
        if seen_long.len() >= 2 {
            seen_long.sort_unstable();
            let mut idx = 0usize;
            while idx + 1 < seen_long.len() {
                let width = seen_long[idx];
                let still_allowed = self
                    .long_alive
                    .as_ref()
                    .is_none_or(|prev| prev.binary_search(&width).is_ok());
                if width != open_len && still_allowed && seen_long[idx + 1] == width {
                    long_twice.push(width);
                    while idx < seen_long.len() && seen_long[idx] == width {
                        idx += 1;
                    }
                } else {
                    idx += 1;
                }
            }
        }
        self.long_alive = Some(long_twice);
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
        let mut seen_short_once = 0u64;
        let mut seen_short_twice = 0u64;
        let mut seen_long: SmallVec<[usize; 8]> = SmallVec::new();
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
            BacktickDeadEnds::note_run(
                run,
                cursor > 0 && bytes[cursor - 1] == b'\\',
                &mut seen_short_once,
                &mut seen_short_twice,
                &mut seen_long,
            );
            cursor += run;
        }

        dead_backticks.record_eof_scan(open_len, saw_any_tick, seen_short_twice, seen_long);

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

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use super::BacktickDeadEnds;
    use crate::allocator::Allocator;
    use crate::ast::Node;
    use crate::parser::Parser;

    #[test]
    fn distinct_unmatched_backtick_widths_scan_to_eof_only_once() {
        let source_arena = Allocator::new();
        let mut source = source_arena.new_string();
        for width in 1..=128 {
            if width > 1 {
                source.push(' ');
            }
            for _ in 0..width {
                source.push('`');
            }
        }
        let source = source.into_bump_str();
        let allocator = Allocator::for_source_len(source.len());
        let parser = Parser::new(&allocator, source);
        let mut children = allocator.new_vec();
        let mut dead = BacktickDeadEnds::default();
        let mut pos = 0usize;
        while pos < source.len() {
            if source.as_bytes()[pos] == b'`' {
                parser.parse_inline_code(source, 0, &mut children, &mut pos, &mut dead);
            } else {
                pos += 1;
            }
        }
        assert_eq!(
            dead.eof_scans, 1,
            "all 128 distinct unmatched backtick runs should trigger exactly one suffix scan"
        );
        assert!(children.iter().all(|node| matches!(node, Node::Text(_))));
    }

    #[test]
    fn dead_end_tracker_keeps_paired_short_and_long_backticks_alive() {
        let source_arena = Allocator::new();
        let mut source = source_arena.new_string_from("``` ``x`` ");
        for _ in 0..70 {
            source.push('`');
        }
        source.push('y');
        for _ in 0..70 {
            source.push('`');
        }
        let source = source.into_bump_str();
        let allocator = Allocator::for_source_len(source.len());
        let parser = Parser::new(&allocator, source);
        let mut children = allocator.new_vec();
        let mut dead = BacktickDeadEnds::default();
        let mut pos = 0usize;
        while pos < source.len() {
            if source.as_bytes()[pos] == b'`' {
                parser.parse_inline_code(source, 0, &mut children, &mut pos, &mut dead);
            } else {
                pos += 1;
            }
        }
        assert_eq!(dead.eof_scans, 1);
        let codes: SmallVec<[&str; 4]> = children
            .iter()
            .filter_map(|node| match node {
                Node::InlineCode(code) => Some(code.value),
                _ => None,
            })
            .collect();
        assert_eq!(codes.as_slice(), ["x", "y"]);
    }

    #[test]
    fn dead_end_tracker_preserves_backslash_split_opener() {
        let source = "``` \\``foo`";
        let allocator = Allocator::for_source_len(source.len());
        let doc = Parser::new(&allocator, source)
            .parse()
            .expect("parse should succeed");
        let Node::Paragraph(p) = &doc.children[0] else {
            panic!("expected paragraph");
        };
        assert!(
            p.children
                .iter()
                .any(|node| matches!(node, Node::InlineCode(code) if code.value == "foo")),
            "backslash-split backtick opener should still close on trailing single backtick: {:?}",
            p.children
        );
    }

    #[test]
    fn failed_long_width_stays_dead_despite_backslash_split_runs() {
        let source_arena = Allocator::new();
        let mut source = source_arena.new_string();
        for _ in 0..65 {
            source.push('`');
        }
        source.push(' ');
        for _ in 0..67 {
            source.push('`');
        }
        for _ in 0..2 {
            source.push_str(" \\");
            for _ in 0..66 {
                source.push('`');
            }
            source.push_str(" \\");
            for _ in 0..68 {
                source.push('`');
            }
        }
        let source = source.into_bump_str();
        let allocator = Allocator::for_source_len(source.len());
        let parser = Parser::new(&allocator, source);
        let mut children = allocator.new_vec();
        let mut dead = BacktickDeadEnds::default();
        let mut pos = 0usize;
        // First failed scan for width 65 excludes 65 while keeping 67 alive
        // (because `\``x68 appeared twice).
        parser.parse_inline_code(source, 0, &mut children, &mut pos, &mut dead);
        assert_eq!(dead.eof_scans, 1);
        assert!(dead.is_dead(65));
        assert!(!dead.is_dead(67));
        // Second failed scan for width 67 must NOT revive 65 even though
        // `\``x66 still appears twice ahead.
        pos += 1; // skip space
        parser.parse_inline_code(source, 0, &mut children, &mut pos, &mut dead);
        assert_eq!(dead.eof_scans, 2);
        assert!(
            dead.is_dead(65) && dead.is_dead(67),
            "second failed long scan must preserve prior exclusion of width 65"
        );
    }
}
