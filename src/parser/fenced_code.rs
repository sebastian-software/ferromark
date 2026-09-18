use crate::ast::{Node, Span};

use super::Parser;
use super::line_scan::{line_end, line_terminator_end, next_line_start};
use super::prepass::next_fence_run_line;
use crate::parser::error::ParseResult;

impl<'a> Parser<'a> {
    /// Finds the body end and cursor position after a closing fence.
    ///
    /// This helper is used only by the zero-copy fenced-code path where the
    /// opening fence has no indentation. In that case body lines do not need
    /// stripping, so the parser can search line starts in the original source
    /// and return a borrowed body slice. The tuple separates the end of code
    /// content from the end of the closing fence line.
    pub(super) fn find_fenced_close(
        &self,
        fence_char: char,
        fence_len: usize,
        body_start: usize,
    ) -> (usize, usize) {
        fenced_close_bounds(
            self.source.as_bytes(),
            fence_char as u8,
            fence_len,
            body_start,
        )
    }
}

/// The closing-fence rule, shared by fenced-code parsing and the definition
/// pre-pass's segment planner.
///
/// Both callers must agree on exactly where an unindented fence ends: the
/// planner treats that range as opaque, so a closer it located differently
/// from the parser would shift every later block boundary. Returning the two
/// offsets keeps the rule in one place — the end of the code content and the
/// start of the line after the closing fence (both `bytes.len()` when the
/// fence is never closed).
pub(in crate::parser) fn fenced_close_bounds(
    bytes: &[u8],
    fence_byte: u8,
    fence_len: usize,
    body_start: usize,
) -> (usize, usize) {
    let mut from = body_start;

    // A closing fence holds at least three fence bytes in a row, so only
    // lines containing such a run can close the block. Jump between those
    // with the pre-pass's cached searcher instead of visiting every body
    // line: code blocks are long and closers rare.
    while let Some(line_start) = next_fence_run_line(bytes, from, fence_byte) {
        // Skip up to 3 leading spaces.
        let mut cursor = line_start;
        let max_indent_end = (line_start + 3).min(bytes.len());
        while cursor < max_indent_end && bytes[cursor] == b' ' {
            cursor += 1;
        }

        // Count the run of `fence_char`.
        let fence_start = cursor;
        while cursor < bytes.len() && bytes[cursor] == fence_byte {
            cursor += 1;
        }
        let count = cursor - fence_start;

        if count >= fence_len {
            // A closing fence carries nothing but trailing whitespace
            // (``` aaa is content, not a closer). Neither the indent nor the
            // fence run can hold a terminator, so this scan finds the whole
            // line's end and the next line starts just past it.
            let line_end = line_end(bytes, cursor);
            let after_fence = line_terminator_end(bytes, line_end);
            let only_ws = bytes[cursor..line_end]
                .iter()
                .all(|byte| matches!(byte, b' ' | b'\t' | b'\r'));
            if only_ws {
                // Body ends at `line_start`; the fence line ends at the next
                // newline (inclusive) or EOF.
                return (line_start, after_fence);
            }
            from = after_fence;
            continue;
        }

        // Not a closing fence — move to the next line.
        from = next_line_start(bytes, line_start);
    }

    // No closing fence; consume everything as body.
    (bytes.len(), bytes.len())
}

impl<'a> Parser<'a> {
    /// Parses a fenced code block.
    ///
    /// `opening_indent` is the fence line's indentation in columns, which
    /// block dispatch measured before it recognized the fence. It is below
    /// four there, so it is a plain run of spaces and no tab-stop arithmetic
    /// can disagree with it.
    pub(super) fn parse_fenced_code(
        &mut self,
        start: usize,
        opening_indent: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        debug_assert_eq!(
            opening_indent,
            self.calc_indentation(start).min(3),
            "fence indentation must match a fresh indentation scan"
        );
        debug_assert!(
            self.source.as_bytes()[start..start + opening_indent]
                .iter()
                .all(|&byte| byte == b' '),
            "an indent below four columns cannot contain a tab"
        );
        self.position = start + opening_indent;

        // Block dispatch only reaches this parser after `try_parse_fenced_code_at`
        // has seen the fence run, so the fence character is always present.
        // Treat its absence as "not a fenced code block" instead of reporting
        // an error kind no real input can produce.
        let Some(fence_char) = self.peek() else {
            debug_assert!(false, "fenced code dispatch without a fence character");
            return Ok(None);
        };
        let mut fence_len = 0;

        while self.peek() == Some(fence_char) {
            fence_len += 1;
            self.advance();
        }

        // Parse info string (language)
        self.skip_whitespace();
        let info_start = self.position;
        while let Some(ch) = self.peek() {
            if matches!(ch, '\n' | '\r') {
                break;
            }
            self.advance();
        }
        let info = self.source[info_start..self.position].trim();
        let (lang, meta) = if info.is_empty() {
            (None, None)
        } else if let Some(space_idx) = info.find(' ') {
            (Some(&info[..space_idx]), Some(&info[space_idx + 1..]))
        } else {
            (Some(info), None)
        };
        // Backslash escapes and entity references apply in info strings.
        let lang = lang.map(|lang| self.unescape_link_component(lang));

        let bytes = self.source.as_bytes();
        // The info-string walk stopped on the terminator (or at EOF), so
        // stepping over it needs no further search.
        self.position = line_terminator_end(bytes, self.position);

        // Fast path: when the opening fence has no indentation, the body
        // lines need no indent stripping — we can find the closing fence
        // by scanning the source and emit a zero-copy `&str` slice. This
        // is the overwhelmingly common case (almost no code block in the
        // wild is indented), and it removes both the per-line copy into a
        // growing `String` *and* the trailing `alloc_str` (which used to
        // double-allocate every code block's body).
        let body_start = self.position;
        let span;
        let value: &'a str = if opening_indent == 0 {
            let (body_end, fence_line_end) =
                self.find_fenced_close(fence_char, fence_len, body_start);
            self.position = fence_line_end;
            span = Span::new(start as u32, self.position as u32);
            let body = &self.source[body_start..body_end];
            if body.as_bytes().contains(&b'\r') {
                self.normalize_code_block_line_endings(body)
            } else {
                body
            }
        } else {
            // Indented opening fence: lines may need leading-space stripping,
            // so a borrowed source slice would be wrong. Materialize only this
            // uncommon case, and write directly into a bump-allocated string
            // so the final AST can borrow it without a second arena copy.
            let remaining_estimate = self.source.len().saturating_sub(self.position);
            let mut value = crate::allocator::String::with_capacity_in(
                remaining_estimate.min(8 * 1024),
                self.allocator.bump(),
            );

            loop {
                if self.is_at_end() {
                    break;
                }

                let line_start = self.position;
                let (line, next_line) = self.line_and_next(line_start);
                let line_indent = Self::indentation_columns(line);

                if line_indent <= 3 {
                    self.position = line_start;
                    // `line_indent` was just computed from the same leading
                    // whitespace `calc_indentation(line_start)` would re-scan,
                    // and is already <= 3, so the `.min(3)` was a no-op.
                    let indent_to_skip = line_indent;
                    for _ in 0..indent_to_skip {
                        if self.peek() == Some(' ') {
                            self.advance();
                        }
                    }

                    // Check for closing fence
                    let mut closing_fence_len = 0;
                    while self.peek() == Some(fence_char) {
                        closing_fence_len += 1;
                        self.advance();
                    }

                    if closing_fence_len >= fence_len {
                        // Skip rest of line
                        while let Some(ch) = self.peek() {
                            if matches!(ch, '\n' | '\r') {
                                self.position = line_terminator_end(bytes, self.position);
                                break;
                            }
                            self.advance();
                        }
                        break;
                    }
                }

                // Not a closing fence, reset and consume line
                self.position = line_start;
                let stripped = Self::strip_indent_columns(line, opening_indent);
                value.push_str(stripped);
                if line_start + line.len() < bytes.len() {
                    value.push('\n');
                }
                self.position = next_line;
            }

            span = Span::new(start as u32, self.position as u32);
            value.into_bump_str()
        };

        Ok(Some(Node::CodeBlock(self.allocator.boxed(
            crate::ast::CodeBlock {
                lang,
                meta,
                value,
                span,
            },
        ))))
    }

    fn normalize_code_block_line_endings(&self, source: &str) -> &'a str {
        let mut value =
            crate::allocator::String::with_capacity_in(source.len(), self.allocator.bump());
        let bytes = source.as_bytes();
        let mut chunk_start = 0;
        let mut cursor = 0;

        while cursor < bytes.len() {
            if bytes[cursor] != b'\r' {
                cursor += 1;
                continue;
            }

            value.push_str(&source[chunk_start..cursor]);
            value.push('\n');
            cursor += if bytes.get(cursor + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
            chunk_start = cursor;
        }

        value.push_str(&source[chunk_start..]);
        value.into_bump_str()
    }
}

#[cfg(test)]
mod fence_close_tests {
    // Owned strings keep the test oracle independent of production arena storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use crate::allocator::Allocator;

    use super::super::Parser;
    use super::super::line_scan::{line_end, line_terminator_end, next_line_start};

    /// The original line-by-line walk, kept as the oracle.
    fn scalar_close(
        source: &str,
        fence_byte: u8,
        fence_len: usize,
        body_start: usize,
    ) -> (usize, usize) {
        let bytes = source.as_bytes();
        let mut line_start = body_start;
        while line_start < bytes.len() {
            let mut cursor = line_start;
            let max_indent_end = (line_start + 3).min(bytes.len());
            while cursor < max_indent_end && bytes[cursor] == b' ' {
                cursor += 1;
            }
            let fence_start = cursor;
            while cursor < bytes.len() && bytes[cursor] == fence_byte {
                cursor += 1;
            }
            if cursor - fence_start >= fence_len {
                let end = line_end(bytes, cursor);
                if bytes[cursor..end]
                    .iter()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
                {
                    return (line_start, line_terminator_end(bytes, end));
                }
            }
            line_start = next_line_start(bytes, line_start);
        }
        (bytes.len(), bytes.len())
    }

    #[test]
    fn close_search_matches_line_walk() {
        let lines = [
            "code",
            "",
            "  ",
            "```",
            "````",
            "~~~",
            " ```",
            "   ```",
            "    ```",
            "``` trailing",
            "```\t",
            "x```",
            "`` `",
            "~~~~ ",
            "text with ``` inside",
            "\r",
            "é中🙂",
        ];
        let mut state = 0x5bd1_e995_2d3c_7a11u64;
        for _ in 0..3000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let count = (state >> 33) as usize % 12;
            let mut source = String::new();
            for _ in 0..count {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                source.push_str(lines[(state >> 33) as usize % lines.len()]);
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                source.push_str(if (state >> 40).is_multiple_of(4) {
                    "\r\n"
                } else {
                    "\n"
                });
            }
            if (state >> 20).is_multiple_of(2) {
                source.push_str("tail");
            }
            let allocator = Allocator::new();
            let parser = Parser::new(&allocator, &source);
            for (fence_char, fence_len) in [('`', 3), ('`', 4), ('~', 3), ('~', 5)] {
                assert_eq!(
                    parser.find_fenced_close(fence_char, fence_len, 0),
                    scalar_close(&source, fence_char as u8, fence_len, 0),
                    "source {source:?} fence {fence_char}{fence_len}"
                );
            }
        }
    }
}
