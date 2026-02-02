//! HTML escaping utilities.
//!
//! Fast-path optimized: scans for first escapable character,
//! then bulk-copies segments between escapes.


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
    let mut pos = 0;

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
    let mut pos = 0;

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
