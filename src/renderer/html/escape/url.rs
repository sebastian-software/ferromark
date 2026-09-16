//! URL attribute escaping, percent encoding and IPv6 authority preservation.
//!
//! Use the shared scan/copy primitives without mixing URL policy into HTML text
//! escaping. Existing percent escapes and reserved URL delimiters stay intact.

use super::nibble::URL_ESCAPE_NIBBLES;
use super::{HIGH, first_flagged, has_zero, push_run, splat};

static URL_ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "%3C";
    table[b'>' as usize] = "%3E";
    table[b'"' as usize] = "%22";
    table[b' ' as usize] = "%20";
    table[b'\\' as usize] = "%5C";
    table[b'[' as usize] = "%5B";
    table[b']' as usize] = "%5D";
    table[b'`' as usize] = "%60";
    table
};

// Non-ASCII bytes are flagged too: they are percent encoded, so one scan
// finds both kinds of replacement instead of bounding ASCII runs first.
static URL_ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b' ' as usize] = 1;
    t[b'\\' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b']' as usize] = 1;
    t[b'`' as usize] = 1;
    let mut b = 0x80;
    while b < 256 {
        t[b] = 1;
        b += 1;
    }
    t
};

/// Nonzero iff `word` holds any URL-sensitive byte or a non-ASCII byte.
///
/// The first three tests retain the compact folds used by text escaping;
/// syntax-sensitive URL bytes use exact one-needle tests so reserved URL
/// delimiters such as `?`, `#`, and `:` remain untouched. The high bit of
/// every lane flags UTF-8 bytes, which are percent encoded.
#[inline]
const fn url_escape_mask(word: u64) -> u64 {
    (word & HIGH)
        | has_zero((word | splat(0x02)) ^ splat(b'>'))
        | has_zero((word | splat(0x02)) ^ splat(b'"'))
        | has_zero(word ^ splat(b'&'))
        | has_zero(word ^ splat(b'\\'))
        | has_zero(word ^ splat(b'['))
        | has_zero(word ^ splat(b']'))
        | has_zero(word ^ splat(b'`'))
}

/// Escapes URL syntax and UTF-8 bytes with one scan. The classifier flags
/// replacement bytes and non-ASCII bytes alike, so a URL is never walked
/// byte by byte just to bound its ASCII runs. Percent encoding operates on
/// the UTF-8 bytes, as required by cmark's URI renderer, and leaves existing
/// `%HH` sequences alone.
pub(super) fn write_url_segment(out: &mut String, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    loop {
        let i = next_url_flagged(bytes, start);
        if i >= bytes.len() {
            break;
        }
        if start < i {
            push_run(out, &s[start..i]);
        }
        let byte = bytes[i];
        if byte >= 0x80 {
            // `start` sits on a char boundary and this is the first non-ASCII
            // byte after it, so the run begins at a leading byte and ends at
            // the next ASCII byte or the end of the string.
            let unicode_end = bytes[i..]
                .iter()
                .position(|&byte| byte < 0x80)
                .map_or(bytes.len(), |offset| i + offset);
            for &byte in &bytes[i..unicode_end] {
                push_percent_byte(out, byte);
            }
            start = unicode_end;
        } else {
            push_run(out, URL_ESCAPE_TABLE[byte as usize]);
            start = i + 1;
        }
    }
    if start < bytes.len() {
        push_run(out, &s[start..]);
    }
}

/// Offset of the next URL byte that needs replacing or percent encoding.
///
/// The first scan of a segment goes straight to the vector scanner: most
/// URLs flag nothing. After a hit, a short table walk runs first, because
/// re-entering the scanner would reload its tables and read a full vector
/// for what is often a one- or two-byte gap. Entity- or Unicode-dense URLs
/// flag a byte every few positions and stayed scalar in the original
/// two-pass escaper; this keeps them there while a long clean run after a
/// hit still hands over to the vector scan after eight bytes.
#[inline]
fn next_url_flagged(bytes: &[u8], from: usize) -> usize {
    const SHORT_RUN_PREFIX: usize = 8;
    let mut i = from;
    if from != 0 {
        let quick = from.saturating_add(SHORT_RUN_PREFIX).min(bytes.len());
        while i < quick && URL_ESCAPE_FLAG[bytes[i] as usize] == 0 {
            i += 1;
        }
        if i < quick {
            return i;
        }
    }
    first_flagged(
        bytes,
        i,
        url_escape_mask,
        &URL_ESCAPE_FLAG,
        &URL_ESCAPE_NIBBLES,
    )
}

#[inline]
fn push_percent_byte(out: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(HEX[(byte >> 4) as usize] as char);
    out.push(HEX[(byte & 0x0F) as usize] as char);
}

/// Returns the bracket pair delimiting an IPv6 host in a URL authority.
/// Brackets elsewhere are data and remain subject to URL escaping.
pub(super) fn ipv6_authority_brackets(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    memchr::memchr(b'[', bytes)?;
    let authority_start = if bytes.starts_with(b"//") {
        2
    } else {
        let first = *bytes.first()?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        let mut scheme_end = 1;
        while let Some(&byte) = bytes.get(scheme_end) {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.') {
                scheme_end += 1;
            } else {
                break;
            }
        }
        (bytes.get(scheme_end..scheme_end + 3) == Some(b"://")).then_some(scheme_end + 3)?
    };
    let authority_end = bytes[authority_start..]
        .iter()
        .position(|&byte| matches!(byte, b'/' | b'?' | b'#'))
        .map_or(bytes.len(), |offset| authority_start + offset);
    let userinfo_end = bytes[authority_start..authority_end]
        .iter()
        .rposition(|&byte| byte == b'@')
        .map_or(authority_start, |offset| authority_start + offset + 1);
    let open = userinfo_end;
    if bytes.get(open) != Some(&b'[') {
        return None;
    }
    let close = bytes[open + 1..authority_end]
        .iter()
        .position(|&byte| byte == b']')?
        + open
        + 1;
    let host = std::str::from_utf8(&bytes[open + 1..close]).ok()?;
    host.parse::<std::net::Ipv6Addr>().ok()?;
    match bytes.get(close + 1..authority_end)? {
        [] => Some((open, close)),
        [b':', port @ ..] if port.iter().all(u8::is_ascii_digit) => Some((open, close)),
        _ => None,
    }
}

#[cfg(test)]
pub(super) mod tests;
