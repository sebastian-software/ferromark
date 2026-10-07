//! Ferromark v2: an arena-allocated Markdown parser with HTML and JSX renderers.
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
//! [`HtmlRenderer`] or [`JsxRenderer`] directly. JSX output is source code for
//! a downstream compiler; Ferromark does not compile it or add a framework
//! runtime.

#![warn(missing_docs)]

pub mod allocator;
pub mod ast;
pub mod outline;
pub mod parser;
pub mod renderer;

/// Ferriki's Rust API, available with the `ferriki` feature.
#[cfg(feature = "ferriki")]
pub use ferriki;

mod callout;
mod convenience;

pub use allocator::Allocator;
pub use convenience::{
    to_html, to_html_into, to_html_into_with_options, to_html_with_options,
    to_html_with_options_and_abbreviations,
};
pub use outline::{OutlineEntry, OutlineOptions};
pub use parser::{ParseError, ParseErrorKind, ParseResult, Parser, ParserOptions, parse};
pub use renderer::{
    AbbreviationOptions, AutolinkMatcher, CodeAnnotationSyntax, CodeHighlightInput,
    HEADING_PERMALINK_CLASS, HeadingIdPlanner, HighlightedCodeBlock, HtmlRenderContext,
    HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions, InvalidHeadingIdPrefix,
    JsxCodeBlock, JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxModuleSource, JsxOutput,
    JsxRenderHooks, JsxRenderer, JsxRendererOptions, JsxSourceMapping, NoHtmlRenderHooks,
    NoJsxRenderHooks, collect_heading_text, find_autolink_ranges, map_heading_level,
    slugify_heading,
};

#[cfg(feature = "jsx")]
pub use renderer::{JsxModuleError, JsxModuleOptions, JsxModuleOutput, JsxSourceMap};

#[cfg(feature = "ferriki")]
pub use renderer::FerrikiHighlightHooks;

#[cfg(feature = "ferriki")]
pub use renderer::FerrikiJsxHooks;
