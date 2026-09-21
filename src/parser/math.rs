//! Opt-in `$...$` inline and `$$...$$` display math nodes.

use crate::allocator::Vec;
use crate::ast::{InlineMath, MathBlock, Node, Span};
use memchr::{memchr, memchr2};

use super::Parser;
use super::line_scan::{is_line_ending_byte, next_line_start};
use crate::parser::error::ParseResult;

/// What one scan for an inline-math closer settled, for every opener that
/// starts at or after the scan did.
///
/// `candidate` is the first `$` at or after `origin` that could close an
/// opener of this width, or the content length when the rest of the content
/// holds none. Only the bytes around a `$` decide that, so no closer stands
/// between `origin` and `candidate` for any start in that range either.
#[derive(Clone, Copy)]
pub(super) struct MathClose {
    origin: usize,
    candidate: usize,
    /// No backtick stands in `origin..=candidate`, so the scan that skips
    /// closed code spans reads exactly the same bytes over that range.
    code_free: bool,
}

/// One forward window over a content slice: the first `*` or `_` at or
/// after `origin`, or the slice length when it holds none.
///
/// Nothing stands between `origin` and `hit`, so the answer holds for every
/// start in that range — the same shape as `bracket_text_stop`.
#[derive(Clone, Copy)]
pub(super) struct MathEmphasis {
    content: usize,
    len: usize,
    origin: usize,
    hit: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn try_parse_math_block_at(
        &self,
        line_start: usize,
        line: &str,
        trimmed: &str,
    ) -> bool {
        let trimmed_offset = trimmed.as_ptr() as usize - line.as_ptr() as usize;
        self.options.math
            && Self::indentation_columns(line) <= 3
            && trimmed.starts_with("$$")
            && self
                .math_block_close_from(line_start + trimmed_offset + 2)
                .is_some()
    }

    pub(super) fn parse_math_block(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        // Only the leading whitespace run matters here, and it can never
        // reach the terminator, so walking it directly beats scanning the
        // whole line just to measure its front.
        let bytes = self.source.as_bytes();
        let mut open = start;
        while matches!(bytes.get(open), Some(b' ' | b'\t')) {
            open += 1;
        }
        let Some(close) = self.math_block_close_from(open + 2) else {
            return self.parse_paragraph(start, None);
        };
        let close_end = close + 2;
        self.position = next_line_start(bytes, close_end);
        let value = &self.source[open + 2..close];
        Ok(Some(Node::MathBlock(self.allocator.boxed(MathBlock {
            value,
            span: Span::new(start as u32, self.position as u32),
        }))))
    }

    /// The `$$` that ends a display-math block opened before `cursor`, or
    /// `None` when the rest of the source holds none.
    ///
    /// [`math_block_close`] answers that by walking to the end of the
    /// source, and every line opening with `$$` asks it — once through the
    /// block dispatch and once more through the block-start probe — so a run
    /// of such lines cost one walk each. Whether a byte ends a block is
    /// decided by that byte and its neighbors alone, so the first terminator
    /// at or after one start is the first terminator for every start up to
    /// it, and one walk settles the whole run.
    fn math_block_close_from(&self, cursor: usize) -> Option<usize> {
        let end = self.source.len();
        if let Some((origin, close)) = self.math_block_close.get()
            && origin <= cursor
            && cursor <= close
        {
            return (close < end).then_some(close);
        }
        let close = math_block_close(self.source.as_bytes(), cursor);
        self.math_block_close
            .set(Some((cursor, close.unwrap_or(end))));
        close
    }

    pub(super) fn parse_inline_math(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        let bytes = content.as_bytes();
        let open_len = if bytes.get(*pos + 1) == Some(&b'$') {
            2
        } else {
            1
        };
        if !can_open_inline(bytes, *pos, open_len)
            && !self.can_open_digit_prefixed_inline_math(content, *pos, open_len)
        {
            Self::push_text(
                children,
                &content[*pos..*pos + 1],
                offset + *pos,
                offset + *pos + 1,
            );
            *pos += 1;
            return;
        }

        let inner_start = *pos + open_len;
        if let Some(close) = self.inline_math_close(content, inner_start, open_len) {
            let span = Span::new((offset + *pos) as u32, (offset + close + open_len) as u32);
            children.push(Node::InlineMath(InlineMath {
                value: &content[inner_start..close],
                span,
            }));
            *pos = close + open_len;
            return;
        }

        Self::push_text(
            children,
            &content[*pos..*pos + open_len],
            offset + *pos,
            offset + *pos + open_len,
        );
        *pos += open_len;
    }

    /// The `$` run that closes an opener of `open_len` bytes somewhere at or
    /// after `from`, or `None` when nothing in the rest of `content` does.
    ///
    /// [`scan_inline_math_close`] reports "nothing closes this" only after
    /// reading to the end of the content — and, unlike the `^`/`~` script
    /// spans, it keeps going past a `$` that cannot close — so a run of
    /// openers paid one walk each: 128 KiB of `$a ` took 6.2 s, x16 for
    /// every x4 of input.
    ///
    /// The memo is what one scan settles for every later opener. A `$` can
    /// only close when its own bytes and its neighbors allow it, which is
    /// what [`first_close_candidate`] tests, and skipping a code span only
    /// ever removes candidates from the scan above. So a range with no
    /// candidate in it holds no closer for *any* start inside it, and where
    /// no backtick stands in that range the two scans read the same bytes
    /// and reach the same `$`.
    fn inline_math_close(&self, content: &'a str, from: usize, open_len: usize) -> Option<usize> {
        let bytes = content.as_bytes();
        let key = (content.as_ptr() as usize, content.len(), open_len as u8);
        let cached = self.math_closers.borrow().get(&key).copied();
        let memo = match cached {
            Some(memo) if memo.origin <= from && from <= memo.candidate => memo,
            _ => {
                let memo = first_close_candidate(bytes, from, open_len);
                self.math_closers.borrow_mut().insert(key, memo);
                memo
            }
        };

        if memo.candidate == content.len() {
            return None;
        }
        if memo.code_free {
            return Some(memo.candidate);
        }
        scan_inline_math_close(bytes, from, open_len)
    }

    /// A `$` before a digit opens math only when something closes it, which
    /// is the one place the scan runs without a node to show for it.
    fn can_open_digit_prefixed_inline_math(
        &self,
        content: &'a str,
        index: usize,
        open_len: usize,
    ) -> bool {
        let bytes = content.as_bytes();
        let next = bytes.get(index + open_len).copied();
        let prev = index
            .checked_sub(1)
            .and_then(|prev| bytes.get(prev).copied());
        matches!(next, Some(b'0'..=b'9'))
            && !matches!(prev, Some(b'0'..=b'9'))
            && self.has_closing_inline_math(content, index, open_len)
    }

    fn has_closing_inline_math(&self, content: &'a str, index: usize, open_len: usize) -> bool {
        let inner_start = index + open_len;
        let Some(close) = self.inline_math_close(content, inner_start, open_len) else {
            return false;
        };
        self.first_emphasis_byte(content, inner_start) < close
    }

    /// The first `*` or `_` at or after `from`, or the content length.
    ///
    /// The probe above searches the whole span up to the closer, so a run of
    /// `$1 ` openers sharing one closer far behind them searched the same
    /// bytes once per opener. One forward window answers all of them.
    fn first_emphasis_byte(&self, content: &'a str, from: usize) -> usize {
        let base = content.as_ptr() as usize;
        if let Some(memo) = self.math_emphasis.get()
            && memo.content == base
            && memo.len == content.len()
            && from >= memo.origin
            && from <= memo.hit
        {
            return memo.hit;
        }
        let hit = memchr2(b'*', b'_', &content.as_bytes()[from..])
            .map_or(content.len(), |relative| from + relative);
        self.math_emphasis.set(Some(MathEmphasis {
            content: base,
            len: content.len(),
            origin: from,
            hit,
        }));
        hit
    }
}

fn math_block_close(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    while let Some(relative) = memchr(b'$', &bytes[cursor..]) {
        let dollar = cursor + relative;
        if is_escaped_marker(bytes, dollar) {
            cursor = dollar + 1;
            continue;
        }
        if bytes.get(dollar + 1) == Some(&b'$') && is_line_end(bytes, dollar + 2) {
            return Some(dollar);
        }
        cursor = dollar + 1;
    }
    None
}

/// The first `$` at or after `from` that closes an opener of `open_len`
/// bytes, with closed code spans skipped whole so that `` $a `$` b$ ``
/// closes on the last `$` and not the quoted one.
fn scan_inline_math_close(bytes: &[u8], from: usize, open_len: usize) -> Option<usize> {
    let mut cursor = from;
    while let Some(relative) = memchr2(b'$', b'`', &bytes[cursor..]) {
        let candidate = cursor + relative;
        if bytes[candidate] == b'`' {
            cursor = Parser::closed_code_span_end(bytes, candidate)
                .unwrap_or_else(|| candidate + Parser::marker_run_len(bytes, candidate, b'`'));
            continue;
        }
        if can_close_at(bytes, candidate, open_len) {
            return Some(candidate);
        }
        cursor = candidate + 1;
    }
    None
}

/// [`scan_inline_math_close`] without the code-span skip: the first `$` at
/// or after `from` that could close on its own bytes, and whether a backtick
/// stands before it.
///
/// Skipping only removes candidates, so "no candidate at all" is an answer
/// the scan above cannot disagree with, wherever it starts.
fn first_close_candidate(bytes: &[u8], from: usize, open_len: usize) -> MathClose {
    let mut cursor = from;
    let mut code_free = true;
    while let Some(relative) = memchr2(b'$', b'`', &bytes[cursor..]) {
        let at = cursor + relative;
        if bytes[at] == b'`' {
            code_free = false;
        } else if can_close_at(bytes, at, open_len) {
            return MathClose {
                origin: from,
                candidate: at,
                code_free,
            };
        }
        cursor = at + 1;
    }
    MathClose {
        origin: from,
        candidate: bytes.len(),
        code_free,
    }
}

fn can_close_at(bytes: &[u8], index: usize, open_len: usize) -> bool {
    !is_escaped_marker(bytes, index)
        && marker_len_at(bytes, index) >= open_len
        && can_close_inline(bytes, index, open_len)
}

fn can_open_inline(bytes: &[u8], index: usize, open_len: usize) -> bool {
    let next = bytes.get(index + open_len).copied();
    let prev = index
        .checked_sub(1)
        .and_then(|prev| bytes.get(prev).copied());
    !matches!(next, None | Some(b' ' | b'\t' | b'\n' | b'0'..=b'9'))
        && !matches!(prev, Some(b'0'..=b'9'))
}

fn can_close_inline(bytes: &[u8], index: usize, close_len: usize) -> bool {
    let prev = index
        .checked_sub(1)
        .and_then(|prev| bytes.get(prev).copied());
    let next = bytes.get(index + close_len).copied();
    !matches!(prev, None | Some(b' ' | b'\t' | b'\n')) && !matches!(next, Some(b'0'..=b'9'))
}

fn marker_len_at(bytes: &[u8], start: usize) -> usize {
    let mut len = 0;
    while bytes.get(start + len) == Some(&b'$') {
        len += 1;
    }
    len
}

fn is_escaped_marker(bytes: &[u8], pos: usize) -> bool {
    let mut count = 0usize;
    let mut cursor = pos;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        count += 1;
        cursor -= 1;
    }
    count % 2 == 1
}

fn is_line_end(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b' ' | b'\t' => cursor += 1,
            byte if is_line_ending_byte(byte) => return true,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests;
