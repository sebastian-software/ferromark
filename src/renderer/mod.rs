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
mod render;

pub use html::{
    CodeAnnotationSyntax, HEADING_PERMALINK_CLASS, HtmlRenderContext, HtmlRenderControl,
    HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks, collect_heading_text,
    slugify_heading,
};
pub use render::{RenderError, RenderResult, Renderer};
