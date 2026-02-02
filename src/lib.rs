//! md-fast: Ultra-high-performance Markdown to HTML compiler
//!
//! This crate provides a streaming Markdown parser optimized for speed,
//! targeting 20-30% better throughput than existing Rust parsers.
//!
//! # Design Principles
//! - No AST: streaming events only
//! - No regex: pure byte-level scanning
//! - No backtracking: O(n) time on all inputs
//! - Minimal allocations: ranges into input buffer

pub mod block;
pub mod cursor;
pub mod escape;
pub mod inline;
pub mod limits;
pub mod range;
pub mod render;

// Re-export primary types
pub use block::{BlockEvent, BlockParser};
pub use inline::{InlineEvent, InlineParser};
pub use range::Range;
pub use render::HtmlWriter;

/// Convert Markdown to HTML.
///
/// This is the primary API for simple use cases.
///
/// # Example
/// ```
/// let html = md_fast::to_html("# Hello\n\nWorld");
/// assert!(html.contains("<h1>Hello</h1>"));
/// assert!(html.contains("<p>World</p>"));
/// ```
pub fn to_html(input: &str) -> String {
    let mut writer = HtmlWriter::with_capacity_for(input.len());
    render_to_writer(input.as_bytes(), &mut writer);
    writer.into_string()
}

/// Convert Markdown to HTML, writing into a provided buffer.
///
/// This avoids allocation if the buffer has sufficient capacity.
pub fn to_html_into(input: &str, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(input.len() + input.len() / 4);
    let mut writer = HtmlWriter::with_capacity(0);
    // Use the provided buffer directly
    std::mem::swap(writer.buffer_mut(), out);
    render_to_writer(input.as_bytes(), &mut writer);
    std::mem::swap(writer.buffer_mut(), out);
}

/// Render Markdown to an HtmlWriter.
fn render_to_writer(input: &[u8], writer: &mut HtmlWriter) {
    // Parse blocks
    let mut parser = BlockParser::new(input);
    let mut events = Vec::new();
    parser.parse(&mut events);

    // Create inline parser for text content
    let mut inline_parser = InlineParser::new();
    let mut inline_events = Vec::new();

    // Render events to HTML
    for event in events {
        render_block_event(input, &event, writer, &mut inline_parser, &mut inline_events);
    }
}

/// Render a single block event to HTML.
fn render_block_event(
    input: &[u8],
    event: &BlockEvent,
    writer: &mut HtmlWriter,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
) {
    match event {
        BlockEvent::ParagraphStart => {
            writer.paragraph_start();
        }
        BlockEvent::ParagraphEnd => {
            writer.paragraph_end();
        }
        BlockEvent::HeadingStart { level } => {
            writer.heading_start(*level);
        }
        BlockEvent::HeadingEnd { level } => {
            writer.heading_end(*level);
        }
        BlockEvent::ThematicBreak => {
            writer.thematic_break();
        }
        BlockEvent::SoftBreak => {
            writer.write_str("\n");
        }
        BlockEvent::Text(range) => {
            // Process text through inline parser
            let text = range.slice(input);
            inline_events.clear();
            inline_parser.parse(text, inline_events);

            // Render inline events
            for inline_event in inline_events.iter() {
                render_inline_event(text, inline_event, writer);
            }
        }
        BlockEvent::Code(range) => {
            // Code block content - no inline parsing
            writer.write_escaped_text(range.slice(input));
        }
        BlockEvent::CodeBlockStart { info } => {
            let lang = info.as_ref().map(|r| r.slice(input));
            writer.code_block_start(lang);
        }
        BlockEvent::CodeBlockEnd => {
            writer.code_block_end();
        }
        BlockEvent::BlockQuoteStart => {
            writer.blockquote_start();
        }
        BlockEvent::BlockQuoteEnd => {
            writer.blockquote_end();
        }
        BlockEvent::ListStart { kind } => {
            match kind {
                block::ListKind::Unordered => writer.ul_start(),
                block::ListKind::Ordered { start } => {
                    writer.ol_start(if *start == 1 { None } else { Some(*start) })
                }
            }
        }
        BlockEvent::ListEnd { kind } => {
            match kind {
                block::ListKind::Unordered => writer.ul_end(),
                block::ListKind::Ordered { .. } => writer.ol_end(),
            }
        }
        BlockEvent::ListItemStart { .. } => {
            writer.li_start();
        }
        BlockEvent::ListItemEnd => {
            writer.li_end();
        }
    }
}

/// Render a single inline event to HTML.
fn render_inline_event(text: &[u8], event: &InlineEvent, writer: &mut HtmlWriter) {
    match event {
        InlineEvent::Text(range) => {
            writer.write_escaped_text(range.slice(text));
        }
        InlineEvent::Code(range) => {
            writer.write_str("<code>");
            writer.write_escaped_text(range.slice(text));
            writer.write_str("</code>");
        }
        InlineEvent::EmphasisStart => {
            writer.write_str("<em>");
        }
        InlineEvent::EmphasisEnd => {
            writer.write_str("</em>");
        }
        InlineEvent::StrongStart => {
            writer.write_str("<strong>");
        }
        InlineEvent::StrongEnd => {
            writer.write_str("</strong>");
        }
        InlineEvent::SoftBreak => {
            writer.write_str("\n");
        }
        InlineEvent::HardBreak => {
            writer.write_str("<br />\n");
        }
        InlineEvent::EscapedChar(ch) => {
            // Write the escaped character (the actual char, not the backslash)
            let bytes = [*ch];
            writer.write_escaped_text(&bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_paragraph() {
        let html = to_html("Hello, world!");
        assert_eq!(html, "<p>Hello, world!</p>\n");
    }

    #[test]
    fn test_paragraph_escaping() {
        let html = to_html("<script>alert('xss')</script>");
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_heading_h1() {
        let html = to_html("# Hello");
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_heading_h2() {
        let html = to_html("## World");
        assert!(html.contains("<h2>World</h2>"));
    }

    #[test]
    fn test_heading_all_levels() {
        for level in 1..=6 {
            let input = format!("{} Heading", "#".repeat(level));
            let html = to_html(&input);
            assert!(
                html.contains(&format!("<h{level}>Heading</h{level}>")),
                "Failed for level {level}: {html}"
            );
        }
    }

    #[test]
    fn test_thematic_break() {
        let html = to_html("---");
        assert_eq!(html, "<hr />\n");
    }

    #[test]
    fn test_thematic_break_variants() {
        assert_eq!(to_html("---"), "<hr />\n");
        assert_eq!(to_html("***"), "<hr />\n");
        assert_eq!(to_html("___"), "<hr />\n");
        assert_eq!(to_html("- - -"), "<hr />\n");
        assert_eq!(to_html("----------"), "<hr />\n");
    }

    #[test]
    fn test_multiple_paragraphs() {
        let html = to_html("First\n\nSecond");
        assert!(html.contains("<p>First</p>"));
        assert!(html.contains("<p>Second</p>"));
    }

    #[test]
    fn test_heading_and_paragraph() {
        let html = to_html("# Title\n\nContent here.");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<p>Content here.</p>"));
    }

    #[test]
    fn test_heading_with_closing_hashes() {
        let html = to_html("# Hello #");
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_complex_document() {
        let input = r#"# Main Title

This is the first paragraph.

## Section 1

More content here.

---

## Section 2

Final paragraph."#;

        let html = to_html(input);

        assert!(html.contains("<h1>Main Title</h1>"));
        assert!(html.contains("<h2>Section 1</h2>"));
        assert!(html.contains("<h2>Section 2</h2>"));
        assert!(html.contains("<hr />"));
        assert!(html.contains("<p>This is the first paragraph.</p>"));
    }

    #[test]
    fn test_multiline_paragraph() {
        let html = to_html("Line 1\nLine 2\nLine 3");
        // All lines should be in the same paragraph
        assert!(html.starts_with("<p>"));
        assert!(html.contains("Line 1"));
        assert!(html.contains("Line 2"));
        assert!(html.contains("Line 3"));
        assert!(html.ends_with("</p>\n"));
    }

    #[test]
    fn test_empty_input() {
        let html = to_html("");
        assert_eq!(html, "");
    }

    #[test]
    fn test_only_whitespace() {
        let html = to_html("   \n\n   ");
        assert_eq!(html, "");
    }

    #[test]
    fn test_to_html_into() {
        let mut buffer = Vec::new();
        to_html_into("# Test", &mut buffer);
        let html = String::from_utf8(buffer).unwrap();
        assert!(html.contains("<h1>Test</h1>"));
    }

    // Code block tests

    #[test]
    fn test_code_block_basic() {
        let html = to_html("```\ncode\n```");
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("code"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_code_block_with_language() {
        let html = to_html("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn test_code_block_escapes_html() {
        let html = to_html("```\n<script>alert('xss')</script>\n```");
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_code_block_multiline() {
        let html = to_html("```\nline1\nline2\n```");
        assert!(html.contains("line1"));
        assert!(html.contains("line2"));
    }

    #[test]
    fn test_code_block_in_document() {
        let input = r#"# Title

Some text.

```python
print("hello")
```

More text."#;
        let html = to_html(input);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<p>Some text.</p>"));
        assert!(html.contains("<pre><code class=\"language-python\">"));
        assert!(html.contains("print"));
        assert!(html.contains("<p>More text.</p>"));
    }
}
