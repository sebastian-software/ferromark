//! Block-level event types.

use crate::Range;

/// GitHub-style callout/admonition type.
///
/// Future releases may add callout types. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
///
/// Future releases may add alignment modes. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
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

/// The source form of a code block.
///
/// Future releases may add code-block forms. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CodeBlockKind {
    /// A backtick- or tilde-fenced code block.
    Fenced {
        /// Optional CommonMark info string.
        info: Option<Range>,
    },
    /// A code block introduced by four columns of indentation.
    Indented,
}

/// Events emitted by the block parser.
///
/// Future releases may add events. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

    /// Start of a code block.
    CodeBlockStart {
        /// Whether the block is fenced or indented.
        kind: CodeBlockKind,
    },
    /// End of a code block.
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

    /// Start of a definition list.
    DefinitionListStart,
    /// End of a definition list.
    DefinitionListEnd,
    /// Start of a definition term.
    DefinitionTermStart,
    /// End of a definition term.
    DefinitionTermEnd,
    /// Start of a definition description.
    DefinitionDescriptionStart {
        /// Whether the first paragraph should omit its `<p>` wrapper.
        tight: bool,
    },
    /// End of a definition description.
    DefinitionDescriptionEnd,

    /// A thematic break (horizontal rule).
    ///
    /// The `Range` covers the marker run and any intervening or trailing
    /// horizontal whitespace on the same line. It excludes container
    /// indentation (leading spaces already consumed by the block loop) and
    /// the line ending character.
    ThematicBreak(Range),

    /// A source-only line comment omitted by the HTML renderer.
    Comment(Range),

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
    /// Start of relative column-width hints for a table.
    TableColumnWidthsStart,
    /// A numeric table column-width hint in hundredths of one percent.
    TableColumnWidth {
        /// Width in basis points (`2500` means `25%`).
        basis_points: u16,
    },
    /// End of relative column-width hints for a table.
    TableColumnWidthsEnd,
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
        /// Number of source columns covered by this cell.
        colspan: u16,
    },
    /// End of a table cell.
    TableCellEnd,
}

impl BlockEvent {
    /// Shift source ranges when an event stream is moved into a larger source.
    ///
    /// Block events are normally relative to the Markdown slice parsed by the
    /// block parser. MDX keeps footnote definitions until the assembled body
    /// has rendered, so those definition events need to refer to the original
    /// document instead.
    #[cfg(feature = "mdx")]
    pub(crate) fn shift_ranges(&mut self, offset: u32) {
        fn shift(range: &mut Range, offset: u32) {
            range.start = range
                .start
                .checked_add(offset)
                .expect("shifted source range exceeds the supported input size");
            range.end = range
                .end
                .checked_add(offset)
                .expect("shifted source range exceeds the supported input size");
        }

        match self {
            Self::ThematicBreak(range)
            | Self::Comment(range)
            | Self::HtmlBlockText(range)
            | Self::Text(range)
            | Self::Code(range) => shift(range, offset),
            Self::CodeBlockStart {
                kind: CodeBlockKind::Fenced { info: Some(range) },
            } => shift(range, offset),
            Self::CodeBlockStart { .. }
            | Self::ParagraphStart
            | Self::ParagraphEnd
            | Self::HeadingStart { .. }
            | Self::HeadingEnd { .. }
            | Self::CodeBlockEnd
            | Self::BlockQuoteStart { .. }
            | Self::BlockQuoteEnd
            | Self::ListStart { .. }
            | Self::ListEnd { .. }
            | Self::ListItemStart { .. }
            | Self::ListItemEnd
            | Self::DefinitionListStart
            | Self::DefinitionListEnd
            | Self::DefinitionTermStart
            | Self::DefinitionTermEnd
            | Self::DefinitionDescriptionStart { .. }
            | Self::DefinitionDescriptionEnd
            | Self::HtmlBlockStart
            | Self::HtmlBlockEnd
            | Self::SoftBreak
            | Self::VirtualSpaces(_)
            | Self::TableStart
            | Self::TableColumnWidthsStart
            | Self::TableColumnWidth { .. }
            | Self::TableColumnWidthsEnd
            | Self::TableEnd
            | Self::TableHeadStart
            | Self::TableHeadEnd
            | Self::TableBodyStart
            | Self::TableBodyEnd
            | Self::TableRowStart
            | Self::TableRowEnd
            | Self::TableCellStart { .. }
            | Self::TableCellEnd => {}
        }
    }
}

/// Type of list.
///
/// Future releases may add list forms. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
///
/// Future releases may add task states. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
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
