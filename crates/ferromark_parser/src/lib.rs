//! High-performance Markdown parser for Ferromark.
//!
//! This crate provides a fast, arena-allocated Markdown parser following
//! the CommonMark specification with GFM extensions.
//!
//! # Features
//!
//! - Arena-based allocation for zero-copy parsing
//! - CommonMark compliant with GFM extensions
//! - Optional MDX, math, footnotes, and other Markdown syntax extensions
//!
//! # Example
//!
//! ```
//! use ferromark_allocator::Allocator;
//! use ferromark_parser::Parser;
//!
//! let allocator = Allocator::new();
//! let source = "# Hello World\n\nThis is a paragraph.";
//! let parser = Parser::new(&allocator, source);
//! let document = parser.parse();
//! ```

#![deny(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod error;
mod parser;

pub use error::{ParseError, ParseErrorKind, ParseResult};
pub use parser::{Parser, ParserOptions};

/// Parses Markdown source into an AST.
///
/// This is a convenience function that creates a parser with default options.
pub fn parse<'a>(
    allocator: &'a ferromark_allocator::Allocator,
    source: &'a str,
) -> ParseResult<ferromark_ast::Document<'a>> {
    Parser::new(allocator, source).parse()
}
