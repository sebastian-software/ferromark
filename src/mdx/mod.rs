//! MDX segmenter: splits MDX input into typed segments.
//!
//! MDX combines Markdown with JSX/JavaScript. Instead of parsing the full MDX
//! syntax, this module splits the input into typed blocks. Only the Markdown
//! segments need to go through ferromark's Markdown parser; JSX, expressions,
//! and ESM statements are passed through unchanged.
//!
//! This module is gated behind the `mdx` Cargo feature.
//!
//! # Example
//! ```
//! use ferromark::mdx::{segment, Segment};
//!
//! let input = "import A from 'a'\n\n# Hello\n\n<Card>\nWorld\n</Card>\n";
//! let segments = segment(input);
//! for seg in &segments {
//!     match seg {
//!         Segment::Markdown(md) => {
//!             // Parse with ferromark::to_html(md)
//!         }
//!         _ => {
//!             // Pass through unchanged
//!         }
//!     }
//! }
//! ```
//!
//! # Differences from the official mdxjs compiler
//!
//! This segmenter covers the block-level MDX patterns used in real-world
//! documentation (Next.js, Docusaurus, Astro). It intentionally does **not**
//! replicate the full `@mdx-js/mdx` compiler. The differences:
//!
//! ## Block-level only
//!
//! The segmenter detects JSX and expressions at block level (start of a line).
//! Inline JSX (`paragraph with <em>JSX</em> inside`) and inline expressions
//! (`text {variable} here`) stay inside Markdown segments and are **not**
//! split out. The official mdxjs compiler handles both flow and text positions.
//!
//! ## No JavaScript validation
//!
//! Official mdxjs pipes ESM and expressions through acorn (or swc) to validate
//! the JavaScript syntax. We use heuristics: `import`/`export` at column 0,
//! brace-depth counting for expressions. This means:
//! - We won't reject syntactically invalid JS (e.g. `export const = ;`)
//! - Multi-line ESM uses blank-line termination, not parser-driven boundaries
//! - Exotic edge cases (e.g. `export var a = 1\nvar b`) may be grouped differently
//!
//! ## No Markdown syntax modifications
//!
//! Official mdxjs alters the Markdown grammar:
//! - **Indented code blocks disabled** — 4-space indented lines are paragraphs
//! - **HTML (flow + inline) disabled** — `<div>` is always JSX, never raw HTML
//! - **Autolinks disabled** — `<https://...>` is JSX, not an autolink
//!
//! We leave the Markdown parser untouched. Markdown segments are parsed with
//! standard CommonMark/GFM rules. This is a deliberate trade-off: it keeps
//! ferromark's core parser unmodified and lets the caller decide how to handle
//! HTML-like syntax inside Markdown segments.
//!
//! ## No container awareness
//!
//! JSX/ESM inside block containers is not detected:
//! ```text
//! > <Component>   ← treated as blockquote + markdown, not JSX
//! - import x      ← treated as list item, not ESM
//! ```
//!
//! The official compiler tracks container context (blockquote markers, list
//! indentation) and can detect JSX/ESM inside them.
//!
//! ## No TypeScript generics in JSX
//!
//! `<Component<T>>` with TypeScript generics is not supported by the tag
//! parser. The official compiler (when configured with acorn-jsx + TypeScript)
//! handles this.
//!
//! ## Silent fallback instead of errors
//!
//! Invalid JSX or unterminated expressions are silently treated as Markdown.
//! The official compiler reports parse errors with source positions.

pub mod expr;
pub mod jsx_tag;
pub mod render;
mod splitter;

/// A typed segment of an MDX document.
///
/// All variants are zero-copy `&str` slices into the original input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment<'a> {
    /// ESM statement (`import` / `export`) — pass through unchanged.
    Esm(&'a str),
    /// Markdown content — parse with ferromark's Markdown parser.
    Markdown(&'a str),
    /// JSX block opening tag (e.g. `<Component prop="x">`).
    JsxBlockOpen(&'a str),
    /// JSX block closing tag (e.g. `</Component>`).
    JsxBlockClose(&'a str),
    /// JSX self-closing block tag (e.g. `<Component />`).
    JsxBlockSelfClose(&'a str),
    /// JavaScript expression (e.g. `{expression}`).
    Expression(&'a str),
}

/// Segment an MDX document into typed blocks.
///
/// This is the primary entry point. The returned segments cover the entire
/// input — no bytes are dropped.
pub fn segment(input: &str) -> Vec<Segment<'_>> {
    splitter::split(input)
}

pub use render::{MdxOutput, render, render_with_options};
