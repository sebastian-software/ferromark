//! Ferromark v2: an arena-allocated Markdown parser and HTML renderer.
//!
//! This implementation derives from OX-Content. Its API is not
//! compatible with Ferromark v1; see the migration guide in the repository.
//!
//! ```
//! let html = ferromark::to_html("Hello, **world**!")?;
//! assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>\n");
//! # Ok::<(), ferromark::ParseError>(())
//! ```
//!
//! Use [`to_html_with_options`] to choose syntax and output settings. Rust
//! defaults preserve raw HTML; enable [`HtmlRendererOptions::sanitize`] for
//! untrusted input. For AST access, use [`Allocator`], [`Parser`], and
//! [`HtmlRenderer`] directly.

#![warn(missing_docs)]

pub mod allocator;
pub mod ast;
pub mod outline;
pub mod parser;
pub mod renderer;

mod convenience;

pub use allocator::Allocator;
pub use convenience::{to_html, to_html_into, to_html_into_with_options, to_html_with_options};
pub use outline::{OutlineEntry, OutlineOptions};
pub use parser::{ParseError, ParseErrorKind, ParseResult, Parser, ParserOptions, parse};
pub use renderer::{
    AutolinkMatcher, CodeAnnotationSyntax, HEADING_PERMALINK_CLASS, HeadingIdPlanner,
    HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions,
    InvalidHeadingIdPrefix, NoHtmlRenderHooks, collect_heading_text, find_autolink_ranges,
    map_heading_level, slugify_heading,
};
