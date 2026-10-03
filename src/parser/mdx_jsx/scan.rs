//! JSX tag scanning: fragments, member names, named attrs, and spreads.

use smallvec::SmallVec;

use crate::allocator::Vec;
use crate::ast::{
    MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxAttributeValueExpression,
    MdxJsxExpressionAttribute, Span,
};

use super::super::line_scan::{is_line_ending_byte, line_terminator_end};
use super::braces::{skip_backticks, skip_quoted};

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

pub(super) fn looks_like_compatible_jsx_open(bytes: &[u8], at: usize) -> bool {
    bytes.get(at) == Some(&b'<')
        && bytes.get(at + 1).is_some_and(|byte| {
            byte.is_ascii_alphabetic() || matches!(byte, b'>' | b'_' | b'$') || *byte >= 0x80
        })
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

/// The opening tag at `start`, with its attributes.
///
/// `skip_brace` is the caller's memoized brace scan, as in
/// [`record_matching_closes`]: an attribute expression that never closes is
/// only known not to close once something has read to the end of the slice,
/// and every opening tag in a run asks before anything else does. 128 KiB
/// of `<A {>` took 1.3 s.
pub(super) fn scan_jsx_open<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
) -> Option<JsxOpen<'a>> {
    scan_jsx_open_mode(source, start, offset, attributes, skip_brace, false)
}

#[cfg(feature = "jsx")]
pub(super) fn scan_compatible_jsx_open<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
) -> Option<JsxOpen<'a>> {
    scan_jsx_open_mode(source, start, offset, attributes, skip_brace, true)
}

fn scan_jsx_open_mode<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    compatible: bool,
) -> Option<JsxOpen<'a>> {
    let bytes = source.as_bytes();
    if !(if compatible {
        looks_like_compatible_jsx_open(bytes, start)
    } else {
        looks_like_jsx_open(bytes, start)
    }) {
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
    let name_end = if compatible {
        scan_compatible_name(source, name_start)?
    } else {
        scan_jsx_name(bytes, name_start)?
    };
    let name = &source[name_start..name_end];
    let (self_closing, end) =
        scan_attributes(source, name_end, offset, attributes, skip_brace, compatible)?;
    Some(JsxOpen {
        name: Some(name),
        self_closing,
        end,
    })
}

/// Strict JSX balancing considers every element name, so crossed closing
/// tags cannot silently become HTML. Markdown code spans/fences and JS
/// expressions hide tags that belong to their source instead of JSX.
#[cfg(feature = "jsx")]
pub(super) fn compatible_matching_close<'a>(
    source: &'a str,
    from: usize,
    name: Option<&'a str>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
) -> Result<Option<(usize, usize)>, usize> {
    let bytes = source.as_bytes();
    let mut names: SmallVec<[Option<&str>; 16]> = SmallVec::new();
    names.push(name);
    let mut cursor = from;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor = (cursor + 2).min(bytes.len()),
            b'{' => cursor = skip_brace(cursor).ok_or(cursor)?,
            b'`' | b'~' => {
                cursor = compatible_fence_end(source, cursor)
                    .or_else(|| {
                        (bytes[cursor] == b'`')
                            .then(|| skip_backticks(bytes, cursor))
                            .flatten()
                    })
                    .unwrap_or(cursor + 1);
            }
            b'<' => {
                if let Some(end) = super::super::inline::autolink_end(source, cursor) {
                    cursor = end;
                    continue;
                }
                let closing = bytes.get(cursor + 1) == Some(&b'/');
                let start = cursor;
                let name_start = cursor + 1 + usize::from(closing);
                let (tag_name, name_end) = if bytes.get(name_start) == Some(&b'>') {
                    (None, name_start)
                } else if let Some(end) = scan_compatible_name(source, name_start) {
                    (Some(&source[name_start..end]), end)
                } else {
                    cursor += 1;
                    continue;
                };
                let mut end = name_end;
                let self_closing = if closing {
                    skip_ws(bytes, &mut end);
                    if bytes.get(end) != Some(&b'>') {
                        return Err(start);
                    }
                    end += 1;
                    false
                } else {
                    skip_tag_rest(source, &mut end, skip_brace, true).ok_or(start)?
                };
                if closing {
                    if names.pop() != Some(tag_name) {
                        return Err(start);
                    }
                    if names.is_empty() {
                        return Ok(Some((start, end)));
                    }
                } else if !self_closing {
                    names.push(tag_name);
                }
                cursor = end;
            }
            _ => cursor += 1,
        }
    }
    Ok(None)
}

#[cfg(feature = "jsx")]
pub(super) fn compatible_fence_end(source: &str, at: usize) -> Option<usize> {
    use super::super::line_scan::{line_end, next_line_start};
    use super::super::reference::{fence_open, is_fence_close};
    let bytes = source.as_bytes();
    let start = bytes[..at]
        .iter()
        .rposition(|byte| matches!(byte, b'\n' | b'\r'))
        .map_or(0, |position| position + 1);
    if !bytes[start..at]
        .iter()
        .all(|byte| matches!(byte, b' ' | b'\t'))
    {
        return None;
    }
    let (character, length) = fence_open(&source[at..line_end(bytes, at)])?;
    let mut cursor = next_line_start(bytes, at);
    while cursor < bytes.len() {
        let end = line_end(bytes, cursor);
        let mut content = cursor;
        while content < end && matches!(bytes[content], b' ' | b'\t') {
            content += 1;
        }
        if is_fence_close(&source[content..end], character, length) {
            return Some(next_line_start(bytes, cursor));
        }
        cursor = next_line_start(bytes, cursor);
    }
    Some(bytes.len())
}

fn scan_compatible_name(source: &str, start: usize) -> Option<usize> {
    let mut chars = source.get(start..)?.char_indices();
    let (_, first) = chars.next()?;
    if !(first.is_alphabetic() || matches!(first, '_' | '$')) {
        return None;
    }
    let mut end = start + first.len_utf8();
    for (relative, character) in chars {
        if !(character.is_alphanumeric() || matches!(character, '_' | '$' | '-' | ':' | '.')) {
            break;
        }
        end = start + relative + character.len_utf8();
    }
    Some(end)
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
                let Some(tag) = scan_tag_skip(source, cursor, skip_brace) else {
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
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    compatible: bool,
) -> Option<(bool, usize)> {
    let bytes = source.as_bytes();
    loop {
        let had_ws = skip_ws(bytes, &mut cursor);
        match bytes.get(cursor)? {
            b'/' if bytes.get(cursor + 1) == Some(&b'>') => return Some((true, cursor + 2)),
            b'>' => return Some((false, cursor + 1)),
            b'{' if had_ws => {
                cursor = push_expression_attribute(source, cursor, offset, attributes, skip_brace)?;
            }
            _ if !had_ws => return None,
            _ => {
                cursor = push_named_attribute(
                    source, cursor, offset, attributes, skip_brace, compatible,
                )?;
            }
        }
    }
}

fn push_expression_attribute<'a>(
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
) -> Option<usize> {
    let end = skip_brace(start)?;
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
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    compatible: bool,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let name_end = if compatible {
        scan_compatible_name(source, start)?
    } else {
        scan_attr_name(bytes, start)?
    };
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
    let (value, value_end) = scan_attr_value(source, cursor, offset, skip_brace, compatible)?;
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
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    compatible: bool,
) -> Option<(MdxJsxAttributeValue<'a>, usize)> {
    let bytes = source.as_bytes();
    match *bytes.get(start)? {
        b'"' | b'\'' => {
            let end = if compatible {
                skip_jsx_quoted(bytes, start)?
            } else {
                skip_quoted(bytes, start)?
            };
            Some((
                MdxJsxAttributeValue::Literal(&source[start + 1..end - 1]),
                end,
            ))
        }
        b'{' => {
            let end = skip_brace(start)?;
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

fn scan_tag_skip<'a>(
    source: &'a str,
    start: usize,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
) -> Option<TagSkip<'a>> {
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
    let self_closing = skip_tag_rest(source, &mut cursor, skip_brace, false)?;
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

fn skip_tag_rest(
    source: &str,
    cursor: &mut usize,
    skip_brace: &mut impl FnMut(usize) -> Option<usize>,
    compatible: bool,
) -> Option<bool> {
    let bytes = source.as_bytes();
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
            b'{' => *cursor = skip_brace(*cursor)?,
            b'"' | b'\'' | b'`' => {
                *cursor = if compatible {
                    skip_jsx_quoted(bytes, *cursor)?
                } else {
                    skip_quoted(bytes, *cursor)?
                }
            }
            byte if is_attr_name_start(*byte) || (compatible && *byte >= 0x80) => {
                *cursor = if compatible {
                    scan_compatible_name(source, *cursor)?
                } else {
                    scan_attr_name(bytes, *cursor)?
                };
                skip_ws(bytes, cursor);
                if bytes.get(*cursor) == Some(&b'=') {
                    *cursor += 1;
                    skip_ws(bytes, cursor);
                    match bytes.get(*cursor)? {
                        b'"' | b'\'' => {
                            *cursor = if compatible {
                                skip_jsx_quoted(bytes, *cursor)?
                            } else {
                                skip_quoted(bytes, *cursor)?
                            }
                        }
                        b'{' => *cursor = skip_brace(*cursor)?,
                        _ => skip_unquoted_value(bytes, cursor),
                    }
                }
            }
            _ => return None,
        }
    }
}

fn skip_jsx_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    memchr::memchr(quote, &bytes[start + 1..]).map(|relative| start + relative + 2)
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
