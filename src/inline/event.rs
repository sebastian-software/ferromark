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

    /// Start of strikethrough (`~~text~~` or `~text~`).
    StrikethroughStart,
    /// End of strikethrough.
    StrikethroughEnd,

    /// Start of a link `[text](url)`.
    LinkStart {
        /// URL destination.
        url: Range,
        /// Optional title.
        title: Option<Range>,
    },
    /// Start of a reference-style link `[text][label]`.
    LinkStartRef {
        /// Index into the link reference store.
        def_index: u32,
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
    /// Start of a reference-style image `![alt][label]`.
    ImageStartRef {
        /// Index into the link reference store.
        def_index: u32,
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

    /// Autolink literal (bare URL, www link, or email without angle brackets).
    AutolinkLiteral {
        /// The matched text range.
        url: Range,
        /// Kind of autolink literal.
        kind: crate::inline::links::AutolinkLiteralKind,
    },

    /// Raw inline HTML (not escaped or parsed).
    Html(Range),

    /// Soft line break (newline in source).
    SoftBreak,

    /// Hard line break (two spaces + newline or backslash + newline).
    HardBreak,

    /// Backslash escape - the escaped character.
    EscapedChar(u8),

    /// Footnote reference `[^label]`.
    FootnoteRef {
        /// Index into the footnote store.
        def_index: u32,
    },

    /// Inline math span (`$...$`).
    MathInline(Range),

    /// Display math span (`$$...$$`).
    MathDisplay(Range),
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
