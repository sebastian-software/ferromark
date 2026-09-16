use crate::ast::{Html, Node, Span};
use memchr::{memchr, memmem};

use super::Parser;
use super::line_scan::{line_end, line_terminator_end, next_line_start};
use crate::parser::error::ParseResult;

mod start;

pub(in crate::parser) use start::HtmlBlockStart;

impl<'a> Parser<'a> {
    /// Parses an HTML block previously classified by `parse_html_block_start`.
    ///
    /// `block_start` is trusted to match the current line. This is why the
    /// function no longer rechecks the opener: the outer dispatcher owns that
    /// responsibility, and this function only advances `self.position` to the
    /// end of the block.
    pub(super) fn parse_html_block(
        &mut self,
        start: usize,
        block_start: HtmlBlockStart,
    ) -> ParseResult<Option<Node<'a>>> {
        debug_assert_eq!(
            self.position, start,
            "an HTML block is dispatched from its own first line"
        );
        self.position = html_block_end(self.source.as_bytes(), start, block_start);

        let span = Span::new(start as u32, self.position as u32);
        let value = &self.source[start..self.position];
        Ok(Some(Node::Html(Html { value, span })))
    }
}

/// End of the HTML block that `block_start` opens on the line beginning at
/// `start`, shared by block parsing and the definition pre-pass's segment
/// planner.
///
/// The planner treats the same range as opaque, so both callers have to agree
/// byte for byte on where the block stops; keeping the four closing rules in
/// one function is what guarantees that.
pub(in crate::parser) fn html_block_end(
    bytes: &[u8],
    start: usize,
    block_start: HtmlBlockStart,
) -> usize {
    html_block_bounds(bytes, start, block_start).0
}

/// [`html_block_end`] plus whether the block's own closing condition was met
/// inside `bytes` rather than the block simply running out of input.
///
/// The planner searches a truncated view of the document, where the two cases
/// can end at the same offset and mean opposite things: a terminator on the
/// last line before the truncation closes the block, while running out of
/// input means the block could reach past it.
pub(in crate::parser) fn html_block_bounds(
    bytes: &[u8],
    start: usize,
    block_start: HtmlBlockStart,
) -> (usize, bool) {
    match block_start {
        // Types 2-5 close on the first line CONTAINING their terminator, so
        // one whole-block substring search replaces the per-line `contains`
        // loop. The opener line participates in the search (a one-line
        // `<!-- ... -->` closes immediately), exactly like the old first
        // `consume_line` iteration did.
        HtmlBlockStart::Comment => block_end_past(bytes, start, b"-->"),
        HtmlBlockStart::Terminated(terminator) => {
            block_end_past(bytes, start, terminator.as_bytes())
        }
        // Type 1 blocks (`<pre>`, `<script>`, `<style>`, `<textarea>`) close
        // on the first line containing `</tag`, searched case-insensitively
        // across the whole remaining source in one scan.
        HtmlBlockStart::Type1(tag) => match find_closing_tag(bytes, start, tag.closing_name()) {
            Some(at) => (end_of_line_after(bytes, at), true),
            None => (bytes.len(), false),
        },
        HtmlBlockStart::Other => block_end_before_blank(bytes, next_line_start(bytes, start)),
    }
}

/// The offset just past the line that contains `needle`, or EOF when the
/// remaining source never contains it.
fn block_end_past(bytes: &[u8], from: usize, needle: &[u8]) -> (usize, bool) {
    match memmem::find(&bytes[from..], needle) {
        Some(off) => (end_of_line_after(bytes, from + off + needle.len()), true),
        None => (bytes.len(), false),
    }
}

/// Walks a regular HTML block to the next blank line.
///
/// The result must stop *before* that blank line so the outer block parser
/// can consume it through `skip_blank_lines`. A single memchr newline
/// iterator walks the block; each line costs one first-byte check unless it
/// actually starts with whitespace.
fn block_end_before_blank(bytes: &[u8], from: usize) -> (usize, bool) {
    let mut line_start = from;

    while line_start < bytes.len() {
        let current_line_end = line_end(bytes, line_start);
        if is_blank_line(&bytes[line_start..current_line_end]) {
            return (line_start, true);
        }
        // The terminator is already located; step over it instead of
        // searching the line for it a second time.
        let next_line = line_terminator_end(bytes, current_line_end);
        if next_line == line_start {
            break;
        }
        line_start = next_line;
    }

    // Trailing line without a newline: a whitespace-only remainder stays
    // unconsumed, anything else belongs to the block.
    if line_start >= bytes.len() || is_blank_line(&bytes[line_start..]) {
        (line_start.min(bytes.len()), false)
    } else {
        (bytes.len(), false)
    }
}

/// Position of the first case-insensitive `</tag` at or after `from`.
///
/// Type-1 HTML blocks close on the first line containing their closing
/// tag; searching for `<` with `memchr` skips the common case of long
/// text/code runs that contain no tag-looking byte at all.
pub(in crate::parser) fn find_closing_tag(bytes: &[u8], from: usize, tag: &[u8]) -> Option<usize> {
    let mut search = from;
    while let Some(off) = memchr(b'<', &bytes[search..]) {
        let at = search + off;
        if at + tag.len() + 2 <= bytes.len()
            && bytes[at + 1] == b'/'
            && bytes[at + 2..at + 2 + tag.len()].eq_ignore_ascii_case(tag)
        {
            return Some(at);
        }
        search = at + 1;
    }
    None
}

/// True when `line` holds only spaces, tabs, and carriage returns
/// (including the empty line).
#[inline]
fn is_blank_line(line: &[u8]) -> bool {
    line.iter().all(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
}

/// Byte offset just past the newline of the line containing `at` (or EOF).
#[inline]
fn end_of_line_after(bytes: &[u8], at: usize) -> usize {
    next_line_start(bytes, at)
}
