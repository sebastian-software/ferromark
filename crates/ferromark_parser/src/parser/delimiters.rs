//! Shared delimiter scans for links, images, code spans, math and MDX.
//!
//! Keep marker runs, closed-code skipping and cached closer discovery alongside
//! balanced bracket scanning. Syntax-specific parsing stays with its construct.

use memchr::memchr;

use super::Parser;
use super::byte_class::ByteClass;

impl<'a> Parser<'a> {
    pub(super) fn marker_run_len(bytes: &[u8], start: usize, marker: u8) -> usize {
        let mut count = 1;
        while start + count < bytes.len() && bytes[start + count] == marker {
            count += 1;
        }
        count
    }

    /// Returns the byte after a closed code span that opens at `start`.
    ///
    /// Inline constructs outside code spans use this to skip over backtick
    /// regions while scanning for their own closing delimiter. An unmatched
    /// opener stays literal and therefore is not skipped.
    pub(super) fn closed_code_span_end(bytes: &[u8], start: usize) -> Option<usize> {
        let open_len = Self::marker_run_len(bytes, start, b'`');
        let mut cursor = start + open_len;
        while cursor < bytes.len() {
            let relative = memchr::memchr(b'`', &bytes[cursor..])?;
            cursor += relative;
            let close_len = Self::marker_run_len(bytes, cursor, b'`');
            if close_len == open_len {
                return Some(cursor + close_len);
            }
            cursor += close_len;
        }
        None
    }

    /// Reports whether `closer` occurs at or after `from` in `content`.
    ///
    /// The balanced scans (`scan_balanced` for `]`, `skip_braces` for `}`)
    /// only report that nothing closed after walking to the end of the
    /// content, so a run of unclosed openers pays one full walk each and
    /// costs O(n²). The position of the last closer settles it for every
    /// opener in the slice at once, so the run costs one scan in total.
    pub(super) fn has_closer_from(&self, content: &'a str, from: usize, closer: u8) -> bool {
        let key = (content.as_ptr() as usize, content.len(), closer);

        let cached = self.last_closer.borrow().get(&key).copied();
        let last = if let Some(last) = cached {
            last
        } else {
            let last = memchr::memrchr(closer, content.as_bytes());
            self.last_closer.borrow_mut().insert(key, last);
            last
        };

        last.is_some_and(|last| last >= from)
    }

    /// Scans a bracketed region and returns the index of the `]` that closes
    /// it, or `content.len()` when the brackets never balance, together with
    /// whether an unescaped `[` occurred inside the region. Callers use that
    /// flag in place of a second search for nested brackets.
    ///
    /// Constructs that bind tighter than brackets are skipped whole:
    /// backslash escapes, code spans (an unmatched opener stays literal),
    /// autolinks, and inline raw HTML. This is what makes
    /// `[not a `link](/foo`)` a code span instead of a link.
    ///
    /// Only the five bytes that can change the verdict are inspected; the
    /// ordinary text between them is skipped with [`BRACKET_STOP`]. Link
    /// text is the second-largest scalar walk after destinations on
    /// link-dense documents, so this matters for every `[`.
    pub(super) fn scan_balanced(content: &str, mut cursor: usize) -> (usize, bool) {
        let bytes = content.as_bytes();
        let mut depth = 1;
        let mut nested = false;
        loop {
            cursor = BRACKET_STOP.first_in(bytes, cursor);
            let Some(&byte) = bytes.get(cursor) else {
                break;
            };
            match byte {
                b'\\' => {
                    // An escaped ASCII punctuation byte (which covers both
                    // delimiters) is inert for bracket matching.
                    let escapes_next =
                        cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_punctuation();
                    cursor += if escapes_next { 2 } else { 1 };
                }
                b'`' => {
                    let run = Self::marker_run_len(bytes, cursor, b'`');
                    cursor += run;
                    let mut scan = cursor;
                    while scan < bytes.len() {
                        let Some(off) = memchr(b'`', &bytes[scan..]) else {
                            break;
                        };
                        scan += off;
                        let closer = Self::marker_run_len(bytes, scan, b'`');
                        if closer == run {
                            cursor = scan + closer;
                            break;
                        }
                        scan += closer;
                    }
                }
                b'<' => {
                    if let Some(end) = super::inline::autolink_end(content, cursor) {
                        cursor = end;
                    } else if let Some((_, end)) = Parser::parse_inline_html(content, cursor, 0) {
                        cursor = end;
                    } else {
                        cursor += 1;
                    }
                }
                b'[' => {
                    depth += 1;
                    nested = true;
                    cursor += 1;
                }
                b']' => {
                    depth -= 1;
                    // Stop AT the closing delimiter.
                    if depth == 0 {
                        return (cursor, nested);
                    }
                    cursor += 1;
                }
                _ => cursor += 1,
            }
        }
        (cursor, nested)
    }
}

/// Bytes that can change the outcome of [`Parser::scan_balanced`]: the
/// escape and code-span markers, the start of an autolink or raw HTML tag,
/// and the brackets themselves.
static BRACKET_STOP: ByteClass = ByteClass::from_flags({
    let mut t = [0u8; 256];
    t[b'\\' as usize] = 1;
    t[b'`' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b']' as usize] = 1;
    t
});

#[cfg(test)]
mod tests;
