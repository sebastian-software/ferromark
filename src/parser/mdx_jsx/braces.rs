//! Brace, string, and comment skip for JSX tags and MDX expressions.
//!
//! This is not a JavaScript parser. Unclosed input returns `None` so the
//! caller can refuse to emit a half-parsed node.

use smallvec::SmallVec;

/// What one brace walk settled.
pub(super) struct BraceWalk {
    /// Byte after the `}` that closes the brace the walk started at.
    pub close: Option<usize>,
    /// The range the walk read byte by byte, after the last region it
    /// stepped over whole. It is only meaningful when `close` is `None`:
    /// the walk saw every `{` in it, so one that `matched` did not report
    /// is a `{` nothing closes.
    pub read: (usize, usize),
}

/// [`skip_braces`] for the brace at `start` and for every brace the walk
/// passes on the way, reported through `matched`.
///
/// A `{` that closes nothing is only known to close nothing once the walk
/// reaches the end of the content, so a run of them cost one walk each —
/// and one `}` anywhere behind the run is enough to defeat a cheap
/// last-closer guard in front of it: 128 KiB of `{` followed by a single
/// `}` took 5.4 s. Every decision below depends on the position alone and
/// never on where the walk began, so the `}` that returns the walk to a
/// brace's own depth is the `}` a walk starting at that brace stops at, and
/// a brace this walk leaves open is one such a walk leaves open too. One
/// walk therefore decides the whole run.
///
/// A run of thousands of braces that close nothing is reported as one
/// range instead of one record each. The range starts after the last string
/// or comment the walk stepped over, because a `{` inside one of those is a
/// `{` this walk never looked at; the open braces before it are reported
/// one by one, so a run interleaved with strings still costs one walk. The
/// brace the walk starts from is never reported: its answer is the return
/// value.
pub(super) fn record_brace_matches(
    bytes: &[u8],
    start: usize,
    matched: &mut impl FnMut(usize, Option<usize>),
) -> BraceWalk {
    if bytes.get(start) != Some(&b'{') {
        return BraceWalk {
            close: None,
            read: (start, start),
        };
    }
    // Deep enough for any nesting an expression holds by hand; the runs
    // that go past it are the ones the record exists for.
    let mut open: SmallVec<[usize; 16]> = SmallVec::new();
    let mut cursor = start;
    let mut read_from = start;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' | b'\'' | b'`' => match skip_quoted(bytes, cursor) {
                Some(end) => {
                    cursor = end;
                    read_from = end;
                }
                // An unterminated string ends every walk that reaches it,
                // wherever it started.
                None => break,
            },
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(bytes, cursor);
                read_from = cursor;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => match skip_block_comment(bytes, cursor)
            {
                Some(end) => {
                    cursor = end;
                    read_from = end;
                }
                None => break,
            },
            b'{' => {
                open.push(cursor);
                cursor += 1;
            }
            b'}' => {
                cursor += 1;
                if let Some(brace) = open.pop() {
                    if open.is_empty() {
                        // The brace the walk started from: its answer is
                        // the return value, so a document of expressions
                        // that close at once keeps no record at all.
                        return BraceWalk {
                            close: Some(cursor),
                            read: (start, cursor),
                        };
                    }
                    matched(brace, Some(cursor));
                }
            }
            _ => cursor += 1,
        }
    }
    for brace in open {
        if brace != start && brace < read_from {
            matched(brace, None);
        }
    }
    BraceWalk {
        close: None,
        read: (read_from, cursor),
    }
}

/// The byte after the `}` that closes the `{` at `start`, found by a walk
/// that keeps nothing.
///
/// Every scan in the parse takes that answer from
/// [`record_brace_matches`], which keeps what it passes; this is the plain
/// walk the tests hold that record against.
#[cfg(test)]
pub(super) fn skip_braces(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'{') {
        return None;
    }
    let mut cursor = start;
    let mut depth = 0u32;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' | b'\'' | b'`' => cursor = skip_quoted(bytes, cursor)?,
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(bytes, cursor);
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                cursor = skip_block_comment(bytes, cursor)?;
            }
            b'{' => {
                depth = depth.saturating_add(1);
                cursor += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                cursor += 1;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}

pub(super) fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

pub(super) fn skip_backticks(bytes: &[u8], start: usize) -> Option<usize> {
    let mut open = 0usize;
    while start + open < bytes.len() && bytes[start + open] == b'`' {
        open += 1;
    }
    if open == 0 {
        return None;
    }
    let mut cursor = start + open;
    while cursor + open <= bytes.len() {
        if bytes[cursor..cursor + open]
            .iter()
            .all(|byte| *byte == b'`')
            && bytes.get(cursor + open) != Some(&b'`')
        {
            return Some(cursor + open);
        }
        cursor += 1;
    }
    None
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 2;
    while cursor < bytes.len() && !matches!(bytes[cursor], b'\n' | b'\r') {
        cursor += 1;
    }
    cursor
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 2;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    None
}

#[cfg(test)]
mod tests;
