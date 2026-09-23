use memchr::{memchr, memchr3};

use super::Parser;
use super::line_scan::{
    is_line_ending_byte, line_end, line_terminator_end, next_line_start as scan_next_line_start,
};
use super::whitespace;

/// The leading space/tab run of a line, in the units the block parsers
/// measure indentation in.
///
/// The two column counts deliberately disagree on tabs, because their
/// callers always have: [`Self::columns`] is the real column the run ends
/// on, while [`Self::flat_columns`] reproduces
/// [`Parser::calc_indentation`], which charges every tab four columns
/// wherever it sits. List continuation depth has always been compared in
/// the flat unit, so unifying the two would move item boundaries.
#[derive(Clone, Copy)]
pub(super) struct LineIndent {
    /// Length of the run in bytes.
    pub bytes: usize,
    /// True columns: a tab advances to the next multiple of four, as in
    /// `line_indent_width` and the tracker's `leading_indent`.
    pub columns: usize,
    /// [`Parser::calc_indentation`]'s count: a space is one column and a
    /// tab a flat four.
    pub flat_columns: usize,
    /// Whether the run holds a tab at all. Without one both counts are the
    /// byte length and column arithmetic collapses into slicing.
    pub has_tab: bool,
}

impl LineIndent {
    /// Measures the space/tab run that starts at `line_start`.
    ///
    /// Neither terminator byte is a space or a tab, so the run stops at the
    /// line's end by itself: the walk needs no line bound, and a caller can
    /// look at the byte after the run before it pays for a terminator
    /// search.
    pub(super) fn at(bytes: &[u8], line_start: usize) -> Self {
        let mut indent = LineIndent {
            bytes: 0,
            columns: 0,
            flat_columns: 0,
            has_tab: false,
        };
        for &byte in &bytes[line_start..] {
            match byte {
                b' ' => {
                    indent.columns += 1;
                    indent.flat_columns += 1;
                }
                b'\t' => {
                    indent.columns = (indent.columns / 4 + 1) * 4;
                    indent.flat_columns += 4;
                    indent.has_tab = true;
                }
                _ => break,
            }
            indent.bytes += 1;
        }
        indent
    }
}

/// Everything a container's line walk needs to know about one source line.
///
/// List items and block quotes ask the same questions of every line they
/// consume — where it ends, where the next one starts, how deeply it is
/// indented, whether it is blank, and what it looks like past its
/// indentation — and each answer used to come from its own scan over the
/// same bytes. These facts come from one walk of the leading whitespace
/// followed by one terminator search that starts where the walk stopped.
pub(super) struct LineFacts<'a> {
    /// The line without its terminator.
    pub line: &'a str,
    /// Where the next line starts: past the terminator, or `source.len()`
    /// when the line is unterminated.
    pub next: usize,
    /// The leading space/tab run, measured once.
    pub indent: LineIndent,
}

impl<'a> LineFacts<'a> {
    /// The line past its leading space/tab run — what a container strips
    /// before looking for its marker.
    pub fn after_indent(&self) -> &'a str {
        &self.line[self.indent.bytes..]
    }

    /// Byte-identical to [`whitespace::trim_start`] of the whole line, but
    /// resumed where the space/tab run ended: a single byte test when the
    /// run ends on content. Only a vertical tab or a form feed walks on,
    /// which is also why this can be shorter than [`Self::after_indent`].
    pub fn trimmed(&self) -> &'a str {
        whitespace::trim_start(self.after_indent())
    }

    /// Whether the line holds nothing but whitespace, matching the
    /// [`whitespace::is_blank`] test the list walk has always used. A
    /// no-break space is content, so a line of them is not blank.
    pub fn is_blank(&self) -> bool {
        self.trimmed().is_empty()
    }
}

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

    /// Returns true when the line at `line_start` begins a block-level
    /// construct, given the offset of its first non-space, non-tab byte.
    ///
    /// This is the paragraph-continuation counterpart of `parse_block`'s
    /// first-byte dispatcher. Paragraph parsing calls it for each following
    /// line, so it deliberately avoids `current_line().trim_start()` unless
    /// the leading marker byte makes a block parse plausible. Keeping this
    /// byte-dispatch table aligned with `parse_block` preserves Markdown
    /// behavior while avoiding repeated full-line scans on ordinary prose.
    ///
    /// The callers reach this from their own line walk, which located that
    /// byte on the way in: taking it as an argument keeps the indentation
    /// from being walked a second time here.
    pub(super) fn line_starts_block(&self, line_start: usize, trimmed_start: usize) -> bool {
        debug_assert_eq!(
            Some(trimmed_start),
            self.first_non_whitespace_in_line(line_start),
            "a reported first non-whitespace byte must match a fresh scan"
        );
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
                // The balanced scan walks to the end of the source to report
                // that nothing closed, so a run of lines starting with an
                // unclosed `{` would pay one walk each.
                self.options.mdx
                    && self.has_closer_from(self.source, trimmed_start + 1, b'}')
                    && self.looks_like_mdx_flow_expression(trimmed_start)
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

    /// Everything the container walks need to know about the line starting
    /// at `line_start`, from one pass over it. See [`LineFacts`].
    pub(super) fn line_facts(&self, line_start: usize) -> LineFacts<'a> {
        let indent = LineIndent::at(self.source.as_bytes(), line_start);
        self.line_facts_with_indent(line_start, indent)
    }

    /// [`Self::line_facts`] for a caller that has already measured the
    /// line's indentation: a block quote does, so that its closing blank
    /// line stops the walk before any terminator search.
    ///
    /// The indentation holds no terminator, so the search starts where the
    /// run ends and never reads those bytes a second time.
    pub(super) fn line_facts_with_indent(
        &self,
        line_start: usize,
        indent: LineIndent,
    ) -> LineFacts<'a> {
        let source = self.source;
        let bytes = source.as_bytes();
        let end = line_end(bytes, line_start + indent.bytes);
        LineFacts {
            line: &source[line_start..end],
            next: line_terminator_end(bytes, end),
            indent,
        }
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
    use super::LineIndent;
    use super::line_reuse_corpus::SOURCES;
    use crate::allocator::Allocator;
    use crate::parser::line_scan::{is_line_ending_byte, line_end, next_line_start};
    use crate::parser::whitespace;
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

    /// Line shapes the container walks treat differently from the scanner
    /// corpus above: no-break spaces, which are content rather than
    /// blankness, a vertical tab or form feed behind the indentation, which
    /// the ASCII trim strips but the space/tab run does not, and tabs mixed
    /// into the run at every offset from a tab stop.
    const CONTAINER_SOURCES: &[&str] = &[
        "\u{a0}\n",
        " \u{a0}\r\n  \u{a0} x\r",
        "\u{a0}- item\n  \u{a0}\n",
        " \u{c}- item\n\u{b}\n \u{c} \n",
        "\t \tx\r\n  \t\n \t\t\r",
        ">\t\tcode\n  > \tquote\n   \t>x",
        "   - a\n\t- b\n  \t - c\n",
    ];

    #[test]
    fn line_facts_match_the_separate_scans_they_replace() {
        // Every fact the one-pass walk reports has to equal the answer the
        // helper it replaced would have produced, including the two column
        // counts, which disagree on tabs on purpose.
        for source in SOURCES.iter().chain(CONTAINER_SOURCES) {
            let allocator = Allocator::new();
            let parser = Parser::new(&allocator, source);
            let bytes = source.as_bytes();
            for offset in 0..=source.len() {
                if !source.is_char_boundary(offset) {
                    continue;
                }
                let facts = parser.line_facts(offset);
                assert_eq!(
                    (facts.line, facts.next),
                    parser.line_and_next(offset),
                    "source {source:?} offset {offset}"
                );
                assert_eq!(
                    facts.trimmed(),
                    whitespace::trim_start(facts.line),
                    "source {source:?} offset {offset}"
                );
                assert_eq!(
                    facts.is_blank(),
                    whitespace::is_blank(facts.line),
                    "source {source:?} offset {offset}"
                );
                assert_eq!(
                    facts.indent.flat_columns,
                    parser.calc_indentation(offset),
                    "source {source:?} offset {offset}"
                );
                assert_eq!(
                    facts.indent.columns,
                    parser.line_indent_width(offset, offset + facts.indent.bytes),
                    "source {source:?} offset {offset}"
                );
                assert_eq!(
                    facts.after_indent(),
                    facts.line.trim_start_matches([' ', '\t']),
                    "source {source:?} offset {offset}"
                );
                assert_eq!(
                    facts.indent.has_tab,
                    facts.after_indent().len() != facts.line.trim_start_matches(' ').len(),
                    "source {source:?} offset {offset}"
                );
                // The block quote's own blank test, which it runs on the
                // measured run before it builds the facts.
                let run_end = offset + LineIndent::at(bytes, offset).bytes;
                assert_eq!(
                    facts.after_indent().is_empty(),
                    run_end >= bytes.len() || is_line_ending_byte(bytes[run_end]),
                    "source {source:?} offset {offset}"
                );
                if !facts.after_indent().is_empty() {
                    assert_eq!(
                        parser.first_non_whitespace_in_line(offset),
                        Some(run_end),
                        "source {source:?} offset {offset}"
                    );
                }
            }
        }
    }

    #[test]
    fn dedenting_a_measured_line_agrees_with_the_walking_form() {
        // The tab-free fast path has to reproduce the column walk exactly,
        // both in what it writes and in the source offset it reports.
        let lines = [
            "", " ", "    ", "\t", " \tx", "x", "  x", "     x", "\tx", "\t\tx", "  \tx", " x \t",
            " \u{a0}x", "  \u{c}x",
        ];
        for line in lines {
            for columns in 0..8usize {
                let allocator = Allocator::new();
                let mut measured = allocator.new_string();
                let mut walked = allocator.new_string();
                let indent = LineIndent::at(line.as_bytes(), 0);
                let measured_offset = Parser::push_measured_line_without_indent(
                    &mut measured,
                    line,
                    &indent,
                    columns,
                );
                let walked_offset = Parser::push_line_without_indent(&mut walked, line, columns);
                assert_eq!(*measured, *walked, "line {line:?} columns {columns}");
                assert_eq!(
                    measured_offset, walked_offset,
                    "line {line:?} columns {columns}"
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
