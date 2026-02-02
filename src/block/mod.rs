//! Block-level parser for Markdown.
//!
//! The block parser is line-oriented and handles:
//! - Thematic breaks
//! - ATX headings
//! - Fenced code blocks
//! - Blockquotes
//! - Lists
//! - Paragraphs

mod event;
mod parser;

pub use event::{BlockEvent, ListKind, TaskState};
pub use parser::BlockParser;
