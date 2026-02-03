//! HTML output writer with optimized buffer management.
//!
//! Uses md4c's growth strategy: 1.5x + 128-byte alignment.

use crate::escape;
use crate::Range;

/// HTML output writer with pre-allocated, reusable buffer.
///
/// # Example
/// ```
/// use md_fast::HtmlWriter;
///
/// let mut writer = HtmlWriter::with_capacity_for(1000);
/// writer.write_str("<p>");
/// writer.write_escaped_text(b"Hello <World>");
/// writer.write_str("</p>");
///
/// let html = writer.into_string();
/// assert_eq!(html, "<p>Hello &lt;World&gt;</p>");
/// ```
pub struct HtmlWriter {
    out: Vec<u8>,
}

impl HtmlWriter {
    /// Create a new writer with default capacity.
    #[inline]
    pub fn new() -> Self {
        Self {
            out: Vec::with_capacity(1024),
        }
    }

    /// Create with pre-allocated capacity based on expected input size.
    ///
    /// Typical HTML is ~1.25x input size; we reserve extra for safety.
    #[inline]
    pub fn with_capacity_for(input_len: usize) -> Self {
        let capacity = input_len + input_len / 4;
        Self {
            out: Vec::with_capacity(capacity),
        }
    }

    /// Create with explicit capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            out: Vec::with_capacity(capacity),
        }
    }

    /// Grow buffer using md4c's strategy: 1.5x + 128-byte alignment.
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn grow(&mut self, needed: usize) {
        let new_cap = ((self.out.len() + needed) * 3 / 2 + 128) & !127;
        self.out.reserve(new_cap.saturating_sub(self.out.capacity()));
    }

    /// Ensure capacity for additional bytes.
    #[inline]
    #[allow(dead_code)]
    fn ensure_capacity(&mut self, additional: usize) {
        if self.out.len() + additional > self.out.capacity() {
            self.grow(additional);
        }
    }

    /// Write raw bytes without escaping.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// Write a static string (compile-time known).
    #[inline]
    pub fn write_str(&mut self, s: &'static str) {
        self.out.extend_from_slice(s.as_bytes());
    }

    /// Write a dynamic string without escaping.
    #[inline]
    pub fn write_string(&mut self, s: &str) {
        self.out.extend_from_slice(s.as_bytes());
    }

    /// Write a single byte.
    #[inline]
    pub fn write_byte(&mut self, b: u8) {
        self.out.push(b);
    }

    /// Write text with HTML escaping (for text content).
    #[inline]
    pub fn write_escaped_text(&mut self, text: &[u8]) {
        escape::escape_text_into(&mut self.out, text);
    }

    /// Write text with HTML escaping from a range.
    #[inline]
    pub fn write_escaped_range(&mut self, input: &[u8], range: Range) {
        escape::escape_text_into(&mut self.out, range.slice(input));
    }

    /// Write attribute value with full escaping (including quotes).
    #[inline]
    pub fn write_escaped_attr(&mut self, attr: &[u8]) {
        escape::escape_full_into(&mut self.out, attr);
    }

    /// Write URL/title with backslash escape processing and HTML attribute escaping.
    /// Backslash-escaped punctuation characters have the backslash removed.
    #[inline]
    pub fn write_escaped_link_attr(&mut self, attr: &[u8]) {
        let mut pos = 0;
        while pos < attr.len() {
            if attr[pos] == b'\\' && pos + 1 < attr.len() && is_link_escapable(attr[pos + 1]) {
                // Skip backslash, write escaped char (with HTML escaping)
                pos += 1;
                escape::escape_full_into(&mut self.out, &attr[pos..pos + 1]);
                pos += 1;
            } else {
                // Find next backslash or end
                let start = pos;
                while pos < attr.len() && !(attr[pos] == b'\\' && pos + 1 < attr.len() && is_link_escapable(attr[pos + 1])) {
                    pos += 1;
                }
                escape::escape_full_into(&mut self.out, &attr[start..pos]);
            }
        }
    }

    /// Write link title with entity decoding, backslash escape processing, and HTML escaping.
    #[inline]
    pub fn write_link_title(&mut self, title: &[u8]) {
        // First decode entities
        let title_str = core::str::from_utf8(title).unwrap_or("");
        let decoded = html_escape::decode_html_entities(title_str);
        let decoded_bytes = decoded.as_bytes();

        // Then process backslash escapes and HTML-escape
        let mut pos = 0;
        while pos < decoded_bytes.len() {
            if decoded_bytes[pos] == b'\\' && pos + 1 < decoded_bytes.len() && is_link_escapable(decoded_bytes[pos + 1]) {
                // Skip backslash, write escaped char (with HTML escaping)
                pos += 1;
                escape::escape_full_into(&mut self.out, &decoded_bytes[pos..pos + 1]);
                pos += 1;
            } else {
                // Find next backslash or end
                let start = pos;
                while pos < decoded_bytes.len() && !(decoded_bytes[pos] == b'\\' && pos + 1 < decoded_bytes.len() && is_link_escapable(decoded_bytes[pos + 1])) {
                    pos += 1;
                }
                escape::escape_full_into(&mut self.out, &decoded_bytes[start..pos]);
            }
        }
    }

    /// Write autolink URL with percent-encoding and HTML escaping.
    /// Used for autolink hrefs per CommonMark spec.
    #[inline]
    pub fn write_url_encoded(&mut self, url: &[u8]) {
        escape::url_encode_then_html_escape(&mut self.out, url);
    }

    /// Write link destination with backslash escape processing and URL encoding.
    /// Used for link destinations in `[text](url)` syntax.
    #[inline]
    pub fn write_link_url(&mut self, url: &[u8]) {
        escape::url_escape_link_destination(&mut self.out, url);
    }

    /// Write a newline.
    #[inline]
    pub fn newline(&mut self) {
        self.out.push(b'\n');
    }

    /// Current output length.
    #[inline]
    pub fn len(&self) -> usize {
        self.out.len()
    }

    /// Check if output is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.out.is_empty()
    }

    /// Clear output for reuse (keeps capacity).
    #[inline]
    pub fn clear(&mut self) {
        self.out.clear();
    }

    /// Get output as byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.out
    }

    /// Get output as str (assumes valid UTF-8).
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: We only write valid UTF-8 (ASCII tags + escaped content)
        unsafe { std::str::from_utf8_unchecked(&self.out) }
    }

    /// Take ownership of output buffer.
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.out
    }

    /// Take ownership as String.
    #[inline]
    pub fn into_string(self) -> String {
        // SAFETY: We only write valid UTF-8
        unsafe { String::from_utf8_unchecked(self.out) }
    }

    /// Get mutable reference to internal buffer.
    ///
    /// Use with caution - allows bypassing escaping.
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.out
    }

    // --- HTML Tag Helpers ---

    /// Write opening tag: `<tagname>`
    #[inline]
    pub fn open_tag(&mut self, tag: &'static str) {
        self.write_byte(b'<');
        self.write_str(tag);
        self.write_byte(b'>');
    }

    /// Write closing tag: `</tagname>`
    #[inline]
    pub fn close_tag(&mut self, tag: &'static str) {
        self.write_str("</");
        self.write_str(tag);
        self.write_byte(b'>');
    }

    /// Write self-closing tag: `<tagname />`
    #[inline]
    pub fn self_closing_tag(&mut self, tag: &'static str) {
        self.write_byte(b'<');
        self.write_str(tag);
        self.write_str(" />");
    }

    /// Write opening tag with newline: `<tagname>\n`
    #[inline]
    pub fn open_tag_nl(&mut self, tag: &'static str) {
        self.open_tag(tag);
        self.newline();
    }

    /// Write closing tag with newline: `</tagname>\n`
    #[inline]
    pub fn close_tag_nl(&mut self, tag: &'static str) {
        self.close_tag(tag);
        self.newline();
    }

    // --- Common HTML Elements ---

    /// Write paragraph start: `<p>`
    #[inline]
    pub fn paragraph_start(&mut self) {
        self.write_str("<p>");
    }

    /// Write paragraph end: `</p>\n`
    #[inline]
    pub fn paragraph_end(&mut self) {
        self.write_str("</p>\n");
    }

    /// Write heading start: `<hN>`
    #[inline]
    pub fn heading_start(&mut self, level: u8) {
        debug_assert!(level >= 1 && level <= 6);
        self.write_str("<h");
        self.write_byte(b'0' + level);
        self.write_byte(b'>');
    }

    /// Write heading end: `</hN>\n`
    #[inline]
    pub fn heading_end(&mut self, level: u8) {
        debug_assert!(level >= 1 && level <= 6);
        self.write_str("</h");
        self.write_byte(b'0' + level);
        self.write_str(">\n");
    }

    /// Write code block start with optional language class.
    /// Processes backslash escapes in the language string.
    #[inline]
    pub fn code_block_start(&mut self, lang: Option<&[u8]>) {
        match lang {
            Some(l) if !l.is_empty() => {
                self.write_str("<pre><code class=\"language-");
                // Process backslash escapes in info string
                self.write_escaped_link_attr(l);
                self.write_str("\">");
            }
            _ => {
                self.write_str("<pre><code>");
            }
        }
    }

    /// Write code block end: `</code></pre>\n`
    #[inline]
    pub fn code_block_end(&mut self) {
        self.write_str("</code></pre>\n");
    }

    /// Write thematic break: `<hr />\n`
    #[inline]
    pub fn thematic_break(&mut self) {
        self.write_str("<hr />\n");
    }

    /// Write blockquote start: `<blockquote>\n`
    #[inline]
    pub fn blockquote_start(&mut self) {
        self.write_str("<blockquote>\n");
    }

    /// Write blockquote end: `</blockquote>\n`
    #[inline]
    pub fn blockquote_end(&mut self) {
        self.write_str("</blockquote>\n");
    }

    /// Write list start (unordered): `<ul>\n`
    #[inline]
    pub fn ul_start(&mut self) {
        self.write_str("<ul>\n");
    }

    /// Write list end (unordered): `</ul>\n`
    #[inline]
    pub fn ul_end(&mut self) {
        self.write_str("</ul>\n");
    }

    /// Write list start (ordered): `<ol>\n` or `<ol start="N">\n`
    #[inline]
    pub fn ol_start(&mut self, start: Option<u32>) {
        match start {
            Some(n) if n != 1 => {
                self.write_str("<ol start=\"");
                self.write_u32(n);
                self.write_str("\">\n");
            }
            _ => {
                self.write_str("<ol>\n");
            }
        }
    }

    /// Write list end (ordered): `</ol>\n`
    #[inline]
    pub fn ol_end(&mut self) {
        self.write_str("</ol>\n");
    }

    /// Write list item start: `<li>`
    #[inline]
    pub fn li_start(&mut self) {
        self.write_str("<li>");
    }

    /// Write list item end: `</li>\n`
    #[inline]
    pub fn li_end(&mut self) {
        self.write_str("</li>\n");
    }

    /// Write inline code: `<code>escaped_content</code>`
    #[inline]
    pub fn inline_code(&mut self, content: &[u8]) {
        self.write_str("<code>");
        self.write_escaped_text(content);
        self.write_str("</code>");
    }

    /// Write emphasis start: `<em>`
    #[inline]
    pub fn em_start(&mut self) {
        self.write_str("<em>");
    }

    /// Write emphasis end: `</em>`
    #[inline]
    pub fn em_end(&mut self) {
        self.write_str("</em>");
    }

    /// Write strong start: `<strong>`
    #[inline]
    pub fn strong_start(&mut self) {
        self.write_str("<strong>");
    }

    /// Write strong end: `</strong>`
    #[inline]
    pub fn strong_end(&mut self) {
        self.write_str("</strong>");
    }

    /// Write strikethrough start: `<del>`
    #[inline]
    pub fn del_start(&mut self) {
        self.write_str("<del>");
    }

    /// Write strikethrough end: `</del>`
    #[inline]
    pub fn del_end(&mut self) {
        self.write_str("</del>");
    }

    /// Write link start: `<a href="url">`
    #[inline]
    pub fn link_start(&mut self, url: &[u8], title: Option<&[u8]>) {
        self.write_str("<a href=\"");
        self.write_escaped_attr(url);
        if let Some(t) = title {
            self.write_str("\" title=\"");
            self.write_escaped_attr(t);
        }
        self.write_str("\">");
    }

    /// Write link end: `</a>`
    #[inline]
    pub fn link_end(&mut self) {
        self.write_str("</a>");
    }

    /// Write line break: `<br />\n`
    #[inline]
    pub fn line_break(&mut self) {
        self.write_str("<br />\n");
    }

    /// Write a u32 as decimal.
    fn write_u32(&mut self, mut n: u32) {
        if n == 0 {
            self.write_byte(b'0');
            return;
        }

        let mut buf = [0u8; 10]; // Max digits for u32
        let mut i = buf.len();

        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }

        self.write_bytes(&buf[i..]);
    }
}

/// Characters that can be escaped with backslash in CommonMark links.
#[inline]
fn is_link_escapable(b: u8) -> bool {
    matches!(b,
        b'!' | b'"' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'(' | b')' |
        b'*' | b'+' | b',' | b'-' | b'.' | b'/' | b':' | b';' | b'<' |
        b'=' | b'>' | b'?' | b'@' | b'[' | b'\\' | b']' | b'^' | b'_' |
        b'`' | b'{' | b'|' | b'}' | b'~'
    )
}

impl Default for HtmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Write for HtmlWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.out.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_new() {
        let writer = HtmlWriter::new();
        assert!(writer.is_empty());
    }

    #[test]
    fn test_writer_capacity() {
        let writer = HtmlWriter::with_capacity_for(1000);
        assert!(writer.out.capacity() >= 1250);
    }

    #[test]
    fn test_writer_write_str() {
        let mut writer = HtmlWriter::new();
        writer.write_str("<p>");
        assert_eq!(writer.as_str(), "<p>");
    }

    #[test]
    fn test_writer_escaped_text() {
        let mut writer = HtmlWriter::new();
        writer.write_escaped_text(b"<script>");
        assert_eq!(writer.as_str(), "&lt;script&gt;");
    }

    #[test]
    fn test_writer_paragraph() {
        let mut writer = HtmlWriter::new();
        writer.paragraph_start();
        writer.write_escaped_text(b"Hello");
        writer.paragraph_end();
        assert_eq!(writer.as_str(), "<p>Hello</p>\n");
    }

    #[test]
    fn test_writer_heading() {
        let mut writer = HtmlWriter::new();
        writer.heading_start(1);
        writer.write_escaped_text(b"Title");
        writer.heading_end(1);
        assert_eq!(writer.as_str(), "<h1>Title</h1>\n");
    }

    #[test]
    fn test_writer_heading_levels() {
        for level in 1..=6 {
            let mut writer = HtmlWriter::new();
            writer.heading_start(level);
            writer.heading_end(level);
            let expected = format!("<h{level}></h{level}>\n");
            assert_eq!(writer.as_str(), expected);
        }
    }

    #[test]
    fn test_writer_code_block() {
        let mut writer = HtmlWriter::new();
        writer.code_block_start(Some(b"rust"));
        writer.write_escaped_text(b"fn main() {}");
        writer.code_block_end();
        assert_eq!(
            writer.as_str(),
            "<pre><code class=\"language-rust\">fn main() {}</code></pre>\n"
        );
    }

    #[test]
    fn test_writer_code_block_no_lang() {
        let mut writer = HtmlWriter::new();
        writer.code_block_start(None);
        writer.write_escaped_text(b"code");
        writer.code_block_end();
        assert_eq!(writer.as_str(), "<pre><code>code</code></pre>\n");
    }

    #[test]
    fn test_writer_thematic_break() {
        let mut writer = HtmlWriter::new();
        writer.thematic_break();
        assert_eq!(writer.as_str(), "<hr />\n");
    }

    #[test]
    fn test_writer_link() {
        let mut writer = HtmlWriter::new();
        writer.link_start(b"https://example.com", None);
        writer.write_escaped_text(b"link");
        writer.link_end();
        assert_eq!(writer.as_str(), "<a href=\"https://example.com\">link</a>");
    }

    #[test]
    fn test_writer_link_with_title() {
        let mut writer = HtmlWriter::new();
        writer.link_start(b"https://example.com", Some(b"My Title"));
        writer.write_escaped_text(b"link");
        writer.link_end();
        assert_eq!(
            writer.as_str(),
            "<a href=\"https://example.com\" title=\"My Title\">link</a>"
        );
    }

    #[test]
    fn test_writer_link_escape_url() {
        let mut writer = HtmlWriter::new();
        writer.link_start(b"https://example.com?a=1&b=2", None);
        writer.link_end();
        assert_eq!(
            writer.as_str(),
            "<a href=\"https://example.com?a=1&amp;b=2\"></a>"
        );
    }

    #[test]
    fn test_writer_clear_reuse() {
        let mut writer = HtmlWriter::new();
        writer.write_str("first");
        let cap1 = writer.out.capacity();

        writer.clear();
        assert!(writer.is_empty());
        assert_eq!(writer.out.capacity(), cap1);

        writer.write_str("second");
        assert_eq!(writer.as_str(), "second");
    }

    #[test]
    fn test_writer_into_string() {
        let mut writer = HtmlWriter::new();
        writer.write_str("<p>Hello</p>");
        let s = writer.into_string();
        assert_eq!(s, "<p>Hello</p>");
    }

    #[test]
    fn test_writer_ol_with_start() {
        let mut writer = HtmlWriter::new();
        writer.ol_start(Some(5));
        writer.ol_end();
        assert_eq!(writer.as_str(), "<ol start=\"5\">\n</ol>\n");
    }

    #[test]
    fn test_writer_ol_default_start() {
        let mut writer = HtmlWriter::new();
        writer.ol_start(Some(1));
        writer.ol_end();
        assert_eq!(writer.as_str(), "<ol>\n</ol>\n");
    }

    #[test]
    fn test_write_u32() {
        let mut writer = HtmlWriter::new();
        writer.write_u32(0);
        assert_eq!(writer.as_str(), "0");

        writer.clear();
        writer.write_u32(42);
        assert_eq!(writer.as_str(), "42");

        writer.clear();
        writer.write_u32(1234567890);
        assert_eq!(writer.as_str(), "1234567890");
    }
}
