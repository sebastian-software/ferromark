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
pub struct ParseError(Box<ParseErrorKind>);

/// The kinds of parse error.
///
/// The set of variants is deliberately open: a future release can report a
/// new error category additively, so callers must keep a wildcard arm.
#[derive(Debug, Error)]
#[non_exhaustive]
#[allow(clippy::disallowed_types)]
pub enum ParseErrorKind {
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
        Self(Box::new(kind))
    }

    /// Returns the kind of error.
    #[must_use]
    pub fn kind(&self) -> &ParseErrorKind {
        &self.0
    }

    /// Unwraps the kind of error.
    #[must_use]
    pub fn into_kind(self) -> ParseErrorKind {
        *self.0
    }

    /// Returns the span where the error occurred.
    #[must_use]
    pub fn span(&self) -> Span {
        self.0.span()
    }

    pub(in crate::parser) fn span_mut(&mut self) -> &mut Span {
        match &mut *self.0 {
            ParseErrorKind::NestingTooDeep { span, .. } => span,
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
        let span = self.span_mut();
        *span = map.map_span(*span);
        self
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
