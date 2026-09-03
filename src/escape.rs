//! HTML escaping utilities.
//!
//! Fast-path optimized: scans for first escapable character,
//! then bulk-copies segments between escapes.

use memchr::{memchr, memchr2, memchr3};

/// Escape HTML text content into output buffer.
///
/// Escapes `<`, `>`, and `&` to their HTML entity equivalents.
///
/// # Example
/// ```
/// use ferromark::escape_text_into;
///
/// let mut out = Vec::new();
/// escape_text_into(&mut out, b"<script>");
/// assert_eq!(out, b"&lt;script&gt;");
/// ```
#[inline]
pub fn escape_text_into(out: &mut Vec<u8>, input: &[u8]) {
    if input.is_empty() {
        return;
    }

    let mut start = 0usize;
    while let Some(rel) = first_text_escape(&input[start..]) {
        let pos = start + rel;
        if pos > start {
            out.extend_from_slice(&input[start..pos]);
        }
        push_text_escape(out, input[pos]);
        start = pos + 1;
    }
    if start < input.len() {
        out.extend_from_slice(&input[start..]);
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

    let mut start = 0usize;
    while let Some(rel) = first_attr_escape(&input[start..]) {
        let pos = start + rel;
        if pos > start {
            out.extend_from_slice(&input[start..pos]);
        }
        push_attr_escape(out, input[pos]);
        start = pos + 1;
    }
    if start < input.len() {
        out.extend_from_slice(&input[start..]);
    }
}

/// Escape HTML attribute value into output buffer.
///
/// Escapes `<`, `>`, `&`, `"`, and `'` to their HTML entity equivalents.
///
/// # Example
/// ```
/// use ferromark::escape_attr_into;
///
/// let mut out = Vec::new();
/// escape_attr_into(&mut out, b"value=\"test\"");
/// assert_eq!(out, b"value=&quot;test&quot;");
/// ```
#[inline]
pub fn escape_attr_into(out: &mut Vec<u8>, input: &[u8]) {
    escape_full_into(out, input)
}

/// Below this length the local short scan beats SIMD memchr passes, whose
/// per-call dispatch and setup cost more than the scan itself. Inline text
/// segments between escapes are short in practice (median ~17 bytes, >95%
/// under 64 on prose corpora); long inputs (code block content) still go
/// through memchr, which scans wider per iteration.
const SHORT_SCAN_MAX: usize = 128;

#[inline]
fn first_text_escape(input: &[u8]) -> Option<usize> {
    if input.len() <= SHORT_SCAN_MAX {
        return first_escape_short::<false>(input);
    }
    let a = memchr3(b'<', b'>', b'&', input);
    // A '"' is only relevant if it appears before the first <>& hit, so the
    // second pass never scans past it.
    let limit = a.unwrap_or(input.len());
    memchr(b'"', &input[..limit]).or(a)
}

#[inline]
fn first_attr_escape(input: &[u8]) -> Option<usize> {
    if input.len() <= SHORT_SCAN_MAX {
        return first_escape_short::<true>(input);
    }
    let a = memchr3(b'<', b'>', b'&', input);
    let limit = a.unwrap_or(input.len());
    memchr2(b'"', b'\'', &input[..limit]).or(a)
}

/// Find the first escapable byte (`<>&"`, plus `'` when `ATTR`) in a short
/// input, without memchr's per-call runtime dispatch: SSE2 is baseline on
/// x86_64 and NEON on AArch64, so both compile to direct SIMD with no
/// feature detection. Other architectures fall back to a table scan.
#[inline]
fn first_escape_short<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    return first_escape_sse2::<ATTR>(input);
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    return unsafe { first_escape_neon::<ATTR>(input) };
    #[cfg(not(any(
        target_arch = "x86_64",
        all(target_arch = "aarch64", target_feature = "neon")
    )))]
    {
        let table = if ATTR {
            &ATTR_ESCAPE_TABLE
        } else {
            &TEXT_ESCAPE_TABLE
        };
        input.iter().position(|&b| table[b as usize])
    }
}

/// Bitmask of lanes equal to any escapable byte in a 16-byte SSE2 vector.
#[cfg(target_arch = "x86_64")]
#[inline]
fn escape_mask_sse2<const ATTR: bool>(v: std::arch::x86_64::__m128i) -> u32 {
    use std::arch::x86_64::*;
    // SAFETY: SSE2 is part of the x86_64 baseline.
    unsafe {
        let lt = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'<' as i8));
        let gt = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'>' as i8));
        let amp = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'&' as i8));
        let quot = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'"' as i8));
        let mut m = _mm_or_si128(_mm_or_si128(lt, gt), _mm_or_si128(amp, quot));
        if ATTR {
            m = _mm_or_si128(m, _mm_cmpeq_epi8(v, _mm_set1_epi8(b'\'' as i8)));
        }
        _mm_movemask_epi8(m) as u32
    }
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn first_escape_sse2<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;
    let len = input.len();
    if len < 16 {
        // Pad into a stack buffer; NUL is never an escape byte, so the
        // padding cannot produce a hit.
        let mut buf = [0u8; 16];
        buf[..len].copy_from_slice(input);
        // SAFETY: `buf` is 16 bytes; unaligned load is allowed.
        let v = unsafe { _mm_loadu_si128(buf.as_ptr().cast()) };
        let m = escape_mask_sse2::<ATTR>(v);
        return (m != 0).then(|| m.trailing_zeros() as usize);
    }
    let mut pos = 0;
    while pos + 16 <= len {
        // SAFETY: `pos + 16 <= len` bounds the unaligned 16-byte load.
        let v = unsafe { _mm_loadu_si128(input.as_ptr().add(pos).cast()) };
        let m = escape_mask_sse2::<ATTR>(v);
        if m != 0 {
            return Some(pos + m.trailing_zeros() as usize);
        }
        pos += 16;
    }
    if pos < len {
        // Overlapping final chunk; lanes before `pos` were already scanned
        // clean, so any set bit belongs to the unscanned tail.
        // SAFETY: `len >= 16` in this branch, so `len - 16` is in bounds.
        let v = unsafe { _mm_loadu_si128(input.as_ptr().add(len - 16).cast()) };
        let m = escape_mask_sse2::<ATTR>(v);
        if m != 0 {
            return Some(len - 16 + m.trailing_zeros() as usize);
        }
    }
    None
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
fn is_escape_byte<const ATTR: bool>(b: u8) -> bool {
    matches!(b, b'<' | b'>' | b'&' | b'"') || (ATTR && b == b'\'')
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn first_escape_neon<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    use std::arch::aarch64::*;

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn chunk_hits<const ATTR: bool>(v: uint8x16_t) -> bool {
        let lt = vceqq_u8(v, vdupq_n_u8(b'<'));
        let gt = vceqq_u8(v, vdupq_n_u8(b'>'));
        let amp = vceqq_u8(v, vdupq_n_u8(b'&'));
        let quot = vceqq_u8(v, vdupq_n_u8(b'"'));
        let mut m = vorrq_u8(vorrq_u8(lt, gt), vorrq_u8(amp, quot));
        if ATTR {
            m = vorrq_u8(m, vceqq_u8(v, vdupq_n_u8(b'\'')));
        }
        vmaxvq_u8(m) != 0
    }

    let len = input.len();
    if len < 16 {
        // Pad into a stack buffer; NUL is never an escape byte.
        let mut buf = [0u8; 16];
        buf[..len].copy_from_slice(input);
        let v = vld1q_u8(buf.as_ptr());
        if chunk_hits::<ATTR>(v) {
            return input.iter().position(|&b| is_escape_byte::<ATTR>(b));
        }
        return None;
    }
    let mut pos = 0;
    while pos + 16 <= len {
        let v = vld1q_u8(input.as_ptr().add(pos));
        if chunk_hits::<ATTR>(v) {
            for (i, &b) in input[pos..pos + 16].iter().enumerate() {
                if is_escape_byte::<ATTR>(b) {
                    return Some(pos + i);
                }
            }
        }
        pos += 16;
    }
    if pos < len {
        // Overlapping final chunk; lanes before `pos` were already scanned
        // clean, so the first scalar hit is the overall first.
        let v = vld1q_u8(input.as_ptr().add(len - 16));
        if chunk_hits::<ATTR>(v) {
            for (i, &b) in input[pos..].iter().enumerate() {
                if is_escape_byte::<ATTR>(b) {
                    return Some(pos + i);
                }
            }
        }
    }
    None
}

/// URL percent-encode special characters, then HTML-escape for href attribute.
/// This is specifically for autolink URLs per CommonMark spec.
///
/// Check if a character is ASCII punctuation (can be backslash-escaped in URLs)
#[inline]
fn is_ascii_punctuation(b: u8) -> bool {
    matches!(
        b,
        b'!' | b'"'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b'-'
            | b'.'
            | b'/'
            | b':'
            | b';'
            | b'<'
            | b'='
            | b'>'
            | b'?'
            | b'@'
            | b'['
            | b'\\'
            | b']'
            | b'^'
            | b'_'
            | b'`'
            | b'{'
            | b'|'
            | b'}'
            | b'~'
    )
}

/// Process a link URL: decode entities, handle backslash escapes, and percent-encode.
/// This is used for link destinations in `[text](url)` syntax.
#[inline]
pub fn url_escape_link_destination(out: &mut Vec<u8>, input: &[u8]) {
    if memchr(b'&', input).is_none() {
        url_escape_link_destination_raw(out, input);
        return;
    }

    // First decode HTML entities
    let input_str = core::str::from_utf8(input).unwrap_or("");
    let decoded = html_escape::decode_html_entities(input_str);
    let decoded_bytes = decoded.as_bytes();

    url_escape_link_destination_raw(out, decoded_bytes);
}

#[inline]
fn push_text_escape(out: &mut Vec<u8>, b: u8) {
    match b {
        b'<' => out.extend_from_slice(b"&lt;"),
        b'>' => out.extend_from_slice(b"&gt;"),
        b'&' => out.extend_from_slice(b"&amp;"),
        b'"' => out.extend_from_slice(b"&quot;"),
        _ => out.push(b),
    }
}

#[inline]
fn push_attr_escape(out: &mut Vec<u8>, b: u8) {
    match b {
        b'<' => out.extend_from_slice(b"&lt;"),
        b'>' => out.extend_from_slice(b"&gt;"),
        b'&' => out.extend_from_slice(b"&amp;"),
        b'"' => out.extend_from_slice(b"&quot;"),
        b'\'' => out.extend_from_slice(b"&#39;"),
        _ => out.push(b),
    }
}

/// Process a link URL without entity decoding (used after entities are already decoded).
#[inline]
fn url_escape_link_destination_raw(out: &mut Vec<u8>, input: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    if input.is_ascii()
        && memchr2(b'\\', b' ', input).is_none()
        && memchr3(b'"', b'<', b'>', input).is_none()
        && memchr2(b'&', b'\'', input).is_none()
        && !input
            .iter()
            .any(|&b| matches!(b, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F))
    {
        out.extend_from_slice(input);
        return;
    }

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
        assert_eq!(
            out,
            b"&lt;a href=&quot;test&quot;&gt;link &amp; stuff&lt;/a&gt;"
        );
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
    fn test_escape_unicode() {
        let mut out = Vec::new();
        escape_text_into(&mut out, "Hallo Welt! <tag>".as_bytes());
        assert_eq!(out, b"Hallo Welt! &lt;tag&gt;");
    }

    /// Build an input of `len` filler bytes with `payload` spliced in at `at`.
    fn padded(len: usize, at: usize, payload: &[u8]) -> Vec<u8> {
        let mut v = vec![b'x'; len];
        v[at..at + payload.len()].copy_from_slice(payload);
        v
    }

    /// The short-scan and memchr scanners must agree for every escape
    /// character at every position across chunk and length thresholds.
    #[test]
    fn test_scanner_threshold_boundary() {
        for len in [
            1,
            2,
            15,
            16,
            17,
            31,
            32,
            33,
            SHORT_SCAN_MAX - 1,
            SHORT_SCAN_MAX,
            SHORT_SCAN_MAX + 1,
        ] {
            for &c in b"<>&\"" {
                for at in [0, len / 2, len - 1] {
                    let input = padded(len, at, &[c]);
                    assert_eq!(
                        first_text_escape(&input),
                        Some(at),
                        "text: len={len} at={at} c={}",
                        c as char
                    );
                }
            }
            for &c in b"<>&\"'" {
                let input = padded(len, len - 1, &[c]);
                assert_eq!(first_attr_escape(&input), Some(len - 1));
            }
            assert_eq!(first_text_escape(&vec![b'x'; len]), None);
            assert_eq!(first_attr_escape(&vec![b'x'; len]), None);
        }
    }

    /// Exhaustive position sweep around the SIMD chunk boundary, including
    /// two escape bytes where the earlier one must win.
    #[test]
    fn test_scanner_every_position() {
        for len in 1..=40usize {
            for at in 0..len {
                let input = padded(len, at, b"&");
                assert_eq!(first_text_escape(&input), Some(at), "len={len} at={at}");
                assert_eq!(first_attr_escape(&input), Some(at), "len={len} at={at}");
                if at + 1 < len {
                    let mut two = input.clone();
                    two[len - 1] = b'<';
                    assert_eq!(first_text_escape(&two), Some(at), "two: len={len} at={at}");
                }
            }
        }
    }

    /// Long-input path: a quote before the first `<>&` hit must win, and a
    /// quote after it must not mask it (the bounded second pass).
    #[test]
    fn test_long_input_quote_ordering() {
        let long = 4 * SHORT_SCAN_MAX;

        let quote_first = padded(long, 10, b"\"");
        let quote_first = {
            let mut v = quote_first;
            v[long - 10] = b'<';
            v
        };
        assert_eq!(first_text_escape(&quote_first), Some(10));

        let angle_first = padded(long, 10, b"<");
        let angle_first = {
            let mut v = angle_first;
            v[long - 10] = b'"';
            v
        };
        assert_eq!(first_text_escape(&angle_first), Some(10));

        let quote_only = padded(long, long - 1, b"\"");
        assert_eq!(first_text_escape(&quote_only), Some(long - 1));

        let single_quote_late = padded(long, long - 1, b"'");
        assert_eq!(first_attr_escape(&single_quote_late), Some(long - 1));
        // '\'' is not escaped in text context
        assert_eq!(first_text_escape(&single_quote_late), None);
    }

    /// End-to-end escaping of a long segment goes through the memchr path.
    #[test]
    fn test_escape_long_input_end_to_end() {
        let long = 3 * SHORT_SCAN_MAX;
        let mut input = vec![b'a'; long];
        input[SHORT_SCAN_MAX + 5] = b'"';
        input[2 * SHORT_SCAN_MAX] = b'&';
        let mut out = Vec::new();
        escape_text_into(&mut out, &input);

        let mut expected = Vec::new();
        for (i, &b) in input.iter().enumerate() {
            match i {
                _ if b == b'"' => expected.extend_from_slice(b"&quot;"),
                _ if b == b'&' => expected.extend_from_slice(b"&amp;"),
                _ => expected.push(b),
            }
        }
        assert_eq!(out, expected);

        let mut attr_out = Vec::new();
        input[10] = b'\'';
        escape_full_into(&mut attr_out, &input);
        assert!(attr_out.windows(5).any(|w| w == b"&#39;"));
        assert!(attr_out.windows(6).any(|w| w == b"&quot;"));
        assert!(attr_out.windows(5).any(|w| w == b"&amp;"));
    }
}
