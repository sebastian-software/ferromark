//! Inline-level event types.

use crate::Range;

/// Events emitted by the inline parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineEvent {
    /// Plain text content.
    Text(Range),

    /// Inline code content (already resolved, no further parsing).
    Code(Range),

    /// Start of emphasis (`*em*` or `_em_`).
    EmphasisStart,
    /// End of emphasis.
    EmphasisEnd,

    /// Start of strong emphasis (`**strong**` or `__strong__`).
    StrongStart,
    /// End of strong emphasis.
    StrongEnd,

    /// Soft line break (newline in source).
    SoftBreak,

    /// Hard line break (two spaces + newline or backslash + newline).
    HardBreak,

    /// Backslash escape - the escaped character.
    EscapedChar(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_size() {
        // Events should be reasonably small
        assert!(std::mem::size_of::<InlineEvent>() <= 16);
    }
}
