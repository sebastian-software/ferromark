//! Ferromark v2: an arena-allocated Markdown parser and HTML renderer.
//!
//! This implementation derives from OX-Content. Its API is not
//! compatible with Ferromark v1 and is not yet a stable v2 release.
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

pub mod allocator;
pub mod ast;
pub mod parser;
pub mod renderer;

mod convenience;

pub use allocator::Allocator;
pub use convenience::{to_html, to_html_into, to_html_into_with_options, to_html_with_options};
pub use parser::{ParseError, ParseErrorKind, ParseResult, Parser, ParserOptions, parse};
pub use renderer::{
    CodeAnnotationSyntax, HEADING_PERMALINK_CLASS, HtmlRenderContext, HtmlRenderControl,
    HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks, RenderError,
    RenderResult, Renderer, collect_heading_text, slugify_heading,
};
