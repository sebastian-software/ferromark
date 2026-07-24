//! MDX segmentation, rendering, diagnostics, and semantic events.
//!
//! MDX combines Markdown with JSX/JavaScript. Instead of parsing the full MDX
//! syntax, this module splits the input into typed blocks. Only the Markdown
//! segments need to go through ferromark's Markdown parser; JSX, expressions,
//! and ESM statements are passed through unchanged.
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
//! - Multi-line ESM uses blank-line termination, not parser-driven boundaries
//! - Exotic edge cases (e.g. `export var a = 1\nvar b`) may be grouped differently
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
//! indentation) and can detect JSX/ESM inside them. [`parse_events`] still
//! exposes well-delimited tags and expressions in container prose as inline
//! MDX events, which is sufficient for consumers that need to separate
//! translatable text from syntax without changing Markdown flow semantics.
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
pub mod expr;
pub mod jsx_tag;
pub mod render;
mod splitter;
mod strict;

pub use events::{
    MDX_EVENT_STREAM_VERSION, MdxEvent, MdxEventStream, parse_events, parse_events_strict,
};

/// A typed segment of an MDX document.
///
/// All variants are zero-copy `&str` slices into the original input.
#[derive(Debug, Clone, PartialEq, Eq)]
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
/// complete input without gaps or overlap.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdxDiagnosticCode {
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
}

/// A structural MDX diagnostic returned by [`segment_strict`].
///
/// `primary_range` is always a valid UTF-8 byte range into the original input.
/// For a mismatched JSX closing tag, `related_range` identifies the innermost
/// opening tag that the closing tag cannot close past.
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
/// input — no bytes are dropped.
#[must_use]
pub fn segment(input: &str) -> Vec<Segment<'_>> {
    splitter::split(input)
}

/// Segment an MDX document and retain exact byte ranges for each segment.
///
/// This is the source-location-aware counterpart to [`segment`]. It has the
/// same segmentation semantics, while each result records its position in the
/// original UTF-8 input. The range includes every byte represented by the
/// segment, including MDX delimiters and any trailing newline owned by that
/// segment.
///
/// # Panics
///
/// Panics when `input` is larger than [`u32::MAX`] bytes, matching the size
/// limit of [`crate::Range`].
#[must_use]
pub fn segment_spanned(input: &str) -> Vec<SpannedSegment<'_>> {
    let input_start = input.as_ptr() as usize;

    segment(input)
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
/// # Panics
///
/// Panics when `input` is larger than [`u32::MAX`] bytes, matching the size
/// limit of [`crate::Range`].
pub fn segment_strict(input: &str) -> Result<Vec<SpannedSegment<'_>>, Vec<MdxDiagnostic>> {
    strict::segment_strict(input)
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
    assert!(
        byte_offset <= input.len(),
        "byte offset is outside the input"
    );
    assert!(
        input.is_char_boundary(byte_offset),
        "byte offset is not a UTF-8 character boundary"
    );

    let before = &input[..byte_offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |offset| offset + 1);
    let column = input[line_start..byte_offset].chars().count() + 1;

    SourceLocation {
        line: u32::try_from(line).expect("line number exceeds u32::MAX"),
        column: u32::try_from(column).expect("column number exceeds u32::MAX"),
    }
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

pub use render::{MdxOutput, render, render_with_options};
