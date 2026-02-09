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
pub mod footnote;
pub mod inline;
pub mod link_ref;
pub mod limits;
pub mod range;
pub mod render;

// Re-export primary types
pub use block::{fixup_list_tight, Alignment, BlockEvent, BlockParser, CalloutType};
pub use footnote::FootnoteStore;
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
    /// Enable GFM table extension.
    pub tables: bool,
    /// Enable GFM strikethrough extension (`~~text~~`).
    pub strikethrough: bool,
    /// Enable GFM task list extension (`[ ]` / `[x]`).
    pub task_lists: bool,
    /// Enable GFM autolink literals extension (bare URLs, www, emails).
    pub autolink_literals: bool,
    /// Enable GFM disallowed raw HTML extension (filter dangerous tags).
    pub disallowed_raw_html: bool,
    /// Enable footnotes extension (`[^label]` references and `[^label]:` definitions).
    pub footnotes: bool,
    /// Enable front matter detection (`---`/`+++` delimited metadata at document start).
    pub front_matter: bool,
    /// Generate GitHub-compatible heading IDs (`<h1 id="slug">`).
    pub heading_ids: bool,
    /// Enable math spans (`$inline$` and `$$display$$`).
    pub math: bool,
    /// Enable GitHub-style callouts/admonitions (`> [!NOTE]`, `> [!WARNING]`, etc.).
    pub callouts: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            allow_html: true,
            allow_link_refs: true,
            tables: true,
            strikethrough: true,
            task_lists: true,
            autolink_literals: false,
            disallowed_raw_html: true,
            footnotes: false,
            front_matter: false,
            heading_ids: true,
            math: false,
            callouts: true,
        }
    }
}

/// Result of parsing Markdown with front matter extraction.
pub struct ParseResult<'a> {
    /// Rendered HTML output.
    pub html: String,
    /// Raw front matter content (between delimiters), if detected.
    pub front_matter: Option<&'a str>,
}

/// Extract front matter from the start of a document.
///
/// Returns `Some((content, rest_offset))` where `content` is the raw text between
/// delimiters and `rest_offset` is the byte offset where the remaining markdown begins.
/// Returns `None` if no valid front matter is found.
fn extract_front_matter(input: &str) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    if bytes.len() < 3 {
        return None;
    }

    // Determine delimiter character: must be exactly 3 of `-` or `+` at byte 0
    let delim_char = match bytes[0] {
        b'-' | b'+' => bytes[0],
        _ => return None,
    };

    // Verify exactly 3 delimiter chars (not 4+)
    if bytes.len() < 3 || bytes[1] != delim_char || bytes[2] != delim_char {
        return None;
    }

    // After the 3 delimiter chars, only whitespace allowed before newline
    let mut pos = 3;
    if pos < bytes.len() && bytes[pos] == delim_char {
        // 4+ delimiter chars — not front matter
        return None;
    }
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }

    // Must hit newline (or end of input for degenerate case, but that means no closing)
    if pos >= bytes.len() {
        return None;
    }
    if bytes[pos] == b'\r' {
        pos += 1;
    }
    if pos >= bytes.len() || bytes[pos] != b'\n' {
        return None;
    }
    pos += 1;

    let content_start = pos;

    // Search for closing delimiter
    loop {
        if pos >= bytes.len() {
            // No closing delimiter found
            return None;
        }

        // Check if current line is a closing delimiter
        let line_start = pos;
        if pos + 2 < bytes.len()
            && bytes[pos] == delim_char
            && bytes[pos + 1] == delim_char
            && bytes[pos + 2] == delim_char
        {
            let mut p = pos + 3;
            // Must not have 4+ delimiter chars
            if p < bytes.len() && bytes[p] == delim_char {
                // Not a closing delimiter, skip this line
            } else {
                // Optional trailing whitespace
                while p < bytes.len() && (bytes[p] == b' ' || bytes[p] == b'\t') {
                    p += 1;
                }
                // Must be at newline or EOF
                let at_end = if p >= bytes.len() {
                    true
                } else if bytes[p] == b'\n' {
                    true
                } else if bytes[p] == b'\r' && p + 1 < bytes.len() && bytes[p + 1] == b'\n' {
                    true
                } else {
                    false
                };

                if at_end {
                    let content = &input[content_start..line_start];
                    // Advance past the closing delimiter line
                    let mut rest = p;
                    if rest < bytes.len() {
                        if bytes[rest] == b'\r' {
                            rest += 1;
                        }
                        if rest < bytes.len() && bytes[rest] == b'\n' {
                            rest += 1;
                        }
                    }
                    return Some((content, rest));
                }
            }
        }

        // Skip to next line
        while pos < bytes.len() && bytes[pos] != b'\n' {
            pos += 1;
        }
        if pos < bytes.len() {
            pos += 1; // skip \n
        }

        // Safety: if we haven't advanced past line_start, force progress
        if pos <= line_start {
            break;
        }
    }

    None
}

/// Parse Markdown and return both HTML and front matter (if present).
///
/// Uses default options with `front_matter: true`.
///
/// # Example
/// ```
/// let result = ferromark::parse("---\ntitle: Hello\n---\n# Content");
/// assert_eq!(result.front_matter, Some("title: Hello\n"));
/// assert!(result.html.contains("Content</h1>"));
/// ```
pub fn parse(input: &str) -> ParseResult<'_> {
    let mut options = Options::default();
    options.front_matter = true;
    parse_with_options(input, &options)
}

/// Parse Markdown with options and return both HTML and front matter.
///
/// Front matter is only extracted when `options.front_matter` is `true`.
pub fn parse_with_options<'a>(input: &'a str, options: &Options) -> ParseResult<'a> {
    let (front_matter, markdown) = if options.front_matter {
        match extract_front_matter(input) {
            Some((fm, offset)) => (Some(fm), &input[offset..]),
            None => (None, input),
        }
    } else {
        (None, input)
    };

    let html = to_html_with_options(markdown, options);
    ParseResult { html, front_matter }
}

/// Convert Markdown to HTML.
///
/// This is the primary API for simple use cases.
///
/// # Example
/// ```
/// let html = ferromark::to_html("# Hello\n\nWorld");
/// assert!(html.contains("Hello</h1>"));
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
///
/// When `options.front_matter` is `true`, any front matter at the start of the
/// document is silently stripped before parsing.
pub fn to_html_with_options(input: &str, options: &Options) -> String {
    let markdown = if options.front_matter {
        match extract_front_matter(input) {
            Some((_, offset)) => &input[offset..],
            None => input,
        }
    } else {
        input
    };
    let mut writer = HtmlWriter::with_capacity_for(markdown.len());
    render_to_writer(markdown.as_bytes(), &mut writer, options);
    writer.into_string()
}

/// Convert Markdown to HTML into a provided buffer with options.
///
/// When `options.front_matter` is `true`, any front matter at the start of the
/// document is silently stripped before parsing.
pub fn to_html_into_with_options(input: &str, out: &mut Vec<u8>, options: &Options) {
    let markdown = if options.front_matter {
        match extract_front_matter(input) {
            Some((_, offset)) => &input[offset..],
            None => input,
        }
    } else {
        input
    };
    out.clear();
    out.reserve(markdown.len() + markdown.len() / 4);
    let mut writer = HtmlWriter::with_capacity(0);
    // Use the provided buffer directly
    std::mem::swap(writer.buffer_mut(), out);
    render_to_writer(markdown.as_bytes(), &mut writer, options);
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
    /// Current heading level (stored for deferred tag emission).
    level: u8,
}

impl HeadingState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(64),
            in_heading: false,
            level: 0,
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

/// Tracker for deduplicating heading IDs.
struct HeadingIdTracker {
    used: Vec<String>,
}

impl HeadingIdTracker {
    fn new() -> Self {
        Self { used: Vec::new() }
    }

    /// Generate a unique slug, appending `-1`, `-2`, etc. on collision.
    fn unique_slug(&mut self, base: String) -> String {
        let slug = if base.is_empty() { "heading".to_string() } else { base };
        let count = self.used.iter().filter(|s| **s == slug).count();
        let result = if count == 0 {
            slug.clone()
        } else {
            format!("{}-{}", slug, count)
        };
        self.used.push(slug);
        result
    }
}

/// Generate a GitHub-compatible slug from raw heading text.
///
/// Steps:
/// 1. Strip inline markup delimiters (`*`, `_`, `~`, `` ` ``, `[`, `]`, `!`, `#`)
/// 2. Lowercase
/// 3. Replace whitespace runs with `-`
/// 4. Remove chars that are not alphanumeric, `-`, `_`, or space
/// 5. Strip leading/trailing `-`
fn generate_slug(raw: &[u8]) -> String {
    let mut slug = Vec::with_capacity(raw.len());
    let mut prev_was_space = false;

    for &b in raw {
        // Strip inline markup delimiters (keep _ since it's valid in slugs)
        if matches!(b, b'*' | b'~' | b'`' | b'[' | b']' | b'!' | b'#') {
            continue;
        }

        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            if !prev_was_space && !slug.is_empty() {
                slug.push(b'-');
                prev_was_space = true;
            }
            continue;
        }

        prev_was_space = false;

        // Lowercase ASCII
        let ch = if b.is_ascii_uppercase() { b + 32 } else { b };

        // Keep alphanumeric, hyphen, underscore, and multibyte UTF-8
        if ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' || ch >= 0x80 {
            slug.push(ch);
        }
    }

    // Strip trailing hyphen
    while slug.last() == Some(&b'-') {
        slug.pop();
    }
    // Strip leading hyphen
    while slug.first() == Some(&b'-') {
        slug.remove(0);
    }

    String::from_utf8(slug).unwrap_or_default()
}

/// State for collecting table cell content before inline parsing.
struct CellState {
    /// Collected text content.
    content: Vec<u8>,
    /// Whether we're currently in a cell.
    in_cell: bool,
}

impl CellState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(64),
            in_cell: false,
        }
    }

    fn start(&mut self) {
        self.in_cell = true;
        self.content.clear();
    }

    fn add_text(&mut self, text: &[u8]) {
        // In table cells, \| is a table-level escape meaning literal |
        // Replace \| with | before inline parsing
        let mut i = 0;
        while i < text.len() {
            if text[i] == b'\\' && i + 1 < text.len() && text[i + 1] == b'|' {
                self.content.push(b'|');
                i += 2;
            } else {
                self.content.push(text[i]);
                i += 1;
            }
        }
    }

    fn finish(&mut self) -> &[u8] {
        self.in_cell = false;
        // Trim trailing whitespace
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
    let footnote_store = if options.footnotes {
        Some(parser.take_footnote_store())
    } else {
        None
    };

    // Fix up list tight status (ListStart gets its tight value from ListEnd)
    fixup_list_tight(&mut events);

    // Create inline parser for text content
    let mut inline_parser = InlineParser::new();
    let mut inline_events = Vec::with_capacity(64);

    // State for accumulating paragraph content
    let mut para_state = ParagraphState::new();
    let mut heading_state = HeadingState::new();
    let mut cell_state = CellState::new();

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

    // Track whether we're in a table header
    let mut in_table_head = false;

    // Pending task checkbox (emitted at start of first paragraph in list item)
    let mut pending_task = block::TaskState::None;

    // Track footnote reference order (def_index -> sequential number)
    let mut footnote_order: Vec<usize> = Vec::new();
    let fn_store_ref = footnote_store.as_ref();

    // Heading ID tracker for deduplication
    let mut heading_id_tracker = HeadingIdTracker::new();

    // Track callout type for each open blockquote (None = regular blockquote)
    let mut callout_stack: Vec<Option<block::CalloutType>> = Vec::new();

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
            &mut cell_state,
            &mut tight_list_stack,
            &mut at_tight_li_start,
            &mut need_newline_before_block,
            &mut pending_loose_li_newline,
            &mut blockquote_depth,
            &mut in_table_head,
            &mut pending_task,
            &link_refs,
            fn_store_ref,
            &mut footnote_order,
            &mut heading_id_tracker,
            &mut callout_stack,
            options,
        );
    }

    // Render footnote section at document end
    if let Some(fn_store) = &footnote_store {
        if !footnote_order.is_empty() {
            render_footnote_section(
                input,
                fn_store,
                &footnote_order,
                writer,
                &mut inline_parser,
                &mut inline_events,
                &link_refs,
                options,
            );
        }
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
    cell_state: &mut CellState,
    tight_list_stack: &mut Vec<(bool, u32)>,
    at_tight_li_start: &mut bool,
    need_newline_before_block: &mut bool,
    pending_loose_li_newline: &mut bool,
    blockquote_depth: &mut u32,
    in_table_head: &mut bool,
    pending_task: &mut block::TaskState,
    link_refs: &LinkRefStore,
    footnote_store: Option<&FootnoteStore>,
    footnote_order: &mut Vec<usize>,
    heading_id_tracker: &mut HeadingIdTracker,
    callout_stack: &mut Vec<Option<block::CalloutType>>,
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

            // Emit pending task checkbox before paragraph content
            emit_pending_task_checkbox(pending_task, writer);

            if !content.is_empty() {
                inline_events.clear();
                inline_events.reserve((content.len() / 8).max(8));
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse_with_options(content, refs, options.allow_html, options.strikethrough, options.autolink_literals, options.math, footnote_store, inline_events);

                // Render inline events
                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(content, inline_event, writer, &mut image_state, link_refs, options.disallowed_raw_html, footnote_store, footnote_order);
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
            // Defer heading open tag to HeadingEnd so we can generate the slug
            // from collected content before emitting the tag.
            heading_state.start();
            heading_state.level = *level;
        }
        BlockEvent::HeadingEnd { level } => {
            let content = heading_state.finish();

            // Emit heading open tag (deferred from HeadingStart)
            if options.heading_ids {
                let slug = generate_slug(content);
                let id = heading_id_tracker.unique_slug(slug);
                writer.heading_start_with_id(*level, &id);
            } else {
                writer.heading_start(*level);
            }

            if !content.is_empty() {
                inline_events.clear();
                inline_events.reserve((content.len() / 8).max(8));
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse_with_options(content, refs, options.allow_html, options.strikethrough, options.autolink_literals, options.math, footnote_store, inline_events);

                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(content, inline_event, writer, &mut image_state, link_refs, options.disallowed_raw_html, footnote_store, footnote_order);
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
            if options.disallowed_raw_html {
                writer.write_html_filtered(range.slice(input));
            } else {
                writer.write_bytes(range.slice(input));
            }
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
            } else if cell_state.in_cell {
                cell_state.add_text(text);
            } else {
                // Parse immediately (e.g., heading content)
                inline_events.clear();
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse_with_options(text, refs, options.allow_html, options.strikethrough, options.autolink_literals, options.math, footnote_store, inline_events);

                // Render inline events
                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(text, inline_event, writer, &mut image_state, link_refs, options.disallowed_raw_html, footnote_store, footnote_order);
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
        BlockEvent::BlockQuoteStart { callout } => {
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
            callout_stack.push(*callout);
            if let Some(ct) = callout {
                writer.callout_start(*ct);
            } else {
                writer.blockquote_start();
            }
        }
        BlockEvent::BlockQuoteEnd => {
            *blockquote_depth = blockquote_depth.saturating_sub(1);
            match callout_stack.pop() {
                Some(Some(_)) => writer.callout_end(),
                _ => writer.blockquote_end(),
            }
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
        BlockEvent::ListItemStart { task } => {
            writer.li_start();
            // In loose lists, defer newline until content appears (for empty items)
            if !in_tight_list {
                *pending_loose_li_newline = true;
            } else {
                // In tight lists, mark that we may need newline if block content follows
                *at_tight_li_start = true;
            }
            // Store task state for rendering at the start of paragraph content
            if options.task_lists {
                *pending_task = *task;
            }
        }
        BlockEvent::ListItemEnd => {
            *at_tight_li_start = false;
            *need_newline_before_block = false;
            *pending_loose_li_newline = false;
            *pending_task = block::TaskState::None;
            writer.li_end();
        }

        // --- Table events ---
        BlockEvent::TableStart => {
            if *pending_loose_li_newline {
                writer.newline();
                *pending_loose_li_newline = false;
            }
            if *need_newline_before_block {
                writer.newline();
                *need_newline_before_block = false;
            }
            if *at_tight_li_start {
                writer.newline();
                *at_tight_li_start = false;
            }
            writer.table_start();
        }
        BlockEvent::TableEnd => {
            writer.table_end();
        }
        BlockEvent::TableHeadStart => {
            *in_table_head = true;
            writer.thead_start();
        }
        BlockEvent::TableHeadEnd => {
            *in_table_head = false;
            writer.thead_end();
        }
        BlockEvent::TableBodyStart => {
            writer.tbody_start();
        }
        BlockEvent::TableBodyEnd => {
            writer.tbody_end();
        }
        BlockEvent::TableRowStart => {
            writer.tr_start();
        }
        BlockEvent::TableRowEnd => {
            writer.tr_end();
        }
        BlockEvent::TableCellStart { alignment } => {
            if *in_table_head {
                writer.th_start(*alignment);
            } else {
                writer.td_start(*alignment);
            }
            cell_state.start();
        }
        BlockEvent::TableCellEnd => {
            let content = cell_state.finish();
            if !content.is_empty() {
                inline_events.clear();
                inline_events.reserve((content.len() / 8).max(8));
                let refs = if options.allow_link_refs { Some(link_refs) } else { None };
                inline_parser.parse_with_options(content, refs, options.allow_html, options.strikethrough, options.autolink_literals, options.math, footnote_store, inline_events);

                let mut image_state = None;
                for inline_event in inline_events.iter() {
                    render_inline_event(content, inline_event, writer, &mut image_state, link_refs, options.disallowed_raw_html, footnote_store, footnote_order);
                }
            }
            if *in_table_head {
                writer.th_end();
            } else {
                writer.td_end();
            }
        }
    }
}

/// Emit a pending task checkbox and reset the state.
#[inline]
fn emit_pending_task_checkbox(pending_task: &mut block::TaskState, writer: &mut HtmlWriter) {
    match *pending_task {
        block::TaskState::Unchecked => {
            writer.write_bytes(b"<input type=\"checkbox\" disabled=\"\" /> ");
        }
        block::TaskState::Checked => {
            writer.write_bytes(b"<input type=\"checkbox\" checked=\"\" disabled=\"\" /> ");
        }
        block::TaskState::None => {}
    }
    *pending_task = block::TaskState::None;
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
    filter_html: bool,
    footnote_store: Option<&FootnoteStore>,
    footnote_order: &mut Vec<usize>,
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
        InlineEvent::StrikethroughStart => {
            if !in_image {
                writer.write_str("<del>");
            }
        }
        InlineEvent::StrikethroughEnd => {
            if !in_image {
                writer.write_str("</del>");
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
        InlineEvent::AutolinkLiteral { url, kind } => {
            use crate::inline::AutolinkLiteralKind;
            if in_image {
                writer.write_escaped_attr(url.slice(text));
            } else {
                writer.write_str("<a href=\"");
                match kind {
                    AutolinkLiteralKind::Url => {
                        writer.write_link_url(url.slice(text));
                    }
                    AutolinkLiteralKind::Www => {
                        writer.write_str("http://");
                        writer.write_link_url(url.slice(text));
                    }
                    AutolinkLiteralKind::Email => {
                        writer.write_str("mailto:");
                        writer.write_link_url(url.slice(text));
                    }
                }
                writer.write_str("\">");
                writer.write_escaped_text(url.slice(text));
                writer.write_str("</a>");
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
            } else if filter_html {
                writer.write_html_filtered(range.slice(text));
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
        InlineEvent::FootnoteRef { def_index } => {
            if !in_image {
                if let Some(fn_store) = footnote_store {
                    let def_idx = *def_index as usize;
                    // Assign sequential number based on first-appearance order
                    let number = if let Some(pos) = footnote_order.iter().position(|&i| i == def_idx) {
                        pos + 1
                    } else {
                        footnote_order.push(def_idx);
                        footnote_order.len()
                    };
                    if let Some(def) = fn_store.get(def_idx) {
                        writer.write_str("<sup><a href=\"#user-content-fn-");
                        writer.write_string(&def.label);
                        writer.write_str("\" id=\"user-content-fnref-");
                        writer.write_string(&def.label);
                        writer.write_str("\" data-footnote-ref>");
                        let num_str = number.to_string();
                        writer.write_string(&num_str);
                        writer.write_str("</a></sup>");
                    }
                }
            }
        }
        InlineEvent::MathInline(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                writer.write_str("<code class=\"language-math math-inline\">");
                let content = range.slice(text);
                for &b in content {
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
        InlineEvent::MathDisplay(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                writer.write_str("<code class=\"language-math math-display\">");
                let content = range.slice(text);
                for &b in content {
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
    }
}

/// Render the footnote section at document end.
fn render_footnote_section(
    input: &[u8],
    footnote_store: &FootnoteStore,
    footnote_order: &[usize],
    writer: &mut HtmlWriter,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
    link_refs: &LinkRefStore,
    options: &Options,
) {
    writer.write_str("<section data-footnotes class=\"footnotes\">\n<ol>\n");

    for (seq_num, &def_idx) in footnote_order.iter().enumerate() {
        let def = match footnote_store.get(def_idx) {
            Some(d) => d,
            None => continue,
        };
        let number = seq_num + 1;

        writer.write_str("<li id=\"user-content-fn-");
        writer.write_string(&def.label);
        writer.write_str("\">\n");

        // Render the footnote's block events
        let fn_events = &def.events;
        let fn_store_ref = Some(footnote_store);
        // We need a separate footnote_order for nested footnote refs inside footnotes
        // but for simplicity, we'll share the same order (GitHub does this too)
        let mut fn_footnote_order: Vec<usize> = Vec::new();

        let mut para_state = ParagraphState::new();
        let mut heading_state = HeadingState::new();
        let mut cell_state = CellState::new();
        let mut tight_list_stack: Vec<(bool, u32)> = Vec::new();
        let mut at_tight_li_start = false;
        let mut need_newline_before_block = false;
        let mut pending_loose_li_newline = false;
        let mut blockquote_depth = 0u32;
        let mut in_table_head = false;
        let mut pending_task = block::TaskState::None;
        let mut fn_heading_id_tracker = HeadingIdTracker::new();
        let mut fn_callout_stack: Vec<Option<block::CalloutType>> = Vec::new();

        // Track if the last event was ParagraphEnd (to insert backref)
        let last_para_end_idx = fn_events.iter().rposition(|e| matches!(e, BlockEvent::ParagraphEnd));

        for (i, event) in fn_events.iter().enumerate() {
            // If this is the last ParagraphEnd, we need to inject the backref before closing
            if Some(i) == last_para_end_idx {
                // Flush paragraph content first but don't close the tag yet
                // Actually, we need to render the paragraph content, inject backref, then close
                // The cleanest approach: render normally, then strip the closing </p>\n and re-add with backref

                // Capture writer position before this event
                let pos_before = writer.buffer_mut().len();

                render_block_event(
                    input,
                    event,
                    writer,
                    inline_parser,
                    inline_events,
                    &mut para_state,
                    &mut heading_state,
                    &mut cell_state,
                    &mut tight_list_stack,
                    &mut at_tight_li_start,
                    &mut need_newline_before_block,
                    &mut pending_loose_li_newline,
                    &mut blockquote_depth,
                    &mut in_table_head,
                    &mut pending_task,
                    link_refs,
                    fn_store_ref,
                    &mut fn_footnote_order,
                    &mut fn_heading_id_tracker,
                    &mut fn_callout_stack,
                    options,
                );

                // Check if the output ends with </p>\n and inject backref before it
                let buf = writer.buffer_mut();
                if buf.len() >= pos_before + 5 && buf.ends_with(b"</p>\n") {
                    let insert_pos = buf.len() - 5; // before </p>\n
                    let backref = format!(
                        " <a href=\"#user-content-fnref-{}\" class=\"data-footnote-backref\" aria-label=\"Back to reference {}\">\u{21a9}</a>",
                        def.label, number
                    );
                    let backref_bytes = backref.as_bytes();
                    let suffix = buf[insert_pos..].to_vec();
                    buf.truncate(insert_pos);
                    buf.extend_from_slice(backref_bytes);
                    buf.extend_from_slice(&suffix);
                }
                continue;
            }

            render_block_event(
                input,
                event,
                writer,
                inline_parser,
                inline_events,
                &mut para_state,
                &mut heading_state,
                &mut cell_state,
                &mut tight_list_stack,
                &mut at_tight_li_start,
                &mut need_newline_before_block,
                &mut pending_loose_li_newline,
                &mut blockquote_depth,
                &mut in_table_head,
                &mut pending_task,
                link_refs,
                fn_store_ref,
                &mut fn_footnote_order,
                &mut fn_heading_id_tracker,
                &mut fn_callout_stack,
                options,
            );
        }

        writer.write_str("</li>\n");
    }

    writer.write_str("</ol>\n</section>\n");
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
        // GFM disallowed raw HTML: <script> is filtered by default.
        assert_eq!(html, "&lt;script>alert('xss')&lt;/script>");
    }

    #[test]
    fn test_heading_h1() {
        let html = to_html("# Hello");
        assert!(html.contains("Hello</h1>"));
    }

    #[test]
    fn test_heading_h2() {
        let html = to_html("## World");
        assert!(html.contains("World</h2>"));
    }

    #[test]
    fn test_heading_all_levels() {
        for level in 1..=6 {
            let input = format!("{} Heading", "#".repeat(level));
            let html = to_html(&input);
            assert!(
                html.contains(&format!("Heading</h{level}>")),
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
        assert!(html.contains("Title</h1>"));
        assert!(html.contains("<p>Content here.</p>"));
    }

    #[test]
    fn test_heading_with_closing_hashes() {
        let html = to_html("# Hello #");
        assert!(html.contains("Hello</h1>"));
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

        assert!(html.contains("Main Title</h1>"));
        assert!(html.contains("Section 1</h2>"));
        assert!(html.contains("Section 2</h2>"));
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
        assert!(html.contains("Test</h1>"));
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
        assert!(html.contains("Title</h1>"));
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
