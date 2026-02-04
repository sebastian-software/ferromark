//! HTML escaping utilities.
//!
//! Fast-path optimized: scans for first escapable character,
//! then bulk-copies segments between escapes.

use memchr::{memchr, memchr2, memchr3};



/// Characters that need escaping in HTML text content.
#[allow(dead_code)]
const TEXT_ESCAPE_CHARS: &[u8] = b"<>&";

/// Characters that need escaping in HTML attribute values.
#[allow(dead_code)]
const ATTR_ESCAPE_CHARS: &[u8] = b"<>&\"'";

/// Lookup table for escapable characters in text content.
/// Index by byte value, true if needs escaping.
/// Note: We escape " as &quot; for CommonMark spec compliance.
const TEXT_ESCAPE_TABLE: [bool; 256] = {
    let mut table = [false; 256];
    table[b'<' as usize] = true;
    table[b'>' as usize] = true;
    table[b'&' as usize] = true;
    table[b'"' as usize] = true;
    table
};

/// Lookup table for escapable characters in attributes.
const ATTR_ESCAPE_TABLE: [bool; 256] = {
    let mut table = [false; 256];
    table[b'<' as usize] = true;
    table[b'>' as usize] = true;
    table[b'&' as usize] = true;
    table[b'"' as usize] = true;
    table[b'\'' as usize] = true;
    table
};

/// Escape HTML text content into output buffer.
///
/// Escapes `<`, `>`, and `&` to their HTML entity equivalents.
///
/// # Example
/// ```
/// use md_fast::escape::escape_text_into;
///
/// let mut out = Vec::new();
/// escape_text_into(&mut out, b"<script>");
/// assert_eq!(out, b"&lt;script&gt;");
/// ```
#[inline]
pub fn escape_text_into(out: &mut Vec<u8>, input: &[u8]) {
    escape_into_with_table(out, input, &TEXT_ESCAPE_TABLE)
}

/// Escape HTML attribute value into output buffer.
///
/// Escapes `<`, `>`, `&`, `"`, and `'` to their HTML entity equivalents.
///
/// # Example
/// ```
/// use md_fast::escape::escape_attr_into;
///
/// let mut out = Vec::new();
/// escape_attr_into(&mut out, b"value=\"test\"");
/// assert_eq!(out, b"value=&quot;test&quot;");
/// ```
#[inline]
pub fn escape_attr_into(out: &mut Vec<u8>, input: &[u8]) {
    escape_full_into(out, input)
}

/// Internal escaping with a custom lookup table.
#[inline]
fn escape_into_with_table(out: &mut Vec<u8>, input: &[u8], escape_table: &[bool; 256]) {
    if input.is_empty() {
        return;
    }

    let mut pos = match first_text_escape(input) {
        Some(p) => p,
        None => {
            out.extend_from_slice(input);
            return;
        }
    };

    if pos > 0 {
        out.extend_from_slice(&input[..pos]);
    }

    while pos < input.len() {
        // Scan for any escapable character using lookup table
        let scan_start = pos;
        while pos < input.len() && !escape_table[input[pos] as usize] {
            pos += 1;
        }

        // Copy non-escaped portion
        if pos > scan_start {
            out.extend_from_slice(&input[scan_start..pos]);
        }

        // Handle escape if found
        if pos < input.len() {
            let escape_seq = match input[pos] {
                b'<' => b"&lt;" as &[u8],
                b'>' => b"&gt;",
                b'&' => b"&amp;",
                b'"' => b"&quot;",
                b'\'' => b"&#39;",
                _ => {
                    // Shouldn't happen, but handle gracefully
                    out.push(input[pos]);
                    pos += 1;
                    continue;
                }
            };
            out.extend_from_slice(escape_seq);
            pos += 1;
        }
    }
}

/// Escape HTML text content, checking for quotes as well (for attribute context).
///
/// This version handles all 5 escapable characters.
#[inline]
pub fn escape_full_into(out: &mut Vec<u8>, input: &[u8]) {
    if input.is_empty() {
        return;
    }

    let mut pos = match first_attr_escape(input) {
        Some(p) => p,
        None => {
            out.extend_from_slice(input);
            return;
        }
    };

    if pos > 0 {
        out.extend_from_slice(&input[..pos]);
    }

    while pos < input.len() {
        // Scan for any escapable character using lookup table
        let scan_start = pos;
        while pos < input.len() && !ATTR_ESCAPE_TABLE[input[pos] as usize] {
            pos += 1;
        }

        // Copy non-escaped portion
        if pos > scan_start {
            out.extend_from_slice(&input[scan_start..pos]);
        }

        // Handle escape if found
        if pos < input.len() {
            let escape_seq = match input[pos] {
                b'<' => b"&lt;" as &[u8],
                b'>' => b"&gt;",
                b'&' => b"&amp;",
                b'"' => b"&quot;",
                b'\'' => b"&#39;",
                _ => unreachable!(),
            };
            out.extend_from_slice(escape_seq);
            pos += 1;
        }
    }
}

/// Check if a byte slice needs any escaping for text content.
#[inline]
pub fn needs_text_escape(input: &[u8]) -> bool {
    input.iter().any(|&b| TEXT_ESCAPE_TABLE[b as usize])
}

/// Check if a byte slice needs any escaping for attribute values.
#[inline]
pub fn needs_attr_escape(input: &[u8]) -> bool {
    input.iter().any(|&b| ATTR_ESCAPE_TABLE[b as usize])
}

#[inline]
fn first_text_escape(input: &[u8]) -> Option<usize> {
    let a = memchr3(b'<', b'>', b'&', input);
    let b = memchr(b'"', input);
    min_opt(a, b)
}

#[inline]
fn first_attr_escape(input: &[u8]) -> Option<usize> {
    let a = memchr3(b'<', b'>', b'&', input);
    let b = memchr2(b'"', b'\'', input);
    min_opt(a, b)
}

#[inline]
fn min_opt(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Escape and return as a new Vec.
///
/// Prefer `escape_text_into` to reuse buffers.
pub fn escape_text(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + input.len() / 8);
    escape_text_into(&mut out, input);
    out
}

/// Escape and return as a String.
///
/// Prefer `escape_text_into` to reuse buffers.
pub fn escape_text_to_string(input: &str) -> String {
    let escaped = escape_text(input.as_bytes());
    // SAFETY: We only add ASCII sequences, so if input was valid UTF-8,
    // output is also valid UTF-8
    unsafe { String::from_utf8_unchecked(escaped) }
}

/// URL percent-encode special characters, then HTML-escape for href attribute.
/// This is specifically for autolink URLs per CommonMark spec.
///
/// Check if a character is ASCII punctuation (can be backslash-escaped in URLs)
#[inline]
fn is_ascii_punctuation(b: u8) -> bool {
    matches!(b,
        b'!' | b'"' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'(' | b')' |
        b'*' | b'+' | b',' | b'-' | b'.' | b'/' | b':' | b';' | b'<' |
        b'=' | b'>' | b'?' | b'@' | b'[' | b'\\' | b']' | b'^' | b'_' |
        b'`' | b'{' | b'|' | b'}' | b'~'
    )
}

/// Process a link URL: decode entities, handle backslash escapes, and percent-encode.
/// This is used for link destinations in `[text](url)` syntax.
#[inline]
pub fn url_escape_link_destination(out: &mut Vec<u8>, input: &[u8]) {
    // First decode HTML entities
    let input_str = core::str::from_utf8(input).unwrap_or("");
    let decoded = html_escape::decode_html_entities(input_str);
    let decoded_bytes = decoded.as_bytes();

    url_escape_link_destination_raw(out, decoded_bytes);
}

/// Process a link URL without entity decoding (used after entities are already decoded).
#[inline]
fn url_escape_link_destination_raw(out: &mut Vec<u8>, input: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut pos = 0;
    while pos < input.len() {
        let b = input[pos];

        // Handle backslash escapes: \X where X is ASCII punctuation
        if b == b'\\' && pos + 1 < input.len() && is_ascii_punctuation(input[pos + 1]) {
            // Skip the backslash, encode the escaped character
            pos += 1;
            let escaped = input[pos];
            // The escaped character still needs HTML attribute escaping
            match escaped {
                b'<' => out.extend_from_slice(b"&lt;"),
                b'>' => out.extend_from_slice(b"&gt;"),
                b'&' => out.extend_from_slice(b"&amp;"),
                b'"' => out.extend_from_slice(b"%22"),
                b'\'' => out.extend_from_slice(b"&#39;"),
                _ => out.push(escaped),
            }
            pos += 1;
            continue;
        }

        // Handle characters that need encoding
        match b {
            // Characters that need URL percent-encoding
            b'\\' => out.extend_from_slice(b"%5C"),
            b' ' => out.extend_from_slice(b"%20"),
            b'"' => out.extend_from_slice(b"%22"),
            // Characters that need HTML escaping (but are valid in URLs)
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'&' => out.extend_from_slice(b"&amp;"),
            b'\'' => out.extend_from_slice(b"&#39;"),
            // Control characters (0x00-0x1F except tab, LF, CR) and 0x7F
            0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F => {
                out.push(b'%');
                out.push(HEX[(b >> 4) as usize]);
                out.push(HEX[(b & 0xF) as usize]);
            }
            // Non-ASCII bytes need percent-encoding
            0x80..=0xFF => {
                out.push(b'%');
                out.push(HEX[(b >> 4) as usize]);
                out.push(HEX[(b & 0xF) as usize]);
            }
            // Everything else passes through
            _ => out.push(b),
        }
        pos += 1;
    }
}

/// Characters that need percent-encoding in URLs:
/// - Backslash `\` → `%5C`
/// - `[` → `%5B`
/// - `]` → `%5D`
/// - Backtick → `%60`
/// - Control characters
/// - Non-ASCII characters
#[inline]
pub fn url_encode_then_html_escape(out: &mut Vec<u8>, input: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    for &b in input {
        match b {
            // Characters that need URL percent-encoding
            b'\\' => out.extend_from_slice(b"%5C"),
            b'[' => out.extend_from_slice(b"%5B"),
            b']' => out.extend_from_slice(b"%5D"),
            b'`' => out.extend_from_slice(b"%60"),
            b' ' => out.extend_from_slice(b"%20"),
            // Characters that need HTML escaping
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'&' => out.extend_from_slice(b"&amp;"),
            b'"' => out.extend_from_slice(b"&quot;"),
            b'\'' => out.extend_from_slice(b"&#39;"),
            // Control characters (0x00-0x1F except tab, LF, CR) and non-ASCII
            0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x80..=0xFF => {
                out.push(b'%');
                out.push(HEX[(b >> 4) as usize]);
                out.push(HEX[(b & 0xF) as usize]);
            }
            // Everything else passes through
            _ => out.push(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_text_basic() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"Hello, World!");
        assert_eq!(out, b"Hello, World!");
    }

    #[test]
    fn test_escape_text_lt() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"<script>");
        assert_eq!(out, b"&lt;script&gt;");
    }

    #[test]
    fn test_escape_text_gt() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"1 > 0");
        assert_eq!(out, b"1 &gt; 0");
    }

    #[test]
    fn test_escape_text_amp() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"a & b");
        assert_eq!(out, b"a &amp; b");
    }

    #[test]
    fn test_escape_text_mixed() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"<a href=\"test\">link & stuff</a>");
        assert_eq!(out, b"&lt;a href=&quot;test&quot;&gt;link &amp; stuff&lt;/a&gt;");
    }

    #[test]
    fn test_escape_text_empty() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"");
        assert_eq!(out, b"");
    }

    #[test]
    fn test_escape_attr_quotes() {
        let mut out = Vec::new();
        escape_full_into(&mut out, b"\"hello\"");
        assert_eq!(out, b"&quot;hello&quot;");
    }

    #[test]
    fn test_escape_attr_single_quote() {
        let mut out = Vec::new();
        escape_full_into(&mut out, b"it's");
        assert_eq!(out, b"it&#39;s");
    }

    #[test]
    fn test_escape_attr_all() {
        let mut out = Vec::new();
        escape_full_into(&mut out, b"<>&\"'");
        assert_eq!(out, b"&lt;&gt;&amp;&quot;&#39;");
    }

    #[test]
    fn test_needs_escape() {
        assert!(!needs_text_escape(b"hello"));
        assert!(needs_text_escape(b"<hello>"));
        assert!(needs_text_escape(b"a & b"));
        assert!(!needs_text_escape(b""));
    }

    #[test]
    fn test_escape_consecutive() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"<<<");
        assert_eq!(out, b"&lt;&lt;&lt;");
    }

    #[test]
    fn test_escape_at_boundaries() {
        let mut out = Vec::new();
        escape_text_into(&mut out, b"<");
        assert_eq!(out, b"&lt;");

        out.clear();
        escape_text_into(&mut out, b"hello<");
        assert_eq!(out, b"hello&lt;");

        out.clear();
        escape_text_into(&mut out, b"<hello");
        assert_eq!(out, b"&lt;hello");
    }

    #[test]
    fn test_escape_to_string() {
        let result = escape_text_to_string("<script>");
        assert_eq!(result, "&lt;script&gt;");
    }

    #[test]
    fn test_escape_unicode() {
        let mut out = Vec::new();
        escape_text_into(&mut out, "Hallo Welt! <tag>".as_bytes());
        assert_eq!(out, b"Hallo Welt! &lt;tag&gt;");
    }
}
