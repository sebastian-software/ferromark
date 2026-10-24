//! Error types for the parser.

use crate::ast::Span;
use thiserror::Error;

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse error.
///
/// The payload lives behind one pointer so that `ParseResult<T>` stays small
/// on the hot path: every inline construct returns a `ParseResult<()>`, which
/// is then a single register and a null test at each `?`, and every block
/// returns a `ParseResult<Option<Node>>` that no longer spills the unused
/// error payload. Errors are rare, so the allocation only happens when one
/// is actually reported.
#[derive(Debug, Error)]
#[error(transparent)]
#[allow(clippy::disallowed_types)]
pub struct ParseError(Box<ParseErrorData>);

#[derive(Debug, Error)]
#[error("{kind}")]
struct ParseErrorData {
    kind: ParseErrorKind,
    mdx: Option<MdxDiagnostic>,
}

/// Machine-readable detail for strict MDX failures. Messages remain on
/// [`ParseErrorKind::InvalidMdx`] and are not a stable classification API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MdxDiagnostic {
    /// Structural or grammar category.
    pub kind: MdxDiagnosticKind,
    /// Offending construct in original UTF-8 source coordinates.
    pub span: Span,
    /// Conflicting or unclosed opening tag, when known.
    pub related_span: Option<Span>,
}

/// Strict MDX failure categories. More categories may be added in the future.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MdxDiagnosticKind {
    /// Invalid or unclosed JavaScript child expression.
    InvalidExpression,
    /// Invalid or unclosed JSX opening tag or attributes.
    InvalidOpeningTag,
    /// An opening tag has no matching closer.
    UnclosedElement,
    /// A closing tag conflicts with the most recent opening tag.
    MismatchedClosingTag,
    /// A closing tag has no opening tag in this content.
    UnexpectedClosingTag,
    /// Malformed closing-tag syntax; recovery cannot skip it safely.
    InvalidClosingTag,
    /// Invalid JavaScript module block (including unsupported TypeScript).
    InvalidModule,
}

/// Failed diagnostic-oriented parse. No partial AST can be rendered.
#[derive(Debug, Error)]
#[error("parse failed with {} diagnostic(s)", errors.len())]
#[allow(clippy::disallowed_types)]
pub struct ParseDiagnostics {
    /// Errors in source order, limited to the requested maximum.
    pub errors: Vec<ParseError>,
    /// Parsing stopped at an ambiguous construct or the diagnostic limit.
    /// Later independent errors may therefore be absent.
    pub stopped_early: bool,
}

/// The kinds of parse error.
///
/// The set of variants is deliberately open: a future release can report a
/// new error category additively, so callers must keep a wildcard arm.
#[derive(Debug, Error)]
#[non_exhaustive]
#[allow(clippy::disallowed_types)]
pub enum ParseErrorKind {
    /// Invalid MDX syntax in JSX-compatible parsing.
    #[error("invalid MDX at {span:?}: {message}")]
    InvalidMdx {
        /// Source range containing the invalid construct.
        span: Span,
        /// JavaScript or JSX syntax diagnostic.
        message: String,
    },
    /// Nesting too deep.
    #[error("nesting too deep at {span:?}: maximum depth is {max_depth}")]
    NestingTooDeep {
        /// The span where the error occurred.
        span: Span,
        /// Maximum allowed depth.
        max_depth: usize,
    },
}

impl ParseError {
    /// Wraps an error kind.
    #[must_use]
    #[cold]
    #[allow(clippy::disallowed_types)]
    pub fn new(kind: ParseErrorKind) -> Self {
        Self(Box::new(ParseErrorData { kind, mdx: None }))
    }

    /// Returns the kind of error.
    #[must_use]
    pub fn kind(&self) -> &ParseErrorKind {
        &self.0.kind
    }

    /// Unwraps the kind of error.
    #[must_use]
    pub fn into_kind(self) -> ParseErrorKind {
        self.0.kind
    }

    /// Returns the span where the error occurred.
    #[must_use]
    pub fn span(&self) -> Span {
        self.0.kind.span()
    }

    /// Typed MDX detail, available for syntax failures in strict MDX mode.
    /// Configuration and nesting-limit errors have no MDX detail.
    pub fn mdx_diagnostic(&self) -> Option<&MdxDiagnostic> {
        self.0.mdx.as_ref()
    }

    #[cfg(feature = "jsx")]
    pub(in crate::parser) fn with_mdx_diagnostic(mut self, detail: MdxDiagnostic) -> Self {
        self.0.mdx = Some(detail);
        self
    }

    pub(in crate::parser) fn span_mut(&mut self) -> &mut Span {
        match &mut self.0.kind {
            ParseErrorKind::NestingTooDeep { span, .. } => span,
            ParseErrorKind::InvalidMdx { span, .. } => span,
        }
    }

    /// Translates the span through `map`.
    ///
    /// Container sub-parsers (block quotes, list items, footnote and
    /// definition bodies, JSX children) parse a stripped copy of their
    /// content, so an error raised inside one carries offsets into that copy.
    /// Each container maps its children's spans back to its own source, and
    /// the error takes the same map on its way out, so the offset a caller
    /// reads points into the document it handed to the root parser.
    #[must_use]
    pub(in crate::parser) fn remapped(mut self, map: &impl super::spans::SpanMap) -> Self {
        self.remap_in_place(map);
        self
    }

    pub(in crate::parser) fn remap_in_place(&mut self, map: &impl super::spans::SpanMap) {
        if let Some(detail) = &mut self.0.mdx {
            detail.span = map.map_inline_span(detail.span);
            detail.related_span = detail.related_span.map(|span| map.map_inline_span(span));
            let span = detail.span;
            *self.span_mut() = span;
        } else {
            let span = self.span_mut();
            *span = map.map_span(*span);
        }
    }
}

impl From<ParseErrorKind> for ParseError {
    #[cold]
    fn from(kind: ParseErrorKind) -> Self {
        Self::new(kind)
    }
}

impl ParseErrorKind {
    /// Returns the span where the error occurred.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::NestingTooDeep { span, .. } => *span,
            Self::InvalidMdx { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::*;

    #[test]
    fn results_stay_small() {
        assert_eq!(
            std::mem::size_of::<ParseError>(),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<ParseResult<()>>(),
            std::mem::size_of::<usize>()
        );
        assert!(std::mem::size_of::<ParseResult<Option<crate::ast::Node<'_>>>>() <= 40);
    }

    #[test]
    fn kind_and_span_round_trip() {
        let span = Span::new(3, 7);
        let error = ParseError::from(ParseErrorKind::NestingTooDeep { span, max_depth: 2 });
        assert_eq!(error.span(), span);
        assert!(matches!(
            error.kind(),
            ParseErrorKind::NestingTooDeep { max_depth: 2, .. }
        ));
        assert_eq!(
            error.to_string(),
            "nesting too deep at Span { start: 3, end: 7 }: maximum depth is 2"
        );
        assert!(matches!(
            error.into_kind(),
            ParseErrorKind::NestingTooDeep { .. }
        ));
    }
}
