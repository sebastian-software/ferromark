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

pub use event::{Alignment, BlockEvent, CalloutType, ListKind, TaskState};
pub use parser::BlockParser;

/// Post-process events to fix up list tight status.
///
/// Since we emit `ListStart` before knowing if the list is tight,
/// we need to patch the tight value using information from `ListEnd`.
pub fn fixup_list_tight(events: &mut [BlockEvent]) {
    // First pass: collect (start_idx, tight) pairs
    let mut list_starts: Vec<usize> = Vec::new();
    let mut patches: Vec<(usize, bool)> = Vec::new();

    for i in 0..events.len() {
        match &events[i] {
            BlockEvent::ListStart { .. } => {
                list_starts.push(i);
            }
            BlockEvent::ListEnd { tight, .. } => {
                if let Some(start_idx) = list_starts.pop() {
                    patches.push((start_idx, *tight));
                }
            }
            _ => {}
        }
    }

    // Second pass: apply patches
    for (start_idx, tight) in patches {
        if let BlockEvent::ListStart { kind, .. } = &events[start_idx] {
            let kind = *kind;
            events[start_idx] = BlockEvent::ListStart { kind, tight };
        }
    }
}
