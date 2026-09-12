//! MDX segmentation, rendering, diagnostics, and semantic events.
//!
//! MDX combines Markdown with JSX/JavaScript. Instead of parsing the full MDX
//! syntax, this module splits the input into typed blocks. Only the Markdown
//! segments need to go through ferromark's Markdown parser; JSX, expressions,
//! and ESM statements are passed through unchanged.
//!
//! All entry points ignore one leading UTF-8 byte-order mark. Source ranges
//! remain absolute offsets into the original input, so content after a BOM
//! starts at byte 3.
//!
//! This module is gated behind the `mdx` Cargo feature.
//!
//! # Example
//! ```
//! use ferromark::mdx::{segment, Segment};
//!
//! let input = "import A from 'a'\n\n# Hello\n\n<Card>\nWorld\n</Card>\n";
//! let segments = segment(input);
//! for seg in &segments {
//!     match seg {
//!         Segment::Markdown(md) => {
//!             // Parse with ferromark::to_html(md)
//!         }
//!         _ => {
//!             // Pass through unchanged
//!         }
//!     }
//! }
//! ```
//!
//! Compiler consumers that need Markdown block boundaries and resolved inline
//! semantics can use the separate, collected event stream:
//!
//! ```
//! use ferromark::InlineEvent;
//! use ferromark::mdx::{MdxEvent, parse_events};
//!
//! let input = "# Hello {name}\n";
//! let stream = parse_events(input);
//! let translatable = stream.events.iter().filter_map(|event| match event {
//!     MdxEvent::Inline(InlineEvent::Text(range)) => {
//!         Some(range.slice_str(input.as_bytes()).unwrap())
//!     }
//!     _ => None,
//! }).collect::<Vec<_>>();
//! assert_eq!(translatable, vec!["Hello "]);
//! ```
//!
//! [`parse_events`] is opt-in and does not participate in the default HTML
//! rendering path. Its flat ordering and balancing contract is versioned by
//! [`MDX_EVENT_STREAM_VERSION`].
//!
//! # Differences from the official mdxjs compiler
//!
//! This segmenter covers the block-level MDX patterns used in real-world
//! documentation (Next.js, Docusaurus, Astro). It intentionally does **not**
//! replicate the full `@mdx-js/mdx` compiler. The differences:
//!
//! ## Block-level segmentation
//!
//! [`segment`] detects JSX and expressions at block level (start of a line).
//! Inline JSX (`paragraph with <em>JSX</em> inside`) and inline expressions
//! (`text {variable} here`) stay inside Markdown segments and are **not** split
//! out. For consumers that need typed inline constructs, the opt-in
//! [`crate::InlineParser::parse_mdx`] method emits source-ranged MDX inline
//! events while preserving the surrounding Markdown events. The official mdxjs
//! compiler handles both flow and text positions in a single parse.
//!
//! ## No JavaScript validation
//!
//! Official mdxjs pipes ESM and expressions through acorn (or swc) to validate
//! the JavaScript syntax. We use heuristics: `import`/`export` at column 0,
//! brace-depth counting for expressions. This means:
//! - We won't reject syntactically invalid JS (e.g. `export const = ;`)
//! - Multi-line ESM follows lexical continuation boundaries (delimiters,
//!   strings, comments, templates, and module clauses), not a full JS parser
//! - A terminal expression followed on the next line by a valid ECMAScript
//!   suffix (`[index]`, `(args)`, `.member`, or a tagged template) continues
//!   as ESM. This intentionally follows ECMAScript's newline/ASI rules even
//!   where the same bytes could be Markdown; put a semicolon or blank line
//!   before Markdown to make that boundary explicit.
//! - Incomplete ESM falls back to Markdown in permissive mode and is diagnosed
//!   by [`segment_strict`]
//!
//! ## No Markdown syntax modifications
//!
//! Official mdxjs alters the Markdown grammar:
//! - **Indented code blocks disabled** — 4-space indented lines are paragraphs
//! - **HTML (flow + inline) disabled** — `<div>` is always JSX, never raw HTML
//! - **Autolinks disabled** — `<https://...>` is JSX, not an autolink
//!
//! We leave the Markdown parser untouched. Markdown segments are parsed with
//! standard CommonMark/GFM rules. This is a deliberate trade-off: it keeps
//! ferromark's core parser unmodified and lets the caller decide how to handle
//! HTML-like syntax inside Markdown segments.
//!
//! ## No container awareness
//!
//! Flow JSX/ESM inside block containers is not detected:
//! ```text
//! > <Component>   ← treated as blockquote + markdown, not JSX
//! - import x      ← treated as list item, not ESM
//! ```
//!
//! The official compiler tracks container context (blockquote markers, list
//! indentation) and can detect JSX/ESM inside them. [`parse_events`] promotes a
//! container paragraph containing only one well-delimited tag or expression to
//! a flow event while preserving the surrounding Markdown container events.
//! Mixed prose keeps its inline MDX events. Multiline constructs split across
//! repeated container prefixes and container-local ESM remain Markdown
//! recovery.
//!
//! ## No TypeScript generics in JSX
//!
//! `<Component<T>>` with TypeScript generics is not supported by the tag
//! parser. The official compiler (when configured with acorn-jsx + TypeScript)
//! handles this.
//!
//! ## Silent fallback instead of errors
//!
//! [`segment`] and [`segment_spanned`] preserve the original permissive
//! behavior: invalid JSX or unterminated expressions are treated as Markdown.
//! [`segment_strict`] is an opt-in validation pass that returns structural MDX
//! diagnostics with source ranges instead. It does not validate JavaScript or
//! TypeScript syntax inside otherwise well-delimited ESM and expressions.

mod events;
/// JavaScript-expression boundary scanning used by MDX parsers.
pub mod expr;
/// JSX tag parsing and structural metadata.
pub mod jsx_tag;
/// Rendering MDX segments as a JavaScript component module.
pub mod render;
mod splitter;
mod strict;

pub use events::{
    MDX_EVENT_STREAM_VERSION, MdxEvent, MdxEventStream, parse_events, parse_events_strict,
    try_parse_events,
};

/// A typed segment of an MDX document.
///
/// All variants are zero-copy `&str` slices into the original input.
/// Future releases may add segment types. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Segment<'a> {
    /// ESM statement (`import` / `export`) — pass through unchanged.
    Esm(&'a str),
    /// Markdown content — parse with ferromark's Markdown parser.
    Markdown(&'a str),
    /// JSX block opening tag (e.g. `<Component prop="x">`).
    JsxBlockOpen(&'a str),
    /// JSX block closing tag (e.g. `</Component>`).
    JsxBlockClose(&'a str),
    /// JSX self-closing block tag (e.g. `<Component />`).
    JsxBlockSelfClose(&'a str),
    /// JavaScript expression (e.g. `{expression}`).
    Expression(&'a str),
}

/// A typed MDX segment together with its exact byte range in the input.
///
/// The range covers precisely the bytes in [`Self::segment`], including
/// delimiters, indentation, and a trailing line ending when the segmenter
/// includes one. The returned ranges are ordered, contiguous, and cover the
/// complete parsed content without gaps or overlap. A leading UTF-8 byte-order
/// mark is ignored, so it is not covered by a segment range.
///
/// Like [`Segment`], this type borrows from the input and performs no copying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedSegment<'a> {
    /// The zero-copy MDX segment.
    pub segment: Segment<'a>,
    /// Exact UTF-8 byte range of [`Self::segment`] in the original input.
    pub range: crate::Range,
}

/// A stable category for a structural MDX diagnostic.
///
/// Its [`std::fmt::Display`] representation is a stable kebab-case identifier
/// suitable for logs and command-line diagnostics.
///
/// Future releases may add diagnostic categories. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MdxDiagnosticCode {
    /// The document exceeds the maximum size representable by source ranges.
    InputTooLarge,
    /// A flow expression has no closing `}`.
    UnterminatedExpression,
    /// A JSX tag has no closing `>`.
    UnterminatedJsxTag,
    /// A JSX tag has an invalid name, attribute, or closing-tag structure.
    InvalidJsxTag,
    /// A closing JSX tag does not have a matching opening tag.
    UnexpectedJsxClosingTag,
    /// A closing JSX tag does not match the innermost opening tag.
    MismatchedJsxClosingTag,
    /// An opening JSX tag is not closed before the end of the document.
    UnclosedJsxTag,
    /// An ESM block is indented or interrupts a Markdown paragraph.
    InvalidEsmPosition,
    /// An ESM declaration needs a continuation that could not be safely separated from Markdown.
    IncompleteEsm,
}

impl std::fmt::Display for MdxDiagnosticCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InputTooLarge => "input-too-large",
            Self::UnterminatedExpression => "unterminated-expression",
            Self::UnterminatedJsxTag => "unterminated-jsx-tag",
            Self::InvalidJsxTag => "invalid-jsx-tag",
            Self::UnexpectedJsxClosingTag => "unexpected-jsx-closing-tag",
            Self::MismatchedJsxClosingTag => "mismatched-jsx-closing-tag",
            Self::UnclosedJsxTag => "unclosed-jsx-tag",
            Self::InvalidEsmPosition => "invalid-esm-position",
            Self::IncompleteEsm => "incomplete-esm",
        })
    }
}

/// A structural MDX diagnostic returned by [`segment_strict`].
///
/// `primary_range` is always a valid UTF-8 byte range into the original input.
/// For a mismatched JSX closing tag, `related_range` identifies the innermost
/// opening tag that the closing tag cannot close past.
/// Its [`std::fmt::Display`] representation includes the diagnostic code,
/// message, primary byte range, and related byte range when present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MdxDiagnostic {
    /// Stable machine-readable diagnostic category.
    pub code: MdxDiagnosticCode,
    /// Concise human-readable explanation.
    pub message: &'static str,
    /// Primary source range for this diagnostic.
    pub primary_range: crate::Range,
    /// Related source range for a mismatched JSX closing tag's blocking opening tag.
    pub related_range: Option<crate::Range>,
}

impl std::fmt::Display for MdxDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}: {} at bytes {}..{}",
            self.code, self.message, self.primary_range.start, self.primary_range.end
        )?;
        if let Some(related_range) = self.related_range {
            write!(
                formatter,
                ", related bytes {}..{}",
                related_range.start, related_range.end
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for MdxDiagnostic {}

/// A one-based source location derived from a UTF-8 byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// One-based line number.
    pub line: u32,
    /// One-based Unicode scalar column number.
    pub column: u32,
}

/// Segment an MDX document into typed blocks.
///
/// This is the primary entry point. The returned segments cover the entire
/// input except for an optional leading UTF-8 byte-order mark, which is
/// ignored before parsing.
///
/// # Panics
///
/// Panics when the input exceeds [`crate::MAX_INPUT_BYTES`]. Use
/// [`try_segment`] to handle the limit as an error.
#[must_use]
pub fn segment(input: &str) -> Vec<Segment<'_>> {
    try_segment(input).unwrap_or_else(|error| panic!("{error}"))
}

/// Segment an MDX document without panicking for oversized input.
pub fn try_segment(input: &str) -> Result<Vec<Segment<'_>>, crate::InputSizeError> {
    crate::validate_input_size(input.len())?;
    let (content, _) = content_after_bom(input);
    Ok(splitter::split(content))
}

/// Segment an MDX document and retain exact byte ranges for each segment.
///
/// This is the source-location-aware counterpart to [`segment`]. It has the
/// same segmentation semantics, while each result records its position in the
/// original UTF-8 input. The range includes every byte represented by the
/// segment, including MDX delimiters and any trailing newline owned by that
/// segment. An optional leading UTF-8 byte-order mark is ignored, so the first
/// range begins at byte 3 when one is present.
///
/// # Panics
///
/// Panics when the input exceeds [`crate::MAX_INPUT_BYTES`]. Use
/// [`try_segment_spanned`] to handle the limit as an error.
///
#[must_use]
pub fn segment_spanned(input: &str) -> Vec<SpannedSegment<'_>> {
    try_segment_spanned(input).unwrap_or_else(|error| panic!("{error}"))
}

/// Segment an MDX document with byte ranges without panicking for oversized
/// input.
pub fn try_segment_spanned(input: &str) -> Result<Vec<SpannedSegment<'_>>, crate::InputSizeError> {
    crate::validate_input_size(input.len())?;
    let (content, _) = content_after_bom(input);
    Ok(spanned_segments(input, splitter::split(content)))
}

pub(crate) fn segment_spanned_with_expression_ends<'a>(
    input: &'a str,
    content: &'a str,
    expression_ends: &expr::ExpressionEnds,
) -> Vec<SpannedSegment<'a>> {
    spanned_segments(
        input,
        splitter::split_with_expression_ends(content, expression_ends),
    )
}

pub(crate) fn content_after_bom(input: &str) -> (&str, usize) {
    input
        .strip_prefix('\u{feff}')
        .map_or((input, 0), |content| (content, '\u{feff}'.len_utf8()))
}

fn spanned_segments<'a>(input: &'a str, segments: Vec<Segment<'a>>) -> Vec<SpannedSegment<'a>> {
    let input_start = input.as_ptr() as usize;

    segments
        .into_iter()
        .map(|segment| {
            let text = segment.as_str();
            let start = (text.as_ptr() as usize)
                .checked_sub(input_start)
                .expect("MDX segment must borrow from its input");
            let end = start + text.len();
            let range = crate::Range::from_usize(start, end);
            SpannedSegment { segment, range }
        })
        .collect()
}

/// Validate structural MDX and return source-spanned segments on success.
///
/// This opt-in API adds diagnostics for malformed flow expressions, malformed
/// JSX tags, JSX tag nesting, and ESM blocks at invalid boundaries. The
/// permissive [`segment`] APIs deliberately retain their silent Markdown
/// fallback. JavaScript and TypeScript inside a correctly delimited expression
/// or ESM block are not parsed or type-checked.
///
/// When a malformed construct makes later segmentation ambiguous, validation
/// stops at that construct. Otherwise, independent diagnostics are collected.
///
pub fn segment_strict(input: &str) -> Result<Vec<SpannedSegment<'_>>, Vec<MdxDiagnostic>> {
    validate_mdx_input_size(input.len())?;
    strict::segment_strict(input)
}

pub(super) fn validate_mdx_input_size(input_len: usize) -> Result<(), Vec<MdxDiagnostic>> {
    crate::validate_input_size(input_len).map_err(|_| vec![input_size_diagnostic()])
}

fn input_size_diagnostic() -> MdxDiagnostic {
    MdxDiagnostic {
        code: MdxDiagnosticCode::InputTooLarge,
        message: "input exceeds the maximum supported size of 4294967294 bytes",
        primary_range: crate::Range::empty_at(0),
        related_range: None,
    }
}

/// Translate a UTF-8 byte offset into a one-based line and Unicode scalar column.
///
/// `byte_offset` must be at a UTF-8 character boundary and may equal
/// `input.len()`. Diagnostic range boundaries returned by [`segment_strict`]
/// always meet that requirement.
///
/// # Panics
///
/// Panics when `byte_offset` is greater than `input.len()` or is not a UTF-8
/// character boundary.
#[must_use]
pub fn source_location(input: &str, byte_offset: usize) -> SourceLocation {
    crate::range::assert_input_size(input.len());
    assert!(
        byte_offset <= input.len(),
        "byte offset is outside the input"
    );
    assert!(
        input.is_char_boundary(byte_offset),
        "byte offset is not a UTF-8 character boundary"
    );

    let before = &input[..byte_offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = before.rfind('\n').map_or(0, |offset| offset + 1);
    let column = input[line_start..byte_offset].chars().count();

    SourceLocation {
        line: one_based_source_count(line),
        column: one_based_source_count(column),
    }
}

fn one_based_source_count(zero_based_count: usize) -> u32 {
    let one_based_count = zero_based_count
        .checked_add(1)
        .expect("source position exceeds usize::MAX");
    u32::try_from(one_based_count).expect("source position exceeds u32::MAX")
}

impl<'a> Segment<'a> {
    /// Return the source text represented by this segment.
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        match self {
            Self::Esm(text)
            | Self::Markdown(text)
            | Self::JsxBlockOpen(text)
            | Self::JsxBlockClose(text)
            | Self::JsxBlockSelfClose(text)
            | Self::Expression(text) => text,
        }
    }
}

pub use render::{MdxOutput, render, render_with_options, try_render, try_render_with_options};

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn strict_size_validation_returns_a_stable_diagnostic() {
        let diagnostics = validate_mdx_input_size(crate::MAX_INPUT_BYTES + 1).unwrap_err();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, MdxDiagnosticCode::InputTooLarge);
        assert_eq!(diagnostics[0].primary_range, crate::Range::empty_at(0));
        assert!(
            diagnostics[0]
                .message
                .contains(&crate::MAX_INPUT_BYTES.to_string())
        );
    }

    #[test]
    fn maximum_input_length_preserves_one_based_source_positions() {
        assert_eq!(one_based_source_count(crate::MAX_INPUT_BYTES), u32::MAX);
    }

    #[test]
    fn diagnostic_codes_have_stable_display_names() {
        let cases = [
            (MdxDiagnosticCode::InputTooLarge, "input-too-large"),
            (
                MdxDiagnosticCode::UnterminatedExpression,
                "unterminated-expression",
            ),
            (
                MdxDiagnosticCode::UnterminatedJsxTag,
                "unterminated-jsx-tag",
            ),
            (MdxDiagnosticCode::InvalidJsxTag, "invalid-jsx-tag"),
            (
                MdxDiagnosticCode::UnexpectedJsxClosingTag,
                "unexpected-jsx-closing-tag",
            ),
            (
                MdxDiagnosticCode::MismatchedJsxClosingTag,
                "mismatched-jsx-closing-tag",
            ),
            (MdxDiagnosticCode::UnclosedJsxTag, "unclosed-jsx-tag"),
            (
                MdxDiagnosticCode::InvalidEsmPosition,
                "invalid-esm-position",
            ),
            (MdxDiagnosticCode::IncompleteEsm, "incomplete-esm"),
        ];

        for (code, expected) in cases {
            assert_eq!(code.to_string(), expected);
        }
    }

    #[test]
    fn diagnostics_format_their_code_message_and_ranges() {
        fn assert_error<T: std::error::Error>() {}

        assert_error::<MdxDiagnostic>();
        let diagnostic = MdxDiagnostic {
            code: MdxDiagnosticCode::MismatchedJsxClosingTag,
            message: "closing JSX tag does not match the innermost opening tag",
            primary_range: crate::Range::new(16, 24),
            related_range: Some(crate::Range::new(8, 15)),
        };

        assert_eq!(
            diagnostic.to_string(),
            "mismatched-jsx-closing-tag: closing JSX tag does not match the innermost opening tag at bytes 16..24, related bytes 8..15"
        );
    }

    #[test]
    fn rendered_mdx_output_is_debuggable() {
        let output = render("# Title\n");
        let debug = format!("{output:?}");

        assert!(debug.contains("MdxOutput"));
        assert!(debug.contains("<h1 id=\\\"title\\\">Title</h1>"));
    }
}
