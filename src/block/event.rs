//! Block-level event types.

use crate::Range;

/// Events emitted by the block parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockEvent {
    /// Start of a paragraph.
    ParagraphStart,
    /// End of a paragraph.
    ParagraphEnd,

    /// Start of a heading.
    HeadingStart {
        /// Heading level (1-6).
        level: u8,
    },
    /// End of a heading.
    HeadingEnd {
        /// Heading level (1-6).
        level: u8,
    },

    /// Start of a fenced code block.
    CodeBlockStart {
        /// Info string (language identifier).
        info: Option<Range>,
    },
    /// End of a fenced code block.
    CodeBlockEnd,

    /// Start of a blockquote.
    BlockQuoteStart,
    /// End of a blockquote.
    BlockQuoteEnd,

    /// Start of a list.
    ListStart {
        /// List type (ordered or unordered).
        kind: ListKind,
    },
    /// End of a list.
    ListEnd,

    /// Start of a list item.
    ListItemStart {
        /// Task state for task list items.
        task: TaskState,
    },
    /// End of a list item.
    ListItemEnd,

    /// A thematic break (horizontal rule).
    ThematicBreak,

    /// Inline content range to be parsed by the inline parser.
    Text(Range),

    /// Raw code content (not to be inline-parsed).
    Code(Range),
}

/// Type of list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    /// Unordered list (bullet points).
    Unordered,
    /// Ordered list with starting number.
    Ordered {
        /// Starting number.
        start: u32,
    },
}

/// Task list item state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskState {
    /// Not a task item.
    #[default]
    None,
    /// Unchecked task `[ ]`.
    Unchecked,
    /// Checked task `[x]` or `[X]`.
    Checked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_size() {
        // Events should be reasonably small
        assert!(std::mem::size_of::<BlockEvent>() <= 24);
    }

    #[test]
    fn test_list_kind() {
        let ul = ListKind::Unordered;
        let ol = ListKind::Ordered { start: 1 };
        assert_ne!(ul, ol);
    }

    #[test]
    fn test_task_state_default() {
        assert_eq!(TaskState::default(), TaskState::None);
    }
}
