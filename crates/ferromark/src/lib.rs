//! Ferromark v2: an arena-allocated Markdown parser and HTML renderer.
//!
//! This local development baseline derives from OX-Content. Its API is not
//! compatible with Ferromark v1 and is not yet a stable v2 release.
//!
//! ```
//! use ferromark::{Allocator, HtmlRenderer, Parser};
//!
//! let source = "Hello, **world**!";
//! let allocator = Allocator::for_source_len(source.len());
//! let document = Parser::new(&allocator, source).parse()?;
//! let html = HtmlRenderer::new().render(&document);
//! assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>\n");
//! # Ok::<(), ferromark::ParseError>(())
//! ```

pub use ferromark_allocator::Allocator;
pub use ferromark_ast as ast;
pub use ferromark_parser::{ParseError, ParseErrorKind, ParseResult, Parser, ParserOptions, parse};
pub use ferromark_renderer::{
    CodeAnnotationSyntax, HEADING_PERMALINK_CLASS, HtmlRenderContext, HtmlRenderControl,
    HtmlRenderHooks, HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks, RenderError,
    RenderResult, Renderer, slugify_heading,
};
