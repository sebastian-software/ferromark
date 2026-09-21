//! JSX tag scanning: fragments, member names, named attrs, and spreads.

use smallvec::SmallVec;

use crate::allocator::Vec;
use crate::ast::{
    MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxAttributeValueExpression,
    MdxJsxExpressionAttribute, Span,
};

use super::super::line_scan::{is_line_ending_byte, line_terminator_end};
use super::braces::{skip_backticks, skip_braces, skip_quoted};

/// Opening tag accepted by this slice.
pub(super) struct JsxOpen<'a> {
    pub name: Option<&'a str>,
    pub self_closing: bool,
    pub end: usize,
}

struct TagSkip<'a> {
    name: Option<&'a str>,
    start: usize,
    closing: bool,
    self_closing: bool,
    end: usize,
}

#[inline]
pub(super) fn looks_like_jsx_open(bytes: &[u8], at: usize) -> bool {
    bytes.get(at) == Some(&b'<')
        && match bytes.get(at + 1) {
            Some(b'>') => true,
            Some(byte) => byte.is_ascii_uppercase(),
            None => false,
        }
}

pub(super) fn only_ws_until_eol(bytes: &[u8], mut cursor: usize) -> bool {
    while cursor < bytes.len() {
        match bytes[cursor] {
            b' ' | b'\t' => cursor += 1,
            byte if is_line_ending_byte(byte) => return true,
            _ => return false,
        }
    }
    true
}

pub(super) fn after_trailing_line_ws(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
        cursor += 1;
    }
    if cursor < bytes.len() && is_line_ending_byte(bytes[cursor]) {
        line_terminator_end(bytes, cursor)
    } else {
        cursor
    }
}

pub(super) fn scan_jsx_open<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<JsxOpen<'a>> {
    let bytes = source.as_bytes();
    if !looks_like_jsx_open(bytes, start) {
        return None;
    }
    if bytes.get(start + 1) == Some(&b'>') {
        return Some(JsxOpen {
            name: None,
            self_closing: false,
            end: start + 2,
        });
    }
    let name_start = start + 1;
    let name_end = scan_jsx_name(bytes, name_start)?;
    let name = &source[name_start..name_end];
    let (self_closing, end) = scan_attributes(source, name_end, offset, attributes)?;
    Some(JsxOpen {
        name: Some(name),
        self_closing,
        end,
    })
}

/// The closing tag that matches the opening tag ending at `from`, found by
/// a walk that keeps nothing.
///
/// The parse takes the same answer from [`record_matching_closes`], which
/// keeps what it passes; this is the plain walk the tests hold that record
/// against.
#[cfg(test)]
pub(super) fn find_matching_close(
    source: &str,
    from: usize,
    name: Option<&str>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
) -> Option<(usize, usize)> {
    walk_close::<false>(source, from, name, skip_brace, &mut |_, _| {}).0
}

/// What one closing-tag walk settled.
pub(super) struct CloseWalk {
    /// The closing tag that matched the opening tag the walk started from.
    pub close: Option<(usize, usize)>,
    /// The range the walk read without stepping over anything that could
    /// hide an opening tag. It is only meaningful when `close` is `None`:
    /// an opener in that range which `matched` did not report is an opener
    /// nothing closes.
    pub read: (usize, usize),
}

/// The closing tag that matches the opening tag ending at `from`, recording
/// the same answer for every opener the walk passes.
///
/// `skip_brace` reports the byte after the `}` that closes the `{` at a
/// position, or `None` when nothing does. It is the caller's memoized scan:
/// a brace that closes nothing is only known to close nothing once
/// something has read to the end of the slice, so a run of them inside one
/// walk cost one read each — 128 KiB of `<A>` over `{` with a single `}`
/// behind it took 10.7 s.
///
/// An opening tag only becomes a node once something closes it, and `<A>`
/// repeated nests one level per tag: every opener but the innermost is
/// unclosed, and each one used to walk the rest of the slice for itself.
/// Every decision below depends on the position alone and never on where
/// the walk began, so the closing tag that returns the walk to an opener's
/// own depth is the one a walk starting at that opener stops at, and an
/// opener this walk leaves open is one such a walk leaves open too. One
/// walk therefore decides the whole run.
///
/// The openers left open at the end are reported as a range rather than one
/// by one, so the record of a long run costs what the walk does. The range
/// starts after the last region the walk stepped over that could hold a
/// `<` — an expression, a backtick run, a tag whose own body holds one —
/// because an opener in there is an opener this walk never scanned. Open
/// tags before that point are reported individually, so a run interleaved
/// with such regions still costs one walk.
///
/// The opener the walk starts from is not reported: its answer is the
/// return value, and a document of independent elements would otherwise pay
/// a record for every one of them.
pub(super) fn record_matching_closes(
    source: &str,
    from: usize,
    name: Option<&str>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    matched: &mut impl FnMut(usize, Option<(usize, usize)>),
) -> CloseWalk {
    let (close, read) = walk_close::<true>(source, from, name, skip_brace, matched);
    CloseWalk { close, read }
}

/// The closing-tag walk both scans above are.
///
/// With `RECORD` the walk keeps the opener stack it otherwise only counts,
/// and reports the openers it passes: closed at the tag that returns the
/// walk to that opener's depth, open at the end of the slice.
fn walk_close<const RECORD: bool>(
    source: &str,
    from: usize,
    name: Option<&str>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    matched: &mut impl FnMut(usize, Option<(usize, usize)>),
) -> (Option<(usize, usize)>, (usize, usize)) {
    let bytes = source.as_bytes();
    // Deep enough for any nesting a document holds by hand; the runs that
    // go past it are the ones the record exists for, and they spill once.
    // Never touched without `RECORD`.
    let mut open: SmallVec<[usize; 16]> = SmallVec::new();
    if RECORD {
        open.push(from);
    }
    let mut cursor = from;
    let mut depth = 1u32;
    let mut read_from = from;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'{' => {
                if let Some(next) = skip_brace(cursor) {
                    cursor = next;
                    read_from = next;
                } else {
                    cursor += 1;
                }
            }
            b'`' => {
                if let Some(next) = skip_backticks(bytes, cursor) {
                    cursor = next;
                    read_from = next;
                } else {
                    cursor += 1;
                }
            }
            b'<' => {
                let Some(tag) = scan_tag_skip(source, cursor) else {
                    cursor += 1;
                    continue;
                };
                if tag.name == name {
                    if tag.closing {
                        depth = depth.saturating_sub(1);
                        if RECORD
                            && let Some(opener) = open.pop()
                            && opener != from
                        {
                            matched(opener, Some((tag.start, tag.end)));
                        }
                        if depth == 0 {
                            return (Some((tag.start, tag.end)), (from, tag.end));
                        }
                    } else if !tag.self_closing {
                        depth = depth.saturating_add(1);
                        if RECORD {
                            open.push(tag.end);
                        }
                    }
                }
                // A tag whose own body holds a `<` hides whatever that `<`
                // starts, so the run of scanned bytes restarts after it.
                if memchr::memchr(b'<', &bytes[cursor + 1..tag.end]).is_some() {
                    read_from = tag.end;
                }
                cursor = tag.end;
            }
            _ => cursor += 1,
        }
    }
    if RECORD {
        for opener in open {
            if opener != from && opener < read_from {
                matched(opener, None);
            }
        }
    }
    (None, (read_from, cursor))
}

fn scan_jsx_name(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes.get(start)?.is_ascii_uppercase() {
        return None;
    }
    scan_member_name(bytes, start)
}

fn scan_attributes<'a>(
    source: &'a str,
    mut cursor: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<(bool, usize)> {
    let bytes = source.as_bytes();
    loop {
        let had_ws = skip_ws(bytes, &mut cursor);
        match bytes.get(cursor)? {
            b'/' if bytes.get(cursor + 1) == Some(&b'>') => return Some((true, cursor + 2)),
            b'>' => return Some((false, cursor + 1)),
            b'{' if had_ws => {
                cursor = push_expression_attribute(source, cursor, offset, attributes)?;
            }
            _ if !had_ws => return None,
            _ => cursor = push_named_attribute(source, cursor, offset, attributes)?,
        }
    }
}

fn push_expression_attribute<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<usize> {
    let end = skip_braces(source.as_bytes(), start)?;
    attributes.push(MdxJsxAttributeEntry::Expression(
        MdxJsxExpressionAttribute {
            value: &source[start + 1..end - 1],
            span: Span::new((offset + start) as u32, (offset + end) as u32),
        },
    ));
    Some(end)
}

fn push_named_attribute<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let name_end = scan_attr_name(bytes, start)?;
    let name = &source[start..name_end];
    let mut cursor = name_end;
    skip_ws(bytes, &mut cursor);
    if bytes.get(cursor) != Some(&b'=') {
        attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name,
            value: None,
            span: Span::new((offset + start) as u32, (offset + name_end) as u32),
        }));
        return Some(name_end);
    }
    cursor += 1;
    skip_ws(bytes, &mut cursor);
    let (value, value_end) = scan_attr_value(source, cursor, offset)?;
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name,
        value: Some(value),
        span: Span::new((offset + start) as u32, (offset + value_end) as u32),
    }));
    Some(value_end)
}

fn scan_attr_value<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
) -> Option<(MdxJsxAttributeValue<'a>, usize)> {
    let bytes = source.as_bytes();
    match *bytes.get(start)? {
        b'"' | b'\'' => {
            let end = skip_quoted(bytes, start)?;
            Some((
                MdxJsxAttributeValue::Literal(&source[start + 1..end - 1]),
                end,
            ))
        }
        b'{' => {
            let end = skip_braces(bytes, start)?;
            Some((
                MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
                    value: &source[start + 1..end - 1],
                    span: Span::new((offset + start) as u32, (offset + end) as u32),
                }),
                end,
            ))
        }
        _ => None,
    }
}

fn scan_tag_skip(source: &str, start: usize) -> Option<TagSkip<'_>> {
    let bytes = source.as_bytes();
    if bytes.get(start)? != &b'<' {
        return None;
    }
    let mut cursor = start + 1;
    let closing = bytes.get(cursor) == Some(&b'/');
    if closing {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'>') {
        return Some(TagSkip {
            name: None,
            start,
            closing,
            self_closing: false,
            end: cursor + 1,
        });
    }
    let name_end = scan_member_name(bytes, cursor)?;
    let name = &source[cursor..name_end];
    cursor = name_end;
    let self_closing = skip_tag_rest(bytes, &mut cursor)?;
    if closing && self_closing {
        return None;
    }
    Some(TagSkip {
        name: Some(name),
        start,
        closing,
        self_closing,
        end: cursor,
    })
}

fn skip_tag_rest(bytes: &[u8], cursor: &mut usize) -> Option<bool> {
    loop {
        skip_ws(bytes, cursor);
        match bytes.get(*cursor)? {
            b'/' if bytes.get(*cursor + 1) == Some(&b'>') => {
                *cursor += 2;
                return Some(true);
            }
            b'>' => {
                *cursor += 1;
                return Some(false);
            }
            b'{' => *cursor = skip_braces(bytes, *cursor)?,
            b'"' | b'\'' | b'`' => *cursor = skip_quoted(bytes, *cursor)?,
            byte if is_attr_name_start(*byte) => {
                *cursor = scan_attr_name(bytes, *cursor)?;
                skip_ws(bytes, cursor);
                if bytes.get(*cursor) == Some(&b'=') {
                    *cursor += 1;
                    skip_ws(bytes, cursor);
                    match bytes.get(*cursor)? {
                        b'"' | b'\'' => *cursor = skip_quoted(bytes, *cursor)?,
                        b'{' => *cursor = skip_braces(bytes, *cursor)?,
                        _ => skip_unquoted_value(bytes, cursor),
                    }
                }
            }
            _ => return None,
        }
    }
}

fn scan_member_name(bytes: &[u8], start: usize) -> Option<usize> {
    let mut end = scan_ident(bytes, start)?;
    while bytes.get(end) == Some(&b'.') {
        end = scan_ident(bytes, end + 1)?;
    }
    Some(end)
}

fn scan_ident(bytes: &[u8], start: usize) -> Option<usize> {
    let first = *bytes.get(start)?;
    if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b'$')) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'$'))
    {
        end += 1;
    }
    Some(end)
}

fn scan_attr_name(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes
        .get(start)
        .is_some_and(|byte| is_attr_name_start(*byte))
    {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric()
            || matches!(bytes[end], b'_' | b'$' | b'-' | b':' | b'.'))
    {
        end += 1;
    }
    Some(end)
}

#[inline]
fn is_attr_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn skip_ws(bytes: &[u8], cursor: &mut usize) -> bool {
    let start = *cursor;
    while *cursor < bytes.len() && matches!(bytes[*cursor], b' ' | b'\t' | b'\n' | b'\r') {
        *cursor += 1;
    }
    *cursor > start
}

fn skip_unquoted_value(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len()
        && !matches!(
            bytes[*cursor],
            b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'' | b'=' | b'<' | b'>' | b'`' | b'/'
        )
    {
        *cursor += 1;
    }
}

#[cfg(test)]
mod tests;
