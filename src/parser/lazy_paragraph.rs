//! Whether a container's content still ends in an open paragraph.
//!
//! Block quotes and list items collect their content one line at a time and
//! hand the stripped text to a sub-parser afterwards, so while they collect
//! they have no block tree to consult. CommonMark lets a line that lacks the
//! container's marker continue the container only as paragraph continuation
//! text — laziness, spec 5.1 and 5.2 — so after a closed fence, an HTML
//! block, a heading or a table the container ends instead. This tracker
//! follows the stripped lines closely enough to answer that question: fenced
//! code and HTML blocks with their closing rules, ATX and setext headings,
//! thematic breaks, indented code, GFM tables and blank lines all close a
//! paragraph or keep one closed, and nested container markers are looked
//! through so the innermost content decides, the way the sub-parser will.
//!
//! It is a tracker, not a parser: a line is classified from its own bytes
//! and the state the lines before it left behind. That is exact for every
//! shape the conformance suites and the regression tests pin, and where a
//! contrived mix of nested containers could still fool it, the worst case is
//! the old behavior — a line absorbed into, or split off from, the
//! container — never a wrong block inside the sub-parse.

use memchr::{memchr, memmem};

use super::Parser;
use super::ParserOptions;
use super::html::HtmlBlockStart;
use super::reference::{fence_open, is_fence_close};

/// How an open HTML block ends (CommonMark 4.6).
#[derive(Clone, Copy)]
enum HtmlBlockEnd {
    /// Type 1: a line containing `</pre>`, `</script>`, `</style>` or
    /// `</textarea>`, in any case.
    Type1,
    /// Types 2–5: a line containing the terminator.
    Terminator(&'static str),
    /// Types 6–7: a blank line.
    Blank,
}

/// The nested container markers a line carried before its innermost content:
/// block quote markers and list markers, counted separately.
///
/// A GFM table's rows must sit under the same markers as its header; a line
/// under different ones starts something else, however row-like it looks.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Markers {
    quotes: u8,
    items: u8,
}

#[derive(Default)]
pub(super) struct OpenParagraph {
    /// The fence character and length of an open fenced code block.
    fence: Option<(u8, usize)>,
    /// The closing rule of an open HTML block.
    html: Option<HtmlBlockEnd>,
    /// The markers of an open GFM table; its rows continue until a blank line
    /// or a block start.
    table: Option<Markers>,
    /// The cell count of the previous line when it was paragraph text that
    /// could head a table, with the markers it sat under.
    header: Option<(Markers, usize)>,
    paragraph_open: bool,
}

impl OpenParagraph {
    /// Whether the content seen so far ends in a paragraph that a lazy line
    /// may continue.
    pub(super) fn paragraph_open(&self) -> bool {
        self.paragraph_open
    }

    /// Records a blank line: it closes a paragraph, a table and an HTML block
    /// of type 6 or 7.
    pub(super) fn observe_blank(&mut self) {
        if matches!(self.html, Some(HtmlBlockEnd::Blank)) {
            self.html = None;
        }
        self.paragraph_open = false;
        self.table = None;
        self.header = None;
    }

    /// Records one line of the container's stripped content.
    pub(super) fn observe(&mut self, line: &str, options: &ParserOptions) {
        if line.trim().is_empty() {
            self.observe_blank();
            return;
        }

        // Nested container markers are stripped first, so a fence or an
        // HTML block inside a nested quote or item closes on its own lines.
        let (content, markers) = strip_container_markers(line, self.paragraph_open);
        let (indent_bytes, indent_columns) = leading_indent(content);
        let trimmed = &content[indent_bytes..];

        if let Some((fence_byte, fence_len)) = self.fence {
            if indent_columns < 4 && is_fence_close(trimmed, fence_byte, fence_len) {
                self.fence = None;
            }
            self.close_paragraph();
            return;
        }
        if let Some(end) = self.html {
            if html_block_ends(trimmed, end) {
                self.html = None;
            }
            self.close_paragraph();
            return;
        }

        let table = self.table.take();

        if trimmed.is_empty() {
            // Only markers: an empty item or quote, which holds no paragraph.
            self.close_paragraph();
            return;
        }
        if indent_columns >= 4 {
            // Indented code, unless a paragraph is open — then the line is
            // its continuation text however far it is indented.
            if !self.paragraph_open {
                self.close_paragraph();
            } else if let Some(markers) = table {
                self.continue_table(markers);
            }
            return;
        }
        if Parser::try_parse_thematic_break_line(content) || is_atx_heading(trimmed) {
            self.close_paragraph();
            return;
        }
        if self.paragraph_open && is_setext_underline(trimmed) {
            self.close_paragraph();
            return;
        }
        if let Some(fence) = fence_open(trimmed) {
            self.fence = Some(fence);
            self.close_paragraph();
            return;
        }
        if let Some(start) = Parser::parse_html_block_start(trimmed) {
            self.open_html(trimmed, start);
            return;
        }
        // A type-7 block — one complete tag on its own line — cannot
        // interrupt a paragraph; with one open, the line is its text.
        if !self.paragraph_open && Parser::is_html_block_type7_line(trimmed) {
            self.open_html(trimmed, HtmlBlockStart::Other);
            return;
        }
        if options.tables {
            if let Some((header_markers, header_cells)) = self.header
                && self.paragraph_open
                && header_markers == markers
                && Parser::table_delimiter_cells(trimmed) == Some(header_cells)
            {
                self.header = None;
                self.paragraph_open = false;
                self.table = Some(markers);
                return;
            }
            if let Some(table_markers) = table
                && table_markers == markers
            {
                self.continue_table(markers);
                return;
            }
        }

        // Paragraph text, which the next line may still turn into a table
        // header when it holds a pipe.
        self.paragraph_open = true;
        self.header = (options.tables && memchr(b'|', trimmed.as_bytes()).is_some()).then(|| {
            (
                markers,
                Parser::table_header_cells(options.merged_table_cells, trimmed),
            )
        });
    }

    fn close_paragraph(&mut self) {
        self.paragraph_open = false;
        self.table = None;
        self.header = None;
    }

    fn continue_table(&mut self, markers: Markers) {
        self.paragraph_open = false;
        self.table = Some(markers);
        self.header = None;
    }

    fn open_html(&mut self, trimmed: &str, start: HtmlBlockStart) {
        let end = match start {
            HtmlBlockStart::Comment => HtmlBlockEnd::Terminator("-->"),
            HtmlBlockStart::Terminated(terminator) => HtmlBlockEnd::Terminator(terminator),
            HtmlBlockStart::Type1 => HtmlBlockEnd::Type1,
            HtmlBlockStart::Other => HtmlBlockEnd::Blank,
        };
        // Types 1–5 may close on their opening line.
        if !html_block_ends(trimmed, end) {
            self.html = Some(end);
        }
        self.close_paragraph();
    }
}

/// Removes the nested container markers a line starts with: block quote
/// markers, and list markers when the line can start an item where it is.
///
/// A list marker only interrupts an open paragraph when the item it opens is
/// non-empty and, for an ordered item, numbered one; otherwise the line is
/// paragraph text and keeps its marker.
fn strip_container_markers(line: &str, paragraph_open: bool) -> (&str, Markers) {
    let mut content = line;
    let mut markers = Markers::default();
    loop {
        let (indent_bytes, indent_columns) = leading_indent(content);
        if indent_columns >= 4 {
            return (content, markers);
        }
        let trimmed = &content[indent_bytes..];
        if let Some(rest) = trimmed.strip_prefix('>') {
            content = rest.strip_prefix(' ').unwrap_or(rest);
            markers.quotes = markers.quotes.saturating_add(1);
            continue;
        }
        if Parser::try_parse_thematic_break_line(content) || !Parser::try_parse_list_line(trimmed) {
            return (content, markers);
        }
        // With a paragraph open, only an interrupting item is a marker; a
        // paragraph is open below every marker stripped so far too, so the
        // first marker on the line decides for the rest.
        if paragraph_open
            && markers == Markers::default()
            && !Parser::try_parse_list_interrupt(trimmed)
        {
            return (content, markers);
        }
        let bytes = trimmed.as_bytes();
        let mut marker_end = if bytes[0].is_ascii_digit() {
            bytes
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count()
                + 1
        } else {
            1
        };
        while matches!(bytes.get(marker_end), Some(b' ' | b'\t')) {
            marker_end += 1;
        }
        content = &trimmed[marker_end..];
        markers.items = markers.items.saturating_add(1);
    }
}

/// The leading spaces and tabs of `content`, as bytes and as columns.
fn leading_indent(content: &str) -> (usize, usize) {
    let mut columns = 0usize;
    for (index, byte) in content.bytes().enumerate() {
        match byte {
            b' ' => columns += 1,
            b'\t' => columns = (columns / 4 + 1) * 4,
            _ => return (index, columns),
        }
    }
    (content.len(), columns)
}

fn is_atx_heading(trimmed: &str) -> bool {
    let bytes = trimmed.as_bytes();
    let hashes = bytes.iter().take_while(|&&byte| byte == b'#').count();
    (1..=6).contains(&hashes) && matches!(bytes.get(hashes), None | Some(b' ' | b'\t'))
}

/// A setext underline of `=` (the `-` form is a thematic break as well and
/// is recognized as one before this runs).
fn is_setext_underline(trimmed: &str) -> bool {
    let bytes = trimmed.trim_end().as_bytes();
    !bytes.is_empty() && bytes.iter().all(|&byte| byte == b'=')
}

fn html_block_ends(trimmed: &str, end: HtmlBlockEnd) -> bool {
    match end {
        HtmlBlockEnd::Type1 => contains_type1_closer(trimmed.as_bytes()),
        HtmlBlockEnd::Terminator(terminator) => {
            memmem::find(trimmed.as_bytes(), terminator.as_bytes()).is_some()
        }
        HtmlBlockEnd::Blank => false,
    }
}

/// Whether the line contains `</pre>`, `</script>`, `</style>` or
/// `</textarea>`, in any case: any of the four ends a type-1 block. The
/// block parser owns that rule, so the tracker reuses its scanner.
fn contains_type1_closer(bytes: &[u8]) -> bool {
    super::html::find_type1_end_tag(bytes, 0).is_some()
}
