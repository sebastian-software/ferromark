//! ferromark: Ultra-high-performance Markdown to HTML compiler
//!
//! This crate provides a streaming Markdown parser optimized for speed,
//! targeting 20-30% better throughput than existing Rust parsers.
//!
//! # Design Principles
//! - No AST: streaming events only
//! - No regex: pure byte-level scanning
//! - No backtracking: O(n) time on all inputs
//! - Minimal allocations: ranges into input buffer
//!
//! # Future Optimizations
//! - `simdutf` / `simdutf8`: SIMD-accelerated UTF-8 validation for input
//! - NEON intrinsics for ARM: inline marker scanning
//! - Loop unrolling in hot paths (4x unroll like md4c)

pub mod block;
pub mod cursor;
pub mod escape;
pub mod inline;
pub mod link_ref;
pub mod limits;
pub mod range;
pub mod render;

// Re-export primary types
pub use block::{fixup_list_tight, BlockEvent, BlockParser};
pub use inline::{InlineEvent, InlineParser};
pub use link_ref::{LinkRefDef, LinkRefStore};
pub use range::Range;
pub use render::HtmlWriter;

/// Parsing/rendering options.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Allow raw inline and block HTML.
    pub allow_html: bool,
    /// Resolve link reference definitions and reference-style links.
    pub allow_link_refs: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            allow_html: true,
            allow_link_refs: true,
        }
    }
}

/// Convert Markdown to HTML.
///
/// This is the primary API for simple use cases.
///
/// # Example
/// ```
/// let html = ferromark::to_html("# Hello\n\nWorld");
/// assert!(html.contains("<h1>Hello</h1>"));
/// assert!(html.contains("<p>World</p>"));
/// ```
pub fn to_html(input: &str) -> String {
    let mut writer = HtmlWriter::with_capacity_for(input.len());
    render_to_writer(input.as_bytes(), &mut writer, &Options::default());
    writer.into_string()
}

/// Convert Markdown to HTML, writing into a provided buffer.
///
/// This avoids allocation if the buffer has sufficient capacity.
pub fn to_html_into(input: &str, out: &mut Vec<u8>) {
    to_html_into_with_options(input, out, &Options::default());
}

/// Convert Markdown to HTML with options.
pub fn to_html_with_options(input: &str, options: &Options) -> String {
    let mut writer = HtmlWriter::with_capacity_for(input.len());
    render_to_writer(input.as_bytes(), &mut writer, options);
    writer.into_string()
}

/// Convert Markdown to HTML into a provided buffer with options.
pub fn to_html_into_with_options(input: &str, out: &mut Vec<u8>, options: &Options) {
    out.clear();
    out.reserve(input.len() + input.len() / 4);
    let mut writer = HtmlWriter::with_capacity(0);
    // Use the provided buffer directly
    std::mem::swap(writer.buffer_mut(), out);
    render_to_writer(input.as_bytes(), &mut writer, options);
    std::mem::swap(writer.buffer_mut(), out);
}

/// State for collecting paragraph content before inline parsing.
struct ParagraphState {
    /// Collected text content (joined with newlines).
    content: Vec<u8>,
    /// Whether we're currently in a paragraph.
    in_paragraph: bool,
}

impl ParagraphState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(256),
            in_paragraph: false,
        }
    }

    fn start(&mut self) {
        self.in_paragraph = true;
        self.content.clear();
    }

    fn add_text(&mut self, text: &[u8]) {
        self.content.extend_from_slice(text);
    }

    fn add_soft_break(&mut self) {
        self.content.push(b'\n');
    }

    fn finish(&mut self) -> &[u8] {
        self.in_paragraph = false;
        // CommonMark: strip trailing spaces/tabs from paragraph content
        while self.content.last().map_or(false, |&b| b == b' ' || b == b'\t') {
            self.content.pop();
        }
        &self.content
    }
}

/// State for collecting heading content before inline parsing.
struct HeadingState {
    /// Collected text content (joined with newlines).
    content: Vec<u8>,
    /// Whether we're currently in a heading.
    in_heading: bool,
}

impl HeadingState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(64),
            in_heading: false,
        }
    }

    fn start(&mut self) {
        self.in_heading = true;
        self.content.clear();
    }

    fn add_text(&mut self, text: &[u8]) {
        self.content.extend_from_slice(text);
    }

    fn add_soft_break(&mut self) {
        self.content.push(b'\n');
    }

    fn finish(&mut self) -> &[u8] {
        self.in_heading = false;
        while self.content.last().map_or(false, |&b| b == b' ' || b == b'\t') {
            self.content.pop();
        }
        &self.content
    }
}

/// Render Markdown to an HtmlWriter.
fn render_to_writer(input: &[u8], writer: &mut HtmlWriter, options: &Options) {
    // Parse blocks
    let mut parser = BlockParser::new_with_options(input, *options);
    let mut events = Vec::with_capacity((input.len() / 16).max(64));
    parser.parse(&mut events);
    let link_refs = parser.take_link_refs();

    // Fix up list tight status (ListStart gets its tight value from ListEnd)
    fixup_list_tight(&mut events);

    // Create inline parser for text content
    let mut inline_parser = InlineParser::new();
    let mut inline_events = Vec::with_capacity(64);

    // State for accumulating paragraph content
    let mut para_state = ParagraphState::new();
    let mut heading_state = HeadingState::new();

    // Track tight/loose status for nested lists (stack - (tight, blockquote_depth_at_start))
    let mut tight_list_stack: Vec<(bool, u32)> = Vec::new();

    // Track if we just started a tight list item (need newline before block content)
    let mut at_tight_li_start = false;

    // Track if we need newline before next block element (after paragraph in tight list)
    let mut need_newline_before_block = false;

    // Track if we need a newline after <li> in loose list (deferred until content appears)
    let mut pending_loose_li_newline = false;

    // Track blockquote depth (paragraphs in blockquotes always get <p> tags)
    let mut blockquote_depth = 0u32;

    // Render events to HTML
    for event in &events {
        render_block_event(
            input,
            event,
            writer,
            &mut inline_parser,
            &mut inline_events,
            &mut para_state,
            &mut heading_state,
            &mut tight_list_stack,
            &mut at_tight_li_start,
            &mut need_newline_before_block,
            &mut pending_loose_li_newline,
            &mut blockquote_depth,
            &link_refs,
            options,
        );
    }
}

/// Render a single block event to HTML.
fn render_block_event(
    input: &[u8],
    event: &BlockEvent,
    writer: &mut HtmlWriter,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
    para_state: &mut ParagraphState,
    heading_state: &mut HeadingState,
    tight_list_stack: &mut Vec<(bool, u32)>,
    at_tight_li_start: &mut bool,
    need_newline_before_block: &mut bool,
    pending_loose_li_newline: &mut bool,
    blockquote_depth: &mut u32,
    link_refs: &LinkRefStore,
    options: &Options,
) {
    // Check if we're in a tight list (innermost list is tight)
    // BUT: paragraphs inside blockquotes that started AFTER the list need <p> tags
    let in_tight_list = tight_list_stack.last().map_or(false, |(tight, bq_depth_at_start)| {
        *tight && *blockquote_depth <= *bq_depth_at_start
    });

    match event {
        BlockEvent::ParagraphStart => {
            // Write pending newline from loose list item start
            if *pending_loose_li_newline {
                writer.newline();
                *pending_loose_li_newline = false;
            }
            // In tight lists, don't emit <p> tags
            if !in_tight_list {
                writer.paragraph_start();
            }
            para_state.start();
            // Paragraph content is inline, so we don't add newline
            *at_tight_li_start = false;
        }
        BlockEvent::ParagraphEnd => {
            // Check if we're in a tight list (innermost list is tight)
            // BUT: paragraphs inside blockquotes that started AFTER the list need </p> tags
            let in_tight_list = tight_list_stack.last().map_or(false, |(tight, bq_depth_at_start)| {
                *tight && *blockquote_depth <= *bq_depth_at_start
            });

            // Parse all accumulated paragraph content at once
            let content = para_state.finish();
            if !content.is_empty() {
                inline_events.clear();
                inline_events.reserve((content.len() / 8).max(8));
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse(content, refs, options.allow_html, inline_events);

                // Render inline events
                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(content, inline_event, writer, &mut image_state, link_refs);
                }
            }
            // In tight lists, don't emit </p> tags
            if !in_tight_list {
                writer.paragraph_end();
            } else {
                // Mark that we need newline before next block element
                *need_newline_before_block = true;
            }
        }
        BlockEvent::HeadingStart { level } => {
            if *need_newline_before_block {
                writer.newline();
                *need_newline_before_block = false;
            }
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
            writer.heading_start(*level);
            heading_state.start();
        }
        BlockEvent::HeadingEnd { level } => {
            let content = heading_state.finish();
            if !content.is_empty() {
                inline_events.clear();
                inline_events.reserve((content.len() / 8).max(8));
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse(content, refs, options.allow_html, inline_events);

                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(content, inline_event, writer, &mut image_state, link_refs);
                }
            }
            writer.heading_end(*level);
        }
        BlockEvent::ThematicBreak => {
            // If we're at the start of a tight list item, add newline before block content
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
            writer.thematic_break();
        }
        BlockEvent::HtmlBlockStart => {
            // Write pending newline from loose list item start
            if *pending_loose_li_newline {
                writer.newline();
                *pending_loose_li_newline = false;
            }
            // If we're at the start of a tight list item, add newline before block content
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
        }
        BlockEvent::HtmlBlockText(range) => {
            writer.write_bytes(range.slice(input));
        }
        BlockEvent::HtmlBlockEnd => {}
        BlockEvent::SoftBreak => {
            if para_state.in_paragraph {
                para_state.add_soft_break();
            } else if heading_state.in_heading {
                heading_state.add_soft_break();
            } else {
                writer.write_str("\n");
            }
        }
        BlockEvent::Text(range) => {
            let text = range.slice(input);
            if para_state.in_paragraph {
                // Accumulate for later parsing
                para_state.add_text(text);
            } else if heading_state.in_heading {
                heading_state.add_text(text);
            } else {
                // Parse immediately (e.g., heading content)
                inline_events.clear();
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse(text, refs, options.allow_html, inline_events);

                // Render inline events
                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(text, inline_event, writer, &mut image_state, link_refs);
                }
            }
        }
        BlockEvent::Code(range) => {
            // Code block content - no inline parsing
            writer.write_escaped_text(range.slice(input));
        }
        BlockEvent::VirtualSpaces(count) => {
            // Emit spaces for tab expansion in indented code blocks
            for _ in 0..*count {
                writer.write_byte(b' ');
            }
        }
        BlockEvent::CodeBlockStart { info } => {
            // Write pending newline from loose list item start
            if *pending_loose_li_newline {
                writer.newline();
                *pending_loose_li_newline = false;
            }
            // If we're at the start of a tight list item, add newline before block content
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
            let lang = info.as_ref().map(|r| r.slice(input));
            writer.code_block_start(lang);
        }
        BlockEvent::CodeBlockEnd => {
            writer.code_block_end();
        }
        BlockEvent::BlockQuoteStart => {
            // Write pending newline from loose list item start
            if *pending_loose_li_newline {
                writer.newline();
                *pending_loose_li_newline = false;
            }
            // If we need newline (after paragraph in tight list), add it
            if *need_newline_before_block {
                writer.newline();
                *need_newline_before_block = false;
            }
            // If we're at the start of a tight list item, add newline before block content
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
            *blockquote_depth += 1;
            writer.blockquote_start();
        }
        BlockEvent::BlockQuoteEnd => {
            *blockquote_depth = blockquote_depth.saturating_sub(1);
            writer.blockquote_end();
        }
        BlockEvent::ListStart { kind, tight } => {
            // Write pending newline from loose list item start
            if *pending_loose_li_newline {
                writer.newline();
                *pending_loose_li_newline = false;
            }
            // If we need newline (after paragraph in tight list), add it
            if *need_newline_before_block {
                writer.newline();
                *need_newline_before_block = false;
            }
            // If we're at the start of a tight list item, add newline before nested list
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
            // Push the tight status and current blockquote depth for this list
            tight_list_stack.push((*tight, *blockquote_depth));
            match kind {
                block::ListKind::Unordered => writer.ul_start(),
                block::ListKind::Ordered { start, .. } => {
                    writer.ol_start(if *start == 1 { None } else { Some(*start) })
                }
            }
        }
        BlockEvent::ListEnd { kind, .. } => {
            match kind {
                block::ListKind::Unordered => writer.ul_end(),
                block::ListKind::Ordered { .. } => writer.ol_end(),
            }
            // Pop the tight status for this list
            tight_list_stack.pop();
        }
        BlockEvent::ListItemStart { .. } => {
            writer.li_start();
            // In loose lists, defer newline until content appears (for empty items)
            if !in_tight_list {
                *pending_loose_li_newline = true;
            } else {
                // In tight lists, mark that we may need newline if block content follows
                *at_tight_li_start = true;
            }
        }
        BlockEvent::ListItemEnd => {
            *at_tight_li_start = false;
            *need_newline_before_block = false;
            *pending_loose_li_newline = false;
            writer.li_end();
        }
    }
}

/// State for tracking image rendering.
/// Since we need to render: <img src="..." alt="ALT_TEXT_HERE" title="..." />
/// But alt text comes as Text events between ImageStart and ImageEnd,
/// we need to track:
/// 1. The title to render at ImageEnd
/// 2. The nesting depth (to handle nested images like ![foo ![bar](url1)](url2))
struct ImageState {
    title_range: Option<Range>,
    title_bytes: Option<Vec<u8>>,
    /// Nesting depth: 1 = in outermost image, 2+ = in nested image
    depth: u32,
}

/// Render a single inline event to HTML.
fn render_inline_event(
    text: &[u8],
    event: &InlineEvent,
    writer: &mut HtmlWriter,
    image_state: &mut Option<ImageState>,
    link_refs: &LinkRefStore,
) {
    // Check if we're inside an image (for alt text rendering)
    let in_image = image_state.as_ref().map_or(false, |s| s.depth > 0);

    match event {
        InlineEvent::Text(range) => {
            // In image alt text, we still write the text (escaped for attributes)
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                // Decode HTML entities and escape for output
                writer.write_text_with_entities(range.slice(text));
            }
        }
        InlineEvent::Code(range) => {
            // In image alt text, just write the code content as plain text
            if in_image {
                let code_content = range.slice(text);
                for &b in code_content {
                    if b == b'\n' {
                        writer.write_str(" ");
                    } else if b == b'<' {
                        writer.write_str("&lt;");
                    } else if b == b'>' {
                        writer.write_str("&gt;");
                    } else if b == b'&' {
                        writer.write_str("&amp;");
                    } else if b == b'"' {
                        writer.write_str("&quot;");
                    } else {
                        writer.buffer_mut().push(b);
                    }
                }
            } else {
                writer.write_str("<code>");
                // CommonMark: line endings in code spans are converted to spaces
                let code_content = range.slice(text);
                for &b in code_content {
                    if b == b'\n' {
                        writer.write_str(" ");
                    } else if b == b'<' {
                        writer.write_str("&lt;");
                    } else if b == b'>' {
                        writer.write_str("&gt;");
                    } else if b == b'&' {
                        writer.write_str("&amp;");
                    } else if b == b'"' {
                        writer.write_str("&quot;");
                    } else {
                        writer.buffer_mut().push(b);
                    }
                }
                writer.write_str("</code>");
            }
        }
        InlineEvent::EmphasisStart => {
            // Suppress HTML tags inside image alt text
            if !in_image {
                writer.write_str("<em>");
            }
        }
        InlineEvent::EmphasisEnd => {
            if !in_image {
                writer.write_str("</em>");
            }
        }
        InlineEvent::StrongStart => {
            if !in_image {
                writer.write_str("<strong>");
            }
        }
        InlineEvent::StrongEnd => {
            if !in_image {
                writer.write_str("</strong>");
            }
        }
        InlineEvent::LinkStart { url, title } => {
            // Suppress link tags inside image alt text
            if !in_image {
                writer.write_str("<a href=\"");
                writer.write_link_url(url.slice(text));
                writer.write_str("\"");
                if let Some(t) = title {
                    writer.write_str(" title=\"");
                    writer.write_link_title(t.slice(text));
                    writer.write_str("\"");
                }
                writer.write_str(">");
            }
        }
        InlineEvent::LinkStartRef { def_index } => {
            if !in_image {
                if let Some(def) = link_refs.get(*def_index as usize) {
                    writer.write_str("<a href=\"");
                    writer.write_link_url(&def.url);
                    writer.write_str("\"");
                    if let Some(title) = &def.title {
                        writer.write_str(" title=\"");
                        writer.write_link_title(title);
                        writer.write_str("\"");
                    }
                    writer.write_str(">");
                }
            }
        }
        InlineEvent::LinkEnd => {
            if !in_image {
                writer.write_str("</a>");
            }
        }
        InlineEvent::ImageStart { url, title } => {
            // If we're already inside an image, just increment depth
            // (the inner image's alt text becomes plain text in outer alt)
            if let Some(state) = image_state.as_mut() {
                state.depth += 1;
            } else {
                // Outermost image - emit the img tag start
                writer.write_str("<img src=\"");
                writer.write_link_url(url.slice(text));
                writer.write_str("\" alt=\"");
                *image_state = Some(ImageState {
                    title_range: title.clone(),
                    title_bytes: None,
                    depth: 1,
                });
            }
        }
        InlineEvent::ImageStartRef { def_index } => {
            if let Some(state) = image_state.as_mut() {
                state.depth += 1;
            } else if let Some(def) = link_refs.get(*def_index as usize) {
                writer.write_str("<img src=\"");
                writer.write_link_url(&def.url);
                writer.write_str("\" alt=\"");
                *image_state = Some(ImageState {
                    title_range: None,
                    title_bytes: def.title.clone(),
                    depth: 1,
                });
            }
        }
        InlineEvent::ImageEnd => {
            if let Some(state) = image_state.as_mut() {
                state.depth -= 1;
                // Only close when we exit the outermost image
                if state.depth == 0 {
                    writer.write_str("\"");
                    // Add title attribute if present
                    let title_range = state.title_range.clone();
                    let title_bytes = state.title_bytes.clone();
                    *image_state = None;
                    if let Some(bytes) = title_bytes {
                        writer.write_str(" title=\"");
                        writer.write_link_title(&bytes);
                        writer.write_str("\"");
                    } else if let Some(title_range) = title_range {
                        writer.write_str(" title=\"");
                        writer.write_link_title(title_range.slice(text));
                        writer.write_str("\"");
                    }
                    writer.write_str(" />");
                }
            }
        }
        InlineEvent::Autolink { url, is_email } => {
            // In image alt text, just output the URL as plain text
            if in_image {
                writer.write_escaped_attr(url.slice(text));
            } else {
                writer.write_str("<a href=\"");
                if *is_email {
                    writer.write_str("mailto:");
                }
                // URL-encode special chars then HTML-escape
                writer.write_url_encoded(url.slice(text));
                writer.write_str("\">");
                // Display text is shown as-is (with HTML escaping)
                writer.write_escaped_text(url.slice(text));
                writer.write_str("</a>");
            }
        }
        InlineEvent::Html(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                writer.write_bytes(range.slice(text));
            }
        }
        InlineEvent::SoftBreak => {
            // In image alt text, use space instead of newline
            if in_image {
                writer.write_str(" ");
            } else {
                writer.write_str("\n");
            }
        }
        InlineEvent::HardBreak => {
            // In image alt text, use space instead of <br />
            if in_image {
                writer.write_str(" ");
            } else {
                writer.write_str("<br />\n");
            }
        }
        InlineEvent::EscapedChar(ch) => {
            // Write the escaped character (the actual char, not the backslash)
            let bytes = [*ch];
            if in_image {
                writer.write_escaped_attr(&bytes);
            } else {
                writer.write_escaped_text(&bytes);
            }
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
        // CommonMark preserves raw HTML when HTML is enabled (default).
        assert_eq!(html, "<script>alert('xss')</script>");
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

    // Tight/loose list tests

    #[test]
    fn test_tight_list_unordered() {
        let html = to_html("- foo\n- bar\n- baz");
        // Tight list: no <p> tags inside list items
        assert!(html.contains("<li>foo</li>"));
        assert!(html.contains("<li>bar</li>"));
        assert!(html.contains("<li>baz</li>"));
        assert!(!html.contains("<li><p>"));
    }

    #[test]
    fn test_loose_list_unordered() {
        let html = to_html("- foo\n\n- bar\n\n- baz");
        // Loose list: <p> tags inside list items (with newline after <li>)
        assert!(html.contains("<li>\n<p>foo</p>"));
        assert!(html.contains("<li>\n<p>bar</p>"));
        assert!(html.contains("<li>\n<p>baz</p>"));
    }

    #[test]
    fn test_tight_list_ordered() {
        let html = to_html("1. first\n2. second\n3. third");
        // Tight list: no <p> tags
        assert!(html.contains("<li>first</li>"));
        assert!(html.contains("<li>second</li>"));
        assert!(html.contains("<li>third</li>"));
        assert!(!html.contains("<li><p>"));
    }

    #[test]
    fn test_loose_list_ordered() {
        let html = to_html("1. first\n\n2. second");
        // Loose list: <p> tags (with newline after <li>)
        assert!(html.contains("<li>\n<p>first</p>"));
        assert!(html.contains("<li>\n<p>second</p>"));
    }

    // Image tests

    #[test]
    fn test_image_basic() {
        let html = to_html("![alt](image.png)");
        // Should have img tag with src and alt
        assert!(html.contains("<img src=\"image.png\""), "Missing img src");
        assert!(html.contains("alt=\"alt\""), "Missing alt attribute");
        // Should NOT have standalone ! before the img tag
        assert!(!html.contains("!<img"), "Stray ! before img tag");
    }

    #[test]
    fn test_image_with_title() {
        let html = to_html("![alt](image.png \"title\")");
        // Should have img tag with title
        assert!(html.contains("<img"), "No img tag found");
        assert!(html.contains("title=\"title\""), "Missing title attribute");
        assert!(!html.contains("!<img"), "Stray ! before img tag");
    }

    #[test]
    fn test_image_in_text() {
        let html = to_html("text before ![img](url) text after");
        // Image should be between text
        assert!(html.contains("text before"));
        assert!(html.contains("<img src=\"url\""));
        assert!(html.contains("text after"));
    }

    #[test]
    fn test_image_with_nested_emphasis() {
        // CommonMark: alt text should be plain text, not HTML
        let html = to_html("![foo *bar*](/url)");
        // Should have alt="foo bar" (plain text, no <em> tags)
        assert!(html.contains("alt=\"foo bar\""), "Alt text should be plain: {html}");
        assert!(!html.contains("<em>"), "No <em> tags in alt text");
    }

    #[test]
    fn test_image_with_nested_strong() {
        let html = to_html("![foo **bar**](/url)");
        // Should have alt="foo bar" (plain text, no <strong> tags)
        assert!(html.contains("alt=\"foo bar\""), "Alt text should be plain: {html}");
        assert!(!html.contains("<strong>"), "No <strong> tags in alt text");
    }
}

#[cfg(test)]
mod entity_tests {
    #[test]
    fn test_html_escape_entities() {
        use html_escape::decode_html_entities;
        
        assert_eq!(decode_html_entities("&auml;").as_ref(), "ä");
        assert_eq!(decode_html_entities("&#228;").as_ref(), "ä");
        assert_eq!(decode_html_entities("&#xE4;").as_ref(), "ä");
        assert_eq!(decode_html_entities("&amp;").as_ref(), "&");
        assert_eq!(decode_html_entities("foo%20b&auml;").as_ref(), "foo%20bä");
    }
}
