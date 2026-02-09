//! Block-level event types.

use crate::Range;

/// GitHub-style callout/admonition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutType {
    /// Informational note.
    Note,
    /// Helpful tip.
    Tip,
    /// Important information.
    Important,
    /// Warning about potential issues.
    Warning,
    /// Critical caution about dangerous actions.
    Caution,
}

impl CalloutType {
    /// CSS class suffix (lowercase).
    pub fn css_suffix(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }

    /// Display title for the callout.
    pub fn title(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }
}

/// Column alignment for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    /// No alignment specified.
    #[default]
    None,
    /// Left-aligned (`:---`).
    Left,
    /// Center-aligned (`:---:`).
    Center,
    /// Right-aligned (`---:`).
    Right,
}

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

    /// Start of a blockquote (possibly a callout/admonition).
    BlockQuoteStart {
        /// Callout type, if this blockquote starts with `[!TYPE]`.
        callout: Option<CalloutType>,
    },
    /// End of a blockquote.
    BlockQuoteEnd,

    /// Start of a list.
    ListStart {
        /// List type (ordered or unordered).
        kind: ListKind,
        /// Whether the list is tight (no blank lines between items).
        tight: bool,
    },
    /// End of a list.
    ListEnd {
        /// List type (ordered or unordered).
        kind: ListKind,
        /// Whether the list is tight.
        tight: bool,
    },

    /// Start of a list item.
    ListItemStart {
        /// Task state for task list items.
        task: TaskState,
    },
    /// End of a list item.
    ListItemEnd,

    /// A thematic break (horizontal rule).
    ThematicBreak,

    /// Start of an HTML block.
    HtmlBlockStart,
    /// End of an HTML block.
    HtmlBlockEnd,
    /// Raw HTML block content (not to be inline-parsed or escaped).
    HtmlBlockText(Range),

    /// Soft line break (newline within paragraph).
    SoftBreak,

    /// Inline content range to be parsed by the inline parser.
    Text(Range),

    /// Raw code content (not to be inline-parsed).
    Code(Range),

    /// Virtual spaces to prepend to code content (from tab expansion).
    /// This event is followed by a Code or Text event.
    VirtualSpaces(u8),

    /// Start of a table.
    TableStart,
    /// End of a table.
    TableEnd,
    /// Start of a table header section.
    TableHeadStart,
    /// End of a table header section.
    TableHeadEnd,
    /// Start of a table body section.
    TableBodyStart,
    /// End of a table body section.
    TableBodyEnd,
    /// Start of a table row.
    TableRowStart,
    /// End of a table row.
    TableRowEnd,
    /// Start of a table cell with alignment.
    TableCellStart {
        /// Column alignment for this cell.
        alignment: Alignment,
    },
    /// End of a table cell.
    TableCellEnd,
}

/// Type of list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    /// Unordered list (bullet points).
    Unordered,
    /// Ordered list with starting number and delimiter.
    Ordered {
        /// Starting number.
        start: u32,
        /// Delimiter character ('.' or ')').
        delimiter: u8,
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
        let ol = ListKind::Ordered {
            start: 1,
            delimiter: b'.',
        };
        assert_ne!(ul, ol);
    }

    #[test]
    fn test_task_state_default() {
        assert_eq!(TaskState::default(), TaskState::None);
    }
}
