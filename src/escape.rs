//! HTML escaping utilities.
//!
//! Fast-path optimized: scans for first escapable character,
//! then bulk-copies segments between escapes.

use memchr::{memchr, memchr2, memchr3};

/// Escape HTML text content into output buffer.
///
/// Escapes `<`, `>`, `&`, and `"` to their HTML entity equivalents.
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

/// Keep the shared short scan inline. On NEON, longer writes continue in an
/// outlined backend that bounds the work needed to locate the next escape.
const SHORT_SCAN_MAX: usize = 128;

#[inline]
pub(crate) fn first_text_escape(input: &[u8]) -> Option<usize> {
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        first_escape::<false>(input)
    }
    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
    {
        if input.len() <= SHORT_SCAN_MAX {
            return first_escape_short::<false>(input);
        }
        let a = memchr3(b'<', b'>', b'&', input);
        // A '"' is only relevant if it appears before the first <>& hit, so the
        // second pass never scans past it.
        let limit = a.unwrap_or(input.len());
        memchr(b'"', &input[..limit]).or(a)
    }
}

#[inline]
fn first_attr_escape(input: &[u8]) -> Option<usize> {
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        first_escape::<true>(input)
    }
    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
    {
        if input.len() <= SHORT_SCAN_MAX {
            return first_escape_short::<true>(input);
        }
        let a = memchr3(b'<', b'>', b'&', input);
        let limit = a.unwrap_or(input.len());
        memchr2(b'"', b'\'', &input[..limit]).or(a)
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
fn first_escape<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    let prefix_len = input.len().min(SHORT_SCAN_MAX);
    if let Some(at) = first_escape_in_set::<ATTR>(&input[..prefix_len]) {
        return Some(at);
    }
    if input.len() > prefix_len {
        first_escape_long::<ATTR>(&input[prefix_len..]).map(|at| prefix_len + at)
    } else {
        None
    }
}

// Keep the long-search loop out of short writes and inline entity probes.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(never)]
fn first_escape_long<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    if input.len() >= SHORT_SCAN_MAX * 2 {
        // The shared NEON scanner stops at any escape in a single pass. Keep
        // short tails on the bounded memchr window below.
        let found = first_escape_in_set::<ATTR>(input);
        #[cfg(test)]
        tests::record_long_search(input.len(), found);
        return found;
    }
    let mut start = 0;
    let mut width = SHORT_SCAN_MAX * 2;
    while start < input.len() {
        let len = width.min(input.len() - start);
        let chunk = &input[start..start + len];
        let common = memchr3(b'<', b'>', b'&', chunk);
        #[cfg(test)]
        tests::record_long_search(chunk.len(), common);
        let limit = common.unwrap_or(chunk.len());
        let quote = if ATTR {
            memchr2(b'"', b'\'', &chunk[..limit])
        } else {
            memchr(b'"', &chunk[..limit])
        };
        if let Some(at) = quote.or(common) {
            return Some(start + at);
        }
        start += len;
        // Both searches stay within a window. Doubling bounds how far either
        // can scan past the next escape, while amortizing setup on plain text.
        width = width.saturating_mul(2);
    }
    None
}

// Escape policy stays here; the shared scanner knows only byte sets.
const TEXT_SPECIALS: crate::byte_search::ByteSet<4> = crate::byte_search::ByteSet::new(b"<>&\"");
const ATTR_SPECIALS: crate::byte_search::ByteSet<5> = crate::byte_search::ByteSet::new(b"<>&\"'");

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
fn first_escape_in_set<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    if ATTR {
        ATTR_SPECIALS.find(input)
    } else {
        TEXT_SPECIALS.find(input)
    }
}

#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline]
fn first_escape_short<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    if ATTR {
        ATTR_SPECIALS.find(input)
    } else {
        TEXT_SPECIALS.find(input)
    }
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

    // One eligibility scan instead of separate ASCII, punctuation and control
    // scans. The fallback below retains all escaping and backslash semantics.
    if input.iter().all(|&byte| {
        !matches!(
            byte,
            b'\\' | b' ' | b'"' | b'<' | b'>' | b'&' | b'\''
                | 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F..=0xFF
        )
    }) {
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

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    std::thread_local! {
        static LONG_SEARCH_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    // Count the logical prefix inspected by each long-search window. This is
    // compiled out of release builds and avoids timing thresholds in tests.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    pub(super) fn record_long_search(len: usize, found: Option<usize>) {
        let inspected = found.map_or(len, |at| at + 1);
        LONG_SEARCH_BYTES.with(|count| count.set(count.get().saturating_add(inspected)));
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    #[test]
    fn quoted_code_search_work_grows_linearly() {
        for padding in [0, 255] {
            let line = format!("{}let s = \"text\";\n", "x".repeat(padding));
            let mut counts = Vec::new();
            for repeats in [128, 256] {
                let body = line.repeat(repeats);
                let input = format!("```\n{body}```\n");
                LONG_SEARCH_BYTES.with(|count| count.set(0));
                let actual = crate::to_html(&input);
                counts.push(LONG_SEARCH_BYTES.with(std::cell::Cell::get));
                assert_eq!(
                    actual,
                    format!("<pre><code>{}</code></pre>\n", body.replace('"', "&quot;"))
                );
            }
            if padding != 0 {
                assert!(
                    counts[0] > 0,
                    "sparse quotes must exercise the counted search"
                );
            }
            assert!(
                counts[1] <= 2 * counts[0] + 16_384,
                "padding={padding}, search work: {counts:?}"
            );
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    #[test]
    fn quoted_attribute_search_work_grows_linearly() {
        for padding in [0, 255] {
            let item = format!("{}value='x' title=\"y\" ", "x".repeat(padding));
            let mut counts = Vec::new();
            for repeats in [128, 256] {
                let input = item.repeat(repeats);
                let mut actual = Vec::new();
                LONG_SEARCH_BYTES.with(|count| count.set(0));
                escape_attr_into(&mut actual, input.as_bytes());
                counts.push(LONG_SEARCH_BYTES.with(std::cell::Cell::get));
                let expected = input.replace('\'', "&#39;").replace('"', "&quot;");
                assert_eq!(actual, expected.as_bytes());
            }
            if padding != 0 {
                assert!(
                    counts[0] > 0,
                    "sparse quotes must exercise the counted search"
                );
            }
            assert!(
                counts[1] <= 2 * counts[0] + 16_384,
                "padding={padding}, search work: {counts:?}"
            );
        }
    }

    #[test]
    fn escape_searches_match_scalar_oracle_across_boundaries() {
        for len in [
            127, 128, 129, 383, 384, 385, 895, 896, 897, 1919, 1920, 1921, 3967, 3968, 3969, 8063,
            8064, 8065,
        ] {
            for at in [0, 15, 16, len / 2, len - 1] {
                for byte in 0u8..=255 {
                    let mut input = vec![b'x'; len];
                    input[at] = byte;
                    for later_ampersand in [false, true] {
                        if later_ampersand {
                            input[len - 1] = b'&';
                        }
                        let text = input
                            .iter()
                            .position(|&b| matches!(b, b'<' | b'>' | b'&' | b'"'));
                        let attr = input
                            .iter()
                            .position(|&b| matches!(b, b'<' | b'>' | b'&' | b'"' | b'\''));
                        assert_eq!(
                            first_text_escape(&input),
                            text,
                            "len={len}, at={at}, byte={byte}"
                        );
                        assert_eq!(
                            first_attr_escape(&input),
                            attr,
                            "len={len}, at={at}, byte={byte}"
                        );
                    }
                }
            }
        }
    }

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

    /// Long-input searches must return the earliest escape regardless of kind.
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

    /// End-to-end escaping preserves every segment around long-input matches.
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

    #[test]
    fn link_destination_encoding_preserves_every_byte_at_scan_boundaries() {
        for padding in [0, 1, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129] {
            for byte in 0u8..=255 {
                let mut input = vec![b'x'; padding];
                input.push(byte);
                input.extend_from_slice(b"tail");
                let encoded = match byte {
                    b'\\' => b"%5C".to_vec(),
                    b' ' => b"%20".to_vec(),
                    b'"' => b"%22".to_vec(),
                    b'<' => b"&lt;".to_vec(),
                    b'>' => b"&gt;".to_vec(),
                    b'&' => b"&amp;".to_vec(),
                    b'\'' => b"&#39;".to_vec(),
                    0..=8 | 11 | 12 | 14..=31 | 127..=255 => format!("%{byte:02X}").into_bytes(),
                    _ => vec![byte],
                };
                let mut expected = vec![b'x'; padding];
                expected.extend_from_slice(&encoded);
                expected.extend_from_slice(b"tail");
                let mut output = Vec::new();
                url_escape_link_destination_raw(&mut output, &input);
                assert_eq!(output, expected, "padding={padding}, byte={byte}");
            }
        }
    }

    #[test]
    fn link_destination_fast_path_preserves_backslashes_and_entities() {
        for (input, expected) in [
            (r"a\&b", "a&amp;b"),
            (r"a\'b", "a&#39;b"),
            (r#"a\"b"#, "a%22b"),
            (r"a\ b", "a%5C%20b"),
            (r"a\?b", "a?b"),
            (r"a\\b", r"a\b"),
            ("a&amp;b", "a&amp;b"),
            ("a&#32;b", "a%20b"),
            ("a&#x22;b", "a%22b"),
            ("https://example.org/ü", "https://example.org/%C3%BC"),
        ] {
            let mut output = Vec::new();
            url_escape_link_destination(&mut output, input.as_bytes());
            assert_eq!(output, expected.as_bytes(), "{input}");
        }
    }
}
