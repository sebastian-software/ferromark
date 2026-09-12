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

pub use event::{Alignment, BlockEvent, CalloutType, CodeBlockKind, ListKind, TaskState};
pub use parser::BlockParser;
pub(crate) use parser::BlockScratch;

/// Post-process events to fix up list tight status.
///
/// Since we emit `ListStart` before knowing if the list is tight,
/// we need to patch the tight value using information from `ListEnd`.
pub fn fixup_list_tight(events: &mut [BlockEvent]) {
    let mut list_starts: smallvec::SmallVec<[usize; 8]> = smallvec::SmallVec::new();
    for i in 0..events.len() {
        match events[i] {
            BlockEvent::ListStart { .. } => list_starts.push(i),
            BlockEvent::ListEnd { tight, .. } => {
                if let Some(start) = list_starts.pop()
                    && let BlockEvent::ListStart {
                        tight: start_tight, ..
                    } = &mut events[start]
                {
                    *start_tight = tight;
                }
            }
            _ => {}
        }
    }
}
