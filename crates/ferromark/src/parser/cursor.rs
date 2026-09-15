use memchr::{memchr, memchr3};

use super::Parser;
use super::line_scan::{
    is_line_ending_byte, line_end, line_terminator_end, next_line_start as scan_next_line_start,
};

/// What [`Parser::probe_line`] learned about the current line.
pub(super) struct BlockProbe {
    /// Whether the line begins a block-level construct.
    pub starts_block: bool,
    /// Offset of the line's newline, or `source.len()` for an unterminated
    /// final line — but only when the probe happened to reach it. `None`
    /// means the caller has to find the line end itself.
    pub line_end: Option<usize>,
}

impl<'a> Parser<'a> {
    pub(super) fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Peeks at the current character.
    #[inline]
    pub(super) fn peek(&self) -> Option<char> {
        // ASCII fast path: most parser hot loops only inspect ASCII bytes
        // (`#`, `\``, `~`, `>`, digits, whitespace), so avoid the cost of
        // constructing a `Chars` iterator and decoding UTF-8 when the
        // current byte is plain ASCII.
        let bytes = self.source.as_bytes();
        let pos = self.position;
        let &b = bytes.get(pos)?;
        if b < 0x80 {
            Some(b as char)
        } else {
            self.source[pos..].chars().next()
        }
    }

    /// Advances by one character.
    #[inline]
    pub(super) fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Skips whitespace characters.
    pub(super) fn skip_whitespace(&mut self) {
        let bytes = self.source.as_bytes();
        let mut pos = self.position;
        while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
            pos += 1;
        }
        self.position = pos;
    }

    /// Skips blank lines and reports the first non-space, non-tab byte of
    /// the line it settles on.
    ///
    /// Finding that byte is the walk's own stopping condition, so returning
    /// it saves the caller a second pass over the same indentation — the
    /// answer `first_non_whitespace_in_line` would recompute from the
    /// rewound position. `None` means the input ended in whitespace.
    pub(super) fn skip_blank_lines(&mut self) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let mut pos = self.position;
        loop {
            let line_start = pos;
            // Skip spaces/tabs.
            while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
                pos += 1;
            }
            if pos < bytes.len() && is_line_ending_byte(bytes[pos]) {
                pos = line_terminator_end(bytes, pos);
            } else if pos >= bytes.len() {
                // Spaces/tabs running to end of input with no newline are
                // a blank final line: consume them. Rewinding instead
                // would park the position on whitespace that no block
                // parser consumes, and the block loop — which runs until
                // `is_at_end` — would spin forever.
                self.position = pos;
                return None;
            } else {
                // Content follows the indentation; rewind so the caller
                // still sees the leading whitespace (indented code and
                // list markers depend on it).
                self.position = line_start;
                return Some(pos);
            }
        }
    }

    /// Returns true when the current line begins a block-level construct.
    ///
    /// This is the paragraph-continuation counterpart of `parse_block`'s
    /// first-byte dispatcher. Paragraph parsing calls it for each following
    /// line, so it deliberately avoids `current_line().trim_start()` unless
    /// the leading marker byte makes a block parse plausible. Keeping this
    /// byte-dispatch table aligned with `parse_block` preserves Markdown
    /// behavior while avoiding repeated full-line scans on ordinary prose.
    pub(super) fn line_starts_block(&self) -> bool {
        let line_start = self.position;
        let Some(trimmed_start) = self.first_non_whitespace_in_line(line_start) else {
            return false;
        };
        self.probe_line(line_start, trimmed_start).starts_block
    }

    /// [`Self::line_starts_block`] for a caller that has already found the
    /// line's first non-whitespace byte, and that also wants to know where
    /// the line ends.
    ///
    /// Whatever the probe touches, it reports where the line ended if it
    /// found out — the dispatch arm that sliced the line, or the table
    /// guard's own `memchr3`, which for ordinary prose comes back with the
    /// terminator. That is the very next thing paragraph parsing asks, so
    /// returning the offset lets the paragraph loop consume the line without
    /// a second scan over the same bytes.
    pub(super) fn probe_line(&self, line_start: usize, trimmed_start: usize) -> BlockProbe {
        self.probe_line_inner(line_start, trimmed_start, self.options.tables)
    }

    /// [`Self::probe_line`] without the table check.
    ///
    /// A table's own body rows are the one place the check is pure waste:
    /// consecutive rows always belong to the same table, yet every one of
    /// them re-ran the two-line header/delimiter probe — peeking both lines,
    /// counting the header's cells, and validating a delimiter row that is
    /// really the next data row — only to answer "no".
    pub(super) fn probe_line_without_table(
        &self,
        line_start: usize,
        trimmed_start: usize,
    ) -> BlockProbe {
        self.probe_line_inner(line_start, trimmed_start, false)
    }

    fn probe_line_inner(
        &self,
        line_start: usize,
        trimmed_start: usize,
        check_tables: bool,
    ) -> BlockProbe {
        let bytes = self.source.as_bytes();

        // A line indented four or more columns cannot start any block, so
        // it can never interrupt a paragraph either (lazy continuation).
        if self.line_indent_width(line_start, trimmed_start) >= 4 {
            return BlockProbe {
                starts_block: false,
                line_end: None,
            };
        }

        // Arms that materialize the line have already found its terminator.
        // Recording that offset keeps the table probe and the paragraph loop
        // from searching the same bytes for it again.
        let mut line_end_hint = None;

        let starts_block = match bytes[trimmed_start] {
            b'#' => self.try_parse_heading_start(line_start, trimmed_start),
            b'-' | b'*' => {
                let line = self.line_at(line_start);
                line_end_hint = Some(line_start + line.len());
                let trimmed = &line[trimmed_start - line_start..];
                Self::try_parse_thematic_break_line(line) || Self::try_parse_list_interrupt(trimmed)
            }
            b'_' => {
                let line = self.line_at(line_start);
                line_end_hint = Some(line_start + line.len());
                Self::try_parse_thematic_break_line(line)
            }
            b'>' => true,
            b'`' | b'~' => {
                let line = self.line_at(line_start);
                line_end_hint = Some(line_start + line.len());
                let trimmed = &line[trimmed_start - line_start..];
                Self::try_parse_fenced_code_at(line, trimmed)
            }
            b'$' if self.options.math => {
                let line = self.line_at(line_start);
                line_end_hint = Some(line_start + line.len());
                let trimmed = &line[trimmed_start - line_start..];
                self.try_parse_math_block_at(line_start, line, trimmed)
            }
            b':' => self.starts_definition_body_at(line_start),
            b'{' => {
                // `looks_like_flow_expression` walks to the end of the
                // source to report that nothing closed, so a run of lines
                // starting with an unclosed `{` would pay one walk each.
                self.options.mdx
                    && self.has_closer_from(self.source, trimmed_start + 1, b'}')
                    && super::mdx_jsx::looks_like_flow_expression(self.source, trimmed_start)
            }
            b'<' => {
                let bytes = self.source.as_bytes();
                if self.options.mdx && super::mdx_jsx::looks_like_jsx_open(bytes, trimmed_start) {
                    true
                } else {
                    let line = self.line_at(line_start);
                    line_end_hint = Some(line_start + line.len());
                    Self::parse_html_block_start(&line[trimmed_start - line_start..]).is_some()
                }
            }
            b'+' | b'0'..=b'9' => {
                let line = self.line_at(line_start);
                line_end_hint = Some(line_start + line.len());
                Self::try_parse_list_interrupt(&line[trimmed_start - line_start..])
            }
            b'i' | b'e' => self.options.mdx && super::mdx_esm::looks_like_esm(bytes, trimmed_start),
            _ => false,
        };

        if starts_block {
            return BlockProbe {
                starts_block: true,
                line_end: None,
            };
        }
        let probe = if !check_tables {
            BlockProbe {
                starts_block: false,
                line_end: line_end_hint,
            }
        } else if let Some(end) = line_end_hint {
            // The line is already bounded, so the table guard only has to
            // ask whether a pipe lives inside it.
            let has_pipe = memchr(b'|', &bytes[line_start..end]).is_some();
            BlockProbe {
                starts_block: has_pipe && self.try_parse_table(),
                line_end: Some(end),
            }
        } else {
            match memchr3(b'|', b'\n', b'\r', &bytes[line_start..]) {
                Some(off) if bytes[line_start + off] == b'|' => {
                    if self.try_parse_table() {
                        return BlockProbe {
                            starts_block: true,
                            line_end: None,
                        };
                    }
                    // The pipe stopped the scan short of the terminator.
                    // Finishing the line from there costs one pass over it
                    // in total and spares the caller a fresh one.
                    BlockProbe {
                        starts_block: false,
                        line_end: Some(line_end(bytes, line_start + off + 1)),
                    }
                }
                Some(off) => BlockProbe {
                    starts_block: false,
                    line_end: Some(line_start + off),
                },
                None => BlockProbe {
                    starts_block: false,
                    line_end: Some(self.source.len()),
                },
            }
        };
        debug_assert!(
            probe.starts_block
                || probe
                    .line_end
                    .is_none_or(|end| end == line_end(bytes, line_start)),
            "a reported line end must match a fresh line scan"
        );
        probe
    }

    pub(super) fn line_at(&self, line_start: usize) -> &'a str {
        let bytes = self.source.as_bytes();
        let end = line_end(bytes, line_start);
        &self.source[line_start..end]
    }

    /// The line starting at `line_start` plus the offset where the next line
    /// begins, from a single terminator search.
    ///
    /// `line_at(x)` followed by `next_line_start(x)` scans the same bytes
    /// twice: the second search re-finds the terminator the first one already
    /// stopped on. Line-walking loops take both from one scan through this
    /// helper, since the terminator's width is a two-byte test once its
    /// offset is known.
    pub(super) fn line_and_next(&self, line_start: usize) -> (&'a str, usize) {
        let source = self.source;
        let bytes = source.as_bytes();
        let end = line_end(bytes, line_start);
        (&source[line_start..end], line_terminator_end(bytes, end))
    }

    pub(super) fn next_line_start(&self, line_start: usize) -> usize {
        scan_next_line_start(self.source.as_bytes(), line_start)
    }

    pub(super) fn consume_line(&mut self) -> &'a str {
        let start = self.position;
        let bytes = self.source.as_bytes();
        let end = line_end(bytes, start);
        self.position = line_terminator_end(bytes, end);
        &self.source[start..end]
    }

    /// Finds the first non-space, non-tab byte before the next newline.
    ///
    /// The parser only treats ASCII space and tab as indentation in these
    /// block-level dispatchers. Returning the byte offset, rather than a
    /// trimmed `&str`, lets callers inspect the discriminator byte first and
    /// defer `line_at()` until they know a full line slice is needed.
    pub(super) fn first_non_whitespace_in_line(&self, line_start: usize) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let mut cursor = line_start;

        while cursor < bytes.len() {
            match bytes[cursor] {
                b' ' | b'\t' => cursor += 1,
                byte if is_line_ending_byte(byte) => return None,
                _ => return Some(cursor),
            }
        }

        None
    }
}

#[cfg(test)]
pub(super) mod line_reuse_corpus {
    /// Line shapes whose terminators exercise every branch of the scanner:
    /// LF, CRLF, a lone CR, blank and whitespace-only lines, indentation by
    /// space and tab, multi-byte characters, and an unterminated last line.
    pub(in crate::parser) const SOURCES: &[&str] = &[
        "",
        "\n",
        "\r",
        "\r\n",
        "alpha",
        "alpha\n",
        "alpha\r\nbeta\rgamma\n\n   \n\tdelta",
        "- one\n- two\n\n- three\n  continued\n\n\n- four",
        "1. one\r\n2. two\r\n\r\n   nested\r\n",
        "* a\n*\n\n* c\n",
        "- item\n\n      indented code\n\n- next\n",
        "> quote\n> more\nlazy\n\n> second\n",
        "| a | b |\n| - | - |\n| 1 | 2 |\npipe | in prose\n",
        "# heading\n\ntext with é中🙂 and a tab\there\n\n```\nfence\n```\n",
        "term\n: definition\n\n    indented body\n",
        "text\n---\nsetext above\n===\n",
        "<div>\nhtml block\n</div>\n\n<!-- comment -->\n",
        "[^1]: footnote\n    continued\n\n[ref]: /url\n",
    ];
}

#[cfg(test)]
mod tests {
    use super::line_reuse_corpus::SOURCES;
    use crate::allocator::Allocator;
    use crate::parser::line_scan::{line_end, next_line_start};
    use crate::parser::{Parser, ParserOptions};

    fn option_matrix() -> [ParserOptions; 4] {
        [
            ParserOptions::commonmark(),
            ParserOptions::gfm(),
            ParserOptions::gfm_spec(),
            ParserOptions::mdx(),
        ]
    }

    #[test]
    fn probe_line_reports_the_real_line_end() {
        // Every `Some` the probe hands back has to name the same offset a
        // fresh scan would, whichever branch produced it: a dispatch arm's
        // line slice, the pipe guard's own scan, or the continuation past a
        // pipe that turned out not to open a table.
        for source in SOURCES {
            for options in option_matrix() {
                let allocator = Allocator::new();
                let parser = Parser::with_options(&allocator, source, options);
                let bytes = source.as_bytes();
                let mut line_start = 0;
                while line_start < source.len() {
                    if let Some(trimmed_start) = parser.first_non_whitespace_in_line(line_start) {
                        for probe in [
                            parser.probe_line(line_start, trimmed_start),
                            parser.probe_line_without_table(line_start, trimmed_start),
                        ] {
                            if let Some(end) = probe.line_end {
                                assert_eq!(
                                    end,
                                    line_end(bytes, line_start),
                                    "source {source:?} line at {line_start}"
                                );
                            }
                        }
                    }
                    line_start = next_line_start(bytes, line_start);
                }
            }
        }
    }

    #[test]
    fn skip_blank_lines_reports_the_settled_indentation() {
        for source in SOURCES {
            let allocator = Allocator::new();
            for offset in 0..=source.len() {
                if !source.is_char_boundary(offset) {
                    continue;
                }
                let mut parser = Parser::new(&allocator, source);
                parser.position = offset;
                let reported = parser.skip_blank_lines();
                let settled = parser.position;
                assert_eq!(
                    reported,
                    parser.first_non_whitespace_in_line(settled),
                    "source {source:?} from {offset}"
                );
            }
        }
    }

    #[test]
    fn line_and_next_matches_two_separate_scans() {
        for source in SOURCES {
            let allocator = Allocator::new();
            let parser = Parser::new(&allocator, source);
            for offset in 0..=source.len() {
                if !source.is_char_boundary(offset) {
                    continue;
                }
                assert_eq!(
                    parser.line_and_next(offset),
                    (parser.line_at(offset), parser.next_line_start(offset)),
                    "source {source:?} offset {offset}"
                );
            }
        }
    }
}
