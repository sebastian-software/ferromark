//! Markdown renderer for Ferromark.
//!
//! This crate provides a renderer that converts Markdown AST to HTML.
//!
//! # Example
//!
//! ```
//! use ferromark::allocator::Allocator;
//! use ferromark::parser::Parser;
//! use ferromark::renderer::HtmlRenderer;
//!
//! let allocator = Allocator::new();
//! let source = "# Hello World\n\nThis is a paragraph.";
//! let parser = ferromark::parser::Parser::new(&allocator, source);
//! let document = parser.parse().unwrap();
//!
//! let mut renderer = HtmlRenderer::new();
//! let html = renderer.render(&document);
//! ```

#![deny(clippy::disallowed_macros)]
#![cfg_attr(test, allow(clippy::disallowed_macros))]
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

mod html;
mod jsx;

#[cfg(feature = "ferriki")]
pub use html::FerrikiHighlightHooks;
#[cfg(feature = "ferriki")]
pub use jsx::FerrikiJsxHooks;

pub use html::{
    AbbreviationOptions, AutolinkMatcher, CodeAnnotationSyntax, CodeHighlightInput,
    HEADING_PERMALINK_CLASS, HeadingIdPlanner, HighlightedCodeBlock, HtmlRenderContext,
    HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions, InvalidHeadingIdPrefix,
    NoHtmlRenderHooks, collect_heading_text, find_autolink_ranges, map_heading_level,
    slugify_heading,
};

pub use jsx::{
    JsxCodeBlock, JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxModuleSource, JsxOutput,
    JsxRenderHooks, JsxRenderer, JsxRendererOptions, JsxSourceMapping, NoJsxRenderHooks,
};
