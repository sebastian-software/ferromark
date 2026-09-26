//! Optional ordered passes over Ferromark's arena-backed Markdown AST.
//!
//! This crate depends on [`ferromark`] and is not a dependency of the core
//! parser or renderer. Construct a [`TransformPipeline`] only when AST passes
//! are needed; an empty pipeline performs no traversal or URL scan.
//!
//! # Custom pass
//!
//! A pass borrows the parsed document and arena for one call. The same pass
//! object can be reused for later documents, but it must not retain references
//! into an arena that the caller may reset. See the packaged
//! [`custom pass example`](https://github.com/sebastian-software/ferromark/tree/main/transforms/examples/custom_pass.rs).
//! Pass objects are `Send`, so callers can move a configured pipeline to the
//! thread that owns a document; documents and contexts stay thread-local.

#![deny(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::ops::Range;

use ferromark::ast::{Document, Node, Span, Text};
use ferromark::{Allocator, AutolinkMatcher, HtmlRendererOptions};

/// The boxed, thread-safe error type returned by custom transform passes.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// A stateful pass over one parsed document.
///
/// Passes run in insertion order. A pass may mutate the document before it
/// returns an error; the pipeline does not roll back changes. Callers should
/// discard or reparse that document after a failure. Implementations must keep
/// all document-specific references local to [`Self::apply`], since callers
/// may reset the arena as soon as they have finished rendering the document.
/// Pass values must be `Send` so a configured pipeline can move to the worker
/// that owns a document.
pub trait TransformPass: Send {
    /// Stable name included in pipeline errors.
    fn name(&self) -> &'static str;

    /// Applies this pass to one document.
    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError>;
}

/// Ordered collection of reusable AST transform passes.
#[derive(Default)]
pub struct TransformPipeline {
    passes: Vec<Box<dyn TransformPass>>,
}

impl TransformPipeline {
    /// Creates an empty pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a pass. Passes are called in the order they are added.
    pub fn add<P: TransformPass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    /// Returns the number of passes in this pipeline.
    #[must_use]
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Returns whether this pipeline contains no passes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }

    /// Runs each pass in insertion order and stops at the first error.
    ///
    /// The pipeline stores no reference to `document`, `context`, or the arena,
    /// and can be reused for another document after this call. If a pass fails,
    /// earlier changes (including changes made by the failing pass) remain in
    /// the document; discard or reparse it before rendering.
    pub fn run<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), TransformError> {
        for (pass_index, pass) in self.passes.iter_mut().enumerate() {
            let pass_name = pass.name();
            if let Err(source) = pass.apply(document, context) {
                return Err(TransformError {
                    pass_index,
                    pass_name,
                    source,
                });
            }
        }

        Ok(())
    }
}

/// Context available to a pass while one document is being transformed.
///
/// The context only scans for URLs when a pass explicitly calls
/// [`TextRun::protected_url_ranges`]. It builds one reusable matcher from the
/// supplied renderer options, so disabled autolinks and custom prefixes match
/// the renderer exactly.
pub struct TransformContext<'arena> {
    allocator: &'arena Allocator,
    source: &'arena str,
    autolink_matcher: Option<AutolinkMatcher>,
}

impl<'arena> TransformContext<'arena> {
    /// Creates a context for the exact source and allocator used to parse a document.
    ///
    /// The caller must pass the same allocator and source that produced the
    /// document; the public AST does not retain either identity for validation.
    /// Renderer options are copied into one reusable URL matcher for the
    /// optional URL-protection helper. Construct the renderer from those same
    /// options (or an unchanged clone) so URL protection and output stay aligned.
    /// This matcher is not used unless a pass explicitly scans a text run.
    #[must_use]
    pub fn new(
        allocator: &'arena Allocator,
        source: &'arena str,
        renderer_options: &HtmlRendererOptions,
    ) -> Self {
        Self {
            allocator,
            source,
            autolink_matcher: renderer_options.autolink_matcher(),
        }
    }

    /// Returns the source text parsed into the document.
    #[must_use]
    pub const fn source(&self) -> &'arena str {
        self.source
    }

    /// Allocates a string that can be stored in this document's AST.
    #[must_use]
    pub fn alloc_str(&self, value: &str) -> &'arena str {
        let allocator: &'arena Allocator = self.allocator;
        allocator.alloc_str(value)
    }

    /// Creates text for a replacement while preserving its original source span.
    #[must_use]
    pub fn replacement_text(&self, value: &str, span: Span) -> Text<'arena> {
        Text {
            value: self.alloc_str(value),
            span,
        }
    }

    /// Creates text that was inserted without replacing source content.
    ///
    /// Generated text uses [`Span::empty`]. This intentionally does not
    /// distinguish generated content from a genuine empty source span.
    #[must_use]
    pub fn generated_text(&self, value: &str) -> Text<'arena> {
        self.replacement_text(value, Span::empty())
    }

    /// Replaces a text node's value and leaves its source span unchanged.
    pub fn replace_text_value(&self, text: &mut Text<'arena>, value: &str) {
        text.value = self.alloc_str(value);
    }

    /// Replaces a byte range within adjacent text nodes and returns its source span.
    ///
    /// `node_range` selects one contiguous run from the `nodes` slice. The byte
    /// range addresses the run's coalesced text and must use UTF-8 boundaries.
    /// Unchanged prefix and suffix text retain the source ranges of their
    /// contributing nodes; replacement text receives the bounding span of the
    /// source nodes it overlaps. A zero-width insertion receives an empty span.
    /// If one parsed text node decodes to fewer bytes than its source spelling
    /// (for example a character reference), every piece split from that node
    /// keeps the full node span; this API does not provide character-level maps.
    pub fn replace_text_range(
        &self,
        nodes: &mut ferromark::allocator::Vec<'arena, Node<'arena>>,
        node_range: Range<usize>,
        byte_range: Range<usize>,
        replacement: &str,
    ) -> Result<Span, TextReplacementError> {
        validate_node_range(nodes, &node_range)?;
        validate_text_byte_range(nodes, &node_range, &byte_range)?;

        if byte_range.is_empty() && replacement.is_empty() {
            return Ok(Span::empty());
        }

        let replacement_span = span_for_byte_range(nodes, &node_range, &byte_range);
        if byte_range.is_empty() {
            insert_at_text_offset(self, nodes, &node_range, byte_range.start, replacement);
            return Ok(replacement_span);
        }

        let (first, last, first_offset, last_offset) =
            overlapping_text_nodes(nodes, &node_range, &byte_range)?;
        let (first_value, first_span) = match &nodes[first] {
            Node::Text(text) => (text.value, text.span),
            _ => return Err(TextReplacementError::NonTextNode { index: first }),
        };
        let (last_value, last_span) = match &nodes[last] {
            Node::Text(text) => (text.value, text.span),
            _ => return Err(TextReplacementError::NonTextNode { index: last }),
        };

        // Rebuild only the nodes whose text overlaps the edit. Prefix and
        // suffix slices borrow the original arena strings, and empty nodes in
        // the consumed interval retain their value and span unchanged.
        let mut replacements = std::vec::Vec::with_capacity(last - first + 3);
        let prefix = &first_value[..first_offset];
        if !prefix.is_empty() {
            replacements.push(Node::Text(Text {
                value: prefix,
                span: first_span,
            }));
        }
        if !replacement.is_empty() {
            replacements.push(Node::Text(
                self.replacement_text(replacement, replacement_span),
            ));
        }
        if first < last {
            for node in &nodes[first + 1..last] {
                if let Node::Text(text) = node
                    && text.value.is_empty()
                {
                    replacements.push(Node::Text(Text {
                        value: text.value,
                        span: text.span,
                    }));
                }
            }
        }
        let suffix = &last_value[last_offset..];
        if !suffix.is_empty() {
            replacements.push(Node::Text(Text {
                value: suffix,
                span: last_span,
            }));
        }

        drop(nodes.splice(first..last + 1, replacements));

        Ok(replacement_span)
    }
}

/// A maximal sequence of adjacent `Text` nodes in one sibling list.
///
/// Runs do not cross formatting nodes. A pass that wants context across
/// emphasis or links must traverse those children in order and maintain its
/// own context; it should still use these runs for matching within each child
/// list. Non-text nodes are left to the caller's protected-content policy.
pub struct TextRun<'nodes, 'arena> {
    nodes: &'nodes [Node<'arena>],
    node_range: Range<usize>,
}

impl<'nodes, 'arena> TextRun<'nodes, 'arena> {
    /// The selected nodes' range within the sibling slice passed to [`text_runs`].
    #[must_use]
    pub fn node_range(&self) -> Range<usize> {
        self.node_range.clone()
    }

    /// Returns each text value with the source span of its original node.
    pub fn segments(&self) -> impl Iterator<Item = TextSegment<'arena>> + '_ {
        self.nodes.iter().filter_map(|node| {
            if let Node::Text(text) = node {
                Some(TextSegment {
                    value: text.value,
                    span: text.span,
                })
            } else {
                None
            }
        })
    }

    /// Finds protected URL ranges using the renderer options in `context`.
    ///
    /// Each original text segment is scanned independently, then its ranges
    /// are offset into the coalesced run. This mirrors HTML rendering, where
    /// autolinks do not cross AST text-node boundaries. If URL autolinking is
    /// disabled or no prefixes are configured, this returns no ranges.
    #[must_use]
    pub fn protected_url_ranges(&self, context: &TransformContext<'_>) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        let Some(matcher) = &context.autolink_matcher else {
            return ranges;
        };

        let mut offset = 0;
        for segment in self.segments() {
            ranges.extend(
                matcher
                    .find_ranges(segment.value)
                    .into_iter()
                    .map(|range| range.start + offset..range.end + offset),
            );
            offset += segment.value.len();
        }
        ranges
    }

    /// Returns the coalesced text, borrowing for a one-node run.
    #[must_use]
    pub fn value(&self) -> Cow<'arena, str> {
        if let [Node::Text(text)] = self.nodes {
            return Cow::Borrowed(text.value);
        }

        let capacity = self.segments().map(|segment| segment.value.len()).sum();
        let mut value = String::with_capacity(capacity);
        for segment in self.segments() {
            value.push_str(segment.value);
        }
        Cow::Owned(value)
    }

    /// Returns the bounding non-empty source span of all text in the run.
    #[must_use]
    pub fn source_span(&self) -> Span {
        span_for_byte_range(self.nodes, &(0..self.nodes.len()), &self.value_range())
    }

    /// Returns the bounding source span of text touched by a byte range.
    ///
    /// Insertions (empty ranges) map to [`Span::empty`].
    pub fn span_for_bytes(&self, range: Range<usize>) -> Result<Span, TextReplacementError> {
        let value = self.value();
        validate_byte_range(&value, &range)?;
        Ok(span_for_byte_range(
            self.nodes,
            &(0..self.nodes.len()),
            &range,
        ))
    }

    fn value_range(&self) -> Range<usize> {
        0..self.segments().map(|segment| segment.value.len()).sum()
    }
}

/// One decoded text segment and the source range of its AST node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSegment<'arena> {
    /// Decoded text value stored by Ferromark.
    pub value: &'arena str,
    /// Original source range for the complete AST text node.
    pub span: Span,
}

/// Iterator over maximal adjacent text runs in a node slice.
pub struct TextRuns<'nodes, 'arena> {
    nodes: &'nodes [Node<'arena>],
    cursor: usize,
}

impl<'nodes, 'arena> Iterator for TextRuns<'nodes, 'arena> {
    type Item = TextRun<'nodes, 'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.nodes.len() && !matches!(self.nodes[self.cursor], Node::Text(_)) {
            self.cursor += 1;
        }
        if self.cursor == self.nodes.len() {
            return None;
        }

        let start = self.cursor;
        while self.cursor < self.nodes.len() && matches!(self.nodes[self.cursor], Node::Text(_)) {
            self.cursor += 1;
        }

        Some(TextRun {
            nodes: &self.nodes[start..self.cursor],
            node_range: start..self.cursor,
        })
    }
}

/// Iterates over maximal adjacent `Text` runs in one sibling list.
#[must_use]
pub fn text_runs<'nodes, 'arena>(nodes: &'nodes [Node<'arena>]) -> TextRuns<'nodes, 'arena> {
    TextRuns { nodes, cursor: 0 }
}

/// Error returned when a pass replaces an invalid node or text range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextReplacementError {
    /// The requested range does not select a non-empty slice of nodes.
    InvalidNodeRange {
        /// Requested start index.
        start: usize,
        /// Requested exclusive end index.
        end: usize,
        /// Number of nodes in the sibling slice.
        node_count: usize,
    },
    /// The selected slice includes a node that is not text.
    NonTextNode {
        /// Index of the node relative to the sibling slice.
        index: usize,
    },
    /// The byte range starts after its end.
    ReversedByteRange {
        /// Requested start byte.
        start: usize,
        /// Requested exclusive end byte.
        end: usize,
    },
    /// The byte range extends beyond the coalesced text.
    ByteRangeOutOfBounds {
        /// Requested exclusive end byte.
        end: usize,
        /// Coalesced text length in bytes.
        text_len: usize,
    },
    /// A byte offset splits a UTF-8 code point.
    NotCharBoundary {
        /// Invalid byte offset.
        offset: usize,
    },
}

impl fmt::Display for TextReplacementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNodeRange {
                start,
                end,
                node_count,
            } => write!(
                formatter,
                "text node range {start}..{end} is invalid for {node_count} nodes"
            ),
            Self::NonTextNode { index } => {
                write!(formatter, "node {index} in the selected range is not text")
            }
            Self::ReversedByteRange { start, end } => {
                write!(formatter, "text byte range {start}..{end} is reversed")
            }
            Self::ByteRangeOutOfBounds { end, text_len } => write!(
                formatter,
                "text byte range ends at {end}, past the {text_len}-byte value"
            ),
            Self::NotCharBoundary { offset } => {
                write!(
                    formatter,
                    "text byte offset {offset} is not a UTF-8 boundary"
                )
            }
        }
    }
}

impl Error for TextReplacementError {}

fn validate_node_range(
    nodes: &[Node<'_>],
    range: &Range<usize>,
) -> Result<(), TextReplacementError> {
    if range.start >= range.end || range.end > nodes.len() {
        return Err(TextReplacementError::InvalidNodeRange {
            start: range.start,
            end: range.end,
            node_count: nodes.len(),
        });
    }
    for (index, node) in nodes.iter().enumerate().take(range.end).skip(range.start) {
        if !matches!(node, Node::Text(_)) {
            return Err(TextReplacementError::NonTextNode { index });
        }
    }
    Ok(())
}

fn validate_text_byte_range(
    nodes: &[Node<'_>],
    node_range: &Range<usize>,
    byte_range: &Range<usize>,
) -> Result<(), TextReplacementError> {
    if byte_range.start > byte_range.end {
        return Err(TextReplacementError::ReversedByteRange {
            start: byte_range.start,
            end: byte_range.end,
        });
    }

    let text_len = text_len_in_range(nodes, node_range);
    if byte_range.end > text_len {
        return Err(TextReplacementError::ByteRangeOutOfBounds {
            end: byte_range.end,
            text_len,
        });
    }

    for offset in [byte_range.start, byte_range.end] {
        if !is_text_char_boundary(nodes, node_range, offset) {
            return Err(TextReplacementError::NotCharBoundary { offset });
        }
    }
    Ok(())
}

fn text_len_in_range(nodes: &[Node<'_>], node_range: &Range<usize>) -> usize {
    nodes[node_range.clone()]
        .iter()
        .map(|node| match node {
            Node::Text(text) => text.value.len(),
            _ => 0,
        })
        .sum()
}

fn is_text_char_boundary(nodes: &[Node<'_>], node_range: &Range<usize>, offset: usize) -> bool {
    let mut cursor = 0;
    for node in &nodes[node_range.clone()] {
        let Node::Text(text) = node else {
            return false;
        };
        let end = cursor + text.value.len();
        if offset <= end {
            return text.value.is_char_boundary(offset - cursor);
        }
        cursor = end;
    }
    offset == cursor
}

fn overlapping_text_nodes(
    nodes: &[Node<'_>],
    node_range: &Range<usize>,
    byte_range: &Range<usize>,
) -> Result<(usize, usize, usize, usize), TextReplacementError> {
    let mut cursor = 0;
    let mut first = None;
    let mut last = None;
    let mut first_offset = 0;
    let mut last_offset = 0;

    for index in node_range.clone() {
        let Node::Text(text) = &nodes[index] else {
            continue;
        };
        let end = cursor + text.value.len();
        if cursor < byte_range.end && end > byte_range.start {
            if first.is_none() {
                first = Some(index);
                first_offset = byte_range.start - cursor;
            }
            last = Some(index);
            last_offset = byte_range.end.min(end) - cursor;
        }
        cursor = end;
    }

    let Some(first) = first else {
        return Err(TextReplacementError::ByteRangeOutOfBounds {
            end: byte_range.end,
            text_len: text_len_in_range(nodes, node_range),
        });
    };
    let Some(last) = last else {
        return Err(TextReplacementError::ByteRangeOutOfBounds {
            end: byte_range.end,
            text_len: text_len_in_range(nodes, node_range),
        });
    };
    Ok((first, last, first_offset, last_offset))
}

fn insert_at_text_offset<'arena>(
    context: &TransformContext<'arena>,
    nodes: &mut ferromark::allocator::Vec<'arena, Node<'arena>>,
    node_range: &Range<usize>,
    offset: usize,
    replacement: &str,
) {
    let text_len = text_len_in_range(nodes, node_range);
    if offset == 0 {
        let inserted = Node::Text(context.generated_text(replacement));
        drop(nodes.splice(
            node_range.start..node_range.start,
            std::iter::once(inserted),
        ));
        return;
    }
    if offset == text_len {
        let inserted = Node::Text(context.generated_text(replacement));
        drop(nodes.splice(node_range.end..node_range.end, std::iter::once(inserted)));
        return;
    }

    let mut cursor = 0;
    for index in node_range.clone() {
        let Node::Text(text) = &nodes[index] else {
            continue;
        };
        let end = cursor + text.value.len();
        if cursor < offset && offset < end {
            let (prefix, suffix, span) = (
                &text.value[..offset - cursor],
                &text.value[offset - cursor..],
                text.span,
            );
            let inserted = Node::Text(context.generated_text(replacement));
            let parts = [
                Some(Node::Text(Text {
                    value: prefix,
                    span,
                })),
                Some(inserted),
                Some(Node::Text(Text {
                    value: suffix,
                    span,
                })),
            ];
            drop(nodes.splice(index..index + 1, parts.into_iter().flatten()));
            return;
        }
        cursor = end;
        if cursor == offset {
            let inserted = Node::Text(context.generated_text(replacement));
            drop(nodes.splice(index + 1..index + 1, std::iter::once(inserted)));
            return;
        }
    }

    // Byte-range validation guarantees a boundary in the selected text run.
    // The run-end case is handled above, so this fallback inserts at its end.
    let inserted = Node::Text(context.generated_text(replacement));
    drop(nodes.splice(node_range.end..node_range.end, std::iter::once(inserted)));
}

fn validate_byte_range(value: &str, range: &Range<usize>) -> Result<(), TextReplacementError> {
    if range.start > range.end {
        return Err(TextReplacementError::ReversedByteRange {
            start: range.start,
            end: range.end,
        });
    }
    if range.end > value.len() {
        return Err(TextReplacementError::ByteRangeOutOfBounds {
            end: range.end,
            text_len: value.len(),
        });
    }
    for offset in [range.start, range.end] {
        if !value.is_char_boundary(offset) {
            return Err(TextReplacementError::NotCharBoundary { offset });
        }
    }
    Ok(())
}

fn span_for_byte_range(
    nodes: &[Node<'_>],
    node_range: &Range<usize>,
    byte_range: &Range<usize>,
) -> Span {
    if byte_range.is_empty() {
        return Span::empty();
    }

    let mut byte_cursor = 0;
    let mut span = None;
    for node in nodes.iter().take(node_range.end).skip(node_range.start) {
        if let Node::Text(text) = node {
            let segment_end = byte_cursor + text.value.len();
            if byte_cursor < byte_range.end
                && segment_end > byte_range.start
                && !text.span.is_empty()
            {
                span = Some(span.map_or(text.span, |current: Span| current.merge(text.span)));
            }
            byte_cursor = segment_end;
        }
    }
    span.unwrap_or_else(Span::empty)
}

/// Error from a pass, including its ordered identity in the pipeline.
#[derive(Debug)]
pub struct TransformError {
    pass_index: usize,
    pass_name: &'static str,
    source: BoxError,
}

impl TransformError {
    /// Returns this pass's zero-based position in the pipeline.
    #[must_use]
    pub const fn pass_index(&self) -> usize {
        self.pass_index
    }

    /// Returns the stable name of the failing pass.
    #[must_use]
    pub const fn pass_name(&self) -> &'static str {
        self.pass_name
    }
}

impl fmt::Display for TransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "transform pass {} ({:?}) failed: {}",
            self.pass_index, self.pass_name, self.source
        )
    }
}

impl Error for TransformError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}
