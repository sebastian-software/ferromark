//! HTML renderer implementation.
//!
//! The renderer is organized as a small public facade with focused internal modules:
//! options, escaping, autolinking, code annotations, heading helpers, and visitor
//! rendering. This keeps each implementation file near a reviewable size while
//! preserving the crate-level `HtmlRenderer` API.

mod autolink;
mod callout;
mod code_annotations;
mod escape;
mod heading;
mod html_attr;
mod mdx_payload;
mod options;
mod renderer;
mod tagfilter;

#[cfg(test)]
mod tests;

pub use heading::{
    HEADING_PERMALINK_CLASS, collect_heading_text, map_heading_level, slugify_heading,
};
pub use options::{CodeAnnotationSyntax, HtmlRendererOptions};
pub use renderer::{
    HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks, HtmlRenderer, NoHtmlRenderHooks,
};
