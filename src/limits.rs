//! Enforced parser resource limits.
//!
//! Every constant in this module is consumed by a parser path and covered by
//! black-box tests in `tests/resource_limits_tests.rs`.

/// A bounded parser operation that fell back to literal or truncated output.
///
/// The limits keep adversarial input from causing excessive work or memory
/// growth. Use [`ResourceLimitReport`] to distinguish fully processed output
/// from the documented safe fallbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResourceLimit {
    /// Blockquote or list nesting exceeded [`MAX_BLOCK_NESTING`].
    BlockNesting,
    /// Inline delimiter collection exceeded [`MAX_INLINE_MARKS`].
    InlineMarks,
    /// Reference-link resolution exceeded [`MAX_REFERENCE_RESOLUTION_WORK`].
    ReferenceResolutionWork,
    /// A backtick run exceeded [`MAX_CODE_SPAN_BACKTICKS`].
    CodeSpanBackticks,
    /// Link destination nesting exceeded [`MAX_LINK_PAREN_DEPTH`].
    LinkDestinationParentheses,
    /// An ordered-list marker exceeded [`MAX_LIST_MARKER_DIGITS`].
    OrderedListMarkerDigits,
    /// A table row exceeded [`MAX_TABLE_COLUMNS`].
    TableColumns,
}

impl ResourceLimit {
    const fn bit(self) -> u8 {
        match self {
            Self::BlockNesting => 1 << 0,
            Self::InlineMarks => 1 << 1,
            Self::ReferenceResolutionWork => 1 << 2,
            Self::CodeSpanBackticks => 1 << 3,
            Self::LinkDestinationParentheses => 1 << 4,
            Self::OrderedListMarkerDigits => 1 << 5,
            Self::TableColumns => 1 << 6,
        }
    }
}

/// Set of resource limits exceeded while parsing one document.
///
/// A report is empty when the parser processed the document without taking a
/// resource-limit fallback. Repeated hits of the same limit are intentionally
/// coalesced; callers generally need to know which output guarantees changed,
/// not how often adversarial syntax repeated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceLimitReport(u8);

impl ResourceLimitReport {
    const ALL: [ResourceLimit; 7] = [
        ResourceLimit::BlockNesting,
        ResourceLimit::InlineMarks,
        ResourceLimit::ReferenceResolutionWork,
        ResourceLimit::CodeSpanBackticks,
        ResourceLimit::LinkDestinationParentheses,
        ResourceLimit::OrderedListMarkerDigits,
        ResourceLimit::TableColumns,
    ];

    /// Return whether no resource-limit fallback was used.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Return whether a specific resource limit was exceeded.
    #[must_use]
    pub const fn contains(self, limit: ResourceLimit) -> bool {
        self.0 & limit.bit() != 0
    }

    /// Iterate over the exceeded limits in stable declaration order.
    pub fn iter(self) -> impl Iterator<Item = ResourceLimit> {
        Self::ALL
            .into_iter()
            .filter(move |limit| self.contains(*limit))
    }

    #[inline]
    pub(crate) fn record(&mut self, limit: ResourceLimit) {
        self.0 |= limit.bit();
    }

    #[inline]
    pub(crate) fn extend(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

/// Maximum nesting depth for block containers (lists, blockquotes)
pub const MAX_BLOCK_NESTING: usize = 32;

/// Maximum number of marks collected during inline parsing
pub const MAX_INLINE_MARKS: usize = 4096;

/// Maximum reference-link resolution work per rendered document.
///
/// This counts bracket records and label bytes inspected while resolving
/// reference-style links. Once exhausted, remaining reference links are left
/// as literal text. The limit is shared by all paragraphs so independently
/// bounded pathological paragraphs cannot multiply expensive work.
pub const MAX_REFERENCE_RESOLUTION_WORK: usize = MAX_INLINE_MARKS * 8;

/// Maximum backtick run length for code spans (prevents O(n^2) matching)
/// Longer runs are treated as literal text
pub const MAX_CODE_SPAN_BACKTICKS: usize = 32;

/// Maximum parentheses nesting in link destinations (CommonMark spec: 32)
pub const MAX_LINK_PAREN_DEPTH: usize = 32;

/// Maximum digits in ordered list marker (prevents big-integer parsing)
pub const MAX_LIST_MARKER_DIGITS: usize = 9;

/// Maximum table columns
pub const MAX_TABLE_COLUMNS: usize = 128;
