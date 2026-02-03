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

    /// Start of a link `[text](url)`.
    LinkStart {
        /// URL destination.
        url: Range,
        /// Optional title.
        title: Option<Range>,
    },
    /// End of a link.
    LinkEnd,

    /// Start of an image `![alt](url)`.
    ImageStart {
        /// URL destination.
        url: Range,
        /// Optional title.
        title: Option<Range>,
    },
    /// End of an image (after alt text).
    ImageEnd,

    /// Autolink `<url>` or `<email>`.
    Autolink {
        /// The URL or email.
        url: Range,
        /// Whether this is an email autolink.
        is_email: bool,
    },

    /// Raw inline HTML (not escaped or parsed).
    Html(Range),

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
        // Events should be reasonably small (Link/Image have url + Option<Range>)
        assert!(std::mem::size_of::<InlineEvent>() <= 32);
    }
}
